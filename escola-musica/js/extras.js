// Figurinhas (3 por lição + lendária de cada mestre), frases do dia e o teste vocacional.
"use strict";
const FCAT = { Conceito: ["#5F3DC4", "#9775FA"], Pessoa: ["#E8590C", "#F59F00"], Obra: ["#C2255C", "#F06595"], Som: ["#1864AB", "#4DABF7"], Ferramenta: ["#0B7285", "#3BC9DB"], "Bíblia": ["#2B8A3E", "#69DB7C"] };
// [nome, categoria, descrição] — 1ª: stories · 2ª: prova aprovada · 3ª: nota máxima
const FIGS = {
  t1: [["As 4 propriedades", "Conceito", "Altura, duração, intensidade e timbre."], ["Lá 440", "Som", "A nota de referência da afinação moderna."], ["Os 12 semitons", "Conceito", "Sete naturais e cinco alterados."]],
  t2: [["T T S T T T S", "Conceito", "A fórmula da escala maior."], ["A relativa menor", "Conceito", "Mesmas notas, outro centro."], ["Círculo das quintas", "Ferramenta", "O mapa de todas as tonalidades."]],
  t3: [["A quinta justa", "Som", "3:2, o intervalo mais estável depois da oitava."], ["O trítono", "Som", "Seis semitons: o motor da tensão."], ["Inversão soma 9", "Conceito", "3ª vira 6ª, 2ª vira 7ª."]],
  t4: [["Clave de sol", "Ferramenta", "A espiral abraça o Sol."], ["Semínima", "Conceito", "Um tempo no 4/4."], ["Compasso 6/8", "Conceito", "Dois tempos de três colcheias."]],
  p1: [["Ouvido relativo", "Conceito", "Ouvir relações, não nomes."], ["Dó móvel", "Ferramenta", "A tônica é sempre 1."], ["A harpa que acalma", "Bíblia", "Davi tocava e Saul se acalmava (1 Sm 16.23)."]],
  p2: [["Terça maior", "Som", "Quatro semitons de luz."], ["Terça menor", "Som", "Três semitons de sombra."], ["Intervalo harmônico", "Conceito", "Duas notas ao mesmo tempo."]],
  p3: [["Tônica", "Conceito", "A casa."], ["Dominante", "Conceito", "A tensão que volta para casa."], ["Siga o baixo", "Ferramenta", "O segredo para tirar música de ouvido."]],
  p4: [["Série harmônica", "Som", "A natureza toca um acorde."], ["Cent", "Conceito", "Um centésimo de semitom."], ["Batimento", "Som", "O tremor que some quando afina."]],
  r1: [["Pulso", "Conceito", "A batida que o pé marca."], ["Metrônomo", "Ferramenta", "O professor que nunca acelera."], ["Backbeat", "Som", "A caixa no 2 e no 4."]],
  r2: [["Síncope", "Conceito", "O acento que escorrega."], ["Tresillo", "Som", "3 + 3 + 2."], ["Luiz Gonzaga", "Pessoa", "O Rei do Baião."]],
  r3: [["Prática deliberada", "Conceito", "Meta, foco e feedback."], ["Lento e perfeito", "Ferramenta", "Devagar se vai longe."], ["Respiração", "Conceito", "O apoio da voz."]],
  r4: [["Dinâmica", "Conceito", "Do pp ao ff."], ["Legato", "Som", "Notas ligadas."], ["O tamborim de Miriã", "Bíblia", "“Cantai ao Senhor” (Êx 15.21)."]],
  h1: [["Tríade", "Conceito", "Fundamental, terça e quinta."], ["Cifra com barra", "Ferramenta", "C/E: Dó com Mi no baixo."], ["Os címbalos de Asafe", "Bíblia", "Asafe tocava os címbalos (1 Cr 16.5)."]],
  h2: [["Campo harmônico", "Conceito", "Os sete acordes da casa."], ["M m m M M m °", "Conceito", "O padrão do tom maior."], ["Números de Nashville", "Ferramenta", "Transpor sem pensar duas vezes."]],
  h3: [["Cadência plagal", "Som", "O “Amém”."], ["Dominante secundária", "Conceito", "O V de qualquer acorde."], ["Empréstimo modal", "Som", "O iv menor que dá saudade."]],
  h4: [["Notas-guia", "Conceito", "A 3ª e a 7ª mandam."], ["SubV", "Ferramenta", "D♭7 no lugar de G7."], ["Diminuto de passagem", "Som", "C – C♯° – Dm."]],
  c1: [["Motivo", "Conceito", "Poucas notas, muita identidade."], ["Ta-ta-ta-tááá", "Obra", "A 5ª Sinfonia de Beethoven."], ["Prosódia", "Conceito", "A letra manda no ritmo."]],
  c2: [["Forma ABA", "Conceito", "Sai e volta."], ["Refrão", "Conceito", "O gancho que todos cantam."], ["Forma sonata", "Conceito", "Exposição, desenvolvimento e reexposição."]],
  c3: [["SATB", "Conceito", "Soprano, contralto, tenor e baixo."], ["Movimento contrário", "Som", "Uma sobe, outra desce."], ["Orquestra", "Ferramenta", "Cordas, madeiras, metais e percussão."]],
  c4: [["Expectativa", "Conceito", "Prever, adiar, surpreender."], ["Leitmotiv", "Conceito", "O tema do personagem."], ["O silêncio", "Som", "A pausa que diz tudo."]],
  hi1: [["Flauta de osso", "Obra", "~40 mil anos de música."], ["Canto gregoriano", "Som", "Uma linha só, em latim."], ["Guido d'Arezzo", "Pessoa", "Ut, ré, mi, fá, sol, lá."]],
  hi2: [["Martinho Lutero", "Pessoa", "O povo cantando na própria língua."], ["Messias", "Obra", "Handel e o coro “Aleluia”."], ["J. S. Bach", "Pessoa", "Soli Deo Gloria."]],
  hi3: [["Mozart", "Pessoa", "Mais de 600 obras."], ["Ode à Alegria", "Obra", "Beethoven, 9ª Sinfonia."], ["Villa-Lobos", "Pessoa", "As Bachianas Brasileiras."]],
  hi4: [["Pixinguinha", "Pessoa", "O mestre do choro."], ["Chega de Saudade", "Obra", "O marco da bossa nova (1958)."], ["Harpa Cristã", "Obra", "O hinário de 1922."]],
  s1: [["343 m/s", "Som", "A velocidade do som no ar."], ["Decibel", "Ferramenta", "A régua logarítmica do volume."], ["ADSR", "Conceito", "Ataque, decaimento, sustentação, relaxamento."]],
  s2: [["Microfone dinâmico", "Ferramenta", "O guerreiro do palco."], ["Cardioide", "Conceito", "Ouve pela frente, ignora atrás."], ["Nyquist", "Conceito", "Mais que o dobro da maior frequência."]],
  s3: [["Equalizador", "Ferramenta", "Esculpir as frequências."], ["Compressor", "Ferramenta", "Domar os picos."], ["Referência", "Conceito", "Compare no mesmo volume."]],
  s4: [["LUFS", "Conceito", "O volume que o ouvido sente."], ["Bass trap", "Ferramenta", "Caçador de graves."], ["Sem microfonia", "Conceito", "Caixa à frente, monitor no ponto cego."]]
};
const FCOND = ["Termine os stories da lição", "Aprove a prova da lição", "Tire nota máxima na prova", "Chegue a Mestre com o mestre do nível"];

const QUOTES = [
  ["Cantai-lhe um cântico novo; tocai bem e com júbilo.", "Salmo 33.3"],
  ["O objetivo e a finalidade de toda música não deveria ser outro senão a glória de Deus e o refrigério da alma.", "Atribuído a J. S. Bach"],
  ["Não há nada de notável nisso. Basta tocar as teclas certas na hora certa, e o instrumento toca sozinho.", "Atribuído a J. S. Bach"],
  ["Depois da teologia, dou à música o lugar mais alto e a maior honra.", "Atribuído a Martinho Lutero"],
  ["Tudo quanto tem fôlego louve ao Senhor.", "Salmo 150.6"],
  ["Falando entre vós com salmos, hinos e cânticos espirituais, cantando e salmodiando ao Senhor no vosso coração.", "Efésios 5.19"],
  ["E Quenanias, chefe dos levitas no canto, ensinava o canto, porque era entendido.", "1 Crônicas 15.22"]
];

// Teste: “Qual caminho na música combina com você?” — 5 cursos do guia
const VOC = {
  areas: {
    bach: { n: "Bacharelado (intérprete)", c1: "#6D3FE0", c2: "#B794FF", d: "Você se realiza tocando ou cantando, buscando excelência técnica e expressiva. O bacharelado em instrumento ou canto aprofunda repertório, técnica e performance.", foco: ["Nível 3 · Ritmo e prática", "Academia: intervalos e afinação", "Metrônomo todo dia"], prof: "Instrumentista, cantor, camerista, músico de orquestra ou de estúdio." },
    lic: { n: "Licenciatura (educador)", c1: "#2B8A3E", c2: "#8CE99A", d: "Você gosta de explicar e ver o outro aprender. A licenciatura forma para ensinar música em escolas, projetos e igrejas, com didática e psicologia da educação.", foco: ["Nível 1 · Teoria", "Nível 2 · Percepção", "Guia: plano de estudo"], prof: "Professor de música na escola, em projetos sociais, aulas particulares, ministério de ensino na igreja." },
    comp: { n: "Composição", c1: "#B7791F", c2: "#F6D365", d: "Você ouve músicas que ainda não existem. A composição estuda harmonia, contraponto, orquestração e criação para concerto, canções e audiovisual.", foco: ["Nível 4 · Harmonia", "Nível 5 · Composição e arranjo", "Laboratório: campo harmônico"], prof: "Compositor, arranjador, trilhas para filmes, séries, publicidade e jogos." },
    reg: { n: "Regência", c1: "#C2255C", c2: "#F783AC", d: "Você enxerga o grupo inteiro e gosta de conduzir pessoas. A regência prepara para liderar corais, bandas e orquestras: gesto, leitura de grade e ensaio.", foco: ["Nível 3 · Tocar em grupo", "Nível 5 · Arranjo e vozes", "Nível 6 · História"], prof: "Regente de coral e orquestra, diretor musical, líder de louvor, maestro de banda." },
    prod: { n: "Produção musical", c1: "#0B7285", c2: "#66D9E8", d: "Você se interessa pelo som em si: como gravar, mixar e fazer soar bem. A produção une tecnologia, acústica e decisões artísticas.", foco: ["Nível 7 · Produção", "Academia: ouvido de engenheiro", "Laboratório: mesa de mixagem"], prof: "Produtor musical, técnico de gravação, mixagem e masterização, som ao vivo, áudio para vídeo e jogos." }
  },
  q: [
    ["Num culto ou show, o que mais chama sua atenção?", [["O músico que toca muito bem", "bach"], ["Como a banda se organiza e se comunica", "reg"], ["O arranjo e as escolhas das músicas", "comp"], ["O som: se está claro, alto, embolado", "prod"], ["Quem está aprendendo e precisa de ajuda", "lic"]]],
    ["Um sábado livre ideal seria…", [["Horas estudando meu instrumento", "bach"], ["Gravar e editar uma música no computador", "prod"], ["Escrever uma música nova", "comp"], ["Dar aula para alguém começar a tocar", "lic"], ["Ensaiar um grupo vocal", "reg"]]],
    ["Quando escuta uma música nova, você…", [["Tenta tocar junto na hora", "bach"], ["Repara nos timbres e na mixagem", "prod"], ["Pensa em como mudaria a harmonia", "comp"], ["Imagina como ensinaria a alguém", "lic"], ["Repara em quem entra e sai em cada parte", "reg"]]],
    ["Qual elogio te deixaria mais feliz?", [["“Que execução impecável!”", "bach"], ["“Graças a você eu aprendi.”", "lic"], ["“Essa música é sua? Linda!”", "comp"], ["“O som nunca esteve tão bom.”", "prod"], ["“O grupo nunca soou tão unido.”", "reg"]]],
    ["Num ensaio que está dando errado, você naturalmente…", [["Estuda sua parte até ficar perfeita", "bach"], ["Para tudo e organiza quem faz o quê", "reg"], ["Reescreve a parte que não funciona", "comp"], ["Ajusta os volumes e retornos", "prod"], ["Explica com paciência para quem está perdido", "lic"]]],
    ["Que tipo de prova você prefere?", [["Tocar uma peça difícil para uma banca", "bach"], ["Apresentar uma aula", "lic"], ["Entregar uma composição original", "comp"], ["Mixar uma faixa", "prod"], ["Conduzir um coro por 10 minutos", "reg"]]],
    ["Qual ferramenta você levaria para uma ilha deserta?", [["Meu instrumento", "bach"], ["Papel pautado e lápis", "comp"], ["Notebook com fones", "prod"], ["Uma batuta", "reg"], ["Um quadro branco", "lic"]]],
    ["O que te irrita mais?", [["Errar uma passagem que eu sei", "bach"], ["Um som embolado ou com microfonia", "prod"], ["Uma harmonia previsível demais", "comp"], ["Cada músico tocando para um lado", "reg"], ["Alguém desistir achando que não tem dom", "lic"]]],
    ["Que parte do app você mais curtiu até agora?", [["Metrônomo e treino de ritmo", "bach"], ["Mesa de mixagem e ouvido de engenheiro", "prod"], ["Campo harmônico e rearmonização", "comp"], ["Lições sobre tocar em grupo e regência", "reg"], ["Explicações passo a passo da teoria", "lic"]]],
    ["Daqui a 10 anos, você se vê…", [["No palco ou num estúdio tocando", "bach"], ["Numa sala de aula ou escola de música", "lic"], ["Com obras e trilhas suas por aí", "comp"], ["À frente de um coral ou orquestra", "reg"], ["Num estúdio de gravação próprio", "prod"]]]
  ]
};
