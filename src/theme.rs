//! Visual "vidro" (estilo iPhone): fundo celestial que respira devagar, painéis de vidro fosco
//! com transparência ajustável e brilho suave nas bordas. Tudo desenhado pelo egui (barato na GPU).

use std::cell::Cell;

use eframe::egui::{
    self, epaint::Shadow, Color32, CornerRadius, Mesh, Pos2, Rect, Shape, Stroke, StrokeKind, Vec2,
};

/// Uma paleta de fundo.
pub struct Palette {
    pub name: &'static str,
    top: [u8; 3],
    bottom: [u8; 3],
    orbs: [[u8; 3]; 3],
}

pub const PALETTES: [Palette; 4] = [
    Palette {
        name: "Celestial",
        top: [0x07, 0x0b, 0x1f],
        bottom: [0x1a, 0x12, 0x3d],
        orbs: [[0x3a, 0x5b, 0xff], [0xa1, 0x5c, 0xff], [0x39, 0xd5, 0xff]],
    },
    Palette {
        name: "Aurora",
        top: [0x03, 0x12, 0x16],
        bottom: [0x0a, 0x22, 0x2e],
        orbs: [[0x2b, 0xe3, 0xa1], [0x2b, 0xb7, 0xe3], [0x7a, 0x5c, 0xff]],
    },
    Palette {
        name: "Pôr do sol",
        top: [0x16, 0x0b, 0x1f],
        bottom: [0x2a, 0x12, 0x2e],
        orbs: [[0xff, 0x7a, 0x59], [0xff, 0x4f, 0xa0], [0xff, 0xc1, 0x5a]],
    },
    Palette {
        name: "Grafite",
        top: [0x0c, 0x0d, 0x10],
        bottom: [0x16, 0x17, 0x1b],
        orbs: [[0x4a, 0x52, 0x66], [0x3a, 0x40, 0x50], [0x5a, 0x62, 0x78]],
    },
];

/// Cores da tela já calculadas (paleta + quanto de vidro fosco).
#[derive(Clone, Copy)]
pub struct Theme {
    pub top: Color32,
    pub bottom: Color32,
    pub orbs: [Color32; 3],
    /// Vidro normal / realçado (camada ligada, hover).
    pub glass: Color32,
    pub glass_hi: Color32,
    /// Borda fina do vidro e o "brilho" de cima.
    pub edge: Color32,
    pub shine: Color32,
    /// Barras de cima/baixo.
    pub bar: Color32,
    /// Janelas (timbres, ajustes): mais opacas para ler bem.
    pub window: Color32,
    pub text: Color32,
    pub dim: Color32,
    /// Texto escuro em cima de botão colorido.
    pub ink: Color32,
    pub frost: f32,
    pub glow: f32,
}

fn rgb(c: [u8; 3]) -> Color32 {
    Color32::from_rgb(c[0], c[1], c[2])
}

fn white(a: f32) -> Color32 {
    Color32::from_white_alpha((a.clamp(0.0, 1.0) * 255.0) as u8)
}

impl Theme {
    /// `frost` 0 = vidro bem transparente, 1 = bem fosco. `glow` 0..1 = brilho das bordas.
    pub fn build(palette: usize, frost: f32, glow: f32) -> Self {
        let p = &PALETTES[palette % PALETTES.len()];
        let f = frost.clamp(0.0, 1.0);
        let bottom = rgb(p.bottom);
        let window = {
            let b = p.bottom;
            Color32::from_rgba_unmultiplied(
                b[0].saturating_add(14),
                b[1].saturating_add(14),
                b[2].saturating_add(18),
                236,
            )
        };
        Self {
            top: rgb(p.top),
            bottom,
            orbs: [rgb(p.orbs[0]), rgb(p.orbs[1]), rgb(p.orbs[2])],
            glass: white(0.035 + 0.11 * f),
            glass_hi: white(0.07 + 0.15 * f),
            edge: white(0.10 + 0.10 * f),
            shine: white(0.10 + 0.14 * f),
            bar: Color32::from_rgba_unmultiplied(
                p.top[0],
                p.top[1],
                p.top[2],
                (90.0 + 120.0 * f) as u8,
            ),
            window,
            text: Color32::from_rgb(0xf2, 0xf4, 0xf8),
            dim: Color32::from_rgba_unmultiplied(0xd6, 0xdc, 0xe8, 150),
            ink: Color32::from_rgb(0x0b, 0x0e, 0x18),
            frost: f,
            glow: glow.clamp(0.0, 1.0),
        }
    }
}

thread_local! {
    static CURRENT: Cell<Theme> = Cell::new(Theme::build(0, 0.5, 0.6));
}

/// Tema em uso (só na thread da tela).
pub fn th() -> Theme {
    CURRENT.with(|c| c.get())
}

pub fn set(theme: Theme) {
    CURRENT.with(|c| c.set(theme));
}

/// Fundo: degradê + 3 nuvens de luz que andam devagar + estrelas piscando.
pub fn paint_background(ctx: &egui::Context, time: f64, motion: bool, stars: bool) {
    let t = th();
    let screen = ctx.content_rect();
    let painter = ctx.layer_painter(egui::LayerId::background());

    let mut mesh = Mesh::default();
    mesh.colored_vertex(screen.left_top(), t.top);
    mesh.colored_vertex(screen.right_top(), t.top);
    mesh.colored_vertex(screen.right_bottom(), t.bottom);
    mesh.colored_vertex(screen.left_bottom(), t.bottom);
    mesh.add_triangle(0, 1, 2);
    mesh.add_triangle(0, 2, 3);
    painter.add(Shape::mesh(mesh));

    let tt = if motion { time as f32 } else { 0.0 };
    let (w, h) = (screen.width(), screen.height());
    let orbs = [
        (
            0.18 + 0.05 * (tt * 0.031).sin(),
            0.22 + 0.06 * (tt * 0.023).cos(),
            0.55,
            0.30,
        ),
        (
            0.82 + 0.05 * (tt * 0.027).cos(),
            0.30 + 0.05 * (tt * 0.035).sin(),
            0.50,
            0.26,
        ),
        (
            0.50 + 0.08 * (tt * 0.019).sin(),
            0.92 + 0.04 * (tt * 0.029).cos(),
            0.65,
            0.22,
        ),
    ];
    for (k, (x, y, r, a)) in orbs.iter().enumerate() {
        let c = t.orbs[k];
        let breathe = 1.0
            + if motion {
                0.08 * (tt * 0.05 + k as f32 * 2.1).sin()
            } else {
                0.0
            };
        radial(
            &painter,
            Pos2::new(screen.left() + x * w, screen.top() + y * h),
            r * w.max(h) * 0.6 * breathe,
            Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), (a * 255.0) as u8),
        );
    }

    if stars {
        // Posições fixas (sem aleatório a cada quadro): parecem estrelas de verdade.
        let mut seed: u32 = 0x9e37_79b9;
        let mut rnd = || {
            seed ^= seed << 13;
            seed ^= seed >> 17;
            seed ^= seed << 5;
            (seed as f32) / (u32::MAX as f32)
        };
        for _ in 0..90 {
            let (x, y, s, ph) = (rnd(), rnd(), rnd(), rnd() * std::f32::consts::TAU);
            let tw = if motion {
                0.55 + 0.45 * (tt * (0.6 + s) + ph).sin()
            } else {
                0.8
            };
            let alpha = (0.15 + 0.5 * s) * tw;
            painter.circle_filled(
                Pos2::new(screen.left() + x * w, screen.top() + y * h * 0.85),
                0.6 + 1.1 * s,
                white(alpha),
            );
        }
    }
}

/// Círculo de luz com borda que some (degradê radial).
fn radial(painter: &egui::Painter, c: Pos2, r: f32, color: Color32) {
    let mut mesh = Mesh::default();
    mesh.colored_vertex(c, color);
    let n = 48;
    for i in 0..=n {
        let a = i as f32 / n as f32 * std::f32::consts::TAU;
        mesh.colored_vertex(c + Vec2::new(a.cos(), a.sin()) * r, Color32::TRANSPARENT);
    }
    for i in 1..=n as u32 {
        mesh.add_triangle(0, i, i + 1);
    }
    painter.add(Shape::mesh(mesh));
}

/// Painel de vidro: preenchimento translúcido, brilho em cima, borda fina e (opcional) brilho colorido.
pub fn glass(
    painter: &egui::Painter,
    rect: Rect,
    radius: u8,
    highlight: bool,
    glow: Option<Color32>,
) {
    let t = th();
    if let Some(g) = glow {
        if t.glow > 0.01 {
            // Aura só do lado de fora: anéis que vão sumindo.
            let rings = 7;
            for k in 0..rings {
                let d = 1.0 + k as f32 * (1.2 + 1.6 * t.glow);
                let a = (1.0 - k as f32 / rings as f32).powi(2) * (0.10 + 0.22 * t.glow);
                painter.rect_stroke(
                    rect.expand(d),
                    (radius as f32 + d) as u8,
                    Stroke::new(2.0, g.gamma_multiply(a)),
                    StrokeKind::Outside,
                );
            }
        }
    }
    // Leve tom da cor por dentro (bem de leve, como vidro colorido).
    if let Some(g) = glow {
        painter.rect_filled(rect, radius, g.gamma_multiply(0.05 + 0.05 * t.glow));
    }
    painter.rect_filled(rect, radius, if highlight { t.glass_hi } else { t.glass });
    // "Brilho" do vidro: faixa mais clara que some de cima para baixo (dentro dos cantos).
    let inset = (radius as f32 * 0.55).min(rect.width() / 2.0);
    let sh = (rect.height() * 0.42).min(110.0);
    let a = Pos2::new(rect.left() + inset, rect.top() + 1.0);
    let b = Pos2::new(rect.right() - inset, rect.top() + 1.0);
    let mut mesh = Mesh::default();
    let side_top = rect.top() + radius as f32 * 0.5;
    mesh.colored_vertex(a, t.shine.gamma_multiply(0.45));
    mesh.colored_vertex(b, t.shine.gamma_multiply(0.45));
    mesh.colored_vertex(
        Pos2::new(rect.right() - 1.0, side_top),
        t.shine.gamma_multiply(0.30),
    );
    mesh.colored_vertex(
        Pos2::new(rect.right() - 1.0, rect.top() + sh),
        Color32::TRANSPARENT,
    );
    mesh.colored_vertex(
        Pos2::new(rect.left() + 1.0, rect.top() + sh),
        Color32::TRANSPARENT,
    );
    mesh.colored_vertex(
        Pos2::new(rect.left() + 1.0, side_top),
        t.shine.gamma_multiply(0.30),
    );
    for (i, j, k) in [(0, 1, 2), (0, 2, 5), (5, 2, 3), (5, 3, 4)] {
        mesh.add_triangle(i, j, k);
    }
    painter.add(Shape::mesh(mesh));
    painter.rect_stroke(
        rect,
        radius,
        Stroke::new(
            1.0,
            match glow {
                Some(g) => g.gamma_multiply(0.55),
                None => t.edge,
            },
        ),
        StrokeKind::Inside,
    );
    // linha de luz fininha no topo (como a borda de um vidro)
    painter.line_segment([a, b], Stroke::new(1.0, t.shine));
}

/// Estilo geral dos botões/janelas do egui no visual de vidro.
pub fn apply_style(ctx: &egui::Context, scale: f32) {
    let t = th();
    ctx.set_zoom_factor(scale.clamp(0.75, 1.5));
    ctx.global_style_mut(|s| {
        let v = &mut s.visuals;
        v.panel_fill = Color32::TRANSPARENT;
        v.window_fill = t.window;
        v.extreme_bg_color = Color32::from_black_alpha(90);
        v.faint_bg_color = t.glass;
        v.override_text_color = Some(t.text);
        v.window_corner_radius = CornerRadius::same(18);
        v.window_stroke = Stroke::new(1.0, t.edge);
        v.window_shadow = Shadow {
            offset: [0, 10],
            blur: 40,
            spread: 0,
            color: Color32::from_black_alpha(120),
        };
        v.popup_shadow = Shadow {
            offset: [0, 6],
            blur: 24,
            spread: 0,
            color: Color32::from_black_alpha(110),
        };
        v.menu_corner_radius = CornerRadius::same(14);
        v.selection.bg_fill = Color32::from_rgba_unmultiplied(0x0a, 0x84, 0xff, 170);
        for (w, fill, stroke) in [
            (&mut v.widgets.inactive, t.glass, t.edge),
            (&mut v.widgets.hovered, t.glass_hi, t.shine),
            (&mut v.widgets.active, t.glass_hi, t.shine),
            (&mut v.widgets.open, t.glass_hi, t.edge),
        ] {
            w.weak_bg_fill = fill;
            w.bg_fill = fill;
            w.bg_stroke = Stroke::new(1.0, stroke);
            w.corner_radius = CornerRadius::same(12);
            w.fg_stroke = Stroke::new(1.0, t.text);
        }
        v.widgets.noninteractive.bg_stroke = Stroke::new(1.0, t.edge);
        v.widgets.noninteractive.corner_radius = CornerRadius::same(12);
        s.animation_time = 0.18;
    });
}
