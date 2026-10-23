//! Motor de som: um sintetizador SF2 só, com uma camada por canal MIDI (0..7).
//!
//! Roda dentro da thread de áudio. Nada aqui pode travar: os comandos chegam por
//! um canal sem bloqueio e o sintetizador antigo é devolvido para ser apagado fora daqui.

use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Instant;

use crossbeam_channel::{Receiver, Sender};
use rustysynth::{SoundFont, Synthesizer, SynthesizerSettings};

use crate::model::LAYERS;
use crate::pads::{PadClip, PadVoice};

/// O que a thread de áudio devolve para ser apagado fora dela (liberar memória pode demorar).
pub enum Garbage {
    Synth(Box<Synthesizer>),
    Pad(Arc<PadClip>),
}

/// Máximo de pads soando juntos (o que entra + os que estão sumindo).
const MAX_PADS: usize = 4;

/// Comandos para o motor (vindos do piano, da mesinha ou da tela).
pub enum Cmd {
    NoteOn {
        key: u8,
        vel: u8,
    },
    NoteOff {
        key: u8,
    },
    /// Control Change do piano (pedal, modulação...) repassado a todas as camadas.
    Cc {
        num: u8,
        val: u8,
    },
    PitchBend {
        lsb: u8,
        msb: u8,
    },
    SetPreset {
        layer: usize,
        bank: i32,
        patch: i32,
    },
    SetAudible {
        layer: usize,
        on: bool,
    },
    SetVolume {
        layer: usize,
        value: f32,
    },
    SetReverb {
        layer: usize,
        value: f32,
    },
    SetMaster(f32),
    SetTranspose(i32),
    Panic,
    /// Novo sintetizador (SF2 novo ou taxa de amostragem nova).
    Replace(Box<Synthesizer>),
    /// Taxa de amostragem da placa de som (os pads precisam dela mesmo sem SF2).
    SetSampleRate(u32),
    /// Toca um pad (o que estiver tocando some devagar).
    PadPlay(Arc<PadClip>),
    PadStop,
    PadVolume(f32),
}

/// Números que a tela mostra (atualizados pela thread de áudio).
#[derive(Default)]
pub struct Stats {
    /// Uso do tempo do buffer em % × 10.
    pub cpu_permille: AtomicU32,
    /// Pico de saída × 1000.
    pub peak_milli: AtomicU32,
    pub voices: AtomicUsize,
    /// Callbacks que passaram do tempo (possível estalo).
    pub overruns: AtomicU32,
}

const KILL_DELAY_MS: f32 = 60.0;

pub struct Engine {
    synth: Option<Box<Synthesizer>>,
    rx: Receiver<Cmd>,
    garbage: Sender<Garbage>,
    pub stats: Arc<Stats>,
    sample_rate: u32,
    audible: [bool; LAYERS],
    /// Contagem regressiva (em amostras) para calar de vez uma camada desligada.
    kill_in: [i64; LAYERS],
    /// Tecla física → tecla tocada (com transposição na hora em que foi apertada).
    held: [Option<u8>; 128],
    transpose: i32,
    master: f32,
    // Estado guardado para reaplicar quando o sintetizador é trocado.
    presets: [Option<(i32, i32)>; LAYERS],
    volumes: [f32; LAYERS],
    reverbs: [f32; LAYERS],
    left: Vec<f32>,
    right: Vec<f32>,
    peak_decay: f32,
    pads: Vec<PadVoice>,
    /// Volume dos pads (alvo e valor suavizado).
    pad_vol: f32,
    pad_vol_now: f32,
}

impl Engine {
    pub fn new(rx: Receiver<Cmd>, garbage: Sender<Garbage>, stats: Arc<Stats>) -> Self {
        Self {
            synth: None,
            rx,
            garbage,
            stats,
            sample_rate: 48_000,
            audible: [false; LAYERS],
            kill_in: [-1; LAYERS],
            held: [None; 128],
            transpose: 0,
            master: 0.64,
            presets: [None; LAYERS],
            volumes: [0.8; LAYERS],
            reverbs: [0.25; LAYERS],
            left: vec![0.0; 8192],
            right: vec![0.0; 8192],
            peak_decay: 0.0,
            pads: Vec::with_capacity(MAX_PADS + 1),
            pad_vol: 0.49,
            pad_vol_now: 0.49,
        }
    }

    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    /// Cria um sintetizador (fora da thread de áudio: aloca memória).
    pub fn build_synth(
        sf: &Arc<SoundFont>,
        sample_rate: u32,
        polyphony: usize,
    ) -> Option<Box<Synthesizer>> {
        let mut settings = SynthesizerSettings::new(sample_rate as i32);
        settings.maximum_polyphony = polyphony.clamp(8, 256);
        settings.block_size = 64;
        settings.enable_reverb_and_chorus = true;
        Synthesizer::new(sf, &settings).ok().map(Box::new)
    }

    fn apply_layer(&mut self, i: usize) {
        let Some(s) = self.synth.as_mut() else { return };
        let ch = i as i32;
        if let Some((bank, patch)) = self.presets[i] {
            s.set_channel_preset(ch, bank, patch);
        }
        let v = (self.volumes[i].clamp(0.0, 1.0) * 16383.0) as i32;
        s.process_midi_message(ch, 0xB0, 0x07, v >> 7);
        s.process_midi_message(ch, 0xB0, 0x27, v & 0x7F);
        s.process_midi_message(
            ch,
            0xB0,
            0x5B,
            (self.reverbs[i].clamp(0.0, 1.0) * 127.0) as i32,
        );
        s.process_midi_message(ch, 0xB0, 0x5D, 0);
        s.process_midi_message(ch, 0xB0, 0x0B, if self.audible[i] { 127 } else { 0 });
    }

    fn handle(&mut self, cmd: Cmd) {
        match cmd {
            Cmd::Replace(new) => {
                if let Some(old) = self.synth.replace(new) {
                    let _ = self.garbage.try_send(Garbage::Synth(old));
                }
                self.sample_rate = self
                    .synth
                    .as_ref()
                    .map(|s| s.get_sample_rate() as u32)
                    .unwrap_or(48_000);
                self.held = [None; 128];
                if let Some(s) = self.synth.as_mut() {
                    s.set_master_volume(self.master);
                }
                for i in 0..LAYERS {
                    self.apply_layer(i);
                }
            }
            Cmd::NoteOn { key, vel } => {
                let k = (key as i32 + self.transpose).clamp(0, 127) as u8;
                // Se a mesma tecla ainda estava presa (sem note off), solta antes.
                if let Some(prev) = self.held[key as usize].take() {
                    self.note_off_all_layers(prev);
                }
                self.held[key as usize] = Some(k);
                if let Some(s) = self.synth.as_mut() {
                    for i in 0..LAYERS {
                        if self.audible[i] && self.presets[i].is_some() {
                            s.note_on(i as i32, k as i32, vel as i32);
                        }
                    }
                }
            }
            Cmd::NoteOff { key } => {
                let k = self.held[key as usize].take().unwrap_or(key);
                self.note_off_all_layers(k);
            }
            Cmd::Cc { num, val } => {
                // Volume/expressão/reverb do piano não mexem na mistura: quem manda é a mesa.
                if matches!(num, 0x00 | 0x07 | 0x0B | 0x20 | 0x27 | 0x2B | 0x5B | 0x5D) {
                    return;
                }
                if let Some(s) = self.synth.as_mut() {
                    for i in 0..LAYERS {
                        s.process_midi_message(i as i32, 0xB0, num as i32, val as i32);
                    }
                }
                if num == 0x78 || num == 0x7B {
                    self.held = [None; 128];
                }
            }
            Cmd::PitchBend { lsb, msb } => {
                if let Some(s) = self.synth.as_mut() {
                    for i in 0..LAYERS {
                        s.process_midi_message(i as i32, 0xE0, lsb as i32, msb as i32);
                    }
                }
            }
            Cmd::SetPreset { layer, bank, patch } if layer < LAYERS => {
                self.presets[layer] = Some((bank, patch));
                if let Some(s) = self.synth.as_mut() {
                    // Notas antigas terminam com o timbre antigo; as novas já saem com o novo.
                    s.set_channel_preset(layer as i32, bank, patch);
                }
            }
            Cmd::SetAudible { layer, on } if layer < LAYERS => {
                if self.audible[layer] == on {
                    return;
                }
                self.audible[layer] = on;
                if let Some(s) = self.synth.as_mut() {
                    s.process_midi_message(layer as i32, 0xB0, 0x0B, if on { 127 } else { 0 });
                }
                self.kill_in[layer] = if on {
                    -1
                } else {
                    (self.sample_rate as f32 * KILL_DELAY_MS / 1000.0) as i64
                };
            }
            Cmd::SetVolume { layer, value } if layer < LAYERS => {
                self.volumes[layer] = value;
                if let Some(s) = self.synth.as_mut() {
                    let v = (value.clamp(0.0, 1.0) * 16383.0) as i32;
                    s.process_midi_message(layer as i32, 0xB0, 0x07, v >> 7);
                    s.process_midi_message(layer as i32, 0xB0, 0x27, v & 0x7F);
                }
            }
            Cmd::SetReverb { layer, value } if layer < LAYERS => {
                self.reverbs[layer] = value;
                if let Some(s) = self.synth.as_mut() {
                    s.process_midi_message(
                        layer as i32,
                        0xB0,
                        0x5B,
                        (value.clamp(0.0, 1.0) * 127.0) as i32,
                    );
                }
            }
            Cmd::SetMaster(v) => {
                // Curva suave: fader em 80% ≈ volume 0,64.
                self.master = (v.clamp(0.0, 1.0)).powi(2);
                if let Some(s) = self.synth.as_mut() {
                    s.set_master_volume(self.master);
                }
            }
            Cmd::SetSampleRate(sr) => {
                if self.synth.is_none() {
                    self.sample_rate = sr.max(8000);
                }
            }
            Cmd::PadPlay(clip) => {
                let sr = self.sample_rate;
                for v in &mut self.pads {
                    v.fade_out(sr);
                }
                // Lista cheia: o pad mais antigo sai na hora (já estava sumindo).
                if self.pads.len() >= MAX_PADS {
                    let old = self.pads.remove(0);
                    let _ = self.garbage.try_send(Garbage::Pad(old.clip));
                }
                self.pads.push(PadVoice::new(clip, sr));
            }
            Cmd::PadStop => {
                let sr = self.sample_rate;
                for v in &mut self.pads {
                    v.fade_out(sr);
                }
            }
            Cmd::PadVolume(v) => self.pad_vol = v.clamp(0.0, 1.0).powi(2),
            Cmd::SetTranspose(t) => self.transpose = t.clamp(-24, 24),
            Cmd::Panic => {
                if let Some(s) = self.synth.as_mut() {
                    s.note_off_all(true);
                    for i in 0..LAYERS {
                        s.process_midi_message(i as i32, 0xB0, 0x40, 0);
                        s.process_midi_message(i as i32, 0xE0, 0, 64);
                        s.process_midi_message(i as i32, 0xB0, 0x01, 0);
                    }
                }
                self.held = [None; 128];
            }
            _ => {}
        }
    }

    fn note_off_all_layers(&mut self, key: u8) {
        if let Some(s) = self.synth.as_mut() {
            for i in 0..LAYERS {
                s.note_off(i as i32, key as i32);
            }
        }
    }

    /// Preenche o buffer de saída intercalado (L, R, L, R...).
    pub fn render_interleaved(&mut self, out: &mut [f32], channels: usize) {
        let started = Instant::now();
        while let Ok(cmd) = self.rx.try_recv() {
            self.handle(cmd);
        }
        let channels = channels.max(1);
        let frames = out.len() / channels;

        let mut peak = 0.0f32;
        let mut done = 0;
        let sr = self.sample_rate;
        while done < frames {
            let n = (frames - done).min(self.left.len());
            let (l, r) = (&mut self.left[..n], &mut self.right[..n]);
            match self.synth.as_mut() {
                Some(synth) => synth.render(l, r),
                None => {
                    l.fill(0.0);
                    r.fill(0.0);
                }
            }
            if !self.pads.is_empty() {
                // Volume do pad muda suave (sem degrau quando mexe o fader).
                self.pad_vol_now += (self.pad_vol - self.pad_vol_now) * 0.2;
                let vol = self.pad_vol_now * self.master;
                for v in &mut self.pads {
                    v.mix(l, r, vol, sr);
                }
            }
            for f in 0..n {
                let a = soft_clip(l[f]);
                let b = soft_clip(r[f]);
                peak = peak.max(a.abs()).max(b.abs());
                let o = &mut out[(done + f) * channels..(done + f + 1) * channels];
                o[0] = a;
                if channels > 1 {
                    o[1] = b;
                    for x in &mut o[2..] {
                        *x = 0.0;
                    }
                }
            }
            done += n;
        }

        // Pads que já sumiram: devolve para apagar fora daqui.
        let mut i = 0;
        while i < self.pads.len() {
            if self.pads[i].finished() {
                let v = self.pads.remove(i);
                let _ = self.garbage.try_send(Garbage::Pad(v.clip));
            } else {
                i += 1;
            }
        }

        let Some(synth) = self.synth.as_mut() else {
            self.stats
                .peak_milli
                .store((peak * 1000.0) as u32, Ordering::Relaxed);
            return;
        };

        // Camadas desligadas: depois do "fade", corta as vozes que sobraram (pedal preso etc.).
        for i in 0..LAYERS {
            if self.kill_in[i] >= 0 {
                self.kill_in[i] -= frames as i64;
                if self.kill_in[i] < 0 {
                    synth.note_off_all_channel(i as i32, true);
                }
            }
        }

        // Estatísticas para a tela.
        self.peak_decay = (self.peak_decay * 0.9).max(peak);
        self.stats
            .peak_milli
            .store((self.peak_decay * 1000.0) as u32, Ordering::Relaxed);
        self.stats
            .voices
            .store(synth.active_voice_count(), Ordering::Relaxed);
        let budget = frames as f32 / self.sample_rate as f32;
        if budget > 0.0 {
            let used = started.elapsed().as_secs_f32() / budget;
            let prev = self.stats.cpu_permille.load(Ordering::Relaxed) as f32 / 1000.0;
            let smooth = if used > prev {
                used
            } else {
                prev * 0.97 + used * 0.03
            };
            self.stats
                .cpu_permille
                .store((smooth * 1000.0) as u32, Ordering::Relaxed);
            if used > 0.95 {
                self.stats.overruns.fetch_add(1, Ordering::Relaxed);
            }
        }
    }
}

/// Limitador suave: quase transparente até ~0,6 e nunca passa de 1,0.
#[inline]
pub fn soft_clip(x: f32) -> f32 {
    let x = x.clamp(-1.5, 1.5);
    x - x * x * x / 6.75
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn soft_clip_limits() {
        assert!((soft_clip(0.1) - 0.1).abs() < 0.001);
        assert!(soft_clip(5.0) <= 1.0 && soft_clip(-5.0) >= -1.0);
        assert!((soft_clip(1.5) - 1.0).abs() < 1e-6);
    }
}
