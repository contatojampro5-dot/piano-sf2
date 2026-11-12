//! v0.17 — Efeitos (aba 🥁 Efeitos), inspirados no vídeo que o Theo mandou:
//! - **Tempo**: BPM único para tudo (com "bater o tempo"), para PulsePad e Arpejador andarem juntos.
//! - **PulsePad**: os pads (e o ❄ congelado) pulsam no tempo ("tum... tum...").
//! - **Arpejador**: o Theo segura o acorde e uma camada escolhida toca as notas uma a uma.
//! - **DrumPads**: 8 sons de bateria/efeito gerados pelo próprio programa (sem direito autoral).
//! - **Voice FX**: um coral (textura "Coro" do Pad Vivo) que acompanha cada nota tocada.
//!
//! Tudo que roda na thread de áudio aqui não aloca memória depois de criado.

use std::sync::Arc;

use crate::granular::GrainSource;

// ---------------------------------------------------------------------------------------------
// Tempo
// ---------------------------------------------------------------------------------------------

pub const BPM_MIN: f32 = 40.0;
pub const BPM_MAX: f32 = 220.0;

/// Quantas amostras dura um passo (`per_beat` passos por tempo: 1 = semínima, 2 = colcheia, 4 = semicolcheia).
pub fn step_samples(bpm: f32, per_beat: u32, sr: u32) -> f64 {
    sr as f64 * 60.0 / bpm.clamp(BPM_MIN, BPM_MAX) as f64 / per_beat.clamp(1, 8) as f64
}

/// "Bater o tempo": BPM pela média dos intervalos entre as batidas (precisa de 2 ou mais).
pub fn bpm_from_taps(taps_s: &[f64]) -> Option<f32> {
    if taps_s.len() < 2 {
        return None;
    }
    let n = taps_s.len() - 1;
    let avg = (taps_s[n] - taps_s[0]) / n as f64;
    if avg <= 0.0 {
        return None;
    }
    Some(((60.0 / avg) as f32).clamp(BPM_MIN, BPM_MAX).round())
}

// ---------------------------------------------------------------------------------------------
// PulsePad
// ---------------------------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PulseParams {
    pub on: bool,
    /// Pulsos por tempo (1, 2 ou 4).
    pub per_beat: u32,
    /// Quanto o volume cai entre um pulso e outro (0 = nada, 1 = some).
    pub depth: f32,
}

impl Default for PulseParams {
    fn default() -> Self {
        Self {
            on: false,
            per_beat: 2,
            depth: 0.85,
        }
    }
}

/// Volume do pulso na fase 0..1 do passo: entra rápido (sem estalo) e cai devagar.
pub fn pulse_gain(phase: f32, depth: f32) -> f32 {
    let p = phase.clamp(0.0, 1.0);
    let env = if p < 0.04 {
        p / 0.04
    } else {
        (1.0 - (p - 0.04) / 0.96).powf(1.6)
    };
    1.0 - depth.clamp(0.0, 1.0) * (1.0 - env)
}

// ---------------------------------------------------------------------------------------------
// Arpejador
// ---------------------------------------------------------------------------------------------

pub const ARP_PATTERNS: [&str; 4] = ["Up", "Down", "Up/Down", "Random"];

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ArpParams {
    pub on: bool,
    /// Camada que toca o arpejo (as outras continuam tocando o acorde normal).
    pub layer: usize,
    /// 0 sobe, 1 desce, 2 sobe e desce, 3 aleatório.
    pub pattern: u8,
    /// Notas por tempo (2 = colcheia, 3 = tercina, 4 = semicolcheia).
    pub per_beat: u32,
    /// Quantas oitavas o arpejo sobe (1..3).
    pub octaves: u8,
    /// Segurar: o arpejo continua depois de soltar as teclas (até o próximo acorde).
    pub latch: bool,
}

impl Default for ArpParams {
    fn default() -> Self {
        Self {
            on: false,
            layer: 0,
            pattern: 0,
            per_beat: 2,
            octaves: 2,
            latch: false,
        }
    }
}

/// Estado do arpejador (no motor).
pub struct Arp {
    pub p: ArpParams,
    keys: [u64; 2],
    /// Teclas seguradas agora de verdade (para saber quando começa um acorde novo no "segurar").
    held: u32,
    /// Nota do arpejo soando agora.
    pub playing: Option<u8>,
    step: u32,
    rng: u32,
    pub vel: u8,
}

impl Default for Arp {
    fn default() -> Self {
        Self {
            p: ArpParams::default(),
            keys: [0; 2],
            held: 0,
            playing: None,
            step: 0,
            rng: 0x2545_f491,
            vel: 96,
        }
    }
}

impl Arp {
    fn bit(k: u8) -> (usize, u64) {
        ((k as usize >> 6) & 1, 1u64 << (k & 63))
    }

    pub fn key_down(&mut self, k: u8, vel: u8) {
        if self.p.latch && self.held == 0 {
            // primeira tecla de um acorde novo: troca o acorde guardado
            self.keys = [0; 2];
            self.step = 0;
        }
        if self.is_empty() {
            self.step = 0;
        }
        self.held += 1;
        let (i, b) = Self::bit(k);
        self.keys[i] |= b;
        self.vel = vel.max(1);
    }

    pub fn key_up(&mut self, k: u8) {
        self.held = self.held.saturating_sub(1);
        if !self.p.latch {
            let (i, b) = Self::bit(k);
            self.keys[i] &= !b;
        }
    }

    /// Desligou o "segurar" com as mãos fora do teclado: o acorde guardado sai.
    pub fn drop_latched(&mut self) {
        if self.held == 0 {
            self.keys = [0; 2];
        }
    }

    pub fn clear(&mut self) {
        self.keys = [0; 2];
        self.held = 0;
    }

    pub fn is_empty(&self) -> bool {
        self.keys == [0; 2]
    }

    fn rand(&mut self) -> u32 {
        self.rng ^= self.rng << 13;
        self.rng ^= self.rng >> 17;
        self.rng ^= self.rng << 5;
        self.rng
    }

    /// Próxima nota do arpejo (None = nada segurado).
    pub fn next_note(&mut self) -> Option<u8> {
        let mut notes = [0u8; 128];
        let mut n = 0;
        for k in 0..128u8 {
            let (i, b) = Self::bit(k);
            if self.keys[i] & b != 0 {
                notes[n] = k;
                n += 1;
            }
        }
        if n == 0 {
            return None;
        }
        let len = n * self.p.octaves.clamp(1, 3) as usize;
        let s = self.step as usize;
        let idx = match self.p.pattern {
            0 => s % len,
            1 => len - 1 - s % len,
            2 => {
                if len == 1 {
                    0
                } else {
                    let period = 2 * len - 2;
                    let x = s % period;
                    if x < len {
                        x
                    } else {
                        period - x
                    }
                }
            }
            _ => self.rand() as usize % len,
        };
        self.step = self.step.wrapping_add(1);
        let note = notes[idx % n] as i32 + 12 * (idx / n) as i32;
        Some(note.clamp(0, 127) as u8)
    }
}

// ---------------------------------------------------------------------------------------------
// DrumPads (sons gerados aqui mesmo)
// ---------------------------------------------------------------------------------------------

pub const DRUMS: usize = 8;
pub const DRUM_NAMES: [&str; DRUMS] = [
    "Kick", "Snare", "Clap", "Hi-hat", "Crash", "808", "Riser", "Impact",
];

/// Os 8 sons, prontos (estéreo), na taxa de amostragem da placa.
pub struct DrumKit {
    pub hits: Vec<(Vec<f32>, Vec<f32>)>,
    pub rate: u32,
}

/// Ruído branco simples e repetível.
struct Noise(u32);
impl Noise {
    fn next(&mut self) -> f32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 17;
        self.0 ^= self.0 << 5;
        (self.0 >> 8) as f32 / (1u32 << 23) as f32 - 1.0
    }
}

fn lp_coef(fc: f32, sr: f32) -> f32 {
    1.0 - (-std::f32::consts::TAU * fc / sr).exp()
}

impl DrumKit {
    /// Sons sintetizados (v0.17): reserva, caso um som gravado não abra.
    pub fn build_synth(rate: u32) -> Self {
        let sr = rate as f32;
        let tau = std::f32::consts::TAU;
        let len = |s: f32| (s * sr) as usize;
        let mut hits = Vec::with_capacity(DRUMS);
        let mut nz = Noise(0x1357_9bdf);

        // 0 Bumbo: seno que cai de 120 para 48 Hz + estalinho
        let mut k = vec![0.0f32; len(0.5)];
        let mut ph = 0.0f32;
        for (i, s) in k.iter_mut().enumerate() {
            let t = i as f32 / sr;
            let f = 48.0 + 72.0 * (-t * 35.0).exp();
            ph += tau * f / sr;
            let click = if t < 0.004 { nz.next() * (1.0 - t / 0.004) * 0.3 } else { 0.0 };
            *s = (ph.sin() * (-t * 7.0).exp() + click) * 0.85;
        }
        hits.push((k.clone(), k));

        // 1 Caixa: tom de 190 Hz + ruído aberto
        let (mut l, mut r) = (vec![0.0f32; len(0.35)], vec![0.0f32; len(0.35)]);
        let (mut hp_l, mut prev_l, mut hp_r, mut prev_r) = (0.0f32, 0.0f32, 0.0f32, 0.0f32);
        for i in 0..l.len() {
            let t = i as f32 / sr;
            let tone = (tau * 190.0 * t).sin() * (-t * 28.0).exp() * 0.45;
            let (a, b) = (nz.next(), nz.next());
            hp_l = 0.92 * (hp_l + a - prev_l);
            prev_l = a;
            hp_r = 0.92 * (hp_r + b - prev_r);
            prev_r = b;
            let env = (-t * 16.0).exp() * 0.55;
            l[i] = tone + hp_l * env;
            r[i] = tone + hp_r * env;
        }
        hits.push((l, r));

        // 2 Palmas: 3 rajadas curtas de ruído em ~1,2 kHz e um rabinho
        let (mut l, mut r) = (vec![0.0f32; len(0.4)], vec![0.0f32; len(0.4)]);
        let (mut bl, mut br) = (0.0f32, 0.0f32);
        let a = lp_coef(1800.0, sr);
        for i in 0..l.len() {
            let t = i as f32 / sr;
            let burst = [0.0f32, 0.011, 0.022]
                .iter()
                .map(|&o| if t >= o { (-(t - o) * 120.0).exp() } else { 0.0 })
                .sum::<f32>();
            let env = burst * 0.6 + (-t * 14.0).exp() * 0.35;
            let (x, y) = (nz.next(), nz.next());
            bl += (x - bl) * a;
            br += (y - br) * a;
            l[i] = (x - bl) * env * 0.9;
            r[i] = (y - br) * env * 0.9;
        }
        hits.push((l, r));

        // 3 Chimbal: ruído bem agudo e curtinho
        let (mut l, mut r) = (vec![0.0f32; len(0.12)], vec![0.0f32; len(0.12)]);
        let (mut ll, mut lr) = (0.0f32, 0.0f32);
        let a = lp_coef(7000.0, sr);
        for i in 0..l.len() {
            let t = i as f32 / sr;
            let env = (-t * 55.0).exp() * 0.5;
            let (x, y) = (nz.next(), nz.next());
            ll += (x - ll) * a;
            lr += (y - lr) * a;
            l[i] = (x - ll) * env;
            r[i] = (y - lr) * env;
        }
        hits.push((l, r));

        // 4 Prato: ruído agudo + parciais metálicas, cauda longa
        let (mut l, mut r) = (vec![0.0f32; len(2.4)], vec![0.0f32; len(2.4)]);
        let (mut ll, mut lr) = (0.0f32, 0.0f32);
        let a = lp_coef(3500.0, sr);
        let partials = [421.0f32, 607.0, 893.0, 1187.0, 1553.0, 2311.0];
        for i in 0..l.len() {
            let t = i as f32 / sr;
            let env = (-t * 2.2).exp() * 0.4 * (t / 0.002).min(1.0);
            let metal: f32 = partials.iter().map(|f| (tau * f * t).sin().signum()).sum::<f32>() / 12.0;
            let (x, y) = (nz.next(), nz.next());
            ll += (x - ll) * a;
            lr += (y - lr) * a;
            l[i] = ((x - ll) + metal * 0.3) * env;
            r[i] = ((y - lr) + metal * 0.3) * env;
        }
        hits.push((l, r));

        // 5 808: grave longo (55 Hz), com um tiquinho de saturação
        let mut k = vec![0.0f32; len(1.6)];
        let mut ph = 0.0f32;
        for (i, s) in k.iter_mut().enumerate() {
            let t = i as f32 / sr;
            let f = 55.0 + 30.0 * (-t * 25.0).exp();
            ph += tau * f / sr;
            let x = ph.sin() * (-t * 2.0).exp() * 1.4;
            *s = x.tanh() * 0.8;
        }
        hits.push((k.clone(), k));

        // 6 Subida (riser): ruído que abre e sobe por 4 s + um tom subindo
        let n = len(4.0);
        let (mut l, mut r) = (vec![0.0f32; n], vec![0.0f32; n]);
        let (mut ll, mut lr) = (0.0f32, 0.0f32);
        let mut ph = 0.0f32;
        for i in 0..n {
            let x01 = i as f32 / n as f32;
            let a = lp_coef(300.0 + 9000.0 * x01 * x01, sr);
            let (x, y) = (nz.next(), nz.next());
            ll += (x - ll) * a;
            lr += (y - lr) * a;
            ph += tau * (180.0 + 900.0 * x01 * x01) / sr;
            let env = x01.powf(1.8) * 0.5 * (1.0 - ((x01 - 0.985) / 0.015).clamp(0.0, 1.0));
            l[i] = (ll + ph.sin() * 0.25) * env;
            r[i] = (lr + ph.sin() * 0.25) * env;
        }
        hits.push((l, r));

        // 7 Impacto: estrondo grave + ruído escuro, cauda de 2,5 s
        let n = len(2.5);
        let (mut l, mut r) = (vec![0.0f32; n], vec![0.0f32; n]);
        let (mut ll, mut lr) = (0.0f32, 0.0f32);
        let a = lp_coef(900.0, sr);
        let mut ph = 0.0f32;
        for i in 0..n {
            let t = i as f32 / sr;
            ph += tau * (38.0 + 60.0 * (-t * 12.0).exp()) / sr;
            let boom = ph.sin() * (-t * 1.6).exp() * 0.9;
            let (x, y) = (nz.next(), nz.next());
            ll += (x - ll) * a;
            lr += (y - lr) * a;
            let env = (-t * 2.8).exp() * 0.9;
            l[i] = (boom + ll * env).tanh() * 0.85;
            r[i] = (boom + lr * env).tanh() * 0.85;
        }
        hits.push((l, r));

        // Cada som no seu volume de pico (bumbo e 808 cheios, chimbal mais baixinho), nunca estourando.
        const PEAKS: [f32; DRUMS] = [0.9, 0.8, 0.75, 0.45, 0.5, 0.9, 0.6, 0.9];
        for (i, (l, r)) in hits.iter_mut().enumerate() {
            let peak = l.iter().chain(r.iter()).fold(0.0f32, |m, x| m.max(x.abs()));
            if peak > 0.0 {
                let g = PEAKS[i] / peak;
                l.iter_mut().chain(r.iter_mut()).for_each(|x| *x *= g);
            }
            // últimos 40 ms somem (sem "clique" no fim do som)
            let fade = (rate as usize / 25).min(l.len());
            let n = l.len();
            for k in 0..fade {
                let g = k as f32 / fade as f32;
                l[n - 1 - k] *= g;
                r[n - 1 - k] *= g;
            }
        }

        Self { hits, rate }
    }
}

/// v0.19: maior duração de um som próprio num DrumPad (o resto do arquivo é ignorado).
pub const DRUM_FILE_MAX_S: usize = 8;

/// Abre o som do Theo para um DrumPad: pula o silêncio do começo (a batida sai na hora),
/// converte para a taxa da placa, guarda no máximo 8 s, não deixa estourar e some no finalzinho.
pub fn load_drum_file(path: &str, rate: u32) -> Result<(Vec<f32>, Vec<f32>), String> {
    let file = std::fs::File::open(path).map_err(|e| format!("Não consegui abrir {path}: {e}"))?;
    let ext = std::path::Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase());
    let clip = crate::pads::decode(Box::new(file), ext.as_deref(), false)?;
    drum_from_clip(&clip, rate)
}

/// v0.20: bateria de verdade que vem dentro do programa (MuseScore General, licença MIT).
/// 0 Bumbo, 1 Caixa, 2 Palmas, 3 Chimbal, 4 Prato, 5 808, 6 Subida, 7 Impacto.
static DRUM_SAMPLES: [&[u8]; DRUMS] = [
    include_bytes!("../assets/sons-livres/drums/00.ogg"),
    include_bytes!("../assets/sons-livres/drums/01.ogg"),
    include_bytes!("../assets/sons-livres/drums/02.ogg"),
    include_bytes!("../assets/sons-livres/drums/03.ogg"),
    include_bytes!("../assets/sons-livres/drums/04.ogg"),
    include_bytes!("../assets/sons-livres/drums/05.ogg"),
    include_bytes!("../assets/sons-livres/drums/06.ogg"),
    include_bytes!("../assets/sons-livres/drums/07.ogg"),
];

fn load_drum_bytes(bytes: &'static [u8], rate: u32) -> Result<(Vec<f32>, Vec<f32>), String> {
    let clip = crate::pads::decode(Box::new(std::io::Cursor::new(bytes)), Some("ogg"), false)?;
    drum_from_clip(&clip, rate)
}

fn drum_from_clip(clip: &crate::pads::PadClip, rate: u32) -> Result<(Vec<f32>, Vec<f32>), String> {
    let frames = clip.frames().min(clip.rate as usize * DRUM_FILE_MAX_S);
    if frames < 32 {
        return Err("o arquivo não tem som".into());
    }
    let d = &clip.data;
    // silêncio do começo (abaixo de ~ −40 dB) fora
    let start = (0..frames)
        .find(|&f| d[2 * f].unsigned_abs() > 320 || d[2 * f + 1].unsigned_abs() > 320)
        .unwrap_or(0)
        .saturating_sub(16);
    let ratio = clip.rate as f64 / rate.max(8000) as f64;
    let n = ((frames - start) as f64 / ratio) as usize;
    let (mut l, mut r) = (Vec::with_capacity(n), Vec::with_capacity(n));
    for k in 0..n {
        let pos = start as f64 + k as f64 * ratio;
        let i = (pos as usize).min(frames - 2);
        let t = (pos - i as f64) as f32;
        let at = |f: usize, c: usize| d[2 * f + c] as f32 / 32768.0;
        l.push(at(i, 0) + (at(i + 1, 0) - at(i, 0)) * t);
        r.push(at(i, 1) + (at(i + 1, 1) - at(i, 1)) * t);
    }
    let peak = l.iter().chain(r.iter()).fold(0.0f32, |m, x| m.max(x.abs()));
    if peak < 0.001 {
        return Err("o arquivo está em silêncio".into());
    }
    if peak > 0.95 {
        let g = 0.95 / peak;
        l.iter_mut().chain(r.iter_mut()).for_each(|x| *x *= g);
    }
    let fade = (rate as usize / 50).min(l.len());
    let len = l.len();
    for k in 0..fade {
        let g = k as f32 / fade as f32;
        l[len - 1 - k] *= g;
        r[len - 1 - k] *= g;
    }
    Ok((l, r))
}

impl DrumKit {
    /// Os 8 sons padrão: bateria gravada de verdade (embutida). Se algum não abrir, fica o sintetizado.
    pub fn build(rate: u32) -> Self {
        let mut kit = Self::build_synth(rate);
        for (i, bytes) in DRUM_SAMPLES.iter().enumerate() {
            if let Ok(hit) = load_drum_bytes(bytes, rate) {
                kit.hits[i] = hit;
            }
        }
        kit
    }

    /// Os sons do programa, trocando os pads que têm arquivo do Theo. Devolve também os erros
    /// (arquivo sumiu, formato estranho...): esses pads ficam com o som do programa.
    pub fn build_with(rate: u32, files: &[(usize, String)]) -> (Self, Vec<(usize, String)>) {
        let mut kit = Self::build(rate);
        let mut errors = Vec::new();
        for (pad, path) in files {
            if *pad >= kit.hits.len() {
                continue;
            }
            match load_drum_file(path, rate) {
                Ok(hit) => kit.hits[*pad] = hit,
                Err(e) => errors.push((*pad, e)),
            }
        }
        (kit, errors)
    }
}

#[derive(Clone, Copy, Default)]
struct DrumVoice {
    pad: usize,
    pos: usize,
    gain: f32,
    active: bool,
}

const DRUM_VOICES: usize = 16;

/// Batidas tocando agora (no motor).
pub struct Drums {
    pub kit: Option<Arc<DrumKit>>,
    v: [DrumVoice; DRUM_VOICES],
    next: usize,
}

impl Default for Drums {
    fn default() -> Self {
        Self {
            kit: None,
            v: [DrumVoice::default(); DRUM_VOICES],
            next: 0,
        }
    }
}

impl Drums {
    pub fn hit(&mut self, pad: usize, vel: u8) {
        if self.kit.as_ref().is_none_or(|k| pad >= k.hits.len()) {
            return;
        }
        // chimbal corta o chimbal anterior (fica natural)
        if pad == 3 {
            for v in &mut self.v {
                if v.active && v.pad == 3 {
                    v.active = false;
                }
            }
        }
        let g = (vel.max(1) as f32 / 127.0).powf(0.8);
        self.v[self.next] = DrumVoice {
            pad,
            pos: 0,
            gain: g,
            active: true,
        };
        self.next = (self.next + 1) % DRUM_VOICES;
    }

    pub fn stop(&mut self) {
        for v in &mut self.v {
            v.active = false;
        }
    }

    pub fn mix(&mut self, l: &mut [f32], r: &mut [f32], gain: f32) {
        let Some(kit) = self.kit.as_ref() else { return };
        for v in &mut self.v {
            if !v.active {
                continue;
            }
            let (hl, hr) = &kit.hits[v.pad];
            let n = l.len().min(hl.len() - v.pos);
            let g = v.gain * gain;
            for i in 0..n {
                l[i] += hl[v.pos + i] * g;
                r[i] += hr[v.pos + i] * g;
            }
            v.pos += n;
            if v.pos >= hl.len() {
                v.active = false;
            }
        }
    }
}

// ---------------------------------------------------------------------------------------------
// Voice FX: coral que segue as notas
// ---------------------------------------------------------------------------------------------

#[derive(Clone, Copy, Default)]
struct CGrain {
    pos: f64,
    step: f64,
    len: u32,
    t: u32,
    gl: f32,
    gr: f32,
}

const CGRAINS: usize = 10;
const CWINDOW: usize = 1024;
const CHOIR_VOICES: usize = 16;
const CHOIR_ATTACK_S: f32 = 0.25;
const CHOIR_RELEASE_S: f32 = 0.9;

#[derive(Clone, Copy)]
struct ChoirVoice {
    active: bool,
    key: u8,
    ratio: f64,
    grains: [CGrain; CGRAINS],
    n: usize,
    next_in: f32,
    playhead: f64,
    env: f32,
    releasing: bool,
    sustained: bool,
    amp: f32,
    rng: u32,
}

impl Default for ChoirVoice {
    fn default() -> Self {
        Self {
            active: false,
            key: 0,
            ratio: 1.0,
            grains: [CGrain::default(); CGRAINS],
            n: 0,
            next_in: 0.0,
            playhead: 0.0,
            env: 0.0,
            releasing: false,
            sustained: false,
            amp: 1.0,
            rng: 1,
        }
    }
}

impl ChoirVoice {
    fn rand(&mut self) -> f32 {
        self.rng ^= self.rng << 13;
        self.rng ^= self.rng >> 17;
        self.rng ^= self.rng << 5;
        (self.rng >> 8) as f32 / (1u32 << 24) as f32
    }
}

/// Coral do Voice FX (no motor).
pub struct VoiceFx {
    pub src: Option<Arc<GrainSource>>,
    voices: [ChoirVoice; CHOIR_VOICES],
    window: Box<[f32; CWINDOW]>,
    pub on: bool,
    pub level: f32,
    pub octave: i8,
    sustain: bool,
    seed: u32,
}

impl Default for VoiceFx {
    fn default() -> Self {
        Self::new()
    }
}

impl VoiceFx {
    /// Cria fora da thread de áudio (aloca a janela).
    pub fn new() -> Self {
        let mut window = Box::new([0f32; CWINDOW]);
        for (i, w) in window.iter_mut().enumerate() {
            let x = i as f32 / (CWINDOW - 1) as f32;
            *w = 0.5 - 0.5 * (std::f32::consts::TAU * x).cos();
        }
        Self {
            src: None,
            voices: [ChoirVoice::default(); CHOIR_VOICES],
            window,
            on: false,
            level: 0.7,
            octave: 0,
            sustain: false,
            seed: 0x9e37_79b9,
        }
    }

    pub fn note_on(&mut self, key: u8, vel: u8, sr: u32) {
        let Some(src) = self.src.as_ref() else { return };
        let note = (key as i32 + 12 * self.octave as i32).clamp(0, 127);
        let ratio = 2f64.powf((note - src.root as i32) as f64 / 12.0) * src.rate as f64 / sr as f64;
        let src_len = src.data.len() as f64;
        // voz livre (ou a que está mais quieta)
        let slot = self
            .voices
            .iter()
            .position(|v| !v.active)
            .unwrap_or_else(|| {
                let mut best = 0;
                for (i, v) in self.voices.iter().enumerate() {
                    if v.releasing && (!self.voices[best].releasing || v.env < self.voices[best].env) {
                        best = i;
                    }
                }
                best
            });
        self.seed = self.seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        self.voices[slot] = ChoirVoice {
            active: true,
            key,
            ratio,
            playhead: src_len * 0.3,
            amp: (vel.max(1) as f32 / 127.0).powf(0.7),
            rng: self.seed | 1,
            ..ChoirVoice::default()
        };
    }

    pub fn note_off(&mut self, key: u8) {
        for v in &mut self.voices {
            if v.active && v.key == key && !v.releasing {
                if self.sustain {
                    v.sustained = true;
                } else {
                    v.releasing = true;
                }
            }
        }
    }

    pub fn set_sustain(&mut self, down: bool) {
        self.sustain = down;
        if !down {
            for v in &mut self.voices {
                if v.sustained {
                    v.sustained = false;
                    v.releasing = true;
                }
            }
        }
    }

    /// Solta tudo (pânico / desligou o Voice FX).
    pub fn all_off(&mut self) {
        for v in &mut self.voices {
            if v.active {
                v.releasing = true;
                v.sustained = false;
            }
        }
    }

    pub fn active_voices(&self) -> usize {
        self.voices.iter().filter(|v| v.active).count()
    }

    /// Soma o coral em `l`/`r` com volume `gain`.
    pub fn mix(&mut self, l: &mut [f32], r: &mut [f32], sr: u32, gain: f32) {
        let Some(src) = self.src.as_ref() else { return };
        let d = &src.data;
        let src_len = d.len() as f64;
        if d.len() < 4096 {
            return;
        }
        let up = 1.0 / (CHOIR_ATTACK_S * sr as f32);
        let down = 1.0 / (CHOIR_RELEASE_S * sr as f32);
        let avg_len = 0.26 * sr as f32;
        let interval = avg_len / 5.0;
        let norm = 0.55 * gain;
        let drift = 0.15 * src.rate as f64 / sr as f64;
        let w = &self.window;
        for v in self.voices.iter_mut().filter(|v| v.active) {
            for i in 0..l.len() {
                v.next_in -= 1.0;
                if v.next_in <= 0.0 && v.n < CGRAINS {
                    let detune = (v.rand() as f64 - 0.5) * 0.006; // ±5 cents
                    let step = v.ratio * (1.0 + detune);
                    let len = (avg_len * (0.75 + 0.5 * v.rand())) as u32;
                    let need = len as f64 * step + 2.0;
                    if need < src_len {
                        let jitter = 0.12 * src.rate as f64;
                        let pos = (v.playhead + (v.rand() as f64 - 0.5) * 2.0 * jitter)
                            .clamp(0.0, src_len - need);
                        let pan = 0.2 + 0.6 * v.rand();
                        v.grains[v.n] = CGrain {
                            pos,
                            step,
                            len: len.max(64),
                            t: 0,
                            gl: (1.0 - pan).sqrt(),
                            gr: pan.sqrt(),
                        };
                        v.n += 1;
                    }
                    v.next_in += interval * (0.7 + 0.6 * v.rand());
                }
                v.playhead += drift;
                if v.playhead > src_len * 0.8 {
                    v.playhead = src_len * 0.2;
                }
                let (mut sl, mut sr_) = (0.0f32, 0.0f32);
                let mut g = 0;
                while g < v.n {
                    let gr = &mut v.grains[g];
                    let idx = gr.pos as usize;
                    let frac = (gr.pos - idx as f64) as f32;
                    let s = d[idx] + (d[idx + 1] - d[idx]) * frac;
                    let x = s * w[(gr.t as usize * (CWINDOW - 1)) / gr.len as usize];
                    sl += x * gr.gl;
                    sr_ += x * gr.gr;
                    gr.pos += gr.step;
                    gr.t += 1;
                    if gr.t >= gr.len {
                        v.n -= 1;
                        v.grains[g] = v.grains[v.n];
                    } else {
                        g += 1;
                    }
                }
                if v.releasing {
                    v.env = (v.env - down).max(0.0);
                } else {
                    v.env = (v.env + up).min(1.0);
                }
                let e = (v.env * std::f32::consts::FRAC_PI_2).sin() * v.amp * norm;
                l[i] += sl * e;
                r[i] += sr_ * e;
            }
            if v.releasing && v.env <= 0.0 {
                v.active = false;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tap_tempo_averages_intervals() {
        assert_eq!(bpm_from_taps(&[0.0, 0.5, 1.0, 1.5]), Some(120.0));
        assert_eq!(bpm_from_taps(&[10.0]), None);
        assert_eq!(bpm_from_taps(&[0.0, 0.75]), Some(80.0));
    }

    #[test]
    fn pulse_shape_has_no_jump_and_dips() {
        assert!(pulse_gain(0.0, 1.0) < 0.05);
        assert!((pulse_gain(0.04, 1.0) - 1.0).abs() < 1e-4);
        assert!(pulse_gain(0.9, 1.0) < 0.1);
        assert_eq!(pulse_gain(0.5, 0.0), 1.0);
    }

    #[test]
    fn arp_patterns_and_octaves() {
        let mut a = Arp::default();
        a.p.octaves = 2;
        for k in [55u8, 59, 62] {
            a.key_down(k, 100);
        }
        let up: Vec<u8> = (0..7).map(|_| a.next_note().unwrap()).collect();
        assert_eq!(up, vec![55, 59, 62, 67, 71, 74, 55]);
        a.p.pattern = 2;
        a.step = 0;
        a.p.octaves = 1;
        let ud: Vec<u8> = (0..6).map(|_| a.next_note().unwrap()).collect();
        assert_eq!(ud, vec![55, 59, 62, 59, 55, 59]);
        // sem segurar: soltou tudo, acabou
        for k in [55u8, 59, 62] {
            a.key_up(k);
        }
        assert!(a.next_note().is_none());
    }

    #[test]
    fn arp_latch_keeps_chord_until_next_one() {
        let mut a = Arp::default();
        a.p.latch = true;
        a.p.octaves = 1;
        a.key_down(60, 90);
        a.key_down(64, 90);
        a.key_up(60);
        a.key_up(64);
        assert_eq!(a.next_note(), Some(60));
        assert_eq!(a.next_note(), Some(64));
        // acorde novo troca o guardado
        a.key_down(67, 90);
        assert_eq!(a.next_note(), Some(67));
        assert_eq!(a.next_note(), Some(67));
    }

    #[test]
    fn drum_kit_sounds_and_stops() {
        let kit = DrumKit::build(48_000);
        assert_eq!(kit.hits.len(), DRUMS);
        for (i, (l, r)) in kit.hits.iter().enumerate() {
            let peak = l.iter().chain(r).fold(0.0f32, |m, x| m.max(x.abs()));
            assert!(peak > 0.05 && peak <= 1.0, "{} pico {peak}", DRUM_NAMES[i]);
            assert!(l.last().unwrap().abs() < 0.05, "{} termina quieto", DRUM_NAMES[i]);
        }
        let mut d = Drums {
            kit: Some(Arc::new(kit)),
            ..Drums::default()
        };
        d.hit(0, 127);
        let (mut l, mut r) = (vec![0.0f32; 512], vec![0.0f32; 512]);
        d.mix(&mut l, &mut r, 1.0);
        assert!(l.iter().any(|x| x.abs() > 0.1));
        for _ in 0..100 {
            l.fill(0.0);
            r.fill(0.0);
            d.mix(&mut l, &mut r, 1.0);
        }
        assert!(l.iter().all(|x| *x == 0.0), "o bumbo acabou");
    }

    #[test]
    fn choir_follows_notes_and_releases() {
        let rate = 48_000;
        // textura de teste: um "aah" de 220 Hz, 3 s, em A3 (57)
        let data: Vec<f32> = (0..rate * 3)
            .map(|i| (std::f32::consts::TAU * 220.0 * i as f32 / rate as f32).sin() * 0.3)
            .collect();
        let mut v = VoiceFx::new();
        v.src = Some(Arc::new(GrainSource { data, rate: rate as u32, root: 57 }));
        v.note_on(60, 100, rate as u32);
        v.note_on(64, 100, rate as u32);
        assert_eq!(v.active_voices(), 2);
        let (mut l, mut r) = (vec![0.0f32; 512], vec![0.0f32; 512]);
        let mut level = 0.0;
        for _ in 0..200 {
            l.fill(0.0);
            r.fill(0.0);
            v.mix(&mut l, &mut r, rate as u32, 1.0);
            level = (l.iter().map(|x| x * x).sum::<f32>() / 512.0).sqrt();
        }
        assert!(level > 0.02, "coral soando: {level}");
        v.set_sustain(true);
        v.note_off(60);
        v.note_off(64);
        for _ in 0..200 {
            v.mix(&mut l, &mut r, rate as u32, 1.0);
        }
        assert_eq!(v.active_voices(), 2, "pedal segura");
        v.set_sustain(false);
        for _ in 0..200 {
            l.fill(0.0);
            r.fill(0.0);
            v.mix(&mut l, &mut r, rate as u32, 1.0);
        }
        assert_eq!(v.active_voices(), 0, "soltou o pedal: o coral some");
    }
}

// ---------------------------------------------------------------------------------------------
// v0.23: Filtro de "abrir e fechar" o som (passa-baixa) e Swell (entra/sai suave)
// ---------------------------------------------------------------------------------------------

/// Onde o filtro está: 0 = bem fechado (escuro, ~150 Hz) … 1 = todo aberto (som normal).
pub fn filter_hz(amount: f32) -> f32 {
    let a = amount.clamp(0.0, 1.0);
    150.0 * (20_000.0f32 / 150.0).powf(a)
}

/// Passa-baixa estéreo (biquad) com o corte mudando suave (sem "degrau" quando gira o knob).
/// Ressonância 0..1 = um pouco de "brilho" perto do corte, típico da abertura de filtro.
pub struct Lowpass {
    z: [[f32; 2]; 2],
    c: [f32; 5],
    /// Corte atual (Hz), andando devagar até o pedido.
    now_hz: f32,
}

impl Default for Lowpass {
    fn default() -> Self {
        Self {
            z: [[0.0; 2]; 2],
            c: [1.0, 0.0, 0.0, 0.0, 0.0],
            now_hz: 20_000.0,
        }
    }
}

impl Lowpass {
    fn coefs(&mut self, hz: f32, q: f32, sr: f32) {
        let w = std::f32::consts::TAU * hz.min(sr * 0.45) / sr;
        let (s, c) = (w.sin(), w.cos());
        let alpha = s / (2.0 * q);
        let a0 = 1.0 + alpha;
        let b1 = (1.0 - c) / a0;
        self.c = [b1 * 0.5, b1, b1 * 0.5, (-2.0 * c) / a0, (1.0 - alpha) / a0];
    }

    /// Está todo aberto (dá para pular o filtro, gastando nada)?
    pub fn is_open(&self, target_hz: f32) -> bool {
        target_hz >= 19_000.0 && self.now_hz >= 19_000.0
    }

    pub fn process(&mut self, l: &mut [f32], r: &mut [f32], target_hz: f32, resonance: f32, sr: u32) {
        if self.is_open(target_hz) {
            self.now_hz = 20_000.0;
            self.z = [[0.0; 2]; 2];
            return;
        }
        let sr = sr as f32;
        let q = 0.707 + resonance.clamp(0.0, 1.0) * 3.3;
        // ~60 ms para chegar no corte novo (em escala de oitavas: gira natural)
        let k = 1.0 - (-32.0 / (0.06 * sr)).exp();
        let mut i = 0;
        while i < l.len() {
            let n = (l.len() - i).min(32);
            let ln = self.now_hz.ln() + (target_hz.max(20.0).ln() - self.now_hz.ln()) * k;
            self.now_hz = ln.exp();
            self.coefs(self.now_hz, q, sr);
            let [b0, b1, b2, a1, a2] = self.c;
            for (ch, buf) in [&mut l[i..i + n], &mut r[i..i + n]].into_iter().enumerate() {
                let z = &mut self.z[ch];
                for x in buf.iter_mut() {
                    let y = b0 * *x + z[0];
                    z[0] = b1 * *x - a1 * y + z[1];
                    z[1] = b2 * *x - a2 * y;
                    *x = y;
                }
            }
            i += n;
        }
    }
}

/// Swell: volume que entra/sai devagar (curva suave, igual à do pad).
pub struct Swell {
    pub gain: f32,
    target: f32,
    rate: f32,
}

impl Default for Swell {
    fn default() -> Self {
        Self {
            gain: 1.0,
            target: 1.0,
            rate: 0.0,
        }
    }
}

impl Swell {
    /// `inn` = entrar (vai a 1) ou sair (vai a 0) em `secs` segundos (0 = na hora).
    pub fn set(&mut self, inn: bool, secs: f32, sr: u32) {
        self.target = if inn { 1.0 } else { 0.0 };
        if secs <= 0.0 {
            self.gain = self.target;
            self.rate = 0.0;
        } else {
            self.rate = 1.0 / (secs * sr as f32);
        }
    }

    pub fn moving(&self) -> bool {
        self.gain != self.target
    }

    pub fn is_full(&self) -> bool {
        self.gain >= 1.0 && self.target >= 1.0
    }

    /// Enche `out` com o ganho de cada amostra (curva de seno) e anda a rampa.
    pub fn fill(&mut self, out: &mut [f32]) {
        for g in out.iter_mut() {
            if self.gain < self.target {
                self.gain = (self.gain + self.rate).min(self.target);
            } else if self.gain > self.target {
                self.gain = (self.gain - self.rate).max(self.target);
            }
            *g = (self.gain * std::f32::consts::FRAC_PI_2).sin();
        }
    }
}

#[cfg(test)]
mod filtro_tests {
    use super::*;

    fn tone(hz: f32, n: usize) -> Vec<f32> {
        (0..n).map(|i| (std::f32::consts::TAU * hz * i as f32 / 48_000.0).sin() * 0.5).collect()
    }

    fn level(x: &[f32]) -> f32 {
        (x.iter().map(|v| v * v).sum::<f32>() / x.len() as f32).sqrt()
    }

    #[test]
    fn closed_filter_darkens_and_open_is_untouched() {
        let mut f = Lowpass::default();
        let (mut l, mut r) = (tone(3000.0, 48_000), tone(3000.0, 48_000));
        f.process(&mut l, &mut r, filter_hz(0.0), 0.0, 48_000);
        assert!(level(&l[24_000..]) < 0.01, "agudo some com o filtro fechado: {}", level(&l[24_000..]));
        let mut g = Lowpass::default();
        let (mut l2, mut r2) = (tone(3000.0, 4800), tone(3000.0, 4800));
        let before = l2.clone();
        g.process(&mut l2, &mut r2, filter_hz(1.0), 0.0, 48_000);
        assert_eq!(l2, before, "aberto = som igual");
        // grave passa mesmo fechado
        let mut h = Lowpass::default();
        let (mut l3, mut r3) = (tone(80.0, 48_000), tone(80.0, 48_000));
        h.process(&mut l3, &mut r3, filter_hz(0.0), 0.0, 48_000);
        assert!(level(&l3[24_000..]) > 0.25);
    }

    #[test]
    fn swell_goes_out_and_in_smoothly() {
        let mut s = Swell::default();
        let mut buf = vec![0.0f32; 48_000];
        s.set(false, 2.0, 48_000);
        s.fill(&mut buf);
        assert!(buf[0] > 0.99 && buf[47_999] > 0.6 && buf[47_999] < 0.8, "no meio do caminho em 1 s");
        s.fill(&mut buf);
        assert!(buf[47_999] < 1e-6, "saiu em 2 s");
        let jumps = buf.windows(2).map(|w| (w[1] - w[0]).abs()).fold(0.0f32, f32::max);
        assert!(jumps < 0.001, "sem degrau");
        s.set(true, 0.0, 48_000);
        s.fill(&mut buf[..1]);
        assert_eq!(buf[0], 1.0);
    }
}
