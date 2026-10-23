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
