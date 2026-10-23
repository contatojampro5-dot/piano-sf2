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
pub const PAD_BANKS: usize = 2;

pub const KEY_NAMES: [&str; PAD_KEYS] = [
    "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
];
pub const KEY_NAMES_PT: [&str; PAD_KEYS] = [
    "Dó", "Dó#", "Ré", "Ré#", "Mi", "Fá", "Fá#", "Sol", "Sol#", "Lá", "Lá#", "Si",
];
pub const BANK_NAMES: [&str; PAD_BANKS] = ["Escuro", "Coral"];
pub const BANK_HINTS: [&str; PAD_BANKS] = ["grave, fechado, místico", "vozes graves e médias"];

/// Um pad decodificado: estéreo intercalado (L, R, L, R...).
pub struct PadClip {
    pub data: Vec<i16>,
    pub rate: u32,
}

impl PadClip {
    pub fn frames(&self) -> usize {
        self.data.len() / 2
    }
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

static BUILTIN: [[&[u8]; PAD_KEYS]; PAD_BANKS] = [bank!("escuro"), bank!("coral")];

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
    loop {
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
        for frame in b.samples().chunks(channels) {
            let l = frame[0];
            let r = if channels > 1 { frame[1] } else { l };
            data.push(to_i16(l));
            data.push(to_i16(r));
        }
    }
    if data.len() < 2 * 4410 {
        return Err("Áudio curto demais (precisa de pelo menos 0,1 s)".into());
    }
    if smooth {
        // Arquivos do usuário podem não ter emenda perfeita: suaviza o ponto do loop (80 ms).
        smooth_loop(&mut data, rate as usize * 80 / 1000);
    }
    Ok(PadClip { data, rate })
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
}

pub const FADE_IN_S: f32 = 2.5;
pub const FADE_OUT_S: f32 = 3.0;

impl PadVoice {
    pub fn new(clip: Arc<PadClip>, sample_rate: u32) -> Self {
        Self {
            clip,
            pos: 0.0,
            gain: 0.0,
            target: 1.0,
            rate: 1.0 / (FADE_IN_S * sample_rate as f32),
        }
    }

    pub fn fade_out(&mut self, sample_rate: u32) {
        self.target = 0.0;
        self.rate = 1.0 / (FADE_OUT_S * sample_rate as f32);
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
        let step = self.clip.rate as f64 / sample_rate as f64;
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
