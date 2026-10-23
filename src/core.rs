//! Cérebro do programa: guarda o estado da mesa e traduz piano/mesinha/tela em comandos
//! para o motor. Usado pela tela e pelas threads de MIDI (sempre atrás de um Mutex).

use std::collections::HashMap;
use std::io::Read;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use crossbeam_channel::Sender;
use rustysynth::SoundFont;

use crate::engine::{Cmd, Engine};
use crate::model::*;
use crate::pads::{self, PadClip, PAD_BANKS, PAD_KEYS};

/// Pedido de carregar um pad: (banco, tom, arquivo próprio).
pub type PadJob = (usize, usize, Option<String>);

pub struct Learn {
    pub active: bool,
    pub target: Option<Target>,
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
    pad_jobs: std::collections::HashSet<(usize, usize)>,
    /// Thread que decodifica pads (sem ela, decodifica na hora — usado nos testes).
    pub pad_loader: Option<Sender<PadJob>>,
}

pub type SharedCore = Arc<Mutex<Core>>;

impl Core {
    pub fn new(cfg: Config, tx: Sender<Cmd>) -> Self {
        Self {
            cfg,
            tx,
            presets: Vec::new(),
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
            pad_jobs: Default::default(),
            pad_loader: None,
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
            self.send(Cmd::SetAudible {
                layer: i,
                on: mix.audible(i) && l.preset.is_some(),
            });
        }
        self.send(Cmd::SetMaster(self.cfg.master));
        self.send(Cmd::SetTranspose(mix.transpose));
        self.send(Cmd::PadVolume(self.cfg.pad_volume));
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

    fn start_pad(&mut self, key: usize) {
        let bank = self.cfg.pad_bank;
        self.pad_active = Some(key);
        match self.pad_clips[bank][key].clone() {
            Some(clip) => {
                self.pad_pending = None;
                self.send(Cmd::PadPlay(clip));
            }
            None => {
                self.pad_pending = Some((bank, key));
                self.request_pad(bank, key);
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
        let bank = bank % PAD_BANKS;
        if bank == self.cfg.pad_bank {
            return;
        }
        self.cfg.pad_bank = bank;
        self.dirty = true;
        self.preload_pad_bank(bank);
        if let Some(k) = self.pad_active {
            self.start_pad(k); // troca suave para o mesmo tom no outro banco
        }
        self.notify(format!("Pads: {}", pads::BANK_NAMES[bank]));
    }

    /// Põe um arquivo próprio num botão (None = volta ao som padrão).
    pub fn set_pad_file(&mut self, bank: usize, key: usize, path: Option<String>) {
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
        self.pad_clips[bank][key].is_some()
    }

    fn sync_audible(&self) {
        let mix = &self.cfg.mix;
        for i in 0..LAYERS {
            let on = mix.audible(i) && mix.layers[i].preset.is_some();
            self.send(Cmd::SetAudible { layer: i, on });
        }
    }

    // ---------- Ações (tela, mesinha e teclado chamam estas) ----------

    pub fn set_volume(&mut self, i: usize, v: f32) {
        let v = v.clamp(0.0, 1.0);
        self.cfg.mix.layers[i].volume = v;
        self.send(Cmd::SetVolume { layer: i, value: v });
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
        self.sync_audible();
        self.dirty = true;
    }

    /// Troca para o timbre anterior/próximo da lista do SF2.
    pub fn step_preset(&mut self, i: usize, delta: i32) {
        if self.presets.is_empty() {
            return;
        }
        let cur = self.cfg.mix.layers[i].preset.as_ref().and_then(|p| {
            self.presets
                .iter()
                .position(|q| q.bank == p.bank && q.patch == p.patch)
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

    pub fn panic(&mut self) {
        self.send(Cmd::Panic);
        self.notify("Pânico: todas as notas cortadas");
    }

    pub fn load_scene(&mut self, n: usize) {
        if n >= SCENES {
            return;
        }
        match self.cfg.scenes[n].mix.clone() {
            Some(mix) => {
                self.cfg.mix = mix;
                self.cfg.mix.normalize();
                self.cfg.current_scene = Some(n);
                self.sync_engine();
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
        if n >= SCENES {
            return;
        }
        self.cfg.scenes[n].mix = Some(self.cfg.mix.clone());
        self.cfg.current_scene = Some(n);
        self.dirty = true;
        self.notify(format!("Salvo em {}", self.cfg.scenes[n].name));
    }

    pub fn step_scene(&mut self, delta: i32) {
        let cur = self
            .cfg
            .current_scene
            .map(|c| c as i32)
            .unwrap_or(if delta > 0 { -1 } else { 0 });
        let next = (cur + delta).rem_euclid(SCENES as i32) as usize;
        self.load_scene(next);
    }

    pub fn reset_mappings(&mut self) {
        self.cfg.mappings = smc_default_mappings();
        self.dirty = true;
        self.notify("Controles da SMC-Mixer voltaram ao padrão");
    }

    fn apply_button(&mut self, t: Target) {
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
            _ => {}
        }
    }

    fn apply_continuous(&mut self, t: Target, value: ContValue) {
        let cur = match t {
            Target::Volume(i) => self.cfg.mix.layers[i].volume,
            Target::Reverb(i) => self.cfg.mix.layers[i].reverb,
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
            Target::Master => self.set_master(v),
            Target::PadVolume => self.set_pad_volume(v),
            _ => {}
        }
    }

    // ---------- MIDI ----------

    /// Mensagem vinda do piano: toca direto no motor.
    pub fn handle_piano(&mut self, msg: &[u8]) {
        let Some(&status) = msg.first() else { return };
        let d1 = msg.get(1).copied().unwrap_or(0) & 0x7F;
        let d2 = msg.get(2).copied().unwrap_or(0) & 0x7F;
        self.piano_activity = Some(Instant::now());
        match status & 0xF0 {
            0x90 if d2 > 0 => self.send(Cmd::NoteOn { key: d1, vel: d2 }),
            0x90 | 0x80 => self.send(Cmd::NoteOff { key: d1 }),
            0xB0 => self.send(Cmd::Cc { num: d1, val: d2 }),
            0xE0 => self.send(Cmd::PitchBend { lsb: d1, msb: d2 }),
            0xC0 if self.cfg.program_change_scenes && (d1 as usize) < SCENES => {
                self.load_scene(d1 as usize);
                self.request_repaint();
            }
            _ => {}
        }
    }

    /// Mensagem vinda da mesinha. Devolve true se algo mudou na tela.
    pub fn handle_controller(&mut self, msg: &[u8]) -> bool {
        let Some(ev) = ControlEvent::parse(msg) else {
            return false;
        };
        self.controller_activity = Some(Instant::now());

        // Botão "apertado" agora? (CC: subiu de <64 para >=64; nota: velocity > 0)
        let pressed = match ev {
            ControlEvent::Note { vel, .. } => vel > 0,
            ControlEvent::Cc { ch, num, val } => {
                let prev = self.last_cc.insert((ch, num), val).unwrap_or(0);
                val >= 64 && prev < 64
            }
            ControlEvent::PitchBend { .. } => false,
        };

        if self.learn.active {
            if let Some(t) = self.learn.target {
                if let Some(src) = ev.source_for(t, pressed) {
                    self.cfg
                        .mappings
                        .retain(|m| !same_source(m.source, src) && m.target != t);
                    self.cfg.mappings.push(Mapping {
                        source: src,
                        target: t,
                    });
                    self.learn.target = None;
                    self.dirty = true;
                    self.notify(format!("Aprendido: {} → {}", src.label(), t.label()));
                    return true;
                }
                return false;
            }
        }

        let hits: Vec<Mapping> = self
            .cfg
            .mappings
            .iter()
            .filter(|m| ev.matches(m.source))
            .copied()
            .collect();
        let mut changed = false;
        for m in hits {
            if m.target.is_continuous() {
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
            })
            .collect();
        presets.sort_by_key(|p| (p.bank, p.patch));
        presets.dedup_by_key(|p| (p.bank, p.patch));
        self.presets = presets;
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
                if let Some(q) = presets
                    .iter()
                    .find(|q| q.bank == p.bank && q.patch == p.patch)
                {
                    p.name = q.name.clone();
                }
            }
        };
        for l in &mut self.cfg.mix.layers {
            fix(l, &self.presets);
        }
        self.rebuild_synth();
    }

    /// Cria um sintetizador novo com o SF2 atual e manda para o motor.
    pub fn rebuild_synth(&mut self) {
        if let Some(sf) = &self.sound_font {
            if let Some(synth) = Engine::build_synth(sf, self.sample_rate, self.cfg.polyphony) {
                self.send(Cmd::Replace(synth));
                self.sync_engine();
            }
        }
    }
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
            while let Ok((bank, key, custom)) = rx.recv() {
                let r = pads::load(bank, key, custom.as_deref());
                if let Ok(mut c) = worker.lock() {
                    c.pad_loaded(bank, key, r);
                }
            }
        })
        .expect("thread dos pads");
    // Os dois bancos ficam prontos (o atual primeiro): trocar Escuro/Coral no culto é na hora.
    let mut c = core.lock().unwrap();
    c.preload_pad_bank(bank);
    for b in 0..PAD_BANKS {
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
            ControlEvent::Note { ch, num, vel } if !t.is_continuous() && vel > 0 => {
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
            },
            PresetRef {
                bank: 0,
                patch: 48,
                name: "Cordas".into(),
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
}
