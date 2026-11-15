// Lições — Nível 5 (Composição, Salomão), Nível 6 (História, Jubal) e Nível 7 (Produção, Bezalel).
"use strict";
LESSONS.c1 = {
  title: "Melodia: motivo, frase e prosódia", sub: "Como nasce uma melodia que gruda e respeita a letra",
  goals: ["Criar e variar um motivo", "Escrever frases de pergunta e resposta", "Casar as sílabas tônicas com os tempos fortes"],
  mis: "Componha uma melodia de 8 compassos para um versículo curto (ex.: Sl 23.1). Use um motivo de 3 ou 4 notas, uma pergunta (termina no 2 ou no 5) e uma resposta (termina no 1). Grave no celular.",
  deep: ["Regra clássica do contorno: depois de um salto grande, volte por grau conjunto na direção oposta (“recuperação do salto”).", "As quatro primeiras notas da 5ª Sinfonia de Beethoven (1808) são o exemplo mais famoso de motivo: tudo no 1º movimento nasce delas."],
  s: [
    { k: "mentor", t: "Eu sou **Salomão**. Diz a Escritura que compus mil e cinco cânticos (1 Rs 4.32). E também escrevi: há tempo para tudo (Ec 3.1). **Inclusive para o silêncio dentro de uma música.**" },
    { k: "txt", h: "O que é uma melodia", p: "Uma sequência de alturas **no tempo** que faz sentido como uma frase. Ela tem **contorno** (sobe e desce), **ritmo** e um **centro** (a tônica). As melodias mais lembradas costumam ter um arco: sobem até um ponto alto e voltam." },
    { k: "play", h: "Graus conjuntos × saltos", b: [["Graus conjuntos (suave)", "C4 D4 E4 F4 E4 D4 C4:2"], ["Saltos (enérgico)", "C4 G4 E4 C5 G4 E4 C4:2"], ["Salto + recuperação", "C4 A4 G4 F4 E4 D4 C4:2"]] },
    { k: "txt", h: "O motivo", p: "Um **motivo** é a menor ideia reconhecível: 3 a 5 notas com um ritmo marcante. O exemplo mais famoso: **sol-sol-sol-mi♭** da 5ª Sinfonia de Beethoven. Uma música inteira pode nascer de um motivo." },
    { k: "play", h: "O motivo de Beethoven", b: [["Ta-ta-ta-tááá", "bpm=108 G4:0.5 G4:0.5 G4:0.5 Eb4:3 -:0.5 F4:0.5 F4:0.5 F4:0.5 D4:4"]] },
    { k: "list", h: "Como variar um motivo", items: ["**Repetir** (fixa na memória)", "**Sequência**: o mesmo desenho começando em outra nota", "**Inversão**: o que subia, desce", "**Aumentação/diminuição**: mais lento ou mais rápido", "**Fragmentação**: usar só um pedaço"] },
    { k: "play", h: "Motivo e sequência", b: [["Motivo", "bpm=100 C4 D4 E4 C4:2"], ["Em sequência", "bpm=100 C4 D4 E4 C4 D4 E4 F4 D4 E4 F4 G4 E4:2"]] },
    { k: "txt", h: "Pergunta e resposta", p: "Uma frase que termina **suspensa** (no 2º, 5º ou 7º grau, sobre o V) soa como **pergunta**. A frase seguinte, parecida, termina na **tônica**: a **resposta**. Juntas formam um **período**." },
    { k: "play", h: "Ouça o período", b: [["Pergunta", "bpm=100 C4 E4 G4 E4 F4 D4 D4:2"], ["Resposta", "bpm=100 C4 E4 G4 E4 F4 D4 C4:2"], ["As duas", "bpm=100 C4 E4 G4 E4 F4 D4 D4:2 C4 E4 G4 E4 F4 D4 C4:2"]] },
    { k: "txt", h: "Prosódia: a letra manda", p: "A sílaba **tônica** da palavra deve cair no **tempo forte** da música. “Se-**nhor**” com acento no “se” soa errado (“**SE**-nhor”). Fale a letra em voz alta, marque as tônicas e só então escreva o ritmo." },
    { k: "pick", q: "Qual verso tem a prosódia mais natural para “**Deus** é **fiel**” num compasso que começa forte?", o: ["**Deus** (forte) é (fraco) fi-**el** (forte)", "Deus (fraco) **é** (forte) **fi**-el (forte-fraco)", "Todas as sílabas fortes", "Tanto faz"], c: 0, e: "Tônicas “Deus” e “el” nos tempos fortes." },
    { k: "tf", q: "Toda boa melodia precisa ter saltos grandes.", a: false, e: "Muitas melodias marcantes andam quase só por graus conjuntos. O segredo é o equilíbrio entre repetição e surpresa." },
    { k: "mentor", t: "Uma palavra dita a seu tempo é como maçãs de ouro (Pv 25.11). **Uma nota no lugar certo também.**" }
  ],
  q: [
    { q: "Motivo é:", o: ["A menor ideia musical reconhecível", "O refrão inteiro", "O tom da música", "A introdução"], c: 0, e: "Poucas notas com ritmo marcante." },
    { q: "Sequência é:", o: ["Repetir o motivo começando em outra nota", "Tocar a música do começo", "Inverter o acorde", "Uma escala cromática"], c: 0, e: "Mesmo desenho, outra altura." },
    { q: "Uma frase “pergunta” costuma terminar:", o: ["Suspensa (2º, 5º ou 7º grau)", "Na tônica", "Em silêncio total", "Uma oitava acima"], c: 0, e: "A resposta conclui na tônica." },
    { q: "Prosódia é:", o: ["Casar a sílaba tônica com o tempo forte", "A rima da letra", "O andamento", "O número de estrofes"], c: 0, e: "A letra manda no ritmo." },
    { q: "Depois de um salto grande, a regra clássica sugere:", o: ["Voltar por grau conjunto na direção oposta", "Fazer outro salto maior", "Parar a música", "Repetir o salto"], c: 0, e: "Recuperação do salto." },
    { q: "Ouça: a frase termina como pergunta ou resposta?", play: "bpm=100 C4 E4 G4 E4 F4 D4 C4:2", o: ["Resposta (na tônica)", "Pergunta (suspensa)"], c: 0, e: "Terminou no Dó." }
  ]
};
LESSONS.c2 = {
  title: "Forma musical", sub: "A arquitetura da música: do rondó à forma sonata e à canção com refrão e ponte",
  goals: ["Reconhecer formas clássicas (AB, ABA, rondó, sonata)", "Entender as partes de uma canção", "Planejar a forma de uma composição"],
  mis: "Escolha 2 louvores e escreva o mapa da forma (ex.: Intro – V1 – Pré – R – V2 – Pré – R – Ponte ×4 – R – Final). Compare: o que se repete? Onde está o ponto alto?",
  deep: ["A forma AABA de 32 compassos foi o padrão da canção americana nas décadas de 1920–1950 (muitos standards de jazz).", "A forma sonata organiza o 1º movimento de muitas sinfonias e sonatas do Classicismo: exposição (tema 1 na tônica, tema 2 na dominante ou relativa), desenvolvimento, reexposição (tudo na tônica)."],
  s: [
    { k: "txt", h: "Repetição + contraste", p: "Forma é a **arquitetura** da música no tempo. Toda forma equilibra **repetição** (dá unidade e memória) e **contraste** (dá interesse). Letras indicam as partes: **A** é uma ideia, **B** é outra." },
    { k: "table", h: "Formas clássicas", head: ["Forma", "Esquema", "Ideia"], rows: [["Binária", "A B", "Duas partes complementares"], ["Ternária", "A B A", "Sai e volta (muito comum)"], ["Rondó", "A B A C A", "Um refrão que sempre volta"], ["Tema e variações", "A A' A'' A'''", "O mesmo tema transformado"], ["Forma sonata", "Exposição – Desenvolvimento – Reexposição", "Drama entre dois temas e dois tons"], ["Estrófica", "A A A A", "Mesma música, letras diferentes (hinos)"]] },
    { k: "txt", h: "A canção moderna", p: "**Intro** → **Verso** (conta a história) → **Pré-refrão** (prepara, sobe a energia) → **Refrão** (a mensagem principal, o “gancho”) → **Ponte** (contraste, outro ângulo) → **Final**." },
    { k: "match", h: "Ligue a parte à função", pairs: [["Verso", "Conta a história"], ["Pré-refrão", "Prepara e sobe a energia"], ["Refrão", "Mensagem principal, o gancho"], ["Ponte", "Contraste, outro ângulo"], ["Intro", "Apresenta o clima e o tom"]] },
    { k: "txt", h: "Hinos e louvor", p: "O **hino tradicional** é estrófico (várias estrofes com a mesma música) e muitas vezes tem um **coro** que se repete. O **louvor contemporâneo** costuma ter uma **ponte repetitiva** que cresce em dinâmica, o momento de maior intensidade congregacional." },
    { k: "txt", h: "Forma sonata em 3 atos", p: "**Exposição**: apresenta o tema 1 (na tônica) e o tema 2 (em outro tom). **Desenvolvimento**: fragmenta, modula, cria conflito. **Reexposição**: os temas voltam, agora os dois na tônica: o conflito se resolve. É drama sem palavras." },
    { k: "pick", q: "Qual forma tem um “refrão” que volta entre episódios diferentes?", o: ["Rondó (A B A C A)", "Binária (A B)", "Tema e variações", "Estrófica"], c: 0, e: "O A volta sempre." },
    { k: "txt", h: "A curva de energia", p: "Desenhe a energia da música como um gráfico: começa média, cresce no pré, explode no refrão, recua no verso 2, sobe ainda mais na ponte e fecha. Arranjo, dinâmica e letra devem seguir essa curva." },
    { k: "tf", q: "Repetição demais sempre deixa a música ruim.", a: false, e: "Repetição é o que faz a música ser lembrada. O problema é repetição **sem** contraste em lugar nenhum." },
    { k: "chat", who: "Júlia, compositora", c: "#C2255C", q: "Minha música nova tem 4 versos diferentes e nenhum refrão. O pessoal não consegue cantar 😕", o: [
      { t: "Cria um refrão curto com a ideia central, repete depois de cada verso. Repetição dá memória; os versos dão contraste.", v: 2, r: "Vou condensar a mensagem num refrão de 4 linhas!", tip: "Exatamente: equilíbrio entre repetição e contraste." },
      { t: "Coloca mais versos, aí fica mais rica.", v: 0, r: "Acho que vai ficar mais difícil ainda…", tip: "Sem repetição, a congregação não acompanha." },
      { t: "Faz tudo igual, sem verso nenhum.", v: 1, r: "Mas aí perde a história…", tip: "Repetição pura também cansa." }] },
    { k: "mentor", t: "Eu construí o templo com plano e medida (1 Rs 6). **Uma música também tem planta baixa.** Desenhe a forma antes de levantar as paredes." }
  ],
  q: [
    { q: "A forma ternária é:", o: ["A B A", "A B", "A B A C A", "A A A A"], c: 0, e: "Sai e volta." },
    { q: "A forma estrófica é típica de:", o: ["Hinos tradicionais", "Fugas", "Forma sonata", "Rondós"], c: 0, e: "Várias estrofes, mesma música." },
    { q: "As três partes da forma sonata são:", o: ["Exposição, desenvolvimento, reexposição", "Intro, verso, refrão", "Prelúdio, fuga, coda", "Allegro, adagio, presto"], c: 0, e: "Com dois temas em tons diferentes." },
    { q: "A função do pré-refrão é:", o: ["Preparar e subir a energia para o refrão", "Encerrar a música", "Trocar de instrumento", "Apresentar a banda"], c: 0, e: "Ponte de energia." },
    { q: "Toda forma equilibra:", o: ["Repetição e contraste", "Graves e agudos", "Maior e menor", "Voz e instrumento"], c: 0, e: "Unidade e interesse." },
    { q: "Tema e variações é:", o: ["O mesmo tema transformado várias vezes", "Vários temas sem relação", "Uma música sem tema", "Uma forma só vocal"], c: 0, e: "A A' A'' …" }
  ]
};
LESSONS.c3 = {
  title: "Arranjo e condução de vozes", sub: "Registro, SATB, movimento das vozes, densidade e orquestração",
  goals: ["Conhecer as tessituras das vozes do coral", "Conduzir vozes com movimento mínimo", "Planejar camadas e densidade num arranjo"],
  mis: "Arranje um refrão para 4 vozes (SATB) ou para banda: decida quem entra quando, em que região, e onde todos param. Escreva um “mapa de camadas” por seção.",
  deep: ["Tessituras aproximadas (variam por pessoa): Soprano Dó4–Lá5; Contralto Sol3–Ré5; Tenor Dó3–Lá4; Baixo Mi2–Ré4.", "Quintas e oitavas paralelas são evitadas no estilo coral tradicional porque fazem duas vozes soarem como uma; na música popular (power chords, por exemplo) elas são parte do estilo."],
  s: [
    { k: "txt", h: "Arranjar é decidir", p: "Uma mesma melodia pode virar um hino solene, um samba ou uma balada. Arranjar é decidir **quem toca o quê, em que região, quando entra e quando sai**. O arranjador é o “arquiteto de interiores” da composição." },
    { k: "table", h: "As vozes do coral (SATB)", p: "Tessituras aproximadas: cada voz é única.", head: ["Voz", "Região aproximada"], rows: [["Soprano", "Dó4 a Lá5"], ["Contralto", "Sol3 a Ré5"], ["Tenor", "Dó3 a Lá4 (escrito na clave de sol uma oitava acima)"], ["Baixo", "Mi2 a Ré4"]] },
    { k: "txt", h: "Condução de vozes", p: "Ao passar de um acorde para outro:\n• **Notas comuns ficam paradas.**\n• As outras vozes andam para a nota **mais próxima**.\n• Evite saltos sem motivo.\nO resultado é um som liso, cantável, “amarrado”." },
    { k: "play", h: "Ouça a diferença", b: [["Tudo saltando (blocos)", "bpm=72 [C4 E4 G4]:2 [F4 A4 C5]:2 [G4 B4 D5]:2 [C4 E4 G4]:3"], ["Movimento mínimo", "bpm=72 [C3 E4 G4 C5]:2 [F2 F4 A4 C5]:2 [G2 D4 G4 B4]:2 [C3 E4 G4 C5]:3"]] },
    { k: "list", h: "Tipos de movimento entre duas vozes", items: ["**Direto/paralelo**: as duas vão na mesma direção", "**Contrário**: uma sobe, outra desce (dá independência)", "**Oblíquo**: uma fica, a outra se move"] },
    { k: "txt", h: "Por que evitar quintas paralelas?", p: "No coral tradicional, duas vozes em **5ªs ou 8ªs paralelas** se fundem e soam como uma só: o coral perde uma voz. Já na guitarra de rock, os “power chords” são 5ªs paralelas de propósito. **Regra depende do estilo.**" },
    { k: "play", h: "Paralelas × contrário", b: [["5ªs paralelas", "bpm=80 [C4 G4] [D4 A4] [E4 B4] [F4 C5]:2"], ["Movimento contrário", "bpm=80 [C4 C5] [D4 B4] [E4 A4] [F4 G4]:2"]] },
    { k: "txt", h: "Densidade e espaço", p: "Um arranjo bom **respira**. Comece com poucas camadas e acrescente: verso 1 só teclado e voz; verso 2 entra baixo; refrão entra tudo; ponte recua e cresce. O **silêncio** de um instrumento também é arranjo." },
    { k: "txt", h: "Orquestração em 1 minuto", p: "Famílias: **cordas** (violino, viola, violoncelo, contrabaixo), **madeiras** (flauta, oboé, clarinete, fagote), **metais** (trompa, trompete, trombone, tuba) e **percussão**. Cada família tem cor e força. Dobrar uma melodia em oitavas dá peso; em instrumentos diferentes cria novos timbres." },
    { k: "pick", q: "Qual voz do coral costuma cantar na região de Dó3 a Lá4?", o: ["Tenor", "Soprano", "Contralto", "Baixo"], c: 0, e: "O tenor: a voz masculina mais aguda do coral." },
    { k: "tf", q: "Na condução de vozes, notas comuns entre dois acordes devem, de preferência, ficar paradas.", a: true, e: "Isso deixa o encadeamento mais liso." },
    { k: "mentor", t: "Em casa onde há muitos conselheiros, há segurança (Pv 11.14). **Num arranjo, muitas vozes só ajudam se cada uma souber o seu lugar.**" }
  ],
  q: [
    { q: "SATB significa:", o: ["Soprano, Contralto, Tenor e Baixo", "Som, Arranjo, Tom e Batida", "Solo, Acompanhamento, Tema e Base", "Sax, Acordeon, Trompete e Bateria"], c: 0, e: "As 4 vozes do coral." },
    { q: "Na condução de vozes ideal, as vozes:", o: ["Andam para a nota mais próxima", "Saltam o máximo possível", "Andam sempre paralelas", "Ficam todas na mesma nota"], c: 0, e: "Movimento mínimo." },
    { q: "Movimento contrário é quando:", o: ["Uma voz sobe e a outra desce", "As duas sobem", "Uma fica parada", "As duas param"], c: 0, e: "Dá independência." },
    { q: "Por que o coral tradicional evita quintas paralelas?", o: ["As vozes perdem independência e soam como uma", "Porque é difícil de cantar", "Porque soa desafinado", "Porque é proibido por lei"], c: 0, e: "As duas vozes se fundem." },
    { q: "O trompete pertence à família dos:", o: ["Metais", "Madeiras", "Cordas", "Percussão"], c: 0, e: "Metais." },
    { q: "Um arranjo que “respira”:", o: ["Acrescenta e tira camadas ao longo da música", "Usa todos os instrumentos o tempo todo", "Não tem dinâmica", "Só tem voz"], c: 0, e: "Densidade que varia." }
  ]
};
LESSONS.c4 = {
  title: "Música e emoção: a ciência", sub: "Por que uma trilha dá medo, um acorde arrepia e o maior nem sempre é alegre",
  goals: ["Listar recursos musicais que criam tensão", "Entender a teoria da expectativa", "Saber os limites: cultura e contexto"],
  mis: "Assista a uma cena de suspense de um filme sem som e depois com som. Liste os recursos que você ouviu (dissonância, grave, silêncio, ostinato…). Depois faça o mesmo com um momento de adoração no culto.",
  deep: ["Leonard Meyer (Emotion and Meaning in Music, 1956) e David Huron (Sweet Anticipation, 2006) explicam a emoção musical pela expectativa: prever, confirmar, frustrar, surpreender.", "Estudos de neuroimagem (Blood & Zatorre, 2001; Salimpoor e colegas, 2011) ligam os “arrepios” musicais ao sistema de recompensa do cérebro, com liberação de dopamina."],
  s: [
    { k: "txt", h: "Música também é ciência", p: "Uma trilha de suspense provoca tensão com **dissonâncias, intervalos, timbres, ritmo, silêncio e escolhas harmônicas**. Estudar esses recursos ajuda a entender como eles afetam quem escuta. Mas **não há fórmula única** para cada emoção: contexto, cultura e a pessoa contam." },
    { k: "play", h: "Ferramentas do suspense", b: [["Trítono no grave", "bpm=60 [C2 F#2]:2 [C2 F#2]:2 [C2 F#2]:3"], ["Cluster (notas vizinhas)", "[C4 Db4 D4 Eb4]:4"], ["Ostinato que insiste", "bpm=150 E2 F2 E2 F2 E2 F2 E2 F2 E2 F2 E2 F2:3"], ["Acorde aumentado", "[C4 E4 G#4 C5]:3"]] },
    { k: "list", h: "Recursos que criam tensão", items: ["**Dissonância** (2ªs, 7ªs, trítono, clusters)", "**Registro extremo** (muito grave ou muito agudo)", "**Ostinato**: um padrão que insiste", "**Crescendo** longo sem resolver", "**Silêncio súbito**", "**Harmonia ambígua** (aumentados, diminutos, sem tônica clara)", "**Ritmo irregular** ou acelerando"] },
    { k: "txt", h: "A teoria da expectativa", p: "O cérebro **prevê** o que vem a seguir. Quando a música confirma, sentimos prazer; quando adia ou surpreende (uma cadência interrompida, uma pausa), sentimos tensão e depois alívio. Compositores brincam com essas previsões." },
    { k: "txt", h: "Os arrepios", p: "Aquele arrepio num momento especial da música tem base no corpo: estudos de neuroimagem ligam esses momentos ao **sistema de recompensa** do cérebro, com liberação de **dopamina**, inclusive na **antecipação** do trecho que você ama." },
    { k: "txt", h: "Maior = alegre? Nem sempre", p: "No Ocidente aprendemos a associar maior com alegria e menor com tristeza, mas é **aprendizado cultural** em boa parte. **Andamento, registro, ritmo e timbre** pesam tanto quanto. Um menor rápido pode ser festivo; um maior lento pode ser melancólico." },
    { k: "play", h: "Teste você mesmo", b: [["Maior lento e grave", "bpm=50 C3 E3 G3 E3 C3:3"], ["Menor rápido e agudo", "bpm=170 C5 Eb5 G5 Eb5 C5 Eb5 G5 C6:2"]] },
    { k: "txt", h: "Leitmotiv", p: "Wagner associava temas a personagens e ideias (**leitmotiv**). Trilhas de cinema fazem isso até hoje: o tema volta, transformado, quando o personagem aparece ou muda. Você reconhece o herói antes de vê-lo." },
    { k: "txt", h: "O silêncio", p: "Pausa não é ausência de música: é **expectativa máxima**. Um corte seco antes do refrão final pode ser o momento mais forte de toda a música." },
    { k: "pick", q: "Qual destes recursos **não** costuma ser usado para criar suspense?", o: ["Cadência perfeita em acorde maior, bem resolvida", "Trítono no grave", "Ostinato insistente", "Silêncio súbito"], c: 0, e: "A cadência perfeita resolve a tensão." },
    { k: "chat", who: "Pr. Marcos", c: "#5F3DC4", q: "Preciso de um fundo musical para um momento de oração, sem distrair. Ideias?", o: [
      { t: "Pad suave, andamento lento, harmonia simples com poucas mudanças, sem melodias marcantes e com volume baixo. Algo que acolhe sem chamar atenção.", v: 2, r: "Perfeito, é isso que eu imaginava.", tip: "Recursos a serviço do momento: estabilidade, pouca novidade, timbre suave." },
      { t: "Um solo de guitarra bem expressivo, pra emocionar.", v: 0, r: "Acho que vai roubar a atenção da oração…", tip: "Melodia marcante puxa a atenção." },
      { t: "Acordes diminutos e aumentados pra criar clima.", v: 1, r: "Não vai ficar parecendo filme de terror? 😅", tip: "Ambiguidade harmônica gera tensão, não acolhimento." }] },
    { k: "mentor", t: "Há tempo de chorar e tempo de rir (Ec 3.4). **Conhecer os recursos é saber servir cada tempo com a música certa.**" }
  ],
  q: [
    { q: "Segundo a teoria da expectativa, a emoção musical vem de:", o: ["Previsões confirmadas, adiadas ou frustradas", "Só do volume", "Só da letra", "Da quantidade de instrumentos"], c: 0, e: "Meyer e Huron." },
    { q: "“Arrepios” musicais estão ligados a:", o: ["Sistema de recompensa e dopamina", "Frio no ambiente", "Volume acima de 100 dB", "Apenas a acordes menores"], c: 0, e: "Estudos de neuroimagem." },
    { q: "Leitmotiv é:", o: ["Um tema associado a um personagem ou ideia", "Um tipo de compasso", "Um instrumento alemão", "Uma cadência"], c: 0, e: "Wagner; trilhas de cinema." },
    { q: "Maior = alegre e menor = triste é:", o: ["Uma associação em boa parte cultural, não uma lei", "Uma lei física universal", "Verdade em qualquer andamento", "Regra da forma sonata"], c: 0, e: "Andamento, registro e timbre também pesam." },
    { q: "Qual recurso aumenta a tensão?", o: ["Ostinato insistente no grave", "Cadência V–I", "Acorde maior longo e resolvido", "Diminuir o andamento até parar no I"], c: 0, e: "Insistência sem resolução." },
    { q: "Ouça: que sensação esse acorde passa?", play: "[C4 Db4 D4 Eb4]:4", o: ["Tensão / inquietação", "Repouso total", "Alegria festiva", "Conclusão"], c: 0, e: "Cluster: notas vizinhas, muito dissonante." }
  ]
};

LESSONS.hi1 = {
  title: "Das origens à Idade Média", sub: "Flautas de osso, Pitágoras, os levitas, o gregoriano, Guido d'Arezzo e a polifonia",
  goals: ["Situar as primeiras evidências de música", "Entender o canto gregoriano e a notação", "Saber como nasceu a polifonia"],
  mis: "Ouça uma gravação de canto gregoriano e uma de organum (procure “Pérotin Viderunt omnes”). Escreva 3 diferenças entre os dois.",
  deep: ["Flautas de osso de abutre e de marfim de mamute encontradas em cavernas do sul da Alemanha (Hohle Fels, Geissenklösterle) têm cerca de 40 mil anos.", "A troca de “ut” por “dó” é atribuída a Giovanni Battista Doni, no século XVII; o “si” veio das iniciais de Sancte Iohannes."],
  s: [
    { k: "mentor", t: "Eu sou **Jubal**. Gênesis diz que fui “o pai de todos os que tocam harpa e flauta” (Gn 4.21). Vem comigo ver de onde viemos." },
    { k: "big", n: "~40 mil anos", p: "Idade das **flautas de osso** encontradas em cavernas da Alemanha: os instrumentos mais antigos que conhecemos. Fazer música é tão antigo quanto ser humano." },
    { k: "txt", h: "Pitágoras e os números", p: "Na Grécia antiga, atribui-se a **Pitágoras** a descoberta de que intervalos bonitos vêm de **proporções simples** de comprimento de corda: **2:1 oitava, 3:2 quinta, 4:3 quarta**. Música e matemática nasceram juntas." },
    { k: "txt", h: "Música no templo", p: "Em Israel, a música era organizada: Davi separou os filhos de Asafe, Hemã e Jedutum para profetizar com harpas, liras e címbalos. Eram **288** músicos “instruídos no canto ao Senhor” (1 Cr 25.7). Os **Salmos** eram o hinário." },
    { k: "txt", h: "O canto gregoriano", p: "Na Igreja medieval, o **canto gregoriano**: **monofônico** (uma melodia só), em **latim**, com ritmo livre, seguindo o texto, e construído sobre os **modos eclesiásticos**. Era cantado nas missas e ofícios dos mosteiros." },
    { k: "play", h: "Uma melodia modal (estilo gregoriano)", b: [["Modo dórico em Ré", "bpm=84 D4 F4 G4 A4:2 G4 A4 C5 A4 G4 F4 G4 E4 D4:3"]] },
    { k: "txt", h: "Guido d'Arezzo (séc. XI)", p: "Um monge italiano que popularizou a **pauta com linhas** e um método de solfejo com as sílabas de um hino a São João: **UT** queant laxis, **RE**sonare fibris, **MI**ra gestorum, **FA**muli tuorum, **SOL**ve polluti, **LA**bii reatum. O “ut” virou **dó** séculos depois." },
    { k: "txt", h: "Nasce a polifonia", p: "Aos poucos, acrescentaram uma segunda voz à melodia: o **organum** (vozes em 4ªs e 5ªs paralelas). Em Paris, na **Escola de Notre-Dame** (séc. XII–XIII), **Léonin** e **Pérotin** escreveram organa a várias vozes. No séc. XIV, a **Ars Nova** (Guillaume de Machaut) trouxe novas possibilidades rítmicas." },
    { k: "play", h: "Organum paralelo", b: [["Melodia + quinta paralela", "bpm=80 [D4 A4] [F4 C5] [G4 D5] [A4 E5]:2 [G4 D5] [F4 C5] [E4 B4] [D4 A4]:3"]] },
    { k: "order", h: "Coloque em ordem cronológica", items: ["Flautas de osso", "Pitágoras e as proporções", "Canto gregoriano", "Guido d'Arezzo e a pauta", "Escola de Notre-Dame (Pérotin)"] },
    { k: "tf", q: "O canto gregoriano é polifônico, com várias melodias ao mesmo tempo.", a: false, e: "É monofônico: uma única linha melódica." },
    { k: "mentor", t: "Da flauta de osso ao órgão da catedral, o ser humano sempre fez música para **lembrar, celebrar e adorar**. Você faz parte dessa história." }
  ],
  q: [
    { q: "Os instrumentos mais antigos conhecidos são:", o: ["Flautas de osso de ~40 mil anos", "Pianos medievais", "Violinos egípcios", "Tambores de metal"], c: 0, e: "Encontradas na Alemanha." },
    { q: "Pela tradição pitagórica, a quinta corresponde à proporção:", o: ["3:2", "2:1", "4:3", "5:4"], c: 0, e: "2:1 é a oitava." },
    { q: "O canto gregoriano é:", o: ["Monofônico, em latim, ritmo livre", "Polifônico e em português", "Instrumental", "Em compasso 4/4 fixo"], c: 0, e: "Uma só linha melódica." },
    { q: "Guido d'Arezzo é lembrado por:", o: ["Pauta e sílabas ut-re-mi", "Inventar o piano", "Compor o Messias", "Criar a ópera"], c: 0, e: "Século XI." },
    { q: "Léonin e Pérotin pertencem à:", o: ["Escola de Notre-Dame", "Escola de Viena", "Bossa nova", "Ars Antiqua grega"], c: 0, e: "Paris, séc. XII–XIII." },
    { q: "Organum é:", o: ["Uma das primeiras formas de polifonia", "Um órgão de tubos", "Um tipo de cadência", "Uma dança barroca"], c: 0, e: "Voz acrescentada à melodia." }
  ]
};
LESSONS.hi2 = {
  title: "Renascimento, Reforma e Barroco", sub: "Palestrina, Lutero, a ópera, o baixo contínuo e Bach",
  goals: ["Situar Renascimento e Barroco no tempo", "Entender o impacto da Reforma no canto congregacional", "Conhecer Bach, Handel e Vivaldi"],
  mis: "Ouça o coro “Aleluia” do Messias de Handel e um prelúdio de O Cravo Bem Temperado de Bach. Anote: quantas vozes você consegue seguir em cada um?",
  deep: ["A imprensa musical de Ottaviano Petrucci (Veneza, 1501) espalhou partituras pela Europa.", "O Cravo Bem Temperado (livro 1 em 1722, livro 2 por volta de 1742) tem prelúdio e fuga em todos os 24 tons maiores e menores."],
  s: [
    { k: "txt", tag: "c. 1400–1600", h: "Renascimento", p: "Polifonia vocal equilibrada, vozes que se **imitam**, texto mais claro. Nomes: **Josquin des Prez**, **Palestrina** (missas de grande clareza), **Tomás Luis de Victoria**. A **imprensa musical** (Petrucci, 1501) espalhou partituras pela Europa." },
    { k: "txt", tag: "1517 em diante", h: "A Reforma e o canto do povo", p: "**Martinho Lutero**, que era músico, quis a **congregação cantando** na própria língua: nasceu o **coral luterano** (ex.: “Castelo Forte”). Em Genebra, o **Saltério** cantava os Salmos em métrica. O canto congregacional que você conhece tem raízes aqui." },
    { k: "txt", tag: "c. 1600–1750", h: "Barroco", p: "Nasce a **ópera** (Monteverdi, _L'Orfeo_, 1607). A tonalidade maior/menor se firma. Contraste (solo × tutti, forte × piano) e **ornamentos**. A base de tudo: o **baixo contínuo**, uma linha de baixo com a harmonia improvisada por cravo ou órgão." },
    { k: "txt", h: "Os gigantes", p: "**Vivaldi** (Itália): concertos, _As Quatro Estações_.\n**Handel** (Alemanha/Inglaterra): óperas e oratórios, o _Messias_ (1741–1742) com o coro “Aleluia”.\n**J. S. Bach** (1685–1750): fugas, cantatas, as _Paixões_. Assinava muitas obras com **S.D.G.**: _Soli Deo Gloria_, “só a Deus a glória”." },
    { k: "txt", h: "A fuga", p: "Um **tema** (o sujeito) entra numa voz; depois outra voz o imita (a **resposta**, geralmente na 5ª), e outra, e outra, enquanto as anteriores seguem com contrapontos. É uma conversa de vozes independentes, a arte máxima do contraponto." },
    { k: "play", h: "Imitação: sujeito e resposta", b: [["Sujeito → resposta na 5ª", "bpm=110 C4 E4 G4 F4 E4 D4 C4:2 [G3 G4] [B3 B4] [C4 D5] [D4 C5] [E4 B4] [F4 A4] [E4 G4]:2"]] },
    { k: "txt", h: "O Cravo Bem Temperado", p: "Bach escreveu prelúdios e fugas nos **24 tons** maiores e menores. Isso só era viável com afinações (temperamentos) que permitissem tocar bem em todos os tons. Até hoje é “bíblia” dos pianistas." },
    { k: "match", h: "Ligue a obra ao compositor", pairs: [["Messias", "Handel"], ["As Quatro Estações", "Vivaldi"], ["Paixão segundo São Mateus", "Bach"], ["L'Orfeo", "Monteverdi"]] },
    { k: "tf", q: "Bach assinava muitas obras com S.D.G., “Só a Deus a glória”.", a: true, e: "_Soli Deo Gloria._" },
    { k: "mentor", t: "Quando um povo inteiro canta junto na própria língua, a música vira **teologia cantada**. Lutero sabia disso." }
  ],
  q: [
    { q: "O Barroco vai aproximadamente de:", o: ["1600 a 1750", "1400 a 1600", "1750 a 1820", "1850 a 1900"], c: 0, e: "Monteverdi a Bach." },
    { q: "Lutero incentivou:", o: ["A congregação cantar na própria língua", "Só música instrumental", "Proibir hinos", "Ópera nas igrejas"], c: 0, e: "O coral luterano." },
    { q: "Baixo contínuo é:", o: ["Linha de baixo com harmonia improvisada sobre ela", "Um contrabaixo elétrico", "Uma nota longa sem fim", "Um tipo de voz"], c: 0, e: "Marca do Barroco." },
    { q: "Numa fuga, a “resposta” é:", o: ["O tema imitado por outra voz", "A última nota", "Um solo improvisado", "O aplauso"], c: 0, e: "Geralmente na 5ª." },
    { q: "Quem compôs o Messias?", o: ["Handel", "Bach", "Vivaldi", "Mozart"], c: 0, e: "1741–1742." },
    { q: "A primeira grande ópera que ainda se apresenta, L'Orfeo (1607), é de:", o: ["Monteverdi", "Palestrina", "Wagner", "Verdi"], c: 0, e: "Início do Barroco." }
  ]
};
LESSONS.hi3 = {
  title: "Clássico, Romântico e século XX", sub: "Haydn, Mozart e Beethoven; Chopin e Wagner; Debussy, Stravinsky, Villa-Lobos",
  goals: ["Distinguir Classicismo, Romantismo e música do século XX", "Conhecer os compositores principais e suas marcas", "Situar a música erudita brasileira"],
  mis: "Ouça, no mesmo dia: o 1º movimento da Sinfonia nº 40 de Mozart, um noturno de Chopin, “Clair de Lune” de Debussy e a Bachiana Brasileira nº 5 de Villa-Lobos. Descreva cada uma em 3 palavras.",
  deep: ["A estreia de A Sagração da Primavera (Paris, 1913) foi um dos escândalos mais famosos da história da música.", "Beethoven compôs a 9ª Sinfonia (1824) praticamente surdo; o final usa a “Ode à Alegria” de Schiller com coro."],
  s: [
    { k: "txt", tag: "c. 1750–1820", h: "Classicismo", p: "Clareza, equilíbrio, melodias com acompanhamento simples, **forma sonata**. Nascem a **sinfonia** e o **quarteto de cordas** como os conhecemos. **Haydn** (o “pai da sinfonia”), **Mozart** (gênio precoce, mais de 600 obras) e o jovem **Beethoven**." },
    { k: "txt", h: "Beethoven, a ponte", p: "**Beethoven** (1770–1827) levou a forma clássica ao limite e abriu o Romantismo. Ficou surdo e mesmo assim compôs a **9ª Sinfonia** (1824), com a **Ode à Alegria** cantada por coro, uma novidade numa sinfonia." },
    { k: "play", h: "Ode à Alegria", b: [["Tema", "bpm=110 E4 E4 F4 G4 G4 F4 E4 D4 C4 C4 D4 E4 E4:1.5 D4:0.5 D4:2"]] },
    { k: "txt", tag: "Século XIX", h: "Romantismo", p: "Emoção, individualidade, natureza, virtuosismo e **nacionalismo**. A orquestra cresce. **Schubert** (canções), **Chopin** (piano poético), **Liszt** (virtuose), **Brahms**, **Tchaikovsky**, **Verdi** (ópera italiana) e **Wagner** (drama musical, leitmotiv, harmonia cromática)." },
    { k: "txt", tag: "Século XX", h: "Novos caminhos", p: "**Debussy**: impressionismo, cores, escala de **tons inteiros**. **Stravinsky**: _A Sagração da Primavera_ (1913), ritmos selvagens, escândalo na estreia. **Schoenberg**: atonalismo e **dodecafonismo** (as 12 notas em série). Depois: minimalismo (**Steve Reich**, **Philip Glass**), música eletrônica e trilhas de cinema." },
    { k: "play", h: "A cor de Debussy", b: [["Escala de tons inteiros", "bpm=120 C4 D4 E4 F#4 G#4 A#4 C5:2"], ["Acorde de tons inteiros", "[C4 E4 G#4 A#4 D5]:4"]] },
    { k: "txt", h: "Brasil erudito", p: "**Carlos Gomes** levou a ópera brasileira à Itália com _O Guarani_ (1870). **Heitor Villa-Lobos** misturou Bach e música brasileira nas **Bachianas Brasileiras**. **Camargo Guarnieri** e **Francisco Mignone** seguiram o nacionalismo. **Chiquinha Gonzaga** foi pioneira entre o erudito e o popular." },
    { k: "order", h: "Coloque os períodos em ordem", items: ["Renascimento", "Barroco", "Classicismo", "Romantismo", "Século XX"] },
    { k: "match", h: "Ligue o compositor à marca", pairs: [["Haydn", "Pai da sinfonia"], ["Chopin", "Piano poético"], ["Wagner", "Leitmotiv"], ["Debussy", "Impressionismo"], ["Villa-Lobos", "Bachianas Brasileiras"]] },
    { k: "tf", q: "Beethoven compôs a 9ª Sinfonia já praticamente surdo.", a: true, e: "E ela se tornou uma das obras mais conhecidas do mundo." },
    { k: "mentor", t: "Cada geração respondeu à anterior: equilíbrio, depois paixão, depois ruptura. **Conhecer o caminho te ajuda a escolher o seu.**" }
  ],
  q: [
    { q: "Quem é chamado de “pai da sinfonia”?", o: ["Haydn", "Bach", "Chopin", "Wagner"], c: 0, e: "Escreveu mais de 100 sinfonias." },
    { q: "A Ode à Alegria está na:", o: ["9ª Sinfonia de Beethoven", "Sinfonia nº 40 de Mozart", "Bachiana nº 5", "Sagração da Primavera"], c: 0, e: "1824." },
    { q: "O Romantismo valoriza:", o: ["Emoção, virtuosismo e nacionalismo", "Só equilíbrio e simetria", "Música apenas vocal", "Minimalismo"], c: 0, e: "Século XIX." },
    { q: "Debussy é associado ao:", o: ["Impressionismo", "Barroco", "Dodecafonismo", "Bossa nova"], c: 0, e: "Cores e tons inteiros." },
    { q: "O Guarani (1870) é ópera de:", o: ["Carlos Gomes", "Villa-Lobos", "Tom Jobim", "Pixinguinha"], c: 0, e: "Estreou em Milão." },
    { q: "Ouça: que escala é esta?", play: "bpm=120 C4 D4 E4 F#4 G#4 A#4 C5:2", o: ["Tons inteiros", "Maior", "Cromática", "Pentatônica"], c: 0, e: "Só tons: som “flutuante” de Debussy." }
  ]
};
LESSONS.hi4 = {
  title: "Música popular e louvor", sub: "Spirituals, blues, jazz, gospel, rock, choro, samba, bossa nova e os hinários brasileiros",
  goals: ["Ligar spirituals e blues ao jazz, gospel e rock", "Conhecer marcos da música popular brasileira", "Situar a música cristã no Brasil"],
  mis: "Monte uma playlist de 8 faixas que conte essa história: um spiritual, um blues, um jazz, um gospel, um choro, um samba antigo, uma bossa nova e um hino da Harpa ou do Cantor Cristão.",
  deep: ["“Pelo Telefone” (registrado em 1916, gravado em 1917) é considerado o primeiro samba gravado.", "Salmos e Hinos (1861), organizado por Sarah e Robert Kalley, é considerado o primeiro hinário evangélico em português no Brasil."],
  s: [
    { k: "txt", h: "Spirituals e blues", p: "Nos EUA, os africanos escravizados criaram os **spirituals**: cantos de fé e esperança (“Go Down, Moses”). Do mesmo chão nasceu o **blues**: 12 compassos sobre I, IV e V, com as **blue notes** (3ª e 7ª abaixadas, e a 4ª aumentada)." },
    { k: "play", h: "O som do blues", b: [["Escala blues em Lá", "bpm=110 A3 C4 D4 D#4 E4 G4 A4:2"], ["Acorde I7 do blues", "[A2 E3 G3 C#4]:3"]] },
    { k: "txt", tag: "Início do séc. XX", h: "Jazz", p: "De Nova Orleans para o mundo: **improvisação**, **swing**, harmonias ricas. **Louis Armstrong**, **Duke Ellington**, **Charlie Parker** (bebop), **Miles Davis**, **John Coltrane** (que gravou _A Love Supreme_ como oração)." },
    { k: "txt", h: "Gospel", p: "**Thomas A. Dorsey**, chamado de “pai da música gospel”, uniu blues e hinos nos anos 1930. **Mahalia Jackson** levou o gospel ao mundo. Ele influenciou diretamente o soul (Aretha Franklin, Ray Charles) e o pop." },
    { k: "txt", tag: "Anos 1950", h: "Rock", p: "Blues + country + gospel = **rock and roll**: **Chuck Berry**, **Elvis Presley**, **Little Richard**. Nos anos 60, **Beatles** e **Rolling Stones** mudaram a música pop para sempre." },
    { k: "txt", h: "Brasil popular", p: "**Choro** (fim do séc. XIX): Pixinguinha, Jacob do Bandolim. **Samba**: “Pelo Telefone” (1916/1917), Cartola, Noel Rosa. **Baião**: Luiz Gonzaga. **Bossa nova** (fim dos anos 1950): João Gilberto, Tom Jobim, “Chega de Saudade”. Depois: **MPB**, Tropicália, rock nacional." },
    { k: "play", h: "Harmonia de bossa nova", b: [["Acordes 7M e m7", "bpm=72 [C3 E4 G4 B4 D5]:2 [A2 G3 C4 E4]:2 [D3 F4 A4 C5]:2 [G2 F4 B4 E5]:2 [C3 E4 G4 B4]:4"]] },
    { k: "txt", h: "Música cristã no Brasil", p: "**Salmos e Hinos** (1861, Sarah e Robert Kalley): o primeiro hinário evangélico em português no país. **Cantor Cristão** (1891, batistas). **Harpa Cristã** (1922, Assembleias de Deus). Nas décadas seguintes, corinhos, grupos vocais e, a partir dos anos 1980–90, o movimento de **louvor e adoração** com bandas." },
    { k: "order", h: "Coloque em ordem cronológica", items: ["Salmos e Hinos (1861)", "Cantor Cristão (1891)", "Pelo Telefone (1916/1917)", "Harpa Cristã (1922)", "Chega de Saudade (1958)"] },
    { k: "match", h: "Ligue o artista ao estilo", pairs: [["Pixinguinha", "Choro"], ["Luiz Gonzaga", "Baião"], ["João Gilberto", "Bossa nova"], ["Mahalia Jackson", "Gospel"], ["Charlie Parker", "Bebop"]] },
    { k: "tf", q: "O gospel norte-americano influenciou o soul e o pop.", a: true, e: "Aretha Franklin e Ray Charles vieram da igreja." },
    { k: "mentor", t: "Do spiritual ao hinário, a música do povo sempre carregou **fé, dor e esperança**. O louvor de hoje é mais um capítulo dessa história." }
  ],
  q: [
    { q: "A forma básica do blues tem:", o: ["12 compassos sobre I, IV e V", "8 compassos em menor", "32 compassos AABA", "Forma sonata"], c: 0, e: "O blues de 12 compassos." },
    { q: "Thomas A. Dorsey é chamado de:", o: ["Pai da música gospel", "Rei do baião", "Pai da sinfonia", "Inventor do jazz"], c: 0, e: "Anos 1930." },
    { q: "“Chega de Saudade” é marco da:", o: ["Bossa nova", "Tropicália", "Música barroca", "Jovem Guarda"], c: 0, e: "1958." },
    { q: "A Harpa Cristã é de:", o: ["1922", "1861", "1891", "1958"], c: 0, e: "Assembleias de Deus." },
    { q: "O choro tem como grande nome:", o: ["Pixinguinha", "Elvis Presley", "Bach", "Miles Davis"], c: 0, e: "“Carinhoso”." },
    { q: "Ouça: que escala é essa?", play: "bpm=110 A3 C4 D4 D#4 E4 G4 A4:2", o: ["Blues", "Maior", "Menor harmônica", "Tons inteiros"], c: 0, e: "Com a blue note (Ré♯)." }
  ]
};

LESSONS.s1 = {
  title: "A física do som", sub: "Ondas, frequência, decibéis, série harmônica, envelope e fase",
  goals: ["Calcular comprimento de onda e entender os graves", "Entender decibel e proteger a audição", "Explicar timbre pela série harmônica"],
  mis: "Use um app de decibelímetro no celular durante o culto/ensaio: anote o nível no momento mais alto. Compare com o limite de 85 dB por 8 horas e converse com a equipe de som.",
  deep: ["Comprimento de onda λ = velocidade / frequência. 100 Hz ≈ 3,4 m; 1 kHz ≈ 34 cm; 10 kHz ≈ 3,4 cm.", "Recomendação de saúde ocupacional muito usada (NIOSH): 85 dB(A) por 8 h; a cada +3 dB, o tempo seguro cai pela metade."],
  s: [
    { k: "mentor", t: "Eu sou **Bezalel**. Deus me encheu de sabedoria e habilidade para todo tipo de trabalho artesanal no tabernáculo (Êx 31.3). **Engenharia de som também é artesanato: precisão a serviço da beleza.**" },
    { k: "txt", h: "A onda sonora", p: "Som é uma sequência de **compressões e rarefações** do ar. A cerca de 20 °C, ele viaja a **~343 m/s**. A **frequência** (Hz) é quantas vezes a onda se repete por segundo; a **amplitude** é o tamanho da variação de pressão." },
    { k: "txt", h: "Graves são enormes", p: "**Comprimento de onda = velocidade ÷ frequência.**\n100 Hz → **3,4 m**\n1 kHz → **34 cm**\n10 kHz → **3,4 cm**\nPor isso graves contornam paredes e são difíceis de controlar numa sala, e agudos são bloqueados por quase qualquer coisa." },
    { k: "big", n: "20 Hz – 20 kHz", p: "A faixa da audição humana jovem. Com a idade (e com exposição a som alto), os agudos são os primeiros a ir embora." },
    { k: "txt", h: "Decibel", p: "O **decibel** é logarítmico. **+6 dB** ≈ o dobro da pressão sonora; **+10 dB** soa mais ou menos “o dobro do volume”. Referência de saúde muito usada: **85 dB por até 8 horas**; a cada +3 dB, o tempo seguro cai pela metade. Shows e alguns cultos passam de 100 dB." },
    { k: "play", h: "Mesma nota, 4 níveis", b: [["Do fraco ao forte", "fx:dyn"]] },
    { k: "txt", h: "A série harmônica", p: "Uma corda vibra inteira (a **fundamental**, f) e também em metades (2f), terços (3f), quartos (4f)… Esses **harmônicos** formam: oitava, quinta, quarta, terça maior… A série harmônica é a **física por trás da harmonia**." },
    { k: "play", h: "Ouça os 10 primeiros harmônicos de Dó2", b: [["Série harmônica", "fx:harm"]], after: "Dó, Dó, Sol, Dó, Mi, Sol, (Si♭), Dó, Ré, Mi… o acorde maior está “escondido” na natureza." },
    { k: "txt", h: "Timbre e envelope (ADSR)", p: "Timbre = **receita de harmônicos** + **envelope**: **A**taque (como começa), **D**ecaimento, **S**ustentação e **R**elaxamento (como termina). Um piano tem ataque rápido e decai; um órgão sustenta igual enquanto a tecla está apertada." },
    { k: "play", h: "Ondas com receitas diferentes", b: [["Senoide", "fx:wave:sine"], ["Triangular", "fx:wave:triangle"], ["Quadrada", "fx:wave:square"], ["Dente de serra", "fx:wave:sawtooth"]] },
    { k: "txt", h: "Fase e cancelamento", p: "Duas ondas iguais **em fase** se somam; em **oposição de fase** (180°) se cancelam. Dois microfones captando a mesma fonte a distâncias diferentes criam cancelamentos em certas frequências (**filtro pente**): o som fica “oco”. Por isso existe a regra **3:1** na microfonação." },
    { k: "pick", q: "Qual é o comprimento de onda aproximado de um som de 343 Hz?", o: ["1 metro", "34 cm", "3,4 m", "10 m"], c: 0, e: "343 ÷ 343 = 1 m." },
    { k: "tf", q: "Agudos atravessam paredes com mais facilidade que graves.", a: false, e: "É o contrário: graves têm ondas enormes e atravessam/contornam obstáculos. Por isso você ouve o “tum-tum” do vizinho." },
    { k: "mentor", t: "O tabernáculo tinha medidas exatas. **O som também tem medidas**: quem as conhece serve melhor a música." }
  ],
  q: [
    { q: "A velocidade do som no ar (~20 °C) é cerca de:", o: ["343 m/s", "3.430 m/s", "34 m/s", "300.000 km/s"], c: 0, e: "A luz é que é 300.000 km/s." },
    { q: "Um som de 100 Hz tem comprimento de onda de cerca de:", o: ["3,4 m", "34 cm", "3,4 cm", "34 m"], c: 0, e: "343 ÷ 100." },
    { q: "+10 dB é percebido aproximadamente como:", o: ["O dobro do volume", "10 vezes mais alto", "Igual", "A metade"], c: 0, e: "Percepção logarítmica." },
    { q: "Os harmônicos de uma nota são:", o: ["Múltiplos inteiros da fundamental", "Notas aleatórias", "Sempre mais graves", "Só existem no piano"], c: 0, e: "2f, 3f, 4f…" },
    { q: "No ADSR, o “A” é:", o: ["Ataque", "Amplitude", "Afinação", "Acorde"], c: 0, e: "Como o som começa." },
    { q: "Duas ondas iguais em oposição de fase:", o: ["Se cancelam", "Dobram de volume", "Mudam de altura", "Viram outra nota"], c: 0, e: "180°." },
    { q: "Ouça: que forma de onda?", play: "fx:wave:sine", o: ["Senoide", "Dente de serra", "Quadrada", "Triangular"], c: 0, e: "Só a fundamental: som “puro”, como um assobio." }
  ]
};
LESSONS.s2 = {
  title: "Gravação: do microfone ao arquivo", sub: "Cadeia de sinal, microfones, padrões polares, ganho, digital e latência",
  goals: ["Montar uma cadeia de gravação", "Escolher microfone e posição", "Ajustar ganho sem clipar e entender amostragem e latência"],
  mis: "Grave sua voz no celular a 3 distâncias (5 cm, 20 cm, 1 m) e de frente/de lado. Ouça com fone e anote: onde há mais grave (efeito de proximidade)? Onde há mais sala?",
  deep: ["Teorema de Nyquist–Shannon: para representar uma frequência, a taxa de amostragem precisa ser mais que o dobro dela. 44,1 kHz cobre até ~22 kHz.", "Cada bit acrescenta cerca de 6 dB de faixa dinâmica: 16 bits ≈ 96 dB; 24 bits ≈ 144 dB (teórico)."],
  s: [
    { k: "txt", h: "A cadeia de sinal", p: "**Fonte** (voz, violão) → **microfone** → **pré-amplificador** (ganho) → **conversor A/D** → **computador/DAW** → conversor D/A → **monitores ou fones**. O som só é tão bom quanto o elo mais fraco, e o primeiro elo é **a fonte e a sala**." },
    { k: "table", h: "Tipos de microfone", head: ["Tipo", "Características", "Uso típico"], rows: [["Dinâmico", "Robusto, aguenta muita pressão, menos sensível", "Palco, voz ao vivo, caixa, amplificador"], ["Condensador", "Sensível, detalhado; precisa de phantom power (48 V)", "Estúdio, voz gravada, violão, ambiente"], ["Fita (ribbon)", "Som suave e natural; frágil", "Metais, guitarras, vozes em estúdio"]] },
    { k: "txt", h: "Padrões polares", p: "**Cardioide**: capta a frente, rejeita atrás (o mais comum). **Supercardioide/hipercardioide**: mais estreito, um pouco de captação atrás. **Omnidirecional**: capta em todas as direções. **Figura 8**: frente e trás.\nNo palco: monitor **atrás** do cardioide (180°) ou a ~120° do supercardioide." },
    { k: "txt", h: "Efeito de proximidade", p: "Quanto mais perto de um microfone direcional, **mais grave**. Cantores usam isso para dar corpo (chegar perto) ou aliviar (afastar). Se a voz “estoura” nos graves, afaste um pouco ou use passa-altas." },
    { k: "txt", h: "Ganho: o primeiro botão", p: "**Gain staging** é acertar o nível em cada etapa. No digital, **0 dBFS é o teto**: acima disso a onda é cortada (**clipping**). Grave em 24 bits com picos por volta de **−12 a −6 dBFS**: sobra espaço e não há ruído." },
    { k: "play", h: "Ouça o clipping", b: [["Limpo → clipado", "fx:clip"]], after: "A segunda vez tem o topo da onda cortado: distorção áspera que não sai mais depois de gravada." },
    { k: "txt", h: "Áudio digital", p: "**Taxa de amostragem**: quantas “fotos” por segundo (44,1 kHz no CD, 48 kHz em vídeo). Precisa ser **mais que o dobro** da maior frequência (Nyquist). **Profundidade de bits**: a precisão de cada foto. 16 bits ≈ 96 dB de faixa dinâmica; 24 bits ≈ 144 dB (teórico)." },
    { k: "txt", h: "Latência e buffer", p: "Latência é o atraso entre tocar e ouvir. **Buffer menor = menos atraso, mais esforço da CPU.** Para tocar ao vivo (como um piano virtual), mire **abaixo de ~10 ms**. No Windows, drivers ASIO costumam dar a menor latência." },
    { k: "txt", h: "Direct box (DI)", p: "Violão, baixo e teclado têm sinal de **alta impedância e desbalanceado**: em cabos longos, perde agudos e pega ruído. A **DI** converte em sinal **balanceado** de microfone, que vai limpo até a mesa." },
    { k: "match", h: "Ligue o equipamento à função", pairs: [["Pré-amplificador", "Dá ganho ao sinal do microfone"], ["Phantom power", "Alimenta o condensador (48 V)"], ["Conversor A/D", "Transforma o sinal em números"], ["Direct box", "Balanceia o sinal do instrumento"], ["Buffer", "Define a latência"]] },
    { k: "chat", who: "Dani, da mídia", c: "#1864AB", q: "Na gravação do culto, a voz do pastor ficou toda distorcida nos momentos fortes 😟", o: [
      { t: "O ganho estava alto e clipou. Baixa o ganho no pré para os picos ficarem perto de −12/−6 dBFS. Se precisar, um compressor depois, nunca no lugar do ganho certo.", v: 2, r: "Vou ajustar no soundcheck com ele falando forte.", tip: "Gain staging no lugar certo resolve na origem." },
      { t: "É só abaixar o volume na edição depois.", v: 0, r: "Abaixei, mas a distorção continua lá…", tip: "Clipping fica gravado: abaixar não remove." },
      { t: "Coloca mais reverb pra disfarçar.", v: 0, r: "Agora tá distorcido E distante 😅", tip: "Efeito não conserta sinal ruim." }] },
    { k: "mentor", t: "Bezalel conferia cada peça antes de montar o tabernáculo. **Confira cada etapa da cadeia** antes de apertar o REC." }
  ],
  q: [
    { q: "Qual microfone precisa de phantom power?", o: ["Condensador", "Dinâmico", "Nenhum", "Todos"], c: 0, e: "48 V." },
    { q: "O efeito de proximidade aumenta:", o: ["Os graves", "Os agudos", "A latência", "O ruído digital"], c: 0, e: "Perto de mics direcionais." },
    { q: "Num cardioide, o monitor de palco deve ficar:", o: ["Atrás do microfone (180°)", "Na frente do microfone", "Em cima da cabeça do cantor", "Tanto faz"], c: 0, e: "Onde ele rejeita mais." },
    { q: "Na gravação em 24 bits, picos de referência ficam em torno de:", o: ["−12 a −6 dBFS", "0 dBFS sempre", "+6 dBFS", "−60 dBFS"], c: 0, e: "Espaço de segurança." },
    { q: "Para captar até 20 kHz, a taxa de amostragem precisa ser:", o: ["Mais que 40 kHz", "20 kHz", "10 kHz", "Qualquer uma"], c: 0, e: "Nyquist: mais que o dobro." },
    { q: "Buffer menor resulta em:", o: ["Menos latência e mais uso de CPU", "Mais latência", "Melhor timbre", "Menos CPU"], c: 0, e: "Troca entre atraso e processamento." },
    { q: "A DI serve para:", o: ["Balancear o sinal de instrumentos", "Afinar o violão", "Gravar MIDI", "Aumentar o reverb"], c: 0, e: "Sinal limpo em cabos longos." }
  ]
};
LESSONS.s3 = {
  title: "Mixagem: EQ, compressão e espaço", sub: "Volume, pan, faixas de frequência, EQ subtrativa, compressor, reverb e referência",
  goals: ["Conhecer as faixas de frequência e o que cada uma faz", "Usar EQ subtrativa e passa-altas", "Ajustar um compressor e usar reverb com moderação"],
  mis: "Abra o Laboratório → Mesa de mixagem e faça os 4 experimentos. Depois treine 2 rodadas em Academia → Ouvido de engenheiro.",
  deep: ["Ordem prática de mixagem: organização → volumes e pan → EQ (corte) → compressão → efeitos de espaço → automação → referência.", "O ouvido humano é mais sensível entre ~2 e 5 kHz (curvas de igual intensidade): pequenas mudanças ali soam grandes."],
  s: [
    { k: "txt", h: "O que é mixar", p: "Mixar é **equilibrar** todas as faixas para que a música soe clara e emocionante: **volume, frequência, dinâmica, espaço e tempo**. Não é “colocar efeito”: é decidir o que cada som precisa para servir a canção." },
    { k: "txt", h: "Comece pelo simples", p: "**80% da mixagem é volume e pan.** Antes de qualquer plugin, acerte os níveis e distribua no estéreo (bateria e baixo no centro; violões, teclados e backings para os lados). Confira em **mono**: se algo some, há problema de fase." },
    { k: "table", h: "Faixas de frequência (toque para ouvir +12 dB)", head: ["Faixa", "Região", "Caráter"], rows: [["Sub/grave", "20–150 Hz", "Peso: bumbo, baixo"], ["Médio-grave", "150–500 Hz", "Corpo; em excesso, embolado"], ["Médio", "500 Hz–2 kHz", "Presença, “nasal”, telefone"], ["Médio-agudo", "2–6 kHz", "Clareza, ataque; em excesso, agressivo"], ["Agudo", "6–20 kHz", "Brilho, ar, sibilância (s)"]],
      play: ["fx:eq:100", "fx:eq:300", "fx:eq:1000", "fx:eq:3500", "fx:eq:9000"] },
    { k: "ear", q: "Qual faixa recebeu reforço na segunda metade?", play: "fx:eq:250", o: ["~250 Hz (embolado)", "~3 kHz (presença)", "~9 kHz (ar)"], c: 0, e: "Som “abafado, de caixa”: médio-grave." },
    { k: "txt", h: "EQ subtrativa", p: "**Corte antes de reforçar.** Se a voz está abafada, em vez de subir agudos, tente **cortar 2–4 dB em 250–400 Hz**. Use **passa-altas** em quase tudo que não é grave (vozes, violões, pratos): limpa a lama sem que ninguém perceba." },
    { k: "txt", h: "Mascaramento", p: "Dois sons na mesma faixa disputam espaço e um esconde o outro. Solução: **abrir janelas** com EQ. Ex.: corte um pouco do violão em 2–4 kHz para a voz aparecer, em vez de subir o volume da voz." },
    { k: "txt", h: "O compressor", p: "Reduz a diferença entre partes fortes e fracas.\n• **Limiar (threshold)**: a partir de onde atua.\n• **Razão (ratio)**: quanto reduz (4:1 = cada 4 dB acima vira 1).\n• **Ataque**: rapidez para agir.\n• **Relaxamento (release)**: rapidez para soltar.\n• **Ganho de compensação**: devolve o volume." },
    { k: "txt", h: "Espaço: reverb e delay", p: "**Reverb** coloca o som numa sala; **delay** repete o som. Pouco reverb aproxima, muito afasta. Ao vivo, numa igreja que já tem reverberação, **use pouco**: a sala já faz o trabalho. O **pre-delay** separa a voz do reverb e mantém a clareza." },
    { k: "txt", h: "Referência e tradução", p: "Compare sua mix com músicas bem mixadas **no mesmo volume** (o mais alto sempre parece melhor). Ouça em vários sistemas: fone, carro, caixa de celular. Uma boa mix **traduz** bem em todos." },
    { k: "pick", q: "A voz está abafada. O primeiro movimento recomendado é:", o: ["Cortar um pouco em 250–400 Hz", "Subir 12 dB em 10 kHz", "Colocar mais reverb", "Aumentar o volume geral"], c: 0, e: "EQ subtrativa no embolado." },
    { k: "tf", q: "Numa mixagem, comparar com uma referência mais alta é justo, porque mais alto é melhor.", a: false, e: "Iguale o volume: o mais alto sempre engana o ouvido." },
    { k: "mentor", t: "Ao fazer as cortinas do tabernáculo, cada fio tinha seu lugar. **Na mixagem, cada som precisa do seu espaço.** Vá para a **mesa do Laboratório** e mexa!" }
  ],
  q: [
    { q: "Qual faixa costuma deixar a mix “embolada” em excesso?", o: ["150–500 Hz", "8–12 kHz", "20–40 Hz", "16–20 kHz"], c: 0, e: "Médio-grave." },
    { q: "EQ subtrativa significa:", o: ["Cortar antes de reforçar", "Reforçar tudo", "Tirar o EQ", "Subir o volume"], c: 0, e: "Menos é mais." },
    { q: "Num compressor, ratio 4:1 significa:", o: ["4 dB acima do limiar viram 1 dB", "O volume dobra", "O ataque é de 4 ms", "Quatro bandas de EQ"], c: 0, e: "Redução proporcional." },
    { q: "O passa-altas numa voz serve para:", o: ["Limpar graves desnecessários", "Aumentar os graves", "Criar reverb", "Afinar"], c: 0, e: "Ruído de manuseio, palco, lama." },
    { q: "Ao vivo numa igreja reverberante, o reverb da mesa deve ser:", o: ["Usado com moderação", "No máximo", "Sempre desligado em tudo", "Só no bumbo"], c: 0, e: "A sala já soma reverb." },
    { q: "Ouça: que faixa foi reforçada?", play: "fx:eq:9000", o: ["Agudo (~9 kHz)", "Grave (~100 Hz)", "Médio (~1 kHz)", "Médio-grave (~300 Hz)"], c: 0, e: "Brilho e “ar”." },
    { q: "Mascaramento é:", o: ["Um som escondendo outro na mesma faixa", "Um efeito de voz", "Usar máscara no estúdio", "Um tipo de microfone"], c: 0, e: "Disputa de frequências." }
  ]
};
LESSONS.s4 = {
  title: "Masterização, acústica e som ao vivo", sub: "Loudness em LUFS, limitador, acústica da sala, microfonia e saúde auditiva",
  goals: ["Entender o que a masterização faz e o que é LUFS", "Saber o básico de acústica de sala", "Prevenir microfonia e cuidar da audição da equipe"],
  mis: "Faça um checklist de som ao vivo para a sua igreja: posição dos monitores, passa-altas nas vozes, ganho no soundcheck, nível máximo em dB, protetor auricular para a bateria. Mostre para a equipe.",
  deep: ["Loudness integrado é medido em LUFS (ITU-R BS.1770 / EBU R128). Plataformas de streaming normalizam a reprodução, muitas em torno de −14 LUFS; por isso “masterizar alto demais” só tira a dinâmica.", "RT60 é o tempo para o som cair 60 dB depois que a fonte para. Salas de fala precisam de RT60 curto; igrejas grandes com superfícies duras podem ter RT60 de vários segundos."],
  s: [
    { k: "txt", h: "Masterização", p: "A última etapa: deixar a música (ou o álbum) **coesa e pronta** para ser ouvida em qualquer lugar. EQ geral sutil, compressão leve, **limitador**, ordem das faixas, pausas, metadados e formatos de entrega." },
    { k: "txt", h: "Loudness e LUFS", p: "Volume percebido se mede em **LUFS**. As plataformas **normalizam**: se sua música estiver muito alta, elas abaixam. Masterizar esmagado (a “guerra do volume”) só tira a dinâmica. Referências comuns: em torno de **−14 LUFS** integrado para streaming e **true peak até −1 dBTP**." },
    { k: "txt", h: "Dither", p: "Ao reduzir de 24 para 16 bits, adiciona-se um ruído baixíssimo (**dither**) para evitar distorção de quantização nos trechos suaves. Só se aplica **uma vez**, no final." },
    { k: "txt", h: "Acústica da sala", p: "Três ferramentas:\n• **Absorção** (espumas, lã de rocha, cortinas): tira reflexões; boa para médios e agudos.\n• **Difusão**: espalha as reflexões sem matá-las.\n• **Bass traps**: absorvedores grossos nos cantos para os graves.\nO **RT60** mede a reverberação: quanto maior, mais a palavra embola." },
    { k: "txt", h: "Modos de sala", p: "Em salas fechadas, alguns **graves se acumulam** em certas frequências e posições (ondas estacionárias) por causa das dimensões. Resultado: num lugar o baixo está enorme, um metro depois some. Evite ouvir e mixar no **meio exato** da sala ou colado na parede." },
    { k: "txt", h: "Microfonia (feedback)", p: "O som da caixa volta no microfone, é amplificado de novo e entra em laço: o **apito**. Prevenção:\n• Caixas de PA **à frente** dos microfones.\n• Microfone **perto** da fonte (menos ganho).\n• **Padrão polar** certo e monitor no ponto cego.\n• EQ cortando a frequência que apita.\n• Ganho controlado, sem exageros." },
    { k: "ear", q: "Uma microfonia aguda está apitando. Qual faixa provavelmente precisa de corte estreito?", play: "fx:eq:3000", o: ["Médio-agudo (~2–4 kHz)", "Sub-grave (~40 Hz)", "Médio-grave (~250 Hz)"], c: 0, e: "Microfonias comuns em vozes ficam nos médios e médio-agudos; corte estreito (Q alto) só na frequência que apita." },
    { k: "txt", h: "Monitores e in-ear", p: "Monitores de chão (retornos) são a principal fonte de microfonia e de volume no palco. O **in-ear** (fone no ouvido) reduz o volume do palco, melhora a afinação dos cantores e permite clique para a banda." },
    { k: "txt", h: "Saúde auditiva", p: "Perda auditiva por ruído é **permanente** e silenciosa. Use **protetor auricular** de músico (atenuação plana), faça pausas, controle o nível da igreja com um decibelímetro e cuide da equipe: o baterista e o técnico de som são os mais expostos." },
    { k: "order", h: "Ordem de uma produção musical", items: ["Composição", "Arranjo e pré-produção", "Gravação", "Edição", "Mixagem", "Masterização", "Distribuição"] },
    { k: "chat", who: "Tio Zé, da mesa de som", c: "#2B8A3E", q: "Toda vez que a cantora chega perto da caixa, apita! Já baixei tudo e não resolve 😫", o: [
      { t: "Vamos ver a posição: as caixas do PA precisam ficar à frente dos mics e o retorno no ponto cego do microfone. Depois, achamos a frequência que apita e cortamos só ela no EQ.", v: 2, r: "Mudei o retorno de lugar e já melhorou muito!", tip: "Posição primeiro, EQ estreito depois." },
      { t: "Coloca mais reverb na voz.", v: 0, r: "Agora apita com eco 😂", tip: "Reverb não resolve realimentação." },
      { t: "Pede pra ela não chegar perto da caixa.", v: 1, r: "Ajuda, mas ela anda no palco…", tip: "É paliativo; o certo é posicionar e equalizar." }] },
    { k: "mentor", t: "Você terminou a trilha da **Escola Harpa**: teoria, ouvido, prática, harmonia, composição, história e engenharia. Agora, **sirva com excelência**: “Cantai-lhe um cântico novo; tocai bem” (Sl 33.3)." }
  ],
  q: [
    { q: "LUFS mede:", o: ["Loudness (volume percebido)", "Latência", "Frequência", "Afinação"], c: 0, e: "Volume integrado percebido." },
    { q: "Se uma música for masterizada muito alta, a plataforma de streaming:", o: ["Abaixa o volume na normalização", "Deixa mais alta ainda", "Recusa o arquivo", "Aumenta os graves"], c: 0, e: "E a dinâmica perdida não volta." },
    { q: "Bass traps servem para:", o: ["Controlar graves nos cantos da sala", "Aumentar os agudos", "Proteger o baixo elétrico", "Gravar MIDI"], c: 0, e: "Absorvedores grossos." },
    { q: "RT60 mede:", o: ["O tempo de reverberação da sala", "O volume máximo", "A latência", "O ganho do microfone"], c: 0, e: "Queda de 60 dB." },
    { q: "Qual medida previne microfonia?", o: ["Caixas de PA à frente dos microfones", "Mais ganho no microfone", "Monitor apontado para a frente do mic", "Mais reverb"], c: 0, e: "E retorno no ponto cego." },
    { q: "Dither é aplicado:", o: ["Uma vez, ao reduzir a profundidade de bits no final", "Em cada faixa da mix", "Só ao vivo", "Antes de gravar"], c: 0, e: "Na exportação final." },
    { q: "A etapa que deixa o álbum coeso e pronto para distribuição é a:", o: ["Masterização", "Mixagem", "Gravação", "Composição"], c: 0, e: "A última antes de distribuir." }
  ]
};
