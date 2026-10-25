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
                vivo_root_key: 0,
                prog_idx: 0,
                kb_open: false,
                kb_mouse: None,
                kb_keys: Vec::new(),
                kb_octave: 4,
                kb_sustain: false,
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
        if core.cfg.tab == 0 && self.ui.kb_open {
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
                if core.cfg.tab == 0 {
                    mixer(ui, &mut core, &mut self.ui);
                } else {
                    pads_page(ui, &mut core, &mut self.ui);
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
        drop(core);
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
    if st.kb_open && core.cfg.tab == 0 {
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
        ui.label(RichText::new("🎹 Teclado de teste").strong());
        ui.label(
            RichText::new("clique nas teclas (mais embaixo = mais forte) · computador: A W S E D F T G Y H U J K = notas · Z / X = oitava · espaço = pedal")
                .size(13.0)
                .color(th().dim),
        );
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
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
        ui.label(RichText::new("🎹 Piano SF2").size(20.0).strong());
        ui.add_space(12.0);

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
            let label = core
                .sf2_name
                .clone()
                .unwrap_or_else(|| "Abrir SF2...".into());
            let btn = egui::Button::new(RichText::new(format!("📂 {label}")).size(15.0)).fill(
                if core.sf2_name.is_some() {
                    th().glass_hi
                } else {
                    ORANGE.gamma_multiply(0.6)
                },
            );
            if ui
                .add(btn)
                .on_hover_text("Escolher o arquivo .sf2")
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
            if ui
                .add(egui::Button::new("⚙ Ajustes").fill(th().glass_hi))
                .clicked()
            {
                st.settings = !st.settings;
                st.audio_devices = audio::device_names();
            }
            let edit = egui::Button::new(
                RichText::new(if st.editing {
                    "✔ Concluir"
                } else {
                    "✏ Editar"
                })
                .color(if st.editing { th().ink } else { th().text }),
            )
            .fill(if st.editing {
                Color32::from_rgb(0x0a, 0x84, 0xff)
            } else {
                th().glass_hi
            });
            if ui
                .add(edit)
                .on_hover_text("Tirar ou colocar coisas na tela, criar e apagar cenas")
                .clicked()
            {
                st.editing = !st.editing;
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
            if !core.cfg.is_hidden("bt_button")
                && ui
                    .add(egui::Button::new("Bluetooth").fill(th().glass_hi))
                    .on_hover_text("Procurar e conectar a mesinha pelo Bluetooth")
                    .clicked()
            {
                st.bt_open = true;
            }
            let wiz = egui::Button::new("Configurar mesinha").fill(th().glass_hi);
            if !core.cfg.is_hidden("wizard_button")
                && ui
                    .add(wiz)
                    .on_hover_text(
                        "Mexa os faders e knobs na ordem pedida: o programa aprende tudo sozinho",
                    )
                    .clicked()
            {
                core.wizard_start();
            }
            ui.add_space(10.0);

            // CPU do áudio
            let cpu = stats.cpu_permille.load(Ordering::Relaxed) as f32 / 10.0;
            let cpu_color = if cpu < 60.0 {
                GREEN
            } else if cpu < 85.0 {
                YELLOW
            } else {
                RED
            };
            ui.label(
                RichText::new(format!("CPU {cpu:.0}%"))
                    .color(cpu_color)
                    .monospace(),
            );

            // Áudio
            match &audio.info {
                Some(info) => {
                    let ms = info
                        .buffer_frames
                        .map(|f| f as f32 * 1000.0 / info.sample_rate as f32);
                    let txt = match ms {
                        Some(ms) => format!("🔊 {} kHz · {:.1} ms", info.sample_rate / 1000, ms),
                        None => format!("🔊 {} kHz", info.sample_rate / 1000),
                    };
                    ui.label(RichText::new(txt).color(th().dim))
                        .on_hover_text(&info.device);
                }
                None => {
                    let err = audio.error.clone().unwrap_or_else(|| "sem saída".into());
                    ui.label(RichText::new("🔇 Sem áudio").color(RED))
                        .on_hover_text(err);
                }
            }

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
            None => "MODO APRENDER — 1) clique num controle da tela   2) mexa o controle da mesinha.   (Esc sai)".to_string(),
            Some(t) => format!("Agora mexa na mesinha o controle para: {}", t.label()),
        };
        let last = core
            .last_signal
            .clone()
            .map(|l| format!("   ·   último sinal da mesinha: {l}"))
            .unwrap_or_default();
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
                    if i == 0 { LAYER_COLORS[0] } else { PAD_COLOR },
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

// ---------- Pads ----------

const PAD_COLOR: Color32 = Color32::from_rgb(0xbf, 0x8c, 0xff);

fn pads_page(ui: &mut Ui, core: &mut Core, st: &mut UiState) {
    let avail = ui.available_size();
    let side_w = 170.0;
    let gap = 10.0;

    // Estilos: clássicos (loop) e vivos (motor granular)
    for (title, range) in [
        ("Clássicos:", 0..pads::PAD_BANKS),
        ("Vivos ✨:", pads::PAD_BANKS..pads::ALL_BANKS),
    ] {
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
    // Ajustes ao vivo
    let mut v = core.cfg.vivo.clone();
    let vivo = pads::is_vivo(core.cfg.pad_bank);
    ui.horizontal(|ui| {
        ui.spacing_mut().slider_width = 120.0;
        ui.label(RichText::new("Reverb de igreja").color(th().dim));
        ui.add(egui::Slider::new(&mut v.church_pads, 0.0..=1.0).show_value(false));
        if vivo {
            ui.add_space(12.0);
            ui.label(RichText::new("Brilho").color(th().dim));
            ui.add(egui::Slider::new(&mut v.bright, 0.0..=1.0).show_value(false));
            ui.label(RichText::new("Shimmer").color(th().dim));
            ui.add(egui::Slider::new(&mut v.shimmer, 0.0..=1.0).show_value(false));
            ui.label(RichText::new("Movimento").color(th().dim));
            ui.add(egui::Slider::new(&mut v.motion, 0.0..=1.0).show_value(false));
        }
    });
    if vivo {
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
    let show_scale = !core.cfg.is_hidden("scale_help");
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
            if !show_scale {
                ui.label(
                    RichText::new("Clique num tom para tocar · clique de novo para parar · trocar de tom faz a passagem suave")
                        .size(13.0)
                        .color(th().dim),
                );
            }
        });

        // Coluna da direita: volume, anterior/próximo, parar
        let h = rows as f32 * cell + (rows as f32 - 1.0) * gap;
        let (rect, _) = ui.allocate_exact_size(Vec2::new(side_w, h), Sense::hover());
        crate::theme::glass(ui.painter(), rect, 18, false, None);
        let inner = rect.shrink2(Vec2::new(12.0, 10.0));
        let mut child = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(inner)
                .layout(Layout::top_down(Align::Center)),
        );
        let ui = &mut child;
        ui.label(RichText::new("VOLUME DO PAD").strong().size(14.0));
        ui.label(RichText::new(format!("{:.0}%", core.cfg.pad_volume * 100.0)).monospace());
        let fader_h = (ui.available_height() - 100.0).max(36.0);
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
            "Pad {} ({}) — som padrão {}",
            pads::KEY_NAMES[key],
            pads::KEY_NAMES_PT[key],
            pads::bank_name(bank)
        ),
    });
    resp.context_menu(|ui| {
        if pads::is_vivo(bank) {
            ui.label(
                RichText::new(
                    "Pad Vivo: o som base vale para todos os tons (veja \"Base do som\" acima).",
                )
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
        let save_w = 150.0;
        let kb_w = 120.0;
        let gap = ui.spacing().item_spacing.x;
        let count = core.cfg.scenes.len();
        let kb_w = if core.cfg.is_hidden("kb_button") { 0.0 } else { kb_w };
        let add_w = if st.editing && count < MAX_SCENES { 56.0 } else { 0.0 };
        let slots = count as f32 + if kb_w > 0.0 { 1.0 } else { 0.0 } + if add_w > 0.0 { 1.0 } else { 0.0 };
        let w = ((ui.available_width() - save_w - kb_w - add_w - gap * slots) / count.max(1) as f32).clamp(44.0, 220.0);
        let mut delete: Option<usize> = None;
        for n in 0..count {
            let current = core.cfg.current_scene == Some(n);
            let filled = core.cfg.scenes[n].mix.is_some();
            let name = core.cfg.scenes[n].name.clone();
            let label = if n < 8 { format!("F{}  {}", n + 1, name) } else { name.clone() };
            let text = RichText::new(label)
                .size(15.0)
                .color(if current { th().ink } else if filled { th().text } else { th().dim });
            let btn = egui::Button::new(text)
                .fill(if current { GREEN } else if filled { th().glass_hi } else { th().glass })
                .stroke(if current { Stroke::NONE } else { Stroke::new(1.0, th().glass_hi) });
            let resp = ui.add_sized([w, 44.0], btn.truncate());
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
                    core.dirty = true;
                    ui.close();
                }
            });
        }
        let target = core.cfg.current_scene;
        let save = egui::Button::new(RichText::new("💾 Salvar cena").size(15.0)).fill(th().glass_hi);
        let resp = ui.add_sized([save_w, 44.0], save).on_hover_text(
            "Guarda os sons, volumes e liga/desliga na cena selecionada.\nClique com o botão direito numa cena para renomear.",
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
        if kb_w == 0.0 {
            return;
        }
        let kb = egui::Button::new(
            RichText::new("🎹 Teclado")
                .size(15.0)
                .color(if st.kb_open { th().ink } else { th().text }),
        )
        .fill(if st.kb_open { LAYER_COLORS[0] } else { th().glass_hi });
        if ui
            .add_sized([kb_w, 44.0], kb)
            .on_hover_text("Mostra/esconde um piano na tela para testar os sons sem o teclado ligado")
            .clicked()
        {
            st.kb_open = !st.kb_open;
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
            let r = pad_strip(ui, core, Vec2::new(strip_w, h));
            if st.editing && remove_x(ui, r, ("pad", 0)) {
                remove = Some("pad_strip".into());
            }
        }
        master_strip(ui, core, Vec2::new(master_w, h));
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
fn pad_strip(ui: &mut Ui, core: &mut Core, size: Vec2) -> Rect {
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
        RichText::new(pads::bank_name(core.cfg.pad_bank))
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
    let bank = ui.add_sized(
        [inner.width(), if compact { 24.0 } else { 28.0 }],
        egui::Button::new(RichText::new("ESTILO").size(13.0).color(th().dim)).fill(th().glass),
    );
    let bank = bank.on_hover_text("Troca Escuro → Coral → Suave");
    if !learnable(ui, core, &bank, Target::PadBank) && bank.clicked() {
        core.set_pad_bank(core.cfg.pad_bank + 1);
    }
    let preset_h = if compact { 34.0 } else { 84.0 };
    let fader_h = (ui.available_height() - preset_h - 22.0).max(60.0);
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

    // Knob (reverb)
    let (resp, newv) = knob(
        ui,
        if compact { 38.0 } else { 50.0 },
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

    // Liga/desliga + Solo (lado a lado quando a tela é baixa)
    let (txt, fill, tc) = match (layer.on, compact) {
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
    let (t_resp, s_resp) = if compact {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 4.0;
            let tw = (inner.width() - 4.0) * 0.66;
            let t = ui.add_sized([tw, 32.0], toggle_btn);
            let s = ui.add_sized([inner.width() - 4.0 - tw, 32.0], solo_btn);
            (t, s)
        })
        .inner
    } else {
        let t = ui.add_sized([inner.width(), 40.0], toggle_btn);
        let s = ui.add_sized([inner.width(), 28.0], solo_btn);
        (t, s)
    };
    if !learnable(ui, core, &t_resp, Target::Toggle(i)) && t_resp.clicked() {
        core.toggle(i);
    }
    if !learnable(ui, core, &s_resp, Target::Solo(i)) && s_resp.clicked() {
        core.toggle_solo(i);
    }

    // Fader (usa o espaço que sobra, deixando lugar para o timbre)
    let preset_h = if compact { 34.0 } else { 84.0 };
    let fader_h = (ui.available_height() - preset_h - 22.0).max(60.0);
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

    // Timbre
    let name = layer
        .preset
        .as_ref()
        .map(|p| p.name.clone())
        .unwrap_or_else(|| "— escolher —".into());
    let name_btn = |w: f32| {
        egui::Button::new(
            RichText::new(&name)
                .size(if compact { 12.0 } else { 13.0 })
                .color(if layer.preset.is_some() {
                    th().text
                } else {
                    th().dim
                }),
        )
        .fill(th().glass)
        .truncate()
        .min_size(Vec2::new(w, 0.0))
    };
    let mut step = 0;
    let resp = if compact {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 2.0;
            ui.spacing_mut().button_padding = Vec2::new(2.0, 2.0);
            let aw = 22.0;
            if ui
                .add_sized([aw, 28.0], egui::Button::new("◀").fill(th().glass))
                .clicked()
            {
                step = -1;
            }
            let r = ui.add_sized([inner.width() - 2.0 * aw - 4.0, 28.0], name_btn(0.0));
            if ui
                .add_sized([aw, 28.0], egui::Button::new("▶").fill(th().glass))
                .clicked()
            {
                step = 1;
            }
            r
        })
        .inner
    } else {
        ui.add_space(4.0);
        let r = ui.add_sized([inner.width(), 34.0], name_btn(inner.width()));
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
        r
    };
    if step != 0 {
        core.step_preset(i, step);
    }
    let resp = resp.on_hover_text(format!("{name}\nClique para escolher o timbre"));
    if resp.clicked() && !core.learn.active {
        st.picker = Some(i);
        st.picker_search.clear();
        st.picker_focus = true;
    }
    rect
}

fn master_strip(ui: &mut Ui, core: &mut Core, size: Vec2) {
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

    ui.add_space(if compact { 2.0 } else { 8.0 });
    ui.label(RichText::new(format!("{:.0}%", core.cfg.master * 100.0)).monospace());
    let fader_h = (ui.available_height() - if compact { 48.0 } else { 64.0 }).max(60.0);
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
                None => {
                    ui.label(RichText::new("Setas ◀ ▶ trocam a cena  ·  F1–F8 cenas  ·  Esc pânico  ·  botão direito na cena: salvar/renomear").size(13.0).color(th().dim));
                }
            },
        }
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            let peak = stats.peak_milli.load(Ordering::Relaxed) as f32 / 1000.0;
            meter(ui, peak);
            ui.label(RichText::new(format!("{} vozes", stats.voices.load(Ordering::Relaxed))).size(13.0).color(th().dim));
            let over = stats.overruns.load(Ordering::Relaxed);
            if over > 0 {
                ui.label(RichText::new(format!("⚠ {over} engasgos")).size(13.0).color(YELLOW))
                    .on_hover_text("O computador não deu conta a tempo. Aumente o buffer em Ajustes ou feche outros programas.");
            }
        });
    });
}

fn meter(ui: &mut Ui, level: f32) {
    let (r, _) = ui.allocate_exact_size(Vec2::new(140.0, 10.0), Sense::hover());
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
    let linked = core.cfg.mappings.iter().any(|m| m.target == target);
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

                ui.separator();
                ui.label(RichText::new(format!("Configurações salvas em: {}", config_path.display())).size(12.0).color(th().dim));
                ui.label(RichText::new(concat!("Piano SF2 v", env!("CARGO_PKG_VERSION"))).size(12.0).color(th().dim));
            });
        });
    if !open {
        st.settings = false;
    }
}
