// Lições — Nível 1 (Teoria, Quenanias) e Nível 2 (Percepção, Davi).
"use strict";
const LESSONS = {};
LESSONS.t1 = {
  title: "O som e as 12 notas", sub: "Altura, duração, intensidade, timbre, semitons e o teclado",
  goals: ["Saber as 4 propriedades do som", "Achar qualquer nota no teclado", "Entender semitom, tom, sustenido, bemol e oitava"],
  mis: "No piano (ou no teclado do Laboratório), ache **todos os Dós**. Depois toque de Dó a Dó só nas brancas e aponte onde estão os dois semitons naturais (Mi–Fá e Si–Dó).",
  deep: ["O ouvido humano jovem percebe de cerca de **20 Hz a 20 kHz**. Com a idade, os agudos vão sumindo.", "No temperamento igual, cada semitom multiplica a frequência por 2^(1/12) ≈ **1,0595**. Doze semitons = ×2 = uma oitava.", "O Lá de 440 Hz como referência de afinação foi padronizado internacionalmente no século XX (norma ISO 16)."],
  s: [
    { k: "mentor", t: "A paz! Eu sou **Quenanias**, chefe dos levitas no canto. A Bíblia diz que eu ensinava o canto “porque era entendido” (1 Cr 15.22). Entender vem antes de impressionar. Bora começar pelo próprio som?" },
    { k: "txt", h: "Som é vibração", p: "Uma corda, uma pele ou suas pregas vocais vibram e empurram o ar. Essa onda de pressão viaja a cerca de **343 metros por segundo** até o seu tímpano, que vibra junto. O cérebro transforma isso em música." },
    { k: "list", h: "Todo som tem 4 propriedades", items: ["**Altura**: grave ou agudo (a frequência)", "**Duração**: curto ou longo", "**Intensidade**: fraco ou forte (a amplitude)", "**Timbre**: a “cor” que diferencia um violão de uma flauta"] },
    { k: "play", tag: "Altura e intensidade", h: "Ouça cada propriedade", p: "Mesma nota em alturas diferentes, depois a mesma nota em volumes diferentes, depois a mesma nota em timbres diferentes.", b: [["Grave → agudo", "C2:1.5 C4:1.5 C6:1.5"], ["Fraco → forte", "fx:dyn"], ["4 timbres", "fx:waves"]] },
    { k: "txt", h: "7 nomes, 12 sons", p: "As notas naturais são **Dó, Ré, Mi, Fá, Sol, Lá, Si**. Na cifra usamos letras: **C D E F G A B**. Repare: a cifra começa no Lá (A), não no Dó!\n\nEntre elas há mais 5 sons (as teclas pretas). Total: **12 sons diferentes** antes de repetir." },
    { k: "match", h: "Ligue a nota à cifra", pairs: [["Lá", "A"], ["Ré", "D"], ["Sol", "G"], ["Si", "B"], ["Fá", "F"]], e: "Decore: A Lá, B Si, C Dó, D Ré, E Mi, F Fá, G Sol." },
    { k: "kb", h: "Toque um Dó", p: "Dica: o Dó fica logo à **esquerda do grupo de 2 teclas pretas**.", r: [48, 71], t: ["C"], e: "Qualquer Dó vale. O teclado é um padrão que se repete: 2 pretas, 3 pretas, 2 pretas…" },
    { k: "txt", h: "Semitom e tom", p: "**Semitom** é a menor distância do nosso sistema: a tecla vizinha, preta ou branca. **Tom** = 2 semitons.\n\nEntre **Mi–Fá** e **Si–Dó** não há tecla preta: são semitons naturais. Todo o resto entre brancas é um tom." },
    { k: "play", h: "Semitom × tom", b: [["Semitom (Mi–Fá)", "E4 F4:2"], ["Tom (Dó–Ré)", "C4 D4:2"], ["Os 12 semitons", "bpm=150 C4 C#4 D4 D#4 E4 F4 F#4 G4 G#4 A4 A#4 B4 C5:2"]], kb: [48, 72] },
    { k: "txt", h: "Sustenido ♯ e bemol ♭", p: "**♯ sobe** um semitom; **♭ desce** um semitom. A tecla preta entre Dó e Ré é **Dó♯** e também **Ré♭**: mesmo som, dois nomes. Isso se chama **enarmonia**. O nome certo depende da tonalidade (você vai ver isso na próxima lição)." },
    { k: "kb", h: "Toque um Fá♯", p: "Suba um semitom a partir do Fá.", r: [48, 71], t: ["F#"], e: "Fá♯ = Sol♭. É a primeira preta do grupo de três." },
    { k: "txt", h: "A oitava", p: "Do Lá ao próximo Lá há 12 semitons: uma **oitava**. A frequência dobra: Lá3 = 220 Hz, **Lá4 = 440 Hz** (o “lá do diapasão”, referência de afinação), Lá5 = 880 Hz. Por isso notas a uma oitava soam como “a mesma nota”." },
    { k: "play", h: "Três Lás", b: [["220 → 440 → 880 Hz", "A3 A4 A5:2"], ["Juntos", "[A3 A4 A5]:3"]], kb: [57, 81] },
    { k: "tf", q: "No piano, Mi♯ e Fá são a mesma tecla.", a: true, e: "São enarmônicos. Parece estranho, mas o Mi♯ aparece em tons como Fá♯ maior, onde todo Mi é sustenido." },
    { k: "mentor", t: "Grave isto: **4 propriedades, 12 sons, semitom e tom**. Todo o resto da teoria é montado em cima disso." }
  ],
  q: [
    { q: "Qual propriedade diferencia um violão de uma flauta tocando a mesma nota, no mesmo volume?", o: ["Timbre", "Altura", "Intensidade", "Duração"], c: 0, e: "O timbre é a “cor” do som, dada pelos harmônicos." },
    { q: "Entre Mi e Fá existe…", o: ["Um semitom", "Um tom", "Um tom e meio", "Uma tecla preta"], c: 0, e: "Mi–Fá e Si–Dó são os semitons naturais." },
    { q: "Na cifra, a letra **A** representa:", o: ["Lá", "Dó", "Ré", "Si"], c: 0, e: "A = Lá, B = Si, C = Dó…" },
    { q: "Se o Lá4 tem 440 Hz, o Lá5 tem:", o: ["880 Hz", "660 Hz", "450 Hz", "220 Hz"], c: 0, e: "Uma oitava acima, a frequência dobra." },
    { q: "Dó♯ e Ré♭ são:", o: ["Enarmônicos (mesmo som)", "Um tom de distância", "Notas de oitavas diferentes", "Sempre escritos do mesmo jeito"], c: 0, e: "Mesmo som, nomes diferentes." },
    { q: "Quantos sons diferentes há dentro de uma oitava no sistema temperado?", o: ["12", "7", "8", "24"], c: 0, e: "7 naturais + 5 alterados." },
    { q: "Ouça: a segunda nota está quantos semitons acima?", play: "C4 C#4:2", o: ["1 (semitom)", "2 (tom)", "3", "12"], c: 0, e: "Dó → Dó♯: um semitom." }
  ]
};
LESSONS.t2 = {
  title: "Escalas e tonalidade", sub: "A fórmula da escala maior, a menor, graus, armadura e círculo das quintas",
  goals: ["Montar qualquer escala maior pela fórmula", "Saber o nome e o papel de cada grau", "Entender armadura, relativa menor e círculo das quintas"],
  mis: "Escolha 3 tonalidades (por exemplo Sol, Ré e Fá maior) e toque cada escala subindo e descendo, falando o nome das notas em voz alta. Confira no Laboratório → Teclado.",
  deep: ["**Tonalidade** é o sistema de relações em torno de uma tônica; **escala** é a lista ordenada de notas desse sistema.", "Em tons com bemóis, o penúltimo bemol da armadura é o nome do tom (Si♭, Mi♭ → Si♭ maior). Em tons com sustenidos, o tom fica meio tom acima do último sustenido (Fá♯, Dó♯ → Ré maior)."],
  s: [
    { k: "txt", h: "Escala: o cardápio da música", p: "Uma **escala** é um conjunto de notas em ordem, a partir de uma nota principal, a **tônica**. Quase toda música usa um “cardápio” de 7 notas e volta sempre para a tônica." },
    { k: "big", n: "T T S T T T S", p: "A fórmula da **escala maior** (T = tom, S = semitom). Funciona a partir de qualquer nota." },
    { k: "play", h: "Dó maior", p: "Só teclas brancas, de Dó a Dó. Repare onde caem os semitons: entre o 3º e o 4º grau e entre o 7º e o 8º.", b: [["Subindo", "bpm=110 C4 D4 E4 F4 G4 A4 B4 C5:2"], ["Descendo", "bpm=110 C5 B4 A4 G4 F4 E4 D4 C4:2"]], kb: [48, 72] },
    { k: "kb", h: "Monte Sol maior", p: "Aplique a fórmula a partir do Sol: Sol, Lá, Si, Dó, Ré, Mi e… o 7º grau precisa estar um semitom abaixo do Sol. Toque as 7 notas.", r: [55, 79], t: ["G", "A", "B", "C", "D", "E", "F#"], e: "Sol maior tem **Fá♯**. O sustenido aparece para manter a fórmula." },
    { k: "table", h: "Os graus da escala", p: "Cada posição tem nome e função. Em Dó maior:", head: ["Grau", "Nome", "Em Dó"], rows: [["1º", "**Tônica**: o lar", "Dó"], ["2º", "Supertônica", "Ré"], ["3º", "Mediante", "Mi"], ["4º", "**Subdominante**", "Fá"], ["5º", "**Dominante**: a tensão principal", "Sol"], ["6º", "Superdominante", "Lá"], ["7º", "**Sensível**: quer subir à tônica", "Si"]] },
    { k: "txt", h: "A escala menor", p: "A **menor natural** tem a fórmula **T S T T S T T**. Ela é “escura”, melancólica.\n\nToda escala maior tem uma **relativa menor** com as mesmas notas, começando no 6º grau: Dó maior ↔ **Lá menor**. Mesmas notas, outro centro." },
    { k: "play", h: "Maior × menor", b: [["Dó maior", "bpm=110 C4 D4 E4 F4 G4 A4 B4 C5:2"], ["Dó menor", "bpm=110 C4 D4 Eb4 F4 G4 Ab4 Bb4 C5:2"], ["Lá menor (relativa)", "bpm=110 A3 B3 C4 D4 E4 F4 G4 A4:2"]], kb: [48, 72] },
    { k: "ear", q: "Esta escala é maior ou menor?", play: "bpm=110 D4 E4 F4 G4 A4 Bb4 C5 D5:2", o: ["Menor", "Maior"], c: 0, e: "Ré menor: a 3ª é menor (Ré–Fá) e soa mais escura." },
    { k: "txt", h: "Armadura de clave", p: "Em vez de escrever o ♯ toda vez, ele vai no começo da pauta: a **armadura**. Ela já entrega o tom.\n\nOrdem dos sustenidos: **Fá, Dó, Sol, Ré, Lá, Mi, Si**. A dos bemóis é o contrário: **Si, Mi, Lá, Ré, Sol, Dó, Fá**." },
    { k: "order", h: "Coloque os sustenidos na ordem da armadura", items: ["Fá♯", "Dó♯", "Sol♯", "Ré♯", "Lá♯"], e: "Frase para decorar: “**Fá**ça **Dó**ce, **Sol** **Ré**ina **Lá**…”. Ou invente a sua." },
    { k: "txt", h: "O círculo das quintas", p: "Suba uma **5ª** (Dó → Sol → Ré → Lá…) e a armadura ganha **1 sustenido** a cada passo. Desça uma 5ª (Dó → Fá → Si♭…) e ganha **1 bemol**. Tons vizinhos no círculo trocam só 1 nota: por isso é fácil modular entre eles. Veja no **Laboratório → Círculo das quintas**." },
    { k: "pick", q: "Qual é a relativa menor de Sol maior?", o: ["Mi menor", "Ré menor", "Sol menor", "Lá menor"], c: 0, e: "6º grau de Sol maior = Mi. Mesma armadura (1 sustenido)." },
    { k: "tf", q: "Escala e tonalidade são exatamente a mesma coisa.", a: false, e: "A escala é a lista de notas; a tonalidade é o sistema de relações em torno de uma tônica (inclui hierarquia, acordes, funções)." },
    { k: "mentor", t: "Para descobrir o tom de uma música, pergunte: **onde ela descansa?** Essa nota é a tônica. O resto vem dela." }
  ],
  q: [
    { q: "Qual é a fórmula da escala maior?", o: ["T T S T T T S", "T S T T S T T", "T T T S T T S", "S T T T S T T"], c: 0, e: "Semitons entre 3–4 e 7–8." },
    { q: "Quantos sustenidos tem a escala de Ré maior?", o: ["2 (Fá♯ e Dó♯)", "1 (Fá♯)", "3", "Nenhum"], c: 0, e: "Ré, Mi, Fá♯, Sol, Lá, Si, Dó♯." },
    { q: "O 5º grau da escala se chama:", o: ["Dominante", "Subdominante", "Mediante", "Sensível"], c: 0, e: "Dominante: o centro da tensão." },
    { q: "Qual a relativa menor de Fá maior?", o: ["Ré menor", "Fá menor", "Lá menor", "Mi menor"], c: 0, e: "6º grau de Fá = Ré." },
    { q: "A sensível é:", o: ["O 7º grau, meio tom abaixo da tônica", "O 4º grau", "A nota mais grave", "O 6º grau"], c: 0, e: "Ela “sente” a tônica e quer resolver nela." },
    { q: "Uma armadura com 3 bemóis indica:", o: ["Mi♭ maior ou Dó menor", "Lá maior ou Fá♯ menor", "Si♭ maior ou Sol menor", "Lá♭ maior"], c: 0, e: "Si♭, Mi♭, Lá♭: o penúltimo bemol (Mi♭) dá o tom maior." },
    { q: "Ouça: maior ou menor?", play: "bpm=110 A3 B3 C#4 D4 E4 F#4 G#4 A4:2", o: ["Maior", "Menor"], c: 0, e: "Lá maior (3 sustenidos)." }
  ]
};
LESSONS.t3 = {
  title: "Intervalos", sub: "Distância entre notas: números, qualidades, consonância e inversão",
  goals: ["Nomear qualquer intervalo (número + qualidade)", "Saber os semitons de cada intervalo", "Entender consonância, dissonância e inversão"],
  mis: "Escolha 3 músicas que você conhece e descubra o **primeiro intervalo** da melodia de cada uma (use o teclado do Laboratório). Anote como sua lista pessoal de referências.",
  deep: ["Intervalos justos: uníssono, 4ª, 5ª e 8ª. Os demais (2ª, 3ª, 6ª, 7ª) são maiores ou menores.", "Os gregos ligavam consonância a proporções simples: oitava 2:1, quinta 3:2, quarta 4:3. Quanto mais simples a razão, mais “fundido” o som."],
  s: [
    { k: "txt", h: "O que é um intervalo", p: "É a **distância entre duas notas**. Tem duas partes: o **número** (contando as letras, incluindo as duas pontas) e a **qualidade** (justo, maior, menor, aumentado, diminuto)." },
    { k: "txt", h: "Contando o número", p: "Dó → Mi: Dó, Ré, Mi = **3ª**.\nDó → Sol: Dó, Ré, Mi, Fá, Sol = **5ª**.\nRé → Fá: Ré, Mi, Fá = **3ª** também! Só que menor. Para saber a qualidade, contamos os **semitons**." },
    { k: "table", h: "Os 12 intervalos (toque nas linhas)", head: ["Semitons", "Nome", "Exemplo"], rows: [["1", "2ª menor", "Dó–Ré♭"], ["2", "2ª maior", "Dó–Ré"], ["3", "3ª menor", "Dó–Mi♭"], ["4", "3ª maior", "Dó–Mi"], ["5", "4ª justa", "Dó–Fá"], ["6", "Trítono (4ª aum./5ª dim.)", "Dó–Fá♯"], ["7", "5ª justa", "Dó–Sol"], ["8", "6ª menor", "Dó–Lá♭"], ["9", "6ª maior", "Dó–Lá"], ["10", "7ª menor", "Dó–Si♭"], ["11", "7ª maior", "Dó–Si"], ["12", "Oitava", "Dó–Dó"]],
      play: ["C4 Db4 [C4 Db4]:2", "C4 D4 [C4 D4]:2", "C4 Eb4 [C4 Eb4]:2", "C4 E4 [C4 E4]:2", "C4 F4 [C4 F4]:2", "C4 F#4 [C4 F#4]:2", "C4 G4 [C4 G4]:2", "C4 Ab4 [C4 Ab4]:2", "C4 A4 [C4 A4]:2", "C4 Bb4 [C4 Bb4]:2", "C4 B4 [C4 B4]:2", "C4 C5 [C4 C5]:2"] },
    { k: "play", h: "Referências para memorizar", p: "Associe cada intervalo ao começo de uma música conhecida.", b: [["2M · Parabéns a você", "G4:0.75 G4:0.25 A4 G4 C5 B4:2"], ["5J · Brilha, brilha", "C4 C4 G4 G4 A4 A4 G4:2"], ["4J · Marcha nupcial", "C4 F4:0.75 F4:0.25 F4:2"], ["8J · Over the Rainbow", "C4:2 C5:2"]] },
    { k: "ear", q: "Ouça: terça maior ou terça menor?", play: "C4 E4 [C4 E4]:2", o: ["Terça maior", "Terça menor"], c: 0, e: "4 semitons: terça maior, mais luminosa. A menor (3 semitons) soa mais escura." },
    { k: "txt", h: "Consonância e dissonância", p: "**Consonâncias perfeitas**: uníssono, 8ª, 5ª (e a 4ª, conforme o contexto) — soam “fundidas”.\n**Consonâncias imperfeitas**: 3ªs e 6ªs — doces.\n**Dissonâncias**: 2ªs, 7ªs e trítono — tensas, pedem movimento.\n\nDissonância **não é erro**: é tempero. Sem tensão, não existe resolução." },
    { k: "play", h: "Do mais estável ao mais tenso", b: [["8ª", "[C4 C5]:2"], ["5ª", "[C4 G4]:2"], ["3ª maior", "[C4 E4]:2"], ["2ª maior", "[C4 D4]:2"], ["7ª maior", "[C4 B4]:2"], ["2ª menor", "[C4 Db4]:2"]] },
    { k: "txt", tag: "6 semitons", h: "O trítono", p: "Três tons inteiros (Fá–Si, Dó–Fá♯). Divide a oitava exatamente ao meio e é muito instável. Ganhou o apelido de _diabolus in musica_ na teoria antiga. Hoje ele é o motor do acorde dominante, que você vai estudar em Harmonia." },
    { k: "txt", h: "Inversão de intervalos", p: "Inverter é passar a nota de baixo para cima (ou o contrário). Regras:\n• os números **somam 9** (3ª ↔ 6ª, 2ª ↔ 7ª, 4ª ↔ 5ª);\n• maior ↔ menor; justo ↔ justo; aumentado ↔ diminuto.\nDó–Mi (3ª maior) invertido = Mi–Dó (**6ª menor**)." },
    { k: "pick", q: "Qual é a inversão de uma 2ª maior?", o: ["7ª menor", "7ª maior", "6ª menor", "2ª menor"], c: 0, e: "2 + 7 = 9, e maior vira menor." },
    { k: "kb", h: "Toque uma 5ª justa acima de Ré", p: "Conte 7 semitons a partir de Ré (ou: Ré, Mi, Fá, Sol, **Lá**).", r: [50, 74], t: ["A"], e: "Ré–Lá: 5ª justa." },
    { k: "tf", q: "Ré–Fá e Dó–Mi têm o mesmo número (3ª), mas qualidades diferentes.", a: true, e: "Ré–Fá = 3 semitons (3ª menor). Dó–Mi = 4 semitons (3ª maior)." },
    { k: "mentor", t: "Intervalos são o “átomo” da música: escalas, acordes e melodias são combinações deles. Treine na **Academia → Intervalos** todo dia." }
  ],
  q: [
    { q: "Dó–Sol é uma:", o: ["5ª justa", "4ª justa", "6ª maior", "5ª aumentada"], c: 0, e: "7 semitons." },
    { q: "Quantos semitons tem uma 3ª menor?", o: ["3", "4", "2", "5"], c: 0, e: "Ex.: Lá–Dó." },
    { q: "A inversão de uma 4ª justa é:", o: ["5ª justa", "4ª aumentada", "5ª diminuta", "6ª menor"], c: 0, e: "4 + 5 = 9; justo continua justo." },
    { q: "O trítono tem:", o: ["6 semitons", "3 semitons", "5 semitons", "7 semitons"], c: 0, e: "Três tons inteiros." },
    { q: "Ré–Fá é uma:", o: ["3ª menor", "3ª maior", "2ª aumentada", "4ª diminuta"], c: 0, e: "Ré, Mi, Fá = 3ª; 3 semitons = menor." },
    { q: "Mi–Sol♯ é uma:", o: ["3ª maior", "3ª menor", "4ª justa", "2ª maior"], c: 0, e: "4 semitons." },
    { q: "Ouça o intervalo:", play: "C4 G4 [C4 G4]:2", o: ["5ª justa", "3ª maior", "Oitava", "2ª maior"], c: 0, e: "Som “aberto”, como Brilha, brilha." }
  ]
};
LESSONS.t4 = {
  title: "Partitura e ritmo escrito", sub: "Pauta, claves, figuras, pausas, ponto e compassos",
  goals: ["Ler notas nas claves de sol e de fá", "Saber o valor das figuras", "Entender compassos simples e compostos"],
  mis: "Pegue um hino do hinário com partitura (ou qualquer partitura) e escreva o nome das 8 primeiras notas da melodia. Depois bata o ritmo com a mão enquanto conta os tempos.",
  deep: ["Linhas da clave de sol, de baixo para cima: **Mi, Sol, Si, Ré, Fá**. Espaços: **Fá, Lá, Dó, Mi**.", "Linhas da clave de fá: **Sol, Si, Ré, Fá, Lá**. Espaços: **Lá, Dó, Mi, Sol**."],
  s: [
    { k: "txt", h: "A pauta", p: "Cinco linhas e quatro espaços, contados **de baixo para cima**. Quanto mais alto na pauta, mais aguda a nota. Notas além da pauta usam **linhas suplementares**." },
    { k: "staff", h: "Clave de sol", p: "A espiral da clave abraça a **2ª linha**: ali é o Sol (Sol4). Daí em diante é só contar.", n: "E4 F4 G4 A4 B4 C5 D5 E5 F5" },
    { k: "txt", h: "Truque para ler rápido", p: "**Linhas** (de baixo para cima): Mi – Sol – Si – Ré – Fá.\n**Espaços**: Fá – Lá – Dó – Mi.\nDecore os dois grupos e use os “pontos de apoio”: Sol na 2ª linha, Dó no 3º espaço." },
    { k: "pick", q: "Que nota é esta?", staff: "G4", o: ["Sol", "Lá", "Fá", "Mi"], c: 0, e: "2ª linha da clave de sol." },
    { k: "pick", q: "E esta?", staff: "C5", o: ["Dó", "Si", "Ré", "Lá"], c: 0, e: "3º espaço: Dó5." },
    { k: "staff", h: "Clave de fá", p: "Os dois pontinhos ficam ao redor da **4ª linha**: ali é o Fá3, abaixo do Dó central. Usada para baixo, mão esquerda do piano, tenor e baixo do coral.", n: "G2 A2 B2 C3 D3 E3 F3 G3 A3", clef: "fa" },
    { k: "staff", h: "O Dó central", p: "O **Dó4** fica numa linha suplementar logo abaixo da clave de sol (ou logo acima da clave de fá). É o ponto de encontro das duas mãos no piano.", n: "C4 D4 E4" },
    { k: "table", h: "Figuras de duração", p: "No compasso 4/4, a semínima vale 1 tempo.", head: ["Figura", "Tempos", "Cabem num 4/4"], rows: [["Semibreve 𝅝", "4", "1"], ["Mínima 𝅗𝅥", "2", "2"], ["Semínima ♩", "1", "4"], ["Colcheia ♪", "½", "8"], ["Semicolcheia 𝅘𝅥𝅯", "¼", "16"]] },
    { k: "txt", h: "Pausas e ponto de aumento", p: "Cada figura tem uma **pausa** do mesmo valor: silêncio também é medido.\n\nO **ponto** depois da figura soma metade do valor: mínima pontuada = 2 + 1 = **3 tempos**; semínima pontuada = 1 + ½ = **1½**." },
    { k: "txt", h: "Compasso", p: "A fórmula de compasso tem dois números. **Em cima**: quantos tempos. **Embaixo**: qual figura vale um tempo (4 = semínima, 8 = colcheia).\n\n**4/4**: o mais comum (pop, louvor). **3/4**: valsa. **2/4**: marcha, samba, baião. **6/8**: composto, sentido em 2 tempos de 3 colcheias (muitas baladas e hinos)." },
    { k: "rhythm", h: "Ouça as figuras (grade = semicolcheias)", p: "Cada quadradinho é uma semicolcheia; cada grupo de 4 é um tempo.", pats: [["Semínimas", "x... x... x... x..."], ["Colcheias", "x.x. x.x. x.x. x.x."], ["Semicolcheias", "xxxx xxxx xxxx xxxx"], ["Misturado", "x... x.x. xxxx x..."]], bpm: 76 },
    { k: "tap", h: "Toque este compasso", p: "Depois das 4 batidas de contagem, toque na área abaixo. O metrônomo continua marcando o tempo.", pat: "x... x... x.x. x...", bpm: 72 },
    { k: "pick", q: "Quantas colcheias cabem num compasso 4/4?", o: ["8", "4", "16", "6"], c: 0, e: "4 tempos × 2 colcheias." },
    { k: "mentor", t: "Ler partitura é como ler um texto: no começo, letra por letra; depois, palavras inteiras. **Leia um pouco todo dia**, mesmo que devagar." }
  ],
  q: [
    { q: "Que nota é esta (clave de sol)?", staff: "F5", o: ["Fá", "Mi", "Sol", "Ré"], c: 0, e: "5ª linha: Fá5." },
    { q: "Que nota é esta (clave de fá)?", staff: "A3", clef: "fa", o: ["Lá", "Dó", "Si", "Fá"], c: 0, e: "5ª linha da clave de fá: Lá3." },
    { q: "Uma mínima pontuada vale:", o: ["3 tempos", "2 tempos", "2½ tempos", "4 tempos"], c: 0, e: "2 + metade (1)." },
    { q: "Num compasso 3/4 cabem:", o: ["3 semínimas", "3 mínimas", "4 semínimas", "6 semínimas"], c: 0, e: "3 tempos de semínima." },
    { q: "O 6/8 normalmente é sentido em:", o: ["2 tempos de 3 colcheias", "6 tempos iguais", "3 tempos de 2 colcheias", "8 tempos"], c: 0, e: "É um compasso composto binário." },
    { q: "Quantas semicolcheias cabem numa semínima?", o: ["4", "2", "8", "3"], c: 0, e: "1 tempo = 4 semicolcheias." },
    { q: "Que nota é esta?", staff: "D5", o: ["Ré", "Dó", "Mi", "Si"], c: 0, e: "4ª linha da clave de sol." }
  ]
};

LESSONS.p1 = {
  title: "Como o ouvido aprende", sub: "Ouvido relativo, dó móvel, cantar e a rotina que funciona",
  goals: ["Entender ouvido relativo × absoluto", "Usar o dó móvel (números) para tirar músicas", "Montar uma rotina diária de percepção"],
  mis: "Cante (pode ser baixinho) a melodia de um louvor que você conhece usando **números** em vez da letra: 1, 2, 3… a partir da tônica. Depois toque no instrumento.",
  deep: ["Quem aprende música cedo e/ou fala línguas tonais (como mandarim) tem mais chance de desenvolver ouvido absoluto, mas ele é raro na população em geral.", "Prática espaçada (um pouco por dia) e intercalada (misturar tipos de exercício) são dois dos efeitos mais robustos da psicologia da aprendizagem."],
  s: [
    { k: "mentor", t: "Shalom! Eu sou **Davi**. Antes de ser rei, eu era pastor e tocava harpa. Quando eu tocava para Saul, ele se acalmava (1 Sm 16.23). Tocar bem começa em **ouvir bem**." },
    { k: "txt", h: "Ouvido não é dom fixo", p: "“Eu não tenho ouvido” quase sempre significa “eu nunca treinei”. Percepção é uma **habilidade**, como ler. Com treino diário, qualquer pessoa melhora muito em reconhecer intervalos, acordes e ritmos." },
    { k: "txt", h: "Absoluto × relativo", p: "**Ouvido absoluto**: nomear uma nota sem referência (“isso é um Fá♯”). É raro e **não é necessário** para ser profissional.\n\n**Ouvido relativo**: reconhecer as **relações** entre os sons (“essa nota é a 5ª do tom”). É o que os músicos usam no dia a dia, e pode ser treinado." },
    { k: "txt", h: "O dó móvel", p: "No **dó móvel**, a tônica de qualquer tom se chama “dó” (ou **1**). Em Sol maior, Sol vira 1; em Ré maior, Ré vira 1. Assim você pensa a música em **números**, e transpor fica automático." },
    { k: "play", h: "Mesma melodia, dois tons", p: "As notas mudam, os números não: **1 1 5 5 6 6 5**.", b: [["Em Dó", "C4 C4 G4 G4 A4 A4 G4:2"], ["Em Sol", "G3 G3 D4 D4 E4 E4 D4:2"], ["Em Mi", "E4 E4 B4 B4 C#5 C#5 B4:2"]], kb: [48, 76] },
    { k: "list", h: "Os pilares da percepção", items: ["**Intervalos**: a distância entre notas", "**Graus**: a função de cada nota no tom", "**Acordes**: maior, menor, com sétima…", "**Progressões**: a sequência de acordes", "**Ritmo**: duração e acento", "**Timbre e afinação**: a cor e a precisão"] },
    { k: "ear", q: "Depois da cadência, a última nota soa resolvida ou tensa?", play: "[C4 E4 G4] [F4 A4 C5] [G4 B4 D5] [C4 E4 G4] - B4:2", o: ["Tensa, querendo subir", "Resolvida, em repouso"], c: 0, e: "Era o **Si**, a sensível (grau 7). Ela “puxa” para o Dó. Sentir isso é o começo do ouvido funcional." },
    { k: "txt", h: "Cante!", p: "O segredo dos conservatórios: **o que você consegue cantar, você consegue ouvir**. Cantar intervalos e escalas (solfejo) cria um “mapa interno”. Não precisa cantar bonito, só afinado e com atenção." },
    { k: "list", h: "A rotina que funciona", items: ["**10 minutos por dia** valem mais que 2 horas no sábado", "**Misture** tipos de exercício na mesma sessão", "Comece no nível em que você acerta ~70–80%", "Cante a resposta antes de clicar", "Revise os erros: **toque nas opções** para comparar"] },
    { k: "tf", q: "Para ser músico profissional é preciso ter ouvido absoluto.", a: false, e: "A imensa maioria dos músicos profissionais usa ouvido relativo." },
    { k: "chat", who: "Bia, do louvor", c: "#D6336C", q: "Eu queria tirar música de ouvido igual o tecladista, mas acho que não nasci com isso 😕", o: [
      { t: "Ninguém nasce tirando música. É treino: começa achando a tônica, depois o baixo, depois os acordes. 10 minutos por dia já mudam muito.", v: 2, r: "Sério? Então dá pra começar hoje?", tip: "Isso! Mostrou que é habilidade treinável e deu o caminho." },
      { t: "É verdade, tem gente que tem e gente que não tem.", v: 0, r: "Ah, então não adianta…", tip: "Esse mito desanima. Percepção se treina." },
      { t: "Só se você tiver ouvido absoluto.", v: 0, r: "Nossa, nem sei o que é isso 😅", tip: "Ouvido absoluto não é necessário. O relativo é o que importa." }] },
    { k: "mentor", t: "Cada dia na **Academia de ouvido** é uma pedra na funda. Pouco a pouco, você derruba o gigante do “eu não tenho ouvido”." }
  ],
  q: [
    { q: "Ouvido relativo é:", o: ["Reconhecer notas pelas relações entre elas", "Nomear notas sem referência", "Um dom que poucos têm", "Ouvir só com um ouvido"], c: 0, e: "Relações: intervalos, graus, funções." },
    { q: "No dó móvel, em Ré maior, qual nota é o “1”?", o: ["Ré", "Dó", "Lá", "Fá♯"], c: 0, e: "A tônica sempre é o 1." },
    { q: "Qual rotina rende mais para o ouvido?", o: ["10 minutos todos os dias", "3 horas uma vez por mês", "Só ouvir música passivamente", "Treinar só quando tiver ensaio"], c: 0, e: "Prática espaçada e frequente." },
    { q: "A sensível (7º grau) tende a:", o: ["Subir para a tônica", "Descer para o 5º grau", "Ficar parada", "Subir para o 2º grau"], c: 0, e: "Ela está meio tom abaixo da tônica." },
    { q: "Por que cantar ajuda a percepção?", o: ["Cria um mapa interno dos sons", "Porque deixa a voz bonita", "Não ajuda", "Porque substitui o instrumento"], c: 0, e: "Cantar internaliza alturas e intervalos." },
    { q: "Transpor uma melodia de Dó para Sol muda:", o: ["Os nomes das notas, não os números (graus)", "Os números, não os nomes", "O ritmo", "Nada"], c: 0, e: "1 1 5 5 6 6 5 em qualquer tom." }
  ]
};
LESSONS.p2 = {
  title: "Ouvindo intervalos", sub: "Melódicos e harmônicos, subindo e descendo, por cor e por referência",
  goals: ["Reconhecer intervalos subindo e descendo", "Diferenciar intervalos melódicos e harmônicos", "Ouvir a “cor” de cada intervalo"],
  mis: "Faça 3 rodadas na Academia → Intervalos hoje (manhã, tarde, noite). Anote o intervalo que você mais confunde e crie uma referência musical sua para ele.",
  deep: ["Referências descendentes conhecidas: 3ª menor = “Hey Jude”; 3ª maior = “Swing Low, Sweet Chariot”; 2ª menor = início de “Für Elise”.", "Com o tempo, deixe as referências de lado e reconheça pela **qualidade sonora**: as referências são muletas para o começo."],
  s: [
    { k: "txt", h: "Melódico × harmônico", p: "**Melódico**: uma nota depois da outra (como numa melodia).\n**Harmônico**: as duas juntas (como num acorde). O harmônico é mais difícil: o ouvido precisa “separar” os sons." },
    { k: "play", h: "Cores dos intervalos", p: "Ouça e associe uma sensação. Não existe resposta certa: crie a sua.", b: [["3m · triste", "C4 Eb4 [C4 Eb4]:2"], ["3M · alegre", "C4 E4 [C4 E4]:2"], ["4J · chamado", "C4 F4 [C4 F4]:2"], ["5J · aberto, oco", "C4 G4 [C4 G4]:2"], ["Trítono · instável", "C4 F#4 [C4 F#4]:2"], ["7M · quase chega", "C4 B4 [C4 B4]:2"]] },
    { k: "ear", q: "Qual intervalo subiu?", play: "D4 A4:2", o: ["5ª justa", "4ª justa", "6ª maior"], c: 0, e: "Ré–Lá: 5ª justa, o “Brilha, brilha”." },
    { k: "ear", q: "E este?", play: "E4 A4:2", o: ["4ª justa", "5ª justa", "3ª maior"], c: 0, e: "Mi–Lá: 4ª justa, a da Marcha nupcial." },
    { k: "txt", h: "Estratégia em 3 passos", p: "1. **Cante** as duas notas por dentro.\n2. Pergunte: é pequeno (2ª, 3ª), médio (4ª, 5ª) ou grande (6ª, 7ª, 8ª)?\n3. Compare com sua referência ou “preencha” cantando a escala entre as notas." },
    { k: "play", h: "Descendentes", p: "Descer soa diferente de subir! Treine os dois.", b: [["5J descendo", "G4 C4:2"], ["3m descendo", "E4 C#4:2"], ["3M descendo", "E4 C4:2"], ["2m descendo", "E5 D#5 E5 D#5 E5:2"]] },
    { k: "ear", q: "Ouça as duas notas juntas: terça maior ou menor?", play: "[A3 C4]:2 - [A3 C4]:2", o: ["Terça menor", "Terça maior"], c: 0, e: "Lá–Dó: 3 semitons, terça menor." },
    { k: "ear", q: "Que intervalo é este, bem instável?", play: "C4 F#4 [C4 F#4]:2", o: ["Trítono", "5ª justa", "4ª justa"], c: 0, e: "6 semitons: o trítono. Note como ele “não para quieto”." },
    { k: "tf", q: "No piano, uma 6ª menor e uma 5ª aumentada soam iguais.", a: true, e: "São enarmônicas: 8 semitons (Dó–Lá♭ = Dó–Sol♯). O nome muda conforme o contexto." },
    { k: "txt", h: "Intervalos dentro dos acordes", p: "Um acorde maior é uma **3ª maior** com uma **3ª menor** em cima (Dó–Mi–Sol). O menor é o contrário. Quem ouve bem terças, ouve bem acordes." },
    { k: "mentor", t: "Os salmos subiam e desciam como as montanhas de Judá. **Ouça os intervalos subindo e descendo**, e cante cada um." }
  ],
  q: [
    { q: "Ouça: que intervalo é?", play: "C4 E4:2", o: ["3ª maior", "3ª menor", "4ª justa", "2ª maior"], c: 0, e: "Dó–Mi." },
    { q: "Ouça: que intervalo é?", play: "C4 C5:2", o: ["Oitava", "5ª justa", "7ª maior", "6ª maior"], c: 0, e: "Mesma nota, uma oitava acima." },
    { q: "Ouça (juntas): que intervalo?", play: "[C4 G4]:2", o: ["5ª justa", "3ª maior", "Trítono", "2ª maior"], c: 0, e: "Som aberto e estável." },
    { q: "Intervalo harmônico é quando:", o: ["As notas soam juntas", "As notas soam uma depois da outra", "Só existe em acordes menores", "Uma nota é de outro instrumento"], c: 0, e: "Juntas = harmônico." },
    { q: "Ouça: que intervalo é?", play: "C4 D4:2", o: ["2ª maior", "2ª menor", "3ª menor", "4ª justa"], c: 0, e: "Um tom: “Pa-ra-béns”." },
    { q: "Ouça: subindo, que intervalo?", play: "G3 E4:2", o: ["6ª maior", "5ª justa", "7ª menor", "4ª justa"], c: 0, e: "Sol–Mi: 9 semitons." }
  ]
};
LESSONS.p3 = {
  title: "Ouvindo acordes e funções", sub: "Maior e menor, tônica–subdominante–dominante, cadências e o baixo",
  goals: ["Diferenciar acordes maiores, menores, diminutos e aumentados", "Ouvir a função (T, SD, D) de cada acorde", "Usar o baixo para tirar a harmonia"],
  mis: "Pegue um louvor simples e tente descobrir **só o baixo** (a nota mais grave de cada acorde) cantando junto. Depois diga se cada acorde é maior ou menor.",
  deep: ["Quase todo o repertório popular usa poucas funções. Saber o grau do baixo + a qualidade do acorde já resolve a maioria das músicas.", "Ouvir **progressões inteiras** (e não acorde por acorde) é mais rápido: o ouvido reconhece padrões como I–V–vi–IV."],
  s: [
    { k: "txt", h: "Tudo depende da terça", p: "Acorde **maior** e **menor** só mudam uma nota: a **3ª**. Ela define a cor. A 5ª é igual nos dois." },
    { k: "play", h: "As 4 tríades", b: [["Maior", "[C4 E4 G4]:2 C4:0.4 E4:0.4 G4:1.2"], ["Menor", "[C4 Eb4 G4]:2 C4:0.4 Eb4:0.4 G4:1.2"], ["Diminuto", "[C4 Eb4 Gb4]:2 C4:0.4 Eb4:0.4 Gb4:1.2"], ["Aumentado", "[C4 E4 G#4]:2 C4:0.4 E4:0.4 G#4:1.2"]], kb: [48, 72] },
    { k: "ear", q: "Este acorde é maior ou menor?", play: "[D4 F4 A4]:3", o: ["Menor", "Maior"], c: 0, e: "Ré menor (Ré–Fá–Lá)." },
    { k: "ear", q: "E este?", play: "[F3 A3 C4 F4]:3", o: ["Maior", "Menor"], c: 0, e: "Fá maior." },
    { k: "txt", h: "Três funções", p: "Num tom, os acordes têm papéis:\n• **Tônica (T)**: repouso, casa (I).\n• **Subdominante (SD)**: afastamento, preparação (IV).\n• **Dominante (D)**: tensão que pede volta (V).\nA música vive desse ciclo: casa → saída → tensão → casa." },
    { k: "play", h: "A cadência I – IV – V – I", p: "Ouça o “caminho de volta para casa”.", b: [["Em Dó", "bpm=80 [C3 C4 E4 G4] [F2 C4 F4 A4] [G2 B3 D4 G4] [C3 C4 E4 G4]:2"]] },
    { k: "ear", q: "Esta frase termina resolvida ou suspensa?", play: "bpm=80 [C3 C4 E4 G4] [F2 C4 F4 A4] [C3 C4 E4 G4] [G2 B3 D4 G4]:2", o: ["Suspensa (parou no V)", "Resolvida (voltou ao I)"], c: 0, e: "Parou na dominante: soa como uma pergunta. Chama-se **meia cadência**." },
    { k: "txt", h: "Siga o baixo", p: "Para tirar música de ouvido, comece pelo **baixo**: ele costuma tocar a fundamental do acorde. Descubra o grau do baixo (1, 4, 5, 6…) e depois decida se o acorde é maior ou menor. Pronto: você tem a cifra." },
    { k: "play", h: "A progressão mais famosa do pop", p: "**I – V – vi – IV**. Está em centenas de músicas e louvores.", b: [["Em Dó", "bpm=88 [C3 C4 E4 G4]:2 [G2 B3 D4 G4]:2 [A2 A3 C4 E4]:2 [F2 A3 C4 F4]:2"]] },
    { k: "ear", q: "Qual progressão tocou?", play: "bpm=88 [C3 C4 E4 G4]:2 [A2 A3 C4 E4]:2 [F2 A3 C4 F4]:2 [G2 B3 D4 G4]:2", o: ["I – vi – IV – V", "I – IV – V – I", "I – V – vi – IV"], c: 0, e: "Dó – Lám – Fá – Sol: a progressão “anos 50”." },
    { k: "play", h: "Com sétima", b: [["7 (dominante)", "[C4 E4 G4 Bb4]:3"], ["7M (sétima maior)", "[C4 E4 G4 B4]:3"], ["m7 (menor com 7ª)", "[C4 Eb4 G4 Bb4]:3"]] },
    { k: "tf", q: "O mesmo acorde (ex.: Sol maior) pode ter funções diferentes em tons diferentes.", a: true, e: "Sol é dominante em Dó maior, mas é tônica em Sol maior e subdominante em Ré maior." },
    { k: "mentor", t: "Quando eu tocava, Saul se acalmava: é a força da **resolução**. Ouça sempre: estou em casa, saindo ou voltando?" }
  ],
  q: [
    { q: "O que diferencia um acorde maior de um menor?", o: ["A terça", "A quinta", "A fundamental", "O volume"], c: 0, e: "Maior: 3ª maior; menor: 3ª menor." },
    { q: "Ouça: que tipo de acorde?", play: "[A3 C4 E4]:3", o: ["Menor", "Maior", "Diminuto", "Aumentado"], c: 0, e: "Lá menor." },
    { q: "Ouça: que tipo de acorde?", play: "[C4 E4 G#4]:3", o: ["Aumentado", "Maior", "Menor", "Diminuto"], c: 0, e: "Duas terças maiores: som de suspense." },
    { q: "A função de dominante é:", o: ["Criar tensão que pede volta à tônica", "Dar repouso", "Começar a música", "Mudar de tom sempre"], c: 0, e: "D → T." },
    { q: "Uma frase que termina no V chama-se:", o: ["Meia cadência", "Cadência perfeita", "Cadência plagal", "Modulação"], c: 0, e: "Soa como pergunta." },
    { q: "Para tirar a harmonia de ouvido, por onde começar?", o: ["Pelo baixo", "Pelo prato da bateria", "Pela letra", "Pelo solo de guitarra"], c: 0, e: "O baixo costuma tocar a fundamental." },
    { q: "Ouça: qual progressão?", play: "bpm=88 [C3 C4 E4 G4]:2 [F2 A3 C4 F4]:2 [G2 B3 D4 G4]:2 [C3 C4 E4 G4]:2", o: ["I – IV – V – I", "I – V – vi – IV", "I – vi – IV – V", "ii – V – I – I"], c: 0, e: "A cadência clássica." }
  ]
};
LESSONS.p4 = {
  title: "Ditado, timbre e afinação", sub: "Escrever o que ouve, reconhecer cores e afinar de ouvido",
  goals: ["Seguir os passos do ditado melódico", "Entender timbre como receita de harmônicos", "Perceber afinação em cents e batimentos"],
  mis: "Grave no celular uma melodia curta de um louvor (só a voz). Escreva os graus (1, 2, 3…) e o ritmo. Confira tocando no teclado do app.",
  deep: ["A 3ª maior “pura” (razão 5:4) é cerca de **14 cents mais baixa** que a 3ª maior do piano temperado. Corais e quartetos de cordas ajustam isso naturalmente.", "Afinadores eletrônicos mostram cents: ±5 cents já é afinação muito boa para a maioria dos contextos."],
  s: [
    { k: "txt", h: "Ditado melódico em 4 passos", p: "1. **Tônica**: onde a música repousa?\n2. **Contorno**: sobe, desce, salta?\n3. **Graus**: cante e descubra os números.\n4. **Ritmo**: bata com a mão e encaixe nos tempos.\nNão tente tudo de uma vez: uma camada por vez." },
    { k: "ear", q: "Qual sequência de graus tocou? (tom de Dó)", play: "bpm=96 C4 D4 E4 C4 G4:2", o: ["1 2 3 1 5", "1 2 3 4 5", "1 3 2 1 5", "1 2 1 3 5"], c: 0, e: "Dó Ré Mi Dó Sol. Repare no salto final: 1 → 5." },
    { k: "ear", q: "E esta?", play: "bpm=96 E4 D4 C4 D4 E4 E4 E4:2", o: ["3 2 1 2 3 3 3", "3 2 1 2 3 4 5", "1 2 3 2 1 1 1", "5 4 3 4 5 5 5"], c: 0, e: "Mi Ré Dó Ré Mi Mi Mi: o começo de uma cantiga muito conhecida." },
    { k: "txt", h: "Timbre: a receita de harmônicos", p: "Uma nota real não é um som só: é uma **fundamental** mais vários **harmônicos** (múltiplos dela). A **mistura** desses harmônicos e o **envelope** (o ataque, como o som começa) formam o timbre." },
    { k: "play", h: "Ouça os harmônicos", b: [["Série harmônica", "fx:harm"], ["4 formas de onda", "fx:waves"]], after: "Na série harmônica você ouve oitava, quinta, quarta, terça maior… É a “matéria-prima” da harmonia." },
    { k: "txt", h: "Afinação e cents", p: "Um semitom tem **100 cents**. Diferenças de 5 a 10 cents já incomodam um ouvido treinado. Cantores, violinistas e trombonistas ajustam a afinação o tempo todo, **de ouvido**." },
    { k: "ear", q: "A segunda nota está mais aguda ou mais grave?", play: "fx:tune:30", o: ["Mais aguda", "Mais grave"], c: 0, e: "Estava 30 cents acima. Treine em Academia → Afinação fina." },
    { k: "txt", h: "Batimentos: o segredo de afinar", p: "Duas notas quase iguais tocadas juntas criam um “tremor” no volume: o **batimento**. Quanto mais perto da afinação, mais lento ele fica. Quando some, está afinado. É assim que se afina violão por harmônicos e órgão de tubos." },
    { k: "play", h: "Ouça os batimentos", b: [["3 por segundo", "fx:beats:3"], ["1 por segundo", "fx:beats:1"], ["Afinado (0)", "fx:beats:0"]] },
    { k: "tf", q: "100 cents equivalem a um tom inteiro.", a: false, e: "100 cents = 1 semitom. Um tom = 200 cents." },
    { k: "txt", h: "Afinação em grupo", p: "No coral e na banda, o que importa é **afinar com o grupo**, não com um número. Ouça a voz ao lado, encaixe a sua e evite cantar “em cima” do volume dos outros." },
    { k: "mentor", t: "Nos meus salmos, a harpa e a voz tinham que estar em acordo. **Ouvir a afinação é um ato de humildade**: ajustar-se ao outro." }
  ],
  q: [
    { q: "Qual o primeiro passo do ditado melódico?", o: ["Achar a tônica", "Escrever o ritmo", "Adivinhar o compasso", "Contar quantas notas"], c: 0, e: "Sem tônica, não há graus." },
    { q: "Timbre depende de:", o: ["Harmônicos e envelope", "Só do volume", "Só da altura", "Do andamento"], c: 0, e: "A receita de harmônicos + como o som começa e termina." },
    { q: "Um semitom tem:", o: ["100 cents", "50 cents", "200 cents", "12 cents"], c: 0, e: "O tom tem 200." },
    { q: "Ao afinar, os batimentos:", o: ["Ficam mais lentos até sumir", "Ficam mais rápidos", "Não mudam", "Só existem no piano"], c: 0, e: "Batimento zero = afinado." },
    { q: "Ouça: a 2ª nota está…", play: "fx:tune:-40", o: ["Mais grave", "Mais aguda"], c: 0, e: "40 cents abaixo." },
    { q: "Ouça: qual sequência de graus?", play: "bpm=96 G4 E4 C4 E4 G4:2", o: ["5 3 1 3 5", "5 4 3 2 1", "1 3 5 3 1", "5 3 2 3 5"], c: 0, e: "Arpejo do acorde de Dó: Sol Mi Dó Mi Sol." }
  ]
};
