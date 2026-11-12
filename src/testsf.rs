//! SF2 minúsculo gerado na hora, só para os testes do motor.

use std::sync::Arc;

use crossbeam_channel::unbounded;
use rustysynth::SoundFont;

use crate::core::Core;
use crate::engine::{Cmd, Engine, Stats};
use crate::model::{Config, Target};

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
    // fade (3 s) + cauda do reverb de igreja (~4 s)
    render_secs(&mut engine, 9.0);
    assert!(
        rms(&render_secs(&mut engine, 0.5)) < 1e-4,
        "depois do fade e da cauda do reverb fica mudo"
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

#[test]
fn vivo_pad_plays_in_key_and_switches_from_loop() {
    let (mut core, mut engine) = pad_rig();
    core.pad_press(0); // pad de loop (Escuro)
    render_secs(&mut engine, 1.0);
    core.set_pad_bank(crate::pads::PAD_BANKS); // Vivo · Coro, mesmo tom
    assert_eq!(core.pad_active, Some(0));
    assert!(core.pad_ready(crate::pads::PAD_BANKS, 0));
    let mid = render_secs(&mut engine, 2.0);
    for chunk in mid.chunks(9600) {
        assert!(rms(chunk) > 0.01, "sem buraco na troca loop → Vivo");
    }
    let full = render_secs(&mut engine, 3.0);
    assert!(full.iter().all(|x| x.is_finite() && x.abs() <= 1.0));
    assert!(rms(&full) > 0.03, "Vivo soando: {}", rms(&full));
    // Brilho/shimmer mudam ao vivo sem quebrar
    let mut v = core.cfg.vivo.clone();
    v.bright = 0.1;
    v.shimmer = 1.0;
    v.motion = 1.0;
    core.set_vivo(v);
    let changed = render_secs(&mut engine, 2.0);
    assert!(changed.iter().all(|x| x.is_finite()));
}

/// Peso do Pad Vivo + reverb de igreja (deve ser bem leve).
#[test]
#[ignore]
fn bench_vivo_and_church() {
    let (mut core, mut engine) = pad_rig();
    core.set_pad_bank(crate::pads::PAD_BANKS + 2);
    let mut v = core.cfg.vivo.clone();
    v.motion = 1.0;
    v.church_sf2 = 0.5;
    core.set_vivo(v);
    core.pad_press(3);
    render_secs(&mut engine, 1.0);
    core.pad_press(8); // dois Vivos ao mesmo tempo durante a troca
    let t = std::time::Instant::now();
    render_secs(&mut engine, 10.0);
    println!(
        "Vivo + igreja: {:.1}% de um núcleo",
        t.elapsed().as_secs_f64() / 10.0 * 100.0
    );
}

/// Abre cada SF2 de uma pasta, lista os timbres e toca uma nota (para conferir pacotes novos).
/// `PIANO_SF2_DIR=pasta cargo test --release --no-default-features check_sf2_dir -- --ignored --nocapture`
#[test]
#[ignore]
fn check_sf2_dir() {
    let Ok(dir) = std::env::var("PIANO_SF2_DIR") else {
        return;
    };
    let mut files: Vec<std::path::PathBuf> = Vec::new();
    fn walk(p: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
        for e in std::fs::read_dir(p).unwrap().flatten() {
            let p = e.path();
            if p.is_dir() {
                walk(&p, out);
            } else if p.extension().is_some_and(|x| x.eq_ignore_ascii_case("sf2")) {
                out.push(p);
            }
        }
    }
    walk(std::path::Path::new(&dir), &mut files);
    files.sort();
    for f in files {
        let t = std::time::Instant::now();
        let sf = match SoundFont::new(&mut std::io::BufReader::new(
            std::fs::File::open(&f).unwrap(),
        )) {
            Ok(s) => Arc::new(s),
            Err(e) => {
                println!("ERRO {}: {e}", f.display());
                continue;
            }
        };
        let names: Vec<String> = sf
            .get_presets()
            .iter()
            .map(|p| {
                format!(
                    "{}:{} {}",
                    p.get_bank_number(),
                    p.get_patch_number(),
                    p.get_name().trim()
                )
            })
            .collect();
        let (tx, rx) = unbounded::<Cmd>();
        let (gtx, _g) = unbounded();
        let mut engine = Engine::new(rx, gtx, Arc::new(Stats::default()));
        let mut core = Core::new(Config::default(), tx);
        core.sample_rate = 48_000;
        core.install_sound_font(sf.clone(), "x".into());
        let mut out = vec![0.0; 512];
        engine.render_interleaved(&mut out, 2);
        core.note_on(60, 100);
        let mut all = Vec::new();
        for _ in 0..80 {
            engine.render_interleaved(&mut out, 2);
            all.extend_from_slice(&out);
        }
        let mb = std::fs::metadata(&f).unwrap().len() as f64 / 1e6;
        println!(
            "OK {:>6.1} MB  {:>4.1}s  som {:.3}  {} timbres: {}  <- {}",
            mb,
            t.elapsed().as_secs_f32(),
            rms(&all),
            names.len(),
            names
                .iter()
                .take(6)
                .cloned()
                .collect::<Vec<_>>()
                .join(" | "),
            f.file_name().unwrap().to_string_lossy()
        );
        // cada timbre sozinho tem que soar
        for p in sf.get_presets() {
            let settings = rustysynth::SynthesizerSettings::new(48_000);
            let mut syn = rustysynth::Synthesizer::new(&sf, &settings).unwrap();
            syn.process_midi_message(0, 0xB0, 0, p.get_bank_number());
            syn.process_midi_message(0, 0xC0, p.get_patch_number(), 0);
            syn.note_on(0, 60, 100);
            let (mut l, mut r) = (vec![0.0; 24_000], vec![0.0; 24_000]);
            syn.render(&mut l, &mut r);
            println!(
                "   {:.3}  {}:{} {}",
                rms(&l),
                p.get_bank_number(),
                p.get_patch_number(),
                p.get_name().trim()
            );
        }
    }
}

#[test]
fn my_pad_plays_in_every_key_with_its_name() {
    // arquivo de teste: G2 + D3 (sem terça), 12 s, em WAV
    let dir = std::env::temp_dir().join(format!("piano-sf2-meupad-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("Pad + Reverse em G - Em _ 30 Minutos _ [Bx5PwBH1oGA].wav");
    let sr = 44_100u32;
    let mut wav = Vec::new();
    let n = sr as usize * 12;
    let data_len = (n * 4) as u32;
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&(36 + data_len).to_le_bytes());
    wav.extend_from_slice(b"WAVEfmt ");
    wav.extend_from_slice(&16u32.to_le_bytes());
    wav.extend_from_slice(&1u16.to_le_bytes());
    wav.extend_from_slice(&2u16.to_le_bytes());
    wav.extend_from_slice(&sr.to_le_bytes());
    wav.extend_from_slice(&(sr * 4).to_le_bytes());
    wav.extend_from_slice(&4u16.to_le_bytes());
    wav.extend_from_slice(&16u16.to_le_bytes());
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&data_len.to_le_bytes());
    for i in 0..n {
        let t = i as f32 / sr as f32;
        let x = 0.2 * (t * 98.0 * std::f32::consts::TAU).sin()
            + 0.15 * (t * 146.8 * std::f32::consts::TAU).sin();
        let v = (x * 32767.0) as i16;
        wav.extend_from_slice(&v.to_le_bytes());
        wav.extend_from_slice(&v.to_le_bytes());
    }
    std::fs::write(&path, wav).unwrap();
    let p = path.to_string_lossy().to_string();

    assert_eq!(crate::pads::name_from_file(&p), "Pad + Reverse em G - Em");
    assert_eq!(crate::pads::key_from_name(&p), Some(7));
    assert_eq!(crate::pads::key_from_name("Pad em Ab"), Some(8));
    assert_eq!(
        crate::pads::name_from_file("C# Major Pad [YLDh18SZQv0].mp3"),
        "C# Major Pad"
    );
    assert_eq!(
        crate::pads::name_from_file("Pad de adoração em Dó maior.mp3"),
        "Pad de adoração em Dó maior"
    );
    assert_eq!(
        crate::pads::name_from_file("dfe9eda4-C_Major_Pad_YLDh18SZQv0.mp3"),
        "C Major Pad"
    );
    assert_eq!(
        crate::pads::name_from_file("Pad + Reverse em C - Am _ 30 minutos _ [v9br_Qc26Rk].mp3"),
        "Pad + Reverse em C - Am"
    );
    assert!((crate::pads::my_pad_pitch(7, 7) - 1.0).abs() < 1e-6);
    assert!((crate::pads::my_pad_pitch(7, 9) - 2f32.powf(2.0 / 12.0)).abs() < 1e-5); // A: 2 acima
    assert!((crate::pads::my_pad_pitch(7, 0) - 2f32.powf(-7.0 / 12.0 + 1.0)).abs() < 1e-5); // C: 5 acima

    let (mut core, mut engine) = pad_rig();
    core.add_my_pad(p.clone(), None);
    let bank = crate::pads::MY_BANK;
    assert_eq!(core.cfg.my_pads[0].key, 7, "achou o tom (G) pelo som");
    assert_eq!(core.cfg.pad_bank, bank);
    assert!(core.pad_ready(bank, 0));
    assert_eq!(core.pad_bank_label(bank), "Pad + Reverse em G - Em");
    core.rename_my_pad(0, "Reverse G");
    assert_eq!(core.pad_bank_label(bank), "Reverse G");
    for k in [7usize, 0, 2, 11] {
        core.pad_press(k);
        let a = render_secs(&mut engine, 3.0);
        assert!(rms(&a[a.len() / 2..]) > 0.02, "Meu pad soa no tom {k}");
        assert!(a.iter().all(|x| x.is_finite() && x.abs() <= 1.0));
    }
    // a volta dos estilos passa pelo Meu pad e pula os vazios
    core.set_pad_bank(bank + 1);
    assert_eq!(core.cfg.pad_bank, 0);
    // v0.13: estilos do programa escondidos → só os pads do Theo
    core.cfg.set_hidden("pad_styles", true);
    core.add_my_pad(p.clone(), Some(1));
    core.set_pad_bank(bank + 1);
    assert_eq!(core.cfg.pad_bank, bank + 1);
    core.set_pad_bank(bank + 2); // passou do último: volta ao primeiro pad dele, não ao Escuro
    assert_eq!(core.cfg.pad_bank, bank);
    core.cfg.pad_bank = 0;
    core.pad_press(4);
    assert_eq!(core.cfg.pad_bank, bank, "Escuro escondido: toca o pad dele");
    core.remove_my_pad(1);
    core.remove_my_pad(0);
    assert!(core.cfg.my_pads.is_empty());
    let _ = std::fs::remove_dir_all(&dir);
}

/// Tom dos pads de verdade: `PIANO_SF2_PADS="a.mp3;b.mp3" cargo test --release --no-default-features real_pad_keys -- --ignored --nocapture`
#[test]
#[ignore]
fn real_pad_keys() {
    let Ok(list) = std::env::var("PIANO_SF2_PADS") else {
        return;
    };
    for f in list.split(';') {
        let t = std::time::Instant::now();
        let clip = crate::pads::load_my_pad(f).unwrap();
        let k = crate::pads::detect_key(&clip);
        println!(
            "{} ({:.1}s, {:.0} MB): {}",
            f,
            t.elapsed().as_secs_f32(),
            clip.data.len() as f64 * 2.0 / 1e6,
            crate::pads::KEY_NAMES[k]
        );
    }
}

#[test]
fn old_config_hides_builtin_pads_once() {
    let mut c = Config::default();
    c.normalize();
    assert!(c.is_hidden("pad_styles"));
    c.set_hidden("pad_styles", false); // Theo trouxe de volta no Editar
    c.normalize();
    assert!(!c.is_hidden("pad_styles"), "não esconde de novo");
    // v0.20: 3 da parceria + 3 pads livres (MuseScore General)
    assert_eq!(c.my_pads.len(), 6, "pads da parceria e os livres já colocados");
    assert_eq!(c.pad_bank, crate::pads::MY_BANK);
    for (n, p) in crate::pads::PARTNER_PADS.iter().enumerate() {
        let clip = crate::pads::load_my_pad(&crate::pads::partner_path(n)).unwrap();
        assert_eq!(crate::pads::detect_key(&clip), p.key, "{}", p.name);
        assert!(clip.frames() > 44_100 * 80);
    }
}

#[test]
fn free_sounds_come_inside_the_program() {
    // os 3 pads livres abrem, estão em C e são longos
    for (n, p) in crate::pads::FREE_PADS.iter().enumerate() {
        let clip = crate::pads::load_my_pad(&crate::pads::free_path(n)).unwrap();
        assert_eq!(crate::pads::detect_key(&clip), p.key, "{} está em C", p.name);
        assert!(clip.frames() > 48_000 * 55, "{} tem ~1 min", p.name);
    }
    // config antiga ganha os pads livres uma vez só, sem apagar os do Theo
    let mut c = Config::default();
    c.my_pads.push(crate::model::MyPad {
        name: "Meu pad do YouTube".into(),
        path: "C:/pads/meu.mp3".into(),
        key: 7,
        auto_key: false,
    });
    c.pads_v13 = true;
    c.normalize();
    assert_eq!(c.my_pads.len(), 4);
    assert_eq!(c.my_pads[0].name, "Meu pad do YouTube");
    c.my_pads.retain(|m| m.name != "Marte"); // Theo tirou um
    c.normalize();
    assert_eq!(c.my_pads.len(), 3, "não volta sozinho depois que o Theo tirou");
    // bateria gravada: os 8 sons abrem (nenhum caiu no sintetizado) e são diferentes dos sintetizados
    let real = crate::efeitos::DrumKit::build(48_000);
    let synth = crate::efeitos::DrumKit::build_synth(48_000);
    for i in 0..crate::efeitos::DRUMS {
        assert_ne!(real.hits[i].0.len(), synth.hits[i].0.len(), "{} é o gravado", crate::efeitos::DRUM_NAMES[i]);
    }
    // coral do Voice FX gravado, em C3
    let choir = crate::pads::load_voicefx_choir().unwrap();
    assert_eq!(choir.root, 48);
    assert!(choir.data.len() > 48_000 * 9);
}

/// Prévia para ouvir os "Meus pads" em vários tons:
/// `PIANO_SF2_PADS="a.mp3;b.mp3" PIANO_SF2_OUT=pasta cargo test --release --no-default-features preview_my_pads -- --ignored --nocapture`
#[test]
#[ignore]
fn preview_my_pads() {
    let (Ok(list), Ok(out)) = (
        std::env::var("PIANO_SF2_PADS"),
        std::env::var("PIANO_SF2_OUT"),
    ) else {
        return;
    };
    for (n, f) in list.split(';').enumerate() {
        let (mut core, mut engine) = pad_rig();
        core.add_my_pad(f.to_string(), None);
        let root = core.cfg.my_pads[0].key;
        let mut all = Vec::new();
        // tom original, 2 acima, 2 abaixo, 5 acima (o máximo que muda é 6)
        for d in [0i32, 2, -2, 5] {
            let k = (root as i32 + d).rem_euclid(12) as usize;
            core.pad_press(k);
            all.extend(render_secs(&mut engine, 9.0));
            println!("{}: {} ", n, crate::pads::KEY_NAMES[k]);
        }
        core.pad_stop();
        all.extend(render_secs(&mut engine, 3.5));
        let path = format!("{out}/pad{}.wav", n + 1);
        write_wav(&path, &all, 48_000);
    }
}

fn write_wav(path: &str, stereo: &[f32], sr: u32) {
    let data_len = (stereo.len() * 2) as u32;
    let mut w = Vec::with_capacity(44 + data_len as usize);
    w.extend_from_slice(b"RIFF");
    w.extend_from_slice(&(36 + data_len).to_le_bytes());
    w.extend_from_slice(b"WAVEfmt ");
    w.extend_from_slice(&16u32.to_le_bytes());
    w.extend_from_slice(&1u16.to_le_bytes());
    w.extend_from_slice(&2u16.to_le_bytes());
    w.extend_from_slice(&sr.to_le_bytes());
    w.extend_from_slice(&(sr * 4).to_le_bytes());
    w.extend_from_slice(&4u16.to_le_bytes());
    w.extend_from_slice(&16u16.to_le_bytes());
    w.extend_from_slice(b"data");
    w.extend_from_slice(&data_len.to_le_bytes());
    for x in stereo {
        w.extend_from_slice(&((x.clamp(-1.0, 1.0) * 32767.0) as i16).to_le_bytes());
    }
    std::fs::write(path, w).unwrap();
}

/// v0.16: Congelar com um SF2 de verdade. Toca um Sol no piano, congela, solta as teclas,
/// troca o timbre e confere que o congelado continua firme (sem buraco nem estalo).
/// `PIANO_SF2_FREEZE=<arquivo.sf2>` (e opcional `PIANO_FREEZE_WAV=<saida.wav>` para ouvir).
#[test]
#[ignore]
fn freeze_real_sf2_preview() {
    let Ok(path) = std::env::var("PIANO_SF2_FREEZE") else {
        return;
    };
    let sf = Arc::new(SoundFont::new(&mut std::fs::File::open(&path).unwrap()).unwrap());
    let (tx, rx) = unbounded::<Cmd>();
    let (gtx, _grx) = unbounded();
    let mut engine = Engine::new(rx, gtx, Arc::new(Stats::default()));
    let mut core = Core::new(Config::default(), tx);
    core.sample_rate = 48_000;
    core.install_sound_font(sf, "real.sf2".into());
    let piano = core.presets[0].clone();
    core.set_preset(0, piano.clone());
    core.set_on(0, true);
    println!("timbre: {}", piano.name);
    let mut wav: Vec<f32> = Vec::new();
    let mut render = |engine: &mut Engine, secs: f32, wav: &mut Vec<f32>| -> Vec<f32> {
        let mut out = vec![0.0; (48_000.0 * secs) as usize * 2];
        for chunk in out.chunks_mut(1024) {
            engine.render_interleaved(chunk, 2);
        }
        wav.extend_from_slice(&out);
        out
    };
    render(&mut engine, 0.1, &mut wav);
    for k in [55u8, 59, 62, 67] {
        core.handle_piano(&[0x90, k, 100]);
    }
    let played = render(&mut engine, 1.5, &mut wav);
    {
        let s = crate::freeze::FreezeSource::capture(48_000).unwrap();
        println!("trecho: {:.2} s, rms {:.4}", s.seconds(), s.rms);
    }
    core.freeze_capture();
    assert!(core.freeze_on, "congelou");
    for k in [55u8, 59, 62, 67] {
        core.handle_piano(&[0x80, k, 0]);
    }
    if let Some(other) = core.presets.get(core.presets.len() / 2).cloned() {
        println!("troca para: {}", other.name);
        core.set_preset(0, other);
    }
    let frozen = render(&mut engine, 12.0, &mut wav);
    core.freeze_toggle();
    let tail = render(&mut engine, 4.0, &mut wav);

    let half = 48_000; // 0,5 s estéreo
    let levels: Vec<f32> = frozen.chunks(half).map(rms).collect();
    let jump = frozen
        .chunks(2)
        .collect::<Vec<_>>()
        .windows(2)
        .map(|w| (w[1][0] - w[0][0]).abs())
        .fold(0.0f32, f32::max);
    println!("acorde tocado: {:.4}", rms(&played[played.len() / 2..]));
    println!("congelado (a cada 0,5 s): {:?}", levels.iter().map(|v| (v * 1000.0).round() / 1000.0).collect::<Vec<_>>());
    println!("maior salto entre amostras: {jump:.4}   fim: {:.6}", rms(&tail[tail.len() - 4800..]));
    let (lo, hi) = levels[2..].iter().fold((f32::MAX, 0.0f32), |(a, b), &v| (a.min(v), b.max(v)));
    assert!(lo > 0.0 && hi / lo < 2.5, "volume firme ({lo}..{hi})");
    let now = rms(&played[played.len() - 28_800..]); // último 0,3 s antes de apertar
    let avg = levels[2..].iter().sum::<f32>() / (levels.len() - 2) as f32;
    println!("volume ao apertar: {now:.4}  congelado: {avg:.4}");
    assert!(avg > now * 0.6 && avg < now * 1.6, "mesmo volume do que se ouvia");
    assert!(rms(&tail[tail.len() - 4800..]) < 1e-4, "sumiu no fim");
    if let Ok(out) = std::env::var("PIANO_FREEZE_WAV") {
        let mut b: Vec<u8> = Vec::new();
        let data_len = (wav.len() * 2) as u32;
        b.extend(b"RIFF");
        b.extend((36 + data_len).to_le_bytes());
        b.extend(b"WAVEfmt ");
        b.extend(16u32.to_le_bytes());
        b.extend(1u16.to_le_bytes());
        b.extend(2u16.to_le_bytes());
        b.extend(48_000u32.to_le_bytes());
        b.extend((48_000u32 * 4).to_le_bytes());
        b.extend(4u16.to_le_bytes());
        b.extend(16u16.to_le_bytes());
        b.extend(b"data");
        b.extend(data_len.to_le_bytes());
        for s in &wav {
            b.extend(((s.clamp(-1.0, 1.0) * 32767.0) as i16).to_le_bytes());
        }
        std::fs::write(out, b).unwrap();
    }
}

// ---------- v0.17: aba Efeitos ----------

#[test]
fn arpeggiator_plays_chord_notes_on_the_beat() {
    let mut r = rig();
    r.core.cfg.efeitos.arp_layer = 0;
    r.core.cfg.efeitos.arp_octaves = 2;
    r.core.cfg.efeitos.arp_per_beat = 2;
    r.core.set_bpm(120.0); // colcheia a 120 = 0,25 s = 12000 amostras
    r.core.toggle_arp();
    r.render(64);
    for k in [60u8, 64, 67] {
        r.core.handle_piano(&[0x90, k, 100]);
    }
    let mut seq = Vec::new();
    // logo depois de cada passo (o passo começa quando a 1ª tecla entrou)
    r.render(16);
    seq.push(r.engine.test_arp_playing());
    for _ in 0..5 {
        r.render(12_000);
        seq.push(r.engine.test_arp_playing());
    }
    assert_eq!(
        seq,
        vec![Some(60), Some(64), Some(67), Some(72), Some(76), Some(79)],
        "sobe pelo acorde e pela oitava de cima, uma nota por colcheia"
    );
    // soltou (sem "segurar"): o arpejo para
    for k in [60u8, 64, 67] {
        r.core.handle_piano(&[0x80, k, 0]);
    }
    r.render(12_000);
    assert_eq!(r.engine.test_arp_playing(), None);
}

#[test]
fn arpeggiator_latch_keeps_going_after_release() {
    let mut r = rig();
    r.core.cfg.efeitos.arp_layer = 0;
    r.core.cfg.efeitos.arp_octaves = 1;
    r.core.cfg.efeitos.arp_latch = true;
    r.core.set_bpm(120.0);
    r.core.toggle_arp();
    r.render(64);
    r.core.handle_piano(&[0x90, 60, 100]);
    r.core.handle_piano(&[0x90, 67, 100]);
    r.core.handle_piano(&[0x80, 60, 0]);
    r.core.handle_piano(&[0x80, 67, 0]);
    r.render(16);
    let a = r.engine.test_arp_playing();
    r.render(12_000);
    let b = r.engine.test_arp_playing();
    assert_eq!((a, b), (Some(60), Some(67)), "segurou o acorde");
    r.core.panic();
    r.render(12_100);
    assert_eq!(r.engine.test_arp_playing(), None, "pânico para o arpejo");
}

#[test]
fn pulse_makes_the_frozen_sound_breathe_in_time() {
    let mut r = rig();
    r.render(64);
    r.core.handle_piano(&[0x90, 60, 100]);
    r.render(48_000);
    r.core.freeze_capture();
    assert!(r.core.freeze_on);
    r.core.handle_piano(&[0x80, 60, 0]);
    r.core.set_bpm(60.0); // 1 pulso por tempo a 60 = 1 s
    r.core.cfg.efeitos.pulse_per_beat = 1;
    r.core.cfg.efeitos.pulse_depth = 1.0;
    r.core.toggle_pulse();
    r.render(96_000); // deixa a nota tocada sumir
    r.core.tap_tempo(); // pulso recomeça agora
    r.render(64);
    let out = r.render(48_000);
    let lvl = |a: usize, b: usize| {
        let s = &out[a * 2..b * 2];
        (s.iter().map(|x| x * x).sum::<f32>() / s.len() as f32).sqrt()
    };
    let start = lvl(2_000, 8_000); // logo depois do "tum"
    let end = lvl(40_000, 47_000); // fim do pulso
    assert!(start > end * 4.0, "pulsa: começo {start} > fim {end}");
}

#[test]
fn voice_fx_choir_follows_notes_with_the_real_texture() {
    let mut r = rig();
    r.core.toggle_voicefx();
    assert!(r.core.voicefx_on);
    r.render(64);
    r.core.handle_piano(&[0x90, 60, 100]);
    r.core.handle_piano(&[0x90, 64, 100]);
    let out = r.render(24_000);
    assert_eq!(r.engine.test_choir_voices(), 2);
    assert!(rms(&out) > 0.005);
    r.core.handle_piano(&[0x80, 60, 0]);
    r.core.handle_piano(&[0x80, 64, 0]);
    r.render(60_000);
    assert_eq!(r.engine.test_choir_voices(), 0, "soltou: o coral some");
    r.core.toggle_voicefx();
    r.core.handle_piano(&[0x90, 62, 100]);
    r.render(4_800);
    assert_eq!(r.engine.test_choir_voices(), 0, "desligado não canta");
}

#[test]
fn drum_pads_play_from_the_screen_and_from_an_amw_key_with_velocity() {
    let mut r = rig();
    r.render(64);
    r.core.panic(); // nada do SF2 tocando
    r.render(4_800);
    r.core.drum_hit(0, 127);
    let hard = rms(&r.render(4_800));
    assert!(hard > 0.05, "bumbo soou: {hard}");
    r.render(48_000);
    // AMW: tecla 36 aprendida como Palmas
    r.core.learn.active = true;
    r.core.learn.target = Some(Target::Drum(2));
    r.core.handle_piano(&[0x90, 36, 100]);
    r.core.learn.active = false;
    r.render(24_000);
    r.core.handle_piano(&[0x90, 36, 30]);
    let soft = rms(&r.render(4_800));
    r.render(48_000);
    r.core.handle_piano(&[0x90, 36, 127]);
    let loud = rms(&r.render(4_800));
    assert!(soft > 0.001 && loud > soft * 2.0, "força do dedo: {soft} → {loud}");
}

/// v0.17: prévia de tudo da aba Efeitos com um SF2 de verdade + quanto pesa no processador.
/// `PIANO_SF2_FREEZE=<arquivo.sf2>` e `PIANO_EFEITOS_WAV=<saida.wav>`.
#[test]
#[ignore]
fn efeitos_real_sf2_preview() {
    let Ok(path) = std::env::var("PIANO_SF2_FREEZE") else {
        return;
    };
    let sf = Arc::new(SoundFont::new(&mut std::fs::File::open(&path).unwrap()).unwrap());
    let (tx, rx) = unbounded::<Cmd>();
    let (gtx, _grx) = unbounded();
    let mut engine = Engine::new(rx, gtx, Arc::new(Stats::default()));
    let mut core = Core::new(Config::default(), tx);
    core.sample_rate = 48_000;
    core.install_sound_font(sf, "real.sf2".into());
    let find = |c: &Core, words: &[&str]| {
        c.presets
            .iter()
            .find(|p| words.iter().any(|w| p.name.to_lowercase().contains(w)))
            .cloned()
            .unwrap_or_else(|| c.presets[0].clone())
    };
    let piano = core.presets[0].clone();
    let bells = find(&core, &["sino", "bell", "harpa"]);
    let pad = find(&core, &["pad", "cord", "string"]);
    println!("piano: {}  arpejo: {}  pad: {}", piano.name, bells.name, pad.name);
    core.set_preset(0, piano);
    core.set_on(0, true);
    core.set_preset(1, bells);
    core.set_on(1, true);
    core.cfg.efeitos.arp_layer = 1;
    core.set_bpm(84.0);
    core.cfg.efeitos.arp_per_beat = 4;
    core.toggle_arp();

    let mut wav: Vec<f32> = Vec::new();
    let mut busy = std::time::Duration::ZERO;
    let mut render = |engine: &mut Engine, secs: f32, wav: &mut Vec<f32>| {
        let mut out = vec![0.0; (48_000.0 * secs) as usize * 2];
        let t = std::time::Instant::now();
        for chunk in out.chunks_mut(512) {
            engine.render_interleaved(chunk, 2);
        }
        busy += t.elapsed();
        wav.extend_from_slice(&out);
    };
    let chord = [43u8, 55, 59, 62];
    // 1) Sol com piano + arpejo de sinos
    for k in chord {
        core.handle_piano(&[0x90, k, 95]);
    }
    render(&mut engine, 4.0, &mut wav);
    // 2) congela com um pad e liga o PulsePad
    core.set_preset(0, pad);
    render(&mut engine, 1.0, &mut wav);
    core.freeze_capture();
    for k in chord {
        core.handle_piano(&[0x80, k, 0]);
    }
    core.toggle_arp();
    core.cfg.efeitos.pulse_per_beat = 2;
    core.toggle_pulse();
    core.tap_tempo();
    let beat = 60.0 / 84.0;
    // 3) bateria por cima (bumbo, palmas, chimbal) por 2 compassos
    for b in 0..8 {
        core.drum_hit(if b % 2 == 0 { 0 } else { 2 }, 110);
        render(&mut engine, beat / 2.0, &mut wav);
        core.drum_hit(3, 70);
        render(&mut engine, beat / 2.0, &mut wav);
    }
    // 4) Voice FX: coral com o piano num Mi menor, congelado ainda pulsando
    core.set_preset(0, core.presets[0].clone());
    core.toggle_voicefx();
    for k in [52u8, 59, 64, 67] {
        core.handle_piano(&[0x90, k, 90]);
    }
    render(&mut engine, 4.0, &mut wav);
    for k in [52u8, 59, 64, 67] {
        core.handle_piano(&[0x80, k, 0]);
    }
    core.drum_hit(7, 120);
    core.freeze_toggle();
    render(&mut engine, 4.0, &mut wav);

    let secs = wav.len() as f32 / 2.0 / 48_000.0;
    println!(
        "{secs:.1} s de áudio em {:.2} s ({:.1}% de um núcleo deste PC)",
        busy.as_secs_f32(),
        busy.as_secs_f32() / secs * 100.0
    );
    let peak = wav.iter().fold(0.0f32, |m, x| m.max(x.abs()));
    println!("pico: {peak:.3}");
    assert!(peak < 1.0);
    if let Ok(out) = std::env::var("PIANO_EFEITOS_WAV") {
        let mut b: Vec<u8> = Vec::new();
        let data_len = (wav.len() * 2) as u32;
        b.extend(b"RIFF");
        b.extend((36 + data_len).to_le_bytes());
        b.extend(b"WAVEfmt ");
        b.extend(16u32.to_le_bytes());
        b.extend(1u16.to_le_bytes());
        b.extend(2u16.to_le_bytes());
        b.extend(48_000u32.to_le_bytes());
        b.extend((48_000u32 * 4).to_le_bytes());
        b.extend(4u16.to_le_bytes());
        b.extend(16u16.to_le_bytes());
        b.extend(b"data");
        b.extend(data_len.to_le_bytes());
        for s in &wav {
            b.extend(((s.clamp(-1.0, 1.0) * 32767.0) as i16).to_le_bytes());
        }
        std::fs::write(out, b).unwrap();
    }
}

// ---------- v0.19: sons próprios nos DrumPads e no Voice FX ----------

/// Escreve um WAV estéreo 16 bits de teste e devolve o caminho.
fn write_test_wav(name: &str, rate: u32, samples: &[f32]) -> String {
    let dir = std::env::temp_dir().join(format!("piano-sf2-teste-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join(name);
    let data_len = (samples.len() * 4) as u32;
    let mut b: Vec<u8> = Vec::new();
    b.extend(b"RIFF");
    b.extend((36 + data_len).to_le_bytes());
    b.extend(b"WAVEfmt ");
    b.extend(16u32.to_le_bytes());
    b.extend(1u16.to_le_bytes());
    b.extend(2u16.to_le_bytes());
    b.extend(rate.to_le_bytes());
    b.extend((rate * 4).to_le_bytes());
    b.extend(4u16.to_le_bytes());
    b.extend(16u16.to_le_bytes());
    b.extend(b"data");
    b.extend(data_len.to_le_bytes());
    for s in samples {
        let v = (s.clamp(-1.0, 1.0) * 32767.0) as i16;
        b.extend(v.to_le_bytes());
        b.extend(v.to_le_bytes());
    }
    std::fs::write(&path, b).unwrap();
    path.to_string_lossy().to_string()
}

#[test]
fn drum_pad_uses_my_own_wav_and_goes_back() {
    let mut r = rig();
    // "caixa" de 44,1 kHz com 0,2 s de silêncio antes da batida
    let rate = 44_100;
    let mut s = vec![0.0f32; rate as usize / 5];
    s.extend((0..rate as usize / 2).map(|i| {
        let t = i as f32 / rate as f32;
        (std::f32::consts::TAU * 330.0 * t).sin() * (-t * 12.0).exp() * 0.8
    }));
    let path = write_test_wav("minha_caixa.wav", rate, &s);
    r.render(64);
    r.core.panic();
    r.core.set_drum_file(1, Some(path.clone()));
    assert_eq!(r.core.drum_file_name(1).as_deref(), Some("minha caixa"));
    assert_eq!(r.core.cfg.efeitos.drum_files.len(), 1);
    r.render(48_000);
    r.core.drum_hit(1, 127);
    // o silêncio do começo foi cortado: a batida sai na hora
    let first = r.render(960); // 20 ms
    assert!(rms(&first) > 0.05, "bateu na hora: {}", rms(&first));
    // voltar ao som do programa
    r.core.set_drum_file(1, None);
    assert!(r.core.drum_file_name(1).is_none());
    assert!(r.core.cfg.efeitos.drum_files.is_empty());
    // arquivo que não existe: avisa e não muda nada
    r.core.set_drum_file(2, Some("C:/nao/existe.wav".into()));
    assert!(r.core.drum_file_name(2).is_none());
    assert!(r.core.toast.as_ref().unwrap().0.contains("Não consegui"));
}

#[test]
fn drum_file_that_disappeared_falls_back_with_warning() {
    let mut r = rig();
    let s: Vec<f32> = (0..4800).map(|i| (i as f32 * 0.05).sin() * 0.5).collect();
    let path = write_test_wav("some.wav", 48_000, &s);
    r.core.set_drum_file(0, Some(path.clone()));
    assert!(r.core.drum_file_name(0).is_some());
    // o arquivo sumiu e o programa foi aberto de novo (taxa da placa diferente obriga a remontar)
    std::fs::remove_file(&path).unwrap();
    r.core.sample_rate = 44_100;
    r.core.ensure_drum_kit();
    assert!(r.core.drum_file_name(0).is_none(), "voltou ao som do programa");
    assert!(r.core.toast.as_ref().unwrap().0.contains("voltou ao som do programa"));
    r.core.sample_rate = 48_000;
}

#[test]
fn voice_fx_uses_my_choir_and_finds_its_note() {
    let mut r = rig();
    // "coral" em Lá (A2 + A3 + E4 = tônica e quinta), 3 s
    let rate = 48_000;
    let s: Vec<f32> = (0..rate as usize * 3)
        .map(|i| {
            let t = i as f32 / rate as f32;
            [110.0f32, 220.0, 329.63]
                .iter()
                .map(|f| (std::f32::consts::TAU * f * t).sin())
                .sum::<f32>()
                * 0.2
        })
        .collect();
    let path = write_test_wav("meu_coral_em_la.wav", rate, &s);
    r.core.set_voicefx_file(Some(path.clone()));
    let (p, root) = r.core.cfg.efeitos.voicefx_file.clone().unwrap();
    assert_eq!(p, path);
    assert_eq!(root % 12, 9, "descobriu que está em Lá");
    // corrigir a nota à mão
    r.core.set_voicefx_key(0);
    assert_eq!(r.core.cfg.efeitos.voicefx_file.as_ref().unwrap().1 % 12, 0);
    // toca com o coral novo
    r.core.toggle_voicefx();
    r.render(64);
    r.core.handle_piano(&[0x90, 60, 100]);
    let out = r.render(24_000);
    assert_eq!(r.engine.test_choir_voices(), 1);
    assert!(rms(&out) > 0.005);
    // voltar ao coral do programa
    r.core.set_voicefx_file(None);
    assert!(r.core.cfg.efeitos.voicefx_file.is_none());
}

// ---------- v0.20: gerar os sons "de verdade" a partir do MuseScore General (MIT) ----------

/// Ferramenta (não é teste de verdade): toca o MuseScore_General.sf2 e grava os WAV que viram os sons
/// do programa (8 DrumPads, o coral do Voice FX e 3 pads em C).
/// `PIANO_SF2_MUSESCORE=<MuseScore_General.sf2>` `PIANO_RENDER_DIR=<pasta de saída>`
#[test]
#[ignore]
fn render_sons_livres() {
    use rustysynth::{Synthesizer, SynthesizerSettings};
    let (Ok(path), Ok(out)) = (std::env::var("PIANO_SF2_MUSESCORE"), std::env::var("PIANO_RENDER_DIR")) else {
        return;
    };
    std::fs::create_dir_all(&out).unwrap();
    let sf = Arc::new(SoundFont::new(&mut std::io::BufReader::new(std::fs::File::open(&path).unwrap())).unwrap());
    let sr = 48_000u32;
    for p in sf.get_presets().iter().filter(|p| p.get_bank_number() == 128 || [47, 48, 49, 52, 53, 88, 89, 91, 94, 95, 119].contains(&p.get_patch_number()) && p.get_bank_number() == 0) {
        println!("preset {:>3}:{:<3} {}", p.get_bank_number(), p.get_patch_number(), p.get_name());
    }
    let new_synth = || {
        let mut s = SynthesizerSettings::new(sr as i32);
        s.enable_reverb_and_chorus = true;
        s.maximum_polyphony = 128;
        Synthesizer::new(&sf, &s).unwrap()
    };
    let render = |s: &mut Synthesizer, secs: f32, l: &mut Vec<f32>, r: &mut Vec<f32>| {
        let n = (secs * sr as f32) as usize;
        let (mut bl, mut br) = (vec![0.0f32; 512], vec![0.0f32; 512]);
        let mut done = 0;
        while done < n {
            let k = (n - done).min(512);
            s.render(&mut bl[..k], &mut br[..k]);
            l.extend_from_slice(&bl[..k]);
            r.extend_from_slice(&br[..k]);
            done += k;
        }
    };
    let write = |name: &str, l: &[f32], r: &[f32]| {
        let data_len = (l.len() * 4) as u32;
        let mut b: Vec<u8> = Vec::new();
        b.extend(b"RIFF");
        b.extend((36 + data_len).to_le_bytes());
        b.extend(b"WAVEfmt ");
        b.extend(16u32.to_le_bytes());
        b.extend(1u16.to_le_bytes());
        b.extend(2u16.to_le_bytes());
        b.extend(sr.to_le_bytes());
        b.extend((sr * 4).to_le_bytes());
        b.extend(4u16.to_le_bytes());
        b.extend(16u16.to_le_bytes());
        b.extend(b"data");
        b.extend(data_len.to_le_bytes());
        for (a, c) in l.iter().zip(r) {
            b.extend(((a.clamp(-1.0, 1.0) * 32767.0) as i16).to_le_bytes());
            b.extend(((c.clamp(-1.0, 1.0) * 32767.0) as i16).to_le_bytes());
        }
        std::fs::write(std::path::Path::new(&out).join(name), b).unwrap();
        let peak = l.iter().chain(r).fold(0.0f32, |m, x| m.max(x.abs()));
        println!("{name}: {:.2} s, pico {peak:.2}", l.len() as f32 / sr as f32);
    };
    // corta o fim quando já está em silêncio e normaliza o pico
    let finish = |l: &mut Vec<f32>, r: &mut Vec<f32>, peak_to: f32| {
        let last = l.iter().zip(r.iter()).rposition(|(a, b)| a.abs() > 0.0008 || b.abs() > 0.0008).unwrap_or(0);
        let end = (last + sr as usize / 50).min(l.len());
        l.truncate(end);
        r.truncate(end);
        let peak = l.iter().chain(r.iter()).fold(0.0f32, |m, x| m.max(x.abs()));
        if peak > 0.0 {
            let g = peak_to / peak;
            l.iter_mut().chain(r.iter_mut()).for_each(|x| *x *= g);
        }
    };

    // ---- DrumPads: (nome, banco, programa, nota, força, segundos, reverb) ----
    // 0 Bumbo, 1 Caixa, 2 Palmas, 3 Chimbal, 4 Prato, 5 808, 6 Subida, 7 Impacto
    let hits: [(&str, &[(i32, i32, i32, i32)], f32, i32); 8] = [
        ("drum_00_bumbo.wav", &[(128, 0, 36, 118)], 0.9, 0),
        ("drum_01_caixa.wav", &[(128, 0, 38, 112)], 0.9, 10),
        ("drum_02_palmas.wav", &[(128, 0, 39, 115)], 0.9, 20),
        ("drum_03_chimbal.wav", &[(128, 0, 42, 100)], 0.6, 0),
        ("drum_04_prato.wav", &[(128, 0, 49, 112)], 4.0, 20),
        ("drum_05_808.wav", &[(128, 25, 36, 120)], 2.5, 0),
        ("drum_06_subida.wav", &[(0, 119, 60, 120), (0, 119, 72, 90)], 4.0, 40),
        ("drum_07_impacto.wav", &[(0, 47, 36, 127), (128, 0, 35, 127), (128, 0, 49, 110), (128, 48, 59, 127)], 4.0, 60),
    ];
    for (name, layers, secs, rev) in hits {
        let mut s = new_synth();
        for (ch, (bank, patch, note, vel)) in layers.iter().enumerate() {
            let ch = ch as i32;
            s.set_channel_preset(ch, *bank, *patch);
            s.process_midi_message(ch, 0xB0, 0x5B, rev);
            s.process_midi_message(ch, 0xB0, 0x5D, 0);
            s.note_on(ch, *note, *vel);
        }
        let (mut l, mut r) = (Vec::new(), Vec::new());
        render(&mut s, secs, &mut l, &mut r);
        finish(&mut l, &mut r, 0.9);
        write(name, &l, &r);
    }

    // ---- v0.21: as 4 vozes do Voice FX, separadas (C3, segurado, sem o começo) ----
    for (name, patch, rev) in [("voz_00_aah.wav", 52, 40), ("voz_01_ooh.wav", 53, 40), ("voz_02_celeste.wav", 91, 60), ("voz_03_synth.wav", 54, 40)] {
        let mut s = new_synth();
        s.set_channel_preset(0, 0, patch);
        s.process_midi_message(0, 0xB0, 0x5B, rev);
        s.note_on(0, 48, 100);
        let (mut l, mut r) = (Vec::new(), Vec::new());
        render(&mut s, 13.0, &mut l, &mut r);
        let a = (sr as f32 * 1.5) as usize;
        let b = (sr as f32 * 12.5) as usize;
        let (mut l, mut r) = (l[a..b].to_vec(), r[a..b].to_vec());
        finish(&mut l, &mut r, 0.7);
        write(name, &l, &r);
    }

    // ---- Coral do Voice FX (v0.20): "aah" em C3 (48), segurado, sem o começo ----
    {
        let mut s = new_synth();
        s.set_channel_preset(0, 0, 52); // Choir Aahs
        s.set_channel_preset(1, 0, 53); // Voice Oohs
        s.process_midi_message(1, 0xB0, 0x07, 70);
        for ch in 0..2 {
            s.process_midi_message(ch, 0xB0, 0x5B, 40);
            s.note_on(ch, 48, 100);
        }
        let (mut l, mut r) = (Vec::new(), Vec::new());
        render(&mut s, 13.0, &mut l, &mut r);
        let a = (sr as f32 * 1.5) as usize;
        let b = (sr as f32 * 12.5) as usize;
        let (mut l, mut r) = (l[a..b].to_vec(), r[a..b].to_vec());
        finish(&mut l, &mut r, 0.7);
        write("coral_c3.wav", &l, &r);
    }

    // ---- 3 pads em C (sem terça: serve para C e Cm), com reverb de igreja ----
    let pads: [(&str, &[(i32, i32)], &[i32]); 3] = [
        ("pad_00_coral_celeste.wav", &[(52, 100), (53, 80), (91, 70)], &[48, 55, 60, 67]),
        ("pad_01_cordas_suaves.wav", &[(49, 100), (48, 70), (89, 75)], &[36, 48, 55, 60, 67]),
        ("pad_02_atmosfera.wav", &[(88, 80), (94, 90), (95, 55), (49, 60)], &[36, 48, 55, 60, 67, 72]),
    ];
    for (name, layers, notes) in pads {
        let mut s = new_synth();
        for (ch, (patch, vol)) in layers.iter().enumerate() {
            let ch = ch as i32;
            s.set_channel_preset(ch, 0, *patch);
            s.process_midi_message(ch, 0xB0, 0x07, *vol);
            s.process_midi_message(ch, 0xB0, 0x5B, 90);
            for n in notes.iter() {
                s.note_on(ch, *n, 82);
            }
        }
        let (mut dl, mut dr) = (Vec::new(), Vec::new());
        render(&mut s, 72.0, &mut dl, &mut dr);
        // reverb de igreja do próprio programa por cima
        let mut rev = crate::granular::ChurchReverb::new(sr);
        let (mut l, mut r) = (dl.clone(), dr.clone());
        for (ci, chunk) in (0..dl.len()).step_by(512).enumerate() {
            let _ = ci;
            let e = (chunk + 512).min(dl.len());
            let (inl, inr) = (dl[chunk..e].to_vec(), dr[chunk..e].to_vec());
            rev.process(&inl, &inr, &mut l[chunk..e], &mut r[chunk..e], 0.7);
        }
        let a = sr as usize * 7;
        let b = a + sr as usize * 62;
        let (mut l, mut r) = (l[a..b].to_vec(), r[a..b].to_vec());
        finish(&mut l, &mut r, 0.8);
        write(name, &l, &r);
    }
}

// ---------- v0.21: escolher a voz do Voice FX e nomes de planeta nos pads ----------

#[test]
fn voice_fx_voice_can_be_chosen_while_playing() {
    let mut r = rig();
    r.core.toggle_voicefx();
    r.render(64);
    for v in 0..crate::pads::VOICE_NAMES.len() {
        r.core.set_voicefx_voice(v);
        assert_eq!(r.core.cfg.efeitos.voicefx_voice, v);
        r.core.handle_piano(&[0x90, 60, 100]);
        let out = r.render(24_000);
        assert!(rms(&out) > 0.003, "voz {} canta", crate::pads::VOICE_NAMES[v]);
        r.core.handle_piano(&[0x80, 60, 0]);
        r.render(60_000);
    }
    // "Minha" sem arquivo: avisa e não troca
    r.core.set_voicefx_voice(crate::model::VOICE_MINE);
    assert_ne!(r.core.cfg.efeitos.voicefx_voice, crate::model::VOICE_MINE);
    assert!(r.core.toast.as_ref().unwrap().0.contains("Minha"));
    // escolhendo um coral próprio, "Minha" já fica selecionada; tirando, volta ao Coral
    let s: Vec<f32> = (0..48_000 * 3).map(|i| (std::f32::consts::TAU * 130.81 * i as f32 / 48_000.0).sin() * 0.3).collect();
    let path = write_test_wav("coral_meu.wav", 48_000, &s);
    r.core.set_voicefx_file(Some(path));
    assert_eq!(r.core.cfg.efeitos.voicefx_voice, crate::model::VOICE_MINE);
    r.core.set_voicefx_voice(2);
    assert_eq!(r.core.cfg.efeitos.voicefx_voice, 2, "dá para voltar a uma voz do programa");
    r.core.set_voicefx_file(None);
    assert_eq!(r.core.cfg.efeitos.voicefx_voice, 0);
}

#[test]
fn pads_get_planet_names_once_and_keep_mine() {
    let mut c: Config = serde_json::from_str(
        r#"{"pads_v13":true,"pads_v20":true,"my_pads":[
            {"name":"Pad + Reverse em G - Em","path":"embutido:0","key":7},
            {"name":"Meu C#","path":"embutido:1","key":1},
            {"name":"Coral Celeste","path":"livre:0","key":0}]}"#,
    )
    .unwrap();
    c.normalize();
    let names: Vec<&str> = c.my_pads.iter().map(|m| m.name.as_str()).collect();
    assert_eq!(names, vec!["Júpiter", "Meu C#", "Vênus"], "o que o Theo renomeou fica");
    c.my_pads[0].name = "Sol de domingo".into();
    c.normalize();
    assert_eq!(c.my_pads[0].name, "Sol de domingo", "não troca de novo");
    // config nova já vem com planetas
    let mut d = Config::default();
    d.normalize();
    let names: Vec<&str> = d.my_pads.iter().map(|m| m.name.as_str()).collect();
    assert_eq!(names, vec!["Júpiter", "Saturno", "Netuno", "Vênus", "Terra", "Marte"]);
}

// ---------- v0.23: filtro de abrir/fechar e swell ----------

/// Congelado agudo (3 kHz) tocando, sem SF2: o "fundo" para o filtro e o swell.
fn rig_with_bright_freeze() -> Rig {
    let mut r = rig();
    let rate = 48_000u32;
    let s: Vec<f32> = (0..rate as usize * 2)
        .map(|i| (std::f32::consts::TAU * 3000.0 * i as f32 / rate as f32).sin() * 0.3)
        .collect();
    let src = crate::freeze::FreezeSource::from_audio(&s, &s, rate).unwrap();
    r.core.freeze_play(src);
    r.render(48_000); // entrou
    r
}

#[test]
fn filter_closes_and_opens_the_background() {
    let mut r = rig_with_bright_freeze();
    let open = rms(&r.render(24_000));
    assert!(open > 0.02, "aberto soa: {open}");
    r.core.set_filter(0.0);
    r.render(24_000);
    let closed = rms(&r.render(24_000));
    assert!(closed < open * 0.1, "fechado fica escuro: {closed} vs {open}");
    // knob da mesinha (CC 16 da SMC, relativo) ligado ao filtro pelo Aprender
    r.core.learn.active = true;
    r.core.learn.target = Some(Target::Filter);
    r.core.handle_controller(&[0xB0, 50, 10]);
    r.core.learn.active = false;
    r.core.handle_controller(&[0xB0, 50, 127]);
    assert!((r.core.cfg.efeitos.filtro - 1.0).abs() < 1e-3, "knob abriu tudo");
    r.render(24_000);
    let again = rms(&r.render(24_000));
    assert!(again > open * 0.8, "abriu de novo: {again}");
}

#[test]
fn swell_takes_the_background_out_and_back_and_panic_restores() {
    let mut r = rig_with_bright_freeze();
    r.core.cfg.efeitos.swell_secs = 2.0;
    let full = rms(&r.render(12_000));
    r.core.toggle_swell(); // sai em 2 s
    assert!(!r.core.swell_in);
    let fade = r.render(96_000); // os 2 s da saída
    let (start, end) = (rms(&fade[..19_200]), rms(&fade[172_800..]));
    assert!(end < start * 0.2, "vai baixando: {start} → {end}");
    let gone = rms(&r.render(48_000));
    assert!(gone < full * 0.02, "saiu de vez: {full} → {gone}");
    r.core.toggle_swell(); // entra
    r.render(96_000);
    let back = rms(&r.render(24_000));
    assert!(back > full * 0.8, "entrou de novo: {back}");
    // saiu e alguém apertou pânico: o swell volta a cheio
    r.core.toggle_swell();
    r.render(9_600);
    r.core.panic();
    assert!(r.core.swell_in);
    r.core.handle_piano(&[0x90, 60, 100]);
    assert!(rms(&r.render(4_800)) > 0.01, "o piano não fica mudo depois do pânico");
}

#[test]
fn filter_settings_default_open_and_old_config_loads() {
    let c = Config::default();
    assert_eq!(c.efeitos.filtro, 1.0, "abre o programa com o filtro aberto");
    let old: Config = serde_json::from_str(r#"{"efeitos":{"bpm":90}}"#).unwrap();
    assert_eq!(old.efeitos.filtro, 1.0);
    assert!(old.efeitos.swell_filtro && !old.efeitos.filtro_tudo);
}
