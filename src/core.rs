//! Cérebro do programa: guarda o estado da mesa e traduz piano/mesinha/tela em comandos
//! para o motor. Usado pela tela e pelas threads de MIDI (sempre atrás de um Mutex).

use std::collections::HashMap;
use std::io::Read;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicU8, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use crossbeam_channel::Sender;
use rustysynth::SoundFont;

use crate::engine::{Cmd, Engine};
use crate::granular::{GrainParams, GrainSource, GranularVoice};
use crate::model::*;
use crate::pads::{self, PadClip, ALL_BANKS, MY_BANK, PAD_BANKS, PAD_KEYS};

/// Pedido de carregar um pad: (banco, tom, arquivo próprio).
pub type PadJob = (usize, usize, Option<String>);

pub struct Learn {
    pub active: bool,
    pub target: Option<Target>,
}

/// "Configurar mesinha": o Theo mexe cada controle na ordem pedida e o programa aprende sozinho.
pub struct Wizard {
    pub step: usize,
    pub steps: Vec<(Target, &'static str)>,
    /// Controle candidato do passo atual e quantos sinais dele já chegaram.
    candidate: Option<(ControlEvent, usize, bool)>,
    /// Controles já usados neste assistente (o fader anterior ainda pode estar mandando sinal).
    used: Vec<Source>,
}

impl Wizard {
    pub fn new() -> Self {
        let mut steps: Vec<(Target, &'static str)> = Vec::new();
        const FADERS: [&str; 8] = [
            "FADER 1", "FADER 2", "FADER 3", "FADER 4", "FADER 5", "FADER 6", "FADER 7", "FADER 8",
        ];
        const KNOBS: [&str; 7] = [
            "KNOB 1", "KNOB 2", "KNOB 3", "KNOB 4", "KNOB 5", "KNOB 6", "KNOB 7",
        ];
        for (i, name) in FADERS.iter().enumerate().take(LAYERS) {
            steps.push((Target::Volume(i), name));
        }
        steps.push((Target::PadVolume, FADERS[6]));
        steps.push((Target::Master, FADERS[7]));
        for (i, name) in KNOBS.iter().enumerate().take(LAYERS) {
            steps.push((Target::Reverb(i), name));
        }
        steps.push((Target::PadKnob, KNOBS[6]));
        Self {
            step: 0,
            steps,
            candidate: None,
            used: Vec::new(),
        }
    }

    pub fn current(&self) -> Option<(Target, &'static str)> {
        self.steps.get(self.step).copied()
    }
}

impl Default for Wizard {
    fn default() -> Self {
        Self::new()
    }
}

/// Progresso do carregamento do SF2.
pub struct Loading {
    pub file: String,
    pub read: Arc<AtomicU64>,
    pub total: u64,
}

pub struct Core {
    pub cfg: Config,
    pub tx: Sender<Cmd>,
    pub presets: Vec<PresetRef>,
    /// v0.35: timbres do SF2 principal e da Biblioteca (a `presets` é os dois juntos).
    pub main_presets: Vec<PresetRef>,
    pub lib_presets: Vec<PresetRef>,
    /// Arquivos da Biblioteca abertos (só os que alguma camada usa) e os que estão abrindo.
    pub lib_fonts: HashMap<String, Arc<SoundFont>>,
    pub lib_loading: std::collections::HashSet<String>,
    /// Arquivo cujo sintetizador o motor já tem em cada camada.
    layer_font: [Option<String>; LAYERS],
    pub font_loader: Option<Sender<String>>,
    pub sound_font: Option<Arc<SoundFont>>,
    pub sf2_name: Option<String>,
    pub loading: Option<Loading>,
    pub load_error: Option<String>,
    pub sample_rate: u32,
    pub learn: Learn,
    /// Último valor de cada CC dos controles (para detectar o "aperto" do botão).
    last_cc: HashMap<(u8, u8), u8>,
    /// Mensagem rápida na tela (texto, quando apareceu).
    pub toast: Option<(String, Instant)>,
    pub dirty: bool,
    pub piano_activity: Option<Instant>,
    pub controller_activity: Option<Instant>,
    /// Quando mudar, a tela é redesenhada (MIDI chega de outra thread).
    pub repaint: Option<Box<dyn Fn() + Send>>,
    /// Pads já decodificados [banco][tom].
    pub pad_clips: Vec<Vec<Option<Arc<PadClip>>>>,
    /// Tom do pad tocando agora (ou esperando carregar).
    pub pad_active: Option<usize>,
    pad_pending: Option<(usize, usize)>,
    /// v0.24: tom esperando o segundo pad carregar.
    pad_pending2: Option<usize>,
    pad_jobs: std::collections::HashSet<(usize, usize)>,
    /// Thread que decodifica pads (sem ela, decodifica na hora — usado nos testes).
    pub pad_loader: Option<Sender<PadJob>>,
    /// Último sinal que chegou da mesinha (aparece no modo Aprender).
    pub last_signal: Option<String>,
    /// Texturas do Pad Vivo já carregadas (3).
    pub textures: Vec<Option<Arc<GrainSource>>>,
    /// Som do "Meu pad" (um arquivo para os 12 tons) e se já está sendo carregado.
    pub my_pad_clips: Vec<Option<Arc<PadClip>>>,
    tex_jobs: std::collections::HashSet<usize>,
    grain_seed: u32,
    pub wizard: Option<Wizard>,
    /// v0.15: pitch bend agora (−1 = lá embaixo, 0 = afinado, +1 = lá em cima).
    pub pitch: f32,
    /// v0.15: modulação agora (0..1, CC 1).
    pub modulation: f32,
    /// v0.15: último sinal que chegou do teclado (aparece no modo Aprender).
    pub last_piano_signal: Option<String>,
    /// v0.16: o Congelar está ligado (tem som congelado tocando).
    pub freeze_on: bool,
    // v0.17: aba Efeitos (o que está ligado não fica salvo: abre sempre desligado)
    pub pulse_on: bool,
    pub arp_on: bool,
    pub voicefx_on: bool,
    /// v0.23: o swell está "dentro" (som cheio) ou "fora" (som saiu devagar)?
    pub swell_in: bool,
    /// v0.24: o último Freeze (para o botão ↺).
    last_freeze: Option<Arc<crate::freeze::FreezeSource>>,
    /// Batidas do "bater o tempo" (segundos desde `tap_origin`).
    taps: Vec<f64>,
    tap_origin: Instant,
    drum_kit: Option<Arc<crate::efeitos::DrumKit>>,
    /// Arquivos com que o kit atual foi montado (mudou → monta de novo).
    drum_kit_files: Vec<(usize, String)>,
    voicefx_src: Option<Arc<GrainSource>>,
    /// v0.31: textura do Voice FX carregada (quando a voz escolhida é uma textura).
    voicefx_tex: Option<Arc<crate::efeitos::VoiceTexture>>,
    /// Quando cada DrumPad foi tocado (a tela acende o botão por um instante).
    pub drum_flash: [Option<Instant>; crate::efeitos::MAX_DRUMS],
    /// v0.33: o Sequencer está tocando a batida? (não fica salvo: abre sempre parado)
    pub seq_on: bool,
}

pub type SharedCore = Arc<Mutex<Core>>;

// ---------- v0.15: controles do teclado (porta Piano) ----------
// A thread do MIDI precisa saber, sem travar o `core`, se uma mensagem do teclado é nota para tocar
// ou um controle ligado pelo Aprender. Estas marcas são copiadas do `core` por `sync_piano_hooks`.

/// Aprender esperando um controle (aí tudo que vem do teclado vai para o Aprender, sem tocar).
static PIANO_LEARNING: AtomicBool = AtomicBool::new(false);
/// Notas (0..127) do teclado ligadas a algum controle.
static PIANO_NOTES: [AtomicU64; 2] = [AtomicU64::new(0), AtomicU64::new(0)];
/// CCs (0..127) do teclado ligados a algum controle.
static PIANO_CCS: [AtomicU64; 2] = [AtomicU64::new(0), AtomicU64::new(0)];
/// Pitch bend do teclado ligado a algum controle.
static PIANO_BEND: AtomicBool = AtomicBool::new(false);

fn bit_of(n: u8) -> (usize, u64) {
    ((n as usize >> 6) & 1, 1u64 << (n & 63))
}

/// A mensagem do teclado vai para o `core` (Aprender / controle ligado) em vez de tocar direto?
pub fn piano_routed(msg: &[u8]) -> bool {
    let Some(&status) = msg.first() else {
        return false;
    };
    let kind = status & 0xF0;
    if !matches!(kind, 0x80 | 0x90 | 0xB0 | 0xE0) {
        return false;
    }
    if PIANO_LEARNING.load(Ordering::Relaxed) {
        return true;
    }
    let d1 = msg.get(1).copied().unwrap_or(0) & 0x7F;
    let (i, bit) = bit_of(d1);
    match kind {
        0x80 | 0x90 => PIANO_NOTES[i].load(Ordering::Relaxed) & bit != 0,
        0xB0 => PIANO_CCS[i].load(Ordering::Relaxed) & bit != 0,
        _ => PIANO_BEND.load(Ordering::Relaxed),
    }
}

/// Notas apertadas agora (piano de verdade ou teclado da tela), para acender as teclas na tela.
static HELD: [AtomicU64; 2] = [AtomicU64::new(0), AtomicU64::new(0)];

pub fn mark_note(key: u8, down: bool) {
    let (i, bit) = ((key as usize >> 6) & 1, 1u64 << (key & 63));
    if down {
        HELD[i].fetch_or(bit, Ordering::Relaxed);
        phrase_record(key);
    } else {
        HELD[i].fetch_and(!bit, Ordering::Relaxed);
    }
}

// ---------- v0.34: Learn da Frase (as notas tocadas no teclado viram a frase do Arpeggiator) ----------
// Gravado aqui (sem travar o `core`): a thread do MIDI só marca as notas; a tela lê e guarda.
static PHRASE_REC: AtomicBool = AtomicBool::new(false);
static PHRASE_N: AtomicUsize = AtomicUsize::new(0);
static PHRASE_NOTES: [AtomicU8; crate::efeitos::PHRASE_MAX] = [const { AtomicU8::new(0) }; crate::efeitos::PHRASE_MAX];

fn phrase_record(key: u8) {
    if !PHRASE_REC.load(Ordering::Relaxed) {
        return;
    }
    let n = PHRASE_N.load(Ordering::Relaxed);
    if n < crate::efeitos::PHRASE_MAX {
        PHRASE_NOTES[n].store(key.min(127), Ordering::Relaxed);
        PHRASE_N.store(n + 1, Ordering::Relaxed);
    }
}

/// O Learn da Frase está gravando?
pub fn phrase_recording() -> bool {
    PHRASE_REC.load(Ordering::Relaxed)
}

/// Notas gravadas até agora no Learn da Frase.
pub fn phrase_recorded() -> Vec<u8> {
    let n = PHRASE_N.load(Ordering::Relaxed).min(crate::efeitos::PHRASE_MAX);
    (0..n).map(|i| PHRASE_NOTES[i].load(Ordering::Relaxed)).collect()
}

pub fn note_is_held(key: u8) -> bool {
    HELD[(key as usize >> 6) & 1].load(Ordering::Relaxed) & (1u64 << (key & 63)) != 0
}

fn clear_held() {
    HELD[0].store(0, Ordering::Relaxed);
    HELD[1].store(0, Ordering::Relaxed);
}

impl Core {
    pub fn new(cfg: Config, tx: Sender<Cmd>) -> Self {
        // v0.24: volta com os efeitos que estavam ligados quando o programa fechou
        let (pulse_on, arp_on, voicefx_on) = (cfg.efeitos.pulse_on, cfg.efeitos.arp_on, cfg.efeitos.voicefx_on);
        let mut c = Self::empty(cfg, tx);
        c.pulse_on = pulse_on;
        c.arp_on = arp_on;
        if voicefx_on && c.load_voicefx_source() {
            c.voicefx_on = true;
        }
        c
    }

    fn empty(cfg: Config, tx: Sender<Cmd>) -> Self {
        Self {
            cfg,
            tx,
            presets: Vec::new(),
            main_presets: Vec::new(),
            lib_presets: Vec::new(),
            lib_fonts: HashMap::new(),
            lib_loading: Default::default(),
            layer_font: Default::default(),
            font_loader: None,
            sound_font: None,
            sf2_name: None,
            loading: None,
            load_error: None,
            sample_rate: 48_000,
            learn: Learn {
                active: false,
                target: None,
            },
            last_cc: HashMap::new(),
            toast: None,
            dirty: false,
            piano_activity: None,
            controller_activity: None,
            repaint: None,
            pad_clips: vec![vec![None; PAD_KEYS]; PAD_BANKS],
            pad_active: None,
            pad_pending: None,
            pad_pending2: None,
            pad_jobs: Default::default(),
            pad_loader: None,
            last_signal: None,
            textures: vec![None, None, None],
            my_pad_clips: vec![None; pads::MY_SLOTS],
            tex_jobs: Default::default(),
            grain_seed: 0x1234_5678,
            wizard: None,
            pitch: 0.0,
            modulation: 0.0,
            last_piano_signal: None,
            freeze_on: false,
            pulse_on: false,
            arp_on: false,
            voicefx_on: false,
            swell_in: true,
            last_freeze: None,
            taps: Vec::new(),
            tap_origin: Instant::now(),
            drum_kit: None,
            drum_kit_files: Vec::new(),
            voicefx_src: None,
            voicefx_tex: None,
            drum_flash: [None; crate::efeitos::MAX_DRUMS],
            seq_on: false,
        }
    }

    // ---------- v0.17: aba Efeitos ----------

    /// Manda para o motor tudo da aba Efeitos (ao iniciar, ao trocar de motor e quando muda algo).
    pub fn send_efeitos(&self) {
        let e = &self.cfg.efeitos;
        self.send(Cmd::Tempo(e.bpm));
        self.send(Cmd::Pulse(crate::efeitos::PulseParams {
            on: self.pulse_on,
            per_beat: e.pulse_per_beat,
            depth: e.pulse_depth,
        }));
        let mut phrase = [0u8; crate::efeitos::PHRASE_MAX];
        for (i, n) in e.arp_phrase.iter().take(crate::efeitos::PHRASE_MAX).enumerate() {
            phrase[i] = *n;
        }
        self.send(Cmd::Arp(crate::efeitos::ArpParams {
            // v0.34: gravando a frase, o arpejo descansa (você ouve as notas que está tocando)
            on: self.arp_on && !phrase_recording(),
            layer: e.arp_layer,
            pattern: e.arp_pattern,
            per_beat: e.arp_per_beat,
            octaves: e.arp_octaves,
            latch: e.arp_latch,
            phrase,
            phrase_len: e.arp_phrase.len().min(crate::efeitos::PHRASE_MAX) as u8,
            follow: e.arp_follow,
        }));
        self.send(Cmd::DrumVolume(e.drum_volume));
        self.send(Cmd::Seq(e.seq_params(self.seq_on)));
        if let Some(k) = &self.drum_kit {
            self.send(Cmd::DrumKit(k.clone()));
        }
        if let Some(s) = &self.voicefx_src {
            self.send(Cmd::VoiceFxSource(s.clone()));
        }
        self.send(Cmd::VoiceFxKey(e.voicefx_key as u8));
        self.send(Cmd::VoiceFxTexture(self.voicefx_tex.clone()));
        self.send(Cmd::VoiceFx {
            on: self.voicefx_on,
            level: e.voicefx_volume,
            octave: e.voicefx_octave,
        });
        self.send(Cmd::Filter {
            amount: e.filtro,
            resonance: e.filtro_res,
            all: e.filtro_tudo,
            with_swell: e.swell_filtro,
        });
    }

    /// v0.23: filtro de abrir/fechar (0 = fechado/escuro … 1 = aberto).
    pub fn set_filter(&mut self, v: f32) {
        let mut e = self.cfg.efeitos.clone();
        e.filtro = v.clamp(0.0, 1.0);
        self.set_efeitos(e);
    }

    /// v0.23: Swell — se o som está cheio, sai devagar; se saiu, entra devagar (abrindo o filtro junto).
    pub fn toggle_swell(&mut self) {
        self.swell_in = !self.swell_in;
        self.send(Cmd::Swell {
            inn: self.swell_in,
            secs: self.cfg.efeitos.swell_secs,
        });
        let onde = if self.cfg.efeitos.filtro_tudo {
            "o som"
        } else {
            "o fundo"
        };
        self.notify(if self.swell_in {
            format!("Swell In: {onde} entrando devagar ({:.0} s)", self.cfg.efeitos.swell_secs)
        } else {
            format!("Fade Out: {onde} saindo devagar ({:.0} s)", self.cfg.efeitos.swell_secs)
        });
    }

    /// A tela mudou algum ajuste da aba Efeitos.
    pub fn set_efeitos(&mut self, mut e: Efeitos) {
        // ligado/desligado é dos botões (a tela trabalha numa cópia e não pode desfazer o que um botão fez)
        e.pulse_on = self.cfg.efeitos.pulse_on;
        e.arp_on = self.cfg.efeitos.arp_on;
        e.voicefx_on = self.cfg.efeitos.voicefx_on;
        if e != self.cfg.efeitos {
            self.cfg.efeitos = e;
            self.cfg.efeitos.normalize();
            self.send_efeitos();
            self.dirty = true;
        }
    }

    pub fn set_bpm(&mut self, bpm: f32) {
        let mut e = self.cfg.efeitos.clone();
        e.bpm = bpm;
        self.set_efeitos(e);
    }

    /// Bater o tempo: 2 ou mais batidas seguidas (até 2 s entre elas) acertam o BPM,
    /// e o pulso/arpejo recomeçam na batida.
    pub fn tap_tempo(&mut self) {
        let now = self.tap_origin.elapsed().as_secs_f64();
        if self.taps.last().is_some_and(|t| now - t > 2.0) {
            self.taps.clear();
        }
        self.taps.push(now);
        if self.taps.len() > 5 {
            self.taps.remove(0);
        }
        if let Some(bpm) = crate::efeitos::bpm_from_taps(&self.taps) {
            self.set_bpm(bpm);
            self.notify(format!("Tempo: {bpm:.0} BPM"));
        }
        self.send(Cmd::TempoSync);
    }

    pub fn toggle_pulse(&mut self) {
        self.pulse_on = !self.pulse_on;
        self.cfg.efeitos.pulse_on = self.pulse_on;
        self.dirty = true;
        self.send_efeitos();
        self.notify(if self.pulse_on {
            "Pulse Pad ligado: os pads e o ❄ Freeze pulsam no tempo"
        } else {
            "Pulse Pad desligado"
        });
    }

    pub fn toggle_arp(&mut self) {
        self.arp_on = !self.arp_on;
        self.cfg.efeitos.arp_on = self.arp_on;
        self.dirty = true;
        self.send_efeitos();
        let layer = self.cfg.efeitos.arp_layer;
        let name = self.cfg.mix.layers[layer]
            .preset
            .as_ref()
            .map(|p| p.name.clone())
            .unwrap_or_else(|| "sem timbre".into());
        if self.arp_on && !self.cfg.mix.audible(layer) {
            self.notify(format!(
                "Arpeggiator ligado na camada {} ({name}) — ligue essa camada para ouvir",
                layer + 1
            ));
        } else if self.arp_on {
            self.notify(format!("Arpeggiator ligado na camada {} ({name})", layer + 1));
        } else {
            self.notify("Arpeggiator desligado");
        }
    }

    /// Voz do Voice FX: a escolhida na barra "Voz" (do programa) ou a do Theo ("Minha").
    fn load_voicefx_source(&mut self) -> bool {
        let voice = self.cfg.efeitos.voicefx_voice;
        // v0.31: texturas (Órion, Lyra, Cassiopeia, Andrômeda, Choir Rise, Prayer) tocam no tom escolhido
        if voice >= pads::TEXTURE_BASE {
            return match pads::load_texture_voice(voice - pads::TEXTURE_BASE) {
                Ok(t) => {
                    self.voicefx_tex = Some(t.clone());
                    self.send(Cmd::VoiceFxKey(self.cfg.efeitos.voicefx_key as u8));
                    self.send(Cmd::VoiceFxTexture(Some(t)));
                    true
                }
                Err(e) => {
                    self.notify(format!("Voice FX: {e}"));
                    false
                }
            };
        }
        if self.voicefx_tex.take().is_some() {
            self.send(Cmd::VoiceFxTexture(None));
        }
        let custom = self
            .cfg
            .efeitos
            .voicefx_file
            .clone()
            .filter(|_| voice == VOICE_MINE);
        let result = match &custom {
            Some((path, root)) => pads::load_texture(0, Some((path.as_str(), *root))),
            None => pads::load_voice(voice.min(pads::VOICE_NAMES.len() - 1)),
        };
        match result {
            Ok(src) => {
                self.voicefx_src = Some(src.clone());
                self.send(Cmd::VoiceFxSource(src));
                true
            }
            Err(e) if custom.is_some() => {
                // o arquivo do Theo sumiu ou não abre: volta ao coral do programa
                self.cfg.efeitos.voicefx_file = None;
                self.cfg.efeitos.voicefx_voice = 0;
                self.dirty = true;
                self.notify(format!("Voice FX: {e}. Voltei ao coral do programa."));
                self.load_voicefx_source()
            }
            Err(e) => {
                self.notify(format!("Voice FX: {e}"));
                false
            }
        }
    }

    /// v0.21: escolhe a voz do Voice FX (0..4 do programa, 5 = "Minha"). Troca na hora, mesmo ligado.
    pub fn set_voicefx_voice(&mut self, voice: usize) {
        if voice == VOICE_MINE && self.cfg.efeitos.voicefx_file.is_none() {
            self.notify("Para usar \"Minha\", escolha antes o seu coral no ⚙ do Voice FX");
            return;
        }
        if voice >= pads::TEXTURE_BASE + pads::TEXTURE_VOICES.len() || voice == self.cfg.efeitos.voicefx_voice {
            return;
        }
        self.cfg.efeitos.voicefx_voice = voice;
        self.dirty = true;
        if self.load_voicefx_source() {
            let name = if voice == VOICE_MINE {
                "a sua".to_string()
            } else if voice >= pads::TEXTURE_BASE {
                format!(
                    "{} (no tom de {})",
                    pads::TEXTURE_VOICES[voice - pads::TEXTURE_BASE].name,
                    pads::KEY_NAMES[self.cfg.efeitos.voicefx_key]
                )
            } else {
                pads::VOICE_NAMES[voice].to_string()
            };
            self.notify(format!("Voice FX: voz {name}"));
        }
    }

    pub fn toggle_voicefx(&mut self) {
        if !self.voicefx_on && self.voicefx_src.is_none() && self.voicefx_tex.is_none() && !self.load_voicefx_source() {
            return;
        }
        self.voicefx_on = !self.voicefx_on;
        self.cfg.efeitos.voicefx_on = self.voicefx_on;
        self.dirty = true;
        self.send_efeitos();
        self.notify(if self.voicefx_on {
            "Voice FX ligado: um coral acompanha as notas que você tocar"
        } else {
            "Voice FX desligado"
        });
    }

    /// v0.19: usar um coral do Theo no Voice FX (None = volta ao do programa).
    /// O programa descobre pelo som em que nota ele está (dá para corrigir depois).
    pub fn set_voicefx_file(&mut self, path: Option<String>) {
        match path {
            Some(p) => {
                let file = std::fs::File::open(&p);
                let ext = std::path::Path::new(&p)
                    .extension()
                    .and_then(|e| e.to_str())
                    .map(|e| e.to_lowercase());
                let key = match file
                    .map_err(|e| e.to_string())
                    .and_then(|f| pads::decode(Box::new(f), ext.as_deref(), false))
                {
                    Ok(clip) => pads::detect_key(&clip),
                    Err(e) => {
                        self.notify(format!("Não consegui abrir esse som: {e}"));
                        return;
                    }
                };
                self.cfg.efeitos.voicefx_file = Some((p.clone(), pads::key_midi(key)));
                self.cfg.efeitos.voicefx_voice = VOICE_MINE; // escolheu um coral: ele já fica selecionado
                self.dirty = true;
                if self.load_voicefx_source() {
                    self.notify(format!(
                        "Voice FX com o seu som: {} (está em {}; corrija no ⚙ se precisar)",
                        pads::name_from_file(&p),
                        pads::KEY_NAMES[key]
                    ));
                }
            }
            None => {
                self.cfg.efeitos.voicefx_file = None;
                self.cfg.efeitos.voicefx_voice = 0;
                self.dirty = true;
                self.load_voicefx_source();
                self.notify("Voice FX voltou ao coral do programa");
            }
        }
    }

    /// Nota em que o coral do Theo está (0 = C ... 11 = B).
    /// v0.31: tom em que as texturas do Voice FX tocam (o tom da música).
    pub fn set_voicefx_tone(&mut self, key: usize) {
        let key = key % 12;
        if self.cfg.efeitos.voicefx_key != key {
            self.cfg.efeitos.voicefx_key = key;
            self.dirty = true;
            self.send(Cmd::VoiceFxKey(key as u8));
        }
    }

    pub fn set_voicefx_key(&mut self, key: usize) {
        let Some((_, root)) = &mut self.cfg.efeitos.voicefx_file else {
            return;
        };
        let midi = pads::key_midi(key);
        if *root != midi {
            *root = midi;
            self.dirty = true;
            self.load_voicefx_source();
        }
    }

    /// Prepara os sons dos DrumPads (na taxa da placa de som). Chamado quando a aba abre.
    pub fn ensure_drum_kit(&mut self) {
        let files = self.cfg.efeitos.drum_files.clone();
        let stale = self.drum_kit.as_ref().is_none_or(|k| k.rate != self.sample_rate)
            || files != self.drum_kit_files;
        if stale {
            let (kit, errors) = crate::efeitos::DrumKit::build_with(self.sample_rate, &files);
            let kit = Arc::new(kit);
            self.send(Cmd::DrumKit(kit.clone()));
            self.drum_kit = Some(kit);
            self.drum_kit_files = files;
            if !errors.is_empty() {
                // o arquivo sumiu ou não abre: o pad volta ao som do programa (e avisa)
                let names: Vec<String> = errors
                    .iter()
                    .map(|(p, e)| format!("{}: {e}", self.drum_name(*p)))
                    .collect();
                self.cfg
                    .efeitos
                    .drum_files
                    .retain(|(p, _)| !errors.iter().any(|(q, _)| q == p));
                self.drum_kit_files = self.cfg.efeitos.drum_files.clone();
                self.dirty = true;
                self.notify(format!("DrumPads: {} — voltou ao som do programa", names.join("; ")));
            }
        }
    }

    /// v0.19: usar um som do Theo num DrumPad (None = volta ao som do programa).
    pub fn set_drum_file(&mut self, pad: usize, path: Option<String>) {
        if pad >= crate::efeitos::MAX_DRUMS {
            return;
        }
        if let Some(p) = &path {
            // confere antes de guardar (não deixa um pad mudo)
            if let Err(e) = crate::efeitos::load_drum_file(p, self.sample_rate) {
                self.notify(format!("Não consegui usar esse som: {e}"));
                return;
            }
        }
        self.cfg.efeitos.drum_files.retain(|(i, _)| *i != pad);
        if let Some(p) = &path {
            self.cfg.efeitos.drum_files.push((pad, p.clone()));
        }
        self.cfg.efeitos.normalize();
        self.dirty = true;
        self.ensure_drum_kit();
        self.notify(match &path {
            Some(p) => format!(
                "{} agora toca o seu som: {}",
                self.drum_name(pad),
                pads::name_from_file(p)
            ),
            None => format!("{} voltou ao som do programa", self.drum_name(pad)),
        });
    }

    /// v0.28: nome do DrumPad (o que o Theo escolheu ou o do programa).
    pub fn drum_name(&self, pad: usize) -> String {
        match self.cfg.efeitos.drum_names.get(pad).map(|s| s.trim()) {
            Some(s) if !s.is_empty() => s.to_string(),
            _ => crate::efeitos::drum_default_name(pad),
        }
    }

    /// Nome do som próprio de um DrumPad (para a tela), se tiver.
    pub fn drum_file_name(&self, pad: usize) -> Option<String> {
        self.cfg
            .efeitos
            .drum_files
            .iter()
            .find(|(i, _)| *i == pad)
            .map(|(_, p)| pads::name_from_file(p))
    }

    // ---------- v0.34: Frase do Arpeggiator (Sequencer de notas) ----------

    /// ● Learn: as próximas notas tocadas no teclado (ou no teclado da tela) viram a frase.
    pub fn phrase_learn_start(&mut self) {
        PHRASE_N.store(0, Ordering::Relaxed);
        PHRASE_REC.store(true, Ordering::Relaxed);
        self.send_efeitos(); // o arpejo descansa enquanto grava
        self.notify("● Learn: toque as notas da frase no teclado (até 16)");
    }

    /// ✔ Pronto (guarda as notas tocadas) ou ✖ Cancelar (a frase de antes fica).
    pub fn phrase_learn_finish(&mut self, keep: bool) {
        if !phrase_recording() {
            return;
        }
        let notes = phrase_recorded();
        PHRASE_REC.store(false, Ordering::Relaxed);
        if keep && !notes.is_empty() {
            let n = notes.len();
            let mut e = self.cfg.efeitos.clone();
            e.arp_phrase = notes;
            e.arp_pattern = crate::efeitos::ARP_PHRASE;
            self.set_efeitos(e);
            self.notify(format!("Frase gravada: {n} notas. Ligue o ARP e segure uma tecla"));
        } else {
            self.notify("Learn cancelado: a frase continua a mesma");
        }
        self.send_efeitos(); // o arpejo volta
    }

    // ---------- v0.33: Sequencer (batida em loop no andamento do Tempo) ----------

    /// ▶ / ■: toca a batida desde o começo do compasso, ou para (os sons que já tocaram terminam sozinhos).
    pub fn toggle_seq(&mut self) {
        self.ensure_drum_kit();
        self.seq_on = !self.seq_on;
        self.send(Cmd::Seq(self.cfg.efeitos.seq_params(self.seq_on)));
        let name = crate::efeitos::RHYTHMS[self.cfg.efeitos.seq_rhythm].name;
        self.notify(if self.seq_on {
            format!("▶ Sequencer tocando: {name} · {:.0} BPM", self.cfg.efeitos.bpm)
        } else {
            "■ Sequencer parado".to_string()
        });
    }

    /// Escolhe um ritmo pronto (a batida volta a ser a dele). Se estiver tocando, continua tocando.
    pub fn set_seq_rhythm(&mut self, i: usize) {
        let mut e = self.cfg.efeitos.clone();
        e.seq_rhythm = i.min(crate::efeitos::RHYTHMS.len() - 1);
        e.seq_grid = crate::efeitos::rhythm_grid(e.seq_rhythm).to_vec();
        self.set_efeitos(e);
    }

    /// Acende/apaga um quadradinho (DrumPad `pad`, passo `step`). Parado, toca o som para ouvir.
    pub fn seq_toggle_cell(&mut self, pad: usize, step: usize) {
        if pad >= crate::efeitos::MAX_DRUMS || step >= self.cfg.efeitos.seq_steps() {
            return;
        }
        let mut e = self.cfg.efeitos.clone();
        e.seq_grid.resize(crate::efeitos::MAX_DRUMS, 0);
        e.seq_grid[pad] ^= 1 << step;
        let lit = e.seq_grid[pad] & (1 << step) != 0;
        self.set_efeitos(e);
        if lit && !self.seq_on {
            self.drum_hit(pad, 100);
        }
    }

    /// Apaga toda a batida de um DrumPad (a linha inteira).
    pub fn seq_clear_row(&mut self, pad: usize) {
        let mut e = self.cfg.efeitos.clone();
        if let Some(g) = e.seq_grid.get_mut(pad) {
            *g = 0;
        }
        self.set_efeitos(e);
    }

    pub fn drum_hit(&mut self, pad: usize, vel: u8) {
        if pad >= crate::efeitos::MAX_DRUMS {
            return;
        }
        self.ensure_drum_kit();
        self.send(Cmd::Drum {
            pad,
            vel: vel.clamp(1, 127),
        });
        self.drum_flash[pad] = Some(Instant::now());
    }

    /// v0.16: botão ❄ Congelar. Ligar = o som que as camadas estão tocando agora vira um pad
    /// que fica soando sozinho (trocar timbre, cena ou tocar por cima não mexe nele).
    /// Desligar = some devagar.
    pub fn freeze_toggle(&mut self) {
        if self.freeze_on {
            self.freeze_off();
        } else {
            self.freeze_capture();
        }
    }

    /// Congela o que está soando agora (se já tinha um congelado, ele dá lugar ao novo).
    pub fn freeze_capture(&mut self) {
        match crate::freeze::FreezeSource::capture(self.sample_rate) {
            Some(src) => self.freeze_play(src),
            None => self.notify("❄ Freeze: toque o acorde e aperte FREEZE enquanto ele ainda está soando"),
        }
    }

    /// Toca um trecho já capturado como congelado.
    pub fn freeze_play(&mut self, src: crate::freeze::FreezeSource) {
        let src = Arc::new(src);
        // v0.24: guarda como "último Freeze" (memória e arquivo) para o botão ↺ trazer de volta
        let _ = src.save_wav(&crate::freeze::saved_path());
        self.last_freeze = Some(src.clone());
        self.freeze_start(src);
        self.notify("❄ Freeze: som congelado — pode trocar os timbres e tocar por cima. Aperte de novo para soltar.");
    }

    fn freeze_start(&mut self, src: Arc<crate::freeze::FreezeSource>) {
        self.grain_seed = self.grain_seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        let voice = crate::freeze::FreezeVoice::new(src, self.sample_rate, self.grain_seed);
        self.send(Cmd::FreezePlay(Box::new(voice)));
        self.freeze_on = true;
    }

    /// v0.24: ↺ traz de volta o último Freeze (o que estava congelado antes), mesmo depois de reabrir o programa.
    pub fn freeze_recall(&mut self) {
        if self.last_freeze.is_none() {
            self.last_freeze = crate::freeze::FreezeSource::load_wav(&crate::freeze::saved_path()).map(Arc::new);
        }
        match self.last_freeze.clone() {
            Some(src) => {
                self.freeze_start(src);
                self.notify("❄ Freeze: voltou o último som congelado");
            }
            None => self.notify("❄ Ainda não tem um Freeze guardado: congele um acorde primeiro"),
        }
    }

    /// Tem um Freeze guardado para o ↺?
    pub fn has_last_freeze(&self) -> bool {
        self.last_freeze.is_some() || crate::freeze::saved_path().exists()
    }

    pub fn freeze_off(&mut self) {
        if self.freeze_on {
            self.send(Cmd::FreezeStop(crate::freeze::FREEZE_OUT_S));
            self.freeze_on = false;
            self.notify("❄ Freeze sumindo devagar");
        }
    }

    /// Copia para a thread do MIDI o que do teclado é controle (ver `piano_routed`).
    /// Barato: chamado a cada quadro da tela e depois de cada Aprender.
    pub fn sync_piano_hooks(&self) {
        let learning = self.learn.active && self.learn.target.is_some() && self.wizard.is_none();
        PIANO_LEARNING.store(learning, Ordering::Relaxed);
        let (mut notes, mut ccs, mut bend) = ([0u64; 2], [0u64; 2], false);
        for m in &self.cfg.piano_mappings {
            match m.source {
                Source::Note { num, .. } => {
                    let (i, b) = bit_of(num);
                    notes[i] |= b;
                }
                Source::Cc { num, .. } => {
                    let (i, b) = bit_of(num);
                    ccs[i] |= b;
                }
                Source::PitchBend { .. } => bend = true,
            }
        }
        for i in 0..2 {
            PIANO_NOTES[i].store(notes[i], Ordering::Relaxed);
            PIANO_CCS[i].store(ccs[i], Ordering::Relaxed);
        }
        PIANO_BEND.store(bend, Ordering::Relaxed);
    }

    /// Pitch bend em todas as camadas (−1..1; 0 = afinado).
    pub fn set_pitch(&mut self, v: f32) {
        let v = v.clamp(-1.0, 1.0);
        self.pitch = v;
        let raw = ((v + 1.0) * 8192.0).round().clamp(0.0, 16383.0) as u16;
        self.send(Cmd::PitchBend {
            lsb: (raw & 0x7F) as u8,
            msb: (raw >> 7) as u8,
        });
    }

    /// Modulação (vibrato) em todas as camadas (0..1).
    pub fn set_modulation(&mut self, v: f32) {
        let v = v.clamp(0.0, 1.0);
        self.modulation = v;
        self.send(Cmd::Cc {
            num: 1,
            val: (v * 127.0).round() as u8,
        });
    }

    /// Alcance do pitch bend (semitons).
    pub fn set_pitch_range(&mut self, semis: u8) {
        self.cfg.pitch_range = semis.clamp(1, 12);
        self.send(Cmd::PitchRange(self.cfg.pitch_range));
        self.dirty = true;
    }

    /// Botão "de segurar": apertou = liga, soltou = volta ao normal.
    pub fn hold(&mut self, t: Target, down: bool) {
        match t {
            Target::Pitch | Target::PitchUp => self.set_pitch(if down { 1.0 } else { 0.0 }),
            Target::PitchDown => self.set_pitch(if down { -1.0 } else { 0.0 }),
            Target::Modulation => self.set_modulation(if down { 1.0 } else { 0.0 }),
            _ => {}
        }
    }

    fn send(&self, cmd: Cmd) {
        let _ = self.tx.send(cmd);
    }

    pub fn notify(&mut self, msg: impl Into<String>) {
        self.toast = Some((msg.into(), Instant::now()));
    }

    pub fn request_repaint(&self) {
        if let Some(r) = &self.repaint {
            r();
        }
    }

    /// Manda o estado inteiro para o motor (ao iniciar ou ao trocar o sintetizador).
    pub fn sync_engine(&self) {
        let mix = &self.cfg.mix;
        for (i, l) in mix.layers.iter().enumerate() {
            if let Some(p) = &l.preset {
                self.send(Cmd::SetPreset {
                    layer: i,
                    bank: p.bank,
                    patch: p.patch,
                });
            }
            self.send(Cmd::SetVolume {
                layer: i,
                value: l.volume,
            });
            self.send(Cmd::SetReverb {
                layer: i,
                value: l.reverb,
            });
            self.send(Cmd::SetCutoff {
                layer: i,
                value: l.cutoff,
            });
            self.send(Cmd::SetAudible {
                layer: i,
                on: mix.audible(i) && l.preset.is_some() && self.layer_ready(i),
            });
        }
        self.send(Cmd::SetMaster(self.cfg.master));
        self.send(Cmd::SetTranspose(mix.transpose));
        self.send(Cmd::PitchRange(self.cfg.pitch_range));
        self.send(Cmd::PadVolume(self.cfg.pad_volume));
        self.send_vivo();
        self.send_efeitos();
        self.send_pad_mix();
    }

    fn send_vivo(&self) {
        let v = &self.cfg.vivo;
        self.send(Cmd::GrainParams(GrainParams {
            bright: v.bright,
            shimmer: v.shimmer,
            motion: v.motion,
        }));
        self.send(Cmd::Church {
            pads: v.church_pads,
            sf2: v.church_sf2,
        });
    }

    /// Ajustes do Pad Vivo / reverb de igreja mudaram (na tela).
    pub fn set_vivo(&mut self, v: VivoCfg) {
        if v != self.cfg.vivo {
            self.cfg.vivo = v;
            self.cfg.normalize();
            self.send_vivo();
            self.dirty = true;
        }
    }

    // ---------- Pad Vivo (granular) ----------

    fn request_texture(&mut self, n: usize) {
        if self.textures[n].is_some() || !self.tex_jobs.insert(n) {
            return;
        }
        let custom = self
            .cfg
            .vivo
            .files
            .iter()
            .find(|(k, _, _)| *k == n)
            .map(|(_, p, r)| (p.clone(), *r));
        let job: PadJob = (
            PAD_BANKS + n,
            custom.as_ref().map(|c| c.1 as usize).unwrap_or(0),
            custom.map(|c| c.0),
        );
        match &self.pad_loader {
            Some(tx) => {
                let _ = tx.send(job);
            }
            None => {
                let r = load_job_texture(&job);
                self.texture_loaded(n, r);
            }
        }
    }

    pub fn texture_loaded(&mut self, n: usize, result: Result<Arc<GrainSource>, String>) {
        self.tex_jobs.remove(&n);
        match result {
            Ok(t) => {
                self.textures[n] = Some(t);
                if let Some((bank, key)) = self.pad_pending {
                    if bank == PAD_BANKS + n {
                        self.start_pad(key);
                    }
                }
            }
            Err(e) => {
                if self.cfg.vivo.files.iter().any(|(k, _, _)| *k == n) {
                    self.cfg.vivo.files.retain(|(k, _, _)| *k != n);
                    self.dirty = true;
                    self.notify(format!(
                        "{}: {e}. Voltei ao som padrão.",
                        pads::VIVO_NAMES[n]
                    ));
                    self.request_texture(n);
                } else {
                    self.notify(format!("{}: {e}", pads::VIVO_NAMES[n]));
                }
            }
        }
        self.request_repaint();
    }

    /// Usa um som do Theo como base do Vivo `n` (None = volta ao padrão). `root` = nota do arquivo.
    pub fn set_vivo_file(&mut self, n: usize, file: Option<(String, u8)>) {
        self.cfg.vivo.files.retain(|(k, _, _)| *k != n);
        if let Some((p, root)) = file {
            self.cfg.vivo.files.push((n, p, root));
        }
        self.dirty = true;
        self.textures[n] = None;
        self.request_texture(n);
        if let Some(k) = self.pad_active {
            if self.cfg.pad_bank == PAD_BANKS + n {
                self.pad_pending = Some((PAD_BANKS + n, k));
            }
        }
    }

    /// A placa de som abriu com esta taxa de amostragem.
    pub fn set_sample_rate(&mut self, sr: u32) {
        let changed = sr != self.sample_rate;
        self.sample_rate = sr;
        self.send(Cmd::SetSampleRate(sr));
        if changed {
            self.rebuild_synth();
        }
    }

    // ---------- Meus pads (um arquivo em todos os tons, até 4, com nome) ----------

    fn request_my_pad(&mut self, i: usize) {
        let Some(path) = self.cfg.my_pads.get(i).map(|m| m.path.clone()) else {
            return;
        };
        if self.my_pad_clips[i].is_some() || !self.pad_jobs.insert((MY_BANK + i, 0)) {
            return;
        }
        match &self.pad_loader {
            Some(tx) => {
                let _ = tx.send((MY_BANK + i, 0, Some(path)));
            }
            None => {
                let r = pads::load_my_pad(&path);
                self.my_pad_loaded(i, &path, r);
            }
        }
    }

    /// `path` = arquivo que foi carregado (se o Theo trocou no meio, o resultado velho é jogado fora).
    pub fn my_pad_loaded(&mut self, i: usize, path: &str, result: Result<Arc<PadClip>, String>) {
        self.pad_jobs.remove(&(MY_BANK + i, 0));
        if self.cfg.my_pads.get(i).map(|m| m.path.as_str()) != Some(path) {
            self.request_my_pad(i);
            return;
        }
        match result {
            Ok(clip) => {
                let m = &mut self.cfg.my_pads[i];
                if m.auto_key {
                    m.key = pads::detect_key(&clip);
                    m.auto_key = false;
                    self.dirty = true;
                    let msg = format!("{}: o som está em {}", m.name, pads::KEY_NAMES[m.key]);
                    self.notify(msg);
                }
                self.my_pad_clips[i] = Some(clip);
                if let Some(k) = self.pad_pending2 {
                    if self.cfg.pad_bank2 == Some(MY_BANK + i) && self.pad_active == Some(k) {
                        self.start_pad2(k);
                    }
                }
                if self.pad_pending.is_some_and(|(b, _)| b == MY_BANK + i) {
                    let key = self.pad_pending.unwrap().1;
                    self.start_pad(key);
                }
            }
            Err(e) => {
                let name = self.cfg.my_pads[i].name.clone();
                self.notify(format!("{name}: {e}"));
                self.remove_my_pad(i);
            }
        }
        self.request_repaint();
    }

    /// Põe um arquivo novo nos "Meus pads" (vira o estilo escolhido). Cheio (4) = troca o último.
    /// `key` = None: o programa descobre o tom pelo som.
    pub fn add_my_pad(&mut self, path: String, key: Option<usize>) {
        let name = pads::name_from_file(&path);
        if self.cfg.my_pads.len() >= pads::MY_SLOTS {
            self.cfg.my_pads.pop();
        }
        self.cfg.my_pads.push(MyPad {
            name,
            path,
            key: key.unwrap_or(0) % PAD_KEYS,
            auto_key: key.is_none(),
        });
        let i = self.cfg.my_pads.len() - 1;
        self.my_pad_clips[i] = None;
        self.dirty = true;
        self.cfg.pad_bank = MY_BANK + i;
        self.request_my_pad(i);
        if let Some(k) = self.pad_active {
            self.start_pad(k);
        }
    }

    /// Põe de volta um pad da parceria que foi tirado.
    pub fn restore_partner_pad(&mut self, n: usize) {
        let Some(p) = pads::PARTNER_PADS.get(n) else {
            return;
        };
        let path = pads::partner_path(n);
        if self.cfg.my_pads.len() >= pads::MY_SLOTS
            || self.cfg.my_pads.iter().any(|m| m.path == path)
        {
            return;
        }
        // v0.21: volta já com o nome de planeta
        let name = pads::planet_name(&path).map(|(n, _)| n).unwrap_or(p.name);
        self.cfg.my_pads.push(MyPad {
            name: name.into(),
            path,
            key: p.key,
            auto_key: false,
        });
        let i = self.cfg.my_pads.len() - 1;
        self.my_pad_clips[i] = None;
        self.dirty = true;
        self.cfg.pad_bank = MY_BANK + i;
        self.request_my_pad(i);
        if let Some(k) = self.pad_active {
            self.start_pad(k);
        }
    }

    /// Tira um dos "Meus pads" (os de depois sobem uma posição).
    pub fn remove_my_pad(&mut self, i: usize) {
        if i >= self.cfg.my_pads.len() {
            return;
        }
        let was = pads::my_slot(self.cfg.pad_bank);
        self.cfg.my_pads.remove(i);
        self.my_pad_clips.remove(i);
        self.my_pad_clips.push(None);
        self.dirty = true;
        match was {
            Some(j) if j == i => {
                self.pad_stop();
                self.cfg.pad_bank = 0;
            }
            Some(j) if j > i => self.cfg.pad_bank -= 1,
            _ => {}
        }
        match self.cfg.pad_bank2.and_then(pads::my_slot) {
            Some(j) if j == i => self.cfg.pad_bank2 = None,
            Some(j) if j > i => self.cfg.pad_bank2 = Some(MY_BANK + j - 1),
            _ => {}
        }
        self.send_pad_mix();
    }

    pub fn rename_my_pad(&mut self, i: usize, name: &str) {
        if let Some(m) = self.cfg.my_pads.get_mut(i) {
            if m.name != name {
                m.name = name.chars().take(28).collect();
                self.dirty = true;
            }
        }
    }

    /// Estilos de pad do programa (Escuro, Coral, Suave, Vivos) estão na tela? (v0.13: escondidos,
    /// ficam só os "Meus pads"; voltam pelo ✏ Editar.)
    pub fn builtin_pads(&self) -> bool {
        !self.cfg.is_hidden("pad_styles")
    }

    /// Nome do estilo para a tela (os "Meus pads" aparecem com o nome que o Theo deu).
    pub fn pad_bank_label(&self, bank: usize) -> String {
        match pads::my_slot(bank % ALL_BANKS).and_then(|i| self.cfg.my_pads.get(i)) {
            Some(m) => m.name.clone(),
            None => pads::bank_name(bank).to_string(),
        }
    }

    /// Corrige em que tom o som do "Meu pad" `i` está.
    pub fn set_my_pad_key(&mut self, i: usize, key: usize) {
        let Some(m) = self.cfg.my_pads.get_mut(i) else {
            return;
        };
        if m.key == key % PAD_KEYS {
            return;
        }
        m.key = key % PAD_KEYS;
        m.auto_key = false;
        self.dirty = true;
        if let Some(k) = self.pad_active.filter(|_| self.cfg.pad_bank == MY_BANK + i) {
            self.start_pad(k);
        }
    }

    // ---------- Pads ----------

    fn request_pad(&mut self, bank: usize, key: usize) {
        if self.pad_clips[bank][key].is_some() || !self.pad_jobs.insert((bank, key)) {
            return;
        }
        let custom = self.cfg.pad_file(bank, key).map(str::to_string);
        match &self.pad_loader {
            Some(tx) => {
                let _ = tx.send((bank, key, custom));
            }
            None => {
                let r = pads::load(bank, key, custom.as_deref());
                self.pad_loaded(bank, key, r);
            }
        }
    }

    /// Deixa os 12 tons do banco prontos (em segundo plano).
    pub fn preload_pad_bank(&mut self, bank: usize) {
        if let Some(i) = pads::my_slot(bank % ALL_BANKS) {
            self.request_my_pad(i);
            return;
        }
        if !self.builtin_pads() {
            return; // escondidos: nem ocupam memória
        }
        if pads::is_vivo(bank % ALL_BANKS) {
            self.request_texture(bank % ALL_BANKS - PAD_BANKS);
            return;
        }
        for k in 0..PAD_KEYS {
            self.request_pad(bank % PAD_BANKS, k);
        }
    }

    /// Chamado quando um pad terminou de decodificar.
    pub fn pad_loaded(&mut self, bank: usize, key: usize, result: Result<Arc<PadClip>, String>) {
        self.pad_jobs.remove(&(bank, key));
        match result {
            Ok(clip) => {
                self.pad_clips[bank][key] = Some(clip);
                if self.pad_pending == Some((bank, key)) {
                    self.start_pad(key);
                }
            }
            Err(e) => {
                if self.cfg.pad_file(bank, key).is_some() {
                    // Arquivo próprio com problema: volta ao som padrão.
                    self.cfg
                        .pad_files
                        .retain(|f| !(f.bank == bank && f.key == key));
                    self.dirty = true;
                    self.notify(format!(
                        "Pad {}: {e}. Voltei ao som padrão.",
                        pads::KEY_NAMES[key]
                    ));
                    self.request_pad(bank, key);
                } else {
                    self.notify(format!("Pad {}: {e}", pads::KEY_NAMES[key]));
                    if self.pad_pending == Some((bank, key)) {
                        self.pad_pending = None;
                        self.pad_active = None;
                    }
                }
            }
        }
        self.request_repaint();
    }

    /// Toca o pad no tom `key`: o principal e, se tiver, o segundo pad junto (v0.24).
    fn start_pad(&mut self, key: usize) {
        if self.start_pad1(key) {
            self.start_pad2(key);
        }
    }

    /// v0.24: o segundo pad no mesmo tom (só depois que o principal já saiu, para entrarem juntos).
    fn start_pad2(&mut self, key: usize) {
        self.pad_pending2 = None;
        let Some(i) = self.cfg.pad_bank2.and_then(pads::my_slot) else {
            return;
        };
        match (self.my_pad_clips.get(i).cloned().flatten(), self.cfg.my_pads.get(i)) {
            (Some(clip), Some(m)) => {
                let (c, pitch) = pads::clip_for_key(&clip, m.key, key);
                self.send(Cmd::PadLayer2(c, pitch));
            }
            (None, Some(_)) => {
                self.pad_pending2 = Some(key);
                self.request_my_pad(i);
            }
            _ => {}
        }
    }

    /// Volume de cada pad no mix (equal power: no meio, os dois a ~71%, sem ficar mais alto que um só).
    fn pad_mix_gains(&self) -> (f32, f32) {
        if self.cfg.pad_bank2.is_none() {
            return (1.0, 0.0);
        }
        let a = self.cfg.pad_mix.clamp(0.0, 1.0) * std::f32::consts::FRAC_PI_2;
        (a.cos(), a.sin())
    }

    fn send_pad_mix(&self) {
        let (a, b) = self.pad_mix_gains();
        self.send(Cmd::PadMix(a, b));
    }

    /// v0.24: escolhe o segundo pad (None = só um pad). Se já estiver tocando, entra junto na hora.
    pub fn set_pad_bank2(&mut self, bank: Option<usize>) {
        let bank = bank.filter(|b| {
            *b != self.cfg.pad_bank && pads::my_slot(*b).is_some_and(|i| i < self.cfg.my_pads.len())
        });
        if bank == self.cfg.pad_bank2 {
            return;
        }
        self.cfg.pad_bank2 = bank;
        self.dirty = true;
        self.send_pad_mix();
        if let Some(i) = bank.and_then(pads::my_slot) {
            self.request_my_pad(i);
        }
        if let Some(k) = self.pad_active {
            self.start_pad(k);
        }
        match bank {
            Some(b) => self.notify(format!(
                "Pads: {} + {} juntos (ajuste o mix)",
                self.pad_bank_label(self.cfg.pad_bank),
                self.pad_bank_label(b)
            )),
            None => self.notify(format!("Pads: só {}", self.pad_bank_label(self.cfg.pad_bank))),
        }
    }

    /// v0.24: mix entre os dois pads (0 = só o principal … 1 = só o segundo).
    pub fn set_pad_mix(&mut self, v: f32) {
        self.cfg.pad_mix = v.clamp(0.0, 1.0);
        self.dirty = true;
        self.send_pad_mix();
    }

    /// Devolve true se mandou o pad principal tocar (false = ainda carregando ou sem pad).
    fn start_pad1(&mut self, key: usize) -> bool {
        if self.cfg.pad_bank < MY_BANK && !self.builtin_pads() {
            if self.cfg.my_pads.is_empty() {
                self.pad_active = None;
                self.notify(
                    "Pads: ponha um pad seu em \"+ Pôr um pad meu\" (aba Pads).".to_string(),
                );
                return false;
            }
            self.cfg.pad_bank = MY_BANK;
            self.dirty = true;
        }
        let bank = self.cfg.pad_bank;
        self.pad_active = Some(key);
        if let Some(i) = pads::my_slot(bank) {
            match (self.my_pad_clips[i].clone(), self.cfg.my_pads.get(i)) {
                (Some(clip), Some(m)) => {
                    self.pad_pending = None;
                    let (c, pitch) = pads::clip_for_key(&clip, m.key, key);
                    self.send(Cmd::PadPlay(c, pitch));
                    return true;
                }
                (None, Some(_)) => {
                    self.pad_pending = Some((bank, key));
                    self.request_my_pad(i);
                }
                _ => {
                    self.pad_active = None;
                    self.notify("Meu pad: escolha o arquivo primeiro (aba Pads).".to_string());
                }
            }
            return false;
        }
        if pads::is_vivo(bank) {
            let n = bank - PAD_BANKS;
            match self.textures[n].clone() {
                Some(tex) => {
                    self.pad_pending = None;
                    self.grain_seed = self
                        .grain_seed
                        .wrapping_mul(1_664_525)
                        .wrapping_add(1_013_904_223);
                    let voice = GranularVoice::new(
                        tex,
                        pads::key_midi(key),
                        self.sample_rate,
                        self.grain_seed,
                    );
                    self.send(Cmd::GrainPlay(Box::new(voice)));
                    return true;
                }
                None => {
                    self.pad_pending = Some((bank, key));
                    self.request_texture(n);
                }
            }
            return false;
        }
        match self.pad_clips[bank][key].clone() {
            Some(clip) => {
                self.pad_pending = None;
                self.send(Cmd::PadPlay(clip, 1.0));
                true
            }
            None => {
                self.pad_pending = Some((bank, key));
                self.request_pad(bank, key);
                false
            }
        }
    }

    /// Botão de um tom: liga; se já está tocando esse tom, para.
    pub fn pad_press(&mut self, key: usize) {
        if self.pad_active == Some(key) {
            self.pad_stop();
        } else {
            self.start_pad(key % PAD_KEYS);
        }
    }

    pub fn pad_stop(&mut self) {
        self.pad_active = None;
        self.pad_pending = None;
        self.pad_pending2 = None;
        self.send(Cmd::PadStop);
    }

    pub fn pad_step(&mut self, delta: i32) {
        let cur = self
            .pad_active
            .map(|k| k as i32)
            .unwrap_or(if delta > 0 { -1 } else { 0 });
        let next = (cur + delta).rem_euclid(PAD_KEYS as i32) as usize;
        self.start_pad(next);
    }

    pub fn set_pad_volume(&mut self, v: f32) {
        self.cfg.pad_volume = v.clamp(0.0, 1.0);
        self.send(Cmd::PadVolume(self.cfg.pad_volume));
        self.dirty = true;
    }

    pub fn set_pad_bank(&mut self, bank: usize) {
        let mut bank = bank % ALL_BANKS;
        if pads::my_slot(bank).is_some_and(|i| i >= self.cfg.my_pads.len()) {
            bank = 0; // só entram na volta dos estilos os "Meus pads" que têm arquivo
        }
        if bank < MY_BANK && !self.builtin_pads() && !self.cfg.my_pads.is_empty() {
            bank = MY_BANK; // estilos do programa escondidos: a volta é só entre os pads do Theo
        }
        if bank == self.cfg.pad_bank {
            return;
        }
        self.cfg.pad_bank = bank;
        if self.cfg.pad_bank2 == Some(bank) {
            self.cfg.pad_bank2 = None;
            self.send_pad_mix();
        }
        self.dirty = true;
        self.preload_pad_bank(bank);
        if let Some(k) = self.pad_active {
            self.start_pad(k); // troca suave para o mesmo tom no outro banco
        }
        self.notify(format!("Pads: {}", self.pad_bank_label(bank)));
    }

    /// Põe um arquivo próprio num botão (None = volta ao som padrão).
    pub fn set_pad_file(&mut self, bank: usize, key: usize, path: Option<String>) {
        if bank >= PAD_BANKS {
            return; // Vivo e Meu pad usam um arquivo só para todos os tons
        }
        self.cfg
            .pad_files
            .retain(|f| !(f.bank == bank && f.key == key));
        if let Some(p) = path {
            self.cfg.pad_files.push(PadFile { bank, key, path: p });
        }
        self.dirty = true;
        self.pad_clips[bank][key] = None;
        self.request_pad(bank, key);
        if self.pad_active == Some(key) && self.cfg.pad_bank == bank {
            self.pad_pending = Some((bank, key));
        }
    }

    pub fn pad_ready(&self, bank: usize, key: usize) -> bool {
        if let Some(i) = pads::my_slot(bank) {
            return self.my_pad_clips[i].is_some();
        }
        if pads::is_vivo(bank) {
            return self.textures[bank - PAD_BANKS].is_some();
        }
        self.pad_clips[bank][key].is_some()
    }

    fn sync_audible(&self) {
        let mix = &self.cfg.mix;
        for i in 0..LAYERS {
            let on = mix.audible(i) && mix.layers[i].preset.is_some() && self.layer_ready(i);
            self.send(Cmd::SetAudible { layer: i, on });
        }
    }

    // ---------- v0.35: Biblioteca (timbre de outro arquivo em cada camada) ----------

    /// O som da camada já está pronto no motor? (timbre da Biblioteca precisa do arquivo aberto)
    pub fn layer_ready(&self, i: usize) -> bool {
        match self.cfg.mix.layers[i].preset.as_ref().and_then(|p| p.file.as_ref()) {
            None => true,
            Some(f) => self.layer_font[i].as_ref() == Some(f),
        }
    }

    /// Algum arquivo da Biblioteca abrindo para uma camada (a tela mostra "abrindo...").
    pub fn layer_loading(&self, i: usize) -> bool {
        self.cfg.mix.layers[i]
            .preset
            .as_ref()
            .and_then(|p| p.file.as_ref())
            .is_some_and(|f| self.lib_loading.contains(f))
    }

    /// Junta os timbres do SF2 principal com os da Biblioteca.
    fn merge_presets(&mut self) {
        let mut all = self.main_presets.clone();
        all.extend(self.lib_presets.iter().cloned());
        self.presets = all;
    }

    /// Timbres achados na pasta da Biblioteca (o programa procura ao abrir e depois de baixar).
    pub fn set_library(&mut self, lib: Vec<crate::biblioteca::LibPreset>) {
        self.lib_presets = lib
            .into_iter()
            .map(|p| PresetRef { bank: p.bank, patch: p.patch, name: p.name, file: Some(p.rel) })
            .collect();
        self.merge_presets();
        // nomes guardados atualizados; timbre de arquivo que sumiu fica sem som (não trava nada)
        for l in &mut self.cfg.mix.layers {
            if let Some(p) = &mut l.preset {
                if p.file.is_some() {
                    if let Some(q) = self.lib_presets.iter().find(|q| q.same(p)) {
                        p.name = q.name.clone();
                    }
                }
            }
        }
        self.ensure_layer_fonts();
    }

    /// v0.35: os 6 efeitos do Mobile FX Worship entram nos DrumPads 11 a 16 (uma vez só, em pads vazios).
    pub fn add_library_drums(&mut self) {
        if self.cfg.lib_drums_added {
            return;
        }
        let ready = crate::biblioteca::make_lib_drums();
        if ready.len() < crate::biblioteca::LIB_DRUMS.len() {
            return;
        }
        let e = &mut self.cfg.efeitos;
        let first = crate::efeitos::BUILTIN_DRUMS; // depois do Choir Rise e do Prayer
        for (k, (name, path)) in ready.into_iter().enumerate() {
            let pad = first + k;
            if pad >= crate::efeitos::MAX_DRUMS || e.drum_files.iter().any(|(i, _)| *i == pad) {
                continue;
            }
            e.drum_files.push((pad, path.to_string_lossy().to_string()));
            if e.drum_names.len() <= pad {
                e.drum_names.resize(pad + 1, String::new());
            }
            if e.drum_names[pad].trim().is_empty() {
                e.drum_names[pad] = name.to_string();
            }
        }
        e.drum_count = crate::efeitos::MAX_DRUMS;
        e.normalize();
        self.cfg.lib_drums_added = true;
        self.dirty = true;
    }

    /// v0.35: os pads da Biblioteca (Sol, Lua, Mercúrio...) entram nos "Meus pads" uma vez só,
    /// quando o arquivo deles já está no computador e tem vaga.
    pub fn add_library_pads(&mut self) {
        for (name, path) in pads::LIB_PADS {
            if self.cfg.lib_pads_added.iter().any(|p| p == path) || !pads::lib_pad_available(path) {
                continue;
            }
            if self.cfg.my_pads.iter().any(|m| m.path == path) {
                self.cfg.lib_pads_added.push(path.to_string());
                continue;
            }
            if self.cfg.my_pads.len() >= pads::MY_SLOTS {
                break;
            }
            self.cfg.my_pads.push(MyPad { name: name.into(), path: path.into(), key: 0, auto_key: false });
            self.cfg.lib_pads_added.push(path.to_string());
            let i = self.cfg.my_pads.len() - 1;
            self.my_pad_clips[i] = None;
            self.request_my_pad(i);
            self.dirty = true;
        }
    }

    /// Deixa cada camada com o sintetizador do arquivo certo: abre o que falta (em segundo plano)
    /// e tira da memória o que nenhuma camada usa mais.
    pub fn ensure_layer_fonts(&mut self) {
        let wanted: Vec<Option<String>> = self
            .cfg
            .mix
            .layers
            .iter()
            .map(|l| l.preset.as_ref().and_then(|p| p.file.clone()))
            .collect();
        self.lib_fonts.retain(|k, _| wanted.iter().any(|w| w.as_deref() == Some(k.as_str())));
        for (i, want) in wanted.iter().enumerate().take(LAYERS) {
            if self.layer_font[i] == *want {
                continue;
            }
            match want {
                None => {
                    self.layer_font[i] = None;
                    self.send(Cmd::LayerSynth { layer: i, synth: None });
                }
                Some(f) => match self.lib_fonts.get(f) {
                    Some(sf) => {
                        if let Some(s) = Engine::build_synth(sf, self.sample_rate, self.cfg.polyphony) {
                            self.layer_font[i] = Some(f.clone());
                            self.send(Cmd::LayerSynth { layer: i, synth: Some(s) });
                        }
                    }
                    None => self.request_font(f.clone()),
                },
            }
        }
        self.sync_audible();
    }

    fn request_font(&mut self, rel: String) {
        if !self.lib_loading.insert(rel.clone()) {
            return;
        }
        match &self.font_loader {
            Some(tx) => {
                let _ = tx.send(rel);
            }
            None => {
                let r = load_font_file(&rel);
                self.font_loaded(rel, r);
            }
        }
    }

    /// Um arquivo da Biblioteca terminou de abrir.
    pub fn font_loaded(&mut self, rel: String, result: Result<Arc<SoundFont>, String>) {
        self.lib_loading.remove(&rel);
        match result {
            Ok(sf) => {
                let used = self.cfg.mix.layers.iter().any(|l| l.preset.as_ref().and_then(|p| p.file.as_ref()) == Some(&rel));
                if used {
                    self.lib_fonts.insert(rel, sf);
                    self.ensure_layer_fonts();
                }
            }
            Err(e) => self.notify(format!("{rel}: {e}")),
        }
        self.request_repaint();
    }

    // ---------- Ações (tela, mesinha e teclado chamam estas) ----------

    pub fn set_volume(&mut self, i: usize, v: f32) {
        let v = v.clamp(0.0, 1.0);
        self.cfg.mix.layers[i].volume = v;
        self.send(Cmd::SetVolume { layer: i, value: v });
        self.dirty = true;
    }

    /// v0.29: Cutoff (brilho) da camada.
    pub fn set_cutoff(&mut self, i: usize, v: f32) {
        let v = v.clamp(0.0, 1.0);
        self.cfg.mix.layers[i].cutoff = v;
        self.send(Cmd::SetCutoff { layer: i, value: v });
        self.dirty = true;
    }

    pub fn set_reverb(&mut self, i: usize, v: f32) {
        let v = v.clamp(0.0, 1.0);
        self.cfg.mix.layers[i].reverb = v;
        self.send(Cmd::SetReverb { layer: i, value: v });
        self.dirty = true;
    }

    pub fn set_on(&mut self, i: usize, on: bool) {
        self.cfg.mix.layers[i].on = on;
        self.sync_audible();
        self.dirty = true;
    }

    pub fn toggle(&mut self, i: usize) {
        let on = !self.cfg.mix.layers[i].on;
        if on && self.cfg.mix.layers[i].preset.is_none() {
            if let Some(p) = self.presets.first().cloned() {
                self.set_preset(i, p);
            }
        }
        self.set_on(i, on);
    }

    pub fn toggle_solo(&mut self, i: usize) {
        let l = &mut self.cfg.mix.layers[i];
        l.solo = !l.solo;
        self.sync_audible();
        self.dirty = true;
    }

    pub fn set_master(&mut self, v: f32) {
        self.cfg.master = v.clamp(0.0, 1.0);
        self.send(Cmd::SetMaster(self.cfg.master));
        self.dirty = true;
    }

    pub fn set_preset(&mut self, i: usize, p: PresetRef) {
        self.send(Cmd::SetPreset {
            layer: i,
            bank: p.bank,
            patch: p.patch,
        });
        self.cfg.mix.layers[i].preset = Some(p);
        // v0.35: timbre de outro arquivo → a camada ganha o sintetizador dele (sync_audible vem junto)
        self.ensure_layer_fonts();
        self.dirty = true;
    }

    /// Troca para o timbre anterior/próximo da lista do SF2.
    pub fn step_preset(&mut self, i: usize, delta: i32) {
        if self.presets.is_empty() {
            return;
        }
        let cur = self.cfg.mix.layers[i].preset.as_ref().and_then(|p| {
            self.presets.iter().position(|q| q.same(p))
        });
        let n = self.presets.len() as i32;
        let next = match cur {
            Some(c) => (c as i32 + delta).rem_euclid(n),
            None => 0,
        };
        let p = self.presets[next as usize].clone();
        self.set_preset(i, p);
    }

    pub fn set_transpose(&mut self, t: i32) {
        self.cfg.mix.transpose = t.clamp(-12, 12);
        self.send(Cmd::SetTranspose(self.cfg.mix.transpose));
        self.dirty = true;
    }

    /// Teclado da tela: toca/solta uma nota como se viesse do piano.
    pub fn note_on(&mut self, key: u8, vel: u8) {
        mark_note(key, true);
        self.send(Cmd::NoteOn {
            key: key.min(127),
            vel: vel.clamp(1, 127),
        });
    }

    pub fn note_off(&mut self, key: u8) {
        mark_note(key, false);
        self.send(Cmd::NoteOff { key: key.min(127) });
    }

    pub fn sustain(&mut self, down: bool) {
        self.send(Cmd::Cc {
            num: 64,
            val: if down { 127 } else { 0 },
        });
    }

    pub fn panic(&mut self) {
        clear_held();
        // o motor zera pitch e modulação no pânico; a tela acompanha
        self.pitch = 0.0;
        self.modulation = 0.0;
        self.freeze_on = false; // o motor cala o congelado no pânico
        self.swell_in = true; // e o swell volta a "cheio"
        self.seq_on = false; // e a batida do Sequencer para
        self.send(Cmd::Panic);
        self.notify("Pânico: todas as notas cortadas");
    }

    pub fn load_scene(&mut self, n: usize) {
        if n >= self.cfg.scenes.len() {
            return;
        }
        match self.cfg.scenes[n].mix.clone() {
            Some(mix) => {
                self.cfg.mix = mix;
                self.cfg.mix.normalize();
                self.cfg.current_scene = Some(n);
                // v0.24: o resto do preset (cenas salvas antes da v0.24 não têm: só as camadas mudam)
                let extra = self.cfg.scenes[n].extra.clone();
                if let Some(x) = &extra {
                    self.apply_scene_extra(x);
                }
                self.sync_engine();
                self.ensure_layer_fonts();
                if let Some(x) = extra {
                    // pads por último (troca suave se estiver tocando)
                    self.set_pad_bank(x.pad_bank);
                    self.cfg.pad_mix = x.pad_mix.clamp(0.0, 1.0);
                    if x.pad_bank2 != self.cfg.pad_bank2 {
                        self.set_pad_bank2(x.pad_bank2);
                    } else {
                        self.send_pad_mix();
                    }
                }
                self.notify(format!("{} carregada", self.cfg.scenes[n].name));
            }
            None => {
                self.cfg.current_scene = Some(n);
                self.notify(format!(
                    "{} está vazia: monte os sons e clique em Salvar cena",
                    self.cfg.scenes[n].name
                ));
            }
        }
        self.dirty = true;
    }

    pub fn save_scene(&mut self, n: usize) {
        if n >= self.cfg.scenes.len() {
            return;
        }
        self.cfg.scenes[n].mix = Some(self.cfg.mix.clone());
        // v0.24: a cena guarda tudo (o preset inteiro), não só as camadas
        self.cfg.scenes[n].extra = Some(SceneExtra {
            efeitos: self.cfg.efeitos.clone(),
            pad_bank: self.cfg.pad_bank,
            pad_bank2: self.cfg.pad_bank2,
            pad_mix: self.cfg.pad_mix,
            pad_volume: self.cfg.pad_volume,
            master: self.cfg.master,
            vivo: self.cfg.vivo.clone(),
        });
        self.cfg.current_scene = Some(n);
        self.dirty = true;
        self.notify(format!("Salvo em {} (sons, efeitos, pads e filtro)", self.cfg.scenes[n].name));
    }

    /// v0.24: aplica efeitos, volumes e reverb guardados numa cena (os pads vêm depois, em `load_scene`).
    fn apply_scene_extra(&mut self, x: &SceneExtra) {
        self.cfg.master = x.master.clamp(0.0, 1.0);
        self.cfg.pad_volume = x.pad_volume.clamp(0.0, 1.0);
        self.cfg.vivo = x.vivo.clone();
        let voice_changed = x.efeitos.voicefx_voice != self.cfg.efeitos.voicefx_voice
            || x.efeitos.voicefx_file != self.cfg.efeitos.voicefx_file;
        self.cfg.efeitos = x.efeitos.clone();
        self.cfg.efeitos.normalize();
        self.pulse_on = self.cfg.efeitos.pulse_on;
        self.arp_on = self.cfg.efeitos.arp_on;
        let want_voice = self.cfg.efeitos.voicefx_on;
        let loaded = |c: &Self| c.voicefx_src.is_some() || c.voicefx_tex.is_some();
        if want_voice && (voice_changed || !loaded(self)) {
            self.load_voicefx_source();
        } else if voice_changed {
            self.voicefx_src = None; // carrega a voz certa quando ligar
            if self.voicefx_tex.take().is_some() {
                self.send(Cmd::VoiceFxTexture(None));
            }
        }
        self.send(Cmd::VoiceFxKey(self.cfg.efeitos.voicefx_key as u8));
        self.voicefx_on = want_voice && loaded(self);
        self.cfg.efeitos.voicefx_on = self.voicefx_on;
    }

    // ---------- Modo Editar ----------

    /// Cria uma cena nova (vazia) no fim.
    pub fn add_scene(&mut self) {
        if self.cfg.scenes.len() >= MAX_SCENES {
            self.notify(format!("Máximo de {MAX_SCENES} cenas"));
            return;
        }
        let n = self.cfg.scenes.len();
        self.cfg.scenes.push(Scene {
            name: format!("Cena {}", n + 1),
            mix: None,
            extra: None,
        });
        self.dirty = true;
    }

    /// Apaga uma cena; as de depois sobem uma posição (e o que a mesinha chamava acompanha).
    pub fn delete_scene(&mut self, n: usize) {
        if n >= self.cfg.scenes.len() || self.cfg.scenes.len() <= 1 {
            return;
        }
        let name = self.cfg.scenes.remove(n).name;
        self.cfg.current_scene = match self.cfg.current_scene {
            Some(c) if c == n => None,
            Some(c) if c > n => Some(c - 1),
            other => other,
        };
        self.cfg.mappings.retain(|m| m.target != Target::Scene(n));
        for m in &mut self.cfg.mappings {
            if let Target::Scene(k) = m.target {
                if k > n {
                    m.target = Target::Scene(k - 1);
                }
            }
        }
        self.dirty = true;
        self.notify(format!("\"{name}\" apagada"));
    }

    /// Tira/coloca algo da tela. Camada tirada também para de tocar.
    pub fn set_hidden(&mut self, id: &str, hidden: bool) {
        self.cfg.set_hidden(id, hidden);
        if hidden {
            if let Some(i) = (0..LAYERS).find(|i| layer_id(*i) == id) {
                if self.cfg.mix.layers[i].on || self.cfg.mix.layers[i].solo {
                    self.cfg.mix.layers[i].solo = false;
                    self.set_on(i, false);
                }
            }
            if id == "tab_pads" && self.cfg.tab == 1 {
                self.cfg.tab = 0;
            }
        }
        self.dirty = true;
    }

    pub fn step_scene(&mut self, delta: i32) {
        let cur = self
            .cfg
            .current_scene
            .map(|c| c as i32)
            .unwrap_or(if delta > 0 { -1 } else { 0 });
        let next = (cur + delta).rem_euclid(self.cfg.scenes.len().max(1) as i32) as usize;
        self.load_scene(next);
    }

    pub fn reset_mappings(&mut self) {
        self.cfg.mappings = smc_default_mappings();
        self.dirty = true;
        self.notify("Controles da SMC-Mixer voltaram ao padrão");
    }

    fn apply_button(&mut self, t: Target) {
        if let Target::Toggle(i) | Target::Solo(i) = t {
            if i >= LAYERS {
                return;
            }
        }
        match t {
            Target::Toggle(i) => self.toggle(i),
            Target::Solo(i) => self.toggle_solo(i),
            Target::PrevScene => self.step_scene(-1),
            Target::NextScene => self.step_scene(1),
            Target::Scene(n) => self.load_scene(n),
            Target::Panic => self.panic(),
            Target::TransposeDown => self.set_transpose(self.cfg.mix.transpose - 1),
            Target::TransposeUp => self.set_transpose(self.cfg.mix.transpose + 1),
            Target::Pad(k) => self.pad_press(k),
            Target::PadStop => self.pad_stop(),
            Target::PadPrev => self.pad_step(-1),
            Target::PadNext => self.pad_step(1),
            Target::PadBank => self.set_pad_bank(self.cfg.pad_bank + 1),
            Target::Freeze => self.freeze_toggle(),
            Target::TapTempo => self.tap_tempo(),
            Target::PulseToggle => self.toggle_pulse(),
            Target::ArpToggle => self.toggle_arp(),
            Target::VoiceFxToggle => self.toggle_voicefx(),
            Target::Drum(i) => self.drum_hit(i, 110),
            Target::SwellToggle => self.toggle_swell(),
            Target::FreezeRecall => self.freeze_recall(),
            Target::SeqToggle => self.toggle_seq(),
            Target::TabToggle => {
                self.cfg.tab = 1 - self.cfg.tab.min(1);
                self.dirty = true;
            }

            _ => {}
        }
    }

    fn apply_continuous(&mut self, t: Target, value: ContValue) {
        if let Target::Volume(i) | Target::Reverb(i) = t {
            if i >= LAYERS {
                return;
            }
        }
        if t == Target::PadKnob {
            match value {
                // Knob infinito: cada "clique" = um tom.
                ContValue::Relative(d) if d > 0.0 => self.pad_step(1),
                ContValue::Relative(d) if d < 0.0 => self.pad_step(-1),
                ContValue::Relative(_) => {}
                // Knob comum (0..127): a posição escolhe o tom (C..B).
                ContValue::Absolute(v) => {
                    let k = ((v.clamp(0.0, 1.0) * (PAD_KEYS as f32 - 1.0)).round()) as usize;
                    if self.pad_active != Some(k) {
                        self.start_pad(k);
                    }
                }
            }
            return;
        }
        if t == Target::Pitch {
            let v = match value {
                // roda/fader 0..1 → −1..1 (meio = afinado; perto do meio encaixa no zero)
                ContValue::Absolute(v) => {
                    let p = v.clamp(0.0, 1.0) * 2.0 - 1.0;
                    if p.abs() < 0.02 {
                        0.0
                    } else {
                        p
                    }
                }
                ContValue::Relative(d) => self.pitch + d * 2.0,
            };
            self.set_pitch(v);
            return;
        }
        if t == Target::PadMix {
            let v = match value {
                ContValue::Absolute(v) => v,
                ContValue::Relative(d) => self.cfg.pad_mix + d,
            };
            self.set_pad_mix(v);
            return;
        }
        if t == Target::Filter {
            let v = match value {
                ContValue::Absolute(v) => v,
                ContValue::Relative(d) => self.cfg.efeitos.filtro + d,
            };
            self.set_filter(v);
            return;
        }
        if matches!(t, Target::DrumVolume | Target::VoiceFxVolume) {
            let mut e = self.cfg.efeitos.clone();
            let cur = if t == Target::DrumVolume {
                &mut e.drum_volume
            } else {
                &mut e.voicefx_volume
            };
            *cur = match value {
                ContValue::Absolute(v) => v,
                ContValue::Relative(d) => *cur + d,
            }
            .clamp(0.0, 1.0);
            self.set_efeitos(e);
            return;
        }
        if t == Target::Modulation {
            let v = match value {
                ContValue::Absolute(v) => v,
                ContValue::Relative(d) => self.modulation + d,
            };
            self.set_modulation(v);
            return;
        }
        let cur = match t {
            Target::Volume(i) => self.cfg.mix.layers[i].volume,
            Target::Reverb(i) => self.cfg.mix.layers[i].reverb,
            Target::Cutoff(i) => self.cfg.mix.layers[i].cutoff,
            Target::Master => self.cfg.master,
            Target::PadVolume => self.cfg.pad_volume,
            _ => return,
        };
        let v = match value {
            ContValue::Absolute(v) => v,
            ContValue::Relative(d) => cur + d,
        };
        match t {
            Target::Volume(i) => self.set_volume(i, v),
            Target::Reverb(i) => self.set_reverb(i, v),
            Target::Cutoff(i) => self.set_cutoff(i, v),
            Target::Master => self.set_master(v),
            Target::PadVolume => self.set_pad_volume(v),
            _ => {}
        }
    }

    // ---------- MIDI ----------

    /// Mensagem vinda do piano: toca direto no motor.
    pub fn handle_piano(&mut self, msg: &[u8]) {
        let Some(&status) = msg.first() else { return };
        if self.piano_is_control(msg) {
            self.handle_piano_control(msg);
            return;
        }
        let d1 = msg.get(1).copied().unwrap_or(0) & 0x7F;
        let d2 = msg.get(2).copied().unwrap_or(0) & 0x7F;
        self.piano_activity = Some(Instant::now());
        match status & 0xF0 {
            0x90 if d2 > 0 => self.send(Cmd::NoteOn { key: d1, vel: d2 }),
            0x90 | 0x80 => self.send(Cmd::NoteOff { key: d1 }),
            0xB0 => self.send(Cmd::Cc { num: d1, val: d2 }),
            0xE0 => self.send(Cmd::PitchBend { lsb: d1, msb: d2 }),
            0xC0 if self.cfg.program_change_scenes && (d1 as usize) < self.cfg.scenes.len() => {
                self.load_scene(d1 as usize);
                self.request_repaint();
            }
            _ => {}
        }
    }

    /// Mensagem vinda da mesinha. Devolve true se algo mudou na tela.
    pub fn handle_controller(&mut self, msg: &[u8]) -> bool {
        self.handle_control(msg, false)
    }

    /// v0.15: mensagem do teclado (porta Piano) que é controle: o Aprender está esperando,
    /// ou ela foi ligada a um controle (ex.: botão do AMW = pitch). Não toca nota.
    pub fn handle_piano_control(&mut self, msg: &[u8]) -> bool {
        let changed = self.handle_control(msg, true);
        self.sync_piano_hooks();
        changed
    }

    /// Mesinha (`from_piano` = false) ou controles do teclado (true). Cada um tem a sua lista.
    fn handle_control(&mut self, msg: &[u8], from_piano: bool) -> bool {
        let Some(ev) = ControlEvent::parse(msg) else {
            return false;
        };
        if from_piano {
            self.piano_activity = Some(Instant::now());
            self.last_piano_signal = Some(ev.label());
        } else {
            self.controller_activity = Some(Instant::now());
            self.last_signal = Some(ev.label());
        }

        // Botão "apertado" agora? (CC: subiu de <64 para >=64; nota: velocity > 0)
        // `down` = está segurado (para os botões "de segurar" do pitch/modulation).
        let (pressed, down) = match ev {
            ControlEvent::Note { vel, .. } => (vel > 0, vel > 0),
            ControlEvent::Cc { ch, num, val } => {
                let key = (ch | if from_piano { 0x10 } else { 0 }, num);
                let prev = self.last_cc.insert(key, val).unwrap_or(0);
                (val >= 64 && prev < 64, val >= 64)
            }
            ControlEvent::PitchBend { .. } => (false, false),
        };

        if self.wizard.is_some() {
            // O assistente é só da mesinha.
            return !from_piano && self.wizard_signal(ev);
        }

        if self.learn.active {
            if let Some(t) = self.learn.target {
                if let Some(src) = ev.source_for(t, pressed) {
                    // Um controle faz uma coisa só. Fader/knob: um por alvo (dois faders brigariam).
                    // Botões: vários botões podem fazer a mesma coisa.
                    let single = t.is_continuous() && !matches!(src, Source::Note { .. });
                    let list = if from_piano {
                        &mut self.cfg.piano_mappings
                    } else {
                        &mut self.cfg.mappings
                    };
                    // Avisa o que saiu do lugar (para não "sumir" sem explicação).
                    let mut lost: Vec<String> = Vec::new();
                    for m in list.iter() {
                        if same_source(m.source, src) && m.target != t {
                            lost.push(format!(
                                "esse controle deixou de fazer \"{}\"",
                                m.target.label()
                            ));
                        } else if single
                            && m.target == t
                            && !same_source(m.source, src)
                            && !matches!(m.source, Source::Note { .. })
                        {
                            lost.push(format!("{} não comanda mais isso", m.source.label()));
                        }
                    }
                    list.retain(|m| {
                        !same_source(m.source, src)
                            && !(single
                                && m.target == t
                                && !matches!(m.source, Source::Note { .. }))
                    });
                    list.push(Mapping {
                        source: src,
                        target: t,
                    });
                    self.learn.target = None;
                    self.dirty = true;
                    let extra = if lost.is_empty() {
                        String::new()
                    } else {
                        format!("  (atenção: {})", lost.join("; "))
                    };
                    let from = if from_piano { "Teclado: " } else { "" };
                    self.notify(format!(
                        "Aprendido: {from}{} → {}{extra}",
                        src.label(),
                        t.label()
                    ));
                    self.sync_piano_hooks();
                    return true;
                }
                return false;
            }
        }

        let list = if from_piano {
            &self.cfg.piano_mappings
        } else {
            &self.cfg.mappings
        };
        let hits: Vec<Mapping> = list
            .iter()
            .filter(|m| ev.matches(m.source))
            .copied()
            .collect();
        let mut changed = false;
        for m in hits {
            let is_button_source = !matches!(m.source, Source::PitchBend { .. })
                && (matches!(m.source, Source::Note { .. }) || !m.target.is_continuous());
            if let (Target::Drum(i), ControlEvent::Note { vel, .. }) = (m.target, ev) {
                // DrumPad numa tecla/botão: a força da batida acompanha a força do dedo.
                if vel > 0 {
                    self.drum_hit(i, vel);
                    changed = true;
                }
            } else if m.target.is_momentary() && is_button_source {
                // Segurar: apertou = liga; soltou = volta (pitch ao meio, vibrato a zero).
                self.hold(m.target, down);
                changed = true;
            } else if m.target.is_continuous() {
                if let Some(v) = ev.value(m.source) {
                    self.apply_continuous(m.target, v);
                    changed = true;
                }
            } else if pressed {
                self.apply_button(m.target);
                changed = true;
            }
        }
        changed
    }

    /// Mesma pergunta de `piano_routed`, mas olhando o próprio estado (sem as marcas globais).
    fn piano_is_control(&self, msg: &[u8]) -> bool {
        let Some(ev) = ControlEvent::parse(msg) else {
            return false;
        };
        if self.learn.active && self.learn.target.is_some() && self.wizard.is_none() {
            return true;
        }
        self.cfg.piano_mappings.iter().any(|m| ev.matches(m.source))
    }

    /// Tira uma ligação do teclado (Ajustes → Controles do teclado).
    pub fn remove_piano_mapping(&mut self, idx: usize) {
        if idx < self.cfg.piano_mappings.len() {
            let m = self.cfg.piano_mappings.remove(idx);
            if m.target.is_momentary() {
                self.hold(m.target, false);
            }
            self.dirty = true;
            self.sync_piano_hooks();
        }
    }

    // ---------- Configurar mesinha (assistente) ----------

    pub fn wizard_start(&mut self) {
        self.learn.active = false;
        self.learn.target = None;
        self.wizard = Some(Wizard::new());
    }

    pub fn wizard_skip(&mut self) {
        if let Some(w) = &mut self.wizard {
            w.step += 1;
            w.candidate = None;
        }
        self.wizard_check_done();
    }

    pub fn wizard_cancel(&mut self) {
        self.wizard = None;
        self.notify("Configuração da mesinha cancelada (o que já foi feito ficou salvo)");
    }

    fn wizard_check_done(&mut self) {
        if self.wizard.as_ref().is_some_and(|w| w.current().is_none()) {
            self.wizard = None;
            self.notify("Mesinha configurada! Faders e knobs prontos.");
        }
    }

    fn wizard_signal(&mut self, ev: ControlEvent) -> bool {
        let Some(w) = self.wizard.as_mut() else {
            return false;
        };
        let Some((target, _)) = w.current() else {
            return false;
        };
        // Só faders e knobs (pitch bend ou CC); botões são ignorados aqui.
        let src = match ev {
            ControlEvent::PitchBend { ch, .. } => Source::PitchBend { ch },
            ControlEvent::Cc { ch, num, .. } => Source::Cc {
                ch,
                num,
                relative: false,
            },
            ControlEvent::Note { .. } => return false,
        };
        if w.used.iter().any(|u| same_source(*u, src)) {
            return false; // ainda é o controle do passo anterior
        }
        // Espera 3 sinais do mesmo controle (evita pegar um esbarrão em outro).
        let small = |v: u8| (1..=15).contains(&v) || (65..=79).contains(&v);
        let (count, all_small) = match (&mut w.candidate, ev) {
            (Some((prev, n, sm)), _) if prev.matches(src) => {
                *n += 1;
                if let ControlEvent::Cc { val, .. } = ev {
                    *sm &= small(val);
                }
                (*n, *sm)
            }
            _ => {
                let sm = matches!(ev, ControlEvent::Cc { val, .. } if small(val));
                w.candidate = Some((ev, 1, sm));
                (1, sm)
            }
        };
        if count < 3 {
            return true;
        }
        // Knob "infinito" manda só valores pequenos (1..15 sobe, 65..79 desce).
        let src = match src {
            Source::Cc { ch, num, .. } => Source::Cc {
                ch,
                num,
                relative: all_small,
            },
            other => other,
        };
        w.used.push(src);
        w.step += 1;
        w.candidate = None;
        self.cfg
            .mappings
            .retain(|m| !same_source(m.source, src) && m.target != target);
        self.cfg.mappings.push(Mapping {
            source: src,
            target,
        });
        self.dirty = true;
        self.notify(format!("✔ {} → {}", src.label(), target.label()));
        self.wizard_check_done();
        true
    }

    /// Luzes dos botões da mesinha: (canal, nota, acesa).
    pub fn leds(&self) -> Vec<(u8, u8, bool)> {
        let mut out = Vec::new();
        for m in &self.cfg.mappings {
            if let Source::Note { ch, num } = m.source {
                let lit = match m.target {
                    Target::Toggle(i) => self.cfg.mix.layers[i].on,
                    Target::Solo(i) => self.cfg.mix.layers[i].solo,
                    Target::Scene(n) => self.cfg.current_scene == Some(n),
                    Target::Pad(k) => self.pad_active == Some(k),
                    Target::Freeze => self.freeze_on,
                    Target::PulseToggle => self.pulse_on,
                    Target::ArpToggle => self.arp_on,
                    Target::VoiceFxToggle => self.voicefx_on,
                    Target::SeqToggle => self.seq_on,
                    _ => continue,
                };
                out.push((ch, num, lit));
            }
        }
        out
    }

    // ---------- SF2 ----------

    /// Chamado quando o SF2 terminou de carregar (ou quando a taxa de amostragem muda).
    pub fn install_sound_font(&mut self, sf: Arc<SoundFont>, name: String) {
        let mut presets: Vec<PresetRef> = sf
            .get_presets()
            .iter()
            .map(|p| PresetRef {
                bank: p.get_bank_number(),
                patch: p.get_patch_number(),
                name: p.get_name().trim().to_string(),
                file: None,
            })
            .collect();
        presets.sort_by_key(|p| (p.bank, p.patch));
        presets.dedup_by_key(|p| (p.bank, p.patch));
        self.main_presets = presets;
        self.merge_presets();
        self.sound_font = Some(sf);
        self.sf2_name = Some(name);
        self.load_error = None;

        // Primeira vez: liga a camada 1 com o primeiro timbre para já sair som.
        if self.cfg.mix.layers.iter().all(|l| l.preset.is_none()) {
            if let Some(p) = self.presets.first().cloned() {
                self.cfg.mix.layers[0].preset = Some(p);
                self.cfg.mix.layers[0].on = true;
            }
        }
        // Atualiza os nomes guardados (caso o SF2 tenha mudado).
        let fix = |l: &mut Layer, presets: &[PresetRef]| {
            if let Some(p) = &mut l.preset {
                if let Some(q) = presets.iter().find(|q| q.same(p)) {
                    p.name = q.name.clone();
                }
            }
        };
        for l in &mut self.cfg.mix.layers {
            fix(l, &self.main_presets);
        }
        self.rebuild_synth();
    }

    /// Cria um sintetizador novo com o SF2 atual e manda para o motor.
    /// v0.35: os das camadas da Biblioteca também são refeitos (taxa de amostragem nova, polifonia...).
    pub fn rebuild_synth(&mut self) {
        if let Some(sf) = &self.sound_font {
            if let Some(synth) = Engine::build_synth(sf, self.sample_rate, self.cfg.polyphony) {
                self.send(Cmd::Replace(synth));
                self.sync_engine();
            }
        }
        if self.layer_font.iter().any(|f| f.is_some()) {
            self.layer_font = Default::default();
        }
        self.ensure_layer_fonts();
    }
}

/// v0.35: abre um arquivo da Biblioteca ("Pads/Hillsong Pad.sf2").
pub fn load_font_file(rel: &str) -> Result<Arc<SoundFont>, String> {
    let path = crate::biblioteca::full_path(rel);
    let f = std::fs::File::open(&path).map_err(|_| "o arquivo não está mais na pasta da Biblioteca".to_string())?;
    let mut r = std::io::BufReader::with_capacity(1 << 20, f);
    SoundFont::new(&mut r)
        .map(Arc::new)
        .map_err(|e| format!("não é um SF2 válido: {e}"))
}

/// v0.35: thread que abre os arquivos da Biblioteca fora da tela e do áudio (um por vez).
pub fn start_font_loader(core: SharedCore) {
    let (tx, rx) = crossbeam_channel::unbounded::<String>();
    core.lock().unwrap().font_loader = Some(tx);
    std::thread::Builder::new()
        .name("abrir-biblioteca".into())
        .spawn(move || {
            while let Ok(rel) = rx.recv() {
                let r = load_font_file(&rel);
                core.lock().unwrap().font_loaded(rel, r);
            }
        })
        .expect("thread da Biblioteca");
}

/// Abre um SF2 em segundo plano (pode levar vários segundos num arquivo de 1,4 GB).
pub fn load_sf2_async(core: SharedCore, path: String) {
    let total = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
    let read = Arc::new(AtomicU64::new(0));
    let file = std::path::Path::new(&path)
        .file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_else(|| path.clone());
    {
        let mut c = core.lock().unwrap();
        if c.loading.is_some() {
            return;
        }
        c.loading = Some(Loading {
            file: file.clone(),
            read: read.clone(),
            total,
        });
        c.load_error = None;
    }
    std::thread::Builder::new()
        .name("carregar-sf2".into())
        .spawn(move || {
            let result = std::fs::File::open(&path)
                .map_err(|e| format!("Não consegui abrir o arquivo: {e}"))
                .and_then(|f| {
                    let mut r = ProgressReader {
                        inner: std::io::BufReader::with_capacity(1 << 20, f),
                        read: read.clone(),
                    };
                    SoundFont::new(&mut r)
                        .map_err(|e| format!("O arquivo não é um SF2 válido: {e}"))
                });
            let mut c = core.lock().unwrap();
            c.loading = None;
            match result {
                Ok(sf) => {
                    c.cfg.sf2_path = Some(path);
                    c.dirty = true;
                    c.install_sound_font(Arc::new(sf), file.clone());
                    c.notify(format!("{file} pronto"));
                }
                Err(e) => c.load_error = Some(e),
            }
            c.request_repaint();
        })
        .expect("thread do SF2");
}

/// Pedido de textura do Vivo: banco = 3 + n, "tom" = nota do arquivo, caminho do arquivo próprio.
fn load_job_texture(job: &PadJob) -> Result<Arc<GrainSource>, String> {
    let (bank, root, custom) = job;
    pads::load_texture(
        bank - PAD_BANKS,
        custom.as_deref().map(|p| (p, *root as u8)),
    )
}

/// Thread que decodifica pads fora da tela e do áudio.
pub fn start_pad_loader(core: SharedCore) {
    let (tx, rx) = crossbeam_channel::unbounded::<PadJob>();
    let bank = {
        let mut c = core.lock().unwrap();
        c.pad_loader = Some(tx);
        c.cfg.pad_bank
    };
    let worker = core.clone();
    std::thread::Builder::new()
        .name("pads".into())
        .spawn(move || {
            while let Ok(job) = rx.recv() {
                let (bank, key, custom) = &job;
                if let Some(i) = pads::my_slot(*bank) {
                    let path = custom.clone().unwrap_or_default();
                    let r = pads::load_my_pad(&path);
                    if let Ok(mut c) = worker.lock() {
                        c.my_pad_loaded(i, &path, r);
                    }
                    continue;
                }
                if *bank >= PAD_BANKS {
                    let r = load_job_texture(&job);
                    if let Ok(mut c) = worker.lock() {
                        c.texture_loaded(bank - PAD_BANKS, r);
                    }
                    continue;
                }
                let r = pads::load(*bank, *key, custom.as_deref());
                if let Ok(mut c) = worker.lock() {
                    c.pad_loaded(*bank, *key, r);
                }
            }
        })
        .expect("thread dos pads");
    // Os dois bancos ficam prontos (o atual primeiro): trocar Escuro/Coral no culto é na hora.
    let mut c = core.lock().unwrap();
    c.preload_pad_bank(bank);
    for b in 0..ALL_BANKS {
        c.preload_pad_bank(b);
    }
}

struct ProgressReader<R> {
    inner: R,
    read: Arc<AtomicU64>,
}

impl<R: Read> Read for ProgressReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let n = self.inner.read(buf)?;
        self.read.fetch_add(n as u64, Ordering::Relaxed);
        Ok(n)
    }
}

// ---------- Mensagens da mesinha ----------

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ControlEvent {
    Note { ch: u8, num: u8, vel: u8 },
    Cc { ch: u8, num: u8, val: u8 },
    PitchBend { ch: u8, value: u16 },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ContValue {
    Absolute(f32),
    Relative(f32),
}

/// Passo de cada "clique" de um knob infinito.
const KNOB_STEP: f32 = 0.02;

impl ControlEvent {
    pub fn label(&self) -> String {
        match *self {
            ControlEvent::Note { ch, num, vel } => format!(
                "botão (nota {num}) canal {} {}",
                ch + 1,
                if vel > 0 { "apertado" } else { "solto" }
            ),
            ControlEvent::Cc { ch, num, val } => format!("CC {num} = {val} canal {}", ch + 1),
            ControlEvent::PitchBend { ch, value } => {
                format!("fader canal {} = {:.0}%", ch + 1, value as f32 / 163.83)
            }
        }
    }

    pub fn parse(msg: &[u8]) -> Option<ControlEvent> {
        let status = *msg.first()?;
        let ch = status & 0x0F;
        let d1 = msg.get(1).copied().unwrap_or(0) & 0x7F;
        let d2 = msg.get(2).copied().unwrap_or(0) & 0x7F;
        match status & 0xF0 {
            0x90 => Some(ControlEvent::Note {
                ch,
                num: d1,
                vel: d2,
            }),
            0x80 => Some(ControlEvent::Note {
                ch,
                num: d1,
                vel: 0,
            }),
            0xB0 => Some(ControlEvent::Cc {
                ch,
                num: d1,
                val: d2,
            }),
            0xE0 => Some(ControlEvent::PitchBend {
                ch,
                value: (d1 as u16) | ((d2 as u16) << 7),
            }),
            _ => None,
        }
    }

    pub fn matches(&self, src: Source) -> bool {
        match (*self, src) {
            (ControlEvent::Note { ch, num, .. }, Source::Note { ch: c, num: n }) => {
                ch == c && num == n
            }
            (ControlEvent::Cc { ch, num, .. }, Source::Cc { ch: c, num: n, .. }) => {
                ch == c && num == n
            }
            (ControlEvent::PitchBend { ch, .. }, Source::PitchBend { ch: c }) => ch == c,
            _ => false,
        }
    }

    pub fn value(&self, src: Source) -> Option<ContValue> {
        match (*self, src) {
            (ControlEvent::PitchBend { value, .. }, _) => {
                Some(ContValue::Absolute(value as f32 / 16383.0))
            }
            (ControlEvent::Cc { val, .. }, Source::Cc { relative: true, .. }) => {
                let d = if val < 64 {
                    val as f32
                } else {
                    -((val - 64) as f32)
                };
                Some(ContValue::Relative(d * KNOB_STEP))
            }
            (ControlEvent::Cc { val, .. }, _) => Some(ContValue::Absolute(val as f32 / 127.0)),
            _ => None,
        }
    }

    /// No modo "Aprender": que fonte usar para este alvo (None = ignorar esta mensagem).
    pub fn source_for(&self, t: Target, pressed: bool) -> Option<Source> {
        match *self {
            ControlEvent::PitchBend { ch, .. } if t.is_continuous() => {
                Some(Source::PitchBend { ch })
            }
            ControlEvent::Cc { ch, num, .. } if t.is_continuous() || pressed => Some(Source::Cc {
                ch,
                num,
                relative: false,
            }),
            // Pitch/modulation também aceitam botão (segurar = liga).
            ControlEvent::Note { ch, num, vel }
                if (!t.is_continuous() || t.is_momentary()) && vel > 0 =>
            {
                Some(Source::Note { ch, num })
            }
            _ => None,
        }
    }
}

fn same_source(a: Source, b: Source) -> bool {
    match (a, b) {
        (Source::Cc { ch, num, .. }, Source::Cc { ch: c, num: n, .. }) => ch == c && num == n,
        (x, y) => x == y,
    }
}

#[cfg(test)]
fn tests_old_defaults() -> Vec<Mapping> {
    crate::model::old_default_mappings_for_tests()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn core() -> (Core, crossbeam_channel::Receiver<Cmd>) {
        let (tx, rx) = crossbeam_channel::unbounded();
        let mut c = Core::new(Config::default(), tx);
        c.presets = vec![
            PresetRef {
                bank: 0,
                patch: 0,
                name: "Piano".into(),
                file: None,
            },
            PresetRef {
                bank: 0,
                patch: 48,
                name: "Cordas".into(),
                file: None,
            },
        ];
        (c, rx)
    }

    #[test]
    fn smc_fader_moves_volume() {
        let (mut c, _rx) = core();
        // Fader 3 (pitch bend canal 3) no máximo.
        assert!(c.handle_controller(&[0xE2, 0x7F, 0x7F]));
        assert!((c.cfg.mix.layers[2].volume - 1.0).abs() < 1e-3);
        // Fader 9 = volume geral, no meio.
        c.handle_controller(&[0xE8, 0x00, 0x40]);
        assert!((c.cfg.master - 0.5).abs() < 0.01);
    }

    #[test]
    fn smc_mute_button_toggles_on_press_only() {
        let (mut c, _rx) = core();
        assert!(!c.cfg.mix.layers[1].on);
        c.handle_controller(&[0x90, 17, 127]); // aperta MUTE 2
        assert!(c.cfg.mix.layers[1].on);
        assert!(
            c.cfg.mix.layers[1].preset.is_some(),
            "liga com o primeiro timbre"
        );
        c.handle_controller(&[0x90, 17, 0]); // solta
        assert!(c.cfg.mix.layers[1].on);
        c.handle_controller(&[0x90, 17, 127]);
        assert!(!c.cfg.mix.layers[1].on);
    }

    #[test]
    fn relative_knob_changes_reverb() {
        let (mut c, _rx) = core();
        let before = c.cfg.mix.layers[0].reverb;
        c.handle_controller(&[0xB0, 16, 0x03]); // gira 3 cliques para a direita
        assert!(c.cfg.mix.layers[0].reverb > before);
        c.handle_controller(&[0xB0, 16, 0x45]); // 5 para a esquerda
        assert!(c.cfg.mix.layers[0].reverb < before);
    }

    #[test]
    fn solo_and_audible() {
        let (mut c, _rx) = core();
        c.toggle(0);
        c.toggle(1);
        assert!(c.cfg.mix.audible(0) && c.cfg.mix.audible(1));
        c.toggle_solo(1);
        assert!(!c.cfg.mix.audible(0) && c.cfg.mix.audible(1));
    }

    #[test]
    fn learn_replaces_mapping() {
        let (mut c, _rx) = core();
        c.learn.active = true;
        c.learn.target = Some(Target::Volume(0));
        c.handle_controller(&[0xB0, 7, 100]); // CC 7 canal 1
        assert!(c.learn.target.is_none());
        c.learn.active = false;
        c.handle_controller(&[0xB0, 7, 127]);
        assert!((c.cfg.mix.layers[0].volume - 1.0).abs() < 1e-3);
        // O fader antigo (pitch bend canal 1) não manda mais na camada 1.
        c.handle_controller(&[0xE0, 0, 0]);
        assert!((c.cfg.mix.layers[0].volume - 1.0).abs() < 1e-3);
    }

    /// Tudo que o motor recebeu até agora.
    fn drain(rx: &crossbeam_channel::Receiver<Cmd>) -> Vec<Cmd> {
        rx.try_iter().collect()
    }

    fn bends(cmds: &[Cmd]) -> Vec<u16> {
        cmds.iter()
            .filter_map(|c| match c {
                Cmd::PitchBend { lsb, msb } => Some((*lsb as u16) | ((*msb as u16) << 7)),
                _ => None,
            })
            .collect()
    }

    fn plays_note(cmds: &[Cmd]) -> bool {
        cmds.iter().any(|c| matches!(c, Cmd::NoteOn { .. }))
    }

    #[test]
    fn piano_key_learned_as_pitch_up_holds_and_releases() {
        // AMW Mini 32: o Theo escolhe uma tecla (nota 36) para segurar o pitch lá em cima.
        let (mut c, rx) = core();
        c.learn.active = true;
        c.learn.target = Some(Target::PitchUp);
        c.handle_piano(&[0x90, 36, 100]);
        assert!(c.learn.target.is_none(), "aprendeu");
        assert_eq!(c.cfg.piano_mappings.len(), 1);
        assert!(c.cfg.mappings.iter().all(|m| m.target != Target::PitchUp), "não foi para a lista da mesinha");
        assert!(!plays_note(&drain(&rx)), "a tecla do Aprender não toca");
        c.learn.active = false;
        c.handle_piano(&[0x80, 36, 0]);
        drain(&rx);

        c.handle_piano(&[0x90, 36, 90]); // segura
        let cmds = drain(&rx);
        assert!(!plays_note(&cmds), "tecla ligada a controle não toca nota");
        assert_eq!(bends(&cmds), vec![16383]);
        assert!((c.pitch - 1.0).abs() < 1e-6);
        c.handle_piano(&[0x80, 36, 0]); // solta: volta ao meio
        assert_eq!(bends(&drain(&rx)), vec![8192]);
        assert_eq!(c.pitch, 0.0);

        // outra tecla continua tocando normal
        c.handle_piano(&[0x90, 60, 100]);
        assert!(plays_note(&drain(&rx)));
    }

    #[test]
    fn piano_key_learned_as_modulation_and_pitch_down() {
        let (mut c, rx) = core();
        c.learn.active = true;
        c.learn.target = Some(Target::Modulation);
        c.handle_piano(&[0x90, 37, 100]);
        c.learn.target = Some(Target::PitchDown);
        c.handle_piano(&[0x90, 38, 100]);
        c.learn.active = false;
        assert_eq!(c.cfg.piano_mappings.len(), 2);
        drain(&rx);

        c.handle_piano(&[0x90, 37, 100]);
        let cmds = drain(&rx);
        assert!(cmds.iter().any(|m| matches!(m, Cmd::Cc { num: 1, val: 127 })), "vibrato cheio");
        c.handle_piano(&[0x80, 37, 0]);
        assert!(drain(&rx).iter().any(|m| matches!(m, Cmd::Cc { num: 1, val: 0 })));

        c.handle_piano(&[0x90, 38, 100]);
        assert_eq!(bends(&drain(&rx)), vec![0]);
        assert!((c.pitch + 1.0).abs() < 1e-6);
        c.handle_piano(&[0x90, 38, 0]); // note on com velocity 0 = soltou
        assert_eq!(bends(&drain(&rx)), vec![8192]);
    }

    #[test]
    fn piano_wheel_or_slider_learned_as_pitch_mod_and_volume() {
        let (mut c, rx) = core();
        c.learn.active = true;
        c.learn.target = Some(Target::Pitch);
        c.handle_piano(&[0xE0, 0, 64]); // roda de pitch
        c.learn.target = Some(Target::Modulation);
        c.handle_piano(&[0xB0, 1, 10]); // roda de mod (CC 1)
        c.learn.target = Some(Target::Master);
        c.handle_piano(&[0xB0, 7, 50]); // slider de volume (CC 7)
        c.learn.active = false;
        assert_eq!(c.cfg.piano_mappings.len(), 3);
        drain(&rx);

        c.handle_piano(&[0xE0, 0x7F, 0x7F]);
        assert!((c.pitch - 1.0).abs() < 0.01);
        c.handle_piano(&[0xE0, 0x00, 0x40]); // meio
        assert_eq!(c.pitch, 0.0);
        c.handle_piano(&[0xB0, 1, 127]);
        assert!((c.modulation - 1.0).abs() < 1e-6);
        c.handle_piano(&[0xB0, 7, 127]);
        assert!((c.cfg.master - 1.0).abs() < 1e-3);
        // a mesinha não é afetada pelo que o teclado aprendeu (CC 7 da mesinha não vira volume geral)
        let before = c.cfg.master;
        c.set_master(0.3);
        c.handle_controller(&[0xB0, 7, 127]);
        assert!((c.cfg.master - 0.3).abs() < 1e-3, "lista da mesinha separada (antes {before})");
    }

    #[test]
    fn piano_mapping_removed_releases_and_plays_again() {
        let (mut c, rx) = core();
        c.learn.active = true;
        c.learn.target = Some(Target::PitchUp);
        c.handle_piano(&[0x90, 40, 100]);
        c.learn.active = false;
        c.handle_piano(&[0x90, 40, 100]);
        assert!(c.pitch > 0.99);
        c.remove_piano_mapping(0);
        assert_eq!(c.pitch, 0.0, "tirou a ligação: pitch volta ao meio");
        drain(&rx);
        c.handle_piano(&[0x90, 40, 100]);
        assert!(plays_note(&drain(&rx)), "a tecla voltou a tocar");
    }

    #[test]
    fn pitch_range_goes_to_engine_and_panic_resets() {
        let (mut c, rx) = core();
        c.set_pitch_range(7);
        assert!(drain(&rx).iter().any(|m| matches!(m, Cmd::PitchRange(7))));
        c.set_pitch_range(40);
        assert_eq!(c.cfg.pitch_range, 12);
        c.set_pitch(0.5);
        c.set_modulation(0.8);
        c.panic();
        assert_eq!((c.pitch, c.modulation), (0.0, 0.0));
        // config antiga (sem os campos novos) abre com o padrão
        let old: Config = serde_json::from_str(r#"{"master":0.5}"#).unwrap();
        assert_eq!(old.pitch_range, 2);
        assert!(old.piano_mappings.is_empty());
    }

    /// Trecho de "acorde" para os testes do Congelar.
    fn test_chord() -> crate::freeze::FreezeSource {
        let rate = 48_000u32;
        let n = rate as usize * 2;
        let l: Vec<f32> = (0..n)
            .map(|i| {
                let t = i as f32 / rate as f32;
                [196.0f32, 246.94, 293.66]
                    .iter()
                    .map(|f| (std::f32::consts::TAU * f * t).sin())
                    .sum::<f32>()
                    * 0.2
                    * (-t * 0.5).exp()
            })
            .collect();
        crate::freeze::FreezeSource::from_audio(&l, &l, rate).unwrap()
    }

    #[test]
    fn freeze_button_toggles_and_panic_stops() {
        let (mut c, rx) = core();
        c.freeze_play(test_chord());
        assert!(c.freeze_on);
        assert!(drain(&rx).iter().any(|m| matches!(m, Cmd::FreezePlay(_))));
        c.freeze_toggle(); // desliga
        assert!(!c.freeze_on);
        assert!(drain(&rx).iter().any(|m| matches!(m, Cmd::FreezeStop(s) if (*s - 3.0).abs() < 0.01)));
        c.freeze_play(test_chord());
        c.panic();
        assert!(!c.freeze_on, "pânico solta o congelado");
        // trocar de cena não mexe no congelado
        c.freeze_play(test_chord());
        c.save_scene(1);
        c.load_scene(1);
        c.set_preset(0, c.presets[1].clone());
        assert!(c.freeze_on);
        assert!(!drain(&rx).iter().any(|m| matches!(m, Cmd::FreezeStop(_))));
    }

    #[test]
    fn freeze_can_be_learned_on_the_amw_and_on_the_mixer() {
        let (mut c, rx) = core();
        c.learn.active = true;
        c.learn.target = Some(Target::Freeze);
        c.handle_piano(&[0x90, 41, 100]); // tecla do AMW
        c.learn.target = Some(Target::Freeze);
        c.handle_controller(&[0x90, 0, 127]); // botão REC 1 da mesinha (livre)
        c.learn.active = false;
        assert_eq!(c.cfg.piano_mappings.last().unwrap().target, Target::Freeze);
        assert!(c.cfg.mappings.iter().any(|m| m.target == Target::Freeze));
        drain(&rx);
        // sem nada tocando: avisa e não liga
        crate::freeze::ring().clear();
        c.handle_piano(&[0x90, 41, 100]);
        assert!(!c.freeze_on);
        assert!(!plays_note(&drain(&rx)), "a tecla do Congelar não toca");
        assert!(c.toast.as_ref().unwrap().0.contains("toque o acorde"));
        // ligado: a luz do botão da mesinha acende
        c.freeze_play(test_chord());
        assert!(c.leds().contains(&(0, 0, true)));
        c.handle_controller(&[0x90, 0, 127]);
        assert!(!c.freeze_on);
    }

    #[test]
    fn engine_keeps_the_frozen_sound_and_fades_it_out() {
        use crate::engine::{Engine, Stats};
        let (tx, rx) = crossbeam_channel::unbounded();
        let (gtx, _grx) = crossbeam_channel::unbounded();
        let mut e = Engine::new(rx, gtx, Arc::new(Stats::default()));
        let v = crate::freeze::FreezeVoice::new(Arc::new(test_chord()), 48_000, 3);
        tx.send(Cmd::FreezePlay(Box::new(v))).unwrap();
        let mut out = vec![0.0f32; 1024];
        let level = |o: &[f32]| (o.iter().map(|x| x * x).sum::<f32>() / o.len() as f32).sqrt();
        // 30 s sem SF2 nenhum: o congelado continua soando
        for _ in 0..(30 * 48_000 / 512) {
            e.render_interleaved(&mut out, 2);
        }
        assert!(level(&out) > 0.01, "ainda soando: {}", level(&out));
        tx.send(Cmd::FreezeStop(3.0)).unwrap();
        for _ in 0..(4 * 48_000 / 512) {
            e.render_interleaved(&mut out, 2);
        }
        assert!(level(&out) < 1e-6, "sumiu: {}", level(&out));
    }

    #[test]
    fn scenes_save_and_load() {
        let (mut c, _rx) = core();
        c.toggle(0);
        c.set_volume(0, 0.3);
        c.save_scene(2);
        c.set_volume(0, 0.9);
        c.step_scene(1); // cena 4 (vazia) não muda a mesa
        assert!((c.cfg.mix.layers[0].volume - 0.9).abs() < 1e-6);
        c.load_scene(2);
        assert!((c.cfg.mix.layers[0].volume - 0.3).abs() < 1e-6);
    }

    #[test]
    fn port_roles_guess() {
        assert_eq!(PortRole::guess("SMC-Mixer"), PortRole::Controller);
        assert_eq!(PortRole::guess("CASIO USB-MIDI"), PortRole::Piano);
    }

    #[test]
    fn config_roundtrip() {
        let mut cfg = Config::default();
        cfg.mix.layers[3].volume = 0.42;
        let s = serde_json::to_string(&cfg).unwrap();
        let back: Config = serde_json::from_str(&s).unwrap();
        assert_eq!(back, cfg);
        // Config velha/incompleta não quebra.
        let partial: Config = serde_json::from_str(r#"{"master":0.5}"#).unwrap();
        assert_eq!(partial.mappings.len(), smc_default_mappings().len());
    }

    #[test]
    fn faders_7_and_8_are_pad_and_master() {
        let (mut c, _rx) = core();
        c.handle_controller(&[0xE6, 0, 0]); // fader 7 no zero
        assert!(c.cfg.pad_volume < 0.01);
        c.handle_controller(&[0xE7, 0x7F, 0x7F]); // fader 8 no máximo
        assert!((c.cfg.master - 1.0).abs() < 1e-3);
        c.handle_controller(&[0xB0, 22, 0x01]); // knob 7 gira = próximo tom do pad
        assert_eq!(c.pad_active, Some(0));
        c.handle_controller(&[0xB0, 22, 0x01]);
        assert_eq!(c.pad_active, Some(1));
        c.handle_controller(&[0xB0, 22, 0x41]);
        assert_eq!(c.pad_active, Some(0));
    }

    #[test]
    fn wizard_learns_faders_and_knobs_in_order() {
        let (mut c, _rx) = core();
        c.wizard_start();
        // Mesinha em "modo CC": faders = CC 0..7 absolutos, knobs = CC 20..26 relativos.
        for f in 0..8u8 {
            for v in [10u8, 20, 30, 40] {
                c.handle_controller(&[0xB0, f, v]);
            }
        }
        for k in 0..7u8 {
            for v in [1u8, 1, 65] {
                c.handle_controller(&[0xB0, 20 + k, v]);
            }
        }
        assert!(c.wizard.is_none(), "terminou");
        c.handle_controller(&[0xB0, 2, 127]);
        assert!((c.cfg.mix.layers[2].volume - 1.0).abs() < 1e-3);
        c.handle_controller(&[0xB0, 6, 0]);
        assert!(c.cfg.pad_volume < 0.01);
        c.handle_controller(&[0xB0, 7, 127]);
        assert!((c.cfg.master - 1.0).abs() < 1e-3);
        let before = c.cfg.mix.layers[4].reverb;
        c.handle_controller(&[0xB0, 24, 0x02]);
        assert!(
            c.cfg.mix.layers[4].reverb > before,
            "knob detectado como infinito"
        );
        // O fader antigo (pitch bend) não manda mais no volume 1.
        let v0 = c.cfg.mix.layers[0].volume;
        c.handle_controller(&[0xE0, 0, 0]);
        assert_eq!(c.cfg.mix.layers[0].volume, v0);
    }

    #[test]
    fn wizard_skip_and_buttons_ignored() {
        let (mut c, _rx) = core();
        c.wizard_start();
        c.handle_controller(&[0x90, 16, 127]); // botão: ignorado no assistente
        assert_eq!(c.wizard.as_ref().unwrap().step, 0);
        c.wizard_skip();
        assert_eq!(c.wizard.as_ref().unwrap().step, 1);
        c.wizard_cancel();
        assert!(c.wizard.is_none());
    }

    #[test]
    fn old_config_with_8_layers_migrates_and_keeps_learned() {
        // Config da v0.2: 8 camadas, padrão antigo + uma coisa que o Theo aprendeu no REC 1.
        let mut old = serde_json::to_value(Config::default()).unwrap();
        let obj = old.as_object_mut().unwrap();
        obj.remove("mappings_version");
        let mut maps = serde_json::to_value(super::tests_old_defaults()).unwrap();
        maps.as_array_mut()
            .unwrap()
            .retain(|m| m["source"] != serde_json::json!({"Note": {"ch": 0, "num": 0}}));
        maps.as_array_mut().unwrap().push(
            serde_json::json!({"source": {"Note": {"ch": 0, "num": 0}}, "target": {"Pad": 3}}),
        );
        obj.insert("mappings".into(), maps);
        let layers: Vec<serde_json::Value> = (0..8)
            .map(|_| serde_json::to_value(Layer::default()).unwrap())
            .collect();
        obj["mix"]["layers"] = serde_json::Value::Array(layers);
        let mut cfg: Config = serde_json::from_value(old).unwrap();
        cfg.normalize();
        assert_eq!(cfg.mix.layers.len(), LAYERS);
        assert!(cfg
            .mappings
            .iter()
            .all(|m| !matches!(m.target, Target::Volume(i) if i >= LAYERS)));
        assert!(
            cfg.mappings.contains(&Mapping {
                source: Source::Note { ch: 0, num: 0 },
                target: Target::Pad(3)
            }),
            "o aprendido fica"
        );
        assert!(cfg.mappings.contains(&Mapping {
            source: Source::PitchBend { ch: 6 },
            target: Target::PadVolume
        }));
        assert!(cfg.mappings.contains(&Mapping {
            source: Source::PitchBend { ch: 7 },
            target: Target::Master
        }));
        assert_eq!(cfg.mappings_version, MAPPINGS_VERSION);
    }

    #[test]
    fn two_buttons_same_function_both_work_and_warning_shown() {
        let (mut c, _rx) = core();
        c.learn.active = true;
        // REC 2 (nota 1) e SELECT 2 (nota 25) ligam a camada 2, junto com o MUTE 2 de fábrica.
        c.learn.target = Some(Target::Toggle(1));
        c.handle_controller(&[0x90, 1, 127]);
        c.learn.target = Some(Target::Toggle(1));
        c.handle_controller(&[0x90, 25, 127]);
        c.learn.active = false;
        for note in [17u8, 1, 25] {
            let before = c.cfg.mix.layers[1].on;
            c.handle_controller(&[0x90, note, 127]);
            assert_ne!(c.cfg.mix.layers[1].on, before, "botão {note} funciona");
        }
        // Reaproveitar um botão avisa o que ele fazia antes.
        c.learn.active = true;
        c.learn.target = Some(Target::Panic);
        c.handle_controller(&[0x90, 1, 127]);
        let msg = c.toast.as_ref().unwrap().0.clone();
        assert!(msg.contains("deixou de fazer"), "{msg}");
    }

    #[test]
    fn edit_mode_scenes_and_hidden_layers() {
        let (mut c, _rx) = core();
        assert_eq!(c.cfg.scenes.len(), SCENES);
        c.add_scene();
        assert_eq!(c.cfg.scenes.len(), SCENES + 1);
        // Botão da mesinha que chamava a cena 4 passa a chamar a 3 quando a 2 é apagada.
        c.cfg.mappings.push(Mapping {
            source: Source::Note { ch: 1, num: 1 },
            target: Target::Scene(4),
        });
        c.cfg.mappings.push(Mapping {
            source: Source::Note { ch: 1, num: 2 },
            target: Target::Scene(2),
        });
        c.cfg.current_scene = Some(5);
        c.delete_scene(2);
        assert_eq!(c.cfg.scenes.len(), SCENES);
        assert_eq!(c.cfg.current_scene, Some(4));
        assert!(c
            .cfg
            .mappings
            .iter()
            .any(|m| m.target == Target::Scene(3) && m.source == Source::Note { ch: 1, num: 1 }));
        assert!(!c
            .cfg
            .mappings
            .iter()
            .any(|m| m.source == Source::Note { ch: 1, num: 2 }));
        // Nunca fica sem cena nenhuma.
        for _ in 0..30 {
            c.delete_scene(0);
        }
        assert_eq!(c.cfg.scenes.len(), 1);
        // Tirar uma camada também desliga ela.
        c.toggle(2);
        assert!(c.cfg.mix.layers[2].on);
        c.set_hidden(&layer_id(2), true);
        assert!(!c.cfg.mix.layers[2].on && c.cfg.is_hidden("layer2"));
        c.set_hidden(&layer_id(2), false);
        assert!(!c.cfg.is_hidden("layer2"));
    }
}
