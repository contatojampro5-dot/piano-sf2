# PIANO SF2 — guia para o Claude (leia antes de mexer)

Programa leve para Windows que toca SF2 ao vivo na igreja. Dono: Theo (GitHub `contatojampro5-dot`), chama o
assistente de **"Zeus"**. Responda sempre em português, simples e direto. Theo é iniciante em GitHub: explicar passo a passo.

## Equipamento do Theo (o programa é feito para isso)
- Notebook Windows 10, i5-5200U (2 núcleos), 8 GB DDR3, SSD 112 GB (pouco espaço livre), Intel HD 5500.
- Piano **Casio Privia PX-S3000** (USB-MIDI). Som do notebook volta para o AUDIO IN do piano → LINE OUT → caixas.
- Mesinha **M-Vave SMC-Mixer** (8 faders, 8 knobs, botões; mapeamento padrão = Mackie Control).
- SF2 principal: *Soundpack Eliab Campos Teclas.sf2* (1,39 GB), na pasta Downloads.
- Motivo do projeto: o Numa Player trava muito nesse notebook. **Leveza e estabilidade ao vivo vêm antes de tudo.**

## Como o projeto é montado
- Rust. `src/model.rs` (dados/config/mapeamento SMC), `src/engine.rs` (motor na thread de áudio, nada de travar ou
  alocar lá), `src/core.rs` (estado + ações, usado pela tela e pelo MIDI), `src/audio.rs` (cpal/WASAPI),
  `src/midi.rs` (midir, reconexão a cada 1,5 s, luzes), `src/app.rs` (tela egui), `src/main.rs`.
- `vendor/rustysynth` = cópia do rustysynth 1.3.8 com ajustes marcados `piano-sf2:` (não trocar pela versão do crates.io).
- Uma camada = um canal MIDI (0..7) de um único `Synthesizer`. Liga/desliga usa expressão (CC11) + corte após 60 ms.
- Testes: `cargo test --no-default-features` (núcleo, sem tela). Carga: `cargo test --release --no-default-features bench -- --ignored --nocapture`.
- Checar Windows daqui: `cargo check --target x86_64-pc-windows-gnu`. O `.exe` é montado pelo GitHub Actions
  (`.github/workflows/windows.yml`) e publicado em Releases (`releases/latest/download/PianoSF2.exe`).
- Print da tela no Linux: Xvfb + `LIBGL_ALWAYS_SOFTWARE=1` + `HOME=<pasta com .config/piano-sf2/config.json>`.

## Regras
- Toda atualização vai para o GitHub (commit + push em `main`) e ganha uma linha em "Versões" no `README.md`.
- Mensagem de commit curta em português começando pela versão (ex.: `v0.2: ...`).
- Pendências/ideias: ASIO (menos atraso), equalizador por camada, dividir teclado (split), gravar WAV.
