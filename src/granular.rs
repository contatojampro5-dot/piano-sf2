//! "Pad Vivo": motor granular + reverb de igreja.
//!
//! Granular: pega uma gravação curta (a "textura", numa nota só) e toca milhares de pedacinhos
//! ("grãos") dela ao mesmo tempo, cada um num lugar, afinação e lado do estéreo um pouco diferentes.
//! Assim a textura vira uma nuvem de som contínua, em qualquer tom, que nunca se repete igual.
//!
//! Tudo aqui roda na thread de áudio: nada aloca memória depois de criado.

use std::sync::Arc;

/// Textura-base: mono, numa nota conhecida (`root`, número MIDI).
pub struct GrainSource {
    pub data: Vec<f32>,
    pub rate: u32,
    pub root: u8,
}

impl GrainSource {
    /// A partir de um áudio estéreo intercalado (i16), misturando os canais.
    pub fn from_stereo_i16(stereo: &[i16], rate: u32, root: u8) -> Self {
        let data = stereo
            .chunks(2)
            .map(|c| (c[0] as f32 + *c.get(1).unwrap_or(&c[0]) as f32) / 65536.0)
            .collect();
        Self { data, rate, root }
    }
}

/// Ajustes ao vivo do Pad Vivo (0..1).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GrainParams {
    /// Brilho: 0 = bem escuro, 1 = aberto.
    pub bright: f32,
    /// Shimmer: quanto de oitavas acima ("brilho celestial").
    pub shimmer: f32,
    /// Movimento: grãos mais curtos e espalhados (1) ou longos e calmos (0).
    pub motion: f32,
}

impl Default for GrainParams {
    fn default() -> Self {
        Self {
            bright: 0.6,
            shimmer: 0.35,
            motion: 0.45,
        }
    }
}

#[derive(Clone, Copy, Default)]
struct Grain {
    pos: f64,
    step: f64,
    len: u32,
    t: u32,
    gl: f32,
    gr: f32,
}

const MAX_GRAINS: usize = 72;
const WINDOW: usize = 1024;

/// Uma nuvem de grãos tocando um tom.
pub struct GranularVoice {
    pub src: Arc<GrainSource>,
    key: u8,
    grains: [Grain; MAX_GRAINS],
    active: usize,
    window: Box<[f32; WINDOW]>,
    next_in: f32,
    playhead: f64,
    rng: u32,
    // fade de entrada/saída (igual ao pad normal)
    gain: f32,
    target: f32,
    rate: f32,
    // filtro de brilho (um polo por lado)
    lp_l: f32,
    lp_r: f32,
}

pub const FADE_IN_S: f32 = 3.0;
pub const FADE_OUT_S: f32 = 3.5;

impl GranularVoice {
    /// `midi` = nota central do pad (ex.: 48 = C3). Cria na thread da tela (aloca a janela).
    pub fn new(src: Arc<GrainSource>, midi: u8, sample_rate: u32, seed: u32) -> Self {
        let mut window = Box::new([0f32; WINDOW]);
        for (i, w) in window.iter_mut().enumerate() {
            // janela de Hann: o grão entra e sai suave
            let x = i as f32 / (WINDOW - 1) as f32;
            *w = 0.5 - 0.5 * (std::f32::consts::TAU * x).cos();
        }
        let len = src.data.len() as f64;
        Self {
            src,
            key: midi,
            grains: [Grain::default(); MAX_GRAINS],
            active: 0,
            window,
            next_in: 0.0,
            playhead: len * 0.3,
            rng: seed | 1,
            gain: 0.0,
            target: 1.0,
            rate: 1.0 / (FADE_IN_S * sample_rate as f32),
            lp_l: 0.0,
            lp_r: 0.0,
        }
    }

    pub fn fade_out(&mut self, sample_rate: u32) {
        self.target = 0.0;
        self.rate = 1.0 / (FADE_OUT_S * sample_rate as f32);
    }

    pub fn finished(&self) -> bool {
        self.target == 0.0 && self.gain <= 0.0
    }

    fn rand(&mut self) -> f32 {
        self.rng ^= self.rng << 13;
        self.rng ^= self.rng >> 17;
        self.rng ^= self.rng << 5;
        (self.rng >> 8) as f32 / (1u32 << 24) as f32
    }

    fn spawn(&mut self, p: &GrainParams, sr: u32) {
        if self.active >= MAX_GRAINS {
            return;
        }
        // Intervalo do grão: a nota, quinta, oitavas (sem terça: serve maior e menor).
        let r = self.rand();
        let shimmer = p.shimmer.clamp(0.0, 1.0);
        let semis: f32 = if r < 0.34 {
            0.0
        } else if r < 0.52 {
            7.0
        } else if r < 0.70 {
            12.0
        } else if r < 0.80 {
            -12.0
        } else if r < 0.80 + 0.12 * shimmer {
            19.0
        } else if r < 0.80 + 0.20 * shimmer {
            24.0
        } else {
            0.0
        };
        let detune = (self.rand() - 0.5) * 0.12; // ±6 cents
        let ratio =
            2f64.powf(((self.key as f32 - self.src.root as f32 + semis + detune) / 12.0) as f64);
        let step = ratio * self.src.rate as f64 / sr as f64;
        // Grãos longos (calmo) ou curtos (mexido).
        let m = p.motion.clamp(0.0, 1.0);
        let dur_s = 0.55 - 0.35 * m + 0.25 * self.rand();
        let len = (dur_s * sr as f32) as u32;
        let src_len = self.src.data.len() as f64;
        let need = len as f64 * step + 2.0;
        if need >= src_len {
            return;
        }
        let jitter = (0.15 + 0.85 * m as f64) * self.src.rate as f64 * 0.8;
        let mut pos = self.playhead + (self.rand() as f64 - 0.5) * 2.0 * jitter;
        pos = pos.clamp(0.0, src_len - need);
        // Lado do estéreo e volume: oitava de cima mais baixinha.
        let pan = self.rand();
        let amp = if semis > 12.0 {
            0.55
        } else if semis < 0.0 {
            0.8
        } else {
            1.0
        };
        let (gl, gr) = ((1.0 - pan).sqrt() * amp, pan.sqrt() * amp);
        self.grains[self.active] = Grain {
            pos,
            step,
            len: len.max(64),
            t: 0,
            gl,
            gr,
        };
        self.active += 1;
    }

    /// Soma a nuvem em `l`/`r` (com o fade e o brilho).
    pub fn mix(&mut self, l: &mut [f32], r: &mut [f32], p: &GrainParams, sr: u32) {
        let src_len = self.src.data.len();
        if src_len < 1024 {
            return;
        }
        // Densidade: grãos por segundo; o volume compensa quantos se sobrepõem.
        let m = p.motion.clamp(0.0, 1.0);
        let per_sec = 14.0 + 22.0 * m;
        let avg_len = 0.55 - 0.35 * m + 0.125;
        let norm = 1.7 / (per_sec * avg_len).sqrt();
        let interval = sr as f32 / per_sec;
        // Brilho: frequência de corte de ~900 Hz a ~9 kHz.
        let fc = 900.0 * 10f32.powf(p.bright.clamp(0.0, 1.0));
        let a = 1.0 - (-std::f32::consts::TAU * fc / sr as f32).exp();
        // A "agulha" anda bem devagar pela textura (ida e volta), para o som mudar de cor sozinho.
        let drift = 0.18 * self.src.rate as f64 / sr as f64;

        for i in 0..l.len() {
            self.next_in -= 1.0;
            if self.next_in <= 0.0 {
                let p2 = *p;
                self.spawn(&p2, sr);
                self.next_in += interval * (0.6 + 0.8 * self.rand());
            }
            self.playhead += drift;
            if self.playhead > src_len as f64 * 0.8 {
                self.playhead = src_len as f64 * 0.2;
            }
            let (mut sl, mut sr_) = (0.0f32, 0.0f32);
            let mut g = 0;
            while g < self.active {
                let gr = &mut self.grains[g];
                let idx = gr.pos as usize;
                let frac = (gr.pos - idx as f64) as f32;
                let d = &self.src.data;
                let s = d[idx] + (d[idx + 1] - d[idx]) * frac;
                let w = self.window[(gr.t as usize * (WINDOW - 1)) / gr.len as usize];
                let v = s * w;
                sl += v * gr.gl;
                sr_ += v * gr.gr;
                gr.pos += gr.step;
                gr.t += 1;
                if gr.t >= gr.len {
                    // grão acabou: troca pelo último (sem alocar)
                    self.active -= 1;
                    self.grains[g] = self.grains[self.active];
                } else {
                    g += 1;
                }
            }
            if self.gain != self.target {
                if self.gain < self.target {
                    self.gain = (self.gain + self.rate).min(self.target);
                } else {
                    self.gain = (self.gain - self.rate).max(self.target);
                }
            }
            let fade = (self.gain * std::f32::consts::FRAC_PI_2).sin() * norm;
            self.lp_l += (sl - self.lp_l) * a;
            self.lp_r += (sr_ - self.lp_r) * a;
            l[i] += self.lp_l * fade;
            r[i] += self.lp_r * fade;
        }
    }
}

// ---------------------------------------------------------------------------------------------
// Reverb de igreja: rede de 8 atrasos com realimentação (FDN), difusores e "ar" que escurece.
// ---------------------------------------------------------------------------------------------

struct Delay {
    buf: Vec<f32>,
    pos: usize,
    len: usize,
}

impl Delay {
    fn new(max: usize) -> Self {
        Self {
            buf: vec![0.0; max.max(2)],
            pos: 0,
            len: max.max(2),
        }
    }
    fn set_len(&mut self, len: usize) {
        self.len = len.clamp(2, self.buf.len());
        self.pos %= self.len;
    }
    #[inline]
    fn read(&self) -> f32 {
        self.buf[self.pos]
    }
    #[inline]
    fn write_advance(&mut self, v: f32) {
        self.buf[self.pos] = v;
        self.pos += 1;
        if self.pos >= self.len {
            self.pos = 0;
        }
    }
}

struct Allpass {
    d: Delay,
    g: f32,
}

impl Allpass {
    #[inline]
    fn process(&mut self, x: f32) -> f32 {
        let z = self.d.read();
        let v = x + self.g * z;
        self.d.write_advance(v);
        z - self.g * v
    }
}

const LINES: usize = 8;
/// Tamanhos (ms) dos 8 atrasos: números "quebrados" para não ressoar.
const LINE_MS: [f32; LINES] = [43.1, 51.7, 59.3, 67.9, 73.3, 83.9, 97.1, 107.3];
const DIFF_MS: [f32; 4] = [4.77, 3.59, 12.73, 9.31];
const MAX_RATE: f32 = 96_000.0;

pub struct ChurchReverb {
    lines: Vec<Delay>,
    fb: [f32; LINES],
    damp_state: [f32; LINES],
    damp: f32,
    diff_l: Vec<Allpass>,
    diff_r: Vec<Allpass>,
    pre_l: Delay,
    pre_r: Delay,
    sample_rate: u32,
}

impl ChurchReverb {
    /// Cria já com memória para até 96 kHz (não aloca depois).
    pub fn new(sample_rate: u32) -> Self {
        let max = |ms: f32| (ms * 0.001 * MAX_RATE) as usize + 4;
        let mut r = Self {
            lines: LINE_MS.iter().map(|&ms| Delay::new(max(ms))).collect(),
            fb: [0.0; LINES],
            damp_state: [0.0; LINES],
            damp: 0.3,
            diff_l: DIFF_MS
                .iter()
                .map(|&ms| Allpass {
                    d: Delay::new(max(ms)),
                    g: 0.7,
                })
                .collect(),
            diff_r: DIFF_MS
                .iter()
                .map(|&ms| Allpass {
                    d: Delay::new(max(ms * 1.07)),
                    g: 0.7,
                })
                .collect(),
            pre_l: Delay::new(max(40.0)),
            pre_r: Delay::new(max(40.0)),
            sample_rate: 0,
        };
        r.set_sample_rate(sample_rate);
        r
    }

    /// Ajusta tamanhos para a taxa da placa (tempo de reverb ~3,8 s, igreja grande).
    pub fn set_sample_rate(&mut self, sr: u32) {
        let sr = (sr as f32).min(MAX_RATE);
        let rt60 = 3.8;
        for (i, d) in self.lines.iter_mut().enumerate() {
            let n = (LINE_MS[i] * 0.001 * sr) as usize;
            d.set_len(n);
            self.fb[i] = 10f32.powf(-3.0 * n as f32 / (rt60 * sr));
        }
        for (i, a) in self.diff_l.iter_mut().enumerate() {
            a.d.set_len((DIFF_MS[i] * 0.001 * sr) as usize);
        }
        for (i, a) in self.diff_r.iter_mut().enumerate() {
            a.d.set_len((DIFF_MS[i] * 1.07 * 0.001 * sr) as usize);
        }
        self.pre_l.set_len((0.028 * sr) as usize);
        self.pre_r.set_len((0.031 * sr) as usize);
        // "ar" da igreja: agudos morrem antes dos graves (~5 kHz)
        self.damp = 1.0 - (-std::f32::consts::TAU * 5000.0 / sr).exp();
        self.sample_rate = sr as u32;
    }

    /// Recebe a entrada (`in_l`/`in_r`) e SOMA o som do reverb em `out_l`/`out_r`.
    #[allow(clippy::needless_range_loop)]
    pub fn process(
        &mut self,
        in_l: &[f32],
        in_r: &[f32],
        out_l: &mut [f32],
        out_r: &mut [f32],
        wet: f32,
    ) {
        if wet <= 0.0001 {
            return;
        }
        let g = wet * 0.32;
        for i in 0..in_l.len() {
            // pré-atraso e difusão (espalha o "ataque")
            let pl = self.pre_l.read();
            self.pre_l.write_advance(in_l[i]);
            let pr = self.pre_r.read();
            self.pre_r.write_advance(in_r[i]);
            let mut xl = pl;
            for a in &mut self.diff_l {
                xl = a.process(xl);
            }
            let mut xr = pr;
            for a in &mut self.diff_r {
                xr = a.process(xr);
            }
            // lê as 8 linhas e mistura com Hadamard (energia se espalha igual entre todas)
            let mut v = [0f32; LINES];
            for k in 0..LINES {
                v[k] = self.lines[k].read();
            }
            hadamard8(&mut v);
            let ol = v[0] + v[2] + v[4] + v[6];
            let or = v[1] + v[3] + v[5] + v[7];
            for k in 0..LINES {
                let inj = if k % 2 == 0 { xl } else { xr };
                // amortecimento (agudos somem antes)
                self.damp_state[k] += (v[k] - self.damp_state[k]) * self.damp;
                let y = self.damp_state[k] * self.fb[k] + inj * 0.5;
                self.lines[k].write_advance(y);
            }
            out_l[i] += ol * g;
            out_r[i] += or * g;
        }
    }
}

/// Mistura de Hadamard 8×8 (normalizada): rápida e sem "cor".
#[inline]
fn hadamard8(v: &mut [f32; 8]) {
    let mut h = 1;
    while h < 8 {
        let mut i = 0;
        while i < 8 {
            for j in i..i + h {
                let (a, b) = (v[j], v[j + h]);
                v[j] = a + b;
                v[j + h] = a - b;
            }
            i += h * 2;
        }
        h *= 2;
    }
    let n = 1.0 / (8f32).sqrt();
    for x in v.iter_mut() {
        *x *= n;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sine_source() -> Arc<GrainSource> {
        let rate = 44_100;
        let f = 130.81; // C3
        let data = (0..rate * 4)
            .map(|i| (i as f32 / rate as f32 * f * std::f32::consts::TAU).sin() * 0.8)
            .collect();
        Arc::new(GrainSource {
            data,
            rate: rate as u32,
            root: 48,
        })
    }

    fn rms(x: &[f32]) -> f32 {
        (x.iter().map(|v| v * v).sum::<f32>() / x.len().max(1) as f32).sqrt()
    }

    #[test]
    fn grain_cloud_sounds_and_stays_sane() {
        let sr = 48_000;
        let mut v = GranularVoice::new(sine_source(), 55, sr, 7);
        let p = GrainParams::default();
        let (mut l, mut r) = (vec![0.0; 256], vec![0.0; 256]);
        let mut all = Vec::new();
        for _ in 0..(sr as usize * 4 / 256) {
            l.fill(0.0);
            r.fill(0.0);
            v.mix(&mut l, &mut r, &p, sr);
            all.extend_from_slice(&l);
            all.extend_from_slice(&r);
        }
        assert!(all.iter().all(|x| x.is_finite()));
        let tail = &all[all.len() - sr as usize..];
        let level = rms(tail);
        assert!(level > 0.05 && level < 1.0, "nível da nuvem: {level}");
        // fade out até sumir
        v.fade_out(sr);
        for _ in 0..(sr as usize * 4 / 256) {
            l.fill(0.0);
            r.fill(0.0);
            v.mix(&mut l, &mut r, &p, sr);
        }
        assert!(v.finished());
    }

    #[test]
    fn reverb_has_long_tail_and_does_not_explode() {
        let sr = 48_000;
        let mut rev = ChurchReverb::new(sr);
        let n = 256;
        let mut tail = Vec::new();
        for block in 0..(sr as usize * 6 / n) {
            let mut inl = vec![0.0; n];
            let inr = vec![0.0; n];
            if block == 0 {
                inl[0] = 1.0;
            }
            let (mut ol, mut or) = (vec![0.0; n], vec![0.0; n]);
            rev.process(&inl, &inr, &mut ol, &mut or, 1.0);
            tail.push(rms(&ol) + rms(&or));
            assert!(ol
                .iter()
                .chain(or.iter())
                .all(|x| x.is_finite() && x.abs() < 2.0));
        }
        let at = |s: f32| tail[(s * sr as f32 / n as f32) as usize];
        assert!(at(0.5) > at(2.5), "decai");
        assert!(at(1.5) > 1e-5, "cauda de igreja: ainda soa em 1,5 s");
        assert!(at(5.8) < at(0.5) * 0.05, "morre até ~6 s");
    }
}

/// Prévia em WAV (para ouvir fora do programa): `PIANO_SF2_PREVIEW=pasta cargo test --release --no-default-features preview -- --ignored`
#[cfg(test)]
mod preview {
    use super::*;

    #[test]
    #[ignore]
    fn preview_vivo_textures() {
        let Ok(dir) = std::env::var("PIANO_SF2_PREVIEW") else {
            return;
        };
        let sr = 44_100u32;
        for n in 0..3 {
            let src = crate::pads::load_texture(n, None).unwrap();
            let mut rev = ChurchReverb::new(sr);
            // Dó por 10 s, depois Sol (troca suave), 10 s.
            let mut voices = vec![GranularVoice::new(
                src.clone(),
                crate::pads::key_midi(0),
                sr,
                3,
            )];
            let p = GrainParams::default();
            let block = 256;
            let mut out: Vec<i16> = Vec::new();
            let total = sr as usize * 20 / block;
            for b in 0..total {
                if b == total / 2 {
                    for v in &mut voices {
                        v.fade_out(sr);
                    }
                    voices.push(GranularVoice::new(
                        src.clone(),
                        crate::pads::key_midi(7),
                        sr,
                        9,
                    ));
                }
                let (mut l, mut r) = (vec![0.0; block], vec![0.0; block]);
                for v in &mut voices {
                    v.mix(&mut l, &mut r, &p, sr);
                }
                let (mut ol, mut or) = (l.clone(), r.clone());
                rev.process(&l, &r, &mut ol, &mut or, 0.45);
                for i in 0..block {
                    out.push((ol[i].clamp(-1.0, 1.0) * 30000.0) as i16);
                    out.push((or[i].clamp(-1.0, 1.0) * 30000.0) as i16);
                }
            }
            let path = format!("{dir}/vivo_{n}.wav");
            let mut f = Vec::new();
            let data_len = (out.len() * 2) as u32;
            f.extend(b"RIFF");
            f.extend((36 + data_len).to_le_bytes());
            f.extend(b"WAVEfmt ");
            f.extend(16u32.to_le_bytes());
            f.extend(1u16.to_le_bytes());
            f.extend(2u16.to_le_bytes());
            f.extend(sr.to_le_bytes());
            f.extend((sr * 4).to_le_bytes());
            f.extend(4u16.to_le_bytes());
            f.extend(16u16.to_le_bytes());
            f.extend(b"data");
            f.extend(data_len.to_le_bytes());
            for s in out {
                f.extend(s.to_le_bytes());
            }
            std::fs::write(path, f).unwrap();
        }
    }
}
