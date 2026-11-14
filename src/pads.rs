//! Pads (fundo contínuo por tom): sons prontos em loop, um por tom (C..B), em bancos (Escuro, Coral).
//! Os sons padrão vão dentro do programa (OGG); cada botão também aceita um arquivo do Theo (WAV/MP3/OGG/FLAC).

use std::io::Cursor;
use std::sync::Arc;

use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::DecoderOptions;
use symphonia::core::errors::Error as SymError;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::{MediaSource, MediaSourceStream};
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

pub const PAD_KEYS: usize = 12;
pub const PAD_BANKS: usize = 3;

pub const KEY_NAMES: [&str; PAD_KEYS] = [
    "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
];
pub const KEY_NAMES_PT: [&str; PAD_KEYS] = [
    "Dó", "Dó#", "Ré", "Ré#", "Mi", "Fá", "Fá#", "Sol", "Sol#", "Lá", "Lá#", "Si",
];
pub const BANK_NAMES: [&str; PAD_BANKS] = ["Escuro", "Coral", "Suave"];
pub const BANK_HINTS: [&str; PAD_BANKS] = [
    "grave, fechado, místico",
    "vozes graves e médias",
    "tranquilo, redondo, para a igreja",
];

/// Um pad decodificado: estéreo intercalado (L, R, L, R...).
pub struct PadClip {
    pub data: Vec<i16>,
    pub rate: u32,
    /// v0.35: pad gravado nos 12 tons (Dream Shimmer, Reverse Waves): uma gravação por tom, sem mudar a altura.
    pub per_key: Vec<Arc<PadClip>>,
}

impl PadClip {
    pub fn frames(&self) -> usize {
        self.data.len() / 2
    }
}

/// v0.35: o que tocar para o tom `key`: a gravação daquele tom (pad de 12 tons) ou o pad mudando a altura.
pub fn clip_for_key(clip: &Arc<PadClip>, root: usize, key: usize) -> (Arc<PadClip>, f32) {
    match clip.per_key.get(key % PAD_KEYS) {
        Some(k) if clip.per_key.len() == PAD_KEYS => (k.clone(), 1.0),
        _ => (clip.clone(), my_pad_pitch(root, key)),
    }
}

/// v0.35: pads da Biblioteca que entram nos "Meus pads" com nome de astro (só se o arquivo foi baixado).
/// (nome, caminho). "tons:X" = pasta com os 12 tons; "sf2:arquivo#banco:programa" = o programa grava o pad do SF2.
pub const LIB_PADS: [(&str, &str); 6] = [
    ("Sol", "tons:Dream Shimmer"),
    ("Lua", "tons:Reverse Waves"),
    ("Mercúrio", "sf2:Pads/Hillsong Pad.sf2#0:0"),
    ("Urano", "sf2:Pads/Warm Pad.sf2#0:0"),
    ("Plutão", "sf2:Pads/Shimmer Dark.sf2#0:0"),
    ("Cometa", "sf2:Pads/Oceans Pad.sf2#0:0"),
];

/// O arquivo de um pad da Biblioteca está no computador?
pub fn lib_pad_available(path: &str) -> bool {
    if let Some(set) = path.strip_prefix("tons:") {
        return (0..PAD_KEYS).all(|k| crate::biblioteca::key_file(set, k).exists());
    }
    if let Some((rel, _)) = path.strip_prefix("sf2:").and_then(|s| s.split_once('#')) {
        return crate::biblioteca::full_path(rel).exists();
    }
    false
}

/// Carrega os 12 tons de um pad gravado em todos os tons.
fn load_key_set(set: &str) -> Result<PadClip, String> {
    let mut keys = Vec::with_capacity(PAD_KEYS);
    for k in 0..PAD_KEYS {
        let path = crate::biblioteca::key_file(set, k);
        let file = std::fs::File::open(&path).map_err(|_| format!("falta o tom {} (baixe a Biblioteca)", KEY_NAMES[k]))?;
        // WAV com cabeçalho fora do padrão (Reverse Waves: fmt de 20 bytes) → leitor simples daqui
        let clip = match decode_part(Box::new(file), Some("wav"), 0, MY_KEEP_S) {
            Ok(c) => c,
            Err(e) => read_plain_wav(&path, MY_KEEP_S).ok_or(e)?,
        };
        let mut data = clip.data;
        smooth_loop(&mut data, clip.rate as usize * 2);
        normalize_level(&mut data);
        keys.push(Arc::new(PadClip { data, rate: clip.rate, per_key: Vec::new() }));
    }
    // o "clipe principal" é o de C (serve para descobrir o tom e para o ✕/renomear da tela)
    Ok(PadClip { data: keys[0].data.clone(), rate: keys[0].rate, per_key: keys })
}

/// v0.35: grava um pad a partir de um timbre de SF2 da Biblioteca: tônica e quinta em C (serve para
/// maior e menor, igual aos outros pads), segurando as notas. Pula o começo (o pad "entrando").
pub fn render_sf2_pad(rel: &str, bank: i32, patch: i32) -> Result<PadClip, String> {
    use rustysynth::{SoundFont, Synthesizer, SynthesizerSettings};
    const RATE: u32 = 44_100;
    const SKIP_S: usize = 5;
    const KEEP_S: usize = 40;
    let path = crate::biblioteca::full_path(rel);
    let f = std::fs::File::open(&path).map_err(|_| "baixe a Biblioteca primeiro".to_string())?;
    let sf = SoundFont::new(&mut std::io::BufReader::with_capacity(1 << 20, f)).map_err(|e| format!("SF2 com problema ({e})"))?;
    let sf = Arc::new(sf);
    let mut settings = SynthesizerSettings::new(RATE as i32);
    settings.enable_reverb_and_chorus = true;
    let mut s = Synthesizer::new(&sf, &settings).map_err(|e| format!("SF2 com problema ({e})"))?;
    s.set_channel_preset(0, bank, patch);
    s.process_midi_message(0, 0xB0, 0x5B, 30); // um pouco de reverb do próprio SF2
    for n in [36, 48, 55, 60, 67] {
        s.note_on(0, n, if n == 36 { 70 } else { 92 });
    }
    let total = RATE as usize * (SKIP_S + KEEP_S);
    let (mut l, mut r) = (vec![0f32; 1024], vec![0f32; 1024]);
    let mut data: Vec<i16> = Vec::with_capacity(RATE as usize * KEEP_S * 2);
    let mut done = 0;
    while done < total {
        let n = (total - done).min(1024);
        s.render(&mut l[..n], &mut r[..n]);
        if done + n > RATE as usize * SKIP_S {
            let from = (RATE as usize * SKIP_S).saturating_sub(done);
            for i in from..n {
                data.push((l[i].clamp(-1.0, 1.0) * 32767.0) as i16);
                data.push((r[i].clamp(-1.0, 1.0) * 32767.0) as i16);
            }
        }
        done += n;
    }
    if data.iter().all(|&x| x.unsigned_abs() < 30) {
        return Err("esse timbre não segura a nota (não serve de pad)".into());
    }
    Ok(PadClip { data, rate: RATE, per_key: Vec::new() })
}

macro_rules! bank {
    ($dir:literal) => {
        [
            include_bytes!(concat!("../assets/pads/", $dir, "/00.ogg")).as_slice(),
            include_bytes!(concat!("../assets/pads/", $dir, "/01.ogg")).as_slice(),
            include_bytes!(concat!("../assets/pads/", $dir, "/02.ogg")).as_slice(),
            include_bytes!(concat!("../assets/pads/", $dir, "/03.ogg")).as_slice(),
            include_bytes!(concat!("../assets/pads/", $dir, "/04.ogg")).as_slice(),
            include_bytes!(concat!("../assets/pads/", $dir, "/05.ogg")).as_slice(),
            include_bytes!(concat!("../assets/pads/", $dir, "/06.ogg")).as_slice(),
            include_bytes!(concat!("../assets/pads/", $dir, "/07.ogg")).as_slice(),
            include_bytes!(concat!("../assets/pads/", $dir, "/08.ogg")).as_slice(),
            include_bytes!(concat!("../assets/pads/", $dir, "/09.ogg")).as_slice(),
            include_bytes!(concat!("../assets/pads/", $dir, "/10.ogg")).as_slice(),
            include_bytes!(concat!("../assets/pads/", $dir, "/11.ogg")).as_slice(),
        ]
    };
}

static BUILTIN: [[&[u8]; PAD_KEYS]; PAD_BANKS] = [bank!("escuro"), bank!("coral"), bank!("suave")];

/// "Meus pads": arquivos do Theo (ex.: pad do YouTube em G) tocados nos 12 tons, mudando a altura.
/// Bancos 6..9 (até 4, cada um com nome).
pub const MY_BANK: usize = PAD_BANKS + 3;
pub const MY_SLOTS: usize = 12; // v0.30: 8 → 12 (pads novos da Pixabay)

/// Estilos ao todo: 3 de loop (Escuro, Coral, Suave) + 3 "Vivos" (motor granular) + 4 "Meus pads".
pub const ALL_BANKS: usize = MY_BANK + MY_SLOTS;

/// Banco de um "Meu pad"? Devolve qual (0..4).
pub fn my_slot(bank: usize) -> Option<usize> {
    (MY_BANK..ALL_BANKS).contains(&bank).then(|| bank - MY_BANK)
}

/// Nome inicial a partir do arquivo: "Pad + Reverse em G - Em _ 30 Minutos _ [Bx5PwBH1oGA].mp3" → "Pad + Reverse em G - Em".
pub fn name_from_file(path: &str) -> String {
    let stem = std::path::Path::new(path)
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();
    // id do YouTube no fim sem colchetes ("..._YLDh18SZQv0")
    let mut stem = stem;
    if let Some(i) = stem
        .len()
        .checked_sub(12)
        .filter(|i| stem.is_char_boundary(*i))
    {
        let (head, id) = stem.split_at(i);
        let id = &id[1..];
        if (stem.as_bytes()[i] == b'_' || stem.as_bytes()[i] == b' ')
            && id
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
            && id.chars().any(|c| c.is_ascii_uppercase())
            && id.chars().any(|c| c.is_ascii_digit())
        {
            stem = head.to_string();
        }
    }
    let mut s = stem.replace('_', " ");
    // tira [id do YouTube] e (coisas entre parênteses)
    for (a, b) in [('[', ']'), ('(', ')')] {
        while let (Some(i), Some(j)) = (s.find(a), s.find(b)) {
            if j < i {
                break;
            }
            s.replace_range(i..=j, " ");
        }
    }
    // corta no "|" ou em "30 Minutos" e afins
    let lower = s.to_lowercase();
    let mut end = s.len();
    for cut in ["|", " minutos", " min ", " hora"] {
        if let Some(i) = lower.find(cut) {
            end = end.min(i);
        }
    }
    let s = s[..end].to_string();
    let mut words: Vec<&str> = s.split_whitespace().collect();
    // número solto no fim ("... 30") sai também
    while words.len() > 1
        && words
            .last()
            .is_some_and(|w| w.chars().all(|c| c.is_ascii_digit()))
    {
        words.pop();
    }
    let mut s = words.join(" ");
    s = s
        .trim_matches(|c: char| c == '-' || c == '+' || c == ' ')
        .to_string();
    // tira o código do começo que alguns programas colocam ("9b497964-Pad ...")
    if let Some((head, rest)) = s.split_once('-') {
        if head.len() == 8 && head.chars().all(|c| c.is_ascii_hexdigit()) {
            s = rest.trim().to_string();
        }
    }
    let s: String = s.chars().take(28).collect();
    if s.trim().is_empty() {
        "Meu pad".into()
    } else {
        s.trim().to_string()
    }
}

/// Tom escrito no nome do arquivo ("... em G - Em", "Pad Ab"), se tiver.
pub fn key_from_name(name: &str) -> Option<usize> {
    let words: Vec<String> = name
        .replace(['_', '-', '|', '(', ')', '[', ']'], " ")
        .split_whitespace()
        .map(str::to_string)
        .collect();
    let parse = |w: &str| -> Option<usize> {
        let w = w.trim_end_matches('m');
        let mut c = w.chars();
        let base = match c.next()? {
            'C' => 0,
            'D' => 2,
            'E' => 4,
            'F' => 5,
            'G' => 7,
            'A' => 9,
            'B' => 11,
            _ => return None,
        };
        let acc = match c.as_str() {
            "" => 0,
            "#" => 1,
            "b" => -1,
            _ => return None,
        };
        Some((base + acc + 12) as usize % 12)
    };
    // o primeiro tom depois de "em"/"in"; senão, o primeiro tom solto
    words
        .windows(2)
        .find(|w| {
            matches!(w[0].to_lowercase().as_str(), "em" | "in" | "tom") && parse(&w[1]).is_some()
        })
        .and_then(|w| parse(&w[1]))
        .or_else(|| words.iter().find_map(|w| parse(w)))
}

/// Estilo "Vivo" (granular)? Bancos 3, 4 e 5.
pub fn is_vivo(bank: usize) -> bool {
    (PAD_BANKS..MY_BANK).contains(&bank)
}

pub fn bank_name(bank: usize) -> &'static str {
    let b = bank % ALL_BANKS;
    if my_slot(b).is_some() {
        "Meu pad"
    } else if is_vivo(b) {
        VIVO_NAMES[b - PAD_BANKS]
    } else {
        BANK_NAMES[b]
    }
}

pub fn bank_hint(bank: usize) -> &'static str {
    let b = bank % ALL_BANKS;
    if my_slot(b).is_some() {
        "seu arquivo em todos os tons"
    } else if is_vivo(b) {
        VIVO_HINTS[b - PAD_BANKS]
    } else {
        BANK_HINTS[b]
    }
}

/// Quanto acelerar o "Meu pad" (gravado em `root`) para soar no tom `key`: no máximo 6 semitons
/// para cima ou para baixo, para não mudar demais o jeito do som.
pub fn my_pad_pitch(root: usize, key: usize) -> f32 {
    let mut s = (key as i32 - root as i32).rem_euclid(12);
    if s > 6 {
        s -= 12;
    }
    2f32.powf(s as f32 / 12.0)
}

/// Pedaço usado do arquivo do "Meu pad": pula o começo (onde costuma entrar aos poucos) e guarda 90 s,
/// que já dá um loop longo sem pesar na memória (pad de 30 minutos inteiro seria mais de 300 MB).
const MY_SKIP_S: usize = 8;
const MY_KEEP_S: usize = 90;

/// Descobre o tom do pad pelo som: as duas notas que mais soam de E1 a B4. Pad quase sempre é
/// tônica + quinta (G e D, C# e G#): se as duas estão a uma quinta, a de baixo da quinta é o tom.
/// Nome de arquivo engana ("C_Major_Pad" era C#), o som não.
pub fn detect_key(clip: &PadClip) -> usize {
    let frames = clip.frames();
    // mono, 4 amostras viram 1 (média = filtro simples), ~11 kHz
    let dec = 4;
    let sr = clip.rate as f32 / dec as f32;
    let block = 8192usize;
    let blocks = 24usize;
    let mut score = [0f64; 12];
    let total = frames / dec;
    if total < block {
        return 0;
    }
    for b in 0..blocks {
        let start = (total - block) * b / blocks.max(1);
        let x: Vec<f32> = (0..block)
            .map(|i| {
                let f = (start + i) * dec;
                (0..dec)
                    .map(|j| {
                        let p = (f + j).min(frames - 1) * 2;
                        (clip.data[p] as f32 + clip.data[p + 1] as f32) * 0.5
                    })
                    .sum::<f32>()
                    / dec as f32
            })
            .collect();
        for midi in 28..72 {
            let freq = 440.0 * 2f32.powf((midi as f32 - 69.0) / 12.0);
            // Goertzel: energia nesta frequência
            let w = std::f32::consts::TAU * freq / sr;
            let c = 2.0 * w.cos();
            let (mut s1, mut s2) = (0f32, 0f32);
            for (i, v) in x.iter().enumerate() {
                let win = 0.5 - 0.5 * (std::f32::consts::TAU * i as f32 / block as f32).cos();
                let s0 = v * win + c * s1 - s2;
                s2 = s1;
                s1 = s0;
            }
            let p = (s1 * s1 + s2 * s2 - c * s1 * s2) as f64;
            score[midi % 12] += p;
        }
    }
    let mut order: Vec<usize> = (0..12).collect();
    order.sort_by(|a, b| score[*b].total_cmp(&score[*a]));
    let (top, second) = (order[0], order[1]);
    if top == (second + 7) % 12 {
        second // ex.: G mais forte que C, mas G é a quinta de C
    } else {
        top
    }
}

/// Pads da parceria que já vêm dentro do programa (90 s de cada, já sem o começo).
/// Caminho "embutido:N" nos "Meus pads". Repositório privado: não redistribuir.
pub struct PartnerPad {
    pub name: &'static str,
    pub key: usize,
    pub credit: &'static str,
    bytes: &'static [u8],
}

pub const PARTNER_PADS: [PartnerPad; 3] = [
    PartnerPad {
        name: "Pad + Reverse em G - Em",
        key: 7,
        credit: "House of Pads",
        bytes: include_bytes!("../assets/pads/parceria/00.ogg"),
    },
    PartnerPad {
        name: "C# Major Pad",
        key: 1,
        credit: "Vishal Bhojane",
        bytes: include_bytes!("../assets/pads/parceria/01.ogg"),
    },
    PartnerPad {
        name: "Pad + Reverse em C - Am",
        key: 0,
        credit: "House of Pads",
        bytes: include_bytes!("../assets/pads/parceria/02.ogg"),
    },
];

pub fn partner_path(n: usize) -> String {
    format!("embutido:{n}")
}

/// v0.20: pads gravados a partir do MuseScore General (licença MIT, pode ir junto com o programa).
/// Caminho "livre:N" nos "Meus pads". Todos em C, sem terça (servem para maior e menor).
pub struct FreePad {
    pub name: &'static str,
    pub key: usize,
    bytes: &'static [u8],
}

pub const FREE_PADS: [FreePad; 3] = [
    FreePad {
        name: "Vênus",
        key: 0,
        bytes: include_bytes!("../assets/sons-livres/pads/00.ogg"),
    },
    FreePad {
        name: "Terra",
        key: 0,
        bytes: include_bytes!("../assets/sons-livres/pads/01.ogg"),
    },
    FreePad {
        name: "Marte",
        key: 0,
        bytes: include_bytes!("../assets/sons-livres/pads/02.ogg"),
    },
];

pub const FREE_CREDIT: &str = "MuseScore General (MIT)";

pub fn free_path(n: usize) -> String {
    format!("livre:{n}")
}

/// v0.21: nome curto (de planeta) de cada pad que vem no programa. Os nomes antigos viram estes uma vez só
/// (`cfg.pads_v21`); se o Theo renomeou, fica o nome dele.
pub fn planet_name(path: &str) -> Option<(&'static str, &'static str)> {
    // (nome novo, nome antigo)
    match path {
        "embutido:0" => Some(("Júpiter", "Pad + Reverse em G - Em")),
        "embutido:1" => Some(("Saturno", "C# Major Pad")),
        "embutido:2" => Some(("Netuno", "Pad + Reverse em C - Am")),
        "livre:0" => Some(("Vênus", "Coral Celeste")),
        "livre:1" => Some(("Terra", "Cordas Suaves")),
        "livre:2" => Some(("Marte", "Atmosfera")),
        _ => None,
    }
}

/// v0.20/v0.21: vozes de verdade do Voice FX (MuseScore General, todas em C3).
/// 0 Coral (aah + ooh), 1 Aah, 2 Ooh, 3 Celeste, 4 Synth.
pub const VOICE_NAMES: [&str; 5] = ["Choir", "Choir Aahs", "Voice Oohs", "Space Voice", "Synth Voice"];
pub const VOICE_HINTS: [&str; 5] = [
    "coral misturado (aah + ooh)",
    "coral cantando \"aah\"",
    "vozes suaves \"ooh\"",
    "vozes etéreas, de pad",
    "voz de sintetizador",
];
static VOICES: [&[u8]; 5] = [
    include_bytes!("../assets/sons-livres/coral_c3.ogg"),
    include_bytes!("../assets/sons-livres/vozes/00.ogg"),
    include_bytes!("../assets/sons-livres/vozes/01.ogg"),
    include_bytes!("../assets/sons-livres/vozes/02.ogg"),
    include_bytes!("../assets/sons-livres/vozes/03.ogg"),
];

pub fn load_voice(n: usize) -> Result<Arc<crate::granular::GrainSource>, String> {
    let clip = decode(Box::new(Cursor::new(VOICES[n % VOICES.len()])), Some("ogg"), false)?;
    Ok(Arc::new(crate::granular::GrainSource::from_stereo_i16(
        &clip.data, clip.rate, 48,
    )))
}

pub fn load_voicefx_choir() -> Result<Arc<crate::granular::GrainSource>, String> {
    load_voice(0)
}

/// Carrega o arquivo do "Meu pad" (ou um pad da parceria, "embutido:N").
pub fn load_my_pad(path: &str) -> Result<Arc<PadClip>, String> {
    let partner = path
        .strip_prefix("embutido:")
        .and_then(|n| n.parse::<usize>().ok())
        .and_then(|n| PARTNER_PADS.get(n));
    let free = path
        .strip_prefix("livre:")
        .and_then(|n| n.parse::<usize>().ok())
        .and_then(|n| FREE_PADS.get(n));
    // v0.35: pads da Biblioteca
    if let Some(set) = path.strip_prefix("tons:") {
        return load_key_set(set).map(Arc::new);
    }
    if let Some(spec) = path.strip_prefix("sf2:") {
        let (rel, bp) = spec.split_once('#').ok_or("pad de SF2 sem timbre")?;
        let (b, p) = bp.split_once(':').ok_or("pad de SF2 sem timbre")?;
        let clip = render_sf2_pad(rel, b.parse().unwrap_or(0), p.parse().unwrap_or(0))?;
        let mut data = clip.data;
        smooth_loop(&mut data, clip.rate as usize * 2);
        normalize_level(&mut data);
        return Ok(Arc::new(PadClip { data, rate: clip.rate, per_key: Vec::new() }));
    }
    let clip = match (partner, free) {
        (Some(p), _) => decode_part(Box::new(Cursor::new(p.bytes)), Some("ogg"), 0, MY_KEEP_S)?,
        (_, Some(p)) => decode_part(Box::new(Cursor::new(p.bytes)), Some("ogg"), 0, MY_KEEP_S)?,
        (None, None) => {
            let file =
                std::fs::File::open(path).map_err(|e| format!("Não consegui abrir {path}: {e}"))?;
            let ext = std::path::Path::new(path)
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| e.to_lowercase());
            decode_part(Box::new(file), ext.as_deref(), MY_SKIP_S, MY_KEEP_S)?
        }
    };
    let mut data = clip.data;
    // Emenda longa (2 s): pad que muda devagar não pode "pular" na volta do loop.
    smooth_loop(&mut data, clip.rate as usize * 2);
    normalize_level(&mut data);
    Ok(Arc::new(PadClip {
        data,
        rate: clip.rate,
        per_key: Vec::new(),
    }))
}

/// Texturas do "Pad Vivo" (motor granular), todas em C3.
pub const VIVO_NAMES: [&str; 3] = ["Vivo · Coro", "Vivo · Cordas", "Vivo · Cristal"];
pub const VIVO_HINTS: [&str; 3] = [
    "vozes que respiram",
    "cordas suaves e quentes",
    "brilho de vidro e sino",
];
static TEXTURES: [&[u8]; 3] = [
    include_bytes!("../assets/texturas/coro.ogg"),
    include_bytes!("../assets/texturas/cordas.ogg"),
    include_bytes!("../assets/texturas/cristal.ogg"),
];
/// Nota das texturas que vêm no programa.
pub const TEXTURE_ROOT: u8 = 48;

/// Carrega a textura do Pad Vivo `n` (ou o arquivo do Theo, numa nota `root`).
pub fn load_texture(
    n: usize,
    custom: Option<(&str, u8)>,
) -> Result<Arc<crate::granular::GrainSource>, String> {
    let (clip, root) = match custom {
        Some((path, root)) => {
            let file =
                std::fs::File::open(path).map_err(|e| format!("Não consegui abrir {path}: {e}"))?;
            let ext = std::path::Path::new(path)
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| e.to_lowercase());
            (decode(Box::new(file), ext.as_deref(), false)?, root)
        }
        None => (
            decode(Box::new(Cursor::new(TEXTURES[n % 3])), Some("ogg"), false)?,
            TEXTURE_ROOT,
        ),
    };
    // Textura muito longa só gasta memória: usa no máximo 20 s.
    let max = clip.rate as usize * 20 * 2;
    let data = &clip.data[..clip.data.len().min(max)];
    Ok(Arc::new(crate::granular::GrainSource::from_stereo_i16(
        data, clip.rate, root,
    )))
}

/// Nota (MIDI) de cada tom do pad: tudo perto de C3 (G..B uma oitava abaixo).
pub fn key_midi(key: usize) -> u8 {
    let k = (key % PAD_KEYS) as u8;
    if k <= 6 {
        48 + k
    } else {
        36 + k
    }
}

pub fn builtin_bytes(bank: usize, key: usize) -> &'static [u8] {
    BUILTIN[bank % PAD_BANKS][key % PAD_KEYS]
}

/// Decodifica o pad padrão ou o arquivo escolhido.
pub fn load(bank: usize, key: usize, custom: Option<&str>) -> Result<Arc<PadClip>, String> {
    match custom {
        Some(path) => {
            let file =
                std::fs::File::open(path).map_err(|e| format!("Não consegui abrir {path}: {e}"))?;
            let ext = std::path::Path::new(path)
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| e.to_lowercase());
            decode(Box::new(file), ext.as_deref(), true)
        }
        None => decode(
            Box::new(Cursor::new(builtin_bytes(bank, key))),
            Some("ogg"),
            false,
        ),
    }
    .map(Arc::new)
}

pub fn decode(
    source: Box<dyn MediaSource>,
    ext: Option<&str>,
    smooth: bool,
) -> Result<PadClip, String> {
    // Arquivo próprio muito longo: usa só os 3 primeiros minutos (não estoura a memória).
    let mut clip = decode_part(source, ext, 0, 180)?;
    if smooth {
        // Arquivos do usuário podem não ter emenda perfeita: suaviza o ponto do loop (80 ms).
        smooth_loop(&mut clip.data, clip.rate as usize * 80 / 1000);
    }
    Ok(clip)
}

/// Decodifica a partir de `skip_s` segundos, guardando no máximo `keep_s` segundos.
/// Se o arquivo for curto demais para pular, usa desde o começo.
fn decode_part(
    source: Box<dyn MediaSource>,
    ext: Option<&str>,
    skip_s: usize,
    keep_s: usize,
) -> Result<PadClip, String> {
    let mss = MediaSourceStream::new(source, Default::default());
    let mut hint = Hint::new();
    if let Some(e) = ext {
        hint.with_extension(e);
    }
    let probed = symphonia::default::get_probe()
        .format(
            &hint,
            mss,
            &FormatOptions::default(),
            &MetadataOptions::default(),
        )
        .map_err(|e| format!("Formato de áudio não reconhecido ({e})"))?;
    let mut format = probed.format;
    let track = format.default_track().ok_or("Arquivo sem áudio")?;
    let track_id = track.id;
    let mut decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, &DecoderOptions::default())
        .map_err(|e| format!("Não sei tocar esse tipo de áudio ({e})"))?;
    let mut rate = track.codec_params.sample_rate.unwrap_or(44_100);

    let mut data: Vec<i16> = Vec::new();
    let mut buf: Option<SampleBuffer<f32>> = None;
    // Começo guardado à parte: se o arquivo acabar antes de passar do pulo, ele é usado.
    let mut skipped: Vec<i16> = Vec::new();
    loop {
        let keep = rate as usize * keep_s * 2;
        if data.len() >= keep {
            data.truncate(keep);
            break;
        }
        let packet = match format.next_packet() {
            Ok(p) => p,
            Err(SymError::IoError(e)) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
            Err(SymError::ResetRequired) => break,
            Err(e) => return Err(format!("Erro lendo o áudio ({e})")),
        };
        if packet.track_id() != track_id {
            continue;
        }
        let decoded = match decoder.decode(&packet) {
            Ok(d) => d,
            Err(SymError::DecodeError(_)) => continue, // pedaço ruim: pula
            Err(e) => return Err(format!("Erro decodificando ({e})")),
        };
        let spec = *decoded.spec();
        rate = spec.rate;
        let channels = spec.channels.count().max(1);
        let b = buf.get_or_insert_with(|| SampleBuffer::new(decoded.capacity() as u64, spec));
        if b.capacity() < decoded.capacity() * channels {
            *b = SampleBuffer::new(decoded.capacity() as u64, spec);
        }
        b.copy_interleaved_ref(decoded);
        let to_i16 = |x: f32| (x.clamp(-1.0, 1.0) * 32767.0) as i16;
        let skip = rate as usize * skip_s * 2;
        for frame in b.samples().chunks(channels) {
            let l = frame[0];
            let r = if channels > 1 { frame[1] } else { l };
            let out = if skipped.len() < skip {
                &mut skipped
            } else {
                &mut data
            };
            out.push(to_i16(l));
            out.push(to_i16(r));
        }
    }
    if data.len() < 2 * 4410 {
        skipped.extend_from_slice(&data);
        data = skipped;
        data.truncate(rate as usize * keep_s * 2);
    }
    if data.len() < 2 * 4410 {
        return Err("Áudio curto demais (precisa de pelo menos 0,1 s)".into());
    }
    Ok(PadClip { data, rate, per_key: Vec::new() })
}

/// Deixa o pad no mesmo volume dos pads do programa (média ~ -16 dB), sem passar de -1 dB no pico:
/// trocar de um pad do YouTube para outro não muda o volume na igreja.
fn normalize_level(data: &mut [i16]) {
    let n = data.len().max(1) as f64;
    let rms = (data.iter().map(|&x| (x as f64).powi(2)).sum::<f64>() / n).sqrt() / 32768.0;
    let peak = data.iter().map(|&x| (x as i32).abs()).max().unwrap_or(0) as f64 / 32768.0;
    if rms < 1e-5 {
        return;
    }
    let target = 10f64.powf(-16.0 / 20.0);
    let gain = (target / rms).min(0.89 / peak.max(1e-5));
    for x in data.iter_mut() {
        *x = (*x as f64 * gain).clamp(-32768.0, 32767.0) as i16;
    }
}

/// Cruza o fim com o começo para o loop não estalar (encurta o áudio em `fade` quadros).
fn smooth_loop(data: &mut Vec<i16>, fade: usize) {
    let frames = data.len() / 2;
    if fade == 0 || frames < fade * 4 {
        return;
    }
    for i in 0..fade {
        let a = i as f32 / fade as f32 * std::f32::consts::FRAC_PI_2;
        let (head, tail) = (a.sin(), a.cos());
        for c in 0..2 {
            let h = data[i * 2 + c] as f32;
            let t = data[(frames - fade + i) * 2 + c] as f32;
            data[i * 2 + c] = (h * head + t * tail).clamp(-32768.0, 32767.0) as i16;
        }
    }
    data.truncate((frames - fade) * 2);
}

/// Uma "voz" de pad tocando dentro do motor.
pub struct PadVoice {
    pub clip: Arc<PadClip>,
    pos: f64,
    gain: f32,
    target: f32,
    /// Variação do ganho por amostra.
    rate: f32,
    /// Velocidade de leitura (1 = altura original; "Meu pad" muda o tom assim).
    pitch: f64,
    /// v0.24: 0 = pad principal, 1 = segundo pad (o mix entre os dois vale por camada).
    pub layer: u8,
}

pub const FADE_IN_S: f32 = 2.5;
pub const FADE_OUT_S: f32 = 3.0;

impl PadVoice {
    pub fn new(clip: Arc<PadClip>, sample_rate: u32, pitch: f32) -> Self {
        Self {
            clip,
            pitch: pitch as f64,
            pos: 0.0,
            gain: 0.0,
            target: 1.0,
            rate: 1.0 / (FADE_IN_S * sample_rate as f32),
            layer: 0,
        }
    }

    pub fn with_layer(mut self, layer: u8) -> Self {
        self.layer = layer.min(1);
        self
    }

    pub fn fade_out(&mut self, sample_rate: u32) {
        self.target = 0.0;
        self.rate = 1.0 / (FADE_OUT_S * sample_rate as f32);
    }

    /// Está sumindo (já mandaram parar)?
    pub fn fading(&self) -> bool {
        self.target == 0.0
    }

    /// Terminou de sumir?
    pub fn finished(&self) -> bool {
        self.target == 0.0 && self.gain <= 0.0
    }

    /// Soma este pad em `l`/`r` com o ganho `vol`.
    pub fn mix(&mut self, l: &mut [f32], r: &mut [f32], vol: f32, sample_rate: u32) {
        let frames = self.clip.frames();
        if frames < 2 {
            return;
        }
        let step = self.clip.rate as f64 / sample_rate as f64 * self.pitch;
        let data = &self.clip.data;
        let k = 1.0 / 32768.0;
        for i in 0..l.len() {
            if self.gain != self.target {
                if self.gain < self.target {
                    self.gain = (self.gain + self.rate).min(self.target);
                } else {
                    self.gain = (self.gain - self.rate).max(self.target);
                }
            }
            let p = self.pos as usize;
            let frac = (self.pos - p as f64) as f32;
            let n = if p + 1 >= frames { 0 } else { p + 1 };
            // curva "equal power" no fade para não afundar no meio da troca
            let g = vol * (self.gain * std::f32::consts::FRAC_PI_2).sin();
            let (a0, a1) = (data[p * 2] as f32, data[n * 2] as f32);
            let (b0, b1) = (data[p * 2 + 1] as f32, data[n * 2 + 1] as f32);
            l[i] += (a0 + (a1 - a0) * frac) * k * g;
            r[i] += (b0 + (b1 - b0) * frac) * k * g;
            self.pos += step;
            if self.pos >= frames as f64 {
                self.pos -= frames as f64;
            }
        }
    }
}

/// v0.31: texturas do Voice FX (sons da Pixabay que o Theo trouxe — ver assets/sons-livres/pixabay/LEIA.txt).
/// Não seguem nota por nota: tocam no TOM escolhido no Voice FX enquanto você toca.
pub struct TextureVoice {
    pub name: &'static str,
    pub hint: &'static str,
    /// Tom em que foi gravado (achado pelo som); `None` = descobre ao abrir.
    pub root: Option<usize>,
    pub looped: bool,
    pub restart: bool,
    /// v0.35: vem de um SF2 da Biblioteca ("arquivo#banco:programa"): só aparece depois de baixar.
    pub lib: Option<&'static str>,
}

pub const TEXTURE_VOICES: [TextureVoice; 9] = [
    TextureVoice { name: "Órion", hint: "coral angelical, longo", root: Some(1), looped: true, restart: false, lib: None },
    TextureVoice { name: "Lyra", hint: "coral celestial, cheio", root: Some(11), looped: true, restart: false, lib: None },
    TextureVoice { name: "Cassiopeia", hint: "brilho de sinos, \"céu\"", root: Some(6), looped: true, restart: false, lib: None },
    TextureVoice { name: "Andrômeda", hint: "vozes que respiram", root: Some(7), looped: true, restart: false, lib: None },
    TextureVoice { name: "Choir Rise", hint: "coral gospel subindo (toca uma vez)", root: None, looped: false, restart: true, lib: None },
    TextureVoice { name: "Prayer", hint: "trecho em Dó maior, 120 BPM", root: Some(0), looped: true, restart: true, lib: None },
    // v0.35: vozes da Biblioteca (constelações, como as outras texturas)
    TextureVoice { name: "Fênix", hint: "coro de vozes sintéticas (Biblioteca)", root: Some(0), looped: true, restart: false, lib: Some("Voices/Super Voice.sf2#0:0") },
    TextureVoice { name: "Pégaso", hint: "anjos, bem aberto (Biblioteca)", root: Some(0), looped: true, restart: false, lib: Some("Voices/Crying Angels.sf2#0:0") },
    TextureVoice { name: "Cisne", hint: "\"uuh\" suave de vozes (Biblioteca)", root: Some(0), looped: true, restart: false, lib: Some("Voices/Ooh La La.sf2#0:0") },
];

static TEXTURE_BYTES: [&[u8]; 4] = [
    include_bytes!("../assets/sons-livres/pads/03.ogg"),
    include_bytes!("../assets/sons-livres/pads/04.ogg"),
    include_bytes!("../assets/sons-livres/pads/05.ogg"),
    include_bytes!("../assets/sons-livres/pads/06.ogg"),
];

/// Primeiro número de voz do Voice FX que é textura (0..4 vozes, 5 = "Minha", 6.. = texturas).
pub const TEXTURE_BASE: usize = 6;

/// Abre a textura `n` (0..6) pronta para o motor.
pub fn load_texture_voice(n: usize) -> Result<Arc<crate::efeitos::VoiceTexture>, String> {
    let tv = TEXTURE_VOICES.get(n).ok_or("textura não existe")?;
    let clip = match tv.lib {
        // v0.35: voz da Biblioteca: o programa grava a textura a partir do SF2
        Some(spec) => {
            let (rel, bp) = spec.split_once('#').ok_or("textura sem timbre")?;
            let (b, p) = bp.split_once(':').ok_or("textura sem timbre")?;
            render_sf2_pad(rel, b.parse().unwrap_or(0), p.parse().unwrap_or(0))?
        }
        None => {
            let bytes: &'static [u8] = match n {
                0..=3 => TEXTURE_BYTES[n],
                _ => crate::efeitos::EXTRA_DRUM_SAMPLES[n - 4],
            };
            decode(Box::new(Cursor::new(bytes)), Some("ogg"), false)?
        }
    };
    let root = tv.root.unwrap_or_else(|| detect_key(&clip));
    let mut data = clip.data;
    if tv.looped {
        // textura: emenda longa (2 s); trecho musical (Prayer): emenda curtinha, para não sair do tempo
        let fade = if tv.restart { clip.rate as usize / 50 } else { clip.rate as usize * 2 };
        smooth_loop(&mut data, fade);
    }
    normalize_level(&mut data);
    let l = data.chunks(2).map(|c| c[0] as f32 / 32768.0).collect();
    let r = data.chunks(2).map(|c| *c.get(1).unwrap_or(&c[0]) as f32 / 32768.0).collect();
    Ok(Arc::new(crate::efeitos::VoiceTexture {
        l,
        r,
        rate: clip.rate,
        root: root as u8,
        looped: tv.looped,
        restart: tv.restart,
    }))
}

/// v0.35: a textura `n` do Voice FX pode ser usada? (as da Biblioteca só depois de baixar o arquivo)
pub fn texture_available(n: usize) -> bool {
    match TEXTURE_VOICES.get(n) {
        None => false,
        Some(t) => match t.lib {
            None => true,
            Some(spec) => spec
                .split_once('#')
                .is_some_and(|(rel, _)| crate::biblioteca::full_path(rel).exists()),
        },
    }
}

/// v0.35: leitor simples de WAV (PCM 16/24/32 bits ou float 32), para arquivos com cabeçalho
/// que o decodificador recusa. Devolve estéreo intercalado, no máximo `keep_s` segundos.
pub fn read_plain_wav(path: &std::path::Path, keep_s: usize) -> Option<PadClip> {
    let b = std::fs::read(path).ok()?;
    if b.len() < 12 || &b[0..4] != b"RIFF" || &b[8..12] != b"WAVE" {
        return None;
    }
    let u16_at = |i: usize| u16::from_le_bytes([b[i], b[i + 1]]);
    let u32_at = |i: usize| u32::from_le_bytes([b[i], b[i + 1], b[i + 2], b[i + 3]]);
    let (mut fmt, mut data) = (None, None);
    let mut p = 12;
    while p + 8 <= b.len() {
        let size = u32_at(p + 4) as usize;
        let body = p + 8;
        match &b[p..p + 4] {
            b"fmt " if body + 16 <= b.len() => {
                let mut tag = u16_at(body);
                if tag == 0xFFFE && size >= 26 && body + 26 <= b.len() {
                    tag = u16_at(body + 24); // WAVE_FORMAT_EXTENSIBLE: o formato de verdade vem no GUID
                }
                fmt = Some((tag, u16_at(body + 2) as usize, u32_at(body + 4), u16_at(body + 14) as usize));
            }
            b"data" => data = Some((body, size.min(b.len().saturating_sub(body)))),
            _ => {}
        }
        p = body + size + (size & 1);
    }
    let (tag, ch, rate, bits) = fmt?;
    let (start, len) = data?;
    if ch == 0 || !matches!((tag, bits), (1, 16) | (1, 24) | (1, 32) | (3, 32)) {
        return None;
    }
    let bytes = bits / 8;
    let frame = bytes * ch;
    let frames = (len / frame).min(rate as usize * keep_s);
    let sample = |i: usize| -> f32 {
        let s = &b[i..i + bytes];
        match (tag, bits) {
            (1, 16) => i16::from_le_bytes([s[0], s[1]]) as f32 / 32768.0,
            (1, 24) => (i32::from_le_bytes([0, s[0], s[1], s[2]]) >> 8) as f32 / 8_388_608.0,
            (1, 32) => i32::from_le_bytes([s[0], s[1], s[2], s[3]]) as f32 / 2_147_483_648.0,
            _ => f32::from_le_bytes([s[0], s[1], s[2], s[3]]),
        }
    };
    let mut out = Vec::with_capacity(frames * 2);
    for f in 0..frames {
        let i = start + f * frame;
        let l = sample(i);
        let r = if ch > 1 { sample(i + bytes) } else { l };
        out.push((l.clamp(-1.0, 1.0) * 32767.0) as i16);
        out.push((r.clamp(-1.0, 1.0) * 32767.0) as i16);
    }
    (out.len() >= 2 * 4410).then_some(PadClip { data: out, rate, per_key: Vec::new() })
}
