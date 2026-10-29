//! "Sons grátis": sons livres (FreePats) guardados nos Releases do projeto (tag `sons-v1`,
//! montados por `.github/workflows/sons.yml`). Baixa com o `curl.exe` que já vem no Windows 10
//! (sem biblioteca nova no programa) para %APPDATA%\PianoSF2\sons.

use std::path::PathBuf;
use std::process::{Child, Command, Stdio};

const BASE: &str = "https://github.com/contatojampro5-dot/piano-sf2/releases/download/sons-v1/";

pub struct Pack {
    pub file: &'static str,
    pub name: &'static str,
    pub about: &'static str,
    pub license: &'static str,
    /// Tamanho aproximado (só para a barra de progresso e o aviso de espaço).
    pub bytes: u64,
}

pub const CATALOG: [Pack; 2] = [
    Pack {
        file: "PacoteIgreja.sf2",
        name: "Pacote Igreja",
        about: "14 timbres: piano de armário, 4 pianos elétricos, órgão de igreja, Hammond, pads (coral, cordas, New Age, varredura), cordas, harpa e sinos.",
        license: "Domínio público (CC0) · FreePats",
        bytes: 223_917_628,
    },
    Pack {
        file: "PianoSalamander.sf2",
        name: "Piano de Cauda Salamander",
        about: "Piano de cauda Yamaha C5 gravado nota por nota. Som muito bonito, mas é GRANDE: precisa de 1,3 GB livres no SSD e uns 5 s para abrir.",
        license: "CC BY 3.0 · Alexander Holm, versão SF2 do FreePats",
        bytes: 1_266_462_334,
    },
];

pub fn dir() -> PathBuf {
    crate::model::config_path()
        .parent()
        .map(|p| p.join("sons"))
        .unwrap_or_else(|| PathBuf::from("sons"))
}

pub fn path(p: &Pack) -> PathBuf {
    dir().join(p.file)
}

fn part(p: &Pack) -> PathBuf {
    dir().join(format!("{}.baixando", p.file))
}

pub fn installed(p: &Pack) -> bool {
    path(p).exists()
}

/// Um download em andamento (um por vez).
pub struct Download {
    pub idx: usize,
    child: Child,
}

pub enum Poll {
    /// Fração baixada (0..1).
    Running(f32),
    Done,
    Failed(String),
}

#[cfg(windows)]
fn hidden(cmd: &mut Command) -> &mut Command {
    use std::os::windows::process::CommandExt;
    cmd.creation_flags(0x0800_0000) // CREATE_NO_WINDOW: sem janela preta piscando
}
#[cfg(not(windows))]
fn hidden(cmd: &mut Command) -> &mut Command {
    cmd
}

impl Download {
    pub fn start(idx: usize) -> Result<Self, String> {
        let p = &CATALOG[idx];
        std::fs::create_dir_all(dir()).map_err(|e| format!("não deu para criar a pasta: {e}"))?;
        let url = format!("{BASE}{}", p.file);
        let out = part(p);
        let _ = std::fs::remove_file(&out);
        let curl = if cfg!(windows) { "curl.exe" } else { "curl" };
        let spawn = |cmd: &mut Command| {
            hidden(cmd)
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
        };
        let child = match spawn(Command::new(curl).args(["-fsSL", "--retry", "3", "-o"]).arg(&out).arg(&url)) {
            Ok(c) => c,
            // Windows sem curl (bem antigo): usa o PowerShell.
            Err(_) if cfg!(windows) => spawn(Command::new("powershell").args([
                "-NoProfile",
                "-Command",
                &format!(
                    "$ProgressPreference='SilentlyContinue'; Invoke-WebRequest -UseBasicParsing -Uri '{url}' -OutFile '{}'",
                    out.display()
                ),
            ]))
            .map_err(|e| format!("não deu para baixar: {e}"))?,
            Err(e) => return Err(format!("não deu para baixar: {e}")),
        };
        Ok(Self { idx, child })
    }

    pub fn poll(&mut self) -> Poll {
        let p = &CATALOG[self.idx];
        match self.child.try_wait() {
            Ok(None) => {
                let got = std::fs::metadata(part(p)).map(|m| m.len()).unwrap_or(0);
                Poll::Running((got as f32 / p.bytes as f32).min(0.99))
            }
            Ok(Some(s)) if s.success() => match finish(p) {
                Ok(()) => Poll::Done,
                Err(e) => Poll::Failed(e),
            },
            Ok(Some(_)) => {
                let _ = std::fs::remove_file(part(p));
                Poll::Failed("o download falhou (internet caiu?). Tente de novo.".into())
            }
            Err(e) => Poll::Failed(e.to_string()),
        }
    }

    pub fn cancel(mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        let _ = std::fs::remove_file(part(&CATALOG[self.idx]));
    }
}

/// Confere se chegou um SF2 inteiro e tira o ".baixando" do nome.
fn finish(p: &Pack) -> Result<(), String> {
    let tmp = part(p);
    let mut head = [0u8; 12];
    let len = std::fs::metadata(&tmp).map(|m| m.len()).unwrap_or(0);
    let good = std::fs::File::open(&tmp)
        .and_then(|mut f| std::io::Read::read_exact(&mut f, &mut head))
        .is_ok()
        && &head[0..4] == b"RIFF"
        && &head[8..12] == b"sfbk"
        && u32::from_le_bytes([head[4], head[5], head[6], head[7]]) as u64 + 8 == len;
    if !good {
        let _ = std::fs::remove_file(&tmp);
        return Err("o arquivo chegou incompleto. Tente de novo.".into());
    }
    std::fs::rename(&tmp, path(p)).map_err(|e| e.to_string())
}

pub fn remove(p: &Pack) {
    let _ = std::fs::remove_file(path(p));
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Baixa de verdade dos Releases: `HOME=pasta cargo test --no-default-features download -- --ignored --nocapture`
    #[test]
    #[ignore]
    fn download() {
        let mut d = Download::start(0).unwrap();
        loop {
            match d.poll() {
                Poll::Running(f) => {
                    println!("{:.0}%", f * 100.0);
                    std::thread::sleep(std::time::Duration::from_secs(2));
                }
                Poll::Done => break,
                Poll::Failed(e) => panic!("{e}"),
            }
        }
        assert!(installed(&CATALOG[0]));
        println!("ok: {}", path(&CATALOG[0]).display());
    }

    #[test]
    fn incomplete_file_is_rejected() {
        let p = Pack {
            file: "teste-incompleto.sf2",
            name: "",
            about: "",
            license: "",
            bytes: 1,
        };
        std::fs::create_dir_all(dir()).unwrap();
        std::fs::write(part(&p), b"RIFF\xff\x00\x00\x00sfbk").unwrap();
        assert!(finish(&p).is_err());
        assert!(!part(&p).exists() && !installed(&p));
    }
}
