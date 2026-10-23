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
use crate::midi::{self, SharedMidiStatus};
use crate::model::*;
use crate::pads;

// ---------- Cores ----------
const BG: Color32 = Color32::from_rgb(0x0e, 0x10, 0x14);
const PANEL: Color32 = Color32::from_rgb(0x16, 0x19, 0x1f);
const STRIP: Color32 = Color32::from_rgb(0x1c, 0x20, 0x27);
const STRIP_HI: Color32 = Color32::from_rgb(0x24, 0x29, 0x32);
const TEXT: Color32 = Color32::from_rgb(0xe8, 0xea, 0xed);
const DIM: Color32 = Color32::from_rgb(0x86, 0x8f, 0x9c);
const GREEN: Color32 = Color32::from_rgb(0x22, 0xc5, 0x5e);
const YELLOW: Color32 = Color32::from_rgb(0xf5, 0xc2, 0x1b);
const RED: Color32 = Color32::from_rgb(0xe5, 0x48, 0x4d);
const ORANGE: Color32 = Color32::from_rgb(0xff, 0x8a, 0x3d);
const LAYER_COLORS: [Color32; LAYERS] = [
    Color32::from_rgb(0x4f, 0xc3, 0xf7),
    Color32::from_rgb(0x81, 0xc7, 0x84),
    Color32::from_rgb(0xff, 0xb7, 0x4d),
    Color32::from_rgb(0xf0, 0x62, 0x92),
    Color32::from_rgb(0xba, 0x68, 0xc8),
    Color32::from_rgb(0x4d, 0xb6, 0xac),
];

const BUFFERS: [u32; 6] = [0, 64, 128, 256, 512, 1024];
const POLYPHONY: [usize; 5] = [64, 96, 128, 192, 256];

struct UiState {
    picker: Option<usize>,
    picker_search: String,
    picker_focus: bool,
    settings: bool,
    rename: Option<(usize, String)>,
    audio_devices: Vec<String>,
    /// Pedidos que só podem rodar com o `core` destravado (abrir arquivo trava a tela até escolher).
    open_sf2_dialog: bool,
    pad_file_dialog: Option<(usize, usize)>,
}

pub struct PianoApp {
    core: SharedCore,
    audio: Audio,
    stats: Arc<Stats>,
    garbage: Receiver<Garbage>,
    midi_status: SharedMidiStatus,
    midi_rescan: Arc<AtomicBool>,
    ui: UiState,
    last_save: Instant,
    audio_retry: Option<Instant>,
    config_path: std::path::PathBuf,
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
        midi::start(core.clone(), tx, midi_status.clone(), midi_rescan.clone());

        Self {
            core,
            audio,
            stats,
            garbage: grx,
            midi_status,
            midi_rescan,
            ui: UiState {
                picker: None,
                picker_search: String::new(),
                picker_focus: false,
                settings: false,
                rename: None,
                audio_devices: Vec::new(),
                open_sf2_dialog: false,
                pad_file_dialog: None,
            },
            last_save: Instant::now(),
            audio_retry: None,
            config_path,
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

        keyboard(&ctx, &mut core, &mut self.ui);

        egui::Panel::top("topo")
            .frame(
                egui::Frame::new()
                    .fill(PANEL)
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
            .frame(egui::Frame::new().fill(PANEL).inner_margin(egui::Margin {
                left: 14,
                right: 14,
                top: 6,
                bottom: 0,
            }))
            .show(ui, |ui| tabs_bar(ui, &mut core));
        if core.cfg.tab == 0 {
            egui::Panel::top("cenas")
                .frame(egui::Frame::new().fill(BG).inner_margin(egui::Margin {
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
                    .fill(PANEL)
                    .inner_margin(egui::Margin::symmetric(14, 5)),
            )
            .show(ui, |ui| footer(ui, &mut core, &self.stats));
        egui::CentralPanel::no_frame()
            .frame(egui::Frame::new().fill(BG).inner_margin(egui::Margin {
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
                self.core.lock().unwrap().set_pad_file(bank, key, Some(p));
            }
        }

        let busy = self
            .core
            .lock()
            .map(|c| c.loading.is_some())
            .unwrap_or(false);
        ctx.request_repaint_after(Duration::from_millis(if busy { 100 } else { 250 }));
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        if let Ok(c) = self.core.lock() {
            let _ = c.cfg.save_to(&self.config_path);
        }
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        BG.to_normalized_gamma_f32()
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
        s.visuals.panel_fill = BG;
        s.visuals.window_fill = PANEL;
        s.visuals.override_text_color = Some(TEXT);
        s.visuals.widgets.inactive.corner_radius = CornerRadius::same(8);
        s.visuals.widgets.hovered.corner_radius = CornerRadius::same(8);
        s.visuals.widgets.active.corner_radius = CornerRadius::same(8);
        s.visuals.widgets.inactive.weak_bg_fill = STRIP_HI;
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
                    STRIP_HI
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
                .add(egui::Button::new("⚙ Ajustes").fill(STRIP_HI))
                .clicked()
            {
                st.settings = !st.settings;
                st.audio_devices = audio::device_names();
            }
            let learn_txt = if core.learn.active {
                "Aprendendo..."
            } else {
                "Aprender"
            };
            let learn_btn =
                egui::Button::new(RichText::new(learn_txt).color(if core.learn.active {
                    BG
                } else {
                    TEXT
                }))
                .fill(if core.learn.active { ORANGE } else { STRIP_HI });
            if ui
                .add(learn_btn)
                .on_hover_text("Ligar um controle da mesinha a um controle da tela")
                .clicked()
            {
                core.learn.active = !core.learn.active;
                core.learn.target = None;
            }
            let wiz = egui::Button::new("Configurar mesinha").fill(STRIP_HI);
            if ui
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
                    ui.label(RichText::new(txt).color(DIM))
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
                .map(|s| s.ports.clone())
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
            status_dot(
                ui,
                "Mesinha",
                ctl.map(|p| p.name.as_str()),
                blink(core.controller_activity),
            );
            status_dot(
                ui,
                "Piano",
                piano.map(|p| p.name.as_str()),
                blink(core.piano_activity),
            );
        });
    });
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
                        .color(BG)
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
                ui.label(RichText::new(format!("{msg}{last}")).color(BG).strong());
            });
    }
}

fn status_dot(ui: &mut Ui, what: &str, name: Option<&str>, active: bool) {
    let color = match (name.is_some(), active) {
        (true, true) => Color32::WHITE,
        (true, false) => GREEN,
        _ => RED,
    };
    let r = ui.label(RichText::new(what).color(color));
    let c = Pos2::new(r.rect.left() - 8.0, r.rect.center().y);
    ui.painter().circle_filled(c, 5.0, color);
    ui.add_space(20.0);
    r.on_hover_text(
        name.map(|n| format!("Conectado: {n}"))
            .unwrap_or_else(|| format!("{what} não encontrado. Ligue pelo cabo USB.")),
    );
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
            let active = core.cfg.tab == i;
            let (rect, resp) = ui.allocate_exact_size(Vec2::new(210.0, 38.0), Sense::click());
            let radius = CornerRadius {
                nw: 10,
                ne: 10,
                sw: 0,
                se: 0,
            };
            let fill = if active {
                BG
            } else if resp.hovered() {
                STRIP_HI
            } else {
                STRIP
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
                if active { TEXT } else { DIM },
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

const PAD_COLOR: Color32 = Color32::from_rgb(0xa7, 0x8b, 0xfa);

fn pads_page(ui: &mut Ui, core: &mut Core, st: &mut UiState) {
    let avail = ui.available_size();
    let side_w = 170.0;
    let gap = 10.0;

    // Banco: Escuro / Coral
    ui.horizontal(|ui| {
        ui.label(RichText::new("Estilo:").color(DIM));
        for b in 0..pads::PAD_BANKS {
            let sel = core.cfg.pad_bank == b;
            let text = RichText::new(format!(
                "{}  ·  {}",
                pads::BANK_NAMES[b],
                pads::BANK_HINTS[b]
            ))
            .size(15.0)
            .color(if sel { BG } else { TEXT });
            let r = ui.add_sized(
                [290.0, 38.0],
                egui::Button::new(text).fill(if sel { PAD_COLOR } else { STRIP_HI }),
            );
            if !learnable(ui, core, &r, Target::PadBank) && r.clicked() {
                core.set_pad_bank(b);
            }
        }
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.label(
                RichText::new("Sem terça: o pad de C serve para C e Cm.")
                    .size(13.0)
                    .color(DIM),
            );
        });
    });
    ui.add_space(10.0);

    let grid_w = avail.x - side_w - gap;
    let rows_h = ui.available_height() - 30.0;
    let cell = ((grid_w - 5.0 * gap) / 6.0)
        .min((rows_h - gap) / 2.0)
        .max(70.0);
    ui.horizontal_top(|ui| {
        ui.spacing_mut().item_spacing = Vec2::splat(gap);
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing = Vec2::splat(gap);
            for row in 0..2 {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = gap;
                    for col in 0..6 {
                        pad_button(ui, core, st, row * 6 + col, cell);
                    }
                });
            }
            ui.label(
                RichText::new("Clique num tom para tocar · clique de novo para parar · trocar de tom faz a passagem suave · botão direito: usar o seu próprio arquivo")
                    .size(13.0)
                    .color(DIM),
            );
        });

        // Coluna da direita: volume, anterior/próximo, parar
        let h = 2.0 * cell + gap;
        let (rect, _) = ui.allocate_exact_size(Vec2::new(side_w, h), Sense::hover());
        ui.painter_at(rect).rect_filled(rect, 12, PANEL);
        let inner = rect.shrink2(Vec2::new(12.0, 10.0));
        let mut child = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(inner)
                .layout(Layout::top_down(Align::Center)),
        );
        let ui = &mut child;
        ui.label(RichText::new("VOLUME DO PAD").strong().size(14.0));
        ui.label(RichText::new(format!("{:.0}%", core.cfg.pad_volume * 100.0)).monospace());
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
            let prev = ui.add_sized([w, 30.0], egui::Button::new("◀").fill(STRIP_HI)).on_hover_text("Tom anterior");
            if !learnable(ui, core, &prev, Target::PadPrev) && prev.clicked() {
                core.pad_step(-1);
            }
            let next = ui.add_sized([w, 30.0], egui::Button::new("▶").fill(STRIP_HI)).on_hover_text("Próximo tom");
            if !learnable(ui, core, &next, Target::PadNext) && next.clicked() {
                core.pad_step(1);
            }
        });
        let stop = ui.add_sized(
            [inner.width(), 44.0],
            egui::Button::new(RichText::new("■ PARAR PAD").strong().color(Color32::WHITE))
                .fill(if core.pad_active.is_some() { ORANGE } else { STRIP_HI }),
        );
        if !learnable(ui, core, &stop, Target::PadStop) && stop.clicked() {
            core.pad_stop();
        }
    });
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
    let base = if sharp {
        Color32::from_rgb(0x15, 0x17, 0x1c)
    } else {
        STRIP_HI
    };
    let fill = if active {
        PAD_COLOR.gamma_multiply(0.9)
    } else if resp.hovered() {
        Color32::from_rgb(0x2e, 0x34, 0x3f)
    } else {
        base
    };
    let p = ui.painter();
    if active {
        p.rect_filled(rect.expand(4.0), 16, PAD_COLOR.gamma_multiply(0.25));
    }
    p.rect_filled(rect, 14, fill);
    p.rect_stroke(
        rect,
        14,
        Stroke::new(
            if active { 2.0 } else { 1.0 },
            if active {
                Color32::WHITE
            } else {
                Color32::from_rgb(0x34, 0x3a, 0x45)
            },
        ),
        StrokeKind::Inside,
    );
    let tc = if active { BG } else { TEXT };
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
        if active { BG } else { DIM },
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
            if active { BG } else { PAD_COLOR },
            rect.width() - 12.0,
        );
        let pos = Pos2::new(
            rect.center().x - galley.size().x / 2.0,
            rect.bottom() - 8.0 - galley.size().y,
        );
        ui.painter().galley(pos, galley, TEXT);
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
            pads::BANK_NAMES[bank]
        ),
    });
    resp.context_menu(|ui| {
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

// ---------- Cenas ----------

fn scenes_bar(ui: &mut Ui, core: &mut Core, st: &mut UiState) {
    ui.horizontal(|ui| {
        let save_w = 150.0;
        let gap = ui.spacing().item_spacing.x;
        let w = ((ui.available_width() - save_w - gap * SCENES as f32) / SCENES as f32).max(60.0);
        for n in 0..SCENES {
            let current = core.cfg.current_scene == Some(n);
            let filled = core.cfg.scenes[n].mix.is_some();
            let name = core.cfg.scenes[n].name.clone();
            let text = RichText::new(format!("F{}  {}", n + 1, name))
                .size(15.0)
                .color(if current { BG } else if filled { TEXT } else { DIM });
            let btn = egui::Button::new(text)
                .fill(if current { GREEN } else if filled { STRIP_HI } else { STRIP })
                .stroke(if current { Stroke::NONE } else { Stroke::new(1.0, STRIP_HI) });
            let resp = ui.add_sized([w, 44.0], btn);
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
        let save = egui::Button::new(RichText::new("💾 Salvar cena").size(15.0)).fill(STRIP_HI);
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
    let strip_w =
        ((avail.x - master_w - gap * (LAYERS + 1) as f32) / (LAYERS + 1) as f32).clamp(84.0, 190.0);
    let h = avail.y.max(360.0);
    ui.horizontal_top(|ui| {
        ui.spacing_mut().item_spacing.x = gap;
        for i in 0..LAYERS {
            strip(ui, core, st, i, Vec2::new(strip_w, h));
        }
        pad_strip(ui, core, Vec2::new(strip_w, h));
        master_strip(ui, core, Vec2::new(master_w, h));
    });
}

/// Canal 7: o pad (volume, tom, estilo e parar) sem precisar ir para a aba Pads.
fn pad_strip(ui: &mut Ui, core: &mut Core, size: Vec2) {
    let playing = core.pad_active.is_some();
    let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 12, if playing { STRIP_HI } else { STRIP });
    if playing {
        painter.rect_stroke(
            rect,
            12,
            Stroke::new(2.0, PAD_COLOR.gamma_multiply(0.8)),
            StrokeKind::Inside,
        );
    }
    let inner = rect.shrink2(Vec2::new(8.0, 8.0));
    let mut child = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(inner)
            .layout(Layout::top_down(Align::Center)),
    );
    let ui = &mut child;
    ui.horizontal(|ui| {
        let (r, _) = ui.allocate_exact_size(Vec2::new(inner.width(), 22.0), Sense::hover());
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
            if playing { TEXT } else { DIM },
        );
    });
    // Tom atual (grande) — o knob 7 da mesinha escolhe o tom.
    let key = core.pad_active.map(|k| pads::KEY_NAMES[k]).unwrap_or("—");
    let r = ui.add_sized(
        [inner.width(), 50.0],
        egui::Button::new(RichText::new(key).size(26.0).strong().color(if playing {
            BG
        } else {
            DIM
        }))
        .fill(if playing { PAD_COLOR } else { STRIP }),
    );
    let r = r.on_hover_text("Tom do pad (knob 7 da mesinha). Clique para ir à aba Pads.");
    if !learnable(ui, core, &r, Target::PadKnob) && r.clicked() {
        core.cfg.tab = 1;
    }
    ui.label(
        RichText::new(pads::BANK_NAMES[core.cfg.pad_bank])
            .size(12.0)
            .color(DIM),
    );
    let stop = ui.add_sized(
        [inner.width(), 40.0],
        egui::Button::new(
            RichText::new("PARAR")
                .strong()
                .size(15.0)
                .color(if playing { BG } else { DIM }),
        )
        .fill(if playing { ORANGE } else { STRIP }),
    );
    if !learnable(ui, core, &stop, Target::PadStop) && stop.clicked() {
        core.pad_stop();
    }
    let bank = ui.add_sized(
        [inner.width(), 28.0],
        egui::Button::new(RichText::new("ESTILO").size(13.0).color(DIM)).fill(STRIP),
    );
    let bank = bank.on_hover_text("Troca Escuro → Coral → Suave");
    if !learnable(ui, core, &bank, Target::PadBank) && bank.clicked() {
        core.set_pad_bank(core.cfg.pad_bank + 1);
    }
    let preset_h = 84.0;
    let fader_h = (ui.available_height() - preset_h - 22.0).max(90.0);
    ui.label(RichText::new(format!("{:.0}%", core.cfg.pad_volume * 100.0)).monospace());
    let (resp, newv) = fader(
        ui,
        Vec2::new(inner.width(), fader_h),
        core.cfg.pad_volume,
        if playing { PAD_COLOR } else { DIM },
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
            .add_sized([w, 34.0], egui::Button::new("◀").fill(STRIP))
            .on_hover_text("Tom anterior");
        if !learnable(ui, core, &prev, Target::PadPrev) && prev.clicked() {
            core.pad_step(-1);
        }
        let next = ui
            .add_sized([w, 34.0], egui::Button::new("▶").fill(STRIP))
            .on_hover_text("Próximo tom");
        if !learnable(ui, core, &next, Target::PadNext) && next.clicked() {
            core.pad_step(1);
        }
    });
}

fn strip(ui: &mut Ui, core: &mut Core, st: &mut UiState, i: usize, size: Vec2) {
    let color = LAYER_COLORS[i];
    let layer = core.cfg.mix.layers[i].clone();
    let audible = core.cfg.mix.audible(i);
    let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 12, if layer.on { STRIP_HI } else { STRIP });
    if audible {
        painter.rect_stroke(
            rect,
            12,
            Stroke::new(2.0, color.gamma_multiply(0.8)),
            StrokeKind::Inside,
        );
    }
    let inner = rect.shrink2(Vec2::new(8.0, 8.0));
    let mut child = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(inner)
            .layout(Layout::top_down(Align::Center)),
    );
    let ui = &mut child;

    // Número + cor do canal
    ui.horizontal(|ui| {
        let (r, _) = ui.allocate_exact_size(Vec2::new(inner.width(), 22.0), Sense::hover());
        ui.painter().rect_filled(
            Rect::from_min_size(r.min, Vec2::new(r.width(), 4.0)),
            2,
            color,
        );
        ui.painter().text(
            r.center() + Vec2::new(0.0, 4.0),
            Align2::CENTER_CENTER,
            format!("{}", i + 1),
            FontId::proportional(17.0),
            if audible { TEXT } else { DIM },
        );
    });

    // Knob (reverb)
    let (resp, newv) = knob(ui, 50.0, layer.reverb, color, core.learn.active);
    learnable(ui, core, &resp, Target::Reverb(i));
    if let Some(v) = newv {
        core.set_reverb(i, v);
    }
    ui.label(
        RichText::new(format!("Reverb {:.0}%", layer.reverb * 100.0))
            .size(12.0)
            .color(DIM),
    );

    // Liga/desliga
    let (txt, fill, tc) = if layer.on {
        ("LIGADA", GREEN, BG)
    } else {
        ("DESLIGADA", STRIP, DIM)
    };
    let resp = ui.add_sized(
        [inner.width(), 40.0],
        egui::Button::new(RichText::new(txt).strong().size(15.0).color(tc))
            .fill(fill)
            .stroke(Stroke::new(1.0, if layer.on { GREEN } else { STRIP_HI })),
    );
    if !learnable(ui, core, &resp, Target::Toggle(i)) && resp.clicked() {
        core.toggle(i);
    }
    // Solo
    let resp = ui.add_sized(
        [inner.width(), 28.0],
        egui::Button::new(RichText::new("SOLO").size(13.0).color(if layer.solo {
            BG
        } else {
            DIM
        }))
        .fill(if layer.solo { YELLOW } else { STRIP })
        .stroke(Stroke::new(1.0, if layer.solo { YELLOW } else { STRIP_HI })),
    );
    if !learnable(ui, core, &resp, Target::Solo(i)) && resp.clicked() {
        core.toggle_solo(i);
    }

    // Fader (usa o espaço que sobra, deixando lugar para o timbre)
    let preset_h = 84.0;
    let fader_h = (ui.available_height() - preset_h - 22.0).max(90.0);
    ui.label(
        RichText::new(format!("{:.0}%", layer.volume * 100.0))
            .monospace()
            .color(if audible { TEXT } else { DIM }),
    );
    let (resp, newv) = fader(
        ui,
        Vec2::new(inner.width(), fader_h),
        layer.volume,
        if audible { color } else { DIM },
        core.learn.active,
    );
    learnable(ui, core, &resp, Target::Volume(i));
    if let Some(v) = newv {
        core.set_volume(i, v);
    }

    // Timbre
    ui.add_space(4.0);
    let name = layer
        .preset
        .as_ref()
        .map(|p| p.name.clone())
        .unwrap_or_else(|| "— escolher —".into());
    let resp = ui.add_sized(
        [inner.width(), 34.0],
        egui::Button::new(
            RichText::new(&name)
                .size(13.0)
                .color(if layer.preset.is_some() { TEXT } else { DIM }),
        )
        .fill(STRIP)
        .truncate(),
    );
    let resp = resp.on_hover_text(format!("{name}\nClique para escolher o timbre"));
    if resp.clicked() && !core.learn.active {
        st.picker = Some(i);
        st.picker_search.clear();
        st.picker_focus = true;
    }
    ui.horizontal(|ui| {
        let w = (inner.width() - ui.spacing().item_spacing.x) / 2.0;
        if ui
            .add_sized([w, 22.0], egui::Button::new("◀").fill(STRIP))
            .on_hover_text("Timbre anterior")
            .clicked()
        {
            core.step_preset(i, -1);
        }
        if ui
            .add_sized([w, 22.0], egui::Button::new("▶").fill(STRIP))
            .on_hover_text("Próximo timbre")
            .clicked()
        {
            core.step_preset(i, 1);
        }
    });
}

fn master_strip(ui: &mut Ui, core: &mut Core, size: Vec2) {
    let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
    ui.painter_at(rect).rect_filled(rect, 12, PANEL);
    let inner = rect.shrink2(Vec2::new(10.0, 8.0));
    let mut child = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(inner)
            .layout(Layout::top_down(Align::Center)),
    );
    let ui = &mut child;
    ui.label(RichText::new("8 · GERAL").strong().size(16.0));
    ui.add_space(4.0);

    // Transpor
    ui.label(RichText::new("Transpor").size(12.0).color(DIM));
    ui.horizontal(|ui| {
        let w = (inner.width() - 2.0 * ui.spacing().item_spacing.x) / 3.0;
        let down = ui.add_sized([w, 32.0], egui::Button::new("−").fill(STRIP_HI));
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
            [w, 32.0],
            egui::Button::new(
                RichText::new(tl)
                    .strong()
                    .color(if t == 0 { TEXT } else { ORANGE }),
            )
            .fill(STRIP),
        );
        if r.on_hover_text("Clique para voltar a 0").clicked() {
            core.set_transpose(0);
        }
        let up = ui.add_sized([w, 32.0], egui::Button::new("+").fill(STRIP_HI));
        if !learnable(ui, core, &up, Target::TransposeUp) && up.clicked() {
            core.set_transpose(core.cfg.mix.transpose + 1);
        }
    });

    ui.add_space(8.0);
    ui.label(RichText::new(format!("{:.0}%", core.cfg.master * 100.0)).monospace());
    let fader_h = (ui.available_height() - 64.0).max(90.0);
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
    ui.add_space(6.0);
    let panic = ui.add_sized(
        [inner.width(), 48.0],
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

fn footer(ui: &mut Ui, core: &mut Core, stats: &Stats) {
    ui.horizontal(|ui| {
        let toast = core.toast.as_ref().filter(|(_, t)| t.elapsed() < Duration::from_secs(4)).map(|(m, _)| m.clone());
        match toast {
            Some(m) => {
                ui.label(RichText::new(m).color(ORANGE));
            }
            None => {
                ui.label(RichText::new("Setas ◀ ▶ trocam a cena  ·  F1–F8 cenas  ·  Esc pânico  ·  botão direito na cena: salvar/renomear").size(13.0).color(DIM));
            }
        }
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            let peak = stats.peak_milli.load(Ordering::Relaxed) as f32 / 1000.0;
            meter(ui, peak);
            ui.label(RichText::new(format!("{} vozes", stats.voices.load(Ordering::Relaxed))).size(13.0).color(DIM));
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
    p.rect_filled(r, 3, STRIP_HI);
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
        DIM.gamma_multiply(0.5)
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
    let v = new.unwrap_or(value);
    let p = ui.painter();
    let cx = rect.center().x;
    // marcas
    for k in 0..=10 {
        let y = bottom - travel * k as f32 / 10.0;
        let w = if k % 5 == 0 { 14.0 } else { 8.0 };
        p.line_segment(
            [Pos2::new(cx - 18.0 - w, y), Pos2::new(cx - 18.0, y)],
            Stroke::new(1.0, DIM.gamma_multiply(0.5)),
        );
        p.line_segment(
            [Pos2::new(cx + 18.0, y), Pos2::new(cx + 18.0 + w, y)],
            Stroke::new(1.0, DIM.gamma_multiply(0.5)),
        );
    }
    // trilho
    let track = Rect::from_min_max(Pos2::new(cx - 4.0, top), Pos2::new(cx + 4.0, bottom));
    p.rect_filled(track, 4, Color32::from_rgb(0x0a, 0x0b, 0x0e));
    let y = bottom - travel * v;
    p.rect_filled(
        Rect::from_min_max(Pos2::new(cx - 4.0, y), Pos2::new(cx + 4.0, bottom)),
        4,
        color.gamma_multiply(0.85),
    );
    // botão do fader
    let cap_w = (rect.width() * 0.62).clamp(34.0, 64.0);
    let cap = Rect::from_center_size(Pos2::new(cx, y), Vec2::new(cap_w, cap_h));
    let hot = resp.dragged() || resp.hovered();
    p.rect_filled(
        cap,
        5,
        if hot {
            Color32::from_rgb(0x5a, 0x62, 0x70)
        } else {
            Color32::from_rgb(0x44, 0x4a, 0x55)
        },
    );
    p.rect_stroke(
        cap,
        5,
        Stroke::new(1.0, Color32::from_rgb(0x70, 0x78, 0x86)),
        StrokeKind::Inside,
    );
    p.line_segment(
        [
            Pos2::new(cap.left() + 5.0, y),
            Pos2::new(cap.right() - 5.0, y),
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
    let v = new.unwrap_or(value);
    let p = ui.painter();
    let c = rect.center();
    let r = d / 2.0 - 3.0;
    p.circle_filled(c, r, Color32::from_rgb(0x2c, 0x31, 0x3a));
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
        Stroke::new(3.0, Color32::from_rgb(0x0a, 0x0b, 0x0e)),
    ));
    if v > 0.005 {
        p.add(Shape::line(arc(0.0, v), Stroke::new(3.0, color)));
    }
    let a = start + sweep * v;
    p.line_segment(
        [
            c,
            Pos2::new(c.x + a.cos() * (r - 6.0), c.y + a.sin() * (r - 6.0)),
        ],
        Stroke::new(2.5, TEXT),
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
                    .color(DIM),
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
                ui.heading("Som");
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
                ui.label(RichText::new("Menos atraso pesa mais no computador. Se ouvir estalos, aumente o buffer.").size(13.0).color(DIM));
                if let Some(e) = &audio.error {
                    ui.label(RichText::new(format!("⚠ {e}")).color(RED));
                }

                ui.separator();
                ui.heading("Aparelhos MIDI");
                let ports = midi_status.lock().map(|s| s.ports.clone()).unwrap_or_default();
                if ports.is_empty() {
                    ui.label(RichText::new("Nenhum aparelho encontrado. Ligue o piano e a mesinha pelo cabo USB.").color(DIM));
                }
                egui::Grid::new("midi").num_columns(3).spacing([12.0, 8.0]).show(ui, |ui| {
                    for p in &ports {
                        let c = if p.connected { GREEN } else { RED };
                        let (r, _) = ui.allocate_exact_size(Vec2::splat(14.0), Sense::hover());
                        ui.painter().circle_filled(r.center(), 5.0, c);
                        let r = ui.label(&p.name);
                        if let Some(e) = &p.error {
                            r.on_hover_text(e);
                        }
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
                            ui.label(RichText::new(m.source.label()).color(DIM));
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
                ui.label(RichText::new(format!("Configurações salvas em: {}", config_path.display())).size(12.0).color(DIM));
                ui.label(RichText::new(concat!("Piano SF2 v", env!("CARGO_PKG_VERSION"))).size(12.0).color(DIM));
            });
        });
    if !open {
        st.settings = false;
    }
}
