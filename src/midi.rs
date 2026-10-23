//! Entradas MIDI (piano e mesinha). Procura aparelhos a cada 1,5 s: se o cabo
//! soltar e voltar, reconecta sozinho.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crossbeam_channel::Sender;
use midir::{Ignore, MidiInput, MidiInputConnection, MidiOutput, MidiOutputConnection};

use crate::core::SharedCore;
use crate::engine::Cmd;
use crate::model::PortRole;

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
}

pub type SharedMidiStatus = Arc<Mutex<MidiStatus>>;

struct Conn {
    role: PortRole,
    _conn: MidiInputConnection<()>,
}

pub fn start(core: SharedCore, tx: Sender<Cmd>, status: SharedMidiStatus, rescan: Arc<AtomicBool>) {
    std::thread::Builder::new()
        .name("midi".into())
        .spawn(move || run(core, tx, status, rescan))
        .expect("thread MIDI");
}

fn run(core: SharedCore, tx: Sender<Cmd>, status: SharedMidiStatus, rescan: Arc<AtomicBool>) {
    let mut conns: HashMap<String, Conn> = HashMap::new();
    let mut errors: HashMap<String, String> = HashMap::new();
    let mut led_out: Option<(String, MidiOutputConnection)> = None;
    let mut leds_sent: HashMap<(u8, u8), bool> = HashMap::new();
    let mut last_scan = Instant::now() - Duration::from_secs(10);

    loop {
        if last_scan.elapsed() >= Duration::from_millis(1500)
            || rescan.swap(false, Ordering::Relaxed)
        {
            last_scan = Instant::now();
            let names = input_names();
            let roles: Vec<(String, PortRole)> = {
                let c = core.lock().unwrap();
                names
                    .iter()
                    .map(|n| (n.clone(), c.cfg.role_for(n)))
                    .collect()
            };
            // Fecha o que sumiu ou mudou de função.
            conns.retain(|name, conn| roles.iter().any(|(n, r)| n == name && *r == conn.role));
            // Abre o que é novo.
            for (name, role) in &roles {
                if *role == PortRole::Ignore || conns.contains_key(name) {
                    continue;
                }
                match connect(name, *role, core.clone(), tx.clone()) {
                    Ok(conn) => {
                        errors.remove(name);
                        conns.insert(
                            name.clone(),
                            Conn {
                                role: *role,
                                _conn: conn,
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
                    let _ = out.send(&[0x90 | (ch & 0x0F), note, if lit { 0x7F } else { 0 }]);
                    leds_sent.insert((ch, note), lit);
                }
            }
        }

        std::thread::sleep(Duration::from_millis(60));
    }
}

fn input_names() -> Vec<String> {
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

fn connect(
    name: &str,
    role: PortRole,
    core: SharedCore,
    tx: Sender<Cmd>,
) -> Result<MidiInputConnection<()>, String> {
    let mut input = MidiInput::new("piano-sf2").map_err(|e| e.to_string())?;
    input.ignore(Ignore::All);
    let port = input
        .ports()
        .into_iter()
        .find(|p| input.port_name(p).ok().as_deref() == Some(name))
        .ok_or("porta sumiu")?;
    let result = match role {
        PortRole::Piano => input.connect(
            &port,
            "piano-sf2-in",
            move |_, msg, _| piano_message(msg, &core, &tx),
            (),
        ),
        _ => input.connect(
            &port,
            "piano-sf2-ctl",
            move |_, msg, _| {
                if let Ok(mut c) = core.lock() {
                    if c.handle_controller(msg) {
                        c.request_repaint();
                    }
                }
            },
            (),
        ),
    };
    // No Windows, se outro programa (ex.: Numa) estiver usando o aparelho, dá erro aqui.
    result.map_err(|e| format!("em uso por outro programa? ({e})"))
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

fn open_output_like(input_name: &str) -> Option<MidiOutputConnection> {
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
    out.connect(&port, "piano-sf2-leds").ok()
}
