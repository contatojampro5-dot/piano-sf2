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
  `src/midi.rs` (midir/WinMM p/ USB + `src/ble.rs` WinRT MIDI/GATT; `src/btdirect.rs` = Bluetooth direto via GATT
  com janela "Procurar", porque o MIDI-Bluetooth do Windows 10 não achou a SMC; `src/blemidi.rs` = pacotes BLE-MIDI), `src/app.rs` (tela egui), `src/main.rs`.
- `vendor/rustysynth` = cópia do rustysynth 1.3.8 com ajustes marcados `piano-sf2:` (não trocar pela versão do crates.io).
- `LAYERS = 6` (v0.3; canais 7/8 da mesinha = PAD/GERAL). Uma camada = um canal MIDI de um único `Synthesizer`. Liga/desliga usa expressão (CC11) + corte após 60 ms.
- `src/pads.rs` (v0.2/v0.3): pads por tom (12 tons × estilos Escuro/Coral/Suave) embutidos em OGG (`assets/pads/`), decodificados
  por uma thread (`start_pad_loader`) e tocados no motor com crossfade (`Cmd::PadPlay`). Gerador: `tools/gerar_pads.py`.
- Pad Vivo (v0.9): `src/granular.rs` (GranularVoice + ChurchReverb FDN), bancos 3..5 (`pads::is_vivo`), texturas em
  `assets/texturas/` (C3, geradas por `tools/gerar_texturas.py`), ajustes em `cfg.vivo`. Prévia WAV: teste `preview` (ignored).
- Ajuda de escala (v0.10): `src/teoria.rs` (escala/pentatônica/acordes com grafia certa), `scale_panel` no app, `cfg.scale_minor`.
- Sons grátis (v0.12): `src/sons.rs` (catálogo + download via `curl.exe` para %APPDATA%\PianoSF2\sons), janela `sons_window`,
  release `sons-v1` montado por `.github/workflows/sons.yml` (lista `tools/sons.txt`, juntador `tools/montar_pacote.py`).
  Só sons com licença livre (FreePats CC0/CC-BY). Nunca redistribuir sons do Numa/Casio/Eliab.
- Meus pads (v0.13): bancos `MY_BANK..ALL_BANKS` (até 4, `cfg.my_pads` com nome/tom), `pads::load_my_pad` (90 s do arquivo,
  volume igualado, `detect_key` acha o tom pelo som), altura por velocidade (`Cmd::PadPlay(clip, pitch)`).
  Pads da parceria embutidos em `assets/pads/parceria/` (`PARTNER_PADS`, caminho "embutido:N") — **repositório privado,
  não publicar esses sons**. Estilos do programa escondidos por `hidden` "pad_styles".
- Pads no navegador (v0.14): `web/index.html` (Web Audio, loop com emendas cruzadas, altura por `playbackRate`) + `web/pads/*.mp3`
  (mesmos pads, volume igualado com loudnorm). Publicada como Artifact privado https://claude.ai/artifact/Csec6zTktwjRpXAC6CmXFj
  (republicar com o mesmo arquivo). Pad novo = mp3 em `web/pads/` + entrada em `PADS`.
- Pitch/Mod do teclado (v0.15): alvos `Target::Pitch/PitchUp/PitchDown/Modulation` (`is_momentary` = botão de segurar),
  `cfg.piano_mappings` (controles da porta Piano, separados da mesinha), `cfg.pitch_range` (RPN 0, `Cmd::PitchRange`).
  A thread do MIDI decide sem travar o `core` com `core::piano_routed` (marcas atualizadas por `sync_piano_hooks`, a cada quadro).
  Tela: `pitch_mod_controls` na coluna GERAL. Teclado do Theo: AMW MIDI Control Mini 32 (aparece como "AKM320"; pelo USB só
  manda notas — rodas e slider não mandaram nada no teste de 12/11/2026).
- Congelar (v0.16): `src/freeze.rs` — `CaptureRing` (memória sem trava, ~5 s do som das camadas, escrita no `render`),
  `pick_region` (fim = som mais recente não silencioso, começo depois do ataque), `FreezeVoice` (granular sem mudar altura,
  volume = RMS dos últimos 0,3 s). `Cmd::FreezePlay/FreezeStop`, `Target::Freeze`, `core.freeze_toggle()`. Botão na coluna GERAL.
  Prévia com SF2 de verdade: teste `freeze_real_sf2_preview` (ignored; `PIANO_SF2_FREEZE`, `PIANO_FREEZE_WAV`).
- Efeitos (v0.17): `src/efeitos.rs` (Tempo/`step_samples`, `pulse_gain`, `Arp`, `DrumKit::build` sons sintetizados, `Drums`,
  `VoiceFx` coral granular por nota com a textura Coro). No motor: `clock`, arpejo com bloco cortado no passo exato
  (`arp_next`), pulso no barramento dos pads e no congelado, coral antes do `ring().push` (o Congelar pega), batidas por último.
  `cfg.efeitos` (ajustes salvos; ligado/desligado não salva), aba 3 `efeitos_page`. Nos testes `freeze::ring()` é por thread.
  Prévia: teste `efeitos_real_sf2_preview` (ignored; `PIANO_SF2_FREEZE`, `PIANO_EFEITOS_WAV`).
- Tela limpa (v0.18): `UiState.mix_details/pad_details/fx_details` (⚙) escondem os ajustes finos; o Aprender mostra tudo.
  Ações raras no menu ⚙ da barra de cima. Fonte não tem ▲▼ nem 🥁: desenhar com o painter ou trocar o ícone.
- Sons próprios (v0.19): `cfg.efeitos.drum_files` + `efeitos::load_drum_file` / `DrumKit::build_with` (`core.set_drum_file`),
  `cfg.efeitos.voicefx_file` (arquivo, nota MIDI; `core.set_voicefx_file` com `pads::detect_key`). Ações da tela que mexem
  em arquivos rodam depois de `set_efeitos(e)` (senão a cópia `e` desfaz).
- Mapeamento padrão em `smc_default_mappings()`; mudou o padrão → subir `MAPPINGS_VERSION` (migra configs
  antigas sem apagar o que o Theo aprendeu). Assistente "Configurar mesinha" = `Wizard` em `core.rs`.
- Visual (v0.5): `src/theme.rs` (temas, vidro `glass()`, fundo `paint_background`), cores via `th()`; ajustes em `cfg.look`.
- Modo Editar (v0.8): `cfg.hidden` com ids de `EDITABLE` (model.rs); cenas dinâmicas 1..`MAX_SCENES` (`add_scene`/`delete_scene`).
- **Nunca** chamar algo que trava o `core` (ex.: `load_sf2_async`, janela de arquivo) com o `core` já travado pela tela:
  use os pedidos adiados do `UiState` (`open_sf2_dialog`, `pad_file_dialog`). Foi isso que travou a v0.1.
- Testes: `cargo test --no-default-features` (núcleo, sem tela). Carga: `cargo test --release --no-default-features bench -- --ignored --nocapture`.
- Checar Windows daqui: `cargo check --target x86_64-pc-windows-gnu`. O `.exe` é montado pelo GitHub Actions
  (`.github/workflows/windows.yml`) e publicado em Releases (`releases/latest/download/PianoSF2.exe`).
- Print da tela no Linux: Xvfb + `LIBGL_ALWAYS_SOFTWARE=1` + `HOME=<pasta com .config/piano-sf2/config.json>`.

## App de estudos (outro projeto)
- O app Firmes (antes na pasta `apologia/`) mora no repositório `contatojampro5-dot/escola-apologia-`
  (https://contatojampro5-dot.github.io/escola-apologia-/). Não existe mais neste repositório.

## Regras
- Toda atualização vai para o GitHub (commit + push em `main`) e ganha uma linha em "Versões" no `README.md`.
- Mensagem de commit curta em português começando pela versão (ex.: `v0.2: ...`).
- Pendências/ideias: guardar o pad nas cenas, ASIO (menos atraso), equalizador por camada, dividir teclado (split), gravar WAV.
