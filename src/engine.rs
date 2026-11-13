//! Motor de som: um sintetizador SF2 só, com uma camada por canal MIDI (0..7).
//!
//! Roda dentro da thread de áudio. Nada aqui pode travar: os comandos chegam por
//! um canal sem bloqueio e o sintetizador antigo é devolvido para ser apagado fora daqui.

use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Instant;

use crossbeam_channel::{Receiver, Sender};
use rustysynth::{SoundFont, Synthesizer, SynthesizerSettings};

use crate::efeitos::{self, Arp, ArpParams, DrumKit, Drums, PulseParams, Seq, SeqParams, VoiceFx};
use crate::freeze::FreezeVoice;
use crate::granular::{ChurchReverb, GrainParams, GranularVoice};
use crate::model::LAYERS;
use crate::pads::{PadClip, PadVoice};

/// O que a thread de áudio devolve para ser apagado fora dela (liberar memória pode demorar).
pub enum Garbage {
    Synth(Box<Synthesizer>),
    Pad(Arc<PadClip>),
    Grain(Box<GranularVoice>),
    Freeze(Box<FreezeVoice>),
}

/// Congelados soando juntos (o atual + os que estão sumindo).
const MAX_FREEZE: usize = 4;

/// Máximo de pads soando juntos (o que entra + os que estão sumindo).
const MAX_PADS: usize = 8;

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
    /// v0.29: Cutoff (brilho) da camada: 1 = aberto (normal) … 0 = bem fechado. Vai como CC74.
    SetCutoff {
        layer: usize,
        value: f32,
    },
    SetMaster(f32),
    SetTranspose(i32),
    /// v0.15: quantos semitons o pitch bend alcança (1..12).
    PitchRange(u8),
    /// v0.16: toca o som congelado (o anterior, se houver, some devagar). Criado fora daqui.
    FreezePlay(Box<FreezeVoice>),
    /// v0.16: desliga o congelado (some em `segundos`).
    FreezeStop(f32),
    /// v0.17: andamento (BPM) do PulsePad e do Arpejador.
    Tempo(f32),
    /// v0.17: "bater o tempo": o pulso e o arpejo recomeçam agora (ficam no tempo da batida).
    TempoSync,
    Pulse(PulseParams),
    Arp(ArpParams),
    /// v0.17: os sons dos DrumPads (gerados fora daqui).
    DrumKit(Arc<DrumKit>),
    Drum {
        pad: usize,
        vel: u8,
    },
    DrumVolume(f32),
    /// v0.33: Sequencer (batida em loop no andamento do Tempo).
    Seq(SeqParams),
    /// v0.17: textura do coral do Voice FX.
    VoiceFxSource(Arc<crate::granular::GrainSource>),
    /// v0.31: textura do Voice FX (None = volta às vozes nota por nota). Criada fora daqui.
    VoiceFxTexture(Option<Arc<crate::efeitos::VoiceTexture>>),
    /// v0.31: tom das texturas (0 = C … 11 = B).
    VoiceFxKey(u8),
    VoiceFx {
        on: bool,
        level: f32,
        octave: i8,
    },
    /// v0.23: filtro de abrir/fechar (0 fechado … 1 aberto), ressonância 0..1,
    /// `all` = tudo menos a bateria (senão só o fundo: pads e congelado), `with_swell` = o swell abre o filtro junto.
    Filter {
        amount: f32,
        resonance: f32,
        all: bool,
        with_swell: bool,
    },
    /// v0.23: swell — entrar (true) ou sair (false) em `secs` segundos.
    Swell {
        inn: bool,
        secs: f32,
    },
    Panic,
    /// Novo sintetizador (SF2 novo ou taxa de amostragem nova).
    Replace(Box<Synthesizer>),
    /// Taxa de amostragem da placa de som (os pads precisam dela mesmo sem SF2).
    SetSampleRate(u32),
    /// Toca um pad (o que estiver tocando some devagar).
    /// Pad em loop com a velocidade de leitura (1 = altura original).
    PadPlay(Arc<PadClip>, f32),
    /// v0.24: segundo pad junto (não apaga o primeiro), na camada 1.
    PadLayer2(Arc<PadClip>, f32),
    /// v0.24: volume de cada camada de pad (principal, segundo) — o mix entre os dois.
    PadMix(f32, f32),
    PadStop,
    PadVolume(f32),
    /// Toca um Pad Vivo (granular); o que estiver tocando some devagar. A voz é criada fora daqui.
    GrainPlay(Box<GranularVoice>),
    GrainParams(GrainParams),
    /// Reverb de igreja: quanto nos pads e quanto nas camadas do SF2 (0..1).
    Church {
        pads: f32,
        sf2: f32,
    },
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
    /// v0.33: passo do Sequencer que acabou de tocar + 1 (0 = parado). A tela acende a coluna e os pads.
    pub seq_pos: AtomicU32,
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
    /// Alcance do pitch bend em semitons (RPN 0), reaplicado em cada camada.
    pitch_range: u8,
    // Estado guardado para reaplicar quando o sintetizador é trocado.
    presets: [Option<(i32, i32)>; LAYERS],
    volumes: [f32; LAYERS],
    reverbs: [f32; LAYERS],
    cutoffs: [f32; LAYERS],
    left: Vec<f32>,
    right: Vec<f32>,
    peak_decay: f32,
    pads: Vec<PadVoice>,
    /// Volume dos pads (alvo e valor suavizado).
    pad_vol: f32,
    pad_vol_now: f32,
    /// v0.24: volume de cada camada (pedido e suavizado).
    pad_layer: [f32; 2],
    pad_layer_now: [f32; 2],
    // Box de propósito: a voz é criada fora da thread de áudio e só "passa a mão" para cá.
    #[allow(clippy::vec_box)]
    grains: Vec<Box<GranularVoice>>,
    grain_params: GrainParams,
    /// v0.16: sons congelados (o último é o atual; os outros estão sumindo).
    #[allow(clippy::vec_box)]
    freezes: Vec<Box<FreezeVoice>>,
    // v0.17: efeitos
    /// Relógio (amostras desde que o motor começou).
    clock: u64,
    bpm: f32,
    pulse: PulseParams,
    /// Onde o pulso começou a contar (bater o tempo recomeça aqui).
    pulse_origin: u64,
    arp: Arp,
    /// Quando toca a próxima nota do arpejo (no relógio).
    arp_next: f64,
    drums: Drums,
    drum_vol: f32,
    /// v0.33: Sequencer e quando toca o próximo passo (no relógio).
    seq: Seq,
    seq_next: f64,
    voicefx: VoiceFx,
    fz_l: Vec<f32>,
    fz_r: Vec<f32>,
    // v0.23: filtro e swell
    filt_amount: f32,
    filt_res: f32,
    filt_all: bool,
    filt_with_swell: bool,
    lp_fz: efeitos::Lowpass,
    lp_pad: efeitos::Lowpass,
    lp_all: efeitos::Lowpass,
    swell: efeitos::Swell,
    swell_buf: Vec<f32>,
    reverb: ChurchReverb,
    church_pads: f32,
    church_sf2: f32,
    // buffers do "barramento" dos pads e da entrada do reverb (criados antes, sem alocar no áudio)
    pad_l: Vec<f32>,
    pad_r: Vec<f32>,
    rev_l: Vec<f32>,
    rev_r: Vec<f32>,
}

impl Engine {
    pub fn new(rx: Receiver<Cmd>, garbage: Sender<Garbage>, stats: Arc<Stats>) -> Self {
        // A memória do Congelar é criada aqui (fora da thread de áudio, que só escreve nela).
        let _ = crate::freeze::ring();
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
            pitch_range: 2,
            presets: [None; LAYERS],
            volumes: [0.8; LAYERS],
            reverbs: [0.25; LAYERS],
            cutoffs: [1.0; LAYERS],
            left: vec![0.0; 8192],
            right: vec![0.0; 8192],
            peak_decay: 0.0,
            pads: Vec::with_capacity(MAX_PADS + 1),
            pad_vol: 0.49,
            pad_vol_now: 0.49,
            pad_layer: [1.0, 0.0],
            pad_layer_now: [1.0, 0.0],
            grains: Vec::with_capacity(MAX_PADS + 1),
            grain_params: GrainParams::default(),
            freezes: Vec::with_capacity(MAX_FREEZE + 1),
            clock: 0,
            bpm: 75.0,
            pulse: PulseParams::default(),
            pulse_origin: 0,
            arp: Arp::default(),
            arp_next: 0.0,
            drums: Drums::default(),
            drum_vol: 0.8,
            seq: Seq::default(),
            seq_next: 0.0,
            voicefx: VoiceFx::new(),
            fz_l: vec![0.0; 8192],
            fz_r: vec![0.0; 8192],
            filt_amount: 1.0,
            filt_res: 0.25,
            filt_all: false,
            filt_with_swell: true,
            lp_fz: efeitos::Lowpass::default(),
            lp_pad: efeitos::Lowpass::default(),
            lp_all: efeitos::Lowpass::default(),
            swell: efeitos::Swell::default(),
            swell_buf: vec![1.0; 8192],
            reverb: ChurchReverb::new(48_000),
            church_pads: 0.4,
            church_sf2: 0.0,
            pad_l: vec![0.0; 8192],
            pad_r: vec![0.0; 8192],
            rev_l: vec![0.0; 8192],
            rev_r: vec![0.0; 8192],
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
        s.process_midi_message(ch, 0xB0, 0x4A, (self.cutoffs[i].clamp(0.0, 1.0) * 127.0).round() as i32);
        s.process_midi_message(ch, 0xB0, 0x0B, if self.audible[i] { 127 } else { 0 });
        Self::send_pitch_range(s, ch, self.pitch_range);
    }

    /// RPN 0 (alcance do pitch bend) num canal; depois "fecha" o RPN para nada mais mexer nele.
    fn send_pitch_range(s: &mut Synthesizer, ch: i32, semis: u8) {
        s.process_midi_message(ch, 0xB0, 0x65, 0);
        s.process_midi_message(ch, 0xB0, 0x64, 0);
        s.process_midi_message(ch, 0xB0, 0x06, semis.clamp(1, 12) as i32);
        s.process_midi_message(ch, 0xB0, 0x26, 0);
        s.process_midi_message(ch, 0xB0, 0x65, 127);
        s.process_midi_message(ch, 0xB0, 0x64, 127);
    }

    fn handle(&mut self, cmd: Cmd) {
        match cmd {
            Cmd::FreezePlay(voice) => {
                let sr = self.sample_rate;
                for f in &mut self.freezes {
                    f.fade_out(sr, crate::freeze::FREEZE_OUT_S);
                }
                if self.freezes.len() >= MAX_FREEZE {
                    let old = self.freezes.remove(0);
                    let _ = self.garbage.try_send(Garbage::Freeze(old));
                }
                self.freezes.push(voice);
            }
            Cmd::FreezeStop(secs) => {
                let sr = self.sample_rate;
                for f in &mut self.freezes {
                    f.fade_out(sr, secs);
                }
            }
            Cmd::PitchRange(semis) => {
                self.pitch_range = semis.clamp(1, 12);
                if let Some(s) = self.synth.as_mut() {
                    for i in 0..LAYERS {
                        Self::send_pitch_range(s, i as i32, self.pitch_range);
                    }
                }
            }
            Cmd::Replace(new) => {
                if let Some(old) = self.synth.replace(new) {
                    let _ = self.garbage.try_send(Garbage::Synth(old));
                }
                self.sample_rate = self
                    .synth
                    .as_ref()
                    .map(|s| s.get_sample_rate() as u32)
                    .unwrap_or(48_000);
                self.reverb.set_sample_rate(self.sample_rate);
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
                    self.arp.key_up(prev);
                    self.voicefx.note_off(prev);
                }
                self.held[key as usize] = Some(k);
                // v0.17: a camada do arpejador não toca o acorde direto (ela arpeja).
                let arp_layer = self.arp.p.on.then_some(self.arp.p.layer);
                if let Some(s) = self.synth.as_mut() {
                    for i in 0..LAYERS {
                        if self.audible[i] && self.presets[i].is_some() && Some(i) != arp_layer {
                            s.note_on(i as i32, k as i32, vel as i32);
                        }
                    }
                }
                if self.arp.p.on {
                    let was_empty = self.arp.is_empty() || self.arp.playing.is_none();
                    self.arp.key_down(k, vel);
                    if was_empty {
                        // primeiro dedo: o arpejo começa na hora
                        self.arp_next = self.clock as f64;
                    }
                }
                if self.voicefx.on {
                    let sr = self.sample_rate;
                    self.voicefx.note_on(k, vel, sr);
                }
            }
            Cmd::NoteOff { key } => {
                let k = self.held[key as usize].take().unwrap_or(key);
                self.note_off_all_layers(k);
                self.arp.key_up(k);
                if self.arp.is_empty() {
                    self.arp_silence();
                }
                self.voicefx.note_off(k);
            }
            Cmd::Tempo(bpm) => self.bpm = bpm.clamp(efeitos::BPM_MIN, efeitos::BPM_MAX),
            Cmd::TempoSync => {
                self.pulse_origin = self.clock;
                self.arp_next = self.clock as f64;
                // v0.33: a batida acerta o tempo na batida do Tap (segue o compasso, pulando para o próximo tempo)
                if self.seq.p.on {
                    let pb = self.seq.p.per_beat.max(1) as usize;
                    let steps = (self.seq.p.steps as usize).clamp(1, efeitos::SEQ_MAX_STEPS);
                    // o passo que acabou de tocar: se já é o começo de um tempo, não repete
                    let last = (self.seq.step + steps - 1) % steps;
                    if last % pb != 0 {
                        self.seq.step = (last / pb + 1) * pb % steps;
                        self.seq_next = self.clock as f64;
                    }
                }
            }
            Cmd::Seq(p) => {
                let start = p.on && !self.seq.p.on;
                self.seq.p = p;
                self.seq.step %= (p.steps as usize).clamp(1, efeitos::SEQ_MAX_STEPS);
                if start {
                    self.seq.step = 0;
                    self.seq_next = self.clock as f64;
                }
                if !p.on {
                    self.stats.seq_pos.store(0, Ordering::Relaxed);
                }
            }
            Cmd::Pulse(p) => {
                if p.on && !self.pulse.on {
                    self.pulse_origin = self.clock;
                }
                self.pulse = p;
            }
            Cmd::Arp(p) => {
                let changed_layer = p.layer != self.arp.p.layer;
                if !p.on || changed_layer {
                    self.arp_silence();
                }
                if self.arp.p.latch && !p.latch {
                    self.arp.drop_latched();
                }
                let turned_on = p.on && !self.arp.p.on;
                self.arp.p = p;
                if turned_on {
                    // começa com as teclas que já estão seguradas
                    self.arp.clear();
                    for h in self.held.iter().flatten() {
                        self.arp.key_down(*h, 96);
                    }
                    self.arp_next = self.clock as f64;
                }
            }
            Cmd::Filter {
                amount,
                resonance,
                all,
                with_swell,
            } => {
                self.filt_amount = amount.clamp(0.0, 1.0);
                self.filt_res = resonance.clamp(0.0, 1.0);
                self.filt_all = all;
                self.filt_with_swell = with_swell;
            }
            Cmd::Swell { inn, secs } => {
                let sr = self.sample_rate;
                self.swell.set(inn, secs, sr);
            }
            Cmd::DrumKit(kit) => self.drums.kit = Some(kit),
            Cmd::Drum { pad, vel } => self.drums.hit(pad, vel),
            Cmd::DrumVolume(v) => self.drum_vol = v.clamp(0.0, 1.0),
            Cmd::VoiceFxSource(src) => self.voicefx.src = Some(src),
            Cmd::VoiceFxTexture(t) => {
                // só troca (e solta as vozes) se for outra textura de verdade
                let same = match (&self.voicefx.tex, &t) {
                    (Some(a), Some(b)) => Arc::ptr_eq(a, b),
                    (None, None) => true,
                    _ => false,
                };
                if !same {
                    self.voicefx.all_off();
                    self.voicefx.tex = t;
                }
            }
            Cmd::VoiceFxKey(k) => self.voicefx.tex_key = k % 12,
            Cmd::VoiceFx { on, level, octave } => {
                if !on && self.voicefx.on {
                    self.voicefx.all_off();
                }
                self.voicefx.on = on;
                self.voicefx.level = level.clamp(0.0, 1.0);
                self.voicefx.octave = octave.clamp(-2, 2);
            }
            Cmd::Cc { num, val } => {
                // Volume/expressão/reverb do piano não mexem na mistura: quem manda é a mesa.
                if matches!(num, 0x00 | 0x07 | 0x0B | 0x20 | 0x27 | 0x2B | 0x5B | 0x5D) {
                    return;
                }
                if num == 64 {
                    self.voicefx.set_sustain(val >= 64);
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
            Cmd::SetCutoff { layer, value } if layer < LAYERS => {
                self.cutoffs[layer] = value;
                if let Some(s) = self.synth.as_mut() {
                    s.process_midi_message(layer as i32, 0xB0, 0x4A, (value.clamp(0.0, 1.0) * 127.0).round() as i32);
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
                    self.reverb.set_sample_rate(self.sample_rate);
                }
            }
            Cmd::GrainPlay(voice) => {
                let sr = self.sample_rate;
                for v in &mut self.pads {
                    v.fade_out(sr);
                }
                for g in &mut self.grains {
                    g.fade_out(sr);
                }
                if self.grains.len() >= MAX_PADS {
                    let old = self.grains.remove(0);
                    let _ = self.garbage.try_send(Garbage::Grain(old));
                }
                self.grains.push(voice);
            }
            Cmd::GrainParams(p) => self.grain_params = p,
            Cmd::Church { pads, sf2 } => {
                self.church_pads = pads.clamp(0.0, 1.0);
                self.church_sf2 = sf2.clamp(0.0, 1.0);
            }
            Cmd::PadPlay(clip, pitch) => {
                let sr = self.sample_rate;
                for v in &mut self.pads {
                    v.fade_out(sr);
                }
                for g in &mut self.grains {
                    g.fade_out(sr);
                }
                // Lista cheia: o pad mais antigo sai na hora (já estava sumindo).
                if self.pads.len() >= MAX_PADS {
                    let old = self.pads.remove(0);
                    let _ = self.garbage.try_send(Garbage::Pad(old.clip));
                }
                self.pads.push(PadVoice::new(clip, sr, pitch));
            }
            Cmd::PadLayer2(clip, pitch) => {
                let sr = self.sample_rate;
                if self.pads.len() >= MAX_PADS {
                    let old = self.pads.remove(0);
                    let _ = self.garbage.try_send(Garbage::Pad(old.clip));
                }
                self.pads.push(PadVoice::new(clip, sr, pitch).with_layer(1));
            }
            Cmd::PadMix(a, b) => self.pad_layer = [a.clamp(0.0, 1.0), b.clamp(0.0, 1.0)],
            Cmd::PadStop => {
                let sr = self.sample_rate;
                for v in &mut self.pads {
                    v.fade_out(sr);
                }
                for g in &mut self.grains {
                    g.fade_out(sr);
                }
            }
            Cmd::PadVolume(v) => self.pad_vol = v.clamp(0.0, 1.0).powi(2),
            Cmd::SetTranspose(t) => self.transpose = t.clamp(-24, 24),
            Cmd::Panic => {
                // pânico também cala o congelado (rápido, sem estalo), o arpejo, o coral e as batidas
                let sr = self.sample_rate;
                // e o swell volta a "cheio" (o próximo som não pode sair mudo)
                self.swell.set(true, 0.0, sr);
                for f in &mut self.freezes {
                    f.fade_out(sr, 0.3);
                }
                self.arp_silence();
                self.arp.clear();
                self.voicefx.set_sustain(false);
                self.voicefx.all_off();
                self.drums.stop();
                self.seq.p.on = false;
                self.stats.seq_pos.store(0, Ordering::Relaxed);
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
        // a camada do arpejador é dele: soltar a tecla não pode cortar a nota do arpejo
        let arp_layer = self.arp.p.on.then_some(self.arp.p.layer);
        if let Some(s) = self.synth.as_mut() {
            for i in 0..LAYERS {
                if Some(i) != arp_layer {
                    s.note_off(i as i32, key as i32);
                }
            }
        }
    }

    /// Para os testes: nota do arpejo soando agora e vozes do coral.
    #[cfg(test)]
    pub fn test_arp_playing(&self) -> Option<u8> {
        self.arp.playing
    }

    #[cfg(test)]
    pub fn test_choir_voices(&self) -> usize {
        self.voicefx.active_voices()
    }

    /// Para os testes: quantos pads tocando (sem estar sumindo) em cada camada, e o volume de cada camada.
    #[cfg(test)]
    pub fn test_pad_layers(&self) -> ([usize; 2], [f32; 2]) {
        let mut n = [0usize; 2];
        for v in &self.pads {
            if !v.fading() {
                n[v.layer as usize] += 1;
            }
        }
        (n, self.pad_layer)
    }

    /// Solta a nota do arpejo que está soando.
    fn arp_silence(&mut self) {
        if let Some(n) = self.arp.playing.take() {
            let layer = self.arp.p.layer.min(LAYERS - 1) as i32;
            if let Some(s) = self.synth.as_mut() {
                s.note_off(layer, n as i32);
            }
        }
    }

    /// Próxima nota do arpejo (chamado no tempo certo pelo `render_interleaved`).
    fn arp_tick(&mut self) {
        self.arp_silence();
        let layer = self.arp.p.layer.min(LAYERS - 1);
        if !(self.audible[layer] && self.presets[layer].is_some()) {
            return;
        }
        if let Some(n) = self.arp.next_note() {
            let vel = self.arp.vel as i32;
            if let Some(s) = self.synth.as_mut() {
                s.note_on(layer as i32, n as i32, vel);
                self.arp.playing = Some(n);
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
            let mut n = (frames - done).min(self.left.len());
            // v0.17: arpejo no tempo certo (o bloco é cortado exatamente na hora da próxima nota).
            if self.arp.p.on {
                let step = efeitos::step_samples(self.bpm, self.arp.p.per_beat, sr);
                while self.clock as f64 >= self.arp_next {
                    self.arp_tick();
                    self.arp_next += step;
                    if self.arp_next < self.clock as f64 {
                        self.arp_next = self.clock as f64 + step; // ficou para trás (mudou o tempo): segue daqui
                    }
                }
                let until = (self.arp_next - self.clock as f64).ceil().max(1.0) as usize;
                n = n.min(until);
            }
            // v0.33: Sequencer no tempo certo (igual ao arpejo: o bloco é cortado na hora do próximo passo)
            if self.seq.p.on {
                let step = efeitos::step_samples(self.bpm, self.seq.p.per_beat, sr);
                while self.clock as f64 >= self.seq_next {
                    let drums = &mut self.drums;
                    let s = self.seq.tick(|pad, vel| drums.hit(pad, vel));
                    self.stats.seq_pos.store(s as u32 + 1, Ordering::Relaxed);
                    self.seq_next += step;
                    if self.seq_next < self.clock as f64 {
                        self.seq_next = self.clock as f64 + step; // ficou para trás (mudou o tempo): segue daqui
                    }
                }
                let until = (self.seq_next - self.clock as f64).ceil().max(1.0) as usize;
                n = n.min(until);
            }
            // Ganho do PulsePad para a amostra `f` deste bloco.
            let (pulse_on, pulse_depth) = (self.pulse.on, self.pulse.depth);
            let pulse_step = efeitos::step_samples(self.bpm, self.pulse.per_beat, sr);
            let pulse_t0 = self.clock.wrapping_sub(self.pulse_origin) as f64;
            let pulse_at = |f: usize| -> f32 {
                let ph = ((pulse_t0 + f as f64) / pulse_step).fract() as f32;
                efeitos::pulse_gain(ph, pulse_depth)
            };
            let (l, r) = (&mut self.left[..n], &mut self.right[..n]);
            match self.synth.as_mut() {
                Some(synth) => synth.render(l, r),
                None => {
                    l.fill(0.0);
                    r.fill(0.0);
                }
            }
            // v0.17: coral do Voice FX junto das camadas (o Congelar também pega ele).
            if self.voicefx.active_voices() > 0 {
                let g = self.voicefx.level * self.master;
                self.voicefx.mix(l, r, sr, g);
            }
            // v0.16: guarda o som das camadas (para o Congelar) e soma o congelado.
            // Ele entra aqui, junto das camadas, para ganhar o mesmo reverb de igreja delas.
            crate::freeze::ring().push(l, r);
            // v0.23: filtro (abrir/fechar) e swell — corte deste bloco e ganho de cada amostra
            let cut = self.filt_amount
                * if self.filt_with_swell {
                    self.swell.gain
                } else {
                    1.0
                };
            let (cut_hz, res, all) = (efeitos::filter_hz(cut), self.filt_res, self.filt_all);
            let swell_on = !self.swell.is_full();
            if swell_on {
                self.swell.fill(&mut self.swell_buf[..n]);
            }
            let sw = &self.swell_buf[..n];
            if !self.freezes.is_empty() {
                let (fl, fr) = (&mut self.fz_l[..n], &mut self.fz_r[..n]);
                fl.fill(0.0);
                fr.fill(0.0);
                for fz in &mut self.freezes {
                    fz.mix(fl, fr, sr);
                }
                if !all {
                    self.lp_fz.process(fl, fr, cut_hz, res, sr);
                }
                for f in 0..n {
                    // v0.17: o congelado também pulsa no PulsePad
                    let mut g = if pulse_on { pulse_at(f) } else { 1.0 };
                    if swell_on && !all {
                        g *= sw[f];
                    }
                    l[f] += fl[f] * g;
                    r[f] += fr[f] * g;
                }
            }
            // Pads (loop e Vivo) num "barramento" próprio; depois vão para a saída e para o reverb de igreja.
            let has_pads = !self.pads.is_empty() || !self.grains.is_empty();
            let church = self.church_pads > 0.001 || self.church_sf2 > 0.001;
            if has_pads || church {
                // Volume do pad muda suave (sem degrau quando mexe o fader).
                self.pad_vol_now += (self.pad_vol - self.pad_vol_now) * 0.2;
                let gain = self.pad_vol_now * self.master;
                let (pl, pr) = (&mut self.pad_l[..n], &mut self.pad_r[..n]);
                pl.fill(0.0);
                pr.fill(0.0);
                // v0.24: cada camada (pad principal / segundo pad) com o seu volume do mix, mudando suave
                for k in 0..2 {
                    self.pad_layer_now[k] += (self.pad_layer[k] - self.pad_layer_now[k]) * 0.2;
                }
                let lg = self.pad_layer_now;
                for v in &mut self.pads {
                    v.mix(pl, pr, lg[v.layer as usize], sr);
                }
                let gp = self.grain_params;
                for g in &mut self.grains {
                    g.mix(pl, pr, &gp, sr);
                }
                if pulse_on && has_pads {
                    // v0.17: PulsePad — o pad pulsa no tempo (antes do reverb: a cauda fica lisa)
                    for f in 0..n {
                        let g = pulse_at(f);
                        pl[f] *= g;
                        pr[f] *= g;
                    }
                }
                if has_pads && !all {
                    // v0.23: filtro e swell no fundo (antes do reverb: a sala acompanha a abertura)
                    self.lp_pad.process(pl, pr, cut_hz, res, sr);
                    if swell_on {
                        for f in 0..n {
                            pl[f] *= sw[f];
                            pr[f] *= sw[f];
                        }
                    }
                }
                if church {
                    let (rl, rr) = (&mut self.rev_l[..n], &mut self.rev_r[..n]);
                    let (sp, ss) = (self.church_pads * gain, self.church_sf2);
                    for f in 0..n {
                        rl[f] = pl[f] * sp + l[f] * ss;
                        rr[f] = pr[f] * sp + r[f] * ss;
                    }
                }
                for f in 0..n {
                    l[f] += pl[f] * gain;
                    r[f] += pr[f] * gain;
                }
                if church {
                    self.reverb
                        .process(&self.rev_l[..n], &self.rev_r[..n], l, r, 1.0);
                }
            }
            if all {
                // v0.23: "tudo menos a bateria": filtro e swell no som inteiro
                self.lp_all.process(l, r, cut_hz, res, sr);
                if swell_on {
                    for f in 0..n {
                        l[f] *= sw[f];
                        r[f] *= sw[f];
                    }
                }
            }
            // v0.17: DrumPads por último, secos (não entram no Congelar nem no pulso).
            self.drums.mix(l, r, self.drum_vol * self.master);
            self.clock += n as u64;
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

        let mut i = 0;
        while i < self.grains.len() {
            if self.grains[i].finished() {
                let v = self.grains.remove(i);
                let _ = self.garbage.try_send(Garbage::Grain(v));
            } else {
                i += 1;
            }
        }

        let mut i = 0;
        while i < self.freezes.len() {
            if self.freezes[i].finished() {
                let v = self.freezes.remove(i);
                let _ = self.garbage.try_send(Garbage::Freeze(v));
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
