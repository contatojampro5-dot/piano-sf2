// Lições — Nível 3 (Ritmo e prática, Miriã) e Nível 4 (Harmonia, Asafe).
"use strict";
LESSONS.r1 = {
  title: "Pulso, andamento e subdivisão", sub: "O tempo que não corre: BPM, subdivisão, acento e contratempo",
  goals: ["Separar pulso, ritmo e andamento", "Usar a subdivisão para não correr", "Tocar no tempo e no contratempo"],
  mis: "Ligue o metrônomo do Laboratório a 70 BPM e bata palmas só no **2 e no 4** por 2 minutos. Depois faça o mesmo no **contratempo** (entre os cliques).",
  deep: ["Subdividir mentalmente (contar “1 e 2 e”) é a técnica número um de músicos de estúdio para manter o tempo estável.", "Acelerar no refrão é natural (empolgação), por isso bandas profissionais ensaiam com metrônomo e, ao vivo, muitas usam clique no fone (in-ear)."],
  s: [
    { k: "mentor", t: "Eu sou **Miriã**, irmã de Moisés. Depois da travessia do mar Vermelho, peguei o tamborim e as mulheres saíram atrás de mim, dançando (Êx 15.20). **Ritmo é o que faz um povo se mover junto.**" },
    { k: "txt", h: "Três palavras diferentes", p: "**Pulso**: a batida regular, a que você marca com o pé.\n**Andamento**: a velocidade do pulso (em BPM, batidas por minuto).\n**Ritmo**: o desenho de durações por cima do pulso (longas, curtas, pausas)." },
    { k: "tap", h: "Marque o pulso", p: "Toque uma vez em cada tempo, junto com o metrônomo.", pat: "x... x... x... x...", bpm: 72 },
    { k: "table", h: "Andamentos em italiano", p: "Valores aproximados: mudam conforme a época e o estilo.", head: ["Termo", "Caráter", "≈ BPM"], rows: [["Largo", "muito lento, amplo", "40–60"], ["Adagio", "lento, à vontade", "66–76"], ["Andante", "andando", "76–108"], ["Moderato", "moderado", "108–120"], ["Allegro", "rápido, alegre", "120–168"], ["Presto", "muito rápido", "168–200"]] },
    { k: "txt", h: "Subdivisão: o antídoto contra correr", p: "Entre um tempo e outro, conte as partes: “**1 e 2 e**” (colcheias), “**1 e e 2 e e**” (tercinas) ou “**1 e & a**” (semicolcheias). Quem subdivide sabe exatamente quando vem o próximo tempo, e não corre." },
    { k: "rhythm", h: "Três subdivisões do mesmo pulso", pats: [["Colcheias (2 por tempo)", "x.x. x.x. x.x. x.x."], ["Semicolcheias (4 por tempo)", "xxxx xxxx xxxx xxxx"]], bpm: 70 },
    { k: "rhythm", h: "Tercinas (3 por tempo)", pat: "xxx xxx xxx xxx", steps: 3, bpm: 70 },
    { k: "txt", h: "Acento e backbeat", p: "No 4/4, a hierarquia é: **1** forte, 2 fraco, **3** meio forte, 4 fraco. Mas no pop, rock e louvor, a caixa da bateria acentua o **2 e o 4**: é o **backbeat**, que dá o “balanço”." },
    { k: "rhythm", h: "Ouça o backbeat", p: "Os quadrados laranja são acentos.", pat: "x... X... x... X...", bpm: 84 },
    { k: "tap", h: "Agora o contratempo", p: "Toque **entre** os cliques (no “e”). Sinta o tempo forte em silêncio.", pat: "..x. ..x. ..x. ..x.", bpm: 66 },
    { k: "chat", who: "Lucas, guitarrista", c: "#1C7ED6", q: "No domingo o baterista acelerou no refrão e todo mundo foi junto. Como evitar? 😬", o: [
      { t: "Ensaiar com metrônomo, subdividir mentalmente e, se der, usar clique no fone ao vivo. E combinar que quem segura o tempo é a bateria e o baixo.", v: 2, r: "Faz sentido. Vou propor o metrônomo no ensaio de quinta.", tip: "Isso. Ensaio com clique + subdivisão + papéis claros." },
      { t: "É só tocar mais alto que ele, aí ele segue você.", v: 0, r: "Hmm, acho que vai virar briga de volume 😅", tip: "Volume não resolve tempo, só cria guerra no palco." },
      { t: "Normal, refrão é pra acelerar mesmo.", v: 1, r: "Mas aí o cantor ficou sem fôlego…", tip: "Energia no refrão vem de dinâmica e arranjo, não de acelerar." }] },
    { k: "mentor", t: "O tamborim não corre na frente do povo: **ele conduz**. Seu tempo firme é um serviço para todos que tocam com você." }
  ],
  q: [
    { q: "BPM mede:", o: ["O andamento (batidas por minuto)", "O volume", "A afinação", "O número de compassos"], c: 0, e: "Batidas por minuto." },
    { q: "Pulso é:", o: ["A batida regular de fundo", "O desenho rítmico da melodia", "O acorde principal", "A nota mais grave"], c: 0, e: "O que você marca com o pé." },
    { q: "Subdividir ajuda a:", o: ["Manter o tempo estável", "Tocar mais alto", "Mudar de tom", "Afinar o instrumento"], c: 0, e: "Você sabe exatamente onde cai o próximo tempo." },
    { q: "Backbeat é o acento nos tempos:", o: ["2 e 4", "1 e 3", "Só no 1", "Em todos"], c: 0, e: "A caixa da bateria no pop." },
    { q: "“Allegro” indica um andamento:", o: ["Rápido", "Muito lento", "Moderado", "Que acelera"], c: 0, e: "Rápido e alegre." },
    { q: "Contratempo é tocar:", o: ["Na parte fraca do tempo (o “e”)", "Sempre no tempo 1", "Fora do compasso", "Mais devagar"], c: 0, e: "Entre os cliques." }
  ]
};
LESSONS.r2 = {
  title: "Síncope e levadas", sub: "Síncope, tresillo, 6/8, valsa e o lugar do bumbo em cada estilo",
  goals: ["Reconhecer e tocar síncope", "Sentir o tresillo (3 + 3 + 2)", "Diferenciar 4/4, 3/4 e 6/8 na prática"],
  mis: "Escolha 3 louvores do repertório da sua igreja e diga: está em 4/4, 3/4 ou 6/8? Conte em voz alta enquanto ouve.",
  deep: ["O agrupamento **3 + 3 + 2** (tresillo) atravessa a música afro-latina e brasileira: baião, maracatu, reggaeton, muito pop atual.", "Swing: no jazz, as colcheias não são iguais: a primeira dura mais que a segunda. Essa “inégalité” também existia no Barroco francês."],
  s: [
    { k: "txt", h: "Síncope", p: "Uma **síncope** acontece quando um som começa num tempo (ou parte) fraco e se prolonga sobre o forte. O acento “escorrega” e o corpo quer dançar. É a alma do samba, do choro e do funk." },
    { k: "rhythm", h: "Reto × sincopado", pats: [["Reto", "x.x. x.x. x.x. x.x."], ["Sincopado", "x.xx .x.x x.xx .x.x"]], bpm: 80 },
    { k: "txt", h: "O tresillo: 3 + 3 + 2", p: "Divida 8 semicolcheias em **3 + 3 + 2**: toque na 1ª, na 4ª e na 7ª. Esse desenho está no **baião** de Luiz Gonzaga, no **reggaeton** e em muito pop. Fale: “**UM** dois três **UM** dois três **UM** dois”." },
    { k: "rhythm", h: "Ouça o tresillo", pat: "X..x..x. X..x..x.", beats: 4, bpm: 84 },
    { k: "tap", h: "Toque o tresillo", pat: "x..x..x. x..x..x.", bpm: 76 },
    { k: "pick", q: "O tresillo agrupa 8 subdivisões em:", o: ["3 + 3 + 2", "4 + 4", "2 + 2 + 2 + 2", "3 + 2 + 3"], c: 0, e: "Três, três, dois." },
    { k: "txt", h: "4/4 de pop e louvor", p: "A levada básica: **bumbo no 1 e no 3**, **caixa no 2 e no 4**, chimbal em colcheias. Variações empurram o bumbo para o contratempo e dão movimento." },
    { k: "rhythm", h: "Valsa 3/4", p: "UM dois três, UM dois três: o primeiro tempo é forte.", pat: "X... x... x...", beats: 3, bpm: 120 },
    { k: "rhythm", h: "Balada em 6/8", p: "Seis colcheias em dois grupos: **UM** dois três **QUA**tro cinco seis. Sinta 2 pulsos grandes. Muitos hinos e baladas de adoração usam esse balanço.", pat: "Xxx xxx Xxx xxx", steps: 3, beats: 4, bpm: 66 },
    { k: "pick", q: "Um louvor com o balanço “UM-dois-três-QUA-tro-cin-co-seis” está provavelmente em:", o: ["6/8", "4/4", "2/4", "3/4"], c: 0, e: "Dois grupos de três colcheias." },
    { k: "txt", h: "Swing e groove", p: "Groove é a sensação de “encaixe” que faz a gente mexer a cabeça. Vem de pequenas variações de tempo e dinâmica. No **swing**, as colcheias ficam desiguais (longa-curta), como “dum-da dum-da”." },
    { k: "play", h: "Colcheias retas × swingadas", b: [["Retas", "bpm=110 C4:0.5 E4:0.5 G4:0.5 E4:0.5 C4:0.5 E4:0.5 G4:1"], ["Swing", "bpm=110 C4:0.66 E4:0.34 G4:0.66 E4:0.34 C4:0.66 E4:0.34 G4:1"]] },
    { k: "mentor", t: "Cada estilo tem seu **lugar do bumbo**. Antes de tocar, ouça onde ele cai: o resto da banda se encaixa ali." }
  ],
  q: [
    { q: "Síncope é:", o: ["Um som que começa no tempo fraco e se prolonga sobre o forte", "Tocar mais rápido", "Uma pausa longa", "Um acorde menor"], c: 0, e: "O acento é deslocado." },
    { q: "Na levada básica de 4/4, a caixa toca nos tempos:", o: ["2 e 4", "1 e 3", "Só no 1", "Em todas as colcheias"], c: 0, e: "O backbeat." },
    { q: "A valsa é em:", o: ["3/4", "4/4", "6/8", "2/4"], c: 0, e: "Forte-fraco-fraco." },
    { q: "No 6/8, sentimos normalmente:", o: ["2 pulsos de 3 colcheias", "6 pulsos iguais", "3 pulsos de 2", "4 pulsos"], c: 0, e: "Binário composto." },
    { q: "O tresillo aparece em:", o: ["Baião e reggaeton", "Só na música erudita", "Só no rock dos anos 50", "Canto gregoriano"], c: 0, e: "E em muito pop atual." },
    { q: "No swing, as colcheias:", o: ["Ficam desiguais (longa-curta)", "Ficam todas iguais", "Viram semicolcheias", "Desaparecem"], c: 0, e: "“Dum-da dum-da”." }
  ]
};
LESSONS.r3 = {
  title: "Técnica e estudo inteligente", sub: "Prática deliberada, metrônomo, intercalar, descansar e cuidar do corpo e da voz",
  goals: ["Estudar um trecho difícil do jeito certo", "Organizar uma sessão de estudo", "Prevenir lesões e cuidar da voz"],
  mis: "Escolha um trecho de 2 compassos que você erra. Use o método: lento e perfeito com metrônomo, 3 vezes seguidas sem erro, sobe 4 BPM. Anote o BPM inicial e o final.",
  deep: ["“Prática deliberada” é o conceito do psicólogo K. Anders Ericsson: metas específicas, concentração, feedback imediato e dificuldade calibrada.", "Dormir depois de estudar ajuda a consolidar habilidades motoras: o cérebro “reorganiza” o que você praticou."],
  s: [
    { k: "txt", h: "Tocar ≠ estudar", p: "Tocar o que você já sabe é bom, mas **não é estudo**. Estudar é trabalhar exatamente o que você ainda não consegue, com foco total e um objetivo claro (“tocar estes 2 compassos limpos a 90 BPM”)." },
    { k: "list", h: "Prática deliberada", items: ["**Meta específica** e mensurável", "**Concentração total** (celular longe)", "**Feedback**: grave-se, use metrônomo, peça a um professor", "Dificuldade **um pouco acima** do confortável", "**Repetição correta**: errar repetido grava o erro"] },
    { k: "order", h: "O caminho para um trecho difícil", items: ["Isolar o trecho (1 ou 2 compassos)", "Tocar bem devagar, sem erro, com metrônomo", "Repetir 3 vezes seguidas perfeito", "Subir 4 BPM e repetir", "Ligar o trecho ao que vem antes e depois"], e: "Lento e certo vira rápido e certo. Rápido e errado vira hábito errado." },
    { k: "tf", q: "Repetir rápido, mesmo errando, até sair é o jeito mais eficiente de estudar.", a: false, e: "Você grava o erro. Desacelere até ficar perfeito." },
    { k: "txt", h: "Intercalar e espaçar", p: "Treinar só uma coisa por uma hora (prática em bloco) dá sensação de progresso. **Alternar** exercícios (escalas → trecho → leitura → percepção) rende menos na hora, mas **fixa muito mais** a longo prazo. E sessões curtas em dias diferentes vencem uma maratona." },
    { k: "table", h: "Uma sessão de 30 minutos", head: ["Min.", "O quê"], rows: [["0–5", "Aquecimento: alongamento leve, escalas lentas"], ["5–15", "Trecho difícil com metrônomo (método acima)"], ["15–20", "Leitura à primeira vista ou técnica"], ["20–25", "Repertório: tocar com expressão, do começo ao fim"], ["25–30", "Percepção: 1 rodada na Academia"]] },
    { k: "txt", h: "Corpo: postura e pausas", p: "Ombros soltos, punhos alinhados, respiração livre. Faça **pausas a cada 25–30 minutos**. **Dor não é sinal de esforço bom**: pare. Tendinite e LER afastam muitos músicos por meses." },
    { k: "txt", h: "Voz: o instrumento que mora em você", p: "• **Respiração** costodiafragmática: expanda as costelas e a barriga, sem subir os ombros.\n• **Aqueça**: vibração de lábios (“brrr”), humming, sirenes suaves.\n• **Hidrate-se** e evite gritar no ensaio.\n• Rouquidão por mais de 2 semanas: procure um otorrino." },
    { k: "chat", who: "Pedro, tecladista", c: "#2B8A3E", q: "Treino 3 horas seguidas no sábado e na semana não encosto no teclado. Tá certo?", o: [
      { t: "Melhor dividir: 20–30 minutos quase todo dia rendem mais que 3 horas uma vez. E com metas claras para cada sessão.", v: 2, r: "Vou tentar 25 min por dia com o metrônomo do app.", tip: "Perfeito: prática espaçada + metas." },
      { t: "Tá ótimo, quanto mais horas seguidas melhor.", v: 0, r: "É, mas no domingo esqueci metade…", tip: "Sem espaçamento, o cérebro consolida menos." },
      { t: "Treina só no domingo, na hora do culto.", v: 0, r: "Aí vou errar na frente de todo mundo 😅", tip: "O palco não é lugar de estudar." }] },
    { k: "mentor", t: "Antes de tocar o tamborim diante do povo, eu tinha tocado muitas vezes sozinha. **Constância vence talento** que não treina." }
  ],
  q: [
    { q: "Qual o princípio central da prática deliberada?", o: ["Meta específica, foco e feedback", "Tocar só o que você já sabe", "Estudar sempre na velocidade final", "Evitar o metrônomo"], c: 0, e: "E dificuldade calibrada." },
    { q: "Num trecho difícil, depois de tocar 3 vezes seguidas perfeito, você deve:", o: ["Subir alguns BPM", "Dobrar a velocidade", "Parar de estudar o trecho para sempre", "Tocar sem metrônomo e mais rápido"], c: 0, e: "Aumentos pequenos e constantes." },
    { q: "A prática intercalada:", o: ["Fixa mais a longo prazo", "É sempre pior", "Só serve para iniciantes", "Substitui o descanso"], c: 0, e: "Rende menos na hora e mais depois." },
    { q: "Sentiu dor no punho tocando. O certo é:", o: ["Parar e descansar", "Continuar até passar", "Tocar mais forte", "Tomar café e continuar"], c: 0, e: "Dor é alerta." },
    { q: "Na respiração para cantar, os ombros devem:", o: ["Ficar soltos, sem subir", "Subir bem alto", "Ficar travados", "Ir para frente"], c: 0, e: "A expansão é nas costelas e no abdômen." },
    { q: "Qual é melhor para aprender?", o: ["25 minutos quase todos os dias", "3 horas uma vez por semana", "Só nos ensaios", "1 dia por mês, 8 horas"], c: 0, e: "Espaçar é mais eficiente." }
  ]
};
LESSONS.r4 = {
  title: "Tocar em grupo e interpretar", sub: "Papéis na banda, dinâmica, articulação, fraseado e regência básica",
  goals: ["Saber o papel de cada instrumento", "Usar dinâmica e articulação com intenção", "Entender os gestos básicos de regência"],
  mis: "No próximo ensaio, combine com a banda um **mapa de dinâmica** do louvor (verso p, pré mf, refrão f, ponte crescendo, final p). Toque assim e conversem depois.",
  deep: ["Os padrões de regência mais comuns: 2/4 (baixo–cima), 3/4 (baixo–direita–cima) e 4/4 (baixo–esquerda–direita–cima).", "Na banda, “ouvir mais do que tocar” é literal: se você não ouve o cantor, provavelmente está alto demais."],
  s: [
    { k: "txt", h: "O grupo é um instrumento só", p: "Tocar bem sozinho não basta. Em grupo, o segredo é **ouvir mais do que tocar**, saber seu papel e deixar espaço. Música boa em grupo é como conversa boa: ninguém fala por cima." },
    { k: "table", h: "Quem faz o quê na banda", head: ["Instrumento", "Papel principal"], rows: [["Bateria", "Tempo e energia"], ["Baixo", "Fundação: liga ritmo e harmonia"], ["Violão", "Ritmo harmônico (a levada dos acordes)"], ["Teclado", "Colchão harmônico, pads, cor"], ["Guitarra", "Texturas, riffs, melodias"], ["Vozes", "A mensagem: tudo existe para servi-las"]] },
    { k: "txt", h: "Espaço de frequências", p: "Se todos tocam na mesma região, vira uma massa. Exemplo: o **teclado não dobra o baixo** com a mão esquerda; violão e teclado não fazem o mesmo acorde na mesma altura. Cada um no seu “andar” do prédio." },
    { k: "txt", h: "Dinâmica", p: "**pp** (pianíssimo), **p** (piano), **mp**, **mf**, **f** (forte), **ff** (fortíssimo). **Crescendo** (<) aumenta, **decrescendo** (>) diminui. A emoção de uma música vem mais da dinâmica do que de tocar alto o tempo todo." },
    { k: "play", h: "Do pp ao ff", b: [["Mesma nota, 4 dinâmicas", "fx:dyn"]] },
    { k: "txt", h: "Articulação", p: "**Legato**: notas ligadas. **Staccato**: curtas e destacadas. **Acento** (>): mais forte no ataque. **Tenuto** (–): valor cheio, com peso." },
    { k: "play", h: "Legato × staccato", b: [["Legato", "bpm=100 C4 D4 E4 F4 G4:2"], ["Staccato", "bpm=100 C4:0.3 -:0.7 D4:0.3 -:0.7 E4:0.3 -:0.7 F4:0.3 -:0.7 G4:0.3"]] },
    { k: "txt", h: "Fraseado e rubato", p: "Frase musical é como frase falada: tem começo, ponto alto e fim. **Respire** com a frase. **Rubato** é a liberdade de “roubar” um pouco de tempo e devolver, para expressar. Use com gosto, não o tempo todo." },
    { k: "match", h: "Ligue o termo ao significado", pairs: [["Piano (p)", "Suave"], ["Crescendo", "Aumentando a intensidade"], ["Ritardando", "Desacelerando"], ["Legato", "Ligado"], ["Fermata", "Sustentar à vontade"]] },
    { k: "txt", h: "Regência básica", p: "O regente marca o tempo com padrões: **2/4** baixo–cima; **3/4** baixo–direita–cima; **4/4** baixo–esquerda–direita–cima. O gesto de **preparação** (antes do 1) diz ao grupo o andamento e a respiração. Em louvor, o líder usa sinais combinados: repetir, ponte, final." },
    { k: "chat", who: "Ana, ministra de louvor", c: "#9C36B5", q: "Na hora de repetir o refrão, ninguém entende meus sinais 😩 Ideia?", o: [
      { t: "Combina no ensaio sinais fixos (ex.: mão girando = repete, punho fechado = final) e faz o gesto um compasso antes, olhando para a banda.", v: 2, r: "Um compasso antes! É isso que faltava.", tip: "Sinal claro, combinado e com antecedência." },
      { t: "Grita no microfone “repete!”.", v: 0, r: "A igreja inteira ouviu 😂", tip: "Funciona, mas quebra o momento." },
      { t: "Deixa rolar, uma hora eles adivinham.", v: 0, r: "Ontem cada um foi pra uma parte…", tip: "Grupo precisa de comunicação clara." }] },
    { k: "mentor", t: "Quando eu cantei “Cantai ao Senhor”, as mulheres responderam (Êx 15.21). **Música em grupo é pergunta e resposta.** Ouça, responda, conduza." }
  ],
  q: [
    { q: "Na banda, quem liga o ritmo à harmonia como “fundação”?", o: ["Baixo", "Guitarra solo", "Backing vocal", "Teclado com pad"], c: 0, e: "O baixo." },
    { q: "“mf” significa:", o: ["Mezzo forte (meio forte)", "Muito forte", "Muito fraco", "Mais firme"], c: 0, e: "Entre piano e forte." },
    { q: "Staccato é:", o: ["Notas curtas e destacadas", "Notas ligadas", "Tocar mais alto", "Tocar mais devagar"], c: 0, e: "Destacado." },
    { q: "O padrão de regência em 3/4 é:", o: ["Baixo – direita – cima", "Baixo – cima", "Baixo – esquerda – direita – cima", "Só para baixo"], c: 0, e: "Três gestos." },
    { q: "Por que o teclado evita dobrar o baixo com a mão esquerda?", o: ["Para não embolar a região grave", "Porque é proibido", "Para descansar a mão", "Porque o baixo é mais alto"], c: 0, e: "Cada um no seu espaço de frequência." },
    { q: "Ritardando significa:", o: ["Desacelerar aos poucos", "Acelerar", "Parar de repente", "Repetir"], c: 0, e: "rit." }
  ]
};

LESSONS.h1 = {
  title: "Acordes, inversões e cifras", sub: "Como montar tríades, ler qualquer cifra e usar o baixo invertido",
  goals: ["Montar as 4 tríades a partir de qualquer nota", "Ler cifras com 7, 7M, °, ø, sus, add e barra", "Usar inversões para um baixo que caminha"],
  mis: "Pegue a cifra de um louvor e, para cada acorde, diga as notas (ex.: G = Sol Si Ré). Se houver acorde com barra (C/E), identifique a nota do baixo.",
  deep: ["Cifra brasileira × americana: 7M = maj7 = Δ; m7(♭5) = ø; ° = dim.", "Em sus4, a 3ª é substituída pela 4ª: o acorde fica “suspenso”, nem maior nem menor."],
  s: [
    { k: "mentor", t: "Eu sou **Asafe**. Davi me colocou como chefe dos cantores diante da arca (1 Cr 16.5), e doze salmos levam meu nome. **Harmonia é a arte de fazer as notas conversarem.**" },
    { k: "txt", h: "Empilhando terças", p: "Uma **tríade** nasce de uma nota (a **fundamental**) mais a **3ª** e a **5ª** acima dela. Em notas da escala, é pular uma: Dó-(ré)-**Mi**-(fá)-**Sol**." },
    { k: "table", h: "As 4 tríades (toque nas linhas)", head: ["Tipo", "Terças", "Exemplo"], rows: [["Maior", "3M + 3m", "Dó Mi Sol (C)"], ["Menor", "3m + 3M", "Dó Mi♭ Sol (Cm)"], ["Diminuta", "3m + 3m", "Dó Mi♭ Sol♭ (C°)"], ["Aumentada", "3M + 3M", "Dó Mi Sol♯ (C+)"]], play: ["[C4 E4 G4]:2", "[C4 Eb4 G4]:2", "[C4 Eb4 Gb4]:2", "[C4 E4 G#4]:2"] },
    { k: "kb", h: "Monte Ré maior", p: "Fundamental Ré, 3ª maior (4 semitons acima) e 5ª justa (7 semitons).", r: [50, 74], t: ["D", "F#", "A"], e: "Ré – Fá♯ – Lá." },
    { k: "kb", h: "Agora Mi menor", r: [52, 76], t: ["E", "G", "B"], e: "Mi – Sol – Si." },
    { k: "table", h: "Lendo cifras", head: ["Cifra", "Significa", "Em Dó"], rows: [["C", "Maior", "Dó Mi Sol"], ["Cm", "Menor", "Dó Mi♭ Sol"], ["C7", "Maior + 7ª menor", "Dó Mi Sol Si♭"], ["C7M", "Maior + 7ª maior", "Dó Mi Sol Si"], ["Cm7", "Menor + 7ª menor", "Dó Mi♭ Sol Si♭"], ["C°", "Diminuto", "Dó Mi♭ Sol♭"], ["Cm7(♭5) ou Cø", "Meio-diminuto", "Dó Mi♭ Sol♭ Si♭"], ["Csus4", "4ª no lugar da 3ª", "Dó Fá Sol"], ["Cadd9", "Maior + 9ª", "Dó Mi Sol Ré"], ["C/E", "Dó com Mi no baixo", "baixo Mi"]],
      play: ["[C4 E4 G4]:2", "[C4 Eb4 G4]:2", "[C4 E4 G4 Bb4]:2", "[C4 E4 G4 B4]:2", "[C4 Eb4 G4 Bb4]:2", "[C4 Eb4 Gb4]:2", "[C4 Eb4 Gb4 Bb4]:2", "[C4 F4 G4]:2", "[C4 E4 G4 D5]:2", "[E3 C4 G4]:2"] },
    { k: "txt", h: "Inversões", p: "O acorde é o mesmo, muda a nota do **baixo**:\n• **Estado fundamental**: fundamental no baixo (C).\n• **1ª inversão**: 3ª no baixo (C/E).\n• **2ª inversão**: 5ª no baixo (C/G)." },
    { k: "play", h: "Ouça as três posições", b: [["C", "[C3 C4 E4 G4]:3"], ["C/E", "[E3 C4 E4 G4]:3"], ["C/G", "[G2 C4 E4 G4]:3"]] },
    { k: "txt", h: "Por que inverter?", p: "Para o baixo **caminhar** em graus vizinhos em vez de saltar. Ouça: **C – C/B – Am – Am/G – F**. O baixo desce Dó, Si, Lá, Sol, Fá, como uma melodia." },
    { k: "play", h: "Baixo que caminha", b: [["C – C/B – Am – Am/G – F", "bpm=80 [C3 G3 C4 E4]:2 [B2 G3 C4 E4]:2 [A2 A3 C4 E4]:2 [G2 A3 C4 E4]:2 [F2 A3 C4 F4]:4"]] },
    { k: "pick", q: "No acorde G/B, qual nota fica no baixo?", o: ["Si", "Sol", "Ré", "Lá"], c: 0, e: "A nota depois da barra é o baixo: Si (a 3ª de Sol)." },
    { k: "tf", q: "Em Csus4, a nota Mi é trocada por Fá.", a: true, e: "Sus4: a 4ª (Fá) substitui a 3ª (Mi)." },
    { k: "mentor", t: "Com terças empilhadas, você monta **qualquer** acorde. Treine no **Laboratório → Teclado** escolhendo acordes de todas as tônicas." }
  ],
  q: [
    { q: "Quais são as notas de Em?", o: ["Mi Sol Si", "Mi Sol♯ Si", "Mi Lá Si", "Mi Sol Si♭"], c: 0, e: "Mi + 3ª menor + 5ª justa." },
    { q: "Uma tríade diminuta tem:", o: ["Duas terças menores", "Duas terças maiores", "3ª maior e 3ª menor", "Uma 4ª e uma 5ª"], c: 0, e: "Ex.: Si Ré Fá." },
    { q: "C7M tem as notas:", o: ["Dó Mi Sol Si", "Dó Mi Sol Si♭", "Dó Mi♭ Sol Si♭", "Dó Mi Sol Lá"], c: 0, e: "7ª maior = Si." },
    { q: "C/E é:", o: ["Dó maior com Mi no baixo", "Dó menor", "Mi maior", "Dó com 7ª"], c: 0, e: "1ª inversão." },
    { q: "Na 2ª inversão, o baixo é:", o: ["A 5ª do acorde", "A 3ª", "A fundamental", "A 7ª"], c: 0, e: "C/G." },
    { q: "Ouça: que acorde?", play: "[G3 B3 D4 F4]:3", o: ["G7 (dominante)", "G7M", "Gm7", "G°"], c: 0, e: "Sol Si Ré Fá." },
    { q: "Quais as notas de F (Fá maior)?", o: ["Fá Lá Dó", "Fá Lá♭ Dó", "Fá Si Dó", "Fá Sol Dó"], c: 0, e: "3ª maior: Lá." }
  ]
};
LESSONS.h2 = {
  title: "Campo harmônico e funções", sub: "Os 7 acordes de cada tom, o padrão que nunca muda e os números de Nashville",
  goals: ["Montar o campo harmônico de qualquer tom maior", "Classificar acordes por função", "Transpor pelos números (graus)"],
  mis: "Escreva o campo harmônico de Sol, Ré e Lá maior. Depois pegue um louvor e reescreva a cifra em números (I, IV, V, vi…). Transponha para outro tom.",
  deep: ["Na menor natural: i – ii° – III – iv – v – VI – VII. Na prática, o V vira maior (V7) usando a 7ª elevada da menor harmônica.", "O “Nashville Number System” (números no lugar de cifras) é usado em estúdios para trocar de tom na hora."],
  s: [
    { k: "txt", h: "O que é campo harmônico", p: "Pegue a escala do tom e monte uma tríade **em cada grau**, usando só notas da escala. O resultado são os 7 acordes “da casa”. É por isso que certos acordes aparecem juntos o tempo todo." },
    { k: "table", h: "Campo harmônico de Dó maior", p: "Toque nas linhas para ouvir.", head: ["Grau", "Acorde", "Notas"], rows: [["I", "C", "Dó Mi Sol"], ["ii", "Dm", "Ré Fá Lá"], ["iii", "Em", "Mi Sol Si"], ["IV", "F", "Fá Lá Dó"], ["V", "G", "Sol Si Ré"], ["vi", "Am", "Lá Dó Mi"], ["vii°", "B°", "Si Ré Fá"]],
      play: ["[C3 C4 E4 G4]:2", "[D3 D4 F4 A4]:2", "[E3 E4 G4 B4]:2", "[F2 F4 A4 C5]:2", "[G2 G4 B4 D5]:2", "[A2 A4 C5 E5]:2", "[B2 B4 D5 F5]:2"] },
    { k: "big", n: "M m m M M m °", p: "O padrão de qualidades do campo maior. **Vale para qualquer tom maior.** Graus maiúsculos (I, IV, V) são maiores; minúsculos (ii, iii, vi) são menores; vii° é diminuto." },
    { k: "order", h: "Monte o campo de Sol maior (I ao vii°)", items: ["G", "Am", "Bm", "C", "D", "Em", "F♯°"], e: "Sol maior tem Fá♯, por isso o vii° é F♯°." },
    { k: "table", h: "Funções no campo", head: ["Função", "Graus", "Sensação"], rows: [["Tônica (T)", "I, vi, iii", "Repouso"], ["Subdominante (SD)", "IV, ii", "Afastamento"], ["Dominante (D)", "V, vii°", "Tensão que volta ao I"]] },
    { k: "txt", h: "Substitutos", p: "Acordes da mesma função podem trocar de lugar: no lugar do **I**, um **vi** (relativo); no lugar do **IV**, um **ii**. Ex.: em vez de C – F – G – C, toque **C – Dm – G – C**. Mesma direção, outra cor." },
    { k: "play", h: "IV ou ii antes do V?", b: [["C – F – G – C", "bpm=84 [C3 C4 E4 G4]:2 [F2 C4 F4 A4]:2 [G2 B3 D4 G4]:2 [C3 C4 E4 G4]:3"], ["C – Dm – G – C", "bpm=84 [C3 C4 E4 G4]:2 [D3 D4 F4 A4]:2 [G2 B3 D4 G4]:2 [C3 C4 E4 G4]:3"]] },
    { k: "txt", h: "Tétrades no campo", p: "Empilhando mais uma terça: **I7M – ii7 – iii7 – IV7M – V7 – vi7 – viiø**. O único **7 (dominante)** natural é o **V7**: por isso ele tem tanta força." },
    { k: "pick", q: "Qual é o vi de Ré maior?", o: ["Bm", "Em", "F♯m", "G"], c: 0, e: "Ré, Mi, Fá♯, Sol, Lá, **Si**: Si menor." },
    { k: "txt", h: "Pensar em números", p: "Se a música é **I – V – vi – IV**, em Dó fica C – G – Am – F; em Ré, D – A – Bm – G; em Sol, G – D – Em – C. Quem pensa em números **transpõe na hora**, quando o cantor pede outro tom." },
    { k: "chat", who: "Rafa, vocalista", c: "#E8590C", q: "Gente, baixa um tom! Tava em Ré, pode ser Dó? 🙏", o: [
      { t: "Bora! I–V–vi–IV em Dó fica C–G–Am–F. Todo mundo pensa em números e segue.", v: 2, r: "Ufa, ficou ótimo pra minha voz!", tip: "Números = transposição instantânea." },
      { t: "Não dá, a cifra tá escrita em Ré.", v: 0, r: "Aí vou desafinar no refrão 😬", tip: "Com o campo harmônico, a cifra deixa de ser prisão." },
      { t: "Só se usar capotraste no teclado.", v: 1, r: "Teclado tem capotraste?? 😂", tip: "O teclado tem botão de transposição, mas o ideal é saber pelos graus." }] },
    { k: "mentor", t: "O campo harmônico é o **mapa da casa**. Explore no **Laboratório → Campo harmônico** e ouça cada tom." }
  ],
  q: [
    { q: "O padrão de qualidades do campo maior é:", o: ["M m m M M m °", "m M M m m M °", "M M m m M m °", "M m M m M m M"], c: 0, e: "I ii iii IV V vi vii°." },
    { q: "Em Sol maior, o acorde do V grau é:", o: ["D", "C", "Em", "Am"], c: 0, e: "Sol, Lá, Si, Dó, **Ré**." },
    { q: "Qual grau tem função de subdominante?", o: ["IV", "V", "I", "vii°"], c: 0, e: "IV (e ii)." },
    { q: "Em Fá maior, o ii é:", o: ["Gm", "Dm", "Am", "B♭"], c: 0, e: "Fá, **Sol**: Sol menor." },
    { q: "A única tétrade 7 (dominante) natural do campo maior está no grau:", o: ["V", "I", "IV", "ii"], c: 0, e: "V7." },
    { q: "I – V – vi – IV em Lá maior é:", o: ["A – E – F♯m – D", "A – D – E – A", "A – E – Bm – D", "A – C♯m – D – E"], c: 0, e: "Lá maior: A B C♯ D E F♯ G♯." },
    { q: "Ouça: este acorde é o vii° de Dó. Que tipo ele é?", play: "[B3 D4 F4]:3", o: ["Diminuto", "Maior", "Menor", "Aumentado"], c: 0, e: "Si Ré Fá." }
  ]
};
LESSONS.h3 = {
  title: "Tensão e resolução", sub: "O trítono do V7, cadências, dominantes secundárias e empréstimo modal",
  goals: ["Explicar por que o V7 resolve no I", "Reconhecer as 4 cadências", "Usar dominantes secundárias e empréstimos modais"],
  mis: "Escolha um louvor em tom maior e acrescente **uma dominante secundária** antes do vi (ex.: em Dó, E7 → Am). Toque para alguém e pergunte o que sentiu.",
  deep: ["Dominante secundária se escreve V7/x (“quinto do x”). Em Dó: D7 = V7/V, A7 = V7/ii, E7 = V7/vi, B7 = V7/iii.", "Acordes emprestados mais comuns no maior: iv (Fm em Dó), ♭VII (B♭), ♭VI (A♭) e ♭III (E♭)."],
  s: [
    { k: "txt", h: "Música é respiração", p: "Toda música alterna **tensão** e **repouso**, como inspirar e expirar. A harmonia tonal é uma máquina de criar expectativa e resolver." },
    { k: "txt", h: "O motor do V7", p: "Em **G7** (Sol Si Ré Fá) existe um **trítono**: **Si–Fá**. Ele é instável e resolve por semitom: **Si sobe para Dó** e **Fá desce para Mi**. Resultado: o acorde de **Dó**. É a resolução mais forte da música tonal." },
    { k: "play", h: "Ouça o trítono resolver", b: [["G7 → C", "bpm=70 [G2 B3 D4 F4]:2 [C3 C4 E4 G4]:3"], ["Só o trítono", "bpm=70 [B3 F4]:2 [C4 E4]:3"]], kb: [43, 67] },
    { k: "table", h: "As 4 cadências (toque para ouvir)", head: ["Cadência", "Movimento", "Sensação"], rows: [["Autêntica (perfeita)", "V → I", "Ponto final"], ["Plagal", "IV → I", "O “Amém” dos hinos"], ["Meia cadência", "… → V", "Vírgula, pergunta"], ["Interrompida (de engano)", "V → vi", "Surpresa"]],
      play: ["bpm=72 [G2 B3 D4 F4]:2 [C3 C4 E4 G4]:3", "bpm=72 [F2 C4 F4 A4]:2 [C3 C4 E4 G4]:3", "bpm=72 [C3 C4 E4 G4]:2 [G2 B3 D4 G4]:3", "bpm=72 [G2 B3 D4 F4]:2 [A2 A3 C4 E4]:3"] },
    { k: "ear", q: "Que cadência foi essa?", play: "bpm=72 [C3 C4 E4 G4]:2 [F2 C4 F4 A4]:2 [G2 B3 D4 F4]:2 [A2 A3 C4 E4]:3", o: ["Interrompida (V → vi)", "Autêntica (V → I)", "Plagal (IV → I)"], c: 0, e: "O ouvido esperava Dó e veio Lá menor." },
    { k: "txt", h: "A sensível", p: "O 7º grau (Si em Dó) está a meio tom da tônica: ele **puxa** para cima. Na menor natural não há sensível (o 7º está a um tom). Por isso o tom menor “pega emprestado” a 7ª elevada (menor harmônica) para ter um V maior." },
    { k: "txt", h: "Dominantes secundárias", p: "Você pode dar uma dominante para **qualquer** acorde do campo. Em Dó, para chegar no **Am** com força, toque antes o **E7** (o V7 de Lá menor). Para ir ao **Dm**, toque **A7**. É o famoso “acorde de passagem” que dá brilho." },
    { k: "play", h: "Com e sem dominante secundária", b: [["C → Am", "bpm=76 [C3 C4 E4 G4]:2 [A2 A3 C4 E4]:3"], ["C → E7 → Am", "bpm=76 [C3 C4 E4 G4]:2 [E2 B3 D4 G#4]:2 [A2 A3 C4 E4]:3"]] },
    { k: "pick", q: "Em Dó maior, qual é a dominante secundária que leva ao Dm?", o: ["A7", "G7", "D7", "E7"], c: 0, e: "Dm é o ii; o V7 de Ré é Lá7." },
    { k: "txt", h: "Empréstimo modal", p: "Pegar um acorde do **tom menor homônimo**. Em Dó maior, usar o **Fm** (iv menor) no lugar do F dá uma “saudade” linda. Também é comum o **B♭** (♭VII), som de rock e de louvor épico." },
    { k: "play", h: "O iv menor", b: [["C – F – Fm – C", "bpm=70 [C3 E4 G4 C5]:2 [F2 F4 A4 C5]:2 [F2 F4 Ab4 C5]:2 [C3 E4 G4 C5]:4"]] },
    { k: "tf", q: "A cadência plagal (IV → I) é a do “Amém” no fim dos hinos.", a: true, e: "Ouça um hino tradicional: o “A-mém” costuma ser IV → I." },
    { k: "mentor", t: "Os salmos de lamento terminam em louvor: **tensão que resolve**. Use a dissonância para levar o ouvinte a algum lugar." }
  ],
  q: [
    { q: "Em G7, qual par de notas forma o trítono?", o: ["Si e Fá", "Sol e Ré", "Sol e Si", "Ré e Fá"], c: 0, e: "6 semitons entre Si e Fá." },
    { q: "A cadência autêntica é:", o: ["V → I", "IV → I", "V → vi", "I → V"], c: 0, e: "A mais conclusiva." },
    { q: "V → vi chama-se:", o: ["Cadência interrompida", "Cadência plagal", "Meia cadência", "Cadência autêntica"], c: 0, e: "Ou “de engano”." },
    { q: "Em Dó maior, E7 é a dominante secundária de:", o: ["Am", "Em", "F", "Dm"], c: 0, e: "Mi é o V de Lá." },
    { q: "O iv menor em Dó maior é:", o: ["Fm", "F", "Dm", "A♭"], c: 0, e: "Emprestado de Dó menor." },
    { q: "Ouça: que cadência?", play: "bpm=72 [F2 C4 F4 A4]:2 [C3 C4 E4 G4]:3", o: ["Plagal", "Autêntica", "Interrompida", "Meia cadência"], c: 0, e: "IV → I: “A-mém”." },
    { q: "Por que a menor harmônica existe?", o: ["Para ter sensível e um V maior", "Para ter menos notas", "Para soar mais alegre", "Para tirar o trítono"], c: 0, e: "7ª elevada = sensível." }
  ]
};
LESSONS.h4 = {
  title: "Tensões e rearmonização", sub: "9, 11 e 13; notas-guia; subV, diminuto de passagem e quando NÃO rearmonizar",
  goals: ["Entender de onde vêm as tensões 9, 11 e 13", "Tocar o ii–V–I só com notas-guia", "Rearmonizar com substitutos sem atrapalhar a melodia"],
  mis: "Pegue um louvor simples (I–IV–V) e faça uma versão com: um ii–V antes do IV, um 7M no I e um diminuto de passagem. Toque as duas versões e compare.",
  deep: ["Sobre acordes maiores, a 11 justa costuma ser evitada (choca com a 3ª); usa-se a ♯11. Sobre dominantes: 9, ♭9, ♯9, ♯11, 13, ♭13 são todas possíveis conforme o estilo.", "Substituição de trítono funciona porque G7 e D♭7 compartilham o mesmo trítono (Si/Dó♭ e Fá)."],
  s: [
    { k: "txt", h: "Além das tríades", p: "Continuando a empilhar terças depois da 7ª chegamos à **9**, **11** e **13** (= 2ª, 4ª e 6ª uma oitava acima). Elas são as **tensões**: cor, não função." },
    { k: "play", h: "Do simples ao rico", b: [["C", "[C3 E4 G4 C5]:3"], ["C7M", "[C3 E4 G4 B4]:3"], ["C7M(9)", "[C3 E4 G4 B4 D5]:3"], ["Am7(11)", "[A2 G3 C4 D4 E4]:3"], ["G7(13)", "[G2 F3 B3 E4]:3"]] },
    { k: "txt", h: "Notas-guia", p: "A **3ª** e a **7ª** definem a qualidade do acorde (a 5ª quase não importa). Num **ii–V–I**, as notas-guia andam por semitom ou ficam paradas: é a condução mais suave possível." },
    { k: "play", h: "ii – V – I só com baixo + 3ª + 7ª", p: "Dm7 – G7 – C7M em Dó: ouça como as vozes mal se mexem.", b: [["Notas-guia", "bpm=72 [D3 F4 C5]:2 [G2 F4 B4]:2 [C3 E4 B4]:4"], ["Completo", "bpm=72 [D3 F4 A4 C5]:2 [G2 F4 B4 D5]:2 [C3 E4 G4 B4]:4"]] },
    { k: "txt", h: "Tensões disponíveis", p: "Sobre acorde **maior**: 9, ♯11, 13 (a 11 justa costuma ser evitada, choca com a 3ª).\nSobre **menor**: 9, 11, 13 com cuidado.\nSobre **dominante**: 9, ♭9, ♯9, ♯11, 13, ♭13: o dominante aceita quase tudo, porque já é tenso." },
    { k: "txt", h: "Rearmonizar", p: "É trocar os acordes **mantendo a melodia**. Ferramentas:\n• **Relativo**: vi no lugar do I.\n• **ii–V** antes de um acorde-alvo.\n• **SubV**: trocar G7 por **D♭7** (mesmo trítono).\n• **Diminuto de passagem**: C – **C♯°** – Dm." },
    { k: "play", h: "Diminuto de passagem e subV", b: [["C – C♯°7 – Dm7 – G7 – C", "bpm=76 [C3 E4 G4 C5]:2 [C#3 E4 G4 Bb4]:2 [D3 F4 A4 C5]:2 [G2 F4 B4 D5]:2 [C3 E4 G4 C5]:3"], ["Dm7 – D♭7 – C7M (subV)", "bpm=72 [D3 F4 A4 C5]:2 [Db3 F4 B4 Eb5]:2 [C3 E4 G4 B4]:4"]] },
    { k: "pick", q: "Qual é o subV (substituto de trítono) de G7?", o: ["D♭7", "D7", "C♯m7", "A♭7"], c: 0, e: "D♭ está a um trítono de Sol; D♭7 e G7 têm o mesmo trítono (Fá e Dó♭/Si)." },
    { k: "txt", h: "Quando NÃO rearmonizar", p: "No louvor congregacional, o povo precisa **cantar junto**. Rearmonização que choca com a melodia ou esconde o tom atrapalha. Use nas introduções, interlúdios, finais e momentos instrumentais. **A harmonia serve à canção.**" },
    { k: "chat", who: "Thiago, tecladista", c: "#0B7285", q: "Coloquei um monte de acorde de jazz no hino e o pessoal se perdeu na hora de cantar 🤔", o: [
      { t: "Deixa a harmonia simples onde o povo canta e guarda as tensões para a intro, as passagens e o final. Testa se a melodia continua clara.", v: 2, r: "Faz sentido. Na intro fica lindo, no refrão simplifico.", tip: "Equilíbrio: cor sem tirar o chão da congregação." },
      { t: "O problema é deles, precisam acostumar.", v: 0, r: "Hmm, mas a música é pra eles cantarem…", tip: "A harmonia serve à canção e ao povo." },
      { t: "Nunca use tensão em hino.", v: 1, r: "Mas um 7M no final fica tão bonito…", tip: "Não é proibido: é questão de lugar e dose." }] },
    { k: "mentor", t: "No meu salmo 73 eu quase escorreguei, até entrar no santuário. **Harmonia rica é bonita, mas o fundamento vem primeiro.**" }
  ],
  q: [
    { q: "A 9ª de um acorde é a mesma nota que a:", o: ["2ª uma oitava acima", "7ª", "4ª", "3ª"], c: 0, e: "9 = 2 + 7." },
    { q: "Quais notas definem a qualidade de uma tétrade?", o: ["3ª e 7ª", "Fundamental e 5ª", "5ª e 9ª", "Só a fundamental"], c: 0, e: "As notas-guia." },
    { q: "O subV de G7 é:", o: ["D♭7", "D7", "F7", "C7"], c: 0, e: "A um trítono de distância." },
    { q: "Sobre um acorde maior, qual tensão costuma ser evitada?", o: ["11 justa", "9", "13", "7M"], c: 0, e: "Ela choca com a 3ª; usa-se ♯11." },
    { q: "C – C♯° – Dm usa:", o: ["Diminuto de passagem", "Empréstimo modal", "SubV", "Cadência plagal"], c: 0, e: "O C♯° liga C a Dm cromaticamente." },
    { q: "No louvor congregacional, rearmonizações devem:", o: ["Servir à melodia e não confundir o povo", "Ser usadas em todo verso", "Mudar o tom a cada refrão", "Evitar qualquer acorde com 7ª"], c: 0, e: "A harmonia serve à canção." }
  ]
};
