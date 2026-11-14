//! v0.35 "Biblioteca": os melhores timbres do pacote do Drive do Theo, cada um no seu arquivo SF2
//! (pianos, EPs, órgãos, pads, strings, synths, effects e vozes) em %APPDATA%\PianoSF2\sons\biblioteca.
//!
//! - Os timbres entram na MESMA lista do "Sons SF2" (junto do SF2 principal), com a categoria do lado.
//! - Só o arquivo que uma camada usa fica na memória (o notebook tem 8 GB; tudo junto passaria de 3 GB).
//! - Para a lista basta ler os nomes dos timbres (pedaço `phdr` do fim do arquivo), sem ler o som.
//! - "Baixar tudo" busca o que faltar no Google Drive (lista em assets/biblioteca.txt).
//!
//! NÃO publicar: são sons de terceiros (pacote do Drive), só para uso do Theo.

use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

/// Um arquivo da lista para baixar.
#[derive(Clone, Debug)]
pub struct Entry {
    /// Pasta (= categoria), ex.: "Pads" ou "Pads 12 tons/Dream Shimmer".
    pub dir: String,
    pub file: String,
    pub id: String,
    pub bytes: u64,
}

impl Entry {
    pub fn rel(&self) -> String {
        format!("{}/{}", self.dir, self.file)
    }
    pub fn path(&self) -> PathBuf {
        dir().join(&self.dir).join(&self.file)
    }
}

const LIST: &str = include_str!("../assets/biblioteca.txt");

pub fn entries() -> Vec<Entry> {
    LIST.lines()
        .filter(|l| !l.trim().is_empty() && !l.starts_with('#'))
        .filter_map(|l| {
            let f: Vec<&str> = l.split('|').map(str::trim).collect();
            (f.len() == 4).then(|| Entry {
                dir: f[0].into(),
                file: f[1].into(),
                id: f[2].into(),
                bytes: (f[3].parse::<f64>().unwrap_or(1.0) * 1_048_576.0) as u64,
            })
        })
        .collect()
}

pub fn dir() -> PathBuf {
    crate::sons::dir().join("biblioteca")
}

/// Caminho completo de um timbre da Biblioteca ("Pads/Hillsong Pad.sf2" → ...\biblioteca\Pads\Hillsong Pad.sf2).
pub fn full_path(rel: &str) -> PathBuf {
    rel.split('/').fold(dir(), |p, part| p.join(part))
}

/// Categorias na ordem da tela: (pasta, nome na tela).
pub const CATEGORIES: [(&str, &str); 8] = [
    ("Pianos", "Piano"),
    ("EP", "EP (piano elétrico)"),
    ("Organ", "Órgão"),
    ("Pads", "Pads"),
    ("Strings", "Strings (cordas)"),
    ("Synths", "Synths"),
    ("Effects", "Effects"),
    ("Voices", "Vozes"),
];

pub fn category_of(rel: &str) -> &'static str {
    let top = rel.split('/').next().unwrap_or("");
    CATEGORIES
        .iter()
        .find(|(d, _)| *d == top)
        .map(|(_, n)| *n)
        .unwrap_or("Outros")
}

/// Um timbre dentro de um arquivo.
#[derive(Clone, Debug, PartialEq)]
pub struct Preset {
    pub bank: i32,
    pub patch: i32,
    pub name: String,
}

/// Lê só os nomes dos timbres de um SF2 (sem ler o som): RIFF sfbk → LIST pdta → phdr.
pub fn read_presets(path: &Path) -> Result<Vec<Preset>, String> {
    let mut f = std::fs::File::open(path).map_err(|e| e.to_string())?;
    let mut head = [0u8; 12];
    f.read_exact(&mut head).map_err(|e| e.to_string())?;
    if &head[0..4] != b"RIFF" || &head[8..12] != b"sfbk" {
        return Err("não é um SF2".into());
    }
    let end = u32::from_le_bytes(head[4..8].try_into().unwrap()) as u64 + 8;
    let mut pos = 12u64;
    let mut ck = [0u8; 12];
    while pos + 12 <= end {
        f.seek(SeekFrom::Start(pos)).map_err(|e| e.to_string())?;
        f.read_exact(&mut ck).map_err(|e| e.to_string())?;
        let size = u32::from_le_bytes(ck[4..8].try_into().unwrap()) as u64;
        if &ck[0..4] == b"LIST" && &ck[8..12] == b"pdta" {
            // dentro do pdta: phdr, pbag, ...
            let mut p = pos + 12;
            let stop = pos + 8 + size;
            let mut sub = [0u8; 8];
            while p + 8 <= stop {
                f.seek(SeekFrom::Start(p)).map_err(|e| e.to_string())?;
                f.read_exact(&mut sub).map_err(|e| e.to_string())?;
                let n = u32::from_le_bytes(sub[4..8].try_into().unwrap()) as u64;
                if &sub[0..4] == b"phdr" {
                    let mut buf = vec![0u8; n as usize];
                    f.read_exact(&mut buf).map_err(|e| e.to_string())?;
                    let mut out: Vec<Preset> = buf
                        .chunks_exact(38)
                        .map(|r| {
                            let name: String = r[..20]
                                .iter()
                                .take_while(|&&b| b != 0)
                                .map(|&b| b as char)
                                .collect();
                            let patch = u16::from_le_bytes([r[20], r[21]]) as i32;
                            let bank = u16::from_le_bytes([r[22], r[23]]) as i32;
                            Preset { bank, patch, name: name.trim().to_string() }
                        })
                        .collect();
                    // o último registro é o terminal (EOP)
                    if buf.len() >= 38 {
                        out.truncate((buf.len() / 38).saturating_sub(1));
                    }
                    out.sort_by_key(|p| (p.bank, p.patch));
                    out.dedup_by_key(|p| (p.bank, p.patch));
                    return Ok(out);
                }
                p += 8 + n + (n & 1);
            }
            break;
        }
        pos += 8 + size + (size & 1);
    }
    Err("SF2 sem lista de timbres".into())
}

/// Um timbre da Biblioteca para a lista: arquivo (relativo) + timbre + nome bonito.
#[derive(Clone, Debug)]
pub struct LibPreset {
    pub rel: String,
    pub bank: i32,
    pub patch: i32,
    /// Nome na tela: o nome do arquivo (se ele tem um timbre só) ou "Arquivo · timbre".
    pub name: String,
}

/// Procura os SF2 baixados (pastas de categoria) e lê os timbres de cada um.
pub fn scan() -> Vec<LibPreset> {
    let mut out = Vec::new();
    let root = dir();
    for (cat, _) in CATEGORIES {
        let Ok(rd) = std::fs::read_dir(root.join(cat)) else { continue };
        let mut files: Vec<PathBuf> = rd
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| p.extension().is_some_and(|x| x.eq_ignore_ascii_case("sf2")))
            .collect();
        files.sort();
        for path in files {
            let Some(fname) = path.file_name().map(|f| f.to_string_lossy().to_string()) else { continue };
            let stem = path.file_stem().map(|f| f.to_string_lossy().to_string()).unwrap_or_default();
            let Ok(presets) = read_presets(&path) else { continue };
            let one = presets.len() == 1;
            for p in presets {
                out.push(LibPreset {
                    rel: format!("{cat}/{fname}"),
                    bank: p.bank,
                    patch: p.patch,
                    name: if one { stem.clone() } else { format!("{stem} · {}", p.name) },
                });
            }
        }
    }
    out
}

/// Arquivos da lista que ainda não estão no computador.
pub fn missing() -> Vec<Entry> {
    entries().into_iter().filter(|e| !e.path().exists()).collect()
}

pub fn total_bytes() -> u64 {
    entries().iter().map(|e| e.bytes).sum()
}

/// Pad gravado nos 12 tons (pasta "Pads 12 tons/<nome>" com C.wav, C#.wav ... B.wav)?
pub fn key_file(set: &str, key: usize) -> PathBuf {
    dir().join("Pads 12 tons").join(set).join(format!("{}.wav", crate::pads::KEY_NAMES[key % 12]))
}

// ---------- Baixar tudo (um arquivo por vez, com o curl do Windows) ----------

pub struct DownloadAll {
    queue: Vec<Entry>,
    cur: Option<(Entry, Child)>,
    done_bytes: u64,
    total: u64,
    pub failed: Vec<String>,
}

pub enum Poll {
    /// Fração baixada (0..1) e o nome do arquivo que está vindo agora.
    Running(f32, String),
    Done,
}

#[cfg(windows)]
fn hidden(cmd: &mut Command) -> &mut Command {
    use std::os::windows::process::CommandExt;
    cmd.creation_flags(0x0800_0000)
}
#[cfg(not(windows))]
fn hidden(cmd: &mut Command) -> &mut Command {
    cmd
}

fn part(e: &Entry) -> PathBuf {
    let mut p = e.path().into_os_string();
    p.push(".baixando");
    PathBuf::from(p)
}

fn url(e: &Entry) -> String {
    format!("https://drive.usercontent.google.com/download?id={}&export=download&confirm=t", e.id)
}

/// Chegou inteiro? (SF2 e WAV são RIFF: o tamanho vem escrito no começo)
fn complete(path: &Path) -> bool {
    let len = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);
    let mut head = [0u8; 8];
    std::fs::File::open(path)
        .and_then(|mut f| f.read_exact(&mut head))
        .is_ok()
        && &head[0..4] == b"RIFF"
        && u32::from_le_bytes(head[4..8].try_into().unwrap()) as u64 + 8 <= len
}

impl DownloadAll {
    pub fn start() -> Self {
        let queue = missing();
        let total = queue.iter().map(|e| e.bytes).sum::<u64>().max(1);
        Self { queue, cur: None, done_bytes: 0, total, failed: Vec::new() }
    }

    pub fn poll(&mut self) -> Poll {
        if let Some((e, child)) = &mut self.cur {
            match child.try_wait() {
                Ok(None) => {
                    let got = std::fs::metadata(part(e)).map(|m| m.len()).unwrap_or(0).min(e.bytes);
                    let f = (self.done_bytes + got) as f32 / self.total as f32;
                    return Poll::Running(f.min(0.99), e.file.clone());
                }
                Ok(Some(s)) => {
                    let tmp = part(e);
                    if s.success() && complete(&tmp) && std::fs::rename(&tmp, e.path()).is_ok() {
                        self.done_bytes += e.bytes;
                    } else {
                        let _ = std::fs::remove_file(&tmp);
                        self.failed.push(e.rel());
                        self.done_bytes += e.bytes;
                    }
                    self.cur = None;
                }
                Err(_) => {
                    self.failed.push(e.rel());
                    self.cur = None;
                }
            }
        }
        // próximo da fila
        while let Some(e) = (!self.queue.is_empty()).then(|| self.queue.remove(0)) {
            if e.path().exists() {
                continue;
            }
            if let Some(d) = e.path().parent() {
                let _ = std::fs::create_dir_all(d);
            }
            let tmp = part(&e);
            let _ = std::fs::remove_file(&tmp);
            let curl = if cfg!(windows) { "curl.exe" } else { "curl" };
            let spawn = hidden(Command::new(curl).args(["-fsSL", "--retry", "3", "-o"]).arg(&tmp).arg(url(&e)))
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn();
            match spawn {
                Ok(c) => {
                    let name = e.file.clone();
                    self.cur = Some((e, c));
                    return Poll::Running(self.done_bytes as f32 / self.total as f32, name);
                }
                Err(_) => self.failed.push(e.rel()),
            }
        }
        Poll::Done
    }

    pub fn cancel(mut self) {
        if let Some((e, mut c)) = self.cur.take() {
            let _ = c.kill();
            let _ = c.wait();
            let _ = std::fs::remove_file(part(&e));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lista_da_biblioteca() {
        let e = entries();
        assert!(e.len() > 90, "{}", e.len());
        let gb = total_bytes() as f64 / 1e9;
        assert!(gb < 3.8, "passou de 3,5 GB: {gb}");
        for x in &e {
            assert!(x.id.len() > 20, "{}", x.file);
            let top = x.dir.split('/').next().unwrap();
            assert!(top == "Pads 12 tons" || CATEGORIES.iter().any(|(d, _)| *d == top), "{}", x.dir);
        }
        // 12 tons completos em cada pad gravado
        for set in ["Dream Shimmer", "Reverse Waves"] {
            assert_eq!(e.iter().filter(|x| x.dir.ends_with(set)).count(), 12);
        }
    }
}

#[cfg(test)]
mod listar {
    /// Lista os timbres de cada arquivo baixado: `cargo test --no-default-features listar -- --ignored --nocapture`
    #[test]
    #[ignore]
    fn listar_timbres() {
        let all = super::scan();
        let mut last = String::new();
        for p in &all {
            if p.rel != last {
                println!("== {}", p.rel);
                last = p.rel.clone();
            }
            println!("   {:03}:{:03} {}", p.bank, p.patch, p.name);
        }
        println!("{} timbres", all.len());
    }
}

#[cfg(test)]
mod regioes {
    /// Teclas de cada som dos Effects: `cargo test --no-default-features --lib regioes -- --ignored --nocapture`
    #[test]
    #[ignore]
    fn regioes_dos_effects() {
        for rel in ["Effects/Mobile FX Worship.sf2", "Effects/Reverse.sf2", "Effects/Reverse Sweep.sf2", "Effects/Metal Swell.sf2", "Effects/Synth FX.sf2"] {
            let Ok(sf) = crate::core::load_font_file(rel) else { continue };
            println!("== {rel}");
            let hs = sf.get_sample_headers();
            for ins in sf.get_instruments() {
                println!("  instr {}", ins.get_name());
                for r in ins.get_regions() {
                    let h = &hs[r.get_sample_id()];
                    let secs = (r.get_sample_end() - r.get_sample_start()) as f32 / h.get_sample_rate() as f32;
                    println!("    {:3}-{:3} {:24} {:.1}s loop={}", r.get_key_range_start(), r.get_key_range_end(), h.get_name(), secs, r.get_sample_end_loop() > r.get_sample_start_loop());
                }
            }
        }
    }
}

#[cfg(test)]
mod analise {
    use rustysynth::{Synthesizer, SynthesizerSettings};
    /// `cargo test --no-default-features --lib analise -- --ignored --nocapture`
    #[test]
    #[ignore]
    fn analisar_fx() {
        let sf = crate::core::load_font_file("Effects/Mobile FX Worship.sf2").unwrap();
        let st = SynthesizerSettings::new(44100);
        for key in 36..=89 {
            let mut s = Synthesizer::new(&sf, &st).unwrap();
            s.set_channel_preset(0, 0, 1);
            s.note_on(0, key, 110);
            let n = 44100 * 7;
            let (mut l, mut r) = (vec![0f32; n], vec![0f32; n]);
            s.render(&mut l, &mut r);
            // envelope em janelas de 100 ms
            let env: Vec<f32> = l.chunks(4410).zip(r.chunks(4410)).map(|(a, b)| (a.iter().chain(b).map(|x| x * x).sum::<f32>() / (2 * a.len()) as f32).sqrt()).collect();
            let peak = env.iter().cloned().fold(0.0, f32::max);
            if peak < 1e-4 { println!("{key}: mudo"); continue; }
            let pk = env.iter().position(|&e| e == peak).unwrap();
            let last = env.iter().rposition(|&e| e > peak * 0.05).unwrap_or(0);
            let zc = l.windows(2).filter(|w| (w[0] >= 0.0) != (w[1] >= 0.0)).count() as f32 / (last.max(1) as f32 * 0.1);
            let shape: String = env.iter().take(last + 1).step_by(2).map(|&e| { let v = (e / peak * 8.0) as usize; [' ', '.', ':', '-', '=', '+', '*', '#', '#'][v.min(8)] }).collect();
            println!("{key}: pico {:.1}s dur {:.1}s zc/s {:5.0} nivel {:.3} |{}|", pk as f32 / 10.0, last as f32 / 10.0, zc, peak, shape);
        }
    }
}

// ---------- v0.35: DrumPads de efeito (tirados do "Mobile FX Worship") ----------

/// (nome do DrumPad, tecla no Mobile FX Worship). Escolhidos pelo formato do som:
/// Boom (grave escuro), Hit FX (pancada com cauda), Swell (cresce e some), Riser FX (sobe até o fim),
/// Reverse FX (som ao contrário, corta no pico) e Whoosh (passa rápido).
pub const LIB_DRUMS: [(&str, i32); 6] = [
    ("Boom", 38),
    ("Hit FX", 47),
    ("Swell", 86),
    ("Riser FX", 88),
    ("Reverse FX", 85),
    ("Whoosh", 89),
];
pub const FX_FILE: &str = "Effects/Mobile FX Worship.sf2";

pub fn drum_wav(name: &str) -> PathBuf {
    dir().join("DrumPads").join(format!("{name}.wav"))
}

/// Grava (uma vez) os sons dos DrumPads de efeito em WAV e devolve os que estão prontos.
pub fn make_lib_drums() -> Vec<(&'static str, PathBuf)> {
    if !full_path(FX_FILE).exists() {
        return Vec::new();
    }
    let need = LIB_DRUMS.iter().any(|(n, _)| !drum_wav(n).exists());
    let sf = if need { crate::core::load_font_file(FX_FILE).ok() } else { None };
    let mut out = Vec::new();
    for (name, key) in LIB_DRUMS {
        let path = drum_wav(name);
        if !path.exists() {
            let Some(sf) = &sf else { continue };
            if render_one_shot(sf, 0, 1, key, &path).is_err() {
                continue;
            }
        }
        out.push((name, path));
    }
    out
}

fn render_one_shot(sf: &std::sync::Arc<rustysynth::SoundFont>, bank: i32, patch: i32, key: i32, path: &Path) -> Result<(), String> {
    use rustysynth::{Synthesizer, SynthesizerSettings};
    const RATE: u32 = 44_100;
    let mut s = Synthesizer::new(sf, &SynthesizerSettings::new(RATE as i32)).map_err(|e| e.to_string())?;
    s.set_channel_preset(0, bank, patch);
    s.note_on(0, key, 112);
    let n = RATE as usize * 7;
    let (mut l, mut r) = (vec![0f32; n], vec![0f32; n]);
    s.render(&mut l, &mut r);
    // corta o silêncio do fim
    let peak = l.iter().chain(&r).fold(0f32, |m, x| m.max(x.abs()));
    if peak < 1e-4 {
        return Err("mudo".into());
    }
    let end = (0..n).rposition(|i| l[i].abs().max(r[i].abs()) > peak * 0.003).unwrap_or(n - 1) + 1;
    let gain = 0.9 / peak;
    let mut pcm = Vec::with_capacity(end * 4);
    for i in 0..end {
        for x in [l[i], r[i]] {
            pcm.extend_from_slice(&(((x * gain).clamp(-1.0, 1.0) * 32767.0) as i16).to_le_bytes());
        }
    }
    if let Some(d) = path.parent() {
        std::fs::create_dir_all(d).map_err(|e| e.to_string())?;
    }
    std::fs::write(path, wav_bytes(&pcm, RATE)).map_err(|e| e.to_string())
}

/// WAV 16 bits estéreo.
fn wav_bytes(pcm: &[u8], rate: u32) -> Vec<u8> {
    let mut w = Vec::with_capacity(44 + pcm.len());
    w.extend_from_slice(b"RIFF");
    w.extend_from_slice(&(36 + pcm.len() as u32).to_le_bytes());
    w.extend_from_slice(b"WAVEfmt ");
    w.extend_from_slice(&16u32.to_le_bytes());
    w.extend_from_slice(&1u16.to_le_bytes()); // PCM
    w.extend_from_slice(&2u16.to_le_bytes()); // estéreo
    w.extend_from_slice(&rate.to_le_bytes());
    w.extend_from_slice(&(rate * 4).to_le_bytes());
    w.extend_from_slice(&4u16.to_le_bytes());
    w.extend_from_slice(&16u16.to_le_bytes());
    w.extend_from_slice(b"data");
    w.extend_from_slice(&(pcm.len() as u32).to_le_bytes());
    w.extend_from_slice(pcm);
    w
}
