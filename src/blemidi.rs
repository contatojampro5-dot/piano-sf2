//! Formato "MIDI por Bluetooth LE" (especificação BLE-MIDI 1.0): cada pacote tem um byte de
//! cabeçalho e, antes de cada mensagem, um byte de tempo. Aqui só desmontamos e montamos pacotes;
//! a conversa com o Bluetooth do Windows fica em `ble.rs`.

/// Serviço e característica padrão do BLE-MIDI.
pub const SERVICE_UUID: u128 = 0x03b8_0e5a_ede8_4b33_a751_6ce3_4ec4_c700;
pub const CHARACTERISTIC_UUID: u128 = 0x7772_e5db_3868_4112_a1a9_f266_9d10_6bf3;

fn data_len(status: u8) -> usize {
    match status & 0xF0 {
        0x80 | 0x90 | 0xA0 | 0xB0 | 0xE0 => 2,
        0xC0 | 0xD0 => 1,
        _ => match status {
            0xF1 | 0xF3 => 1,
            0xF2 => 2,
            _ => 0,
        },
    }
}

/// Desmonta um pacote BLE-MIDI em mensagens MIDI normais (status + dados).
/// Sysex é ignorado (a mesinha não precisa dele aqui).
pub fn parse(packet: &[u8], mut emit: impl FnMut(&[u8])) {
    if packet.len() < 2 || packet[0] & 0x80 == 0 {
        return;
    }
    let mut i = 1;
    let mut status: u8 = 0;
    let mut in_sysex = false;
    while i < packet.len() {
        let b = packet[i];
        if in_sysex {
            // Dentro do sysex: termina num F7 (que vem depois de um byte de tempo).
            if b & 0x80 != 0 && i + 1 < packet.len() && packet[i + 1] == 0xF7 {
                in_sysex = false;
                i += 2;
            } else {
                i += 1;
            }
            continue;
        }
        if b & 0x80 != 0 {
            // Byte de tempo; depois dele pode vir um status novo.
            i += 1;
            if i >= packet.len() {
                break;
            }
            if packet[i] & 0x80 != 0 {
                let s = packet[i];
                i += 1;
                if s == 0xF0 {
                    in_sysex = true;
                    continue;
                }
                if s >= 0xF8 {
                    // Tempo real (relógio etc.): mensagem de 1 byte, não muda o "running status".
                    emit(&[s]);
                    continue;
                }
                status = s;
            }
        }
        if status == 0 {
            // Dado sem status conhecido: pacote estranho, para aqui.
            break;
        }
        let n = data_len(status);
        if n == 0 {
            emit(&[status]);
            status = 0;
            continue;
        }
        if i + n > packet.len() || packet[i..i + n].iter().any(|d| d & 0x80 != 0) {
            break;
        }
        let mut msg = [0u8; 3];
        msg[0] = status;
        msg[1..=n].copy_from_slice(&packet[i..i + n]);
        emit(&msg[..=n]);
        i += n;
    }
}

/// Monta um pacote BLE-MIDI com uma mensagem (para as luzes dos botões).
pub fn packet(msg: &[u8], millis: u16) -> Vec<u8> {
    let t = millis & 0x1FFF;
    let mut v = Vec::with_capacity(msg.len() + 2);
    v.push(0x80 | ((t >> 7) as u8 & 0x3F));
    v.push(0x80 | (t as u8 & 0x7F));
    v.extend_from_slice(msg);
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    fn all(p: &[u8]) -> Vec<Vec<u8>> {
        let mut out = Vec::new();
        parse(p, |m| out.push(m.to_vec()));
        out
    }

    #[test]
    fn single_note_on() {
        assert_eq!(all(&[0x80, 0x81, 0x90, 60, 100]), vec![vec![0x90, 60, 100]]);
    }

    #[test]
    fn running_status_and_new_timestamps() {
        // Fader (pitch bend) mandando 3 valores no mesmo pacote: com e sem byte de tempo.
        let p = [0x80, 0x81, 0xE0, 0x00, 0x40, 0x82, 0x10, 0x41, 0x20, 0x42];
        assert_eq!(
            all(&p),
            vec![
                vec![0xE0, 0x00, 0x40],
                vec![0xE0, 0x10, 0x41],
                vec![0xE0, 0x20, 0x42]
            ]
        );
    }

    #[test]
    fn mixed_messages() {
        let p = [0x80, 0x81, 0xB0, 16, 1, 0x81, 0x90, 22, 127, 0x82, 0xC0, 5];
        assert_eq!(
            all(&p),
            vec![vec![0xB0, 16, 1], vec![0x90, 22, 127], vec![0xC0, 5]]
        );
    }

    #[test]
    fn sysex_is_skipped() {
        let p = [
            0x80, 0x81, 0xF0, 0x7E, 0x00, 0x06, 0x81, 0xF7, 0x82, 0x90, 60, 1,
        ];
        assert_eq!(all(&p), vec![vec![0x90, 60, 1]]);
    }

    #[test]
    fn garbage_does_not_panic() {
        for p in [
            &[][..],
            &[0x00, 0x90][..],
            &[0x80][..],
            &[0x80, 0x81, 0x90, 60][..],
            &[0x80, 0x10, 0x20][..],
        ] {
            let _ = all(p);
        }
    }

    #[test]
    fn packet_roundtrip() {
        let p = packet(&[0x90, 16, 127], 1234);
        assert_eq!(all(&p), vec![vec![0x90, 16, 127]]);
    }
}
