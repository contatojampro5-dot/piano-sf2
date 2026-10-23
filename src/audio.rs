//! Saída de áudio (WASAPI no Windows, via cpal).

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{BufferSize, SampleFormat, StreamConfig, SupportedBufferSize};

use crate::engine::Engine;

#[derive(Clone, Default)]
pub struct AudioInfo {
    pub device: String,
    pub sample_rate: u32,
    pub channels: u16,
    /// Quadros por buffer (quando o Windows informa).
    pub buffer_frames: Option<u32>,
}

pub struct Audio {
    stream: Option<cpal::Stream>,
    pub info: Option<AudioInfo>,
    pub error: Option<String>,
    engine: Arc<Mutex<Engine>>,
    /// O Windows derrubou o áudio (fone desplugado, placa trocada...).
    pub failed: Arc<AtomicBool>,
}

pub fn device_names() -> Vec<String> {
    let host = cpal::default_host();
    host.output_devices()
        .map(|it| it.map(|d| d.to_string()).collect())
        .unwrap_or_default()
}

impl Audio {
    pub fn new(engine: Arc<Mutex<Engine>>) -> Self {
        Self {
            stream: None,
            info: None,
            error: None,
            engine,
            failed: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn stop(&mut self) {
        self.stream = None;
        self.info = None;
    }

    /// Abre a saída. Devolve a taxa de amostragem em uso.
    pub fn start(&mut self, device: Option<&str>, buffer_frames: u32) -> Result<u32, String> {
        self.stop();
        self.failed.store(false, Ordering::Relaxed);
        let result = self.try_start(device, buffer_frames);
        match &result {
            Ok(_) => self.error = None,
            Err(e) => self.error = Some(e.clone()),
        }
        result
    }

    fn try_start(&mut self, device: Option<&str>, buffer_frames: u32) -> Result<u32, String> {
        let host = cpal::default_host();
        let dev = device
            .and_then(|name| host.output_devices().ok()?.find(|d| d.to_string() == name))
            .or_else(|| host.default_output_device())
            .ok_or("Nenhuma saída de som encontrada")?;
        let name = dev.to_string();
        let supported = dev
            .default_output_config()
            .map_err(|e| format!("Saída de som indisponível: {e}"))?;
        let mut config: StreamConfig = supported.config();

        // Buffer menor = menos atraso. Se o aparelho não aceitar, volta ao padrão.
        let wanted = if buffer_frames > 0 {
            match supported.buffer_size() {
                SupportedBufferSize::Range { min, max } => Some(buffer_frames.clamp(*min, *max)),
                SupportedBufferSize::Unknown => Some(buffer_frames),
            }
        } else {
            None
        };
        let format = supported.sample_format();

        let mut last_err = String::new();
        for attempt in [wanted, None] {
            config.buffer_size = attempt
                .map(BufferSize::Fixed)
                .unwrap_or(BufferSize::Default);
            match self.build(&dev, &config, format) {
                Ok(stream) => {
                    stream
                        .play()
                        .map_err(|e| format!("Não consegui tocar: {e}"))?;
                    self.stream = Some(stream);
                    self.info = Some(AudioInfo {
                        device: name,
                        sample_rate: config.sample_rate,
                        channels: config.channels,
                        buffer_frames: attempt,
                    });
                    return Ok(config.sample_rate);
                }
                Err(e) => last_err = e,
            }
            if attempt.is_none() {
                break;
            }
        }
        Err(last_err)
    }

    fn build(
        &self,
        dev: &cpal::Device,
        config: &StreamConfig,
        format: SampleFormat,
    ) -> Result<cpal::Stream, String> {
        let channels = config.channels as usize;
        let failed = self.failed.clone();
        let on_err = move |e: cpal::Error| {
            eprintln!("erro de áudio: {e}");
            failed.store(true, Ordering::Relaxed);
        };
        let engine = self.engine.clone();
        let stream = match format {
            SampleFormat::F32 => dev.build_output_stream::<f32, _, _>(
                *config,
                move |out, _| match engine.try_lock() {
                    Ok(mut e) => e.render_interleaved(out, channels),
                    Err(_) => out.fill(0.0),
                },
                on_err,
                None,
            ),
            SampleFormat::I16 => {
                let mut scratch: Vec<f32> = Vec::with_capacity(16384);
                dev.build_output_stream::<i16, _, _>(
                    *config,
                    move |out, _| {
                        scratch.resize(out.len(), 0.0);
                        match engine.try_lock() {
                            Ok(mut e) => e.render_interleaved(&mut scratch, channels),
                            Err(_) => scratch.fill(0.0),
                        }
                        for (o, s) in out.iter_mut().zip(&scratch) {
                            *o = (s.clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
                        }
                    },
                    on_err,
                    None,
                )
            }
            SampleFormat::I32 => {
                let mut scratch: Vec<f32> = Vec::with_capacity(16384);
                dev.build_output_stream::<i32, _, _>(
                    *config,
                    move |out, _| {
                        scratch.resize(out.len(), 0.0);
                        match engine.try_lock() {
                            Ok(mut e) => e.render_interleaved(&mut scratch, channels),
                            Err(_) => scratch.fill(0.0),
                        }
                        for (o, s) in out.iter_mut().zip(&scratch) {
                            *o = (s.clamp(-1.0, 1.0) as f64 * i32::MAX as f64) as i32;
                        }
                    },
                    on_err,
                    None,
                )
            }
            other => return Err(format!("Formato de som não suportado: {other:?}")),
        };
        stream.map_err(|e| format!("Não consegui abrir a saída de som: {e}"))
    }
}
