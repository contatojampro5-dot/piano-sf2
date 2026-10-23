//! Dados do programa: camadas, cenas, mapeamento da mesinha e configuração salva.

use serde::{Deserialize, Serialize};

/// Número de camadas (uma por fader da SMC-Mixer).
pub const LAYERS: usize = 6;
/// Número de cenas salvas.
pub const SCENES: usize = 8;

/// Timbre escolhido dentro do SF2 (banco + programa, como no MIDI).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PresetRef {
    pub bank: i32,
    pub patch: i32,
    pub name: String,
}

/// Uma camada (um canal da mesinha).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Layer {
    pub on: bool,
    pub solo: bool,
    /// Volume 0..1 (posição do fader).
    pub volume: f32,
    /// Reverb 0..1 (botão giratório).
    pub reverb: f32,
    pub preset: Option<PresetRef>,
}

impl Default for Layer {
    fn default() -> Self {
        Self {
            on: false,
            solo: false,
            volume: 0.8,
            reverb: 0.25,
            preset: None,
        }
    }
}

/// O estado da "mesa" na tela.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Mix {
    pub layers: Vec<Layer>,
    /// Transposição em semitons (-12..12).
    pub transpose: i32,
}

impl Default for Mix {
    fn default() -> Self {
        Self {
            layers: vec![Layer::default(); LAYERS],
            transpose: 0,
        }
    }
}

impl Mix {
    pub fn normalize(&mut self) {
        self.layers.resize(LAYERS, Layer::default());
        self.transpose = self.transpose.clamp(-12, 12);
        for l in &mut self.layers {
            l.volume = l.volume.clamp(0.0, 1.0);
            l.reverb = l.reverb.clamp(0.0, 1.0);
        }
    }

    pub fn any_solo(&self) -> bool {
        self.layers.iter().any(|l| l.solo)
    }

    /// A camada está soando? (ligada e, se houver solo, ela é solo)
    pub fn audible(&self, i: usize) -> bool {
        let l = &self.layers[i];
        l.on && (!self.any_solo() || l.solo)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Scene {
    pub name: String,
    pub mix: Option<Mix>,
}

/// O que um controle da mesinha comanda.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Target {
    Volume(usize),
    Reverb(usize),
    Toggle(usize),
    Solo(usize),
    Master,
    PrevScene,
    NextScene,
    Scene(usize),
    Panic,
    TransposeDown,
    TransposeUp,
    /// Pad no tom 0..11 (C..B).
    Pad(usize),
    PadStop,
    PadVolume,
    PadPrev,
    PadNext,
    /// Troca entre os estilos de pad (Escuro / Coral / Suave).
    PadBank,
    /// Knob que escolhe o tom do pad (girar = próximo/anterior; knob comum = posição).
    PadKnob,
    /// Alterna entre as abas Sons SF2 / Pads.
    TabToggle,
}

impl Target {
    /// Alvo contínuo (fader/knob) ou botão?
    pub fn is_continuous(self) -> bool {
        matches!(
            self,
            Target::Volume(_)
                | Target::Reverb(_)
                | Target::Master
                | Target::PadVolume
                | Target::PadKnob
        )
    }

    pub fn label(self) -> String {
        match self {
            Target::Volume(i) => format!("Volume da camada {}", i + 1),
            Target::Reverb(i) => format!("Reverb da camada {}", i + 1),
            Target::Toggle(i) => format!("Liga/desliga camada {}", i + 1),
            Target::Solo(i) => format!("Solo da camada {}", i + 1),
            Target::Master => "Volume geral".into(),
            Target::PrevScene => "Cena anterior".into(),
            Target::NextScene => "Próxima cena".into(),
            Target::Scene(i) => format!("Cena {}", i + 1),
            Target::Panic => "Pânico (cortar notas)".into(),
            Target::TransposeDown => "Transpor −1".into(),
            Target::TransposeUp => "Transpor +1".into(),
            Target::Pad(k) => format!("Pad {}", crate::pads::KEY_NAMES[k % crate::pads::PAD_KEYS]),
            Target::PadStop => "Parar pad".into(),
            Target::PadVolume => "Volume do pad".into(),
            Target::PadPrev => "Pad: tom anterior".into(),
            Target::PadNext => "Pad: próximo tom".into(),
            Target::PadBank => "Pad: trocar o estilo".into(),
            Target::PadKnob => "Pad: escolher o tom (knob)".into(),
            Target::TabToggle => "Trocar de aba (Sons/Pads)".into(),
        }
    }
}

/// De onde vem o sinal MIDI (canal 0..15).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Source {
    /// Control Change. `relative` = knob infinito (estilo Mackie: 1..63 sobe, 65..127 desce).
    Cc {
        ch: u8,
        num: u8,
        relative: bool,
    },
    Note {
        ch: u8,
        num: u8,
    },
    PitchBend {
        ch: u8,
    },
}

impl Source {
    pub fn label(self) -> String {
        match self {
            Source::Cc { ch, num, relative } => {
                format!(
                    "CC {} · canal {}{}",
                    num,
                    ch + 1,
                    if relative { " (giro)" } else { "" }
                )
            }
            Source::Note { ch, num } => format!("Botão (nota {}) · canal {}", num, ch + 1),
            Source::PitchBend { ch } => format!("Fader · canal {}", ch + 1),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Mapping {
    pub source: Source,
    pub target: Target,
}

/// Versão do mapeamento padrão (sobe quando o padrão muda; configs antigas são migradas).
pub const MAPPINGS_VERSION: u32 = 2;

/// Mapeamento padrão da M-Vave SMC-Mixer no modo Mackie Control (MCU), igual à tela:
/// canais 1..6 = camadas, canal 7 = PAD, canal 8 (e o master) = volume geral.
/// Faders = pitch bend nos canais 1..9, knobs = CC 16..23 relativos,
/// botões: REC = notas 0..7, Solo = 8..15, Mute = 16..23, Select = 24..31; transporte 91..95.
/// REC e Select ficam livres para o Theo usar com "Aprender".
pub fn smc_default_mappings() -> Vec<Mapping> {
    let mut m = Vec::new();
    let mut add = |source: Source, target: Target| m.push(Mapping { source, target });
    let note = |num: u8| Source::Note { ch: 0, num };
    let knob = |n: u8| Source::Cc {
        ch: 0,
        num: 16 + n,
        relative: true,
    };
    for i in 0..LAYERS {
        let n = i as u8;
        add(Source::PitchBend { ch: n }, Target::Volume(i));
        add(knob(n), Target::Reverb(i));
        add(note(16 + n), Target::Toggle(i));
        add(note(8 + n), Target::Solo(i));
    }
    // Canal 7 = PAD: fader = volume, knob = escolhe o tom, MUTE = parar, SOLO = troca o estilo.
    add(Source::PitchBend { ch: 6 }, Target::PadVolume);
    add(knob(6), Target::PadKnob);
    add(note(22), Target::PadStop);
    add(note(14), Target::PadBank);
    // Canal 8 (e o fader master, se houver) = volume geral; MUTE 8 troca de aba.
    add(Source::PitchBend { ch: 7 }, Target::Master);
    add(Source::PitchBend { ch: 8 }, Target::Master);
    add(note(23), Target::TabToggle);
    // Transporte: ◀◀ ▶▶ = cena, ■ = pânico. Setas: esquerda/direita = cena, cima/baixo = transpor.
    add(note(91), Target::PrevScene);
    add(note(92), Target::NextScene);
    add(note(93), Target::Panic);
    add(note(98), Target::PrevScene);
    add(note(99), Target::NextScene);
    add(note(96), Target::TransposeUp);
    add(note(97), Target::TransposeDown);
    add(note(46), Target::PrevScene);
    add(note(47), Target::NextScene);
    m
}

/// O mapeamento padrão antigo (v0.1/v0.2, 8 camadas). Na migração só sai o que for exatamente
/// igual a ele: o que o Theo aprendeu continua.
fn old_default_mappings() -> Vec<Mapping> {
    let mut v = Vec::new();
    for i in 0..8usize {
        let n = i as u8;
        let note = |num: u8| Source::Note { ch: 0, num };
        v.push(Mapping {
            source: Source::PitchBend { ch: n },
            target: Target::Volume(i),
        });
        v.push(Mapping {
            source: Source::Cc {
                ch: 0,
                num: 16 + n,
                relative: true,
            },
            target: Target::Reverb(i),
        });
        v.push(Mapping {
            source: note(16 + n),
            target: Target::Toggle(i),
        });
        v.push(Mapping {
            source: note(8 + n),
            target: Target::Solo(i),
        });
        v.push(Mapping {
            source: note(n),
            target: Target::Toggle(i),
        });
        v.push(Mapping {
            source: note(24 + n),
            target: Target::Toggle(i),
        });
    }
    v.push(Mapping {
        source: Source::PitchBend { ch: 8 },
        target: Target::Master,
    });
    v
}

#[cfg(test)]
pub fn old_default_mappings_for_tests() -> Vec<Mapping> {
    old_default_mappings()
}

/// Função de cada porta MIDI.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PortRole {
    /// Teclado: as notas tocam o som.
    Piano,
    /// Mesinha: só comanda a tela.
    Controller,
    Ignore,
}

impl PortRole {
    pub fn label(self) -> &'static str {
        match self {
            PortRole::Piano => "Piano (toca)",
            PortRole::Controller => "Mesinha (controla)",
            PortRole::Ignore => "Ignorar",
        }
    }

    /// Palpite pelo nome da porta.
    pub fn guess(name: &str) -> PortRole {
        let n = name.to_lowercase();
        if n.contains("midi through") || n.contains("piano-sf2") {
            PortRole::Ignore
        } else if ["smc", "mixer", "m-vave", "mvave", "cuvave", "control"]
            .iter()
            .any(|k| n.contains(k))
        {
            PortRole::Controller
        } else {
            PortRole::Piano
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub sf2_path: Option<String>,
    pub mix: Mix,
    pub master: f32,
    pub scenes: Vec<Scene>,
    pub current_scene: Option<usize>,
    pub mappings: Vec<Mapping>,
    /// Ajuste manual da função de cada porta (nome → função).
    pub port_roles: Vec<(String, PortRole)>,
    pub audio_device: Option<String>,
    /// Tamanho do buffer em quadros (0 = padrão do Windows).
    pub buffer_frames: u32,
    pub polyphony: usize,
    /// "Program Change" vindo do piano troca a cena (programa 1 = cena 1).
    pub program_change_scenes: bool,
    /// Volume dos pads 0..1.
    pub pad_volume: f32,
    /// Banco de pads em uso (0 = Escuro, 1 = Coral).
    pub pad_bank: usize,
    /// Arquivos próprios nos botões de pad.
    pub pad_files: Vec<PadFile>,
    /// Aba aberta (0 = Sons SF2, 1 = Pads).
    pub tab: usize,
    pub look: Appearance,
    /// Versão do mapeamento salvo (ver `MAPPINGS_VERSION`). Config antiga sem o campo = 0.
    #[serde(default)]
    pub mappings_version: u32,
}

/// Aparência da tela (Ajustes → Aparência).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Appearance {
    /// Fundo: 0 Celestial, 1 Aurora, 2 Pôr do sol, 3 Grafite.
    pub palette: usize,
    /// Vidro: 0 = transparente, 1 = bem fosco.
    pub frost: f32,
    /// Brilho colorido em volta do que está ligado.
    pub glow: f32,
    /// Fundo que se mexe devagar (desligar = mais leve).
    pub motion: bool,
    pub stars: bool,
    /// Tamanho da tela (zoom).
    pub scale: f32,
}

impl Default for Appearance {
    fn default() -> Self {
        Self {
            palette: 0,
            frost: 0.45,
            glow: 0.6,
            motion: true,
            stars: true,
            scale: 1.0,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PadFile {
    pub bank: usize,
    pub key: usize,
    pub path: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            sf2_path: None,
            mix: Mix::default(),
            master: 0.8,
            scenes: default_scenes(),
            current_scene: None,
            mappings: smc_default_mappings(),
            port_roles: Vec::new(),
            audio_device: None,
            buffer_frames: 256,
            polyphony: 128,
            program_change_scenes: false,
            pad_volume: 0.7,
            pad_bank: 0,
            pad_files: Vec::new(),
            tab: 0,
            mappings_version: MAPPINGS_VERSION,
            look: Appearance::default(),
        }
    }
}

fn default_scenes() -> Vec<Scene> {
    (0..SCENES)
        .map(|i| Scene {
            name: format!("Cena {}", i + 1),
            mix: None,
        })
        .collect()
}

impl Config {
    pub fn normalize(&mut self) {
        self.mix.normalize();
        self.master = self.master.clamp(0.0, 1.0);
        self.scenes.resize(SCENES, Scene::default());
        for (i, s) in self.scenes.iter_mut().enumerate() {
            if s.name.trim().is_empty() {
                s.name = format!("Cena {}", i + 1);
            }
            if let Some(m) = &mut s.mix {
                m.normalize();
            }
        }
        if self.current_scene.is_some_and(|c| c >= SCENES) {
            self.current_scene = None;
        }
        self.polyphony = self.polyphony.clamp(16, 256);
        self.pad_volume = self.pad_volume.clamp(0.0, 1.0);
        self.pad_bank %= crate::pads::PAD_BANKS;
        self.tab = self.tab.min(1);
        self.look.frost = self.look.frost.clamp(0.0, 1.0);
        self.look.glow = self.look.glow.clamp(0.0, 1.0);
        self.look.scale = self.look.scale.clamp(0.75, 1.5);
        self.pad_files
            .retain(|f| f.bank < crate::pads::PAD_BANKS && f.key < crate::pads::PAD_KEYS);
        // Nada pode apontar para camada que não existe mais (eram 8, agora são 6).
        self.mappings.retain(|m| match m.target {
            Target::Volume(i) | Target::Reverb(i) | Target::Toggle(i) | Target::Solo(i) => {
                i < LAYERS
            }
            Target::Scene(n) => n < SCENES,
            Target::Pad(k) => k < crate::pads::PAD_KEYS,
            _ => true,
        });
        if self.mappings_version < MAPPINGS_VERSION {
            // Mantém o que o Theo aprendeu; completa com o padrão novo onde o controle está livre.
            let old = old_default_mappings();
            self.mappings.retain(|m| !old.contains(m));
            for d in smc_default_mappings() {
                if !self.mappings.iter().any(|m| m.source == d.source) {
                    self.mappings.push(d);
                }
            }
            self.mappings_version = MAPPINGS_VERSION;
        }
    }

    pub fn pad_file(&self, bank: usize, key: usize) -> Option<&str> {
        self.pad_files
            .iter()
            .find(|f| f.bank == bank && f.key == key)
            .map(|f| f.path.as_str())
    }

    pub fn role_for(&self, port: &str) -> PortRole {
        self.port_roles
            .iter()
            .find(|(n, _)| n == port)
            .map(|(_, r)| *r)
            .unwrap_or_else(|| PortRole::guess(port))
    }

    pub fn set_role(&mut self, port: &str, role: PortRole) {
        self.port_roles.retain(|(n, _)| n != port);
        self.port_roles.push((port.to_string(), role));
    }

    pub fn load_from(path: &std::path::Path) -> Config {
        let mut c: Config = std::fs::read_to_string(path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();
        c.normalize();
        c
    }

    pub fn save_to(&self, path: &std::path::Path) -> std::io::Result<()> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let tmp = path.with_extension("tmp");
        std::fs::write(&tmp, serde_json::to_string_pretty(self).unwrap_or_default())?;
        std::fs::rename(tmp, path)
    }
}

/// Onde a configuração fica: %APPDATA%\PianoSF2\config.json (Windows) ou ~/.config/piano-sf2.
pub fn config_path() -> std::path::PathBuf {
    if let Ok(appdata) = std::env::var("APPDATA") {
        return std::path::PathBuf::from(appdata)
            .join("PianoSF2")
            .join("config.json");
    }
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    std::path::PathBuf::from(home)
        .join(".config")
        .join("piano-sf2")
        .join("config.json")
}
