# 🎹 Piano SF2

Programa **leve** para Windows que toca sons **SF2** ao vivo, feito para a igreja:
piano **Casio Privia PX-S3000** + mesinha **M-Vave SMC-Mixer** + notebook simples (i5 de 5ª geração, 8 GB).

- 6 camadas de som ao mesmo tempo, cada uma com **liga/desliga, solo, volume e reverb**.
- A tela copia a SMC-Mixer: **faders 1–6 = camadas**, **fader 7 = PAD**, **fader 8 = GERAL**.
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

## Mesinha pelo Bluetooth

O programa conecta **direto** no Bluetooth da SMC-Mixer (sem driver, sem depender do MIDI do Windows):
1. Ligue o **Bluetooth da mesinha** (botão BT, luz piscando). Se ela estiver conectada num celular/tablet, desconecte de lá.
2. No Piano SF2, clique em **Bluetooth** (lá em cima) — ou clique em **Mesinha** quando estiver vermelha.
3. Espere ela aparecer em *Aparelhos por perto* e clique em **Conectar**.
4. Pronto: a bolinha de **Mesinha** fica verde. Das próximas vezes ela **reconecta sozinha** ao abrir o programa.

Para parar de usar: janela Bluetooth → **Esquecer**. O piano continua melhor no cabo USB (menos atraso).

## Primeiro uso

1. Clique em **📂 Abrir SF2...** e escolha o seu arquivo (ex.: *Soundpack Eliab Campos Teclas.sf2*).
   Um arquivo de 1,4 GB leva uns 10–20 s para carregar. Na próxima vez ele abre sozinho.
2. Em cada camada, clique no nome do timbre (ou use ◀ ▶) para escolher o som.
3. **LIGADA/DESLIGADA** liga a camada; **SOLO** deixa só ela tocando.
4. Monte o som, clique numa cena (ex.: *F2*) e em **💾 Salvar cena**.
   Botão direito numa cena: **salvar aqui / renomear / apagar**.

## Aparência (⚙ Ajustes → Aparência)

Visual de vidro (estilo iPhone) sobre um fundo celestial que respira devagar.
- **Fundo**: Celestial, Aurora, Pôr do sol ou Grafite.
- **Vidro**: do bem transparente ao bem fosco.
- **Brilho** em volta das camadas ligadas e do pad tocando.
- **Tamanho da tela** (80% a 130%), **Fundo em movimento** e **Estrelas**.
- No culto, se o computador ficar pesado, desligue **Fundo em movimento** (a tela fica parada e quase não gasta nada).

## Teclado de teste (sem o piano ligado)

Na aba **Sons SF2**, o botão **🎹 Teclado** (ao lado de *Salvar cena*) mostra/esconde um piano na tela.
- Clique nas teclas para tocar (mais embaixo na tecla = mais forte); arrastar faz glissando.
- Teclado do computador: **A W S E D F T G Y H U J K** = notas, **Z / X** = oitava, **espaço** = pedal.
- Botão **PEDAL** segura as notas.
- Com o piano de verdade ligado, as teclas que você toca nele **acendem** na tela.

## Pads (aba 🎶 Pads)

Lá em cima tem duas abas: **🎹 Sons SF2** (a mesa com as 8 camadas) e **🎶 Pads** (o fundo contínuo por tom).

- **12 botões quadrados**, um por tom (C, C#, D... B). Clique para tocar; clique de novo para parar.
- Trocar de tom faz a **passagem suave** (um some enquanto o outro entra).
- **Sem terça**: o pad de C serve para **C e Cm** (vale para todos os tons).
- Dois estilos prontos, já dentro do programa:
  - **Escuro**: grave, fechado, místico.
  - **Coral**: vozes graves e médias ("aah/ooh").
  - **Suave**: tranquilo e redondo, para a igreja.
- **Seu próprio som**: botão direito num tom → *Usar meu arquivo (WAV/MP3/OGG)*. Para voltar: *Voltar ao som padrão*.
- Volume do pad separado, botões ◀ ▶ (tom anterior/próximo) e **PARAR PAD**.
- Tudo pode ir para a mesinha com **Aprender**: cada tom, volume do pad, parar, anterior/próximo e trocar Escuro/Coral.
  (Ex.: clique em Aprender → clique no botão "C" da tela → aperte um botão da mesinha.)

## Mesinha SMC-Mixer (padrão Mackie)

| Na mesinha | No programa |
|---|---|
| Fader 1–6 | Volume da camada 1–6 |
| Fader 7 | Volume do PAD |
| Fader 8 (e o master, se a sua tiver) | Volume geral |
| Knob 1–6 (girar) | Reverb da camada 1–6 |
| Knob 7 (girar) | Escolhe o tom do pad (C, C#, D...) |
| Botão **M** (mute) 1–6 | Liga/desliga a camada |
| Botão **S** (solo) 1–6 | Solo da camada |
| **M** 7 / **S** 7 | Parar pad / trocar estilo do pad |
| **M** 8 | Trocar de aba (Sons SF2 ↔ Pads) |
| Botões **R** (rec) e **Select** | Livres para você usar com **Aprender** |
| ◀◀ / ▶▶ (e setas ◀ ▶) | Cena anterior / próxima |
| ■ Stop | Pânico |
| Setas ▲ ▼ | Transpor +1 / −1 |

Os botões da mesinha acendem quando a camada está ligada.

**Configurar mesinha (automático):** clique em **Configurar mesinha** lá em cima e mexa os controles na ordem
que a faixa roxa pedir (fader 1, fader 2... fader 8, depois knob 1... knob 7). Não precisa clicar em nada:
o programa aprende sozinho, inclusive se a mesinha estiver num modo diferente. **Pular** deixa um controle como está.
Depois, use **Aprender** só para o que quiser mudar. No modo Aprender aparece o *último sinal da mesinha*,
para você ver o que cada botão manda.
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

Os pads padrão ficam em `assets/pads/<escuro|coral>/NN.ogg` (00 = C ... 11 = B) e são gerados por
`tools/gerar_pads.py` (Python + numpy/scipy; `python3 tools/gerar_pads.py escuro 0 saida.wav`, depois `ffmpeg -c:a libvorbis -q:a 4`).

O GitHub Actions (`.github/workflows/windows.yml`) monta o `PianoSF2.exe` a cada push em `main` e publica em Releases.

## Versões

### v0.7 — Bluetooth direto (janela "Procurar mesinha")
- Botão **Bluetooth**: procura aparelhos por perto e conecta a mesinha direto (BLE-MIDI pelo GATT), sem driver.
- Reconecta sozinha ao abrir o programa e se o Bluetooth cair; luzes dos botões também pelo Bluetooth.

### v0.6.1 — Bluetooth: busca mais ampla e diagnóstico
- Procura a mesinha em todos os aparelhos MIDI que o Windows enxerga (não só pelo "jeito" do id Bluetooth),
  sem abrir duas vezes o que já veio pelo cabo.
- ⚙ Ajustes → Aparelhos MIDI → **Diagnóstico**: mostra tudo que o Windows enxerga e o erro de cada aparelho.

### v0.6 — mesinha pelo Bluetooth
- A SMC-Mixer (e outros aparelhos MIDI Bluetooth pareados no Windows) aparece como "… (Bluetooth)" e funciona sem fio.
- Luzes dos botões também pelo Bluetooth; reconexão automática.

### v0.5 — visual de vidro (estilo iPhone)
- Fundo celestial com nuvens de luz e estrelas (4 temas), painéis de vidro fosco com transparência ajustável.
- Faders e knobs novos (pílula branca com sombra), movimento suave, aura colorida no que está ligado.
- Cores do sistema do iPhone; quadrados de pad de vidro; seção **Aparência** nos Ajustes.

### v0.4 — teclado de teste na tela
- Botão **🎹 Teclado**: piano na tela (mouse e teclado do computador), com pedal; abre e fecha com um clique.
- As notas tocadas no piano de verdade acendem nesse teclado.
- Canais ficam compactos quando a tela fica baixa (teclado aberto num notebook 1366×768).

### v0.3.1 — aviso no Aprender
- Quando um mapeamento novo tira outro do lugar, a tela avisa (ex.: "esse controle deixou de fazer Liga/desliga camada 2").
- Confirmado: vários botões podem fazer a mesma coisa (na v0.2 o segundo botão "desmapeava" o primeiro).

### v0.3 — pad Suave, 6 canais e configurar mesinha automático
- Pad novo **Suave** (12 tons), tranquilo e redondo.
- **6 camadas** (antes 8): o canal 7 virou **PAD** (volume, tom, parar, estilo) e o 8 virou **GERAL**, igual à mesinha.
- **Configurar mesinha (automático)**: mexa faders e knobs na ordem pedida e pronto.
- Mais coisas para mapear: knob escolhe o tom do pad, trocar de aba, e vários botões podem fazer a mesma coisa.
  REC e Select ficam livres no padrão. O modo Aprender mostra o último sinal da mesinha.
- Quem já tinha mapeado: o que você aprendeu continua; só o padrão antigo foi atualizado.

### v0.2 — pads e correção do travamento
- **Corrigido**: escolher o SF2 pelo botão 📂 travava o programa para sempre ("Não está respondendo").
  Agora carrega em segundo plano (um SF2 de 1,4 GB levou ~6 s no teste).
- Abas no topo: **Sons SF2** e **Pads**.
- Pads em 12 tons, estilos **Escuro** e **Coral** (gerados para o projeto, sem terça), troca suave entre tons.
- Cada tom aceita um arquivo próprio (WAV/MP3/OGG/FLAC).
- Pads mapeáveis na mesinha (tons, volume, parar, anterior/próximo, estilo); a luz do botão acende no tom tocando.

### v0.1 — primeira versão
- 8 camadas no estilo SMC-Mixer (liga/desliga, solo, volume, reverb, escolha de timbre com busca).
- Canal geral: volume, transpor (−12 a +12) e pânico.
- 8 cenas com nome, salvas automaticamente.
- SMC-Mixer pronta (padrão Mackie) + modo Aprender para qualquer controle MIDI.
- Reconexão automática de piano/mesinha; luzes dos botões da mesinha.
- Áudio com prioridade alta, limitador suave na saída, medidor de CPU e de volume.
