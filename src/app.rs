//! A tela: 8 canais como na M-Vave SMC-Mixer + canal geral.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crossbeam_channel::Receiver;
use eframe::egui::{
    self, Align, Align2, Color32, CornerRadius, FontId, Key, Layout, Pos2, Rect, RichText, Sense,
    Shape, Stroke, StrokeKind, Ui, Vec2,
};

use crate::audio::{self, Audio};
use crate::core::{load_sf2_async, start_pad_loader, Core, SharedCore};
use crate::engine::{Cmd, Engine, Garbage, Stats};
use crate::midi::{self, BtCmd, SharedMidiStatus};
use crate::model::*;
use crate::pads;
use crate::sons;
use crate::theme::{self, th};

// ---------- Cores ----------
const GREEN: Color32 = Color32::from_rgb(0x30, 0xd1, 0x58);
const YELLOW: Color32 = Color32::from_rgb(0xff, 0xd6, 0x0a);
const RED: Color32 = Color32::from_rgb(0xff, 0x45, 0x3a);
const ORANGE: Color32 = Color32::from_rgb(0xff, 0x9f, 0x0a);
const LAYER_COLORS: [Color32; LAYERS] = [
    Color32::from_rgb(0x64, 0xd2, 0xff),
    Color32::from_rgb(0x30, 0xd1, 0x58),
    Color32::from_rgb(0xff, 0xb3, 0x40),
    Color32::from_rgb(0xff, 0x5e, 0x9a),
    Color32::from_rgb(0xbf, 0x5a, 0xf2),
    Color32::from_rgb(0x5e, 0xe6, 0xd0),
];

const BUFFERS: [u32; 6] = [0, 64, 128, 256, 512, 1024];
const POLYPHONY: [usize; 5] = [64, 96, 128, 192, 256];

struct UiState {
    picker: Option<usize>,
    picker_search: String,
    picker_focus: bool,
    settings: bool,
    /// Modo Editar: tirar/colocar coisas da tela.
    editing: bool,
    /// Janela "Bluetooth" aberta (e se já mandou procurar).
    bt_open: bool,
    bt_was_open: bool,
    rename: Option<(usize, String)>,
    audio_devices: Vec<String>,
    /// Pedidos que só podem rodar com o `core` destravado (abrir arquivo trava a tela até escolher).
    open_sf2_dialog: bool,
    pad_file_dialog: Option<(usize, usize)>,
    /// Janela de arquivo do "＋ Pôr um pad meu".
    my_pad_dialog: bool,
    /// SF2 para abrir assim que o `core` destravar (ex.: botão "Usar este som").
    load_sf2: Option<String>,
    /// Janela "Sons grátis" e o download em andamento.
    sons_open: bool,
    download: Option<sons::Download>,
    dl_frac: f32,
    sons_msg: Option<(bool, String)>,
    /// Nota do som próprio do Pad Vivo (0 = C ... 11 = B).
    vivo_root_key: usize,
    /// Sequência de acordes mostrada na ajuda de escala.
    prog_idx: usize,
    /// Teclado de piano na tela (abre/fecha com o botão "Teclado").
    kb_open: bool,
    /// Nota que o mouse está segurando.
    kb_mouse: Option<u8>,
    /// Teclas do computador seguradas → nota que tocaram.
    kb_keys: Vec<(Key, u8)>,
    /// Oitava das teclas do computador (A = Dó dessa oitava).
    kb_octave: i32,
    kb_sustain: bool,
    /// v0.18 (tela limpa): mostrar os ajustes finos de cada tela (⚙). Fechados = "modo palco".
    mix_details: bool,
    pad_details: bool,
    /// Tempo, PulsePad, Arpejador, Voice FX.
    fx_details: [bool; 5],
    /// v0.19: janelas de arquivo (só com o `core` destravado): som de um DrumPad, coral do Voice FX.
    drum_file_dialog: Option<usize>,
    voicefx_file_dialog: bool,
    /// v0.21: lista de pads aberta (a bolinha de planeta na aba Pads).
    pad_list_open: bool,
    /// v0.24: modo "+ Misturar" (o próximo pad clicado entra como segundo pad).
    pad_mix_mode: bool,
}

pub struct PianoApp {
    core: SharedCore,
    audio: Audio,
    stats: Arc<Stats>,
    garbage: Receiver<Garbage>,
    midi_status: SharedMidiStatus,
    midi_rescan: Arc<AtomicBool>,
    /// Pedidos para o Bluetooth direto (procurar / conectar / esquecer).
    bt: crossbeam_channel::Sender<BtCmd>,
    ui: UiState,
    last_save: Instant,
    audio_retry: Option<Instant>,
    config_path: std::path::PathBuf,
    /// Aparência já aplicada no estilo do egui (reaplica quando muda nos Ajustes).
    look_applied: Option<Appearance>,
}

impl PianoApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        setup_style(&cc.egui_ctx);

        let config_path = config_path();
        let cfg = Config::load_from(&config_path);

        let (tx, rx) = crossbeam_channel::unbounded::<Cmd>();
        let (gtx, grx) = crossbeam_channel::unbounded();
        let stats = Arc::new(Stats::default());
        let engine = Arc::new(Mutex::new(Engine::new(rx, gtx, stats.clone())));

        let mut core = Core::new(cfg, tx.clone());
        let ctx = cc.egui_ctx.clone();
        core.repaint = Some(Box::new(move || ctx.request_repaint()));
        let core: SharedCore = Arc::new(Mutex::new(core));

        let mut audio = Audio::new(engine);
        {
            let mut c = core.lock().unwrap();
            let (dev, buf) = (c.cfg.audio_device.clone(), c.cfg.buffer_frames);
            if let Ok(sr) = audio.start(dev.as_deref(), buf) {
                c.set_sample_rate(sr);
            }
            c.sync_engine();
            if let Some(path) = c.cfg.sf2_path.clone() {
                drop(c);
                if std::path::Path::new(&path).exists() {
                    load_sf2_async(core.clone(), path);
                }
            }
        }

        start_pad_loader(core.clone());

        let midi_status: SharedMidiStatus = Default::default();
        let midi_rescan = Arc::new(AtomicBool::new(true));
        let (bt, bt_rx) = crossbeam_channel::unbounded::<BtCmd>();
        #[cfg(windows)]
        crate::btdirect::start(core.clone(), tx.clone(), midi_status.clone(), bt_rx);
        #[cfg(not(windows))]
        drop(bt_rx);
        midi::start(core.clone(), tx, midi_status.clone(), midi_rescan.clone());

        Self {
            core,
            audio,
            stats,
            garbage: grx,
            midi_status,
            midi_rescan,
            bt,
            ui: UiState {
                picker: None,
                picker_search: String::new(),
                picker_focus: false,
                settings: false,
                editing: false,
                bt_open: false,
                bt_was_open: false,
                rename: None,
                audio_devices: Vec::new(),
                open_sf2_dialog: false,
                pad_file_dialog: None,
                my_pad_dialog: false,
                load_sf2: None,
                sons_open: false,
                download: None,
                dl_frac: 0.0,
                sons_msg: None,
                vivo_root_key: 0,
                prog_idx: 0,
                kb_open: false,
                kb_mouse: None,
                kb_keys: Vec::new(),
                kb_octave: 4,
                kb_sustain: false,
                mix_details: false,
                pad_details: false,
                fx_details: [false; 5],
                drum_file_dialog: None,
                voicefx_file_dialog: false,
                pad_list_open: false,
                pad_mix_mode: false,
            },
            last_save: Instant::now(),
            audio_retry: None,
            config_path,
            look_applied: None,
        }
    }

    fn restart_audio(&mut self) {
        let mut c = self.core.lock().unwrap();
        let (dev, buf) = (c.cfg.audio_device.clone(), c.cfg.buffer_frames);
        if let Ok(sr) = self.audio.start(dev.as_deref(), buf) {
            c.set_sample_rate(sr);
        }
    }

    fn housekeeping(&mut self) {
        // Sintetizadores velhos são apagados aqui, longe da thread de áudio.
        while self.garbage.try_recv().is_ok() {}

        // O Windows derrubou o áudio? Tenta de novo a cada 2 s.
        if self.audio.failed.load(Ordering::Relaxed) || self.audio.info.is_none() {
            match self.audio_retry {
                Some(t) if t.elapsed() < Duration::from_secs(2) => {}
                _ => {
                    self.audio_retry = Some(Instant::now());
                    self.restart_audio();
                }
            }
        }

        let mut c = self.core.lock().unwrap();
        if c.dirty && self.last_save.elapsed() > Duration::from_secs(2) {
            c.dirty = false;
            self.last_save = Instant::now();
            let _ = c.cfg.save_to(&self.config_path);
        }
    }
}

impl eframe::App for PianoApp {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        self.housekeeping();
        let ctx = ui.ctx().clone();
        let core_arc = self.core.clone();
        let mut core = core_arc.lock().unwrap();

        // Visual: tema (vidro + fundo celestial).
        let look = core.cfg.look.clone();
        if self.look_applied.as_ref() != Some(&look) {
            theme::set(theme::Theme::build(look.palette, look.frost, look.glow));
            theme::apply_style(&ctx, look.scale);
            self.look_applied = Some(look.clone());
        }
        theme::paint_background(&ctx, ctx.input(|i| i.time), look.motion, look.stars);

        keyboard(&ctx, &mut core, &mut self.ui);

        egui::Panel::top("topo")
            .frame(
                egui::Frame::new()
                    .fill(th().bar)
                    .inner_margin(egui::Margin::symmetric(14, 8)),
            )
            .show(ui, |ui| {
                top_bar(
                    ui,
                    &mut core,
                    &self.audio,
                    &self.stats,
                    &self.midi_status,
                    &mut self.ui,
                );
            });
        egui::Panel::top("abas")
            .frame(egui::Frame::new().inner_margin(egui::Margin {
                left: 14,
                right: 14,
                top: 6,
                bottom: 0,
            }))
            .show(ui, |ui| tabs_bar(ui, &mut core));
        if core.cfg.tab == 0 && (!core.cfg.is_hidden("scenes") || self.ui.editing) {
            egui::Panel::top("cenas")
                .frame(egui::Frame::new().inner_margin(egui::Margin {
                    left: 14,
                    right: 14,
                    top: 10,
                    bottom: 4,
                }))
                .show(ui, |ui| scenes_bar(ui, &mut core, &mut self.ui));
        }
        egui::Panel::bottom("rodape")
            .frame(
                egui::Frame::new()
                    .fill(th().bar)
                    .inner_margin(egui::Margin::symmetric(14, 5)),
            )
            .show(ui, |ui| footer(ui, &mut core, &self.stats));
        // v0.18.2: o teclado da tela continua aberto ao trocar de aba (Pads, Efeitos)
        if self.ui.kb_open {
            egui::Panel::bottom("teclado")
                .frame(
                    egui::Frame::new()
                        .fill(th().bar)
                        .inner_margin(egui::Margin::symmetric(14, 8)),
                )
                .show(ui, |ui| piano_keyboard(ui, &mut core, &mut self.ui));
        } else {
            release_keyboard(&mut core, &mut self.ui);
        }
        egui::CentralPanel::no_frame()
            .frame(egui::Frame::new().inner_margin(egui::Margin {
                left: 14,
                right: 14,
                top: 6,
                bottom: 10,
            }))
            .show(ui, |ui| {
                match core.cfg.tab {
                    0 => mixer(ui, &mut core, &mut self.ui),
                    1 => pads_page(ui, &mut core, &mut self.ui),
                    _ => efeitos_page(ui, &mut core, &mut self.ui),
                }
            });

        preset_picker(&ctx, &mut core, &mut self.ui);
        let mut restart = false;
        settings_window(
            &ctx,
            &mut core,
            &mut self.ui,
            &self.audio,
            &self.midi_status,
            &self.midi_rescan,
            &mut restart,
            &self.config_path,
        );
        rename_window(&ctx, &mut core, &mut self.ui);
        bt_window(&ctx, &mut self.ui, &self.midi_status, &self.bt);
        sons_window(&ctx, &core, &mut self.ui);
        // v0.15: diz à thread do MIDI o que do teclado é controle (Aprender ligado/desligado pela tela).
        core.sync_piano_hooks();
        drop(core);
        if let Some(path) = self.ui.load_sf2.take() {
            load_sf2_async(self.core.clone(), path);
        }
        if restart {
            self.restart_audio();
        }
        // Janelas de arquivo: só com o `core` destravado (o carregamento também usa ele).
        if std::mem::take(&mut self.ui.open_sf2_dialog) {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("SoundFont", &["sf2", "SF2"])
                .pick_file()
            {
                load_sf2_async(self.core.clone(), path.to_string_lossy().to_string());
            }
        }
        if std::mem::take(&mut self.ui.my_pad_dialog) {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter(
                    "Áudio",
                    &["mp3", "wav", "ogg", "flac", "MP3", "WAV", "OGG", "FLAC"],
                )
                .pick_file()
            {
                let mut c = self.core.lock().unwrap();
                c.add_my_pad(path.to_string_lossy().to_string(), None);
            }
        }
        if let Some((bank, key)) = self.ui.pad_file_dialog.take() {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter(
                    "Áudio",
                    &["wav", "mp3", "ogg", "flac", "WAV", "MP3", "OGG", "FLAC"],
                )
                .pick_file()
            {
                let p = path.to_string_lossy().to_string();
                let mut c = self.core.lock().unwrap();
                if pads::is_vivo(bank) {
                    let root = pads::key_midi(self.ui.vivo_root_key);
                    c.set_vivo_file(bank - pads::PAD_BANKS, Some((p, root)));
                } else {
                    c.set_pad_file(bank, key, Some(p));
                }
            }
        }
        // v0.19: som próprio num DrumPad / coral próprio no Voice FX
        if let Some(pad) = self.ui.drum_file_dialog.take() {
            if let Some(path) = rfd::FileDialog::new()
                .set_title("Escolha o som para este DrumPad (WAV, MP3, OGG ou FLAC)")
                .add_filter("Áudio", &["wav", "mp3", "ogg", "flac", "WAV", "MP3", "OGG", "FLAC"])
                .pick_file()
            {
                let mut c = self.core.lock().unwrap();
                c.set_drum_file(pad, Some(path.to_string_lossy().to_string()));
            }
        }
        if std::mem::take(&mut self.ui.voicefx_file_dialog) {
            if let Some(path) = rfd::FileDialog::new()
                .set_title("Escolha o coral (uma nota só, segurada: WAV, MP3, OGG ou FLAC)")
                .add_filter("Áudio", &["wav", "mp3", "ogg", "flac", "WAV", "MP3", "OGG", "FLAC"])
                .pick_file()
            {
                let mut c = self.core.lock().unwrap();
                c.set_voicefx_file(Some(path.to_string_lossy().to_string()));
            }
        }

        let busy = self
            .core
            .lock()
            .map(|c| c.loading.is_some())
            .unwrap_or(false);
        let fast = busy || self.ui.kb_open;
        let motion = self.core.lock().map(|c| c.cfg.look.motion).unwrap_or(false);
        // Fundo animado: ~25 quadros/s (leve). Sem animação: só quando algo muda.
        let ms = if motion {
            40
        } else if fast {
            60
        } else {
            250
        };
        ctx.request_repaint_after(Duration::from_millis(ms));
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        if let Ok(c) = self.core.lock() {
            let _ = c.cfg.save_to(&self.config_path);
        }
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        crate::theme::th().bottom.to_normalized_gamma_f32()
    }
}

fn setup_style(ctx: &egui::Context) {
    ctx.set_visuals(egui::Visuals::dark());
    ctx.global_style_mut(|s| {
        use egui::TextStyle::*;
        s.text_styles.insert(Body, FontId::proportional(16.0));
        s.text_styles.insert(Button, FontId::proportional(16.0));
        s.text_styles.insert(Small, FontId::proportional(13.0));
        s.text_styles.insert(Heading, FontId::proportional(22.0));
        s.text_styles.insert(Monospace, FontId::monospace(15.0));
        s.spacing.item_spacing = Vec2::new(8.0, 6.0);
        s.spacing.button_padding = Vec2::new(10.0, 6.0);

        s.visuals.widgets.inactive.corner_radius = CornerRadius::same(8);
        s.visuals.widgets.hovered.corner_radius = CornerRadius::same(8);
        s.visuals.widgets.active.corner_radius = CornerRadius::same(8);

        s.visuals.window_corner_radius = CornerRadius::same(12);
    });
}

// ---------- Teclado do computador ----------

fn keyboard(ctx: &egui::Context, core: &mut Core, st: &mut UiState) {
    // Esc fecha primeiro a janela aberta; só depois vira "pânico".
    if st.picker.is_some() || st.rename.is_some() {
        if ctx.input(|i| i.key_pressed(Key::Escape)) {
            st.picker = None;
            st.rename = None;
        }
        return;
    }
    if ctx.egui_wants_keyboard_input() {
        return;
    }
    // v0.18.2: as teclas do computador tocam em qualquer aba (antes só na Sons SF2)
    if st.kb_open {
        computer_keys(ctx, core, st);
    }
    ctx.input_mut(|i| {
        if i.consume_key(egui::Modifiers::NONE, Key::ArrowLeft) {
            core.step_scene(-1);
        }
        if i.consume_key(egui::Modifiers::NONE, Key::ArrowRight) {
            core.step_scene(1);
        }
        let fkeys = [
            Key::F1,
            Key::F2,
            Key::F3,
            Key::F4,
            Key::F5,
            Key::F6,
            Key::F7,
            Key::F8,
        ];
        for (n, k) in fkeys.iter().enumerate() {
            if i.consume_key(egui::Modifiers::NONE, *k) {
                core.load_scene(n);
            }
        }
        if i.consume_key(egui::Modifiers::NONE, Key::Escape) {
            if core.wizard.is_some() {
                core.wizard_cancel();
            } else if core.learn.active {
                core.learn.active = false;
                core.learn.target = None;
            } else {
                core.panic();
            }
        }
    });
}

// ---------- Teclado de piano na tela ----------

const KB_LOW: u8 = 36; // C2
const KB_HIGH: u8 = 96; // C7

fn is_black(n: u8) -> bool {
    matches!(n % 12, 1 | 3 | 6 | 8 | 10)
}

fn note_name(n: u8) -> String {
    format!(
        "{}{}",
        pads::KEY_NAMES[(n % 12) as usize],
        n as i32 / 12 - 1
    )
}

/// Letras do computador → semitom a partir do Dó (linha do meio = brancas, linha de cima = pretas).
const KEYMAP: [(Key, i32); 17] = [
    (Key::A, 0),
    (Key::W, 1),
    (Key::S, 2),
    (Key::E, 3),
    (Key::D, 4),
    (Key::F, 5),
    (Key::T, 6),
    (Key::G, 7),
    (Key::Y, 8),
    (Key::H, 9),
    (Key::U, 10),
    (Key::J, 11),
    (Key::K, 12),
    (Key::O, 13),
    (Key::L, 14),
    (Key::P, 15),
    (Key::Semicolon, 16),
];

fn computer_keys(ctx: &egui::Context, core: &mut Core, st: &mut UiState) {
    let events = ctx.input(|i| i.events.clone());
    for ev in events {
        let egui::Event::Key {
            key,
            pressed,
            repeat,
            modifiers,
            ..
        } = ev
        else {
            continue;
        };
        if repeat || modifiers.ctrl || modifiers.alt {
            continue;
        }
        if pressed && key == Key::Z {
            st.kb_octave = (st.kb_octave - 1).max(1);
            continue;
        }
        if pressed && key == Key::X {
            st.kb_octave = (st.kb_octave + 1).min(7);
            continue;
        }
        if key == Key::Space {
            st.kb_sustain = pressed;
            core.sustain(pressed);
            continue;
        }
        let Some(&(_, semi)) = KEYMAP.iter().find(|(k, _)| *k == key) else {
            continue;
        };
        if pressed {
            if st.kb_keys.iter().any(|(k, _)| *k == key) {
                continue;
            }
            let n = (12 * (st.kb_octave + 1) + semi).clamp(0, 127) as u8;
            st.kb_keys.push((key, n));
            core.note_on(n, 100);
        } else if let Some(pos) = st.kb_keys.iter().position(|(k, _)| *k == key) {
            let (_, n) = st.kb_keys.remove(pos);
            core.note_off(n);
        }
    }
}

/// Solta tudo que o teclado da tela estava segurando (ao fechar o teclado ou trocar de aba).
fn release_keyboard(core: &mut Core, st: &mut UiState) {
    if let Some(n) = st.kb_mouse.take() {
        core.note_off(n);
    }
    for (_, n) in std::mem::take(&mut st.kb_keys) {
        core.note_off(n);
    }
    if st.kb_sustain {
        st.kb_sustain = false;
        core.sustain(false);
    }
}

fn piano_keyboard(ui: &mut Ui, core: &mut Core, st: &mut UiState) {
    ui.horizontal(|ui| {
        ui.label(RichText::new("🎹 Teclado").strong());
        ui.label(
            RichText::new("PC: A W S E D F T G Y H U J K · Z / X oitava · espaço pedal")
                .size(13.0)
                .color(th().dim),
        );
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            // v0.18.2: fecha daqui também (o botão 🎹 só existe na aba Sons SF2)
            if ui
                .add(egui::Button::new(RichText::new("✖").size(14.0)).fill(th().glass_hi))
                .on_hover_text("Fechar o teclado")
                .clicked()
            {
                st.kb_open = false;
            }
            let ped = egui::Button::new(RichText::new("PEDAL").strong().color(if st.kb_sustain { th().ink } else { th().text }))
                .fill(if st.kb_sustain { YELLOW } else { th().glass_hi });
            if ui.add(ped).on_hover_text("Segura as notas (como o pedal do piano)").clicked() {
                st.kb_sustain = !st.kb_sustain;
                core.sustain(st.kb_sustain);
            }
            ui.label(RichText::new(format!("teclas do PC: C{}", st.kb_octave)).size(13.0).color(th().dim));
        });
    });
    ui.add_space(4.0);

    let h = 112.0;
    let (rect, resp) =
        ui.allocate_exact_size(Vec2::new(ui.available_width(), h), Sense::click_and_drag());
    let whites: Vec<u8> = (KB_LOW..=KB_HIGH).filter(|n| !is_black(*n)).collect();
    let ww = rect.width() / whites.len() as f32;
    let bw = ww * 0.62;
    let bh = h * 0.62;
    let white_x =
        |n: u8| whites.iter().position(|w| *w == n).unwrap_or(0) as f32 * ww + rect.left();
    let key_rect = |n: u8| -> Rect {
        if is_black(n) {
            let x = white_x(n - 1) + ww - bw / 2.0;
            Rect::from_min_size(Pos2::new(x, rect.top()), Vec2::new(bw, bh))
        } else {
            Rect::from_min_size(Pos2::new(white_x(n), rect.top()), Vec2::new(ww, h))
        }
    };
    let hit = |p: Pos2| -> Option<u8> {
        if !rect.contains(p) {
            return None;
        }
        (KB_LOW..=KB_HIGH)
            .filter(|n| is_black(*n))
            .find(|n| key_rect(*n).contains(p))
            .or_else(|| whites.iter().copied().find(|n| key_rect(*n).contains(p)))
    };

    // Mouse: segurar toca; arrastar faz glissando; soltar para.
    let pointer = ui.input(|i| i.pointer.interact_pos());
    let down = resp.is_pointer_button_down_on() || (resp.dragged() && st.kb_mouse.is_some());
    let want = if down { pointer.and_then(hit) } else { None };
    if want != st.kb_mouse {
        if let Some(old) = st.kb_mouse.take() {
            core.note_off(old);
        }
        if let (Some(n), Some(p)) = (want, pointer) {
            let kr = key_rect(n);
            let depth = ((p.y - kr.top()) / kr.height()).clamp(0.0, 1.0);
            let vel = (45.0 + depth * 82.0) as u8;
            core.note_on(n, vel);
            st.kb_mouse = Some(n);
        }
    }

    let p = ui.painter_at(rect);
    let lit = |n: u8| {
        crate::core::note_is_held(n)
            || st.kb_mouse == Some(n)
            || st.kb_keys.iter().any(|(_, k)| *k == n)
    };
    for &n in &whites {
        let r = key_rect(n).shrink2(Vec2::new(0.5, 0.0));
        let on = lit(n);
        p.rect_filled(
            r,
            CornerRadius {
                nw: 0,
                ne: 0,
                sw: 5,
                se: 5,
            },
            if on {
                LAYER_COLORS[0]
            } else {
                Color32::from_rgb(0xec, 0xee, 0xf1)
            },
        );
        p.rect_stroke(
            r,
            CornerRadius {
                nw: 0,
                ne: 0,
                sw: 5,
                se: 5,
            },
            Stroke::new(1.0, Color32::from_rgb(0x55, 0x5b, 0x66)),
            StrokeKind::Inside,
        );
        if n % 12 == 0 {
            p.text(
                Pos2::new(r.center().x, r.bottom() - 10.0),
                Align2::CENTER_CENTER,
                note_name(n),
                FontId::proportional(11.0),
                if on {
                    th().ink
                } else {
                    Color32::from_rgb(0x70, 0x76, 0x80)
                },
            );
        }
    }
    for n in (KB_LOW..=KB_HIGH).filter(|n| is_black(*n)) {
        let r = key_rect(n);
        let on = lit(n);
        p.rect_filled(
            r,
            CornerRadius {
                nw: 0,
                ne: 0,
                sw: 4,
                se: 4,
            },
            if on {
                LAYER_COLORS[0].gamma_multiply(0.85)
            } else {
                Color32::from_rgb(0x16, 0x18, 0x1d)
            },
        );
    }
    // Pad tocando: bolinhas nas notas da escala (a tônica em destaque).
    if let Some(key) = core
        .pad_active
        .filter(|_| !core.cfg.is_hidden("scale_help"))
    {
        let minor = core.cfg.scale_minor;
        let tonic = pads::key_midi(key) % 12;
        for n in KB_LOW..=KB_HIGH {
            if crate::teoria::in_scale(key, minor, n % 12) {
                let r = key_rect(n);
                let c = Pos2::new(
                    r.center().x,
                    r.bottom() - if is_black(n) { 9.0 } else { 22.0 },
                );
                p.circle_filled(
                    c,
                    if n % 12 == tonic { 4.5 } else { 3.0 },
                    if n % 12 == tonic { ORANGE } else { PAD_COLOR },
                );
            }
        }
    }
    // Marca onde está a oitava das teclas do computador.
    let base = (12 * (st.kb_octave + 1)).clamp(0, 127) as u8;
    if (KB_LOW..=KB_HIGH).contains(&base) {
        let r = key_rect(base);
        p.rect_filled(
            Rect::from_min_size(
                Pos2::new(r.left() + 3.0, r.bottom() - 4.0),
                Vec2::new(r.width() - 6.0, 3.0),
            ),
            1,
            ORANGE,
        );
    }
}

// ---------- Barra de cima ----------

#[allow(clippy::too_many_arguments)]
fn top_bar(
    ui: &mut Ui,
    core: &mut Core,
    audio: &Audio,
    stats: &Stats,
    midi_status: &SharedMidiStatus,
    st: &mut UiState,
) {
    ui.horizontal(|ui| {
        // v0.22: o nome agora é NoahPad — "Noah" forte e "Pad" na cor dos pads, grudados, como um logo
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            ui.label(RichText::new("Noah").size(20.0).strong().color(th().text));
            ui.label(RichText::new("Pad").size(20.0).strong().color(PAD_COLOR));
        })
        .response
        .on_hover_text("NoahPad — o pad do Noah");
        ui.add_space(10.0);

        // Arquivo SF2
        if let Some(l) = &core.loading {
            let frac = if l.total > 0 {
                l.read.load(Ordering::Relaxed) as f32 / l.total as f32
            } else {
                0.0
            };
            ui.add(
                egui::ProgressBar::new(frac.min(1.0))
                    .desired_width(260.0)
                    .text(format!("Carregando {} · {:.0}%", l.file, frac * 100.0)),
            );
        } else {
            let full = core
                .sf2_name
                .clone()
                .unwrap_or_else(|| "Abrir SF2...".into());
            // Nome comprido não empurra o resto da barra (o nome inteiro aparece ao passar o mouse).
            let label = if full.chars().count() > 20 {
                format!("{}…", full.chars().take(18).collect::<String>())
            } else {
                full.clone()
            };
            let btn = egui::Button::new(RichText::new(format!("📂 {label}")).size(15.0)).fill(
                if core.sf2_name.is_some() {
                    th().glass_hi
                } else {
                    ORANGE.gamma_multiply(0.6)
                },
            );
            if ui
                .add(btn)
                .on_hover_text(format!("{full}\nClique para escolher outro arquivo .sf2"))
                .clicked()
            {
                // Não abre aqui: o `core` está travado por esta tela e o carregamento também precisa dele.
                st.open_sf2_dialog = true;
            }
        }
        if let Some(e) = &core.load_error {
            ui.label(RichText::new(format!("⚠ {e}")).color(RED));
        }

        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            // v0.18: tudo que não é de tocar fica no menu ⚙ (a barra mostra só o essencial).
            ui.menu_button(RichText::new("⚙").size(18.0), |ui| {
                ui.set_min_width(220.0);
                if ui.button("⚙  Ajustes...").clicked() {
                    st.settings = true;
                    st.audio_devices = audio::device_names();
                    ui.close();
                }
                if ui.button("✏  Editar a tela (tirar / pôr coisas)").clicked() {
                    st.editing = true;
                    ui.close();
                }
                ui.separator();
                if ui.button("Configurar mesinha (automático)").clicked() {
                    core.wizard_start();
                    ui.close();
                }
                if ui.button("Bluetooth (mesinha sem fio)").clicked() {
                    st.bt_open = true;
                    ui.close();
                }
                if ui.button("⬇  Sons grátis").clicked() {
                    st.sons_open = true;
                    ui.close();
                }
            });
            if st.editing {
                let done = egui::Button::new(RichText::new("✔ Concluir").color(th().ink))
                    .fill(Color32::from_rgb(0x0a, 0x84, 0xff));
                if ui.add(done).clicked() {
                    st.editing = false;
                }
            }
            let learn_txt = if core.learn.active {
                "Aprendendo..."
            } else {
                "Aprender"
            };
            let learn_btn =
                egui::Button::new(RichText::new(learn_txt).color(if core.learn.active {
                    th().ink
                } else {
                    th().text
                }))
                .fill(if core.learn.active {
                    ORANGE
                } else {
                    th().glass_hi
                });
            if !core.cfg.is_hidden("learn_button")
                && ui
                    .add(learn_btn)
                    .on_hover_text("Ligar um controle da mesinha a um controle da tela")
                    .clicked()
            {
                core.learn.active = !core.learn.active;
                core.learn.target = None;
            }
            ui.add_space(14.0);

            // Pouco espaço (tela pequena): mostra primeiro Mesinha/Piano, depois o áudio, e a CPU só se couber.
            // Aparelhos MIDI
            let ports = midi_status
                .lock()
                .map(|s| s.all_ports())
                .unwrap_or_default();
            let ctl = ports
                .iter()
                .find(|p| p.role == PortRole::Controller && p.connected);
            let piano = ports
                .iter()
                .find(|p| p.role == PortRole::Piano && p.connected);
            let blink = |t: Option<Instant>| {
                t.map(|t| t.elapsed() < Duration::from_millis(150))
                    .unwrap_or(false)
            };
            let r = status_dot(
                ui,
                "Mesinha",
                ctl.map(|p| p.name.as_str()),
                blink(core.controller_activity),
            );
            if r.clicked() {
                st.bt_open = true;
            }
            status_dot(
                ui,
                "Piano",
                piano.map(|p| p.name.as_str()),
                blink(core.piano_activity),
            );

            // Áudio: só aparece quando tem problema (os detalhes ficam em ⚙ Ajustes).
            match &audio.info {
                Some(_) => {}
                None => {
                    let err = audio.error.clone().unwrap_or_else(|| "sem saída".into());
                    let txt = if ui.available_width() > 110.0 {
                        "🔇 Sem áudio"
                    } else {
                        "🔇"
                    };
                    ui.label(RichText::new(txt).color(RED)).on_hover_text(err);
                }
            }

            // CPU do áudio
            let cpu = stats.cpu_permille.load(Ordering::Relaxed) as f32 / 10.0;
            let cpu_color = if cpu < 60.0 {
                GREEN
            } else if cpu < 85.0 {
                YELLOW
            } else {
                RED
            };
            // CPU: só aparece quando o computador está apertado.
            if cpu >= 60.0 && ui.available_width() > 80.0 {
                ui.label(
                    RichText::new(format!("CPU {cpu:.0}%"))
                        .color(cpu_color)
                        .monospace(),
                );
            }
        });
    });
    if st.editing {
        edit_banner(ui, core, st);
    }
    if let Some(w) = &core.wizard {
        let (step, total) = (w.step + 1, w.steps.len());
        let (target, control) = w.current().unwrap_or((Target::Master, ""));
        let mut skip = false;
        let mut cancel = false;
        ui.add_space(4.0);
        egui::Frame::new()
            .fill(PAD_COLOR)
            .corner_radius(6)
            .inner_margin(egui::Margin::symmetric(10, 4))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(format!(
                            "CONFIGURAR MESINHA — passo {step} de {total}:  mexa o {control} da mesinha  ({})",
                            target.label()
                        ))
                        .color(th().ink)
                        .strong(),
                    );
                    if ui.button("Pular").on_hover_text("Esse controle fica como está").clicked() {
                        skip = true;
                    }
                    if ui.button("Cancelar").clicked() {
                        cancel = true;
                    }
                });
            });
        if skip {
            core.wizard_skip();
        }
        if cancel {
            core.wizard_cancel();
        }
    } else if core.learn.active {
        let msg = match core.learn.target {
            None => "MODO APRENDER — 1) clique num controle da tela   2) mexa o controle da mesinha ou do teclado.   (Esc sai)".to_string(),
            Some(t) => format!("Agora mexa na mesinha ou no teclado o controle para: {}", t.label()),
        };
        let mut last = core
            .last_signal
            .clone()
            .map(|l| format!("   ·   último sinal da mesinha: {l}"))
            .unwrap_or_default();
        if let Some(l) = &core.last_piano_signal {
            last.push_str(&format!("   ·   do teclado: {l}"));
        }
        ui.add_space(4.0);
        egui::Frame::new()
            .fill(ORANGE)
            .corner_radius(6)
            .inner_margin(egui::Margin::symmetric(10, 4))
            .show(ui, |ui| {
                ui.label(
                    RichText::new(format!("{msg}{last}"))
                        .color(th().ink)
                        .strong(),
                );
            });
    }
}

fn status_dot(ui: &mut Ui, what: &str, name: Option<&str>, active: bool) -> egui::Response {
    let color = match (name.is_some(), active) {
        (true, true) => Color32::WHITE,
        (true, false) => GREEN,
        _ => RED,
    };
    let r = ui.add(egui::Label::new(RichText::new(what).color(color)).sense(Sense::click()));
    let c = Pos2::new(r.rect.left() - 8.0, r.rect.center().y);
    ui.painter().circle_filled(c, 5.0, color);
    ui.add_space(20.0);
    r.on_hover_text(name.map(|n| format!("Conectado: {n}")).unwrap_or_else(|| {
        format!(
            "{what} não encontrado. Ligue pelo cabo USB (ou clique para procurar no Bluetooth)."
        )
    }))
}

// ---------- Abas ----------

fn tabs_bar(ui: &mut Ui, core: &mut Core) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 4.0;
        let pad_tag = core
            .pad_active
            .map(|k| format!("  ·  {}", pads::KEY_NAMES[k]))
            .unwrap_or_default();
        let tabs = [
            (
                "🎹  Sons SF2".to_string(),
                core.sf2_name.is_some() && core.cfg.mix.layers.iter().any(|l| l.on),
            ),
            (format!("🎶  Pads{pad_tag}"), core.pad_active.is_some()),
            (
                "✨  Efeitos".to_string(),
                core.pulse_on || core.arp_on || core.voicefx_on,
            ),
        ];
        for (i, (label, live)) in tabs.iter().enumerate() {
            if i == 1 && core.cfg.is_hidden("tab_pads") {
                continue;
            }
            let active = core.cfg.tab == i;
            let (rect, resp) = ui.allocate_exact_size(Vec2::new(210.0, 38.0), Sense::click());
            let radius = CornerRadius {
                nw: 10,
                ne: 10,
                sw: 0,
                se: 0,
            };
            let fill = if active {
                th().glass_hi
            } else if resp.hovered() {
                th().glass_hi.gamma_multiply(0.7)
            } else {
                th().glass
            };
            let p = ui.painter();
            p.rect_filled(rect, radius, fill);
            if active {
                p.rect_filled(
                    Rect::from_min_size(rect.min, Vec2::new(rect.width(), 3.0)),
                    CornerRadius {
                        nw: 10,
                        ne: 10,
                        sw: 0,
                        se: 0,
                    },
                    match i {
                        0 => LAYER_COLORS[0],
                        1 => PAD_COLOR,
                        _ => MOD_COLOR,
                    },
                );
            }
            p.text(
                rect.center() + Vec2::new(if *live { 6.0 } else { 0.0 }, 1.0),
                Align2::CENTER_CENTER,
                label,
                FontId::proportional(16.0),
                if active { th().text } else { th().dim },
            );
            if *live {
                p.circle_filled(
                    Pos2::new(rect.left() + 16.0, rect.center().y + 1.0),
                    4.5,
                    GREEN,
                );
            }
            if resp.clicked() {
                core.cfg.tab = i;
                core.dirty = true;
            }
        }
    });
}

// ---------- v0.17: aba Efeitos (v0.18: limpa, com ⚙ em cada painel) ----------

const ARP_COLOR: Color32 = Color32::from_rgb(0x30, 0xd1, 0x58);
const VOICE_COLOR: Color32 = Color32::from_rgb(0xff, 0x6b, 0x9d);
const DRUM_COLOR: Color32 = Color32::from_rgb(0xff, 0x45, 0x3a);

/// Painel de vidro com título (e, se `details` vier, um ⚙ no canto que abre os ajustes finos).
fn fx_card(
    ui: &mut Ui,
    size: Vec2,
    title: &str,
    color: Color32,
    details: Option<(&mut bool, &str)>,
    add: impl FnOnce(&mut Ui),
) {
    let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
    theme::glass(ui.painter(), rect, 18, false, None);
    ui.painter().rect_filled(
        Rect::from_min_size(rect.min + Vec2::new(16.0, 0.0), Vec2::new(rect.width() - 32.0, 3.0)),
        2,
        color,
    );
    let inner = rect.shrink2(Vec2::new(14.0, 10.0));
    let mut child = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(inner)
            .layout(Layout::top_down(Align::Min)),
    );
    child.horizontal(|ui| {
        ui.label(RichText::new(title).strong().size(17.0));
        if let Some((open, hint)) = details {
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if details_button(ui, *open, Vec2::new(34.0, 28.0), hint).clicked() {
                    *open = !*open;
                }
            });
        }
    });
    child.add_space(6.0);
    // v0.18.1: se a janela for pequena e o conteúdo não couber, o painel rola (nada vaza para fora)
    egui::ScrollArea::vertical()
        .id_salt(("fx_card", title))
        .auto_shrink([false, false])
        .show(&mut child, |ui| add(ui));
}

/// Botão grande de liga/desliga (pode ser ligado na mesinha/teclado pelo Aprender).
fn fx_toggle(ui: &mut Ui, core: &mut Core, w: f32, on: bool, label: &str, color: Color32, target: Target, hint: &str) -> bool {
    // v0.18.1: desenhado aqui (a fonte não tem "●" e mostrava um quadradinho)
    let (rect, b) = ui.allocate_exact_size(Vec2::new(w, 52.0), Sense::click());
    let p = ui.painter();
    let fill = if on {
        color
    } else if b.hovered() {
        th().glass_hi.gamma_multiply(1.25)
    } else {
        th().glass_hi
    };
    p.rect_filled(rect, 10, fill);
    let fg = if on { th().ink } else { th().text };
    let galley = p.layout_no_wrap(label.to_string(), FontId::proportional(17.0), fg);
    let dot_r = 6.0;
    let total = dot_r * 2.0 + 10.0 + galley.size().x;
    let x0 = rect.center().x - total / 2.0;
    let dot = Pos2::new(x0 + dot_r, rect.center().y);
    if on {
        p.circle_filled(dot, dot_r, fg);
    } else {
        p.circle_stroke(dot, dot_r, Stroke::new(2.0, fg));
    }
    p.galley(
        Pos2::new(x0 + dot_r * 2.0 + 10.0, rect.center().y - galley.size().y / 2.0),
        galley,
        fg,
    );
    let b = b.on_hover_text(hint);
    !learnable(ui, core, &b, target) && b.clicked()
}

/// Fileira de opções (uma marcada). Devolve a escolhida se mudou.
fn fx_choice<T: Copy + PartialEq>(ui: &mut Ui, label: &str, opts: &[(&str, T)], cur: T, color: Color32) -> Option<T> {
    let mut out = None;
    ui.label(RichText::new(label).size(12.0).color(th().dim));
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing.x = 4.0;
        for (name, v) in opts {
            let sel = *v == cur;
            let b = egui::Button::new(RichText::new(*name).size(13.0).color(if sel { th().ink } else { th().text }))
                .fill(if sel { color } else { th().glass_hi });
            if ui.add(b).clicked() && !sel {
                out = Some(*v);
            }
        }
    });
    out
}

/// Barra deitada de 0..1 que se arrasta (volume, profundidade).
fn fx_bar(ui: &mut Ui, core: &mut Core, w: f32, value: &mut f32, color: Color32, target: Option<Target>) {
    let r = hbar(ui, Vec2::new(w, 28.0), *value, false, color);
    let learning = target.is_some_and(|t| learnable(ui, core, &r, t));
    if !learning && r.is_pointer_button_down_on() {
        if let Some(p) = r.interact_pointer_pos() {
            *value = ((p.x - r.rect.left()) / r.rect.width()).clamp(0.0, 1.0);
        }
    }
}

fn efeitos_page(ui: &mut Ui, core: &mut Core, st: &mut UiState) {
    core.ensure_drum_kit();
    let avail = ui.available_size();
    let gap = 12.0;
    let top_h = (avail.y * 0.56).clamp(240.0, 380.0);
    let drum_h = (avail.y - top_h - gap).max(150.0);
    let card_w = ((avail.x - 4.0 * gap) / 5.0).max(150.0);
    let mut e = core.cfg.efeitos.clone();
    let learning = core.learn.active;
    let [mut d_tempo, mut d_pulse, mut d_arp, mut d_voice, mut d_filt] = st.fx_details;
    let [o_tempo, o_pulse, o_arp, o_voice, o_filt] = st.fx_details;
    // pedidos que rodam no fim (ver depois de `set_efeitos`)
    let mut drum_reset: Option<usize> = None;
    let mut drum_dialog: Option<usize> = None;
    let mut voice_reset = false;
    let mut voice_key: Option<usize> = None;
    let mut st_voice_dialog = false;
    let mut voice_sel: Option<usize> = None;

    ui.horizontal_top(|ui| {
        ui.spacing_mut().item_spacing.x = gap;
        // ---- Tempo ----
        fx_card(ui, Vec2::new(card_w, top_h), "⏱ Tempo", th().text, Some((&mut d_tempo, "Acertar o BPM com − e +")), |ui| {
            let w = ui.available_width();
            ui.vertical_centered(|ui| {
                ui.label(RichText::new(format!("{:.0}", e.bpm)).size(46.0).strong());
                ui.label(RichText::new("BPM").size(12.0).color(th().dim));
            });
            if o_tempo {
                ui.horizontal(|ui| {
                    let bw = (w - 3.0 * ui.spacing().item_spacing.x) / 4.0;
                    for (t, d) in [("−5", -5.0), ("−1", -1.0), ("+1", 1.0), ("+5", 5.0)] {
                        if ui.add_sized([bw, 30.0], egui::Button::new(t).fill(th().glass_hi)).clicked() {
                            e.bpm += d;
                        }
                    }
                });
            }
            ui.add_space(6.0);
            let tap = ui.add_sized(
                [w, (ui.available_height() - 4.0).clamp(50.0, 120.0)],
                egui::Button::new(RichText::new("TAP TEMPO").strong().size(20.0)).fill(th().glass_hi),
            );
            let tap = tap.on_hover_text("Bata 2 a 4 vezes no tempo da música: o PulsePad e o Arpejador acompanham");
            if !learnable(ui, core, &tap, Target::TapTempo) && tap.clicked() {
                core.tap_tempo();
                e.bpm = core.cfg.efeitos.bpm;
            }
        });

        // ---- PulsePad ----
        fx_card(ui, Vec2::new(card_w, top_h), "💓 PulsePad", PAD_COLOR, Some((&mut d_pulse, "Pulsos por tempo e profundidade")), |ui| {
            let w = ui.available_width();
            if fx_toggle(ui, core, w, core.pulse_on, "PULSE", PAD_COLOR, Target::PulseToggle, "Os pads e o ❄ Freeze pulsam no tempo") {
                core.toggle_pulse();
            }
            if o_pulse || learning {
                ui.add_space(6.0);
                if let Some(v) = fx_choice(ui, "Rate (pulsos por tempo)", &[("1/4", 1u32), ("1/8", 2), ("1/16", 4)], e.pulse_per_beat, PAD_COLOR) {
                    e.pulse_per_beat = v;
                }
                ui.add_space(4.0);
                ui.label(RichText::new(format!("Depth {:.0}%", e.pulse_depth * 100.0)).size(12.0).color(th().dim));
                fx_bar(ui, core, w, &mut e.pulse_depth, PAD_COLOR, None);
            }
        });

        // ---- Arpejador ----
        fx_card(ui, Vec2::new(card_w, top_h), "🎼 Arpeggiator", ARP_COLOR, Some((&mut d_arp, "Layer, padrão (Up/Down), Rate, Octaves e Latch")), |ui| {
            let w = ui.available_width();
            if fx_toggle(ui, core, w, core.arp_on, "ARP", ARP_COLOR, Target::ArpToggle, "Segure o acorde: a camada escolhida toca as notas uma a uma") {
                core.toggle_arp();
            }
            let layer = e.arp_layer;
            let pname = core.cfg.mix.layers[layer].preset.as_ref().map(|p| p.name.clone()).unwrap_or_else(|| "sem timbre".into());
            let live = core.cfg.mix.audible(layer);
            ui.label(
                RichText::new(format!("Camada {} · {pname}{}", layer + 1, if live { "" } else { " (desligada)" }))
                    .size(13.0)
                    .color(if live { th().dim } else { ORANGE }),
            );
            if o_arp || learning {
                // v0.18.1: 4 listas compactas (antes eram fileiras de botões que vazavam do painel)
                ui.add_space(6.0);
                let names: Vec<String> = (0..LAYERS)
                    .map(|i| {
                        let p = core.cfg.mix.layers[i].preset.as_ref().map(|p| p.name.as_str()).unwrap_or("sem timbre");
                        format!("{} · {p}", i + 1)
                    })
                    .collect();
                let speeds: [(&str, u32); 3] = [("1/8", 2), ("1/8T (tercina)", 3), ("1/16", 4)];
                let combo_w = (w - 88.0).max(80.0);
                egui::Grid::new("arp_grid").num_columns(2).spacing([8.0, 6.0]).show(ui, |ui| {
                    ui.label(RichText::new("Layer").size(13.0).color(th().dim)).on_hover_text("Camada que arpeja");
                    egui::ComboBox::from_id_salt("arp_layer")
                        .width(combo_w)
                        .selected_text(format!("{}", e.arp_layer + 1))
                        .show_ui(ui, |ui| {
                            for (i, n) in names.iter().enumerate() {
                                ui.selectable_value(&mut e.arp_layer, i, n);
                            }
                        });
                    ui.end_row();
                    ui.label(RichText::new("Pattern").size(13.0).color(th().dim)).on_hover_text("Up = sobe, Down = desce, Up/Down = sobe e desce, Random = aleatório");
                    egui::ComboBox::from_id_salt("arp_pattern")
                        .width(combo_w)
                        .selected_text(crate::efeitos::ARP_PATTERNS[e.arp_pattern as usize % 4])
                        .show_ui(ui, |ui| {
                            for (i, n) in crate::efeitos::ARP_PATTERNS.iter().enumerate() {
                                ui.selectable_value(&mut e.arp_pattern, i as u8, *n);
                            }
                        });
                    ui.end_row();
                    ui.label(RichText::new("Rate").size(13.0).color(th().dim)).on_hover_text("Velocidade das notas");
                    let cur = speeds.iter().find(|(_, v)| *v == e.arp_per_beat).map(|(n, _)| *n).unwrap_or("1/8");
                    egui::ComboBox::from_id_salt("arp_speed")
                        .width(combo_w)
                        .selected_text(cur)
                        .show_ui(ui, |ui| {
                            for (n, v) in speeds {
                                ui.selectable_value(&mut e.arp_per_beat, v, n);
                            }
                        });
                    ui.end_row();
                    ui.label(RichText::new("Octaves").size(13.0).color(th().dim)).on_hover_text("Quantas oitavas o arpejo sobe");
                    egui::ComboBox::from_id_salt("arp_oct")
                        .width(combo_w)
                        .selected_text(format!("{}", e.arp_octaves))
                        .show_ui(ui, |ui| {
                            for o in 1..=3u8 {
                                ui.selectable_value(&mut e.arp_octaves, o, format!("{o}"));
                            }
                        });
                    ui.end_row();
                });
                ui.add_space(2.0);
                ui.checkbox(&mut e.arp_latch, "Latch").on_hover_text("Continua arpejando depois de soltar as teclas, até o próximo acorde");
            }
        });

        // ---- Voice FX ----
        fx_card(ui, Vec2::new(card_w, top_h), "🎤 Voice FX", VOICE_COLOR, Some((&mut d_voice, "Oitava do coral")), |ui| {
            let w = ui.available_width();
            if fx_toggle(ui, core, w, core.voicefx_on, "VOICE", VOICE_COLOR, Target::VoiceFxToggle, "Um coral (\"aah\") acompanha cada nota que você tocar") {
                core.toggle_voicefx();
            }
            // v0.21: escolher a voz que canta (troca na hora, mesmo ligado)
            ui.add_space(6.0);
            let mut voices: Vec<(&str, usize)> = pads::VOICE_NAMES.iter().enumerate().map(|(i, n)| (*n, i)).collect();
            if e.voicefx_file.is_some() {
                voices.push(("Minha", VOICE_MINE));
            }
            if let Some(v) = fx_choice(ui, "Voz", &voices, e.voicefx_voice, VOICE_COLOR) {
                voice_sel = Some(v);
            }
            ui.add_space(6.0);
            ui.label(RichText::new(format!("Volume {:.0}%", e.voicefx_volume * 100.0)).size(12.0).color(th().dim));
            fx_bar(ui, core, w, &mut e.voicefx_volume, VOICE_COLOR, Some(Target::VoiceFxVolume));
            if o_voice || learning {
                ui.add_space(6.0);
                if let Some(v) = fx_choice(ui, "Oitava do coral", &[("grave", -1i8), ("normal", 0), ("agudo", 1)], e.voicefx_octave, VOICE_COLOR) {
                    e.voicefx_octave = v;
                }
                // v0.19: coral próprio
                ui.add_space(6.0);
                match e.voicefx_file.clone() {
                    Some((path, root)) => {
                        ui.label(RichText::new(format!("Coral: ♪ {}", pads::name_from_file(&path))).size(13.0).color(VOICE_COLOR));
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Está em").size(12.0).color(th().dim));
                            let mut key = (0..pads::PAD_KEYS).find(|k| pads::key_midi(*k) == root).unwrap_or(0);
                            let before = key;
                            egui::ComboBox::from_id_salt("voicefx_key")
                                .width(56.0)
                                .selected_text(pads::KEY_NAMES[key])
                                .show_ui(ui, |ui| {
                                    for k in 0..pads::PAD_KEYS {
                                        ui.selectable_value(&mut key, k, pads::KEY_NAMES[k]);
                                    }
                                });
                            if key != before {
                                voice_key = Some(key);
                            }
                        });
                        if ui.button("Voltar ao coral do programa").clicked() {
                            voice_reset = true;
                        }
                    }
                    None => {
                        ui.label(RichText::new("Coral: do programa").size(13.0).color(th().dim));
                    }
                }
                if ui
                    .button("Usar meu coral (WAV/MP3)...")
                    .on_hover_text("Uma nota só, segurada, sem bateria (ex.: \"choir aah\" do Freesound). O programa descobre a nota sozinho.")
                    .clicked()
                {
                    st_voice_dialog = true;
                }
            }
        });

        // ---- v0.23: Abrir / fechar (filtro) + Swell ----
        fx_card(ui, Vec2::new(card_w, top_h), "Filter Sweep", PITCH_COLOR, Some((&mut d_filt, "Resonance, tempo do swell e onde vale")), |ui| {
            let w = ui.available_width();
            ui.horizontal(|ui| {
                ui.label(RichText::new("Cutoff").size(13.0).strong()).on_hover_text("O filtro: fechado = escuro e abafado; aberto = brilhante");
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let t = if e.filtro >= 0.995 { "aberto".to_string() } else if e.filtro <= 0.005 { "fechado".to_string() } else { format!("{:.0}%", e.filtro * 100.0) };
                    ui.label(RichText::new(t).size(12.0).color(th().dim));
                });
            });
            let r = hbar(ui, Vec2::new(w, 34.0), e.filtro, false, PITCH_COLOR);
            let r = r.on_hover_text("Esquerda = som escuro, abafado (de longe) · direita = aberto, brilhante. Ligue num knob da mesinha pelo Aprender.");
            if !learnable(ui, core, &r, Target::Filter) && r.is_pointer_button_down_on() {
                if let Some(p) = r.interact_pointer_pos() {
                    e.filtro = ((p.x - r.rect.left()) / r.rect.width()).clamp(0.0, 1.0);
                }
            }
            ui.horizontal(|ui| {
                ui.label(RichText::new("fechado").size(11.0).color(th().dim));
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.label(RichText::new("aberto").size(11.0).color(th().dim));
                });
            });
            ui.add_space(8.0);
            let label = if core.swell_in { "FADE OUT" } else { "SWELL IN" };
            if fx_toggle(ui, core, w, core.swell_in, label, PITCH_COLOR, Target::SwellToggle, "Swell: o som sai devagar (fechando o filtro) ou entra devagar (abrindo)") {
                core.toggle_swell();
            }
            ui.label(
                RichText::new(if core.swell_in { "som cheio" } else { "som fora — aperte para entrar" })
                    .size(12.0)
                    .color(if core.swell_in { th().dim } else { ORANGE }),
            );
            if o_filt || learning {
                ui.add_space(6.0);
                ui.label(RichText::new(format!("Resonance {:.0}%", e.filtro_res * 100.0)).size(12.0).color(th().dim));
                fx_bar(ui, core, w, &mut e.filtro_res, PITCH_COLOR, None);
                let secs: [(&str, u32); 4] = [("2 s", 2), ("4 s", 4), ("8 s", 8), ("16 s", 16)];
                let cur = e.swell_secs.round() as u32;
                if let Some(v) = fx_choice(ui, "Tempo do swell", &secs, cur, PITCH_COLOR) {
                    e.swell_secs = v as f32;
                }
                if let Some(v) = fx_choice(ui, "Vale para", &[("o fundo", false), ("tudo", true)], e.filtro_tudo, PITCH_COLOR) {
                    e.filtro_tudo = v;
                }
                ui.label(RichText::new("fundo = pads e ❄ congelado · tudo = menos a bateria").size(11.0).color(th().dim));
                ui.checkbox(&mut e.swell_filtro, "Swell abre e fecha o filtro junto");
            }
        });
    });
    st.fx_details = [d_tempo, d_pulse, d_arp, d_voice, d_filt];

    ui.add_space(gap);
    // ---- DrumPads (v0.25: visual novo — vidro escuro, neon por pad, brilho que acende e apaga) ----
    fx_card(ui, Vec2::new(avail.x, drum_h), "DrumPads", DRUM_COLOR, None, |ui| {
        let w = ui.available_width();
        let vol_w = 170.0;
        let gap = 10.0;
        let n = crate::efeitos::DRUMS;
        // grade que deixa os pads maiores: 8 numa fileira ou 4×2
        // margem de 10 px em volta: o brilho neon passa um pouco da borda do pad e não pode ser cortado
        let room_w = w - vol_w - 18.0 - 20.0;
        let room_h = ui.available_height() - 4.0 - 20.0;
        let (cols, rows, pad) = [(n, 1usize), (n / 2, 2)]
            .iter()
            .map(|&(c, r)| {
                let s = ((room_w - (c as f32 - 1.0) * gap) / c as f32).min((room_h - (r as f32 - 1.0) * gap) / r as f32);
                (c, r, s)
            })
            .fold((n, 1, 0.0f32), |best, x| if x.2 > best.2 { x } else { best });
        let pad = pad.max(56.0);
        let now = Instant::now();
        let mut glowing = false;
        egui::Frame::new().inner_margin(10).show(ui, |ui| {
        ui.horizontal_top(|ui| {
            ui.spacing_mut().item_spacing = Vec2::splat(gap);
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing = Vec2::splat(gap);
                for row in 0..rows {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = gap;
                        for col in 0..cols {
                            let i = row * cols + col;
                            if i >= n {
                                break;
                            }
                            // brilho do toque: acende na hora e apaga em ~0,45 s
                            let glow = core
                                .drum_flash[i]
                                .map(|t| 1.0 - (now.duration_since(t).as_secs_f32() / 0.45).min(1.0))
                                .unwrap_or(0.0)
                                .powf(1.6);
                            glowing |= glow > 0.0;
                            let (rect, resp) = ui.allocate_exact_size(Vec2::splat(pad), Sense::click());
                            let own = core.drum_file_name(i);
                            let pressed = resp.is_pointer_button_down_on();
                            // v0.25.1: passou o mouse = acende suave (RGB neon); bateu = acende forte e apaga
                            let hover = ui.ctx().animate_bool_with_time(resp.id.with("luz"), resp.hovered(), 0.18);
                            let light = (0.75 * hover).max(glow);
                            glowing |= hover > 0.0 && hover < 1.0;
                            drum_tile(
                                ui.painter(),
                                if pressed { rect.shrink(2.0) } else { rect },
                                DRUM_ACCENTS[i % DRUM_ACCENTS.len()],
                                light,
                                resp.hover_pos(),
                                i,
                                own.as_deref(),
                            );
                            let resp = resp.on_hover_text(match &own {
                                Some(nm) => format!("{} · seu som: {nm}\nBotão direito: trocar ou voltar ao som do programa", crate::efeitos::DRUM_NAMES[i]),
                                None => format!("{}\nBotão direito: usar um som seu (WAV/MP3)", crate::efeitos::DRUM_NAMES[i]),
                            });
                            if !learnable(ui, core, &resp, Target::Drum(i)) && resp.clicked() {
                                core.drum_hit(i, 110);
                            }
                            resp.context_menu(|ui| {
                                if ui.button("♪ Usar meu som (WAV/MP3)...").clicked() {
                                    drum_dialog = Some(i);
                                    ui.close();
                                }
                                if own.is_some() && ui.button("Voltar ao som do programa").clicked() {
                                    drum_reset = Some(i);
                                    ui.close();
                                }
                            });
                        }
                    });
                }
            });
            ui.vertical(|ui| {
                ui.set_width(vol_w);
                ui.label(RichText::new(format!("Volume {:.0}%", e.drum_volume * 100.0)).size(12.0).color(th().dim));
                fx_bar(ui, core, vol_w, &mut e.drum_volume, DRUM_COLOR, Some(Target::DrumVolume));
            });
        });
        });
        if glowing {
            ui.ctx().request_repaint_after(Duration::from_millis(16));
        }
    });

    core.set_efeitos(e);
    // v0.19: ações que mexem nos arquivos depois de guardar os ajustes (senão a cópia `e` desfazia)
    if let Some(i) = drum_reset {
        core.set_drum_file(i, None);
    }
    if voice_reset {
        core.set_voicefx_file(None);
    }
    if let Some(k) = voice_key {
        core.set_voicefx_key(k);
    }
    if let Some(v) = voice_sel {
        core.set_voicefx_voice(v);
    }
    if drum_dialog.is_some() {
        st.drum_file_dialog = drum_dialog;
    }
    if st_voice_dialog {
        st.voicefx_file_dialog = true;
    }
}

// ---------- Pads ----------

const PAD_COLOR: Color32 = Color32::from_rgb(0xbf, 0x8c, 0xff);

fn pads_page(ui: &mut Ui, core: &mut Core, st: &mut UiState) {
    let avail = ui.available_size();
    let side_w = 170.0;
    let gap = 10.0;

    // Estilos: clássicos (loop) e vivos (motor granular). Escondidos desde a v0.13 (✏ Editar traz de volta).
    let styles: &[(&str, std::ops::Range<usize>)] = if core.builtin_pads() {
        &[
            ("Clássicos:", 0..pads::PAD_BANKS),
            ("Vivos ✨:", pads::PAD_BANKS..pads::MY_BANK),
        ]
    } else {
        &[]
    };
    for (title, range) in styles.iter().cloned() {
        ui.horizontal(|ui| {
            ui.add_sized(
                [78.0, 34.0],
                egui::Label::new(RichText::new(title).color(th().dim)),
            );
            for b in range {
                let sel = core.cfg.pad_bank == b;
                let text =
                    RichText::new(format!("{}  ·  {}", pads::bank_name(b), pads::bank_hint(b)))
                        .size(14.0)
                        .color(if sel { th().ink } else { th().text });
                let r = ui.add_sized(
                    [270.0, 34.0],
                    egui::Button::new(text)
                        .fill(if sel { PAD_COLOR } else { th().glass_hi })
                        .truncate(),
                );
                if !learnable(ui, core, &r, Target::PadBank) && r.clicked() {
                    core.set_pad_bank(b);
                }
            }
            if title.starts_with("Cl") {
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.label(
                        RichText::new("Sem terça: o pad de C serve para C e Cm.")
                            .size(13.0)
                            .color(th().dim),
                    );
                });
            }
        });
    }
    // Meus pads: arquivos do Theo (ex.: pads do YouTube), cada um com nome, tocados em todos os tons.
    // v0.18: só os pads à vista; nome, tom, reverb, pôr/tirar pad ficam no ⚙.
    let details = st.pad_details;
    ui.horizontal(|ui| {
        // v0.21: uma bolinha (planeta) abre a lista de pads, pequenos, do tamanho do nome.
        // Fechada: só a bolinha e o nome do pad escolhido.
        let open = st.pad_list_open || core.learn.active;
        let ball = planet_ball(ui, open);
        if ball
            .on_hover_text(if open { "Fechar a lista de pads" } else { "Escolher outro pad" })
            .clicked()
        {
            st.pad_list_open = !st.pad_list_open;
        }
        let cur = pads::my_slot(core.cfg.pad_bank).filter(|i| *i < core.cfg.my_pads.len());
        let second = core.cfg.pad_bank2;
        ui.spacing_mut().item_spacing.x = 6.0;
        // v0.24: segundo pad (tocando junto) em lilás claro; no modo "Misturar", clicar escolhe o segundo
        const PAD2_COLOR: Color32 = Color32::from_rgb(0xe0, 0xc8, 0xff);
        let mut remove_second = false;
        for i in 0..core.cfg.my_pads.len() {
            let b = pads::MY_BANK + i;
            let sel = core.cfg.pad_bank == b;
            let sel2 = second == Some(b);
            if !open && !sel && !sel2 {
                continue;
            }
            if !open && sel2 {
                ui.label(RichText::new("+").size(16.0).strong().color(th().dim));
            }
            let m = &core.cfg.my_pads[i];
            let text = RichText::new(&m.name)
                .size(if (sel || sel2) && !open { 16.0 } else { 14.0 })
                .strong()
                .color(if sel || sel2 { th().ink } else { th().text });
            let tip = format!("{} · pad em {}", m.name, pads::KEY_NAMES[m.key]);
            let fill = if sel {
                PAD_COLOR
            } else if sel2 {
                PAD2_COLOR
            } else {
                th().glass_hi
            };
            let r = ui
                .add(
                    egui::Button::new(text)
                        .fill(fill)
                        .corner_radius(16)
                        .min_size(Vec2::new(0.0, 32.0)),
                )
                .on_hover_text(tip);
            if !learnable(ui, core, &r, Target::PadBank) && r.clicked() {
                if open && st.pad_mix_mode && !sel {
                    core.set_pad_bank2(Some(b));
                    st.pad_mix_mode = false;
                    st.pad_list_open = false;
                } else if open && !sel {
                    core.set_pad_bank(b);
                    st.pad_list_open = false;
                } else {
                    // clicou no nome com a lista fechada (ou no que já era): abre/fecha a lista
                    st.pad_list_open = !st.pad_list_open;
                    st.pad_mix_mode = false;
                }
            }
            if !open && sel2 {
                if ui
                    .add(egui::Button::new(RichText::new("✖").size(12.0)).fill(th().glass))
                    .on_hover_text("Tirar o segundo pad (fica só um)")
                    .clicked()
                {
                    remove_second = true;
                }
            }
        }
        if remove_second {
            core.set_pad_bank2(None);
        }
        if open && cur.is_some() && core.cfg.my_pads.len() > 1 {
            let on = st.pad_mix_mode;
            let b = egui::Button::new(
                RichText::new(if on { "+ Misturar: escolha o 2º pad" } else { "+ Misturar" })
                    .size(13.0)
                    .color(if on { th().ink } else { th().text }),
            )
            .fill(if on { PAD2_COLOR } else { th().glass })
            .corner_radius(16)
            .min_size(Vec2::new(0.0, 32.0));
            if ui
                .add(b)
                .on_hover_text("Tocar dois pads juntos: clique aqui e depois no pad que vai entrar junto")
                .clicked()
            {
                st.pad_mix_mode = !st.pad_mix_mode;
            }
        }
        // mix entre os dois (só com dois pads)
        if !open && second.is_some() {
            ui.add_space(8.0);
            ui.label(RichText::new("Mix").size(12.0).color(th().dim));
            let r = hbar(ui, Vec2::new(140.0, 28.0), core.cfg.pad_mix, true, PAD_COLOR);
            let r = r.on_hover_text("Esquerda = mais o 1º pad · meio = os dois iguais · direita = mais o 2º. Dá para pôr num knob da mesinha (Aprender).");
            if !learnable(ui, core, &r, Target::PadMix) && r.is_pointer_button_down_on() {
                if let Some(p) = r.interact_pointer_pos() {
                    core.set_pad_mix(((p.x - r.rect.left()) / r.rect.width()).clamp(0.0, 1.0));
                }
            }
        }
        if cur.is_none() && !open {
            ui.label(RichText::new("escolha um pad na bolinha").size(13.0).color(th().dim));
        }
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if details_button(ui, st.pad_details, Vec2::new(48.0, 40.0), "Ajustes dos pads: nome, tom, reverb, pôr ou tirar pad, ajuda de escala").clicked() {
                st.pad_details = !st.pad_details;
            }
        });
    });
    // Ajustes ao vivo
    let mut v = core.cfg.vivo.clone();
    let vivo = pads::is_vivo(core.cfg.pad_bank);
    let cur_slot = pads::my_slot(core.cfg.pad_bank).filter(|i| *i < core.cfg.my_pads.len());
    if details {
        // v0.23.1: todos os ajustes do pad num quadro só, alinhados (antes eram fileiras soltas)
        ui.add_space(4.0);
        egui::Frame::new()
            .fill(th().glass)
            .corner_radius(14)
            .inner_margin(egui::Margin::symmetric(14, 10))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                egui::Grid::new("pad_ajustes")
                    .num_columns(2)
                    .spacing([14.0, 10.0])
                    .show(ui, |ui| {
                        if let Some(i) = cur_slot {
                            ui.label(RichText::new("Nome").color(th().dim));
                            ui.horizontal(|ui| {
                                let mut name = core.cfg.my_pads[i].name.clone();
                                if ui
                                    .add(egui::TextEdit::singleline(&mut name).desired_width(170.0).char_limit(28))
                                    .changed()
                                {
                                    core.rename_my_pad(i, &name);
                                }
                                ui.add_space(12.0);
                                ui.label(RichText::new("está em").color(th().dim));
                                let mut key = core.cfg.my_pads[i].key;
                                egui::ComboBox::from_id_salt("my_pad_key")
                                    .width(56.0)
                                    .selected_text(pads::KEY_NAMES[key])
                                    .show_ui(ui, |ui| {
                                        for k in 0..pads::PAD_KEYS {
                                            ui.selectable_value(&mut key, k, pads::KEY_NAMES[k]);
                                        }
                                    });
                                core.set_my_pad_key(i, key);
                                let path = core.cfg.my_pads[i].path.clone();
                                let credit = path
                                    .strip_prefix("embutido:")
                                    .and_then(|n| n.parse::<usize>().ok())
                                    .and_then(|n| pads::PARTNER_PADS.get(n))
                                    .map(|p| format!("pad de {} (parceria)", p.credit))
                                    .or_else(|| {
                                        path.starts_with("livre:")
                                            .then(|| format!("sons {}", pads::FREE_CREDIT))
                                    })
                                    .unwrap_or_else(|| "o programa descobriu o tom pelo som".into());
                                ui.add_space(12.0);
                                ui.label(RichText::new(credit).size(12.0).color(th().dim));
                            });
                            ui.end_row();
                        }
                        ui.label(RichText::new("Reverb de igreja").color(th().dim));
                        ui.horizontal(|ui| {
                            ui.spacing_mut().slider_width = 200.0;
                            ui.add(egui::Slider::new(&mut v.church_pads, 0.0..=1.0).show_value(false));
                            ui.label(RichText::new(format!("{:.0}%", v.church_pads * 100.0)).size(12.0).color(th().dim));
                            if vivo {
                                ui.spacing_mut().slider_width = 110.0;
                                ui.add_space(12.0);
                                ui.label(RichText::new("Brilho").color(th().dim));
                                ui.add(egui::Slider::new(&mut v.bright, 0.0..=1.0).show_value(false));
                                ui.label(RichText::new("Shimmer").color(th().dim));
                                ui.add(egui::Slider::new(&mut v.shimmer, 0.0..=1.0).show_value(false));
                                ui.label(RichText::new("Movimento").color(th().dim));
                                ui.add(egui::Slider::new(&mut v.motion, 0.0..=1.0).show_value(false));
                            }
                        });
                        ui.end_row();
                        ui.label("");
                        ui.horizontal(|ui| {
                            if core.cfg.my_pads.len() < pads::MY_SLOTS
                                && ui
                                    .button("+ Pôr um pad meu (MP3/WAV)...")
                                    .on_hover_text("Um pad baixado (ex.: do YouTube) em qualquer tom: o programa descobre o tom e toca nos 12")
                                    .clicked()
                            {
                                st.my_pad_dialog = true;
                            }
                            if core.cfg.my_pads.len() < pads::MY_SLOTS {
                                for (n, pp) in pads::PARTNER_PADS.iter().enumerate() {
                                    let path = pads::partner_path(n);
                                    let name = pads::planet_name(&path).map(|(x, _)| x).unwrap_or(pp.name);
                                    if !core.cfg.my_pads.iter().any(|m| m.path == path)
                                        && ui
                                            .button(format!("↺ {name}"))
                                            .on_hover_text("Pôr de volta este pad da parceria")
                                            .clicked()
                                    {
                                        core.restore_partner_pad(n);
                                    }
                                }
                            }
                            if let Some(i) = cur_slot {
                                ui.add_space(12.0);
                                if ui
                                    .button("Tirar este pad")
                                    .on_hover_text("Tira da lista (o arquivo continua no seu computador)")
                                    .clicked()
                                {
                                    core.remove_my_pad(i);
                                }
                            }
                        });
                        ui.end_row();
                    });
            });
        ui.add_space(4.0);
    } else if vivo {
        ui.horizontal(|ui| {
            ui.spacing_mut().slider_width = 120.0;
            ui.label(RichText::new("Reverb de igreja").color(th().dim));
            ui.add(egui::Slider::new(&mut v.church_pads, 0.0..=1.0).show_value(false));
            ui.add_space(12.0);
            ui.label(RichText::new("Brilho").color(th().dim));
            ui.add(egui::Slider::new(&mut v.bright, 0.0..=1.0).show_value(false));
            ui.label(RichText::new("Shimmer").color(th().dim));
            ui.add(egui::Slider::new(&mut v.shimmer, 0.0..=1.0).show_value(false));
            ui.label(RichText::new("Movimento").color(th().dim));
            ui.add(egui::Slider::new(&mut v.motion, 0.0..=1.0).show_value(false));
        });
    }
    if vivo && details {
        let n = core.cfg.pad_bank - pads::PAD_BANKS;
        let custom = core
            .cfg
            .vivo
            .files
            .iter()
            .find(|(k, _, _)| *k == n)
            .cloned();
        ui.horizontal(|ui| {
            ui.label(RichText::new("Base do som:").color(th().dim));
            match &custom {
                Some((_, path, root)) => {
                    let file = std::path::Path::new(path).file_name().map(|f| f.to_string_lossy().to_string()).unwrap_or_default();
                    let k = (0..pads::PAD_KEYS).find(|k| pads::key_midi(*k) == *root).unwrap_or(0);
                    ui.label(RichText::new(format!("♪ {file} (em {})", pads::KEY_NAMES[k])).color(PAD_COLOR));
                    if ui.button("Voltar ao padrão").clicked() {
                        core.set_vivo_file(n, None);
                    }
                }
                None => {
                    ui.label(RichText::new("padrão do programa").color(th().text));
                }
            }
            ui.add_space(8.0);
            ui.label(RichText::new("Seu som está em:").color(th().dim));
            egui::ComboBox::from_id_salt("vivo_root")
                .width(60.0)
                .selected_text(pads::KEY_NAMES[st.vivo_root_key])
                .show_ui(ui, |ui| {
                    for k in 0..pads::PAD_KEYS {
                        ui.selectable_value(&mut st.vivo_root_key, k, pads::KEY_NAMES[k]);
                    }
                });
            if ui
                .button("🎵 Usar meu som (WAV/MP3) como base...")
                .on_hover_text("Ex.: um som do Suno/ElevenLabs numa nota só (sem bateria). O programa transforma em pad de todos os tons.")
                .clicked()
            {
                st.pad_file_dialog = Some((core.cfg.pad_bank, 0));
            }
        });
    }
    core.set_vivo(v);
    ui.add_space(8.0);

    let grid_w = avail.x - side_w - gap;
    // A ajuda de escala grande fica no ⚙ (o lembrete da escala continua na barra de baixo).
    // v0.23.1: a escala só aparece com um pad tocando (sem pad, o espaço vai para os tons e o volume)
    let show_scale = details && core.pad_active.is_some() && !core.cfg.is_hidden("scale_help");
    let scale_h = if show_scale { 196.0 } else { 0.0 };
    let rows_h = ui.available_height() - 30.0 - scale_h;
    // Escolhe a grade (6×2, 4×3 ou 3×4) que deixa os quadrados maiores na tela.
    let (cols, rows, cell) = [(6usize, 2usize), (4, 3), (3, 4)]
        .iter()
        .map(|&(c, r)| {
            let cell = ((grid_w - (c as f32 - 1.0) * gap) / c as f32)
                .min((rows_h - (r as f32 - 1.0) * gap) / r as f32);
            (c, r, cell)
        })
        .fold(
            (6, 2, 0.0f32),
            |best, x| if x.2 > best.2 { x } else { best },
        );
    let cell = cell.max(70.0);
    ui.horizontal_top(|ui| {
        ui.spacing_mut().item_spacing = Vec2::splat(gap);
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing = Vec2::splat(gap);
            for row in 0..rows {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = gap;
                    for col in 0..cols {
                        pad_button(ui, core, st, row * cols + col, cell);
                    }
                });
            }
        });

        // Coluna da direita: volume, anterior/próximo, parar
        let h = (rows as f32 * cell + (rows as f32 - 1.0) * gap).max(250.0);
        let (rect, _) = ui.allocate_exact_size(Vec2::new(side_w, h), Sense::hover());
        crate::theme::glass(ui.painter(), rect, 18, false, None);
        let inner = rect.shrink2(Vec2::new(12.0, 10.0));
        let mut child = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(inner)
                .layout(Layout::top_down(Align::Center)),
        );
        let ui = &mut child;
        ui.label(RichText::new("Volume").strong().size(14.0));
        ui.label(RichText::new(format!("{:.0}%", core.cfg.pad_volume * 100.0)).monospace());
        // v0.23.1: o fader nunca fica espremido (a coluna tem altura mínima)
        let fader_h = (ui.available_height() - 100.0).max(80.0);
        let (resp, newv) = fader(
            ui,
            Vec2::new(inner.width(), fader_h),
            core.cfg.pad_volume,
            PAD_COLOR,
            core.learn.active,
        );
        learnable(ui, core, &resp, Target::PadVolume);
        if let Some(v) = newv {
            core.set_pad_volume(v);
        }
        ui.horizontal(|ui| {
            let w = (inner.width() - ui.spacing().item_spacing.x) / 2.0;
            let prev = ui.add_sized([w, 30.0], egui::Button::new("◀").fill(th().glass_hi)).on_hover_text("Tom anterior");
            if !learnable(ui, core, &prev, Target::PadPrev) && prev.clicked() {
                core.pad_step(-1);
            }
            let next = ui.add_sized([w, 30.0], egui::Button::new("▶").fill(th().glass_hi)).on_hover_text("Próximo tom");
            if !learnable(ui, core, &next, Target::PadNext) && next.clicked() {
                core.pad_step(1);
            }
        });
        let stop = ui.add_sized(
            [inner.width(), 44.0],
            egui::Button::new(RichText::new("■ PARAR PAD").strong().color(Color32::WHITE))
                .fill(if core.pad_active.is_some() { ORANGE } else { th().glass_hi }),
        );
        if !learnable(ui, core, &stop, Target::PadStop) && stop.clicked() {
            core.pad_stop();
        }
    });
    if show_scale {
        ui.add_space(gap);
        let w = ui.available_width();
        scale_panel(ui, core, st, Vec2::new(w, scale_h - gap));
    }
}

fn pad_button(ui: &mut Ui, core: &mut Core, st: &mut UiState, key: usize, size: f32) {
    let bank = core.cfg.pad_bank;
    let (rect, resp) = ui.allocate_exact_size(Vec2::splat(size), Sense::click());
    let active = core.pad_active == Some(key);
    let ready = core.pad_ready(bank, key);
    let custom = core.cfg.pad_file(bank, key).map(|p| {
        std::path::Path::new(p)
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_default()
    });
    let sharp = pads::KEY_NAMES[key].contains('#');
    let p = ui.painter();
    // Vidro; sustenidos um pouco mais escuros (como as teclas pretas).
    if sharp && !active {
        p.rect_filled(rect, 20, Color32::from_black_alpha(70));
    }
    let hover = ui
        .ctx()
        .animate_bool_with_time(resp.id.with("h"), resp.hovered(), 0.15);
    let on = ui
        .ctx()
        .animate_bool_with_time(resp.id.with("on"), active, 0.35);
    theme::glass(p, rect, 20, hover > 0.5, (on > 0.01).then_some(PAD_COLOR));
    if on > 0.01 {
        p.rect_filled(rect, 20, PAD_COLOR.gamma_multiply(0.85 * on));
    }
    let tc = if active { th().ink } else { th().text };
    p.text(
        rect.center() - Vec2::new(0.0, size * 0.08),
        Align2::CENTER_CENTER,
        pads::KEY_NAMES[key],
        FontId::proportional((size * 0.32).clamp(24.0, 56.0)),
        tc,
    );
    p.text(
        rect.center() + Vec2::new(0.0, size * 0.2),
        Align2::CENTER_CENTER,
        pads::KEY_NAMES_PT[key],
        FontId::proportional((size * 0.12).clamp(12.0, 18.0)),
        if active { th().ink } else { th().dim },
    );
    let status = if active && !ready {
        Some("carregando...".to_string())
    } else if active {
        Some("tocando".to_string())
    } else {
        custom.clone().map(|c| format!("♪ {c}"))
    };
    if let Some(sx) = status {
        let galley = ui.painter().layout(
            sx,
            FontId::proportional(11.0),
            if active { th().ink } else { PAD_COLOR },
            rect.width() - 12.0,
        );
        let pos = Pos2::new(
            rect.center().x - galley.size().x / 2.0,
            rect.bottom() - 8.0 - galley.size().y,
        );
        ui.painter().galley(pos, galley, th().text);
    }

    if !learnable(ui, core, &resp, Target::Pad(key)) && resp.clicked() {
        core.pad_press(key);
    }
    let resp = resp.on_hover_text(match &custom {
        Some(c) => format!("Pad {} — arquivo: {c}", pads::KEY_NAMES[key]),
        None => format!(
            "Pad {} ({}) — {}",
            pads::KEY_NAMES[key],
            pads::KEY_NAMES_PT[key],
            core.pad_bank_label(bank)
        ),
    });
    resp.context_menu(|ui| {
        if bank >= pads::PAD_BANKS {
            ui.label(
                RichText::new(if pads::is_vivo(bank) {
                    "Pad Vivo: o som base vale para todos os tons (veja \"Base do som\" acima)."
                } else {
                    "Meu pad: um arquivo só vale para todos os tons (ajustes acima)."
                })
                .size(12.0),
            );
            return;
        }
        if ui.button("📂 Usar meu arquivo (WAV/MP3/OGG)...").clicked() {
            st.pad_file_dialog = Some((bank, key));
            ui.close();
        }
        if custom.is_some() && ui.button("↺ Voltar ao som padrão").clicked() {
            core.set_pad_file(bank, key, None);
            ui.close();
        }
    });
}

// ---------- Bluetooth (conectar a mesinha direto) ----------

fn sons_window(ctx: &egui::Context, core: &Core, st: &mut UiState) {
    if let Some(dl) = &mut st.download {
        match dl.poll() {
            sons::Poll::Running(f) => {
                st.dl_frac = f;
                ctx.request_repaint_after(std::time::Duration::from_millis(250));
            }
            sons::Poll::Done => {
                let p = &sons::CATALOG[dl.idx];
                st.sons_msg = Some((
                    true,
                    format!("{} baixado! Clique em \"Usar este som\".", p.name),
                ));
                st.download = None;
            }
            sons::Poll::Failed(e) => {
                st.sons_msg = Some((false, e));
                st.download = None;
            }
        }
    }
    if !st.sons_open {
        return;
    }
    let mut open = true;
    let current = core.cfg.sf2_path.clone().unwrap_or_default();
    egui::Window::new("Sons grátis")
        .open(&mut open)
        .collapsible(false)
        .resizable(false)
        .default_width(560.0)
        .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
        .show(ctx, |ui| {
            ui.label(RichText::new("Sons livres, que podem ser usados à vontade (até na igreja). 1) Baixar   2) Usar este som.").size(14.0));
            ui.label(
                RichText::new("Depois de usar, escolha o timbre de cada camada clicando no nome dela, como sempre. Seu SF2 de sempre continua aí: é só abrir ele de novo no 📂.")
                    .size(12.0)
                    .color(th().dim),
            );
            ui.add_space(6.0);
            for (i, p) in sons::CATALOG.iter().enumerate() {
                let path = sons::path(p).to_string_lossy().to_string();
                let have = sons::installed(p);
                let in_use = have && current == path;
                egui::Frame::new()
                    .fill(th().glass)
                    .corner_radius(10.0)
                    .inner_margin(10.0)
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(p.name).size(16.0).strong());
                            ui.label(RichText::new(format!("{:.0} MB", p.bytes as f64 / 1e6)).color(th().dim));
                            if in_use {
                                ui.label(RichText::new("✔ tocando agora").color(GREEN));
                            }
                        });
                        ui.label(RichText::new(p.about).size(13.0));
                        ui.label(RichText::new(p.license).size(11.0).color(th().dim));
                        let busy = st.download.as_ref().map(|d| d.idx);
                        ui.horizontal(|ui| {
                            if busy == Some(i) {
                                let frac = st.dl_frac;
                                ui.add(
                                    egui::ProgressBar::new(frac)
                                        .desired_width(300.0)
                                        .text(format!("Baixando · {:.0}%", frac * 100.0)),
                                );
                                if ui.button("Cancelar").clicked() {
                                    if let Some(dl) = st.download.take() {
                                        dl.cancel();
                                    }
                                }
                            } else if have {
                                if !in_use
                                    && ui
                                        .add(egui::Button::new(RichText::new("▶ Usar este som").color(th().ink)).fill(GREEN))
                                        .clicked()
                                {
                                    st.load_sf2 = Some(path.clone());
                                    st.sons_msg = None;
                                }
                                if !in_use
                                    && ui
                                        .small_button("Apagar")
                                        .on_hover_text("Tirar do notebook para liberar espaço")
                                        .clicked()
                                {
                                    sons::remove(p);
                                }
                            } else if ui
                                .add_enabled(busy.is_none(), egui::Button::new(RichText::new("⬇ Baixar").color(th().ink)).fill(ORANGE))
                                .clicked()
                            {
                                match sons::Download::start(i) {
                                    Ok(d) => {
                                        st.download = Some(d);
                                        st.dl_frac = 0.0;
                                        st.sons_msg = None;
                                    }
                                    Err(e) => st.sons_msg = Some((false, e)),
                                }
                            }
                        });
                    });
                ui.add_space(4.0);
            }
            if let Some((ok, m)) = &st.sons_msg {
                ui.label(RichText::new(m).color(if *ok { GREEN } else { RED }));
            }
            ui.label(
                RichText::new(format!("Ficam guardados em {}", sons::dir().display()))
                    .size(11.0)
                    .color(th().dim),
            );
        });
    if !open {
        st.sons_open = false;
    }
}

fn bt_window(
    ctx: &egui::Context,
    st: &mut UiState,
    midi_status: &SharedMidiStatus,
    bt: &crossbeam_channel::Sender<BtCmd>,
) {
    if !st.bt_open {
        if st.bt_was_open {
            let _ = bt.send(BtCmd::StopScan);
            st.bt_was_open = false;
        }
        return;
    }
    if !st.bt_was_open {
        let _ = bt.send(BtCmd::StartScan);
        st.bt_was_open = true;
    }
    let (scanning, err, hits, saved) = midi_status
        .lock()
        .map(|s| {
            (
                s.bt_scanning,
                s.bt_scan_error.clone(),
                s.bt_hits.clone(),
                s.bt_states.clone(),
            )
        })
        .unwrap_or_default();
    let mut open = true;
    egui::Window::new("Bluetooth — conectar a mesinha")
        .open(&mut open)
        .collapsible(false)
        .default_size([520.0, 480.0])
        .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
        .show(ctx, |ui| {
            if cfg!(not(windows)) {
                ui.label(RichText::new("Bluetooth direto só funciona no Windows.").color(YELLOW));
            }
            ui.label(RichText::new("1) Ligue o Bluetooth da mesinha (botão BT, luz piscando).   2) Espere ela aparecer abaixo.   3) Clique em Conectar.").size(14.0));
            ui.label(
                RichText::new("O programa fala direto com o Bluetooth da mesinha, sem precisar de driver. Depois de conectada uma vez, ela reconecta sozinha sempre que você abrir o programa.")
                    .size(12.0)
                    .color(th().dim),
            );
            ui.add_space(6.0);

            if !saved.is_empty() {
                ui.label(RichText::new("Conectados por aqui").strong());
                for (addr, name, state) in &saved {
                    ui.horizontal(|ui| {
                        let ok = state.starts_with("conectado");
                        let (r, _) = ui.allocate_exact_size(Vec2::splat(14.0), Sense::hover());
                        ui.painter().circle_filled(r.center(), 5.0, if ok { GREEN } else { ORANGE });
                        ui.label(RichText::new(name).strong());
                        ui.label(RichText::new(state).size(12.0).color(th().dim));
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if ui.small_button("Esquecer").clicked() {
                                let _ = bt.send(BtCmd::Forget { address: *addr });
                            }
                        });
                    });
                }
                ui.separator();
            }

            ui.horizontal(|ui| {
                ui.label(RichText::new("Aparelhos por perto").strong());
                if scanning {
                    ui.spinner();
                    ui.label(RichText::new("procurando...").size(12.0).color(th().dim));
                }
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ui.button("Procurar de novo").clicked() {
                        let _ = bt.send(BtCmd::StartScan);
                    }
                });
            });
            if let Some(e) = &err {
                ui.label(RichText::new(format!("⚠ {e}")).color(RED));
            }
            if hits.is_empty() {
                ui.label(
                    RichText::new("Nada ainda. Confira se o Bluetooth do notebook está ligado e se a luz da mesinha está piscando. Se ela estiver conectada no celular/tablet, desconecte de lá.")
                        .size(13.0)
                        .color(th().dim),
                );
            }
            egui::ScrollArea::vertical().max_height(260.0).show(ui, |ui| {
                for h in &hits {
                    ui.horizontal(|ui| {
                        let bars = match h.rssi {
                            r if r > -60 => "▮▮▮▮",
                            r if r > -70 => "▮▮▮▯",
                            r if r > -80 => "▮▮▯▯",
                            _ => "▮▯▯▯",
                        };
                        ui.label(RichText::new(bars).monospace().color(if h.rssi > -100 { GREEN } else { th().dim }));
                        let name = if h.name.is_empty() { "(sem nome)".to_string() } else { h.name.clone() };
                        ui.label(RichText::new(&name).strong());
                        if h.midi {
                            ui.label(RichText::new("MIDI").size(11.0).color(LAYER_COLORS[0]));
                        }
                        if h.paired {
                            ui.label(RichText::new("pareado no Windows").size(11.0).color(th().dim));
                        }
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            let already = saved.iter().any(|(a, _, _)| *a == h.address);
                            let b = egui::Button::new(RichText::new(if already { "Conectado" } else { "Conectar" }).color(if already { th().dim } else { th().ink }))
                                .fill(if already { th().glass } else { GREEN });
                            if ui.add_enabled(!already, b).clicked() {
                                let _ = bt.send(BtCmd::Connect { address: h.address, name: name.clone() });
                            }
                        });
                    });
                }
            });
        });
    if !open {
        st.bt_open = false;
    }
}

// ---------- Cenas ----------

fn scenes_bar(ui: &mut Ui, core: &mut Core, st: &mut UiState) {
    ui.horizontal(|ui| {
        // v0.18: cenas grandes; Salvar, Teclado e ⚙ Detalhes viram ícones pequenos no fim.
        let icon_w = 48.0;
        let save_w = icon_w;
        let det_w = icon_w;
        let gap = ui.spacing().item_spacing.x;
        let count = core.cfg.scenes.len();
        let kb_w = if core.cfg.is_hidden("kb_button") { 0.0 } else { icon_w };
        let add_w = if st.editing && count < MAX_SCENES { 56.0 } else { 0.0 };
        let slots = count as f32 + 2.0 + if kb_w > 0.0 { 1.0 } else { 0.0 } + if add_w > 0.0 { 1.0 } else { 0.0 };
        let w = ((ui.available_width() - save_w - det_w - kb_w - add_w - gap * slots) / count.max(1) as f32).clamp(44.0, 240.0);
        let mut delete: Option<usize> = None;
        for n in 0..count {
            let current = core.cfg.current_scene == Some(n);
            let filled = core.cfg.scenes[n].mix.is_some();
            let name = core.cfg.scenes[n].name.clone();
            let label = name.clone();
            let text = RichText::new(label)
                .size(15.0)
                .color(if current { th().ink } else if filled { th().text } else { th().dim });
            let btn = egui::Button::new(text)
                .fill(if current { GREEN } else if filled { th().glass_hi } else { th().glass })
                .stroke(if current { Stroke::NONE } else { Stroke::new(1.0, th().glass_hi) });
            let resp = ui.add_sized([w, 44.0], btn.truncate());
            let resp = if n < 8 {
                resp.on_hover_text(format!("Tecla F{} · botão direito: salvar, renomear, apagar", n + 1))
            } else {
                resp.on_hover_text("Botão direito: salvar, renomear, apagar")
            };
            if st.editing && count > 1 && remove_x(ui, resp.rect, ("cena", n)) {
                delete = Some(n);
            }
            if core.learn.active {
                learn_outline(ui, &resp, core, Target::Scene(n));
                if resp.clicked() {
                    core.learn.target = Some(Target::Scene(n));
                }
            } else if resp.clicked() {
                core.load_scene(n);
            }
            resp.context_menu(|ui| {
                if ui.button("💾 Salvar o som atual aqui").clicked() {
                    core.save_scene(n);
                    ui.close();
                }
                if ui.button("✏ Renomear").clicked() {
                    st.rename = Some((n, core.cfg.scenes[n].name.clone()));
                    ui.close();
                }
                if ui.button("🗑 Apagar").clicked() {
                    core.cfg.scenes[n].mix = None;
                    core.cfg.scenes[n].extra = None;
                    core.dirty = true;
                    ui.close();
                }
            });
        }
        let target = core.cfg.current_scene;
        let save = egui::Button::new(RichText::new("💾").size(18.0)).fill(th().glass_hi);
        let resp = ui.add_sized([save_w, 44.0], save).on_hover_text(
            "Salvar cena: guarda os sons, volumes e liga/desliga na cena selecionada.",
        );
        if resp.clicked() {
            match target {
                Some(n) => core.save_scene(n),
                None => {
                    let free = core.cfg.scenes.iter().position(|s| s.mix.is_none()).unwrap_or(0);
                    core.save_scene(free);
                }
            }
        }
        if add_w > 0.0
            && ui
                .add_sized([add_w, 44.0], egui::Button::new(RichText::new("+").size(22.0)).fill(th().glass_hi))
                .on_hover_text("Nova cena")
                .clicked()
        {
            core.add_scene();
        }
        if let Some(n) = delete {
            core.delete_scene(n);
        }
        if kb_w > 0.0 {
            let kb = egui::Button::new(
                RichText::new("🎹")
                    .size(18.0)
                    .color(if st.kb_open { th().ink } else { th().text }),
            )
            .fill(if st.kb_open { LAYER_COLORS[0] } else { th().glass_hi });
            if ui
                .add_sized([kb_w, 44.0], kb)
                .on_hover_text("Teclado na tela: testar os sons sem o piano ligado")
                .clicked()
            {
                st.kb_open = !st.kb_open;
            }
        }
        if details_button(ui, st.mix_details, Vec2::new(det_w, 44.0), "Detalhes: reverb, solo e ◀ ▶ das camadas; pitch e mod").clicked() {
            st.mix_details = !st.mix_details;
        }
    });
}

fn rename_window(ctx: &egui::Context, core: &mut Core, st: &mut UiState) {
    let Some((n, mut text)) = st.rename.take() else {
        return;
    };
    let mut keep = true;
    egui::Window::new("Renomear cena")
        .collapsible(false)
        .resizable(false)
        .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
        .show(ctx, |ui| {
            let r = ui.add(egui::TextEdit::singleline(&mut text).desired_width(260.0));
            r.request_focus();
            ui.horizontal(|ui| {
                let enter = ui.input(|i| i.key_pressed(Key::Enter));
                if ui.button("OK").clicked() || enter {
                    let name: String = text.trim().chars().take(24).collect();
                    if !name.is_empty() {
                        core.cfg.scenes[n].name = name;
                        core.dirty = true;
                    }
                    keep = false;
                }
                if ui.button("Cancelar").clicked() {
                    keep = false;
                }
            });
        });
    if keep {
        st.rename = Some((n, text));
    }
}

// ---------- Mesa ----------

fn mixer(ui: &mut Ui, core: &mut Core, st: &mut UiState) {
    let avail = ui.available_size();
    let gap = 8.0;
    let master_w = 150.0;
    // Como na SMC-Mixer: canais 1..6 = camadas, 7 = PAD, 8 = GERAL.
    let layers: Vec<usize> = (0..LAYERS)
        .filter(|i| !core.cfg.is_hidden(&layer_id(*i)))
        .collect();
    let show_pad = !core.cfg.is_hidden("pad_strip");
    let n = layers.len() + usize::from(show_pad);
    let strip_w = ((avail.x - master_w - gap * n as f32) / n.max(1) as f32).clamp(84.0, 220.0);
    let h = avail.y.max(360.0);
    let mut remove: Option<String> = None;
    ui.horizontal_top(|ui| {
        ui.spacing_mut().item_spacing.x = gap;
        for &i in &layers {
            let r = strip(ui, core, st, i, Vec2::new(strip_w, h));
            if st.editing && remove_x(ui, r, ("camada", i)) {
                remove = Some(layer_id(i));
            }
        }
        if show_pad {
            let r = pad_strip(ui, core, Vec2::new(strip_w, h), st.mix_details);
            if st.editing && remove_x(ui, r, ("pad", 0)) {
                remove = Some("pad_strip".into());
            }
        }
        master_strip(ui, core, Vec2::new(master_w, h), st.mix_details);
    });
    if let Some(id) = remove {
        core.set_hidden(&id, true);
    }
}

/// Botãozinho ✕ no canto de algo (modo Editar). Devolve true se clicado.
fn remove_x(ui: &mut Ui, rect: Rect, id: impl std::hash::Hash + std::fmt::Debug) -> bool {
    let c = Pos2::new(rect.right() - 13.0, rect.top() + 13.0);
    let hit = Rect::from_center_size(c, Vec2::splat(24.0));
    let resp = ui.interact(hit, egui::Id::unique(("remover", id)), Sense::click());
    let p = ui.painter();
    p.circle_filled(c, if resp.hovered() { 12.0 } else { 11.0 }, RED);
    p.circle_stroke(c, 11.0, Stroke::new(1.5, Color32::WHITE));
    let d = 4.5;
    let x = Stroke::new(2.0, Color32::WHITE);
    p.line_segment([c + Vec2::new(-d, -d), c + Vec2::new(d, d)], x);
    p.line_segment([c + Vec2::new(-d, d), c + Vec2::new(d, -d)], x);
    resp.on_hover_text("Tirar da tela").clicked()
}

/// Faixa azul do modo Editar: o que está escondido volta pelo "+ Adicionar".
fn edit_banner(ui: &mut Ui, core: &mut Core, st: &mut UiState) {
    ui.add_space(4.0);
    egui::Frame::new()
        .fill(Color32::from_rgb(0x0a, 0x84, 0xff))
        .corner_radius(6)
        .inner_margin(egui::Margin::symmetric(10, 4))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("EDITANDO — clique na bolinha vermelha para tirar.")
                        .color(Color32::WHITE)
                        .strong(),
                );
                let hidden: Vec<(&str, &str)> = EDITABLE
                    .iter()
                    .copied()
                    .filter(|(id, _)| core.cfg.is_hidden(id))
                    .collect();
                ui.menu_button(RichText::new("+ Adicionar").strong(), |ui| {
                    if core.cfg.scenes.len() < MAX_SCENES && ui.button("Nova cena").clicked() {
                        core.add_scene();
                        ui.close();
                    }
                    if hidden.is_empty() {
                        ui.label(RichText::new("Nada escondido.").color(th().dim));
                    }
                    for (id, name) in &hidden {
                        if ui.button(format!("Colocar de volta: {name}")).clicked() {
                            core.set_hidden(id, false);
                            ui.close();
                        }
                    }
                });
                ui.menu_button(RichText::new("− Tirar").strong(), |ui| {
                    let others = [
                        ("scenes", "Barra de cenas"),
                        ("tab_pads", "Aba Pads"),
                        ("kb_button", "Botão Teclado"),
                        ("bt_button", "Botão Bluetooth"),
                        ("wizard_button", "Botão Configurar mesinha"),
                        ("learn_button", "Botão Aprender"),
                        ("sons_button", "Botão Sons grátis"),
                        ("pad_styles", "Estilos de pad do programa"),
                    ];
                    for (id, label) in others {
                        if !core.cfg.is_hidden(id) && ui.button(label).clicked() {
                            core.set_hidden(id, true);
                            ui.close();
                        }
                    }
                });
                let _ = &st;
            });
        });
}

/// Canal 7: o pad (volume, tom, estilo e parar) sem precisar ir para a aba Pads.
fn pad_strip(ui: &mut Ui, core: &mut Core, size: Vec2, details: bool) -> Rect {
    let details = details || core.learn.active;
    let playing = core.pad_active.is_some();
    let compact = compact_height(size.y);
    let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
    let painter = ui.painter_at(rect);
    let _ = painter;
    crate::theme::glass(
        ui.painter(),
        rect,
        18,
        playing,
        playing.then_some(PAD_COLOR),
    );
    let inner = rect.shrink2(Vec2::new(8.0, 8.0));
    let mut child = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(inner)
            .layout(Layout::top_down(Align::Center)),
    );
    let ui = &mut child;
    if compact {
        ui.spacing_mut().item_spacing.y = 4.0;
    }
    ui.horizontal(|ui| {
        let (r, _) = ui.allocate_exact_size(
            Vec2::new(inner.width(), if compact { 16.0 } else { 22.0 }),
            Sense::hover(),
        );
        ui.painter().rect_filled(
            Rect::from_min_size(r.min, Vec2::new(r.width(), 4.0)),
            2,
            PAD_COLOR,
        );
        ui.painter().text(
            r.center() + Vec2::new(0.0, 4.0),
            Align2::CENTER_CENTER,
            "7 · PAD",
            FontId::proportional(17.0),
            if playing { th().text } else { th().dim },
        );
    });
    // Tom atual (grande) — o knob 7 da mesinha escolhe o tom.
    let key = core.pad_active.map(|k| pads::KEY_NAMES[k]).unwrap_or("—");
    let r = ui.add_sized(
        [inner.width(), if compact { 34.0 } else { 50.0 }],
        egui::Button::new(RichText::new(key).size(26.0).strong().color(if playing {
            th().ink
        } else {
            th().dim
        }))
        .fill(if playing { PAD_COLOR } else { th().glass }),
    );
    let r = r.on_hover_text("Tom do pad (knob 7 da mesinha). Clique para ir à aba Pads.");
    if !learnable(ui, core, &r, Target::PadKnob) && r.clicked() {
        core.cfg.tab = 1;
    }
    ui.label(
        RichText::new(core.pad_bank_label(core.cfg.pad_bank))
            .size(12.0)
            .color(th().dim),
    );
    let stop = ui.add_sized(
        [inner.width(), if compact { 30.0 } else { 40.0 }],
        egui::Button::new(
            RichText::new("PARAR")
                .strong()
                .size(15.0)
                .color(if playing { th().ink } else { th().dim }),
        )
        .fill(if playing { ORANGE } else { th().glass }),
    );
    if !learnable(ui, core, &stop, Target::PadStop) && stop.clicked() {
        core.pad_stop();
    }
    if details {
        let bank = ui.add_sized(
            [inner.width(), if compact { 24.0 } else { 28.0 }],
            egui::Button::new(RichText::new("ESTILO").size(13.0).color(th().dim)).fill(th().glass),
        );
        let bank = bank.on_hover_text("Troca o estilo do pad (passa pelos seus pads)");
        if !learnable(ui, core, &bank, Target::PadBank) && bank.clicked() {
            core.set_pad_bank(core.cfg.pad_bank + 1);
        }
    }
    let arrows_h = if details { if compact { 34.0 } else { 42.0 } } else { 0.0 };
    let fader_h = (ui.available_height() - arrows_h - 26.0).max(60.0);
    ui.label(RichText::new(format!("{:.0}%", core.cfg.pad_volume * 100.0)).monospace());
    let (resp, newv) = fader(
        ui,
        Vec2::new(inner.width(), fader_h),
        core.cfg.pad_volume,
        if playing { PAD_COLOR } else { th().dim },
        core.learn.active,
    );
    learnable(ui, core, &resp, Target::PadVolume);
    if let Some(v) = newv {
        core.set_pad_volume(v);
    }
    if !details {
        return rect;
    }
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        let w = (inner.width() - ui.spacing().item_spacing.x) / 2.0;
        let prev = ui
            .add_sized(
                [w, if compact { 26.0 } else { 34.0 }],
                egui::Button::new("◀").fill(th().glass),
            )
            .on_hover_text("Tom anterior");
        if !learnable(ui, core, &prev, Target::PadPrev) && prev.clicked() {
            core.pad_step(-1);
        }
        let next = ui
            .add_sized(
                [w, if compact { 26.0 } else { 34.0 }],
                egui::Button::new("▶").fill(th().glass),
            )
            .on_hover_text("Próximo tom");
        if !learnable(ui, core, &next, Target::PadNext) && next.clicked() {
            core.pad_step(1);
        }
    });
    rect
}

/// Tela baixa (ex.: teclado aberto num notebook 1366×768): canal mais enxuto.
fn compact_height(h: f32) -> bool {
    h < 470.0
}

fn strip(ui: &mut Ui, core: &mut Core, st: &mut UiState, i: usize, size: Vec2) -> Rect {
    let color = LAYER_COLORS[i];
    let layer = core.cfg.mix.layers[i].clone();
    let audible = core.cfg.mix.audible(i);
    let compact = compact_height(size.y);
    let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
    let painter = ui.painter_at(rect);
    let _ = painter;
    crate::theme::glass(ui.painter(), rect, 18, layer.on, audible.then_some(color));
    let inner = rect.shrink2(Vec2::new(8.0, if compact { 6.0 } else { 8.0 }));
    let mut child = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(inner)
            .layout(Layout::top_down(Align::Center)),
    );
    let ui = &mut child;
    if compact {
        ui.spacing_mut().item_spacing.y = 4.0;
    }

    // Número + cor do canal
    ui.horizontal(|ui| {
        let hh = if compact { 16.0 } else { 22.0 };
        let (r, _) = ui.allocate_exact_size(Vec2::new(inner.width(), hh), Sense::hover());
        ui.painter().rect_filled(
            Rect::from_min_size(r.min, Vec2::new(r.width(), 4.0)),
            2,
            color,
        );
        ui.painter().text(
            r.center() + Vec2::new(0.0, 3.0),
            Align2::CENTER_CENTER,
            format!("{}", i + 1),
            FontId::proportional(if compact { 14.0 } else { 17.0 }),
            if audible { th().text } else { th().dim },
        );
    });

    // v0.18: "modo palco" = só timbre, liga/desliga e volume. ⚙ Detalhes (ou Aprender) mostra o resto.
    let details = st.mix_details || core.learn.active;

    // Timbre (no topo: é o que identifica a camada)
    let name = layer
        .preset
        .as_ref()
        .map(|p| p.name.clone())
        .unwrap_or_else(|| "— escolher —".into());
    let name_resp = ui.add_sized(
        [inner.width(), if compact { 28.0 } else { 34.0 }],
        egui::Button::new(
            RichText::new(&name)
                .size(if compact { 12.0 } else { 13.0 })
                .color(if layer.preset.is_some() { th().text } else { th().dim }),
        )
        .fill(th().glass)
        .truncate(),
    );
    let name_resp = name_resp.on_hover_text(format!("{name}\nClique para escolher o timbre"));
    if name_resp.clicked() && !core.learn.active {
        st.picker = Some(i);
        st.picker_search.clear();
        st.picker_focus = true;
    }
    if details {
        let mut step = 0;
        ui.horizontal(|ui| {
            let w = (inner.width() - ui.spacing().item_spacing.x) / 2.0;
            if ui
                .add_sized([w, 22.0], egui::Button::new("◀").fill(th().glass))
                .on_hover_text("Timbre anterior")
                .clicked()
            {
                step = -1;
            }
            if ui
                .add_sized([w, 22.0], egui::Button::new("▶").fill(th().glass))
                .on_hover_text("Próximo timbre")
                .clicked()
            {
                step = 1;
            }
        });
        if step != 0 {
            core.step_preset(i, step);
        }
        // Knob (reverb)
        let (resp, newv) = knob(
            ui,
            if compact { 34.0 } else { 44.0 },
            layer.reverb,
            color,
            core.learn.active,
        );
        let resp = resp.on_hover_text(format!("Reverb {:.0}%", layer.reverb * 100.0));
        learnable(ui, core, &resp, Target::Reverb(i));
        if let Some(v) = newv {
            core.set_reverb(i, v);
        }
        if !compact {
            ui.label(
                RichText::new(format!("Reverb {:.0}%", layer.reverb * 100.0))
                    .size(12.0)
                    .color(th().dim),
            );
        }
    } else {
        ui.add_space(if compact { 2.0 } else { 6.0 });
    }

    // Liga/desliga + Solo (lado a lado quando a tela é baixa)
    let (txt, fill, tc) = match (layer.on, compact && details) {
        (true, false) => ("LIGADA", GREEN, th().ink),
        (false, false) => ("DESLIGADA", th().glass, th().dim),
        (true, true) => ("LIGA", GREEN, th().ink),
        (false, true) => ("DESL.", th().glass, th().dim),
    };
    let toggle_btn = egui::Button::new(
        RichText::new(txt)
            .strong()
            .size(if compact { 13.0 } else { 15.0 })
            .color(tc),
    )
    .fill(fill)
    .stroke(Stroke::new(
        1.0,
        if layer.on { GREEN } else { th().glass_hi },
    ));
    let solo_btn = egui::Button::new(
        RichText::new(if compact { "S" } else { "SOLO" })
            .size(13.0)
            .color(if layer.solo { th().ink } else { th().dim }),
    )
    .fill(if layer.solo { YELLOW } else { th().glass })
    .stroke(Stroke::new(
        1.0,
        if layer.solo { YELLOW } else { th().glass_hi },
    ));
    let (t_resp, s_resp) = if !details {
        (ui.add_sized([inner.width(), if compact { 36.0 } else { 48.0 }], toggle_btn), None)
    } else if compact {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 4.0;
            let tw = (inner.width() - 4.0) * 0.66;
            let t = ui.add_sized([tw, 32.0], toggle_btn);
            let s = ui.add_sized([inner.width() - 4.0 - tw, 32.0], solo_btn);
            (t, Some(s))
        })
        .inner
    } else {
        let t = ui.add_sized([inner.width(), 40.0], toggle_btn);
        let s = ui.add_sized([inner.width(), 28.0], solo_btn);
        (t, Some(s))
    };
    if !learnable(ui, core, &t_resp, Target::Toggle(i)) && t_resp.clicked() {
        core.toggle(i);
    }
    if let Some(s_resp) = s_resp {
        if !learnable(ui, core, &s_resp, Target::Solo(i)) && s_resp.clicked() {
            core.toggle_solo(i);
        }
    }

    // Fader (usa todo o espaço que sobra)
    let fader_h = (ui.available_height() - 26.0).max(60.0);
    ui.label(
        RichText::new(format!("{:.0}%", layer.volume * 100.0))
            .monospace()
            .color(if audible { th().text } else { th().dim }),
    );
    let (resp, newv) = fader(
        ui,
        Vec2::new(inner.width(), fader_h),
        layer.volume,
        if audible { color } else { th().dim },
        core.learn.active,
    );
    learnable(ui, core, &resp, Target::Volume(i));
    if let Some(v) = newv {
        core.set_volume(i, v);
    }
    rect
}

fn master_strip(ui: &mut Ui, core: &mut Core, size: Vec2, details: bool) {
    // Pitch/Mod só nos detalhes (ou no Aprender, para dar para ligar nas teclas do AMW).
    let details = details || core.learn.active || core.pitch.abs() > 0.01 || core.modulation > 0.0;
    let compact = compact_height(size.y);
    let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
    crate::theme::glass(ui.painter(), rect, 18, false, None);
    let inner = rect.shrink2(Vec2::new(10.0, 8.0));
    let mut child = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(inner)
            .layout(Layout::top_down(Align::Center)),
    );
    let ui = &mut child;
    if compact {
        ui.spacing_mut().item_spacing.y = 4.0;
    }
    ui.label(RichText::new("8 · GERAL").strong().size(16.0));
    ui.add_space(4.0);

    // Transpor
    if !compact {
        ui.label(RichText::new("Transpor").size(12.0).color(th().dim));
    }
    ui.horizontal(|ui| {
        let w = (inner.width() - 2.0 * ui.spacing().item_spacing.x) / 3.0;
        let down = ui.add_sized(
            [w, if compact { 26.0 } else { 32.0 }],
            egui::Button::new("−").fill(th().glass_hi),
        );
        if !learnable(ui, core, &down, Target::TransposeDown) && down.clicked() {
            core.set_transpose(core.cfg.mix.transpose - 1);
        }
        let t = core.cfg.mix.transpose;
        let tl = if t == 0 {
            "0".to_string()
        } else {
            format!("{t:+}")
        };
        let r = ui.add_sized(
            [w, if compact { 26.0 } else { 32.0 }],
            egui::Button::new(RichText::new(tl).strong().color(if t == 0 {
                th().text
            } else {
                ORANGE
            }))
            .fill(th().glass),
        );
        if r.on_hover_text("Clique para voltar a 0").clicked() {
            core.set_transpose(0);
        }
        let up = ui.add_sized(
            [w, if compact { 26.0 } else { 32.0 }],
            egui::Button::new("+").fill(th().glass_hi),
        );
        if !learnable(ui, core, &up, Target::TransposeUp) && up.clicked() {
            core.set_transpose(core.cfg.mix.transpose + 1);
        }
    });

    if details {
        ui.add_space(if compact { 2.0 } else { 6.0 });
        pitch_mod_controls(ui, core, inner.width(), compact);
    }

    ui.add_space(if compact { 2.0 } else { 8.0 });
    ui.label(RichText::new(format!("{:.0}%", core.cfg.master * 100.0)).monospace());
    let freeze_h = if compact { 32.0 } else { 42.0 };
    let fader_h =
        (ui.available_height() - if compact { 48.0 } else { 64.0 } - freeze_h - 6.0).max(60.0);
    let (resp, newv) = fader(
        ui,
        Vec2::new(inner.width(), fader_h),
        core.cfg.master,
        Color32::WHITE,
        core.learn.active,
    );
    learnable(ui, core, &resp, Target::Master);
    if let Some(v) = newv {
        core.set_master(v);
    }
    ui.add_space(if compact { 2.0 } else { 6.0 });
    // v0.16: Congelar — o acorde que está soando fica tocando sozinho (aperta de novo: some devagar).
    let on = core.freeze_on;
    // v0.24: FREEZE e, do lado, ↺ (volta o último Freeze)
    let (fz, back) = ui
        .horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 4.0;
            let back_w = 36.0;
            let fz = ui.add_sized(
                [inner.width() - back_w - 4.0, freeze_h],
                egui::Button::new(
                    RichText::new(if on { "❄ FREEZE ON" } else { "❄ FREEZE" })
                        .strong()
                        .size(15.0)
                        .color(if on { th().ink } else { th().text }),
                )
                .fill(if on { PITCH_COLOR } else { th().glass_hi }),
            );
            let has = core.has_last_freeze();
            let back = ui.add_sized(
                [back_w, freeze_h],
                egui::Button::new(RichText::new("↺").size(18.0).strong().color(if has { th().text } else { th().dim }))
                    .fill(th().glass_hi),
            );
            (fz, back)
        })
        .inner;
    let fz = fz.on_hover_text(
        "Toque um acorde e aperte: o som fica soando sozinho, mesmo trocando os timbres. \
         Aperte de novo para ele sumir devagar.",
    );
    let back = back.on_hover_text("↺ Voltar o último Freeze (o último som congelado, mesmo depois de reabrir o programa)");
    if !learnable(ui, core, &fz, Target::Freeze) && fz.clicked() {
        core.freeze_toggle();
    }
    if !learnable(ui, core, &back, Target::FreezeRecall) && back.clicked() {
        core.freeze_recall();
    }
    ui.add_space(if compact { 2.0 } else { 6.0 });
    let panic = ui.add_sized(
        [inner.width(), if compact { 36.0 } else { 48.0 }],
        egui::Button::new(
            RichText::new("■ PÂNICO")
                .strong()
                .size(16.0)
                .color(Color32::WHITE),
        )
        .fill(RED),
    );
    let panic = panic.on_hover_text("Corta todas as notas presas (tecla Esc)");
    if !learnable(ui, core, &panic, Target::Panic) && panic.clicked() {
        core.panic();
    }
}

/// v0.15: Pitch (volta ao meio quando solta) com botões de segurar ▼ ▲, e Mod (vibrato).
/// Tudo dá para ligar no teclado (ex.: AMW MIDI Control Mini 32) pelo Aprender.
fn pitch_mod_controls(ui: &mut Ui, core: &mut Core, w: f32, compact: bool) {
    let h = if compact { 24.0 } else { 28.0 };
    let gap = ui.spacing().item_spacing.x;
    let semis = core.cfg.pitch_range as f32;
    if !compact {
        let st = core.pitch * semis;
        let txt = if core.pitch.abs() < 0.005 {
            "Pitch".to_string()
        } else {
            format!("Pitch {st:+.1}")
        };
        ui.label(RichText::new(txt).size(12.0).color(if core.pitch.abs() < 0.005 { th().dim } else { PITCH_COLOR }));
    }
    ui.horizontal(|ui| {
        let bw = 26.0;
        hold_button(ui, core, Vec2::new(bw, h), "▼", Target::PitchDown, core.pitch < -0.99);
        let bar_w = (w - 2.0 * bw - 2.0 * gap).max(30.0);
        let resp = hbar(ui, Vec2::new(bar_w, h), (core.pitch + 1.0) / 2.0, true, PITCH_COLOR);
        let resp = resp.on_hover_text("Pitch bend: arraste para os lados; soltou, volta ao meio");
        if !learnable(ui, core, &resp, Target::Pitch) {
            if resp.is_pointer_button_down_on() {
                if let Some(p) = resp.interact_pointer_pos() {
                    let v = ((p.x - resp.rect.left()) / resp.rect.width()).clamp(0.0, 1.0);
                    core.set_pitch(v * 2.0 - 1.0);
                }
            } else if resp.drag_stopped() || resp.clicked() {
                core.set_pitch(0.0);
            }
        }
        hold_button(ui, core, Vec2::new(bw, h), "▲", Target::PitchUp, core.pitch > 0.99);
    });
    ui.add_space(if compact { 2.0 } else { 4.0 });
    if !compact {
        ui.label(
            RichText::new(format!("Mod {:.0}%", core.modulation * 100.0))
                .size(12.0)
                .color(if core.modulation > 0.0 { MOD_COLOR } else { th().dim }),
        );
    }
    let resp = hbar(ui, Vec2::new(w, h), core.modulation, false, MOD_COLOR);
    let resp = resp.on_hover_text("Modulation (vibrato): arraste; fica onde você deixar");
    if !learnable(ui, core, &resp, Target::Modulation) && resp.is_pointer_button_down_on() {
        if let Some(p) = resp.interact_pointer_pos() {
            let v = ((p.x - resp.rect.left()) / resp.rect.width()).clamp(0.0, 1.0);
            core.set_modulation(v);
        }
    }
}

const PITCH_COLOR: Color32 = Color32::from_rgb(0x64, 0xd2, 0xff);
const MOD_COLOR: Color32 = Color32::from_rgb(0xff, 0x9f, 0x0a);

/// Barra deitada. `centered` = enche a partir do meio (pitch); senão, da esquerda (mod).
fn hbar(ui: &mut Ui, size: Vec2, value: f32, centered: bool, color: Color32) -> egui::Response {
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click_and_drag());
    let p = ui.painter();
    let track = rect.shrink2(Vec2::new(2.0, size.y * 0.3));
    p.rect_filled(track, 6, th().glass_hi);
    let v = value.clamp(0.0, 1.0);
    let x = track.left() + v * track.width();
    let from = if centered { track.center().x } else { track.left() };
    let fill = Rect::from_min_max(
        Pos2::new(from.min(x), track.top()),
        Pos2::new(from.max(x), track.bottom()),
    );
    if fill.width() > 0.5 {
        p.rect_filled(fill, 6, color);
    }
    if centered {
        p.line_segment(
            [Pos2::new(track.center().x, rect.top() + 3.0), Pos2::new(track.center().x, rect.bottom() - 3.0)],
            Stroke::new(1.0, th().dim),
        );
    }
    let knob = Pos2::new(x.clamp(rect.left() + 7.0, rect.right() - 7.0), rect.center().y);
    p.circle_filled(knob, size.y * 0.32, Color32::WHITE);
    p.circle_stroke(knob, size.y * 0.32, Stroke::new(1.0, Color32::from_black_alpha(60)));
    resp
}

/// Botão "de segurar" (pitch ▲/▼): enquanto o mouse segura, liga; soltou, volta.
fn hold_button(ui: &mut Ui, core: &mut Core, size: Vec2, label: &str, target: Target, lit: bool) {
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click_and_drag());
    let learning = learnable(ui, core, &resp, target);
    let id = resp.id.with("segurando");
    let was = ui.data(|d| d.get_temp::<bool>(id)).unwrap_or(false);
    let now = !learning && resp.is_pointer_button_down_on();
    if now != was {
        ui.data_mut(|d| d.insert_temp(id, now));
        core.hold(target, now);
    }
    let on = now || lit;
    let p = ui.painter();
    p.rect_filled(rect, 6, if on { PITCH_COLOR } else { th().glass_hi });
    // v0.18: triângulo desenhado (a fonte do programa não tem ▲ ▼ e mostrava um quadradinho)
    let c = rect.center();
    let s = (rect.height().min(rect.width()) * 0.22).max(4.0);
    let up = label.contains('▲');
    let pts = if up {
        vec![c + Vec2::new(0.0, -s), c + Vec2::new(s, s * 0.7), c + Vec2::new(-s, s * 0.7)]
    } else {
        vec![c + Vec2::new(0.0, s), c + Vec2::new(-s, -s * 0.7), c + Vec2::new(s, -s * 0.7)]
    };
    p.add(egui::Shape::convex_polygon(
        pts,
        if on { th().ink } else { th().text },
        Stroke::NONE,
    ));
    resp.on_hover_text(target.label());
}

/// v0.25.1: as duas cores da luz RGB de cada DrumPad (a borda passa de uma para a outra).
/// Kick, Snare, Clap, Hi-hat, Crash, 808, Riser, Impact.
const DRUM_ACCENTS: [(Color32, Color32); 8] = [
    (Color32::from_rgb(0xff, 0x8a, 0x2a), Color32::from_rgb(0xff, 0x3d, 0x8b)),
    (Color32::from_rgb(0x2e, 0xe6, 0xd6), Color32::from_rgb(0x3d, 0x7b, 0xff)),
    (Color32::from_rgb(0xff, 0x4f, 0xa3), Color32::from_rgb(0xa0, 0x5a, 0xff)),
    (Color32::from_rgb(0xff, 0xd2, 0x3f), Color32::from_rgb(0xff, 0x7a, 0x2a)),
    (Color32::from_rgb(0x4d, 0xa3, 0xff), Color32::from_rgb(0x2e, 0xe6, 0xd6)),
    (Color32::from_rgb(0xa0, 0x66, 0xff), Color32::from_rgb(0xff, 0x4f, 0xc8)),
    (Color32::from_rgb(0x3d, 0xff, 0x9a), Color32::from_rgb(0x2e, 0xc8, 0xe6)),
    (Color32::from_rgb(0xff, 0x4b, 0x4b), Color32::from_rgb(0xff, 0x9a, 0x2a)),
];

fn mix_color(a: Color32, b: Color32, t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    let l = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t).round() as u8;
    Color32::from_rgb(l(a.r(), b.r()), l(a.g(), b.g()), l(a.b(), b.b()))
}

fn with_alpha(c: Color32, a: f32) -> Color32 {
    Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), (a.clamp(0.0, 1.0) * 255.0) as u8)
}

/// Pontos em volta de um retângulo de cantos redondos (sentido horário, começando no canto de cima à esquerda).
fn rounded_outline(rect: Rect, r: f32, per_corner: usize) -> Vec<Pos2> {
    let r = r.min(rect.width() / 2.0).min(rect.height() / 2.0);
    let corners = [
        (Pos2::new(rect.left() + r, rect.top() + r), std::f32::consts::PI),
        (Pos2::new(rect.right() - r, rect.top() + r), -std::f32::consts::FRAC_PI_2),
        (Pos2::new(rect.right() - r, rect.bottom() - r), 0.0),
        (Pos2::new(rect.left() + r, rect.bottom() - r), std::f32::consts::FRAC_PI_2),
    ];
    let mut pts = Vec::with_capacity(4 * (per_corner + 1));
    for (c, a0) in corners {
        for k in 0..=per_corner {
            let a = a0 + k as f32 / per_corner as f32 * std::f32::consts::FRAC_PI_2;
            pts.push(c + Vec2::new(a.cos() * r, a.sin() * r));
        }
    }
    pts
}

/// v0.25.1: DrumPad como nas referências do Theo. Parado: preto, borda fininha discreta, nome pequeno no canto.
/// `light` (0..1) = o quanto está aceso (mouse em cima ou batida): acende como luz RGB neon —
/// borda em degradê de duas cores, luz entrando pelas bordas e subindo de baixo, centro escuro e transparente.
/// `spot` = onde o mouse está (v0.25.1: uma luz mais forte segue o mouse por dentro do pad).
fn drum_tile(p: &egui::Painter, rect: Rect, colors: (Color32, Color32), light: f32, spot: Option<Pos2>, i: usize, own: Option<&str>) {
    let (ca, cb) = colors;
    let r = (rect.width() * 0.14).clamp(10.0, 20.0);
    let light = light.clamp(0.0, 1.0);
    // brilho de fora (halo), só quando acende
    if light > 0.01 {
        for k in 1..=7 {
            let f = 1.0 - k as f32 / 8.0;
            p.rect_stroke(
                rect.expand(k as f32 * 1.7),
                (r + k as f32 * 1.7) as u8,
                Stroke::new(1.8, with_alpha(mix_color(ca, cb, 0.5), light * 0.22 * f * f)),
                StrokeKind::Outside,
            );
        }
    }
    // corpo preto (um tiquinho de cor quando acende: "transparente")
    let body = mix_color(Color32::from_rgb(0x05, 0x06, 0x0a), mix_color(ca, cb, 0.5), 0.10 * light);
    p.rect_filled(rect, r as u8, body);
    let clip = p.with_clip_rect(rect.shrink(1.0));
    if light > 0.01 {
        // luz que sobe de baixo (como uma fita de LED atrás do pad)
        let base = Pos2::new(rect.center().x, rect.bottom() + rect.height() * 0.35);
        for k in 0..10 {
            let f = k as f32 / 10.0;
            clip.circle_filled(
                base,
                rect.width() * (0.95 - 0.07 * k as f32),
                with_alpha(mix_color(ca, cb, 0.3 + 0.4 * f), light * 0.045),
            );
        }
        // luz entrando pelas bordas (o centro fica escuro)
        let steps = 9;
        for k in 0..steps {
            let f = 1.0 - k as f32 / steps as f32;
            let inset = k as f32 * (rect.width() * 0.022).max(1.4);
            clip.rect_stroke(
                rect.shrink(inset),
                (r - inset * 0.6).max(2.0) as u8,
                Stroke::new(2.0, with_alpha(mix_color(ca, cb, k as f32 / steps as f32), light * 0.30 * f * f)),
                StrokeKind::Inside,
            );
        }
    }
    // v0.25.1: holofote que segue o mouse (luz mais forte onde o cursor passa)
    if let Some(m) = spot.filter(|m| rect.contains(*m)) {
        let tx = ((m.x - rect.left()) / rect.width() + (m.y - rect.top()) / rect.height()) * 0.5;
        let c = mix_color(ca, cb, tx);
        for k in 0..9 {
            let f = k as f32 / 9.0;
            clip.circle_filled(m, rect.width() * (0.55 - 0.055 * k as f32), with_alpha(mix_color(c, Color32::WHITE, f * 0.5), 0.05 + 0.03 * f));
        }
    }
    // reflexo leve de vidro em cima
    p.line_segment(
        [Pos2::new(rect.left() + r, rect.top() + 1.2), Pos2::new(rect.right() - r, rect.top() + 1.2)],
        Stroke::new(1.0, Color32::from_white_alpha((18.0 + 50.0 * light) as u8)),
    );
    // borda: parada = cinza fininho; acesa = degradê RGB das duas cores
    let pts = rounded_outline(rect.shrink(0.8), r, 8);
    let n = pts.len();
    for k in 0..n {
        let (a, b) = (pts[k], pts[(k + 1) % n]);
        // posição na diagonal (canto de cima à esquerda = cor A, de baixo à direita = cor B)
        let t = (((a.x - rect.left()) / rect.width()) + ((a.y - rect.top()) / rect.height())) * 0.5;
        let neon = mix_color(ca, cb, t);
        let idle = Color32::from_rgb(0x26, 0x2b, 0x36);
        // perto do mouse a borda brilha mais (a luz "escorre" pela borda)
        let near = spot
            .map(|m| (1.0 - (a.distance(m) / (rect.width() * 0.6))).clamp(0.0, 1.0))
            .unwrap_or(0.0);
        let c = mix_color(mix_color(idle, neon, 0.25 + 0.75 * light), Color32::WHITE, near * near * 0.55);
        p.line_segment([a, b], Stroke::new(1.2 + 1.0 * light + 1.2 * near, c));
    }
    // nome pequeno no canto de cima (como no sampler)
    let pad_in = (rect.width() * 0.10).clamp(8.0, 13.0);
    let font = (rect.width() * 0.105).clamp(10.0, 13.0);
    let name = crate::efeitos::DRUM_NAMES[i % crate::efeitos::DRUMS];
    let txt = mix_color(Color32::from_rgb(0x8a, 0x92, 0xa3), Color32::WHITE, light);
    match own {
        Some(file) => {
            let short: String = file.chars().take(14).collect();
            p.text(rect.min + Vec2::new(pad_in, pad_in), Align2::LEFT_TOP, name.to_uppercase(), FontId::proportional(font), txt);
            p.text(
                Pos2::new(rect.left() + pad_in, rect.bottom() - pad_in),
                Align2::LEFT_BOTTOM,
                format!("♪ {short}"),
                FontId::proportional(font - 1.0),
                mix_color(ca, Color32::WHITE, 0.3),
            );
        }
        None => {
            p.text(rect.min + Vec2::new(pad_in, pad_in), Align2::LEFT_TOP, name.to_uppercase(), FontId::proportional(font), txt);
        }
    }
}

/// v0.21: bolinha com um planetinha desenhado (abre/fecha a lista de pads).
fn planet_ball(ui: &mut Ui, open: bool) -> egui::Response {
    let (rect, resp) = ui.allocate_exact_size(Vec2::splat(40.0), Sense::click());
    let p = ui.painter();
    let c = rect.center();
    let bg = if open {
        PAD_COLOR
    } else if resp.hovered() {
        th().glass_hi.gamma_multiply(1.3)
    } else {
        th().glass_hi
    };
    p.circle_filled(c, 19.0, bg);
    let fg = if open { th().ink } else { PAD_COLOR };
    // planeta
    p.circle_filled(c, 7.5, fg);
    // anel (elipse inclinada)
    let pts: Vec<Pos2> = (0..=40)
        .map(|k| {
            let a = k as f32 / 40.0 * std::f32::consts::TAU;
            let (x, y) = (a.cos() * 14.0, a.sin() * 4.5);
            let (s, co) = (0.35f32.sin(), 0.35f32.cos());
            c + Vec2::new(x * co - y * s, x * s + y * co)
        })
        .collect();
    p.add(egui::Shape::line(pts, Stroke::new(1.8, fg)));
    resp
}

/// v0.18: botão ⚙ que abre/fecha os ajustes finos de uma tela ou painel.
fn details_button(ui: &mut Ui, open: bool, size: Vec2, hint: &str) -> egui::Response {
    let b = egui::Button::new(
        RichText::new("⚙")
            .size(18.0)
            .color(if open { th().ink } else { th().text }),
    )
    .fill(if open { th().text.gamma_multiply(0.85) } else { th().glass_hi });
    ui.add_sized(size, b)
        .on_hover_text(if open { format!("Fechar: {hint}") } else { hint.to_string() })
}

/// Quadro de ajuda: escala, pentatônica e acordes do tom do pad (para quem está começando).
fn scale_panel(ui: &mut Ui, core: &mut Core, st: &mut UiState, size: Vec2) {
    use crate::teoria;
    let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
    theme::glass(ui.painter(), rect, 18, false, None);
    let inner = rect.shrink2(Vec2::new(16.0, 10.0));
    let mut child = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(inner)
            .layout(Layout::top_down(Align::Min)),
    );
    let ui = &mut child;
    let Some(key) = core.pad_active else {
        ui.add_space(14.0);
        ui.label(RichText::new("🎵 Toque um tom do pad e a escala aparece aqui (notas, pentatônica e acordes do tom).").size(16.0).color(th().dim));
        ui.add_space(6.0);
        ui.label(RichText::new("Clique num tom para tocar · clique de novo para parar · trocar de tom faz a passagem suave").size(13.0).color(th().dim));
        return;
    };
    let minor = core.cfg.scale_minor;
    let scale = teoria::scale(key, minor);
    let penta = teoria::pentatonic(key, minor);
    let chords = teoria::chords(key, minor);
    let kb_w = 210.0;

    ui.horizontal(|ui| {
        ui.label(
            RichText::new(format!("Tom: {}", teoria::key_name(key, minor)))
                .size(22.0)
                .strong(),
        );
        if pads::KEY_NAMES[key].contains('#') && !teoria::key_name(key, minor).contains('#') {
            ui.label(
                RichText::new(format!("(= {} no botão)", pads::KEY_NAMES[key]))
                    .size(12.0)
                    .color(th().dim),
            );
        }
        ui.add_space(10.0);
        for (lbl, m) in [("Maior", false), ("Menor", true)] {
            let sel = minor == m;
            let b =
                egui::Button::new(RichText::new(lbl).color(if sel { th().ink } else { th().text }))
                    .fill(if sel { PAD_COLOR } else { th().glass_hi });
            if ui.add(b).clicked() && !sel {
                core.cfg.scale_minor = m;
                core.dirty = true;
            }
        }
        ui.label(
            RichText::new("o pad serve para os dois")
                .size(12.0)
                .color(th().dim),
        );
    });
    ui.add_space(4.0);
    let text_w = (inner.width() - kb_w - 16.0).max(200.0);
    ui.horizontal_top(|ui| {
        ui.vertical(|ui| {
            ui.set_width(text_w);
            // Escala: uma "pastilha" por nota, com o nome em português embaixo.
            ui.horizontal(|ui| {
                ui.add_sized(
                    [96.0, 40.0],
                    egui::Label::new(RichText::new("Escala").color(th().dim)),
                );
                for (i, n) in scale.iter().enumerate() {
                    let (r, _) = ui.allocate_exact_size(Vec2::new(52.0, 42.0), Sense::hover());
                    let p = ui.painter();
                    p.rect_filled(
                        r,
                        10,
                        if i == 0 {
                            PAD_COLOR.gamma_multiply(0.85)
                        } else {
                            th().glass_hi
                        },
                    );
                    p.text(
                        r.center() - Vec2::new(0.0, 7.0),
                        Align2::CENTER_CENTER,
                        &n.name,
                        FontId::proportional(18.0),
                        if i == 0 { th().ink } else { th().text },
                    );
                    p.text(
                        r.center() + Vec2::new(0.0, 12.0),
                        Align2::CENTER_CENTER,
                        &n.solfejo,
                        FontId::proportional(11.0),
                        if i == 0 { th().ink } else { th().dim },
                    );
                }
            });
            ui.horizontal(|ui| {
                ui.add_sized(
                    [96.0, 22.0],
                    egui::Label::new(RichText::new("Pentatônica").color(th().dim)),
                );
                let t = penta
                    .iter()
                    .map(|n| n.name.as_str())
                    .collect::<Vec<_>>()
                    .join("   ");
                ui.label(RichText::new(t).size(17.0).strong());
            });
            ui.horizontal(|ui| {
                ui.add_sized(
                    [96.0, 22.0],
                    egui::Label::new(RichText::new("Acordes do tom").color(th().dim)),
                );
                ui.label(
                    RichText::new(chords.join("   "))
                        .size(17.0)
                        .color(LAYER_COLORS[0]),
                );
            });
            // Sequências: uma por vez, discreta; ◀ ▶ troca.
            let progs = teoria::progressions(key, minor);
            let n = progs.len();
            let pr = &progs[st.prog_idx % n];
            ui.horizontal(|ui| {
                ui.spacing_mut().button_padding = Vec2::new(6.0, 1.0);
                ui.add_sized(
                    [96.0, 22.0],
                    egui::Label::new(RichText::new("Sequência").color(th().dim)),
                );
                if ui.small_button("◀").clicked() {
                    st.prog_idx = (st.prog_idx + n - 1) % n;
                }
                ui.label(RichText::new(&pr.chords).size(15.0).strong());
                ui.label(
                    RichText::new(format!("({})  {}", pr.numbers, pr.name))
                        .size(12.0)
                        .color(th().dim),
                );
                if ui
                    .small_button("▶")
                    .on_hover_text("Próxima sequência")
                    .clicked()
                {
                    st.prog_idx = (st.prog_idx + 1) % n;
                }
                ui.label(
                    RichText::new(format!("{}/{}", st.prog_idx % n + 1, n))
                        .size(11.0)
                        .color(th().dim),
                );
            });
        });
        // Tecladinho de uma oitava com as notas da escala marcadas.
        let (r, _) = ui.allocate_exact_size(Vec2::new(kb_w, 96.0), Sense::hover());
        mini_keyboard(
            ui.painter(),
            r,
            |pc| teoria::in_scale(key, minor, pc),
            pads::key_midi(key) % 12,
        );
    });
}

/// Uma oitava de teclado (Dó a Si) com as notas `mark(pc)` acesas; `tonic` em destaque.
fn mini_keyboard(p: &egui::Painter, r: Rect, mark: impl Fn(u8) -> bool, tonic: u8) {
    let whites: [u8; 7] = [0, 2, 4, 5, 7, 9, 11];
    let ww = r.width() / 7.0;
    for (i, &pc) in whites.iter().enumerate() {
        let k = Rect::from_min_size(
            Pos2::new(r.left() + i as f32 * ww, r.top()),
            Vec2::new(ww - 1.0, r.height()),
        );
        let on = mark(pc);
        let fill = if pc == tonic {
            PAD_COLOR
        } else if on {
            Color32::from_rgb(0xd9, 0xcc, 0xff)
        } else {
            Color32::from_rgb(0xe8, 0xea, 0xee)
        };
        p.rect_filled(
            k,
            CornerRadius {
                nw: 0,
                ne: 0,
                sw: 4,
                se: 4,
            },
            if on || pc == tonic {
                fill
            } else {
                fill.gamma_multiply(0.55)
            },
        );
        if on {
            p.circle_filled(
                Pos2::new(k.center().x, k.bottom() - 10.0),
                4.0,
                if pc == tonic {
                    Color32::WHITE
                } else {
                    PAD_COLOR
                },
            );
        }
    }
    for (after, pc) in [(0usize, 1u8), (1, 3), (3, 6), (4, 8), (5, 10)] {
        let x = r.left() + (after as f32 + 1.0) * ww - ww * 0.3;
        let k = Rect::from_min_size(Pos2::new(x, r.top()), Vec2::new(ww * 0.6, r.height() * 0.6));
        let on = mark(pc);
        let fill = if pc == tonic {
            PAD_COLOR
        } else if on {
            Color32::from_rgb(0x6d, 0x55, 0xc8)
        } else {
            Color32::from_rgb(0x18, 0x1a, 0x20)
        };
        p.rect_filled(
            k,
            CornerRadius {
                nw: 0,
                ne: 0,
                sw: 3,
                se: 3,
            },
            fill,
        );
        if on {
            p.circle_filled(
                Pos2::new(k.center().x, k.bottom() - 8.0),
                3.5,
                Color32::WHITE,
            );
        }
    }
}

fn footer(ui: &mut Ui, core: &mut Core, stats: &Stats) {
    ui.horizontal(|ui| {
        let toast = core.toast.as_ref().filter(|(_, t)| t.elapsed() < Duration::from_secs(7)).map(|(m, _)| m.clone());
        match toast {
            Some(m) => {
                ui.label(RichText::new(m).color(ORANGE));
            }
            None => match core.pad_active.filter(|_| !core.cfg.is_hidden("scale_help")) {
                // Pad tocando: lembrete da escala sempre à vista, em qualquer aba.
                Some(k) => {
                    use crate::teoria;
                    let m = core.cfg.scale_minor;
                    let names = |v: Vec<teoria::Note>| v.iter().map(|n| n.name.clone()).collect::<Vec<_>>().join(" ");
                    ui.label(RichText::new(format!("🎵 Tom {}", teoria::key_name(k, m))).strong().color(PAD_COLOR));
                    ui.label(RichText::new(format!("Escala: {}", names(teoria::scale(k, m)))).size(14.0));
                    ui.label(RichText::new(format!("·  Pentatônica: {}", names(teoria::pentatonic(k, m)))).size(14.0).color(th().dim));
                    ui.label(RichText::new(format!("·  Acordes: {}", teoria::chords(k, m).join(" "))).size(14.0).color(th().dim));
                }
                // v0.18: sem texto fixo (os atalhos aparecem ao passar o mouse nas cenas e no tutorial)
                None => {}
            },
        }
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            let peak = stats.peak_milli.load(Ordering::Relaxed) as f32 / 1000.0;
            meter(ui, peak).on_hover_text(format!(
                "Volume de saída · {} vozes tocando",
                stats.voices.load(Ordering::Relaxed)
            ));
            let over = stats.overruns.load(Ordering::Relaxed);
            if over > 0 {
                ui.label(RichText::new(format!("⚠ {over} engasgos")).size(13.0).color(YELLOW))
                    .on_hover_text("O computador não deu conta a tempo. Aumente o buffer em Ajustes ou feche outros programas.");
            }
        });
    });
}

fn meter(ui: &mut Ui, level: f32) -> egui::Response {
    let (r, resp) = ui.allocate_exact_size(Vec2::new(140.0, 10.0), Sense::hover());
    let p = ui.painter();
    p.rect_filled(r, 3, th().glass_hi);
    let w = r.width() * level.clamp(0.0, 1.0);
    let c = if level < 0.7 {
        GREEN
    } else if level < 0.95 {
        YELLOW
    } else {
        RED
    };
    p.rect_filled(Rect::from_min_size(r.min, Vec2::new(w, r.height())), 3, c);
    resp
}

// ---------- Controles desenhados ----------

/// Modo aprender: marca o controle e, se clicado, vira o alvo. Devolve true se está no modo aprender.
fn learnable(ui: &Ui, core: &mut Core, resp: &egui::Response, target: Target) -> bool {
    if !core.learn.active {
        return false;
    }
    learn_outline(ui, resp, core, target);
    if resp.clicked() || resp.drag_started() {
        core.learn.target = Some(target);
    }
    true
}

fn learn_outline(ui: &Ui, resp: &egui::Response, core: &Core, target: Target) {
    let selected = core.learn.target == Some(target);
    let linked = core
        .cfg
        .mappings
        .iter()
        .chain(core.cfg.piano_mappings.iter())
        .any(|m| m.target == target);
    let color = if selected {
        ORANGE
    } else if linked {
        ORANGE.gamma_multiply(0.45)
    } else {
        th().dim.gamma_multiply(0.5)
    };
    let p = ui.painter();
    if selected {
        p.rect_filled(resp.rect.expand(3.0), 6, ORANGE.gamma_multiply(0.25));
    }
    p.rect_stroke(
        resp.rect.expand(3.0),
        6,
        Stroke::new(if selected { 3.0 } else { 1.5 }, color),
        StrokeKind::Outside,
    );
}

fn fader(
    ui: &mut Ui,
    size: Vec2,
    value: f32,
    color: Color32,
    learning: bool,
) -> (egui::Response, Option<f32>) {
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click_and_drag());
    let cap_h = 26.0;
    let top = rect.top() + cap_h / 2.0;
    let bottom = rect.bottom() - cap_h / 2.0;
    let travel = (bottom - top).max(1.0);
    let mut new = None;
    if !learning {
        if resp.dragged() {
            let dy = resp.drag_delta().y;
            if dy != 0.0 {
                new = Some((value - dy / travel).clamp(0.0, 1.0));
            }
        }
        if resp.hovered() {
            let scroll = ui.input(|i| i.smooth_scroll_delta.y);
            if scroll != 0.0 {
                new = Some((new.unwrap_or(value) + scroll / 600.0).clamp(0.0, 1.0));
            }
        }
        if resp.double_clicked() {
            new = Some(0.8);
        }
    }
    let target = new.unwrap_or(value);
    // Desliza macio até o valor (ex.: quando a mesinha pula de posição).
    let v = if resp.dragged() {
        ui.ctx().animate_value_with_time(resp.id, target, 0.0)
    } else {
        ui.ctx().animate_value_with_time(resp.id, target, 0.12)
    };
    let p = ui.painter();
    let cx = rect.center().x;
    // marcas
    for k in 0..=10 {
        let y = bottom - travel * k as f32 / 10.0;
        let w = if k % 5 == 0 { 10.0 } else { 5.0 };
        let c = th().dim.gamma_multiply(if k % 5 == 0 { 0.55 } else { 0.3 });
        p.line_segment(
            [Pos2::new(cx - 16.0 - w, y), Pos2::new(cx - 16.0, y)],
            Stroke::new(1.0, c),
        );
        p.line_segment(
            [Pos2::new(cx + 16.0, y), Pos2::new(cx + 16.0 + w, y)],
            Stroke::new(1.0, c),
        );
    }
    // trilho de vidro
    let track = Rect::from_min_max(Pos2::new(cx - 3.0, top), Pos2::new(cx + 3.0, bottom));
    p.rect_filled(track, 3, Color32::from_black_alpha(110));
    p.rect_stroke(track, 3, Stroke::new(1.0, th().edge), StrokeKind::Outside);
    let y = bottom - travel * v;
    let fill = Rect::from_min_max(Pos2::new(cx - 3.0, y), Pos2::new(cx + 3.0, bottom));
    p.rect_filled(fill, 3, color);
    // brilho da parte cheia
    p.rect_filled(
        fill.expand2(Vec2::new(3.0, 0.0)),
        6,
        color.gamma_multiply(0.18),
    );
    // botão do fader: "pílula" branca de vidro com sombra (estilo iPhone)
    let cap_w = (rect.width() * 0.58).clamp(34.0, 60.0);
    let cap = Rect::from_center_size(Pos2::new(cx, y), Vec2::new(cap_w, cap_h - 4.0));
    let hot = resp.dragged() || resp.hovered();
    let shadow = egui::epaint::Shadow {
        offset: [0, 3],
        blur: 10,
        spread: 0,
        color: Color32::from_black_alpha(110),
    };
    p.add(shadow.as_shape(cap, CornerRadius::same(11)));
    p.rect_filled(
        cap,
        11,
        if hot {
            Color32::from_rgb(0xff, 0xff, 0xff)
        } else {
            Color32::from_rgb(0xec, 0xee, 0xf3)
        },
    );
    p.rect_stroke(
        cap,
        11,
        Stroke::new(1.0, Color32::from_white_alpha(200)),
        StrokeKind::Inside,
    );
    p.line_segment(
        [
            Pos2::new(cap.left() + 8.0, y),
            Pos2::new(cap.right() - 8.0, y),
        ],
        Stroke::new(2.0, color),
    );
    (resp, new)
}

fn knob(
    ui: &mut Ui,
    d: f32,
    value: f32,
    color: Color32,
    learning: bool,
) -> (egui::Response, Option<f32>) {
    let (rect, resp) = ui.allocate_exact_size(Vec2::splat(d), Sense::click_and_drag());
    let mut new = None;
    if !learning {
        if resp.dragged() {
            let delta = resp.drag_delta();
            let dv = (delta.x - delta.y) / 160.0;
            if dv != 0.0 {
                new = Some((value + dv).clamp(0.0, 1.0));
            }
        }
        if resp.hovered() {
            let scroll = ui.input(|i| i.smooth_scroll_delta.y);
            if scroll != 0.0 {
                new = Some((new.unwrap_or(value) + scroll / 600.0).clamp(0.0, 1.0));
            }
        }
    }
    let v = ui.ctx().animate_value_with_time(
        resp.id,
        new.unwrap_or(value),
        if resp.dragged() { 0.0 } else { 0.12 },
    );
    let p = ui.painter();
    let c = rect.center();
    let r = d / 2.0 - 3.0;
    // knob de vidro com sombra
    let shadow = egui::epaint::Shadow {
        offset: [0, 3],
        blur: 12,
        spread: 0,
        color: Color32::from_black_alpha(90),
    };
    p.add(shadow.as_shape(
        Rect::from_center_size(c, Vec2::splat(r * 1.6)),
        CornerRadius::same((r * 0.8) as u8),
    ));
    p.circle_filled(c, r, th().glass_hi);
    p.circle_filled(
        c,
        r * 0.62,
        Color32::from_white_alpha(if resp.hovered() { 40 } else { 22 }),
    );
    p.circle_stroke(c, r, Stroke::new(1.0, th().edge));
    let start = std::f32::consts::PI * 0.75;
    let sweep = std::f32::consts::PI * 1.5;
    let arc = |from: f32, to: f32| -> Vec<Pos2> {
        let n = 32;
        (0..=n)
            .map(|k| {
                let a = start + sweep * (from + (to - from) * k as f32 / n as f32);
                Pos2::new(c.x + a.cos() * r, c.y + a.sin() * r)
            })
            .collect()
    };
    p.add(Shape::line(
        arc(0.0, 1.0),
        Stroke::new(3.0, Color32::from_black_alpha(110)),
    ));
    if v > 0.005 {
        p.add(Shape::line(arc(0.0, v), Stroke::new(3.0, color)));
    }
    let a = start + sweep * v;
    p.line_segment(
        [
            Pos2::new(c.x + a.cos() * (r * 0.25), c.y + a.sin() * (r * 0.25)),
            Pos2::new(c.x + a.cos() * (r - 6.0), c.y + a.sin() * (r - 6.0)),
        ],
        Stroke::new(2.5, Color32::WHITE),
    );
    (resp, new)
}

// ---------- Janela de timbres ----------

fn preset_picker(ctx: &egui::Context, core: &mut Core, st: &mut UiState) {
    let Some(layer) = st.picker else { return };
    let mut open = true;
    let mut chosen: Option<PresetRef> = None;
    egui::Window::new(format!("Timbre da camada {}", layer + 1))
        .open(&mut open)
        .collapsible(false)
        .default_size([460.0, 520.0])
        .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
        .show(ctx, |ui| {
            if core.presets.is_empty() {
                ui.label("Abra um arquivo SF2 primeiro (botão 📂 lá em cima).");
                return;
            }
            let r = ui.add(
                egui::TextEdit::singleline(&mut st.picker_search)
                    .hint_text("🔍 Procurar (ex.: piano, pad, órgão)")
                    .desired_width(f32::INFINITY),
            );
            if st.picker_focus {
                r.request_focus();
                st.picker_focus = false;
            }
            let q = st.picker_search.to_lowercase();
            let list: Vec<&PresetRef> = core
                .presets
                .iter()
                .filter(|p| q.is_empty() || p.name.to_lowercase().contains(&q))
                .collect();
            let current = core.cfg.mix.layers[layer].preset.clone();
            if ui.input(|i| i.key_pressed(Key::Enter)) {
                if let Some(p) = list.first() {
                    chosen = Some((*p).clone());
                }
            }
            ui.label(
                RichText::new(format!("{} timbres", list.len()))
                    .size(12.0)
                    .color(th().dim),
            );
            let row_h = 30.0;
            egui::ScrollArea::vertical().auto_shrink(false).show_rows(
                ui,
                row_h,
                list.len(),
                |ui, range| {
                    for p in &list[range] {
                        let sel = current
                            .as_ref()
                            .map(|c| c.bank == p.bank && c.patch == p.patch)
                            .unwrap_or(false);
                        let text =
                            RichText::new(format!("{:03}:{:03}   {}", p.bank, p.patch + 1, p.name))
                                .size(15.0);
                        let btn = egui::Button::selectable(sel, text)
                            .min_size(Vec2::new(ui.available_width(), row_h - 4.0));
                        let r = ui.add(btn);
                        if r.clicked() {
                            chosen = Some((*p).clone());
                        }
                    }
                },
            );
        });
    if let Some(p) = chosen {
        core.set_preset(layer, p);
        if !core.cfg.mix.layers[layer].on {
            core.set_on(layer, true);
        }
        st.picker = None;
    } else if !open || ctx.input(|i| i.key_pressed(Key::Escape)) {
        st.picker = None;
    }
}

// ---------- Ajustes ----------

#[allow(clippy::too_many_arguments)]
fn settings_window(
    ctx: &egui::Context,
    core: &mut Core,
    st: &mut UiState,
    audio: &Audio,
    midi_status: &SharedMidiStatus,
    rescan: &AtomicBool,
    restart_audio: &mut bool,
    config_path: &std::path::Path,
) {
    if !st.settings {
        return;
    }
    let mut open = true;
    egui::Window::new("⚙ Ajustes")
        .open(&mut open)
        .collapsible(false)
        .default_size([560.0, 560.0])
        .show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.heading("Aparência");
                let mut look = core.cfg.look.clone();
                ui.horizontal_wrapped(|ui| {
                    ui.label("Fundo:");
                    for (k, pal) in theme::PALETTES.iter().enumerate() {
                        let sel = look.palette == k;
                        let b = egui::Button::new(RichText::new(pal.name).color(if sel { th().ink } else { th().text }))
                            .fill(if sel { Color32::from_rgb(0x0a, 0x84, 0xff) } else { th().glass });
                        if ui.add(b).clicked() {
                            look.palette = k;
                        }
                    }
                });
                egui::Grid::new("aparencia").num_columns(2).spacing([12.0, 8.0]).show(ui, |ui| {
                    ui.label("Vidro (transparente ↔ fosco)");
                    ui.add(egui::Slider::new(&mut look.frost, 0.0..=1.0).show_value(false));
                    ui.end_row();
                    ui.label("Brilho em volta do que está ligado");
                    ui.add(egui::Slider::new(&mut look.glow, 0.0..=1.0).show_value(false));
                    ui.end_row();
                    ui.label("Tamanho da tela");
                    ui.add(egui::Slider::new(&mut look.scale, 0.8..=1.3).custom_formatter(|v, _| format!("{:.0}%", v * 100.0)));
                    ui.end_row();
                });
                ui.horizontal(|ui| {
                    ui.checkbox(&mut look.motion, "Fundo em movimento");
                    ui.checkbox(&mut look.stars, "Estrelas");
                });
                ui.label(
                    RichText::new("Dica: no culto, se o computador ficar pesado, desligue \"Fundo em movimento\".")
                        .size(13.0)
                        .color(th().dim),
                );
                if look != core.cfg.look {
                    core.cfg.look = look;
                    core.dirty = true;
                }
                if ui.button("↺ Visual padrão").clicked() {
                    core.cfg.look = Appearance::default();
                    core.dirty = true;
                }

                ui.separator();
                ui.heading("Som");
                let mut v = core.cfg.vivo.clone();
                ui.horizontal(|ui| {
                    ui.label("Reverb de igreja nas camadas do SF2");
                    ui.add(egui::Slider::new(&mut v.church_sf2, 0.0..=1.0).show_value(false));
                });
                ui.label(RichText::new("O mesmo reverb dos pads (igreja grande). Use pouco: 10% a 30% já deixa o piano mais bonito.").size(12.0).color(th().dim));
                core.set_vivo(v);
                egui::Grid::new("som").num_columns(2).spacing([12.0, 8.0]).show(ui, |ui| {
                    ui.label("Saída");
                    let cur = core.cfg.audio_device.clone().unwrap_or_else(|| "Padrão do Windows".into());
                    egui::ComboBox::from_id_salt("dev").width(320.0).selected_text(cur).show_ui(ui, |ui| {
                        if ui.selectable_label(core.cfg.audio_device.is_none(), "Padrão do Windows").clicked() {
                            core.cfg.audio_device = None;
                            core.dirty = true;
                            *restart_audio = true;
                        }
                        for d in &st.audio_devices {
                            if ui.selectable_label(core.cfg.audio_device.as_deref() == Some(d), d).clicked() {
                                core.cfg.audio_device = Some(d.clone());
                                core.dirty = true;
                                *restart_audio = true;
                            }
                        }
                    });
                    ui.end_row();

                    ui.label("Buffer (atraso)");
                    let sr = audio.info.as_ref().map(|i| i.sample_rate).unwrap_or(48_000) as f32;
                    let fmt = |b: u32| if b == 0 { "Padrão".to_string() } else { format!("{b} ({:.1} ms)", b as f32 * 1000.0 / sr) };
                    egui::ComboBox::from_id_salt("buf").selected_text(fmt(core.cfg.buffer_frames)).show_ui(ui, |ui| {
                        for b in BUFFERS {
                            if ui.selectable_label(core.cfg.buffer_frames == b, fmt(b)).clicked() {
                                core.cfg.buffer_frames = b;
                                core.dirty = true;
                                *restart_audio = true;
                            }
                        }
                    });
                    ui.end_row();

                    ui.label("Vozes ao mesmo tempo");
                    egui::ComboBox::from_id_salt("poly").selected_text(core.cfg.polyphony.to_string()).show_ui(ui, |ui| {
                        for p in POLYPHONY {
                            if ui.selectable_label(core.cfg.polyphony == p, p.to_string()).clicked() {
                                core.cfg.polyphony = p;
                                core.dirty = true;
                                core.rebuild_synth();
                            }
                        }
                    });
                    ui.end_row();
                });
                ui.label(RichText::new("Menos atraso pesa mais no computador. Se ouvir estalos, aumente o buffer.").size(13.0).color(th().dim));
                if let Some(e) = &audio.error {
                    ui.label(RichText::new(format!("⚠ {e}")).color(RED));
                }

                ui.separator();
                ui.heading("Aparelhos MIDI");
                let ports = midi_status.lock().map(|s| s.all_ports()).unwrap_or_default();
                if ui.button("🔍 Procurar mesinha Bluetooth...").clicked() {
                    st.bt_open = true;
                }
                if ports.is_empty() {
                    ui.label(RichText::new("Nenhum aparelho encontrado. Ligue o piano e a mesinha pelo cabo USB ou pelo Bluetooth.").color(th().dim));
                }
                egui::Grid::new("midi").num_columns(3).spacing([12.0, 8.0]).show(ui, |ui| {
                    for p in &ports {
                        let c = if p.connected { GREEN } else { RED };
                        let (r, _) = ui.allocate_exact_size(Vec2::splat(14.0), Sense::hover());
                        ui.painter().circle_filled(r.center(), 5.0, c);
                        ui.vertical(|ui| {
                            ui.label(&p.name);
                            if let Some(e) = &p.error {
                                ui.label(RichText::new(format!("⚠ {e}")).size(12.0).color(YELLOW));
                            }
                        });
                        egui::ComboBox::from_id_salt(&p.name).selected_text(p.role.label()).show_ui(ui, |ui| {
                            for role in [PortRole::Piano, PortRole::Controller, PortRole::Ignore] {
                                if ui.selectable_label(p.role == role, role.label()).clicked() {
                                    core.cfg.set_role(&p.name, role);
                                    core.dirty = true;
                                    rescan.store(true, Ordering::Relaxed);
                                }
                            }
                        });
                        ui.end_row();
                    }
                });
                if ports.iter().any(|p| p.error.is_some()) {
                    ui.label(RichText::new("⚠ Algum aparelho está em uso por outro programa (feche o Numa e outros).").color(YELLOW));
                }
                ui.label(
                    RichText::new("Bluetooth: primeiro ligue o Bluetooth da mesinha e pareie no Windows (Configurações → Dispositivos → Bluetooth e outros → Adicionar → Bluetooth). Depois ela aparece aqui sozinha, com \"(Bluetooth)\" no nome.")
                        .size(13.0)
                        .color(th().dim),
                );
                let seen = midi_status.lock().map(|s| s.winrt_seen.clone()).unwrap_or_default();
                egui::CollapsingHeader::new(format!("Diagnóstico (o que o Windows enxerga: {})", seen.len())).show(ui, |ui| {
                    if seen.is_empty() {
                        ui.label(RichText::new("O Windows não mostrou nenhum aparelho MIDI pela API nova. Se a mesinha está no Bluetooth, confira se ela aparece como \"Pareado\"/\"Conectado\" nas configurações de Bluetooth do Windows.").size(13.0).color(th().dim));
                    }
                    for (name, id, used) in &seen {
                        ui.label(RichText::new(format!("{} {name}", if *used { "✔ usando" } else { "– já veio pelo cabo" })).size(13.0));
                        ui.label(RichText::new(id).size(10.0).color(th().dim));
                    }
                    if ui.button("Procurar aparelhos de novo").clicked() {
                        rescan.store(true, Ordering::Relaxed);
                    }
                });
                if ui.checkbox(&mut core.cfg.program_change_scenes, "Troca de programa do piano muda a cena (programa 1 = Cena 1)").changed() {
                    core.dirty = true;
                }

                ui.separator();
                ui.heading("Controles da mesinha");
                ui.horizontal(|ui| {
                    if ui.button("↺ Restaurar padrão da SMC-Mixer").clicked() {
                        core.reset_mappings();
                    }
                    if ui.button("Configurar mesinha (automático)").clicked() {
                        core.wizard_start();
                        st.settings = false;
                    }
                    if ui.button("Aprender").clicked() {
                        core.learn.active = true;
                        core.learn.target = None;
                    }
                });
                egui::CollapsingHeader::new(format!("Ver ligações ({})", core.cfg.mappings.len())).show(ui, |ui| {
                    let mut remove = None;
                    egui::Grid::new("maps").num_columns(3).striped(true).show(ui, |ui| {
                        for (k, m) in core.cfg.mappings.iter().enumerate() {
                            ui.label(m.target.label());
                            ui.label(RichText::new(m.source.label()).color(th().dim));
                            if ui.small_button("✖").on_hover_text("Remover").clicked() {
                                remove = Some(k);
                            }
                            ui.end_row();
                        }
                    });
                    if let Some(k) = remove {
                        core.cfg.mappings.remove(k);
                        core.dirty = true;
                    }
                });

                // v0.15: controles do próprio teclado (ex.: AMW MIDI Control Mini 32).
                ui.separator();
                ui.heading("Controles do teclado (pitch, modulation...)");
                ui.label(
                    RichText::new(
                        "Clique em Aprender, clique no Pitch, ▲, ▼ ou Mod (coluna GERAL) e aperte/mexa o botão do teclado. \
                         O que for ligado aqui não toca nota.",
                    )
                    .size(12.0)
                    .color(th().dim),
                );
                ui.horizontal(|ui| {
                    ui.label("Alcance do pitch:");
                    let mut r = core.cfg.pitch_range as i32;
                    if ui
                        .add(egui::Slider::new(&mut r, 1..=12).suffix(" semitons"))
                        .changed()
                    {
                        core.set_pitch_range(r as u8);
                    }
                    if ui.button("Aprender").clicked() {
                        core.learn.active = true;
                        core.learn.target = None;
                        st.settings = false;
                    }
                });
                let mut remove = None;
                if core.cfg.piano_mappings.is_empty() {
                    ui.label(RichText::new("Nenhum controle do teclado ligado ainda.").color(th().dim));
                } else {
                    egui::Grid::new("maps_teclado").num_columns(3).striped(true).show(ui, |ui| {
                        for (k, m) in core.cfg.piano_mappings.iter().enumerate() {
                            ui.label(m.target.label());
                            ui.label(RichText::new(m.source.label()).color(th().dim));
                            if ui.small_button("✖").on_hover_text("Remover").clicked() {
                                remove = Some(k);
                            }
                            ui.end_row();
                        }
                    });
                }
                if let Some(k) = remove {
                    core.remove_piano_mapping(k);
                }

                ui.separator();
                ui.label(RichText::new(format!("Configurações salvas em: {}", config_path.display())).size(12.0).color(th().dim));
                ui.label(RichText::new("NoahPad — o pad do Noah · motor Piano SF2 v0.22").size(12.0).color(th().dim));
                // v0.20: créditos dos sons que vêm no programa (a licença MIT pede)
                ui.label(
                    RichText::new(
                        "Bateria, coral do Voice FX e os pads Coral Celeste, Cordas Suaves e Atmosfera: \
                         MuseScore General (licença MIT) — FluidR3 © Frank Wen 2000-2002, FluidR3Mono © Michael Cowgill, \
                         MuseScore General © S. Christian Collins.",
                    )
                    .size(11.0)
                    .color(th().dim),
                );
            });
        });
    if !open {
        st.settings = false;
    }
}
