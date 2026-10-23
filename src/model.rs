//! Dados do programa: camadas, cenas, mapeamento da mesinha e configuração salva.

use serde::{Deserialize, Serialize};

/// Número de camadas (uma por fader da SMC-Mixer).
pub const LAYERS: usize = 8;
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
}

impl Target {
    /// Alvo contínuo (fader/knob) ou botão?
    pub fn is_continuous(self) -> bool {
        matches!(self, Target::Volume(_) | Target::Reverb(_) | Target::Master)
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

/// Mapeamento padrão da M-Vave SMC-Mixer no modo Mackie Control (MCU):
/// faders = pitch bend nos canais 1..8 (9 = master), knobs = CC 16..23 relativos,
/// botões: Solo = notas 8..15, Mute = 16..23, Select = 24..31; transporte 91..95.
pub fn smc_default_mappings() -> Vec<Mapping> {
    let mut m = Vec::new();
    for i in 0..LAYERS {
        let n = i as u8;
        m.push(Mapping {
            source: Source::PitchBend { ch: n },
            target: Target::Volume(i),
        });
        m.push(Mapping {
            source: Source::Cc {
                ch: 0,
                num: 16 + n,
                relative: true,
            },
            target: Target::Reverb(i),
        });
        m.push(Mapping {
            source: Source::Note { ch: 0, num: 16 + n },
            target: Target::Toggle(i),
        });
        m.push(Mapping {
            source: Source::Note { ch: 0, num: 8 + n },
            target: Target::Solo(i),
        });
        // Botão REC (0..7) e SELECT (24..31) também ligam/desligam: o que estiver mais à mão.
        m.push(Mapping {
            source: Source::Note { ch: 0, num: n },
            target: Target::Toggle(i),
        });
        m.push(Mapping {
            source: Source::Note { ch: 0, num: 24 + n },
            target: Target::Toggle(i),
        });
    }
    m.push(Mapping {
        source: Source::PitchBend { ch: 8 },
        target: Target::Master,
    });
    m.push(Mapping {
        source: Source::Note { ch: 0, num: 91 },
        target: Target::PrevScene,
    });
    m.push(Mapping {
        source: Source::Note { ch: 0, num: 92 },
        target: Target::NextScene,
    });
    m.push(Mapping {
        source: Source::Note { ch: 0, num: 93 },
        target: Target::Panic,
    });
    // Setas (MCU): esquerda/direita = cena, cima/baixo = transpor.
    m.push(Mapping {
        source: Source::Note { ch: 0, num: 98 },
        target: Target::PrevScene,
    });
    m.push(Mapping {
        source: Source::Note { ch: 0, num: 99 },
        target: Target::NextScene,
    });
    m.push(Mapping {
        source: Source::Note { ch: 0, num: 96 },
        target: Target::TransposeUp,
    });
    m.push(Mapping {
        source: Source::Note { ch: 0, num: 97 },
        target: Target::TransposeDown,
    });
    m.push(Mapping {
        source: Source::Note { ch: 0, num: 46 },
        target: Target::PrevScene,
    });
    m.push(Mapping {
        source: Source::Note { ch: 0, num: 47 },
        target: Target::NextScene,
    });
    m
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
