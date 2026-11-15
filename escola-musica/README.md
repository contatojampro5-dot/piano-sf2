# Escola Harpa

App de estudos de música no navegador (pode ser instalado na tela inicial do celular). Feito no mesmo molde da Escola Apologia, mas com **som de verdade** gerado no próprio aparelho.

## O que tem
- **No padrão do app Firmes**: boas-vindas com nome, tela inicial com teste, stories dos mestres, “Você está aqui”, missão do dia, álbum e frase do dia; trilha em caminho pontilhado; stories em tela cheia com capa; mestres desenhados que evoluem em 4 fases (motor de arte `personS` do Firmes).
- **Teste**: “Qual caminho na música combina com você?” (Bacharelado, Licenciatura, Composição, Regência, Produção).
- **Álbum**: 91 figurinhas (3 por lição + 1 lendária por mestre).
- **Trilha de 28 lições em 7 níveis** (as áreas de uma faculdade de música): Teoria (Quenanias), Percepção (Davi), Ritmo e prática (Miriã), Harmonia (Asafe), Composição e arranjo (Salomão), História (Jubal), Acústica e produção (Bezalel).
- **Lições em stories** com som: teclado tocável, pauta, grade de ritmo, "toque no ritmo", ouça e responda, verdadeiro/falso, ordem, pares, conversas do dia a dia na banda. Prova no fim e missão prática.
- **Academia de ouvido** (exercícios gerados na hora, com níveis): intervalos, acordes, escalas e modos, graus da tonalidade, progressões, ritmo, afinação fina, timbre e "ouvido de engenheiro" (EQ).
- **Laboratório**: teclado com escalas/acordes/intervalos (grafia correta), campo harmônico, círculo das quintas, metrônomo, ondas e harmônicos, mesa de mixagem (EQ, compressor, reverb, espectro).
- **Cards** com revisão espaçada (~100 termos) e **Guia da faculdade de música**.

Progresso salvo no aparelho (localStorage). `sw.js` deixa abrir sem internet depois da primeira visita.

## Arquivos
`index.html` · `css/app.css` · `js/teoria.js` (teoria com grafia certa) · `js/audio.js` (motor de som Web Audio) · `js/art.js` (desenho dos mestres) · `js/base.js` (mestres, níveis, cards, guia) · `js/extras.js` (figurinhas, frases, teste) · `js/ui2.js` (telas no padrão Firmes) · `js/licoes1-3.js` (conteúdo) · `js/app.js` (telas).

## Versões
- **v2**: visual e fluxo no padrão do Firmes (boas-vindas, início, trilha em caminho, stories em tela cheia, mestres desenhados, álbum, teste vocacional).
- **v1**: primeira versão (28 lições, academia de ouvido, laboratório).
