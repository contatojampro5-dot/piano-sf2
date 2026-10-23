// Sem janela preta de console no Windows.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(feature = "app")]
fn main() -> eframe::Result {
    raise_priority();
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title("Piano SF2")
            .with_inner_size([1280.0, 720.0])
            .with_min_inner_size([960.0, 560.0])
            .with_maximized(true),
        renderer: eframe::Renderer::Glow,
        ..Default::default()
    };
    eframe::run_native(
        "Piano SF2",
        options,
        Box::new(|cc| Ok(Box::new(piano_sf2::app::PianoApp::new(cc)))),
    )
}

#[cfg(not(feature = "app"))]
fn main() {
    eprintln!("Compilado sem a tela (feature \"app\").");
}

#[cfg(feature = "app")]
/// Prioridade alta para o processo: o Windows dá preferência ao nosso som.
fn raise_priority() {
    #[cfg(windows)]
    unsafe {
        use windows_sys::Win32::System::Threading::{
            GetCurrentProcess, SetPriorityClass, HIGH_PRIORITY_CLASS,
        };
        SetPriorityClass(GetCurrentProcess(), HIGH_PRIORITY_CLASS);
    }
}
