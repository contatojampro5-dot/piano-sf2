//! SF2 minúsculo gerado na hora, só para os testes do motor.

use std::sync::Arc;

use crossbeam_channel::unbounded;
use rustysynth::SoundFont;

use crate::core::Core;
use crate::engine::{Cmd, Engine, Stats};
use crate::model::Config;

fn chunk(id: &[u8; 4], data: &[u8]) -> Vec<u8> {
    let mut v = id.to_vec();
    v.extend((data.len() as u32).to_le_bytes());
    v.extend(data);
    if data.len() % 2 == 1 {
        v.push(0);
    }
    v
}

fn list(kind: &[u8; 4], parts: &[Vec<u8>]) -> Vec<u8> {
    let mut d = kind.to_vec();
    for p in parts {
        d.extend(p);
    }
    chunk(b"LIST", &d)
}

fn name20(s: &str) -> Vec<u8> {
    let mut v = s.as_bytes().to_vec();
    v.resize(20, 0);
    v
}

/// Dois timbres (0:0 "Seno" e 0:1 "Seno 2") usando um seno em loop.
pub fn tiny_sf2() -> Vec<u8> {
    let rate = 22_050u32;
    let len = 2205u32; // 0,1 s = 10 ciclos de 100 Hz... afinação não importa aqui
    let mut smpl = Vec::new();
    for i in 0..len {
        let x = (i as f32 / len as f32 * std::f32::consts::TAU * 10.0).sin();
        smpl.extend(((x * 20000.0) as i16).to_le_bytes());
    }
    smpl.extend(vec![0u8; 46 * 2]);

    let info = list(
        b"INFO",
        &[
            chunk(b"ifil", &[2, 0, 1, 0]),
            chunk(b"INAM", b"Teste\0"),
            chunk(b"ZZZZ", b"x"),
        ],
    );
    let sdta = list(b"sdta", &[chunk(b"smpl", &smpl)]);

    let mut phdr = Vec::new();
    for (i, (n, patch)) in [("Seno", 0u16), ("Seno 2", 1)].iter().enumerate() {
        phdr.extend(name20(n));
        phdr.extend(patch.to_le_bytes());
        phdr.extend(0u16.to_le_bytes());
        phdr.extend((i as u16).to_le_bytes());
        phdr.extend([0u8; 12]);
    }
    phdr.extend(name20("EOP"));
    phdr.extend([0u8; 4]);
    phdr.extend(2u16.to_le_bytes());
    phdr.extend([0u8; 12]);

    // pbag: zona 0 → gen 0, zona 1 → gen 1, terminal → gen 2
    let mut pbag = Vec::new();
    for g in 0..3u16 {
        pbag.extend(g.to_le_bytes());
        pbag.extend(0u16.to_le_bytes());
    }
    let pmod = vec![0u8; 10];
    let mut pgen = Vec::new();
    for _ in 0..2 {
        pgen.extend(41u16.to_le_bytes()); // instrument
        pgen.extend(0u16.to_le_bytes());
    }
    pgen.extend([0u8; 4]);

    let mut inst = name20("Inst");
    inst.extend(0u16.to_le_bytes());
    inst.extend(name20("EOI"));
    inst.extend(1u16.to_le_bytes());
    let mut ibag = Vec::new();
    for g in [0u16, 2] {
        ibag.extend(g.to_le_bytes());
        ibag.extend(0u16.to_le_bytes());
    }
    let imod = vec![0u8; 10];
    let mut igen = Vec::new();
    igen.extend(54u16.to_le_bytes()); // sampleModes = loop
    igen.extend(1u16.to_le_bytes());
    igen.extend(53u16.to_le_bytes()); // sampleID
    igen.extend(0u16.to_le_bytes());
    igen.extend([0u8; 4]);

    let mut shdr = name20("Seno");
    for v in [0u32, len, 0, len - 1, rate] {
        shdr.extend(v.to_le_bytes());
    }
    shdr.extend([60u8, 0]);
    shdr.extend(0u16.to_le_bytes());
    shdr.extend(1u16.to_le_bytes());
    shdr.extend(name20("EOS"));
    shdr.extend([0u8; 26]);

    let pdta = list(
        b"pdta",
        &[
            chunk(b"phdr", &phdr),
            chunk(b"pbag", &pbag),
            chunk(b"pmod", &pmod),
            chunk(b"pgen", &pgen),
            chunk(b"inst", &inst),
            chunk(b"ibag", &ibag),
            chunk(b"imod", &imod),
            chunk(b"igen", &igen),
            chunk(b"shdr", &shdr),
        ],
    );

    let mut body = b"sfbk".to_vec();
    body.extend(info);
    body.extend(sdta);
    body.extend(pdta);
    chunk(b"RIFF", &body)
}

fn rms(buf: &[f32]) -> f32 {
    (buf.iter().map(|x| x * x).sum::<f32>() / buf.len() as f32).sqrt()
}

struct Rig {
    core: Core,
    pub engine: Engine,
}

fn rig() -> Rig {
    let sf = Arc::new(SoundFont::new(&mut &tiny_sf2()[..]).expect("SF2 de teste"));
    let (tx, rx) = unbounded::<Cmd>();
    let (gtx, _grx) = unbounded();
    let engine = Engine::new(rx, gtx, Arc::new(Stats::default()));
    let mut core = Core::new(Config::default(), tx);
    core.sample_rate = 48_000;
    core.install_sound_font(sf, "teste.sf2".into());
    // Sem reverb nos testes: a cauda do reverb não é "nota presa".
    for i in 0..crate::model::LAYERS {
        core.set_reverb(i, 0.0);
    }
    Rig { core, engine }
}

impl Rig {
    fn render(&mut self, frames: usize) -> Vec<f32> {
        let mut out = vec![0.0; frames * 2];
        self.engine.render_interleaved(&mut out, 2);
        out
    }
}

#[test]
fn tolerant_parser_reads_unknown_info_chunk() {
    let sf = SoundFont::new(&mut &tiny_sf2()[..]).unwrap();
    assert_eq!(sf.get_presets().len(), 2);
}

#[test]
fn first_load_turns_on_layer_one_and_plays() {
    let mut r = rig();
    assert!(r.core.cfg.mix.layers[0].on);
    assert_eq!(r.core.presets.len(), 2);
    r.render(512);
    r.core.handle_piano(&[0x90, 60, 100]);
    let out = r.render(4800);
    assert!(rms(&out) > 0.01, "tem que sair som");
}

#[test]
fn layers_stack_and_mute() {
    let mut r = rig();
    r.render(256);
    r.core.handle_piano(&[0x90, 60, 100]);
    let one = rms(&r.render(4800));
    r.core.handle_piano(&[0x80, 60, 0]);
    r.core.panic();
    r.render(4800);

    r.core.toggle(1); // segunda camada
    r.core.handle_piano(&[0x90, 60, 100]);
    let two = rms(&r.render(4800));
    assert!(two > one * 1.3, "duas camadas somam: {one} → {two}");

    // Desliga as duas com a nota presa: em ~60 ms tem que ficar mudo.
    r.core.toggle(0);
    r.core.toggle(1);
    r.render(4800);
    let after = rms(&r.render(4800));
    assert!(after < 1e-4, "camada desligada fica muda: {after}");
}

#[test]
fn sustain_pedal_holds_and_panic_cuts() {
    let mut r = rig();
    r.render(256);
    r.core.handle_piano(&[0xB0, 64, 127]);
    r.core.handle_piano(&[0x90, 60, 100]);
    r.render(2400);
    r.core.handle_piano(&[0x80, 60, 0]);
    let held = rms(&r.render(4800));
    assert!(held > 0.01, "pedal segura a nota");
    r.core.panic();
    r.render(256);
    assert!(rms(&r.render(4800)) < 1e-4);
}

#[test]
fn transpose_note_off_matches_note_on() {
    let mut r = rig();
    r.render(256);
    r.core.set_transpose(2);
    r.core.handle_piano(&[0x90, 60, 100]);
    r.render(1200);
    r.core.set_transpose(5); // mudou com a tecla apertada
    r.core.handle_piano(&[0x80, 60, 0]);
    r.render(48_000); // release
    assert!(rms(&r.render(4800)) < 1e-3, "a nota transposta foi solta");
}

#[test]
fn volume_zero_is_silent() {
    let mut r = rig();
    r.render(256);
    r.core.set_volume(0, 0.0);
    r.core.handle_piano(&[0x90, 60, 100]);
    r.render(1200);
    assert!(rms(&r.render(4800)) < 1e-4);
}

/// Carga pesada: 4 camadas, pedal, 32 notas → 128 vozes. `cargo test --release -- --ignored --nocapture`
#[test]
#[ignore]
fn bench_four_layers_full_polyphony() {
    let mut r = rig();
    for i in 0..crate::model::LAYERS {
        r.core.set_reverb(i, 0.3);
    }
    for i in 1..4 {
        r.core.toggle(i);
    }
    r.render(256);
    r.core.handle_piano(&[0xB0, 64, 127]);
    for k in 0..32u8 {
        r.core.handle_piano(&[0x90, 40 + k, 90]);
    }
    let secs = 10.0;
    let t = std::time::Instant::now();
    let mut voices = 0;
    for _ in 0..((48_000.0 * secs) as usize / 256) {
        r.render(256);
        voices = voices.max(
            r.engine
                .stats
                .voices
                .load(std::sync::atomic::Ordering::Relaxed),
        );
    }
    let used = t.elapsed().as_secs_f64() / secs;
    println!(
        "vozes={voices}  uso da CPU (1 núcleo) = {:.1}%",
        used * 100.0
    );
}

// ---------- Pads ----------

fn pad_rig() -> (Core, Engine) {
    let (tx, rx) = unbounded::<Cmd>();
    let (gtx, _grx) = unbounded();
    let mut engine = Engine::new(rx, gtx, Arc::new(Stats::default()));
    let mut core = Core::new(Config::default(), tx);
    core.set_sample_rate(48_000);
    core.sync_engine();
    let mut out = vec![0.0; 512];
    engine.render_interleaved(&mut out, 2);
    (core, engine)
}

fn render_secs(engine: &mut Engine, secs: f32) -> Vec<f32> {
    let mut all = Vec::new();
    let mut out = vec![0.0; 512];
    for _ in 0..((48_000.0 * secs) as usize / 256) {
        engine.render_interleaved(&mut out, 2);
        all.extend_from_slice(&out);
    }
    all
}

#[test]
fn builtin_pads_decode() {
    for bank in 0..crate::pads::PAD_BANKS {
        for key in [0, 7, 11] {
            let clip = crate::pads::load(bank, key, None).expect("pad padrão");
            assert!(
                clip.frames() > clip.rate as usize * 10,
                "pelo menos 10 s de loop"
            );
        }
    }
}

#[test]
fn pad_plays_without_sf2_and_stops_with_fade() {
    let (mut core, mut engine) = pad_rig();
    core.pad_press(0);
    assert_eq!(core.pad_active, Some(0));
    let start = render_secs(&mut engine, 0.2);
    let full = render_secs(&mut engine, 3.0);
    assert!(rms(&start) < rms(&full), "entra devagar");
    assert!(
        rms(&full[full.len() - 48_000..]) > 0.02,
        "pad soando: {}",
        rms(&full)
    );
    core.pad_press(0); // mesmo botão = parar
    assert_eq!(core.pad_active, None);
    let fading = render_secs(&mut engine, 0.5);
    assert!(rms(&fading) > 0.005, "some devagar, não corta seco");
    render_secs(&mut engine, 3.0);
    assert!(
        rms(&render_secs(&mut engine, 0.5)) < 1e-4,
        "depois do fade fica mudo"
    );
}

#[test]
fn pad_change_key_crossfades_and_bank_switch_keeps_key() {
    let (mut core, mut engine) = pad_rig();
    core.pad_press(2);
    render_secs(&mut engine, 3.0);
    core.pad_press(7);
    // No meio da troca não pode ter buraco.
    let mid = render_secs(&mut engine, 1.5);
    for chunk in mid.chunks(9600) {
        assert!(rms(chunk) > 0.02, "sem buraco na troca");
    }
    core.set_pad_bank(1);
    assert_eq!(core.pad_active, Some(7));
    assert_eq!(core.cfg.pad_bank, 1);
}

#[test]
fn pad_volume_and_mapping() {
    let (mut core, mut engine) = pad_rig();
    core.learn.active = true;
    core.learn.target = Some(crate::model::Target::Pad(4));
    core.handle_controller(&[0x90, 60, 127]); // nota 60 do controle = pad E
    core.learn.target = Some(crate::model::Target::PadVolume);
    core.handle_controller(&[0xB0, 30, 64]);
    core.learn.active = false;
    core.handle_controller(&[0x90, 60, 127]);
    assert_eq!(core.pad_active, Some(4));
    core.handle_controller(&[0xB0, 30, 0]);
    assert!(core.cfg.pad_volume < 0.01);
    render_secs(&mut engine, 3.0);
    assert!(
        rms(&render_secs(&mut engine, 0.5)) < 1e-3,
        "volume do pad no zero"
    );
    assert!(
        core.leds().iter().any(|&(_, n, lit)| n == 60 && lit),
        "luz do botão do pad acende"
    );
}

/// Carrega um SF2 grande pelo mesmo caminho do botão 📂 (thread + progresso).
/// `PIANO_SF2_BIG=arquivo.sf2 cargo test --release --no-default-features big_sf2 -- --ignored --nocapture`
#[test]
#[ignore]
fn big_sf2_loads_through_shared_core() {
    let Ok(path) = std::env::var("PIANO_SF2_BIG") else {
        return;
    };
    let (tx, _rx) = unbounded::<Cmd>();
    let core: crate::core::SharedCore =
        Arc::new(std::sync::Mutex::new(Core::new(Config::default(), tx)));
    let t = std::time::Instant::now();
    crate::core::load_sf2_async(core.clone(), path);
    loop {
        std::thread::sleep(std::time::Duration::from_millis(200));
        let c = core.lock().unwrap();
        if let Some(l) = &c.loading {
            let _ = l.read.load(std::sync::atomic::Ordering::Relaxed);
            continue;
        }
        assert!(c.load_error.is_none(), "{:?}", c.load_error);
        println!(
            "carregou em {:.1} s, {} timbres",
            t.elapsed().as_secs_f32(),
            c.presets.len()
        );
        break;
    }
}

#[test]
fn screen_keyboard_plays_and_lights_keys() {
    let mut r = rig();
    r.render(256);
    r.core.note_on(64, 100);
    assert!(crate::core::note_is_held(64));
    assert!(rms(&r.render(4800)) > 0.01, "nota do teclado da tela soa");
    r.core.note_off(64);
    assert!(!crate::core::note_is_held(64));
}
