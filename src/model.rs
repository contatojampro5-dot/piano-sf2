//! Dados do programa: camadas, cenas, mapeamento da mesinha e configuração salva.

use serde::{Deserialize, Serialize};

/// Número de camadas (uma por fader da SMC-Mixer).
pub const LAYERS: usize = 6;
/// Cenas que vêm de fábrica.
pub const SCENES: usize = 8;
/// Máximo de cenas (o Theo pode criar/apagar no modo Editar).
pub const MAX_SCENES: usize = 16;

/// Timbre escolhido dentro do SF2 (banco + programa, como no MIDI).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PresetRef {
    pub bank: i32,
    pub patch: i32,
    pub name: String,
    /// v0.35: timbre de um arquivo da Biblioteca ("Pads/Hillsong Pad.sf2"); None = do SF2 principal.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
}

impl PresetRef {
    /// Mesmo timbre (mesmo arquivo, banco e programa)? O nome não conta.
    pub fn same(&self, o: &PresetRef) -> bool {
        self.bank == o.bank && self.patch == o.patch && self.file == o.file
    }
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
    /// v0.29: Cutoff 0..1 (1 = aberto, o som normal do SF2; menos = mais abafado).
    pub cutoff: f32,
    pub preset: Option<PresetRef>,
}

impl Default for Layer {
    fn default() -> Self {
        Self {
            on: false,
            solo: false,
            volume: 0.8,
            reverb: 0.25,
            cutoff: 1.0,
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
            l.cutoff = l.cutoff.clamp(0.0, 1.0);
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
    /// v0.24: o resto do preset (efeitos e o que está ligado, pads, mix dos pads, filtro, tempo, volume geral).
    /// Cena salva antes da v0.24 não tem: carrega só as camadas, como antes.
    pub extra: Option<SceneExtra>,
}

/// v0.24: tudo da cena além das camadas.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct SceneExtra {
    pub efeitos: Efeitos,
    pub pad_bank: usize,
    pub pad_bank2: Option<usize>,
    pub pad_mix: f32,
    pub pad_volume: f32,
    pub master: f32,
    pub vivo: VivoCfg,
}

/// O que um controle da mesinha comanda.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Target {
    Volume(usize),
    Reverb(usize),
    /// v0.29: Cutoff (brilho) da camada.
    Cutoff(usize),
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
    /// v0.15: pitch bend (roda/fader/knob: meio = afinado). Botão: segurar = sobe tudo.
    Pitch,
    /// v0.15: botão que segura o pitch lá em cima / lá embaixo (soltou, volta ao meio).
    PitchUp,
    PitchDown,
    /// v0.15: modulação (vibrato, CC 1). Botão: segurar = vibrato cheio.
    Modulation,
    /// v0.16: Congelar — liga: o som que está tocando fica soando sozinho; desliga: some devagar.
    Freeze,
    /// v0.17 (aba Efeitos): bater o tempo, ligar PulsePad / Arpejador / Voice FX, DrumPad 0..7 e volumes.
    TapTempo,
    PulseToggle,
    ArpToggle,
    VoiceFxToggle,
    Drum(usize),
    DrumVolume,
    VoiceFxVolume,
    /// v0.23: filtro de abrir/fechar (knob/fader) e o botão de swell (entra/sai suave).
    Filter,
    SwellToggle,
    /// v0.24: ↺ volta o último Freeze; mix entre os dois pads.
    FreezeRecall,
    PadMix,
    /// v0.33: Sequencer (▶ toca a batida / ■ para).
    SeqToggle,
}

impl Target {
    /// Alvo contínuo (fader/knob) ou botão?
    pub fn is_continuous(self) -> bool {
        matches!(
            self,
            Target::Volume(_)
                | Target::Reverb(_)
                | Target::Cutoff(_)
                | Target::Master
                | Target::PadVolume
                | Target::PadKnob
                | Target::Pitch
                | Target::Modulation
                | Target::DrumVolume
                | Target::VoiceFxVolume
                | Target::Filter
                | Target::PadMix
        )
    }

    /// Alvo que também aceita botão "de segurar" (apertou = liga, soltou = volta).
    pub fn is_momentary(self) -> bool {
        matches!(
            self,
            Target::Pitch | Target::PitchUp | Target::PitchDown | Target::Modulation
        )
    }

    pub fn label(self) -> String {
        match self {
            Target::Volume(i) => format!("Volume da camada {}", i + 1),
            Target::Reverb(i) => format!("Reverb da camada {}", i + 1),
            Target::Cutoff(i) => format!("Cutoff da camada {}", i + 1),
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
            Target::Pitch => "Pitch bend".into(),
            Target::PitchUp => "Pitch ▲ (segurar sobe)".into(),
            Target::PitchDown => "Pitch ▼ (segurar desce)".into(),
            Target::Modulation => "Modulation (vibrato)".into(),
            Target::Freeze => "Freeze (liga/desliga)".into(),
            Target::TapTempo => "Tap Tempo".into(),
            Target::PulseToggle => "Pulse Pad (liga/desliga)".into(),
            Target::ArpToggle => "Arpeggiator (liga/desliga)".into(),
            Target::VoiceFxToggle => "Voice FX (liga/desliga)".into(),
            Target::Drum(i) => format!("DrumPad {}", crate::efeitos::drum_default_name(i)),
            Target::DrumVolume => "Volume dos DrumPads".into(),
            Target::VoiceFxVolume => "Volume do Voice FX".into(),
            Target::Filter => "Filter Sweep (Cutoff)".into(),
            Target::SwellToggle => "Swell In / Fade Out".into(),
            Target::FreezeRecall => "Freeze: voltar o último (↺)".into(),
            Target::PadMix => "Mix entre os 2 pads".into(),
            Target::SeqToggle => "Sequencer (▶ tocar / ■ parar)".into(),
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

/// O que dá para tirar/colocar da tela no modo Editar: (id, nome mostrado).
/// Camadas são "layer0".."layer5". Ajustes, Editar, GERAL e Pânico nunca saem (para sempre dar para voltar).
pub const EDITABLE: [(&str, &str); 15] = [
    ("layer0", "Camada 1"),
    ("layer1", "Camada 2"),
    ("layer2", "Camada 3"),
    ("layer3", "Camada 4"),
    ("layer4", "Camada 5"),
    ("layer5", "Camada 6"),
    ("pad_strip", "Canal do PAD (7)"),
    ("scenes", "Barra de cenas"),
    ("tab_pads", "Aba Pads"),
    ("kb_button", "Botão Teclado"),
    ("bt_button", "Botão Bluetooth"),
    ("wizard_button", "Botão Configurar mesinha"),
    ("scale_help", "Ajuda de escala (pads)"),
    ("sons_button", "Botão Sons grátis"),
    (
        "pad_styles",
        "Estilos de pad do programa (Escuro, Coral, Vivos...)",
    ),
];

pub fn layer_id(i: usize) -> String {
    format!("layer{i}")
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
    /// v0.15: controles do próprio teclado (porta "Piano", ex.: AMW MIDI Control Mini 32) ligados pelo Aprender.
    /// Ficam separados dos da mesinha: uma tecla do piano nunca dispara um botão da SMC sem querer.
    /// O que está aqui não toca nota: vira o controle escolhido.
    pub piano_mappings: Vec<Mapping>,
    /// v0.15: quantos semitons o pitch bend sobe/desce no máximo (1..12, padrão 2).
    pub pitch_range: u8,
    /// v0.17: ajustes da aba Efeitos (tempo, PulsePad, Arpejador, DrumPads, Voice FX).
    pub efeitos: Efeitos,
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
    /// v0.24: segundo pad tocando junto (um dos "Meus pads"), e o mix entre os dois (0 = só o 1º, 1 = só o 2º).
    pub pad_bank2: Option<usize>,
    pub pad_mix: f32,
    /// Arquivos próprios nos botões de pad.
    pub pad_files: Vec<PadFile>,
    /// Aba aberta (0 = Sons SF2, 1 = Pads).
    pub tab: usize,
    pub look: Appearance,
    /// Aparelhos Bluetooth conectados direto (ver `ble.rs`).
    pub bt_devices: Vec<BtDevice>,
    /// Coisas tiradas da tela no modo Editar (ids de `EDITABLE`).
    pub hidden: Vec<String>,
    pub vivo: VivoCfg,
    /// Ajuda de escala: mostrar o tom do pad como menor (o pad serve para maior e menor).
    pub scale_minor: bool,
    /// "Meus pads" (até 4): arquivo do Theo, o nome que ele deu e o tom em que o som está (0 = C ... 7 = G).
    pub my_pads: Vec<MyPad>,
    /// Já escondeu os estilos de pad do programa (v0.13: o Theo quis só os pads dele)?
    #[serde(default)]
    pub pads_v13: bool,
    /// v0.20: já colocou os pads livres (MuseScore General) nos "Meus pads"?
    #[serde(default)]
    pub pads_v20: bool,
    /// v0.21: já trocou os nomes compridos pelos nomes de planeta?
    #[serde(default)]
    pub pads_v21: bool,
    /// v0.30: já colocou os pads e DrumPads novos (Pixabay) que o Theo trouxe?
    #[serde(default)]
    pub pads_v30: bool,
    /// v0.31: os 4 sons da Pixabay saíram dos "Meus pads" (viraram vozes do Voice FX)?
    #[serde(default)]
    pub pads_v31: bool,
    /// v0.35: pads da Biblioteca que já entraram nos "Meus pads" (se o Theo tirar, não volta sozinho).
    #[serde(default)]
    pub lib_pads_added: Vec<String>,
    /// v0.35: os DrumPads de efeito da Biblioteca (11 a 16) já entraram?
    #[serde(default)]
    pub lib_drums_added: bool,
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

/// Mesinha (ou outro aparelho) conectada direto pelo Bluetooth: reconecta sozinha ao abrir.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BtDevice {
    pub address: u64,
    pub name: String,
}

/// Ajustes do Pad Vivo (granular) e do reverb de igreja.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct VivoCfg {
    pub bright: f32,
    pub shimmer: f32,
    pub motion: f32,
    /// Reverb de igreja nos pads (0..1).
    pub church_pads: f32,
    /// Reverb de igreja nas camadas do SF2 (0..1).
    pub church_sf2: f32,
    /// Som próprio como base de cada Vivo: (0..2, arquivo, nota MIDI do arquivo).
    pub files: Vec<(usize, String, u8)>,
}

impl Default for VivoCfg {
    fn default() -> Self {
        Self {
            bright: 0.6,
            shimmer: 0.35,
            motion: 0.45,
            church_pads: 0.4,
            church_sf2: 0.0,
            files: Vec::new(),
        }
    }
}

/// v0.17: ajustes da aba Efeitos. O que está ligado (PulsePad, Arpejador, Voice FX) não fica salvo:
/// o programa sempre abre com tudo desligado (mais seguro no culto).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Efeitos {
    pub bpm: f32,
    /// PulsePad: pulsos por tempo (1, 2 ou 4) e o quanto cai entre eles (0..1).
    pub pulse_per_beat: u32,
    pub pulse_depth: f32,
    /// Arpejador: camada (0..5), padrão (0 sobe, 1 desce, 2 sobe e desce, 3 aleatório),
    /// notas por tempo (2, 3 ou 4), oitavas (1..3) e "segurar".
    pub arp_layer: usize,
    pub arp_pattern: u8,
    pub arp_per_beat: u32,
    pub arp_octaves: u8,
    pub arp_latch: bool,
    pub drum_volume: f32,
    pub voicefx_volume: f32,
    /// Voice FX: oitava do coral (−1, 0, +1).
    pub voicefx_octave: i8,
    /// v0.19: som do Theo em cada DrumPad (pad 0..7, arquivo WAV/MP3/OGG/FLAC). Sem = som do programa.
    pub drum_files: Vec<(usize, String)>,
    /// v0.19: coral do Theo no Voice FX (arquivo, nota MIDI em que ele está). Sem = coral do programa.
    pub voicefx_file: Option<(String, u8)>,
    /// v0.21: voz escolhida no Voice FX: 0..4 = vozes do programa (`pads::VOICE_NAMES`), 5 = "Minha" (o arquivo acima).
    pub voicefx_voice: usize,
    /// v0.23: filtro de abrir/fechar: 0 = fechado (escuro) … 1 = aberto (normal).
    pub filtro: f32,
    /// v0.23: ressonância do filtro (0..1, o "brilho" da abertura).
    pub filtro_res: f32,
    /// v0.23: filtro e swell em tudo menos a bateria (false = só no fundo: pads e congelado).
    pub filtro_tudo: bool,
    /// v0.23: quanto tempo o swell leva para entrar/sair (segundos).
    pub swell_secs: f32,
    /// v0.23: o swell abre/fecha o filtro junto com o volume.
    pub swell_filtro: bool,
    /// v0.24: o que estava ligado (o programa abre do jeito que ficou).
    pub pulse_on: bool,
    pub arp_on: bool,
    pub voicefx_on: bool,
    /// v0.28: quantos DrumPads aparecem (4..16; os 8 primeiros têm som do programa, os outros esperam um som seu).
    pub drum_count: usize,
    /// v0.28: nome escolhido para cada DrumPad ("" = nome do programa).
    pub drum_names: Vec<String>,
    /// v0.28: cor de cada DrumPad (índice das cores neon; 255 = a do programa).
    pub drum_colors: Vec<u8>,
    /// v0.31: tom em que as texturas do Voice FX tocam (0 = C … 11 = B).
    pub voicefx_key: usize,
    /// v0.33: Sequencer: ritmo pronto escolhido (`efeitos::RHYTHMS`) e a batida de cada DrumPad
    /// (um número por pad; bit 0 = passo 1). O ▶ não fica salvo: o programa abre com a batida parada.
    pub seq_rhythm: usize,
    pub seq_grid: Vec<u16>,
    /// v0.34: Frase do Arpeggiator (padrão "Frase"): as notas gravadas no Learn (até 16) e se ela segue a tecla.
    pub arp_phrase: Vec<u8>,
    pub arp_follow: bool,
}

/// v0.21: número da voz "Minha" (o arquivo do Theo) no Voice FX.
pub const VOICE_MINE: usize = 5;

impl Default for Efeitos {
    fn default() -> Self {
        Self {
            bpm: 75.0,
            pulse_per_beat: 2,
            pulse_depth: 0.85,
            arp_layer: 1,
            arp_pattern: 0,
            arp_per_beat: 2,
            arp_octaves: 2,
            arp_latch: false,
            drum_volume: 0.8,
            voicefx_volume: 0.6,
            voicefx_octave: 0,
            drum_files: Vec::new(),
            voicefx_file: None,
            voicefx_voice: 0,
            filtro: 1.0,
            filtro_res: 0.25,
            filtro_tudo: false,
            swell_secs: 4.0,
            swell_filtro: true,
            pulse_on: false,
            arp_on: false,
            voicefx_on: false,
            drum_count: crate::efeitos::BUILTIN_DRUMS,
            drum_names: Vec::new(),
            drum_colors: Vec::new(),
            voicefx_key: 0,
            seq_rhythm: 0,
            seq_grid: crate::efeitos::rhythm_grid(0).to_vec(),
            arp_phrase: Vec::new(),
            arp_follow: false,
        }
    }
}

impl Efeitos {
    pub fn normalize(&mut self) {
        self.bpm = self
            .bpm
            .clamp(crate::efeitos::BPM_MIN, crate::efeitos::BPM_MAX)
            .round();
        if !matches!(self.pulse_per_beat, 1 | 2 | 4) {
            self.pulse_per_beat = 2;
        }
        self.pulse_depth = self.pulse_depth.clamp(0.0, 1.0);
        self.arp_layer = self.arp_layer.min(LAYERS - 1);
        self.arp_pattern = self.arp_pattern.min(crate::efeitos::ARP_PHRASE);
        self.arp_phrase.truncate(crate::efeitos::PHRASE_MAX);
        self.arp_phrase.retain(|n| *n < 128);
        if !matches!(self.arp_per_beat, 2..=4) {
            self.arp_per_beat = 2;
        }
        self.arp_octaves = self.arp_octaves.clamp(1, 3);
        self.drum_volume = self.drum_volume.clamp(0.0, 1.0);
        self.voicefx_volume = self.voicefx_volume.clamp(0.0, 1.0);
        self.voicefx_octave = self.voicefx_octave.clamp(-1, 1);
        self.filtro = self.filtro.clamp(0.0, 1.0);
        self.filtro_res = self.filtro_res.clamp(0.0, 1.0);
        self.swell_secs = self.swell_secs.clamp(0.5, 16.0);
        self.voicefx_key %= 12;
        let max_voice = crate::pads::TEXTURE_BASE + crate::pads::TEXTURE_VOICES.len() - 1;
        if self.voicefx_voice > max_voice || (self.voicefx_voice == VOICE_MINE && self.voicefx_file.is_none()) {
            self.voicefx_voice = 0;
        }
        self.drum_files.retain(|(i, _)| *i < crate::efeitos::MAX_DRUMS);
        self.drum_count = if self.drum_count == 0 { crate::efeitos::BUILTIN_DRUMS } else { self.drum_count.clamp(4, crate::efeitos::MAX_DRUMS) };
        self.drum_names.truncate(crate::efeitos::MAX_DRUMS);
        self.drum_colors.truncate(crate::efeitos::MAX_DRUMS);
        self.drum_files.sort_by_key(|(i, _)| *i);
        self.drum_files.dedup_by_key(|(i, _)| *i);
        // v0.33: batida do Sequencer: um número por DrumPad, só os passos do compasso do ritmo
        self.seq_rhythm = self.seq_rhythm.min(crate::efeitos::RHYTHMS.len() - 1);
        self.seq_grid.resize(crate::efeitos::MAX_DRUMS, 0);
        let mask = self.seq_mask();
        for g in &mut self.seq_grid {
            *g &= mask;
        }
    }

    /// v0.33: quantos passos o compasso do ritmo escolhido tem (16, ou 12 no 6/8).
    pub fn seq_steps(&self) -> usize {
        crate::efeitos::RHYTHMS[self.seq_rhythm.min(crate::efeitos::RHYTHMS.len() - 1)].steps as usize
    }

    fn seq_mask(&self) -> u16 {
        ((1u32 << self.seq_steps()) - 1) as u16
    }

    /// v0.33: a batida mudou do ritmo pronto (o Theo acendeu/apagou algum quadradinho)?
    pub fn seq_edited(&self) -> bool {
        self.seq_grid[..] != crate::efeitos::rhythm_grid(self.seq_rhythm)[..]
    }

    /// v0.33: o que o motor precisa para tocar a batida.
    pub fn seq_params(&self, on: bool) -> crate::efeitos::SeqParams {
        let r = &crate::efeitos::RHYTHMS[self.seq_rhythm.min(crate::efeitos::RHYTHMS.len() - 1)];
        let mut grid = [0u16; crate::efeitos::MAX_DRUMS];
        for (i, g) in self.seq_grid.iter().take(crate::efeitos::MAX_DRUMS).enumerate() {
            grid[i] = *g;
        }
        crate::efeitos::SeqParams { on, steps: r.steps, per_beat: r.per_beat, grid }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MyPad {
    pub name: String,
    pub path: String,
    pub key: usize,
    /// Tom ainda não conferido: o programa descobre pelo som quando carregar.
    #[serde(default)]
    pub auto_key: bool,
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
            piano_mappings: Vec::new(),
            pitch_range: 2,
            efeitos: Efeitos::default(),
            port_roles: Vec::new(),
            audio_device: None,
            buffer_frames: 256,
            polyphony: 128,
            program_change_scenes: false,
            pad_volume: 0.7,
            pad_bank: 0,
            pad_bank2: None,
            pad_mix: 0.5,
            pad_files: Vec::new(),
            tab: 0,
            mappings_version: MAPPINGS_VERSION,
            look: Appearance::default(),
            bt_devices: Vec::new(),
            hidden: Vec::new(),
            vivo: VivoCfg::default(),
            scale_minor: false,
            my_pads: Vec::new(),
            pads_v13: false,
            pads_v20: false,
            pads_v21: false,
            pads_v30: false,
            pads_v31: false,
            lib_pads_added: Vec::new(),
            lib_drums_added: false,
        }
    }
}

fn default_scenes() -> Vec<Scene> {
    (0..SCENES)
        .map(|i| Scene {
            name: format!("Cena {}", i + 1),
            mix: None,
            extra: None,
        })
        .collect()
}

impl Config {
    pub fn normalize(&mut self) {
        self.mix.normalize();
        self.master = self.master.clamp(0.0, 1.0);
        self.scenes.truncate(MAX_SCENES);
        if self.scenes.is_empty() {
            self.scenes.push(Scene::default());
        }
        for (i, s) in self.scenes.iter_mut().enumerate() {
            if s.name.trim().is_empty() {
                s.name = format!("Cena {}", i + 1);
            }
            if let Some(m) = &mut s.mix {
                m.normalize();
            }
        }
        if self.current_scene.is_some_and(|c| c >= self.scenes.len()) {
            self.current_scene = None;
        }
        self.polyphony = self.polyphony.clamp(16, 256);
        self.pitch_range = self.pitch_range.clamp(1, 12);
        self.efeitos.normalize();
        self.pad_volume = self.pad_volume.clamp(0.0, 1.0);
        self.pad_bank %= crate::pads::ALL_BANKS;
        self.pad_mix = self.pad_mix.clamp(0.0, 1.0);
        if !self.pads_v13 {
            self.pads_v13 = true;
            if !self.is_hidden("pad_styles") {
                self.hidden.push("pad_styles".into());
            }
            // os 3 pads da parceria já entram colocados
            if self.my_pads.is_empty() {
                self.my_pads = crate::pads::PARTNER_PADS
                    .iter()
                    .enumerate()
                    .map(|(n, p)| MyPad {
                        name: p.name.into(),
                        path: crate::pads::partner_path(n),
                        key: p.key,
                        auto_key: false,
                    })
                    .collect();
                self.pad_bank = crate::pads::MY_BANK;
            }
        }
        if !self.pads_v20 {
            // v0.20: os 3 pads livres entram sozinhos (sem apagar os pads que o Theo já tinha)
            self.pads_v20 = true;
            for (n, p) in crate::pads::FREE_PADS.iter().enumerate() {
                let path = crate::pads::free_path(n);
                if self.my_pads.len() < crate::pads::MY_SLOTS && !self.my_pads.iter().any(|m| m.path == path) {
                    self.my_pads.push(MyPad {
                        name: p.name.into(),
                        path,
                        key: p.key,
                        auto_key: false,
                    });
                }
            }
        }
        if !self.pads_v21 {
            // v0.21: nomes curtos de planeta (só onde ainda está o nome antigo: o que o Theo renomeou fica)
            self.pads_v21 = true;
            for m in &mut self.my_pads {
                if let Some((new, old)) = crate::pads::planet_name(&m.path) {
                    if m.name == old {
                        m.name = new.into();
                    }
                }
            }
        }
        if !self.pads_v30 {
            // v0.30: os 4 pads novos entram nas vagas livres e aparecem os DrumPads 9 e 10 (Choir Rise, Prayer)
            self.pads_v30 = true;
            for (n, p) in crate::pads::FREE_PADS.iter().enumerate().skip(3) {
                let path = crate::pads::free_path(n);
                if self.my_pads.len() < crate::pads::MY_SLOTS && !self.my_pads.iter().any(|m| m.path == path) {
                    self.my_pads.push(MyPad {
                        name: p.name.into(),
                        path,
                        key: p.key,
                        auto_key: false,
                    });
                }
            }
            self.efeitos.drum_count = self.efeitos.drum_count.max(crate::efeitos::BUILTIN_DRUMS);
        }
        if !self.pads_v31 {
            // v0.31: Urano, Mercúrio, Lua e Plutão agora moram no Voice FX (livre:3..6 não existe mais nos pads)
            self.pads_v31 = true;
            self.my_pads.retain(|m| !matches!(m.path.as_str(), "livre:3" | "livre:4" | "livre:5" | "livre:6"));
        }
        self.my_pads.truncate(crate::pads::MY_SLOTS);
        // v0.24: o segundo pad tem que ser um dos "Meus pads" e diferente do principal
        if let Some(b2) = self.pad_bank2 {
            let ok = crate::pads::my_slot(b2).is_some_and(|i| i < self.my_pads.len()) && b2 != self.pad_bank;
            if !ok {
                self.pad_bank2 = None;
            }
        }
        for m in &mut self.my_pads {
            m.key %= crate::pads::PAD_KEYS;
            if m.name.trim().is_empty() {
                m.name = "Meu pad".into();
            }
        }
        if crate::pads::my_slot(self.pad_bank).is_some_and(|i| i >= self.my_pads.len()) {
            self.pad_bank = 0;
        }
        for v in [
            &mut self.vivo.bright,
            &mut self.vivo.shimmer,
            &mut self.vivo.motion,
            &mut self.vivo.church_pads,
            &mut self.vivo.church_sf2,
        ] {
            *v = v.clamp(0.0, 1.0);
        }
        self.vivo.files.retain(|(n, _, _)| *n < 3);
        if self.tab != 4 {
            self.tab = self.tab.min(3);
        }
        self.look.frost = self.look.frost.clamp(0.0, 1.0);
        self.look.glow = self.look.glow.clamp(0.0, 1.0);
        self.look.scale = self.look.scale.clamp(0.75, 1.5);
        self.pad_files
            .retain(|f| f.bank < crate::pads::PAD_BANKS && f.key < crate::pads::PAD_KEYS);
        // Nada pode apontar para camada que não existe mais (eram 8, agora são 6).
        self.mappings.retain(|m| match m.target {
            Target::Volume(i) | Target::Reverb(i) | Target::Cutoff(i) | Target::Toggle(i) | Target::Solo(i) => {
                i < LAYERS
            }
            Target::Scene(n) => n < MAX_SCENES,
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

    pub fn is_hidden(&self, id: &str) -> bool {
        self.hidden.iter().any(|h| h == id)
    }

    pub fn set_hidden(&mut self, id: &str, hidden: bool) {
        self.hidden.retain(|h| h != id);
        if hidden {
            self.hidden.push(id.to_string());
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

#[cfg(test)]
mod workstation_config_tests {
    use super::Config;

    #[test]
    fn workstation_tab_round_trips_and_legacy_tabs_keep_their_indices() {
        for index in 0..=4 {
            let mut data = serde_json::to_value(Config::default()).unwrap();
            data["tab"] = serde_json::json!(index);
            let mut config: Config = serde_json::from_value(data).unwrap();
            config.normalize();
            assert_eq!(config.tab, index);
        }
        let mut data = serde_json::to_value(Config::default()).unwrap();
        data.as_object_mut().unwrap().remove("tab");
        let mut config: Config = serde_json::from_value(data).unwrap();
        config.normalize();
        assert_eq!(config.tab, 0);
    }
}
