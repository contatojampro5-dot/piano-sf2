//! MIDI por Bluetooth (BLE MIDI) no Windows 10/11, pela API nova do Windows (WinRT).
//! O jeito antigo (WinMM, usado para o cabo USB) não enxerga aparelhos Bluetooth.
//! Aqui só entram os aparelhos Bluetooth; os do cabo continuam pelo `midir`.

use std::sync::{Arc, Mutex};

use windows::core::HSTRING;
use windows::Devices::Enumeration::DeviceInformation;
use windows::Devices::Midi::{MidiInPort, MidiMessageReceivedEventArgs, MidiOutPort};
use windows::Foundation::TypedEventHandler;
use windows::Storage::Streams::{DataReader, DataWriter};

/// Serviço GATT do MIDI por Bluetooth: aparece no "id" dos aparelhos BLE MIDI.
const BLE_MIDI_SERVICE: &str = "03b80e5a-ede8-4b33-a751-6ce34ec4c700";

pub fn init_thread() {
    unsafe {
        let _ = windows::Win32::System::WinRT::RoInitialize(
            windows::Win32::System::WinRT::RO_INIT_MULTITHREADED,
        );
    }
}

fn is_ble(id: &str) -> bool {
    let id = id.to_lowercase();
    id.contains(BLE_MIDI_SERVICE) || id.contains("bthle")
}

/// Um aparelho MIDI que a API nova do Windows enxerga.
pub struct WinRtPort {
    pub id: String,
    pub name: String,
    /// O id tem cara de Bluetooth (serviço BLE MIDI).
    pub looks_ble: bool,
}

/// Todas as entradas MIDI que a API nova enxerga (cabo e Bluetooth).
pub fn inputs() -> Vec<WinRtPort> {
    list(MidiInPort::GetDeviceSelector())
}

pub fn outputs() -> Vec<WinRtPort> {
    list(MidiOutPort::GetDeviceSelector())
}

fn list(selector: windows::core::Result<HSTRING>) -> Vec<WinRtPort> {
    let Ok(selector) = selector else {
        return Vec::new();
    };
    let Ok(op) = DeviceInformation::FindAllAsyncAqsFilter(&selector) else {
        return Vec::new();
    };
    let Ok(devices) = op.join() else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for d in devices {
        let (Ok(id), Ok(name)) = (d.Id(), d.Name()) else {
            continue;
        };
        let id = id.to_string_lossy();
        let looks_ble = is_ble(&id);
        out.push(WinRtPort {
            id,
            name: name.to_string_lossy(),
            looks_ble,
        });
    }
    out
}

/// Entrada Bluetooth aberta. Fecha sozinha ao sair de cena.
pub struct BleIn {
    port: MidiInPort,
    token: i64,
}

// O MidiInPort é um objeto WinRT "agile" (pode ser usado de qualquer thread).
unsafe impl Send for BleIn {}

impl BleIn {
    pub fn open(id: &str, callback: impl FnMut(&[u8]) + Send + 'static) -> Result<Self, String> {
        let port = MidiInPort::FromIdAsync(&HSTRING::from(id))
            .and_then(|op| op.join())
            .map_err(|e| format!("não consegui abrir pelo Bluetooth ({e})"))?;
        let cb = Arc::new(Mutex::new(callback));
        let handler =
            TypedEventHandler::<MidiInPort, MidiMessageReceivedEventArgs>::new(move |_, args| {
                let Some(args) = args.as_ref() else {
                    return Ok(());
                };
                let buffer = args.Message()?.RawData()?;
                let len = buffer.Length()? as usize;
                if len == 0 {
                    return Ok(());
                }
                let reader = DataReader::FromBuffer(&buffer)?;
                let mut bytes = vec![0u8; len];
                reader.ReadBytes(&mut bytes)?;
                // Sysex, relógio e "estou vivo" não interessam.
                if !matches!(bytes[0], 0xF0..=0xFF) {
                    if let Ok(mut f) = cb.lock() {
                        f(&bytes);
                    }
                }
                Ok(())
            });
        let token = port
            .MessageReceived(&handler)
            .map_err(|e| format!("Bluetooth sem resposta ({e})"))?;
        Ok(Self { port, token })
    }
}

impl Drop for BleIn {
    fn drop(&mut self) {
        let _ = self.port.RemoveMessageReceived(self.token);
        let _ = self.port.Close();
    }
}

/// Saída Bluetooth (para as luzes dos botões da mesinha).
pub struct BleOut {
    port: windows::Devices::Midi::IMidiOutPort,
}

unsafe impl Send for BleOut {}

impl BleOut {
    pub fn open(id: &str) -> Option<Self> {
        let port = MidiOutPort::FromIdAsync(&HSTRING::from(id))
            .ok()?
            .join()
            .ok()?;
        Some(Self { port })
    }

    pub fn send(&mut self, msg: &[u8]) {
        let send = || -> windows::core::Result<()> {
            let w = DataWriter::new()?;
            w.WriteBytes(msg)?;
            let buf = w.DetachBuffer()?;
            self.port.SendBuffer(&buf)
        };
        let _ = send();
    }
}

impl Drop for BleOut {
    fn drop(&mut self) {
        let _ = self.port.Close();
    }
}

// ---------------------------------------------------------------------------------------------
// Conexão DIRETA com o Bluetooth da mesinha (GATT), sem passar pelo MIDI do Windows.
// No Windows 10 o MIDI-Bluetooth do sistema muitas vezes não aparece ou cai; falando direto com a
// característica BLE-MIDI o programa recebe os faders/botões sem driver nenhum.
// ---------------------------------------------------------------------------------------------

use windows::core::GUID;
use windows::Devices::Bluetooth::Advertisement::{
    BluetoothLEAdvertisementReceivedEventArgs, BluetoothLEAdvertisementWatcher,
    BluetoothLEScanningMode,
};
use windows::Devices::Bluetooth::GenericAttributeProfile::{
    GattCharacteristic, GattClientCharacteristicConfigurationDescriptorValue,
    GattCommunicationStatus, GattSession, GattValueChangedEventArgs, GattWriteOption,
};
use windows::Devices::Bluetooth::{
    BluetoothCacheMode, BluetoothConnectionStatus, BluetoothLEDevice,
};

use crate::blemidi;

/// Aparelho achado na busca.
#[derive(Clone, Debug)]
pub struct ScanHit {
    pub address: u64,
    pub name: String,
    pub rssi: i16,
    /// Anuncia o serviço MIDI (ou já está pareado e parece MIDI).
    pub midi: bool,
    pub paired: bool,
}

fn looks_midi_name(name: &str) -> bool {
    let n = name.to_lowercase();
    [
        "smc", "midi", "m-vave", "mvave", "cuvave", "mixer", "key", "piano", "casio",
    ]
    .iter()
    .any(|k| n.contains(k))
}

/// Busca aparelhos Bluetooth LE por perto (enquanto existir).
pub struct Scanner {
    watcher: BluetoothLEAdvertisementWatcher,
    token: i64,
    pub hits: Arc<Mutex<Vec<ScanHit>>>,
}

unsafe impl Send for Scanner {}

impl Scanner {
    pub fn start() -> Result<Self, String> {
        let watcher = BluetoothLEAdvertisementWatcher::new()
            .map_err(|e| format!("Bluetooth indisponível ({e})"))?;
        let _ = watcher.SetScanningMode(BluetoothLEScanningMode::Active);
        let hits: Arc<Mutex<Vec<ScanHit>>> = Arc::new(Mutex::new(paired_le()));
        let h2 = hits.clone();
        let midi_uuid = GUID::from_u128(blemidi::SERVICE_UUID);
        let handler = TypedEventHandler::<
            BluetoothLEAdvertisementWatcher,
            BluetoothLEAdvertisementReceivedEventArgs,
        >::new(move |_, args| {
            let Some(args) = args.as_ref() else {
                return Ok(());
            };
            let address = args.BluetoothAddress()?;
            let rssi = args.RawSignalStrengthInDBm().unwrap_or(-100);
            let adv = args.Advertisement()?;
            let name = adv
                .LocalName()
                .map(|n| n.to_string_lossy())
                .unwrap_or_default();
            let midi = adv
                .ServiceUuids()
                .map(|v| v.into_iter().any(|u| u == midi_uuid))
                .unwrap_or(false);
            if let Ok(mut list) = h2.lock() {
                match list.iter_mut().find(|h| h.address == address) {
                    Some(h) => {
                        h.rssi = rssi;
                        h.midi |= midi;
                        if h.name.is_empty() && !name.is_empty() {
                            h.name = name;
                        }
                    }
                    None => list.push(ScanHit {
                        address,
                        name,
                        rssi,
                        midi,
                        paired: false,
                    }),
                }
            }
            Ok(())
        });
        let token = watcher
            .Received(&handler)
            .map_err(|e| format!("Bluetooth sem resposta ({e})"))?;
        watcher
            .Start()
            .map_err(|e| format!("Não consegui procurar no Bluetooth (ele está ligado?) ({e})"))?;
        Ok(Self {
            watcher,
            token,
            hits,
        })
    }

    /// Só o que interessa: aparelhos MIDI ou com nome (os sem nome ficam escondidos).
    pub fn snapshot(&self) -> Vec<ScanHit> {
        let mut v: Vec<ScanHit> = self
            .hits
            .lock()
            .map(|l| {
                l.iter()
                    .filter(|h| h.midi || h.paired || looks_midi_name(&h.name))
                    .cloned()
                    .collect()
            })
            .unwrap_or_default();
        v.sort_by_key(|h| (!h.midi, -(h.rssi as i32)));
        v
    }
}

impl Drop for Scanner {
    fn drop(&mut self) {
        let _ = self.watcher.Stop();
        let _ = self.watcher.RemoveReceived(self.token);
    }
}

/// Aparelhos Bluetooth LE já pareados no Windows (mesmo sem estar anunciando).
fn paired_le() -> Vec<ScanHit> {
    let mut out = Vec::new();
    let Ok(sel) = BluetoothLEDevice::GetDeviceSelectorFromPairingState(true) else {
        return out;
    };
    let Ok(op) = DeviceInformation::FindAllAsyncAqsFilter(&sel) else {
        return out;
    };
    let Ok(list) = op.join() else { return out };
    for d in list {
        let Ok(id) = d.Id() else { continue };
        let Ok(dev) = BluetoothLEDevice::FromIdAsync(&id).and_then(|o| o.join()) else {
            continue;
        };
        let (Ok(address), name) = (
            dev.BluetoothAddress(),
            dev.Name().map(|n| n.to_string_lossy()).unwrap_or_default(),
        ) else {
            continue;
        };
        let midi = looks_midi_name(&name);
        out.push(ScanHit {
            address,
            name,
            rssi: -100,
            midi,
            paired: true,
        });
        let _ = dev.Close();
    }
    out
}

/// Mesinha conectada direto pelo Bluetooth.
pub struct GattMidi {
    device: BluetoothLEDevice,
    session: Option<GattSession>,
    ch: GattCharacteristic,
    token: i64,
    pub name: String,
    was_connected: bool,
    started: std::time::Instant,
}

unsafe impl Send for GattMidi {}

impl GattMidi {
    /// Conecta (pode levar alguns segundos). `on_msg` recebe cada mensagem MIDI normal.
    pub fn connect(
        address: u64,
        on_msg: impl FnMut(&[u8]) + Send + 'static,
    ) -> Result<Self, String> {
        let device = BluetoothLEDevice::FromBluetoothAddressAsync(address)
            .and_then(|o| o.join())
            .map_err(|_| {
                "aparelho não encontrado (está ligado e com o Bluetooth aceso?)".to_string()
            })?;
        let name = device
            .Name()
            .map(|n| n.to_string_lossy())
            .unwrap_or_else(|_| "Bluetooth".into());

        // Pede ao Windows para manter a conexão (e reconectar sozinho se cair).
        let session = device
            .BluetoothDeviceId()
            .and_then(|id| GattSession::FromDeviceIdAsync(&id))
            .and_then(|o| o.join())
            .ok();
        if let Some(s) = &session {
            let _ = s.SetMaintainConnection(true);
        }

        let svc_uuid = GUID::from_u128(blemidi::SERVICE_UUID);
        let find_service = |dev: &BluetoothLEDevice| -> Result<_, String> {
            let r = dev
                .GetGattServicesForUuidWithCacheModeAsync(svc_uuid, BluetoothCacheMode::Uncached)
                .and_then(|o| o.join())
                .map_err(|e| format!("não respondeu ({e})"))?;
            let status = r.Status().unwrap_or(GattCommunicationStatus::Unreachable);
            if status != GattCommunicationStatus::Success {
                return Err(format!("sem resposta do Bluetooth ({status:?})"));
            }
            let list = r.Services().map_err(|e| e.to_string())?;
            if list.Size().unwrap_or(0) == 0 {
                return Err("esse aparelho não tem MIDI por Bluetooth".into());
            }
            list.GetAt(0).map_err(|e| e.to_string())
        };
        let service = match find_service(&device) {
            Ok(s) => s,
            Err(first) => {
                // Alguns aparelhos só liberam o MIDI depois de pareados: tenta parear e de novo.
                let paired = device
                    .DeviceInformation()
                    .and_then(|d| d.Pairing())
                    .and_then(|p| {
                        if p.IsPaired()? {
                            Ok(true)
                        } else {
                            p.PairAsync()?.join().map(|_| true)
                        }
                    })
                    .unwrap_or(false);
                if !paired {
                    return Err(first);
                }
                find_service(&device)?
            }
        };

        let ch = service
            .GetCharacteristicsForUuidWithCacheModeAsync(
                GUID::from_u128(blemidi::CHARACTERISTIC_UUID),
                BluetoothCacheMode::Uncached,
            )
            .and_then(|o| o.join())
            .and_then(|r| r.Characteristics())
            .and_then(|l| l.GetAt(0))
            .map_err(|e| format!("MIDI do aparelho não encontrado ({e})"))?;

        let cb = Arc::new(Mutex::new(on_msg));
        let handler = TypedEventHandler::<GattCharacteristic, GattValueChangedEventArgs>::new(
            move |_, args| {
                let Some(args) = args.as_ref() else {
                    return Ok(());
                };
                let buf = args.CharacteristicValue()?;
                let len = buf.Length()? as usize;
                if len == 0 {
                    return Ok(());
                }
                let reader = DataReader::FromBuffer(&buf)?;
                let mut bytes = vec![0u8; len];
                reader.ReadBytes(&mut bytes)?;
                if let Ok(mut f) = cb.lock() {
                    blemidi::parse(&bytes, |m| {
                        if m[0] < 0xF0 {
                            f(m)
                        }
                    });
                }
                Ok(())
            },
        );
        let token = ch.ValueChanged(&handler).map_err(|e| e.to_string())?;
        let me = Self {
            device,
            session,
            ch,
            token,
            name,
            was_connected: true,
            started: std::time::Instant::now(),
        };
        me.subscribe()?;
        Ok(me)
    }

    fn subscribe(&self) -> Result<(), String> {
        let st = self
            .ch
            .WriteClientCharacteristicConfigurationDescriptorAsync(
                GattClientCharacteristicConfigurationDescriptorValue::Notify,
            )
            .and_then(|o| o.join())
            .map_err(|e| format!("não aceitou ligar o envio ({e})"))?;
        if st != GattCommunicationStatus::Success {
            return Err(format!("não aceitou ligar o envio ({st:?})"));
        }
        Ok(())
    }

    pub fn connected(&self) -> bool {
        self.device
            .ConnectionStatus()
            .map(|s| s == BluetoothConnectionStatus::Connected)
            .unwrap_or(false)
    }

    /// Chamado de tempos em tempos: se a conexão voltou, liga o envio de novo.
    pub fn keep_alive(&mut self) {
        let now = self.connected();
        if now && !self.was_connected {
            let _ = self.subscribe();
        }
        self.was_connected = now;
    }

    /// Manda uma mensagem MIDI (luz de botão) para a mesinha.
    pub fn send(&self, msg: &[u8]) {
        let pkt = blemidi::packet(msg, (self.started.elapsed().as_millis() & 0x1FFF) as u16);
        let go = || -> windows::core::Result<()> {
            let w = DataWriter::new()?;
            w.WriteBytes(&pkt)?;
            let buf = w.DetachBuffer()?;
            let _ = self
                .ch
                .WriteValueWithOptionAsync(&buf, GattWriteOption::WriteWithoutResponse)?;
            Ok(())
        };
        let _ = go();
    }
}

impl Drop for GattMidi {
    fn drop(&mut self) {
        let _ = self.ch.RemoveValueChanged(self.token);
        if let Some(s) = &self.session {
            let _ = s.SetMaintainConnection(false);
            let _ = s.Close();
        }
        let _ = self.device.Close();
    }
}
