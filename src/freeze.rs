//! v0.16 — "Congelar": o Theo toca um acorde (ex.: Sol), aperta ❄ e aquele som fica soando sozinho,
//! como um pad, enquanto ele troca os timbres e toca por cima. Apertou de novo: some devagar.
//!
//! Como funciona:
//! 1. `CaptureRing`: a thread de áudio guarda sem parar os últimos segundos do som do SF2
//!    (só as camadas; pads e o próprio congelado ficam de fora). Sem trava e sem alocar.
//! 2. Ao congelar, a tela copia esse trecho (`snapshot`), escolhe a parte que está soando
//!    (`pick_region`: pula o "ataque" do acorde e o silêncio do fim) e cria uma `FreezeVoice`.
//! 3. `FreezeVoice`: granular simples, sem mudar a altura (é o mesmo acorde): pedacinhos longos
//!    do trecho, em lugares sorteados, com janelas suaves. O volume acompanha o do acorde tocado.

use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};
use std::sync::Arc;
#[cfg(not(test))]
use std::sync::OnceLock;

/// Tamanho da memória (quadros, potência de 2): ~5,4 s a 48 kHz, ~2,7 s a 96 kHz.
const RING: usize = 1 << 18;

/// Memória circular do som das camadas (f32 guardado como bits em atômicos: sem trava).
pub struct CaptureRing {
    l: Box<[AtomicU32]>,
    r: Box<[AtomicU32]>,
    /// Total de quadros já escritos (a posição é `written % RING`).
    written: AtomicUsize,
}

impl CaptureRing {
    fn new() -> Self {
        let mk = || (0..RING).map(|_| AtomicU32::new(0)).collect::<Vec<_>>().into_boxed_slice();
        Self {
            l: mk(),
            r: mk(),
            written: AtomicUsize::new(0),
        }
    }

    /// Thread de áudio: guarda um bloco.
    pub fn push(&self, l: &[f32], r: &[f32]) {
        let mut w = self.written.load(Ordering::Relaxed);
        for (a, b) in l.iter().zip(r) {
            let i = w & (RING - 1);
            self.l[i].store(a.to_bits(), Ordering::Relaxed);
            self.r[i].store(b.to_bits(), Ordering::Relaxed);
            w = w.wrapping_add(1);
        }
        self.written.store(w, Ordering::Release);
    }

    /// Fora do áudio: copia os últimos `frames` quadros (o mais antigo primeiro).
    pub fn snapshot(&self, frames: usize) -> (Vec<f32>, Vec<f32>) {
        let w = self.written.load(Ordering::Acquire);
        let n = frames.min(RING - 4096).min(w);
        let start = w.wrapping_sub(n);
        let mut l = Vec::with_capacity(n);
        let mut r = Vec::with_capacity(n);
        for k in 0..n {
            let i = start.wrapping_add(k) & (RING - 1);
            l.push(f32::from_bits(self.l[i].load(Ordering::Relaxed)));
            r.push(f32::from_bits(self.r[i].load(Ordering::Relaxed)));
        }
        (l, r)
    }

    #[cfg(test)]
    pub fn clear(&self) {
        self.written.store(0, Ordering::Release);
    }
}

#[cfg(not(test))]
static RING_CELL: OnceLock<CaptureRing> = OnceLock::new();

/// A memória do som (criada na primeira chamada: chamar antes de o áudio começar, em `Engine::new`).
#[cfg(not(test))]
pub fn ring() -> &'static CaptureRing {
    RING_CELL.get_or_init(CaptureRing::new)
}

// Nos testes, cada teste (uma thread) tem a sua memória: os testes rodam juntos e um não pode
// "ouvir" o som do outro. No programa de verdade é uma só (áudio escreve, tela lê).
#[cfg(test)]
thread_local! {
    static TEST_RING: &'static CaptureRing = Box::leak(Box::new(CaptureRing::new()));
}

#[cfg(test)]
pub fn ring() -> &'static CaptureRing {
    TEST_RING.with(|r| *r)
}

/// Trecho congelado (estéreo).
pub struct FreezeSource {
    pub l: Vec<f32>,
    pub r: Vec<f32>,
    pub rate: u32,
    /// Volume médio (RMS) do trecho: o congelado sai com esse mesmo volume.
    pub rms: f32,
}

fn rms(l: &[f32], r: &[f32]) -> f32 {
    if l.is_empty() {
        return 0.0;
    }
    let s: f32 = l.iter().zip(r).map(|(a, b)| a * a + b * b).sum();
    (s / (2 * l.len()) as f32).sqrt()
}

/// Escolhe a parte do áudio que está soando: termina no último trecho com som, começa depois do
/// "ataque" mais forte (para não ficar repetindo a martelada da tecla) e tem de 0,4 a 1,6 s.
/// `None` = quase silêncio (nada para congelar).
pub fn pick_region(l: &[f32], r: &[f32], rate: u32) -> Option<(usize, usize)> {
    let block = (rate as usize / 50).max(64); // 20 ms
    let blocks = l.len() / block;
    if blocks < 4 {
        return None;
    }
    let levels: Vec<f32> = (0..blocks)
        .map(|b| rms(&l[b * block..(b + 1) * block], &r[b * block..(b + 1) * block]))
        .collect();
    let (peak_b, peak) = levels
        .iter()
        .copied()
        .enumerate()
        .fold((0, 0.0f32), |acc, (i, v)| if v > acc.1 { (i, v) } else { acc });
    if peak < 0.002 {
        return None;
    }
    // Termina no som mais recente que ainda não é silêncio (o que o Theo está ouvindo ao apertar).
    // Piano cai rápido depois da martelada: não dá para exigir que esteja perto do pico.
    let floor = (peak * 0.03).max(0.0008);
    let end_b = levels.iter().rposition(|&v| v >= floor)? + 1;
    let max_len = (rate as f32 * 1.6) as usize;
    let min_len = (rate as f32 * 0.4) as usize;
    let end = end_b * block;
    let after_attack = (peak_b * block + rate as usize * 15 / 100).min(end);
    let mut start = end.saturating_sub(max_len).max(after_attack);
    if end - start < min_len {
        start = end.saturating_sub(min_len);
    }
    if end - start < 2048 {
        return None;
    }
    Some((start, end))
}

impl FreezeSource {
    /// Pega o trecho que está soando do que a memória guardou. `None` = nada tocando.
    pub fn capture(rate: u32) -> Option<Self> {
        let (l, r) = ring().snapshot(rate as usize * 3);
        Self::from_audio(&l, &r, rate)
    }

    pub fn from_audio(l: &[f32], r: &[f32], rate: u32) -> Option<Self> {
        let (a, b) = pick_region(l, r, rate)?;
        // Volume do congelado = o volume do finalzinho (últimos 0,3 s), o que se ouvia ao apertar.
        let last = b.saturating_sub(rate as usize * 3 / 10).max(a);
        let rms = rms(&l[last..b], &r[last..b]);
        let (l, r) = (l[a..b].to_vec(), r[a..b].to_vec());
        Some(Self { l, r, rate, rms })
    }

    pub fn seconds(&self) -> f32 {
        self.l.len() as f32 / self.rate as f32
    }
}

#[derive(Clone, Copy, Default)]
struct Grain {
    pos: f64,
    step: f64,
    len: u32,
    t: u32,
}

const MAX_GRAINS: usize = 24;
const WINDOW: usize = 2048;
/// Entrada rápida: o som continua "colado" no acorde que estava soando.
pub const FREEZE_IN_S: f32 = 0.25;
/// Saída: some devagar quando desliga.
pub const FREEZE_OUT_S: f32 = 3.0;

/// O som congelado tocando (criado fora da thread de áudio).
pub struct FreezeVoice {
    pub src: Arc<FreezeSource>,
    grains: [Grain; MAX_GRAINS],
    active: usize,
    window: Box<[f32; WINDOW]>,
    next_in: f32,
    rng: u32,
    gain: f32,
    target: f32,
    rate: f32,
    /// Volume medido da nuvem de grãos (para igualar ao acorde).
    level: f32,
}

impl FreezeVoice {
    pub fn new(src: Arc<FreezeSource>, sample_rate: u32, seed: u32) -> Self {
        let mut window = Box::new([0f32; WINDOW]);
        for (i, w) in window.iter_mut().enumerate() {
            let x = i as f32 / (WINDOW - 1) as f32;
            *w = 0.5 - 0.5 * (std::f32::consts::TAU * x).cos();
        }
        let level = src.rms;
        Self {
            src,
            grains: [Grain::default(); MAX_GRAINS],
            active: 0,
            window,
            next_in: 0.0,
            rng: seed | 1,
            gain: 0.0,
            target: 1.0,
            rate: 1.0 / (FREEZE_IN_S * sample_rate as f32),
            level,
        }
    }

    pub fn fade_out(&mut self, sample_rate: u32, seconds: f32) {
        self.target = 0.0;
        self.rate = 1.0 / (seconds.max(0.01) * sample_rate as f32);
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

    fn spawn(&mut self, sr: u32) {
        if self.active >= MAX_GRAINS {
            return;
        }
        let src_len = self.src.l.len() as f64;
        // mesma altura; só um tiquinho de desafinação para soar vivo (±3 cents)
        let detune = (self.rand() as f64 - 0.5) * 0.06;
        let step = 2f64.powf(detune / 12.0) * self.src.rate as f64 / sr as f64;
        // grãos longos: 45%..75% do trecho, até 0,9 s (calmo, sem "metralhadora")
        let max_s = (self.src.seconds() * 0.75).min(0.9);
        let dur_s = max_s * (0.6 + 0.4 * self.rand());
        let len = ((dur_s * sr as f32) as u32).max(256);
        let need = len as f64 * step + 2.0;
        if need >= src_len {
            return;
        }
        let pos = self.rand() as f64 * (src_len - need);
        self.grains[self.active] = Grain {
            pos,
            step,
            len,
            t: 0,
        };
        self.active += 1;
    }

    /// Soma o congelado em `l`/`r`.
    pub fn mix(&mut self, l: &mut [f32], r: &mut [f32], sr: u32) {
        if self.src.l.len() < 2048 {
            return;
        }
        // ~4 grãos sobrepostos o tempo todo
        let max_s = (self.src.seconds() * 0.75).min(0.9);
        let avg_len = max_s * 0.8;
        let interval = sr as f32 * avg_len / 4.0;
        // medidor do volume (constante de ~0,4 s)
        let k = 1.0 / (0.4 * sr as f32);
        let want = self.src.rms;
        for i in 0..l.len() {
            self.next_in -= 1.0;
            if self.next_in <= 0.0 {
                self.spawn(sr);
                self.next_in += interval * (0.7 + 0.6 * self.rand());
            }
            let (mut sl, mut srr) = (0.0f32, 0.0f32);
            let mut g = 0;
            while g < self.active {
                let gr = &mut self.grains[g];
                let idx = gr.pos as usize;
                let frac = (gr.pos - idx as f64) as f32;
                let (dl, dr) = (&self.src.l, &self.src.r);
                let w = self.window[(gr.t as usize * (WINDOW - 1)) / gr.len as usize];
                sl += (dl[idx] + (dl[idx + 1] - dl[idx]) * frac) * w;
                srr += (dr[idx] + (dr[idx + 1] - dr[idx]) * frac) * w;
                gr.pos += gr.step;
                gr.t += 1;
                if gr.t >= gr.len {
                    self.active -= 1;
                    self.grains[g] = self.grains[self.active];
                } else {
                    g += 1;
                }
            }
            // iguala o volume ao do acorde tocado (devagar, sem "bombar")
            let p = 0.5 * (sl * sl + srr * srr);
            self.level += (p.sqrt() - self.level) * k;
            let norm = (want / self.level.max(1e-5)).clamp(0.25, 4.0);
            if self.gain != self.target {
                if self.gain < self.target {
                    self.gain = (self.gain + self.rate).min(self.target);
                } else {
                    self.gain = (self.gain - self.rate).max(self.target);
                }
            }
            let fade = (self.gain * std::f32::consts::FRAC_PI_2).sin() * norm;
            l[i] += sl * fade;
            r[i] += srr * fade;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Acorde de Sol "de mentira": G3 B3 D4 com ataque forte que vai caindo.
    fn chord(rate: u32, secs: f32) -> (Vec<f32>, Vec<f32>) {
        let n = (rate as f32 * secs) as usize;
        let freqs = [196.0f32, 246.94, 293.66];
        let mut l = vec![0.0; n];
        for (i, s) in l.iter_mut().enumerate() {
            let t = i as f32 / rate as f32;
            let env = 0.3 * (-t * 0.8).exp();
            *s = freqs.iter().map(|f| (std::f32::consts::TAU * f * t).sin()).sum::<f32>() * env;
        }
        (l.clone(), l)
    }

    #[test]
    fn silence_has_nothing_to_freeze() {
        let z = vec![0.0f32; 48_000 * 2];
        assert!(FreezeSource::from_audio(&z, &z, 48_000).is_none());
    }

    #[test]
    fn region_skips_attack_and_silent_tail() {
        let rate = 48_000;
        let (mut l, mut r) = chord(rate, 2.0);
        // soltou as teclas: 1 s de silêncio no fim
        l.extend(std::iter::repeat(0.0).take(rate as usize));
        r.extend(std::iter::repeat(0.0).take(rate as usize));
        let (a, b) = pick_region(&l, &r, rate).unwrap();
        assert!(a >= rate as usize / 10, "pula o ataque ({a})");
        assert!(b <= 2 * rate as usize + 960, "não pega o silêncio ({b})");
        assert!(b - a >= rate as usize * 4 / 10);
    }

    #[test]
    fn frozen_sound_keeps_going_at_the_same_volume_and_fades_out() {
        let rate = 48_000;
        let (l, r) = chord(rate, 2.0);
        let src = Arc::new(FreezeSource::from_audio(&l, &r, rate).unwrap());
        let want = src.rms;
        let mut v = FreezeVoice::new(src, rate, 7);
        let mut bl = vec![0.0f32; 512];
        let mut br = vec![0.0f32; 512];
        // 20 s depois ainda está soando, com volume parecido com o do acorde
        let mut last = 0.0;
        for blk in 0..(20 * rate as usize / 512) {
            bl.fill(0.0);
            br.fill(0.0);
            v.mix(&mut bl, &mut br, rate);
            if blk * 512 > 18 * rate as usize {
                last = rms(&bl, &br);
            }
        }
        assert!(last > want * 0.5 && last < want * 2.0, "volume {last} vs {want}");
        // desligou: some em ~3 s
        v.fade_out(rate, FREEZE_OUT_S);
        for _ in 0..(4 * rate as usize / 512) {
            bl.fill(0.0);
            br.fill(0.0);
            v.mix(&mut bl, &mut br, rate);
        }
        assert!(v.finished());
        assert!(rms(&bl, &br) < 1e-6);
    }

    #[test]
    fn ring_keeps_the_last_seconds() {
        let ring = CaptureRing::new();
        let a: Vec<f32> = (0..1000).map(|i| i as f32).collect();
        ring.push(&a, &a);
        let (l, _) = ring.snapshot(10);
        assert_eq!(l, (990..1000).map(|i| i as f32).collect::<Vec<_>>());
        // dá a volta na memória sem quebrar
        let big = vec![0.5f32; RING + 123];
        ring.push(&big, &big);
        let (l, _) = ring.snapshot(5000);
        assert_eq!(l.len(), 5000);
        assert!(l.iter().all(|&x| x == 0.5));
    }
}

// ---------------------------------------------------------------------------------------------
// v0.24: o último Freeze fica guardado num arquivo (volta com o botão ↺, até depois de reabrir)
// ---------------------------------------------------------------------------------------------

/// Onde o último Freeze fica salvo (%APPDATA%\PianoSF2\ultimo-freeze.wav). Nos testes, numa pasta temporária.
pub fn saved_path() -> std::path::PathBuf {
    #[cfg(test)]
    {
        // um arquivo por teste (os testes rodam juntos)
        let t: String = format!("{:?}", std::thread::current().id()).chars().filter(|c| c.is_ascii_digit()).collect();
        std::env::temp_dir().join(format!("noahpad-teste-freeze-{}-{t}.wav", std::process::id()))
    }
    #[cfg(not(test))]
    {
        let base = std::env::var_os("APPDATA")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(std::env::temp_dir);
        base.join("PianoSF2").join("ultimo-freeze.wav")
    }
}

impl FreezeSource {
    /// Trecho já escolhido (volume = o do finalzinho, igual ao `from_audio`).
    pub fn from_exact(l: Vec<f32>, r: Vec<f32>, rate: u32) -> Option<Self> {
        if l.len() < 2048 || l.len() != r.len() {
            return None;
        }
        let last = l.len().saturating_sub(rate as usize * 3 / 10);
        let rms = rms(&l[last..], &r[last..]);
        if rms < 1e-5 {
            return None;
        }
        Some(Self { l, r, rate, rms })
    }

    /// Grava como WAV 16 bits estéreo.
    pub fn save_wav(&self, path: &std::path::Path) -> std::io::Result<()> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let data_len = (self.l.len() * 4) as u32;
        let mut b: Vec<u8> = Vec::with_capacity(44 + data_len as usize);
        b.extend(b"RIFF");
        b.extend((36 + data_len).to_le_bytes());
        b.extend(b"WAVEfmt ");
        b.extend(16u32.to_le_bytes());
        b.extend(1u16.to_le_bytes());
        b.extend(2u16.to_le_bytes());
        b.extend(self.rate.to_le_bytes());
        b.extend((self.rate * 4).to_le_bytes());
        b.extend(4u16.to_le_bytes());
        b.extend(16u16.to_le_bytes());
        b.extend(b"data");
        b.extend(data_len.to_le_bytes());
        for (a, c) in self.l.iter().zip(&self.r) {
            b.extend(((a.clamp(-1.0, 1.0) * 32767.0) as i16).to_le_bytes());
            b.extend(((c.clamp(-1.0, 1.0) * 32767.0) as i16).to_le_bytes());
        }
        std::fs::write(path, b)
    }

    /// Abre o último Freeze salvo (None = não tem / arquivo estragado).
    pub fn load_wav(path: &std::path::Path) -> Option<Self> {
        let file = std::fs::File::open(path).ok()?;
        let clip = crate::pads::decode(Box::new(file), Some("wav"), false).ok()?;
        let n = clip.frames();
        let l = (0..n).map(|f| clip.data[2 * f] as f32 / 32768.0).collect();
        let r = (0..n).map(|f| clip.data[2 * f + 1] as f32 / 32768.0).collect();
        Self::from_exact(l, r, clip.rate)
    }
}

#[cfg(test)]
mod saved_tests {
    use super::*;

    #[test]
    fn last_freeze_is_saved_and_comes_back() {
        let rate = 48_000;
        let l: Vec<f32> = (0..rate as usize).map(|i| (i as f32 * 0.03).sin() * 0.4).collect();
        let src = FreezeSource::from_exact(l.clone(), l.clone(), rate).unwrap();
        let path = std::env::temp_dir().join(format!("noahpad-freeze-wav-{}.wav", std::process::id()));
        src.save_wav(&path).unwrap();
        let back = FreezeSource::load_wav(&path).unwrap();
        assert_eq!(back.l.len(), src.l.len());
        assert!((back.rms - src.rms).abs() < 0.01);
        assert!(FreezeSource::load_wav(std::path::Path::new("C:/nao/existe.wav")).is_none());
        let _ = std::fs::remove_file(path);
    }
}
