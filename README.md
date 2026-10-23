# 🎹 Piano SF2

Programa **leve** para Windows que toca sons **SF2** ao vivo, feito para a igreja:
piano **Casio Privia PX-S3000** + mesinha **M-Vave SMC-Mixer** + notebook simples (i5 de 5ª geração, 8 GB).

- 8 camadas de som ao mesmo tempo, cada uma com **liga/desliga, solo, volume e reverb**.
- A tela copia a SMC-Mixer: **fader 1 = camada 1**, **fader 2 = camada 2**...
- **8 cenas** para trocar o som inteiro com um botão (F1–F8, setas ou o transporte da mesinha).
- **Botão de pânico** para cortar nota presa (tecla **Esc**).
- Um SF2 só, carregado **uma vez** na memória e usado pelas 8 camadas (o pacote de 1,4 GB cabe nos 8 GB do notebook).
- Reconecta sozinho se o cabo do piano ou da mesinha soltar.

## Baixar

1. Abra **https://github.com/contatojampro5-dot/piano-sf2/releases/latest**
2. Clique em **PianoSF2.exe** para baixar.
3. Guarde numa pasta (ex.: *Documentos*) e abra com dois cliques.
   Se aparecer "O Windows protegeu o computador": **Mais informações → Executar assim mesmo**.

Não precisa instalar nada. As configurações ficam em `%APPDATA%\PianoSF2\config.json`.

## Ligação

```
Piano Casio ──USB──► Notebook        (o piano manda as notas)
SMC-Mixer   ──USB──► Notebook        (use o cabo, não o Bluetooth)
Notebook (saída de fone) ──cabo P2-P2──► AUDIO IN do piano
Piano (LINE OUT) ──► caixas / mesa da igreja
```

- **Desligue o som próprio do piano**, senão sai o som do Casio junto: no menu de configurações do PX-S3000,
  procure **Local Control** e deixe em **Off**.
- **Feche o Numa** (e qualquer outro programa de música): no Windows, só um programa por vez consegue usar o piano e a mesinha.
- Zumbido com o carregador ligado? Um **isolador de áudio P2** resolve.

## Primeiro uso

1. Clique em **📂 Abrir SF2...** e escolha o seu arquivo (ex.: *Soundpack Eliab Campos Teclas.sf2*).
   Um arquivo de 1,4 GB leva uns 10–20 s para carregar. Na próxima vez ele abre sozinho.
2. Em cada camada, clique no nome do timbre (ou use ◀ ▶) para escolher o som.
3. **LIGADA/DESLIGADA** liga a camada; **SOLO** deixa só ela tocando.
4. Monte o som, clique numa cena (ex.: *F2*) e em **💾 Salvar cena**.
   Botão direito numa cena: **salvar aqui / renomear / apagar**.

## Mesinha SMC-Mixer (padrão Mackie)

| Na mesinha | No programa |
|---|---|
| Fader 1–8 | Volume da camada 1–8 |
| Fader master (o 9º, se a sua tiver) | Volume geral |
| Knob 1–8 (girar) | Reverb da camada 1–8 |
| Botão **M** (mute), **R** (rec) ou **Select** | Liga/desliga a camada |
| Botão **S** (solo) | Solo da camada |
| ◀◀ / ▶▶ (e setas ◀ ▶) | Cena anterior / próxima |
| ■ Stop | Pânico |
| Setas ▲ ▼ | Transpor +1 / −1 |

Os botões da mesinha acendem quando a camada está ligada.
Se algum controle não responder (a mesinha pode estar em outro modo), use **Aprender**:
1. clique em **Aprender** (lá em cima);
2. clique no controle da tela (ex.: o fader da camada 1);
3. mexa o controle da mesinha. Pronto, ficou ligado. **Esc** sai do modo aprender.

Em **⚙ Ajustes → Controles da mesinha** dá para ver/remover as ligações ou voltar ao padrão.
Em **⚙ Ajustes** também dá para ligar "troca de programa do piano muda a cena" (programa 1 = Cena 1), para trocar
de cena pelos botões do próprio piano. Vem desligado.

## Atalhos do teclado do computador

| Tecla | Faz |
|---|---|
| ← → | Cena anterior / próxima |
| F1–F8 | Cena 1–8 |
| Esc | Pânico (corta todas as notas) |

## Dicas para o culto

- Deixe o **carregador ligado** e o Windows em **Melhor desempenho** (ícone da bateria).
- Feche o Chrome e outros programas. Pause o **Windows Update** no dia.
- Abra o programa **antes do culto** (o SF2 grande leva alguns segundos).
- Lá em cima aparece **CPU %**. Verde está ótimo. Se ficar vermelho ou aparecer "engasgos" no rodapé,
  vá em **⚙ Ajustes** e aumente o **Buffer** (ex.: 256 → 512) ou baixe as **Vozes** (128 → 96).
- Menos buffer = menos atraso. 256 (≈5 ms) é um bom começo.

## Para desenvolvedores

Rust + [egui](https://github.com/emilk/egui) (tela), [cpal](https://github.com/RustAudio/cpal) (áudio WASAPI),
[midir](https://github.com/Boddlnagg/midir) (MIDI) e [rustysynth](https://github.com/sinshu/rustysynth) (SF2, cópia
em `vendor/` com ajustes: aceita chunks desconhecidos, regiões quebradas, nomes com acento e banco/programa direto).

```
cargo test --no-default-features   # testes do motor (sem tela)
cargo run --release                # programa completo
```

O GitHub Actions (`.github/workflows/windows.yml`) monta o `PianoSF2.exe` a cada push em `main` e publica em Releases.

## Versões

### v0.1 — primeira versão
- 8 camadas no estilo SMC-Mixer (liga/desliga, solo, volume, reverb, escolha de timbre com busca).
- Canal geral: volume, transpor (−12 a +12) e pânico.
- 8 cenas com nome, salvas automaticamente.
- SMC-Mixer pronta (padrão Mackie) + modo Aprender para qualquer controle MIDI.
- Reconexão automática de piano/mesinha; luzes dos botões da mesinha.
- Áudio com prioridade alta, limitador suave na saída, medidor de CPU e de volume.
