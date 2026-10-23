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
- `LAYERS = 6` (v0.3; canais 7/8 da mesinha = PAD/GERAL). Uma camada = um canal MIDI de um único `Synthesizer`. Liga/desliga usa expressão (CC11) + corte após 60 ms.
- `src/pads.rs` (v0.2/v0.3): pads por tom (12 tons × estilos Escuro/Coral/Suave) embutidos em OGG (`assets/pads/`), decodificados
  por uma thread (`start_pad_loader`) e tocados no motor com crossfade (`Cmd::PadPlay`). Gerador: `tools/gerar_pads.py`.
- Mapeamento padrão em `smc_default_mappings()`; mudou o padrão → subir `MAPPINGS_VERSION` (migra configs
  antigas sem apagar o que o Theo aprendeu). Assistente "Configurar mesinha" = `Wizard` em `core.rs`.
- Visual (v0.5): `src/theme.rs` (temas, vidro `glass()`, fundo `paint_background`), cores via `th()`; ajustes em `cfg.look`.
- **Nunca** chamar algo que trava o `core` (ex.: `load_sf2_async`, janela de arquivo) com o `core` já travado pela tela:
  use os pedidos adiados do `UiState` (`open_sf2_dialog`, `pad_file_dialog`). Foi isso que travou a v0.1.
- Testes: `cargo test --no-default-features` (núcleo, sem tela). Carga: `cargo test --release --no-default-features bench -- --ignored --nocapture`.
- Checar Windows daqui: `cargo check --target x86_64-pc-windows-gnu`. O `.exe` é montado pelo GitHub Actions
  (`.github/workflows/windows.yml`) e publicado em Releases (`releases/latest/download/PianoSF2.exe`).
- Print da tela no Linux: Xvfb + `LIBGL_ALWAYS_SOFTWARE=1` + `HOME=<pasta com .config/piano-sf2/config.json>`.

## Regras
- Toda atualização vai para o GitHub (commit + push em `main`) e ganha uma linha em "Versões" no `README.md`.
- Mensagem de commit curta em português começando pela versão (ex.: `v0.2: ...`).
- Pendências/ideias: guardar o pad nas cenas, ASIO (menos atraso), equalizador por camada, dividir teclado (split), gravar WAV.
