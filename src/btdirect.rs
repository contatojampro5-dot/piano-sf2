//! Bluetooth direto (Windows): procura, conecta e mantém conectados os aparelhos BLE-MIDI salvos
//! (a mesinha), sem depender do MIDI-Bluetooth do Windows. Roda numa thread só dele.

use std::collections::HashMap;
use std::time::{Duration, Instant};

use crossbeam_channel::{Receiver, Sender};

use crate::ble::{self, GattMidi, Scanner};
use crate::core::SharedCore;
use crate::engine::Cmd;
use crate::midi::{make_handler, BtCmd, BtHit, PortInfo, SharedMidiStatus, BT_SUFFIX};
use crate::model::{BtDevice, PortRole};

const RETRY: Duration = Duration::from_secs(5);

struct Live {
    gm: GattMidi,
    role: PortRole,
    leds_sent: HashMap<(u8, u8), bool>,
}

pub fn start(core: SharedCore, tx: Sender<Cmd>, status: SharedMidiStatus, rx: Receiver<BtCmd>) {
    std::thread::Builder::new()
        .name("bluetooth".into())
        .spawn(move || run(core, tx, status, rx))
        .expect("thread Bluetooth");
}

fn shown(name: &str) -> String {
    format!("{name}{BT_SUFFIX}")
}

fn run(core: SharedCore, tx: Sender<Cmd>, status: SharedMidiStatus, rx: Receiver<BtCmd>) {
    ble::init_thread();
    let mut scanner: Option<Scanner> = None;
    let mut live: HashMap<u64, Live> = HashMap::new();
    let mut connecting: HashMap<u64, Instant> = HashMap::new();
    let mut last_try: HashMap<u64, Instant> = HashMap::new();
    let mut states: HashMap<u64, String> = HashMap::new();
    let (res_tx, res_rx) =
        crossbeam_channel::unbounded::<(u64, PortRole, Result<GattMidi, String>)>();
    let mut last_status = Instant::now() - Duration::from_secs(10);
    let mut last_alive = Instant::now();

    loop {
        // Pedidos da tela.
        while let Ok(cmd) = rx.try_recv() {
            match cmd {
                BtCmd::StartScan => {
                    scanner = None;
                    match Scanner::start() {
                        Ok(s) => {
                            scanner = Some(s);
                            status.lock().unwrap().bt_scan_error = None;
                        }
                        Err(e) => status.lock().unwrap().bt_scan_error = Some(e),
                    }
                }
                BtCmd::StopScan => scanner = None,
                BtCmd::Connect { address, name } => {
                    let mut c = core.lock().unwrap();
                    if !c.cfg.bt_devices.iter().any(|d| d.address == address) {
                        let name = if name.trim().is_empty() {
                            "Bluetooth MIDI".to_string()
                        } else {
                            name
                        };
                        c.cfg.bt_devices.push(BtDevice { address, name });
                        c.dirty = true;
                    }
                    last_try.remove(&address); // tenta agora
                }
                BtCmd::Forget { address } => {
                    let mut c = core.lock().unwrap();
                    c.cfg.bt_devices.retain(|d| d.address != address);
                    c.dirty = true;
                    live.remove(&address);
                    states.remove(&address);
                }
            }
        }

        // Resultados das tentativas de conexão.
        while let Ok((address, role, result)) = res_rx.try_recv() {
            connecting.remove(&address);
            let still_saved = core
                .lock()
                .map(|c| c.cfg.bt_devices.iter().any(|d| d.address == address))
                .unwrap_or(false);
            match result {
                Ok(gm) if still_saved => {
                    states.insert(address, "conectado".into());
                    if let Ok(c) = core.lock() {
                        c.request_repaint();
                    }
                    live.insert(
                        address,
                        Live {
                            gm,
                            role,
                            leds_sent: HashMap::new(),
                        },
                    );
                }
                Ok(_) => {}
                Err(e) => {
                    states.insert(address, format!("{e} — tento de novo em 5 s"));
                }
            }
        }

        // Tentativa que travou (o Bluetooth do Windows às vezes não responde): libera para tentar de novo.
        connecting.retain(|addr, t| {
            let stuck = t.elapsed() > Duration::from_secs(30);
            if stuck {
                states.insert(*addr, "sem resposta — tentando de novo".into());
            }
            !stuck
        });

        // Aparelhos salvos: conecta o que falta; fecha o que mudou de função.
        let saved: Vec<(BtDevice, PortRole)> = core
            .lock()
            .map(|c| {
                c.cfg
                    .bt_devices
                    .iter()
                    .map(|d| (d.clone(), c.cfg.role_for(&shown(&d.name))))
                    .collect()
            })
            .unwrap_or_default();
        live.retain(|addr, l| {
            saved
                .iter()
                .any(|(d, r)| d.address == *addr && *r == l.role)
        });
        for (d, role) in &saved {
            if live.contains_key(&d.address)
                || connecting.contains_key(&d.address)
                || *role == PortRole::Ignore
            {
                continue;
            }
            if last_try
                .get(&d.address)
                .is_some_and(|t| t.elapsed() < RETRY)
            {
                continue;
            }
            last_try.insert(d.address, Instant::now());
            connecting.insert(d.address, Instant::now());
            states.insert(d.address, "conectando...".into());
            let (address, role) = (d.address, *role);
            let handler = make_handler(role, core.clone(), tx.clone());
            let res_tx = res_tx.clone();
            let _ = std::thread::Builder::new()
                .name("bt-conectar".into())
                .spawn(move || {
                    ble::init_thread();
                    let r = GattMidi::connect(address, handler);
                    let _ = res_tx.send((address, role, r));
                });
        }

        // Mantém vivo: se a conexão voltou, liga o envio de novo.
        if last_alive.elapsed() > Duration::from_secs(1) {
            last_alive = Instant::now();
            for (addr, l) in live.iter_mut() {
                l.gm.keep_alive();
                states.insert(
                    *addr,
                    if l.gm.connected() {
                        "conectado".into()
                    } else {
                        "esperando a mesinha (ligada e perto?)".into()
                    },
                );
            }
        }

        // Luzes dos botões.
        let leds = core.lock().map(|c| c.leds()).unwrap_or_default();
        for l in live.values_mut().filter(|l| l.role == PortRole::Controller) {
            for &(ch, note, lit) in &leds {
                if l.leds_sent.get(&(ch, note)) != Some(&lit) {
                    l.gm.send(&[0x90 | (ch & 0x0F), note, if lit { 0x7F } else { 0 }]);
                    l.leds_sent.insert((ch, note), lit);
                }
            }
        }

        // Situação para a tela.
        if last_status.elapsed() > Duration::from_millis(400) {
            last_status = Instant::now();
            let mut st = status.lock().unwrap();
            st.bt_scanning = scanner.is_some();
            st.bt_hits = scanner
                .as_ref()
                .map(|s| {
                    s.snapshot()
                        .into_iter()
                        .map(|h| BtHit {
                            address: h.address,
                            name: h.name,
                            rssi: h.rssi,
                            midi: h.midi,
                            paired: h.paired,
                        })
                        .collect()
                })
                .unwrap_or_default();
            st.bt_states = saved
                .iter()
                .map(|(d, _)| {
                    (
                        d.address,
                        d.name.clone(),
                        states.get(&d.address).cloned().unwrap_or_default(),
                    )
                })
                .collect();
            st.bt_ports = saved
                .iter()
                .map(|(d, role)| {
                    let conn = live
                        .get(&d.address)
                        .map(|l| l.gm.connected())
                        .unwrap_or(false);
                    PortInfo {
                        name: shown(&d.name),
                        role: *role,
                        connected: conn,
                        error: if conn {
                            None
                        } else {
                            states.get(&d.address).cloned()
                        },
                    }
                })
                .collect();
            drop(st);
            if scanner.is_some() {
                if let Ok(c) = core.lock() {
                    c.request_repaint();
                }
            }
        }

        std::thread::sleep(Duration::from_millis(50));
    }
}
