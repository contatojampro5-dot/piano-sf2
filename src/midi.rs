//! Entradas MIDI (piano e mesinha), pelo cabo USB e (no Windows) pelo Bluetooth.
//! Procura aparelhos a cada 1,5 s: se o cabo soltar ou o Bluetooth cair, reconecta sozinho.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crossbeam_channel::Sender;
use midir::{Ignore, MidiInput, MidiInputConnection, MidiOutput, MidiOutputConnection};

use crate::core::SharedCore;
use crate::engine::Cmd;
use crate::model::PortRole;

/// Final do nome dos aparelhos que chegam pelo Bluetooth.
pub const BT_SUFFIX: &str = " (Bluetooth)";

#[derive(Clone)]
pub struct PortInfo {
    pub name: String,
    pub role: PortRole,
    pub connected: bool,
    pub error: Option<String>,
}

#[derive(Default)]
pub struct MidiStatus {
    pub ports: Vec<PortInfo>,
    pub led_output: Option<String>,
    /// Diagnóstico: tudo que o Windows enxerga pela API nova (nome, id, usado?).
    pub winrt_seen: Vec<(String, String, bool)>,
    /// Aparelhos conectados direto pelo Bluetooth (ver `btdirect.rs`).
    pub bt_ports: Vec<PortInfo>,
    /// Janela "Procurar Bluetooth": está procurando? o que achou?
    pub bt_scanning: bool,
    pub bt_scan_error: Option<String>,
    pub bt_hits: Vec<BtHit>,
    /// Situação de cada aparelho salvo: (endereço, nome, texto).
    pub bt_states: Vec<(u64, String, String)>,
}

impl MidiStatus {
    /// Todas as portas (cabo, Bluetooth do Windows e Bluetooth direto).
    pub fn all_ports(&self) -> Vec<PortInfo> {
        self.ports
            .iter()
            .chain(self.bt_ports.iter())
            .cloned()
            .collect()
    }
}

/// Aparelho achado na busca Bluetooth.
#[derive(Clone, Debug)]
pub struct BtHit {
    pub address: u64,
    pub name: String,
    pub rssi: i16,
    pub midi: bool,
    pub paired: bool,
}

/// Pedidos da tela para o Bluetooth direto.
pub enum BtCmd {
    StartScan,
    StopScan,
    Connect { address: u64, name: String },
    Forget { address: u64 },
}

pub type SharedMidiStatus = Arc<Mutex<MidiStatus>>;

/// Ligação aberta com um aparelho (fecha sozinha quando sai do mapa).
enum Link {
    Usb(#[allow(dead_code)] MidiInputConnection<()>),
    #[cfg(windows)]
    Bt(#[allow(dead_code)] crate::ble::BleIn),
}

struct Conn {
    role: PortRole,
    _link: Link,
}

/// Saída para as luzes dos botões da mesinha.
enum LedOut {
    Usb(MidiOutputConnection),
    #[cfg(windows)]
    Bt(crate::ble::BleOut),
}

impl LedOut {
    fn send(&mut self, msg: &[u8]) {
        match self {
            LedOut::Usb(c) => {
                let _ = c.send(msg);
            }
            #[cfg(windows)]
            LedOut::Bt(c) => c.send(msg),
        }
    }
}

/// Um aparelho encontrado: nome mostrado na tela e, se for Bluetooth, o id do Windows.
struct Found {
    name: String,
    bt_id: Option<String>,
}

pub fn start(core: SharedCore, tx: Sender<Cmd>, status: SharedMidiStatus, rescan: Arc<AtomicBool>) {
    std::thread::Builder::new()
        .name("midi".into())
        .spawn(move || run(core, tx, status, rescan))
        .expect("thread MIDI");
}

fn run(core: SharedCore, tx: Sender<Cmd>, status: SharedMidiStatus, rescan: Arc<AtomicBool>) {
    #[cfg(windows)]
    crate::ble::init_thread();
    let mut conns: HashMap<String, Conn> = HashMap::new();
    let mut errors: HashMap<String, String> = HashMap::new();
    let mut led_out: Option<(String, LedOut)> = None;
    let mut leds_sent: HashMap<(u8, u8), bool> = HashMap::new();
    let mut last_scan = Instant::now() - Duration::from_secs(10);

    loop {
        if last_scan.elapsed() >= Duration::from_millis(1500)
            || rescan.swap(false, Ordering::Relaxed)
        {
            last_scan = Instant::now();
            let (mut found, seen) = find_inputs();
            // O que já está conectado direto pelo Bluetooth não é aberto de novo pelo MIDI do Windows.
            {
                let c = core.lock().unwrap();
                let direct: Vec<String> = c.cfg.bt_devices.iter().map(|d| plain(&d.name)).collect();
                found.retain(|f| {
                    f.bt_id.is_none()
                        || !direct
                            .iter()
                            .any(|d| !d.is_empty() && plain(&f.name).contains(d.as_str()))
                });
            }
            let roles: Vec<(String, PortRole)> = {
                let c = core.lock().unwrap();
                found
                    .iter()
                    .map(|f| (f.name.clone(), c.cfg.role_for(&f.name)))
                    .collect()
            };
            // Fecha o que sumiu ou mudou de função.
            conns.retain(|name, conn| roles.iter().any(|(n, r)| n == name && *r == conn.role));
            // Abre o que é novo.
            for (f, (name, role)) in found.iter().zip(&roles) {
                if *role == PortRole::Ignore || conns.contains_key(name) {
                    continue;
                }
                let handler = make_handler(*role, core.clone(), tx.clone());
                let link = match &f.bt_id {
                    None => connect_usb(name, handler).map(Link::Usb),
                    #[cfg(windows)]
                    Some(id) => crate::ble::BleIn::open(id, handler).map(Link::Bt),
                    #[cfg(not(windows))]
                    Some(_) => Err("Bluetooth só no Windows".to_string()),
                };
                match link {
                    Ok(link) => {
                        errors.remove(name);
                        conns.insert(
                            name.clone(),
                            Conn {
                                role: *role,
                                _link: link,
                            },
                        );
                    }
                    Err(e) => {
                        errors.insert(name.clone(), e);
                    }
                }
            }
            // Luzes da mesinha: abre a saída com o mesmo nome (ou parecido).
            let controller = roles
                .iter()
                .find(|(n, r)| *r == PortRole::Controller && conns.contains_key(n));
            match controller {
                Some((name, _)) if led_out.as_ref().map(|(n, _)| n != name).unwrap_or(true) => {
                    led_out = open_output_like(name).map(|c| (name.clone(), c));
                    leds_sent.clear();
                }
                None => {
                    led_out = None;
                }
                _ => {}
            }
            let mut st = status.lock().unwrap();
            st.ports = roles
                .iter()
                .map(|(n, r)| PortInfo {
                    name: n.clone(),
                    role: *r,
                    connected: conns.contains_key(n),
                    error: errors.get(n).cloned(),
                })
                .collect();
            st.led_output = led_out.as_ref().map(|(n, _)| n.clone());
            st.winrt_seen = seen;
            drop(st);
            if let Ok(c) = core.lock() {
                c.request_repaint();
            }
        }

        // Atualiza as luzes dos botões que mudaram.
        if let Some((_, out)) = led_out.as_mut() {
            let leds = core.lock().map(|c| c.leds()).unwrap_or_default();
            for (ch, note, lit) in leds {
                if leds_sent.get(&(ch, note)) != Some(&lit) {
                    out.send(&[0x90 | (ch & 0x0F), note, if lit { 0x7F } else { 0 }]);
                    leds_sent.insert((ch, note), lit);
                }
            }
        }

        std::thread::sleep(Duration::from_millis(60));
    }
}

/// Nome "limpo" para comparar aparelhos (sem espaços, símbolos e números de porta).
fn plain(name: &str) -> String {
    name.to_lowercase()
        .chars()
        .filter(|c| c.is_alphabetic())
        .collect::<String>()
        .replace("midi", "")
}

/// Aparelhos do cabo (WinMM) + os que só a API nova enxerga (Bluetooth).
/// Devolve também o diagnóstico (o que a API nova viu e se foi usado).
fn find_inputs() -> (Vec<Found>, Vec<(String, String, bool)>) {
    #[cfg_attr(not(windows), allow(unused_mut))]
    let mut found: Vec<Found> = usb_input_names()
        .into_iter()
        .map(|name| Found { name, bt_id: None })
        .collect();
    #[cfg_attr(not(windows), allow(unused_mut))]
    let mut seen = Vec::new();
    #[cfg(windows)]
    {
        let usb: Vec<String> = found.iter().map(|f| plain(&f.name)).collect();
        for p in crate::ble::inputs() {
            // Mesmo aparelho do cabo? Então não abre de novo (senão cada botão contaria 2 vezes).
            let pn = plain(&p.name);
            let same_as_usb = !pn.is_empty()
                && usb
                    .iter()
                    .any(|u| !u.is_empty() && (u.contains(&pn) || pn.contains(u.as_str())));
            let use_it = p.looks_ble || !same_as_usb;
            seen.push((p.name.clone(), p.id.clone(), use_it));
            if !use_it {
                continue;
            }
            let shown = format!("{}{BT_SUFFIX}", p.name);
            if !found.iter().any(|f| f.name == shown) {
                found.push(Found {
                    name: shown,
                    bt_id: Some(p.id),
                });
            }
        }
    }
    (found, seen)
}

fn usb_input_names() -> Vec<String> {
    let Ok(input) = MidiInput::new("piano-sf2 lista") else {
        return Vec::new();
    };
    let mut names: Vec<String> = input
        .ports()
        .iter()
        .filter_map(|p| input.port_name(p).ok())
        .collect();
    names.dedup();
    names
}

pub type Handler = Box<dyn FnMut(&[u8]) + Send + 'static>;

/// O que fazer com cada mensagem: piano toca; mesinha comanda a tela.
pub fn make_handler(role: PortRole, core: SharedCore, tx: Sender<Cmd>) -> Handler {
    match role {
        PortRole::Piano => Box::new(move |msg| piano_message(msg, &core, &tx)),
        _ => Box::new(move |msg| {
            if let Ok(mut c) = core.lock() {
                if c.handle_controller(msg) {
                    c.request_repaint();
                }
            }
        }),
    }
}

fn connect_usb(name: &str, mut handler: Handler) -> Result<MidiInputConnection<()>, String> {
    let mut input = MidiInput::new("piano-sf2").map_err(|e| e.to_string())?;
    input.ignore(Ignore::All);
    let port = input
        .ports()
        .into_iter()
        .find(|p| input.port_name(p).ok().as_deref() == Some(name))
        .ok_or("porta sumiu")?;
    // No Windows, se outro programa (ex.: Numa) estiver usando o aparelho, dá erro aqui.
    input
        .connect(&port, "piano-sf2-in", move |_, msg, _| handler(msg), ())
        .map_err(|e| format!("em uso por outro programa? ({e})"))
}

/// Piano: notas vão direto para o motor (sem esperar a tela).
fn piano_message(msg: &[u8], core: &SharedCore, tx: &Sender<Cmd>) {
    let Some(&status) = msg.first() else { return };
    let d1 = msg.get(1).copied().unwrap_or(0) & 0x7F;
    let d2 = msg.get(2).copied().unwrap_or(0) & 0x7F;
    match status & 0xF0 {
        0x90 if d2 > 0 => {
            let _ = tx.send(Cmd::NoteOn { key: d1, vel: d2 });
            crate::core::mark_note(d1, true);
        }
        0x90 | 0x80 => {
            let _ = tx.send(Cmd::NoteOff { key: d1 });
            crate::core::mark_note(d1, false);
        }
        0xB0 => {
            let _ = tx.send(Cmd::Cc { num: d1, val: d2 });
        }
        0xE0 => {
            let _ = tx.send(Cmd::PitchBend { lsb: d1, msb: d2 });
        }
        0xC0 => {
            if let Ok(mut c) = core.lock() {
                c.handle_piano(msg);
            }
            return;
        }
        _ => return,
    }
    // Só marca "atividade" (luzinha na tela) se a tela não estiver ocupada.
    if let Ok(mut c) = core.try_lock() {
        let first = c
            .piano_activity
            .map(|t| t.elapsed() > Duration::from_millis(150))
            .unwrap_or(true);
        c.piano_activity = Some(Instant::now());
        if first {
            c.request_repaint();
        }
    }
}

fn open_output_like(input_name: &str) -> Option<LedOut> {
    #[cfg(windows)]
    if let Some(base) = input_name.strip_suffix(BT_SUFFIX) {
        let port = crate::ble::outputs().into_iter().find(|p| p.name == base)?;
        return crate::ble::BleOut::open(&port.id).map(LedOut::Bt);
    }
    let out = MidiOutput::new("piano-sf2 luzes").ok()?;
    let ports = out.ports();
    let key = input_name.to_lowercase();
    let base: String = key.chars().take(6).collect();
    let port = ports
        .iter()
        .find(|p| {
            out.port_name(p)
                .map(|n| n.to_lowercase() == key)
                .unwrap_or(false)
        })
        .or_else(|| {
            ports.iter().find(|p| {
                out.port_name(p)
                    .map(|n| n.to_lowercase().contains(&base))
                    .unwrap_or(false)
            })
        })?
        .clone();
    out.connect(&port, "piano-sf2-leds").ok().map(LedOut::Usb)
}
