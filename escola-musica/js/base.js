// Mestres, níveis da trilha, cards e o guia da faculdade.
"use strict";
const MENTORS = {
  quenanias: { name: "Quenanias", ref: "1 Crônicas 15.22", emoji: "📜", c1: "#6D3FE0", c2: "#B794FF",
    evo: ["Você já lê o mapa: notas, tons e semitons.", "Escalas e intervalos já fazem sentido para você.", "Você domina a gramática da música. Como eu: entendido no canto."] },
  davi: { name: "Davi", ref: "1 Samuel 16.23", emoji: "🎵", c1: "#1C7ED6", c2: "#66D9E8",
    evo: ["Seu ouvido acordou.", "Você já ouve funções e não só notas.", "Ouvido de salmista: você ouve a música por dentro."] },
  miria: { name: "Miriã", ref: "Êxodo 15.20", emoji: "🥁", c1: "#E8590C", c2: "#FFC145",
    evo: ["Seu pulso está firme.", "Você toca junto e faz o grupo soar junto.", "Você conduz o povo com o ritmo, como no mar Vermelho."] },
  asafe: { name: "Asafe", ref: "1 Crônicas 16.5", emoji: "🎼", c1: "#0E9F6E", c2: "#7EE8C1",
    evo: ["Você monta e lê qualquer acorde.", "Tensão e resolução já são suas ferramentas.", "Mestre da harmonia: as notas conversam nas suas mãos."] },
  salomao: { name: "Salomão", ref: "1 Reis 4.32", emoji: "👑", c1: "#B7791F", c2: "#F6D365",
    evo: ["Você já cria motivos e frases.", "Forma e arranjo agora são planejados, não acidentais.", "Compositor: tempo para tudo, inclusive para o silêncio."] },
  jubal: { name: "Jubal", ref: "Gênesis 4.21", emoji: "⏳", c1: "#9C36B5", c2: "#E599F7",
    evo: ["Você conhece as raízes.", "Barroco, Clássico e Romântico têm rosto para você.", "Você enxerga a música como uma história de mil anos."] },
  bezalel: { name: "Bezalel", ref: "Êxodo 31.3", emoji: "🛠️", c1: "#0B7285", c2: "#4DD4E8",
    evo: ["Você entende o som como física.", "Microfone, ganho e EQ deixaram de ser mistério.", "Engenheiro de som: precisão a serviço da beleza."] }
};
const TRACKS = [
  { n: 1, id: "teoria", mk: "quenanias", title: "Teoria musical", short: "Teoria", desc: "Notas, escalas, intervalos e partitura: a gramática da música.", mods: ["t1", "t2", "t3", "t4"] },
  { n: 2, id: "percepcao", mk: "davi", title: "Percepção e treinamento auditivo", short: "Ouvido", desc: "Reconhecer pela escuta: intervalos, acordes, funções, timbre e afinação.", mods: ["p1", "p2", "p3", "p4"] },
  { n: 3, id: "pratica", mk: "miria", title: "Ritmo e prática instrumental", short: "Prática", desc: "Pulso, levadas, técnica de estudo, tocar em grupo e interpretar.", mods: ["r1", "r2", "r3", "r4"] },
  { n: 4, id: "harmonia", mk: "asafe", title: "Harmonia", short: "Harmonia", desc: "Acordes, campo harmônico, tensão e resolução, tensões e rearmonização.", mods: ["h1", "h2", "h3", "h4"] },
  { n: 5, id: "composicao", mk: "salomao", title: "Composição e arranjo", short: "Composição", desc: "Melodia, forma, arranjo e a ciência da emoção musical.", mods: ["c1", "c2", "c3", "c4"] },
  { n: 6, id: "historia", mk: "jubal", title: "História da música", short: "História", desc: "Das flautas de osso ao louvor de hoje, passando por Bach, jazz e samba.", mods: ["hi1", "hi2", "hi3", "hi4"] },
  { n: 7, id: "producao", mk: "bezalel", title: "Acústica, tecnologia e produção", short: "Produção", desc: "Física do som, gravação, mixagem, masterização e som ao vivo.", mods: ["s1", "s2", "s3", "s4"] }
];

const CARDS = [
  // Teoria
  ["Teoria", "Semitom", "A menor distância entre duas notas no sistema ocidental: tecla vizinha no piano, casa vizinha no violão.", "C4 C#4"],
  ["Teoria", "Tom (distância)", "Dois semitons. Ex.: Dó → Ré.", "C4 D4"],
  ["Teoria", "Enarmonia", "Mesmo som, nomes diferentes: Dó♯ = Ré♭, Mi♯ = Fá."],
  ["Teoria", "Oitava", "A mesma nota com o dobro (ou metade) da frequência. Lá4 = 440 Hz, Lá5 = 880 Hz.", "A3 A4"],
  ["Teoria", "Fórmula da escala maior", "T – T – S – T – T – T – S", "C4 D4 E4 F4 G4 A4 B4 C5"],
  ["Teoria", "Fórmula da menor natural", "T – S – T – T – S – T – T", "A3 B3 C4 D4 E4 F4 G4 A4"],
  ["Teoria", "Tônica", "1º grau: o centro, o “lar” da tonalidade."],
  ["Teoria", "Dominante", "5º grau: o ponto de maior tensão “estável”, que puxa de volta para a tônica."],
  ["Teoria", "Subdominante", "4º grau: afasta da tônica sem a tensão da dominante."],
  ["Teoria", "Sensível", "7º grau da escala maior, meio tom abaixo da tônica: quer subir para ela.", "B3 C4:2"],
  ["Teoria", "Relativa menor", "Tom menor com a mesma armadura do maior; começa no 6º grau dele. Dó maior ↔ Lá menor."],
  ["Teoria", "Ordem dos sustenidos", "Fá – Dó – Sol – Ré – Lá – Mi – Si (os bemóis são o contrário)."],
  ["Teoria", "Armadura de clave", "Acidentes fixos no começo da pauta que valem para a música toda e indicam a tonalidade."],
  ["Teoria", "Intervalo justo", "Qualidade dos uníssonos, 4ªs, 5ªs e 8ªs na forma “pura”."],
  ["Teoria", "Trítono", "Três tons (6 semitons): 4ª aumentada ou 5ª diminuta. Muito instável.", "[C4 F#4]"],
  ["Teoria", "Inversão de intervalo", "Os números somam 9; maior ↔ menor, justo ↔ justo, aumentado ↔ diminuto."],
  ["Teoria", "Compasso 6/8", "Composto: 2 tempos, cada um dividido em 3 colcheias."],
  ["Teoria", "Ponto de aumento", "Soma metade do valor da figura. Mínima pontuada = 3 tempos."],
  ["Teoria", "Clave de fá (4ª linha)", "Indica que a 4ª linha é o Fá abaixo do Dó central; usada para sons graves."],
  ["Teoria", "Dó central", "Dó4 (cerca de 261,6 Hz), escrito numa linha suplementar entre as claves de sol e fá.", "C4"],
  // Harmonia
  ["Harmonia", "Tríade maior", "Fundamental + 3ª maior + 5ª justa. Ex.: Dó – Mi – Sol.", "[C4 E4 G4]"],
  ["Harmonia", "Tríade menor", "Fundamental + 3ª menor + 5ª justa. Ex.: Lá – Dó – Mi.", "[A3 C4 E4]"],
  ["Harmonia", "Tríade diminuta", "Duas terças menores: Si – Ré – Fá.", "[B3 D4 F4]"],
  ["Harmonia", "Tríade aumentada", "Duas terças maiores: Dó – Mi – Sol♯.", "[C4 E4 G#4]"],
  ["Harmonia", "Campo harmônico maior", "I – ii – iii – IV – V – vi – vii° (maior, menor, menor, maior, maior, menor, diminuto)."],
  ["Harmonia", "Cifra C/E", "Acorde de Dó com Mi no baixo (1ª inversão).", "[E3 C4 G4]"],
  ["Harmonia", "Acorde 7 (dominante)", "Tríade maior + 7ª menor. Ex.: G7 = Sol Si Ré Fá.", "[G3 B3 D4 F4]"],
  ["Harmonia", "Acorde 7M", "Tríade maior + 7ª maior. Ex.: C7M = Dó Mi Sol Si.", "[C4 E4 G4 B4]"],
  ["Harmonia", "Meio-diminuto (ø)", "m7(♭5): Si Ré Fá Lá. O vii do tom maior e o ii do tom menor.", "[B3 D4 F4 A4]"],
  ["Harmonia", "Cadência autêntica", "V → I. A resolução mais forte da música tonal.", "[G3 B3 D4 F4]:2 [C4 E4 G4 C5]:2"],
  ["Harmonia", "Cadência plagal", "IV → I. A do “Amém” no fim dos hinos.", "[F3 A3 C4 F4]:2 [E3 G3 C4 E4]:2"],
  ["Harmonia", "Meia cadência", "Frase que termina no V: soa como uma pergunta."],
  ["Harmonia", "Cadência interrompida", "V → vi: o ouvido espera o I e recebe o relativo menor.", "[G3 B3 D4 F4]:2 [A3 C4 E4 A4]:2"],
  ["Harmonia", "Dominante secundária", "Acorde dominante de um grau que não é a tônica. Ex.: em Dó, A7 → Dm (V7 do ii)."],
  ["Harmonia", "Empréstimo modal", "Usar um acorde do modo homônimo. Ex.: Fm (iv menor) em Dó maior.", "[C4 E4 G4]:2 [F3 Ab3 C4 F4]:2 [C4 E4 G4]:2"],
  ["Harmonia", "Substituição de trítono (subV)", "Trocar um V7 pelo 7 que está a um trítono: D♭7 no lugar de G7."],
  ["Harmonia", "Notas-guia", "3ª e 7ª de um acorde: definem a qualidade e conduzem a harmonia."],
  ["Harmonia", "Trítono do V7", "Em G7, Si e Fá formam um trítono que resolve em Dó e Mi."],
  // Percepção
  ["Percepção", "Ouvido relativo", "Reconhecer notas pelas relações (intervalos, graus). Treinável por qualquer pessoa."],
  ["Percepção", "Ouvido absoluto", "Nomear uma nota sem referência. Raro; não é necessário para ser músico."],
  ["Percepção", "Dó móvel", "Solfejo em que a tônica de qualquer tom se chama “dó” (ou 1)."],
  ["Percepção", "Cent", "1/100 de semitom. Ouvidos treinados percebem poucas unidades."],
  ["Percepção", "Batimento", "“Tremor” no volume quando duas notas quase iguais soam juntas; some quando afinam.", "fx:beats:3"],
  ["Percepção", "Intervalo melódico × harmônico", "Melódico: uma nota depois da outra. Harmônico: as duas juntas."],
  ["Percepção", "Ditado melódico", "Ouvir uma melodia e escrevê-la: tônica, contorno, graus e ritmo."],
  // Ritmo e prática
  ["Prática", "Pulso", "A batida regular de fundo, que você marca com o pé."],
  ["Prática", "Andamento (BPM)", "A velocidade do pulso, em batidas por minuto."],
  ["Prática", "Subdivisão", "Dividir mentalmente o tempo (colcheias, tercinas, semicolcheias) para não correr nem atrasar."],
  ["Prática", "Síncope", "Som que começa num tempo fraco e se prolonga sobre o forte, deslocando o acento."],
  ["Prática", "Contratempo", "Toque na parte fraca do tempo (o “e” do 1 e 2 e…), com o tempo forte em silêncio."],
  ["Prática", "Tresillo", "Agrupamento 3 + 3 + 2 em oito subdivisões: a base de baião, reggaeton e muito pop."],
  ["Prática", "Prática deliberada", "Treino com meta específica, foco total, feedback e dificuldade logo acima do confortável."],
  ["Prática", "Prática intercalada", "Alternar tipos de exercício na mesma sessão: rende menos na hora e fixa mais."],
  ["Prática", "Legato × staccato", "Legato: notas ligadas. Staccato: notas curtas e destacadas."],
  ["Prática", "Crescendo", "Aumentar a intensidade aos poucos (sinal <)."],
  ["Prática", "Ritardando", "Desacelerar aos poucos (rit.)."],
  ["Prática", "Rubato", "Flexibilidade expressiva do andamento: “roubar” tempo e devolver."],
  ["Prática", "Anacruse", "Nota(s) antes do primeiro tempo forte; na regência, o gesto de preparação."],
  // Composição
  ["Composição", "Motivo", "A menor ideia musical reconhecível, que se repete e se transforma."],
  ["Composição", "Sequência", "Repetir um motivo em outra altura."],
  ["Composição", "Pergunta e resposta", "Duas frases: a primeira termina suspensa, a segunda conclui na tônica."],
  ["Composição", "Prosódia", "Casar a sílaba tônica da palavra com o tempo forte da música."],
  ["Composição", "Forma sonata", "Exposição (2 temas) – desenvolvimento – reexposição."],
  ["Composição", "Rondó", "Forma em que um refrão volta entre episódios: A B A C A."],
  ["Composição", "Quintas paralelas", "Duas vozes que se movem mantendo 5ª justa. Evitadas no coral tradicional por apagar a independência das vozes."],
  ["Composição", "Movimento contrário", "Uma voz sobe enquanto a outra desce: dá independência."],
  ["Composição", "SATB", "Soprano, Contralto, Tenor e Baixo: as quatro vozes do coral."],
  ["Composição", "Leitmotiv", "Tema ligado a um personagem ou ideia (Wagner; trilhas de cinema)."],
  // História
  ["História", "Canto gregoriano", "Canto monofônico em latim da Igreja medieval, ritmo livre e modos eclesiásticos."],
  ["História", "Guido d'Arezzo", "Monge (séc. XI) que popularizou a pauta e as sílabas ut-ré-mi-fá-sol-lá."],
  ["História", "Baixo contínuo", "Linha de baixo com harmonia improvisada sobre ela: marca do Barroco."],
  ["História", "Soli Deo Gloria", "“Só a Deus a glória”: Bach assinava S.D.G. nas partituras."],
  ["História", "Barroco", "c. 1600–1750: Monteverdi, Vivaldi, Handel, Bach."],
  ["História", "Classicismo", "c. 1750–1820: Haydn, Mozart e o jovem Beethoven. Clareza e forma sonata."],
  ["História", "Romantismo", "Século XIX: emoção, virtuosismo, nacionalismo. Chopin, Wagner, Brahms."],
  ["História", "Bossa nova", "Fim dos anos 1950: João Gilberto, Tom Jobim, Vinicius. Harmonia de jazz e batida de samba suavizada."],
  ["História", "Harpa Cristã", "Hinário das Assembleias de Deus no Brasil, de 1922."],
  // Produção
  ["Produção", "Frequência", "Quantas vibrações por segundo (Hz). Define a altura."],
  ["Produção", "Decibel (dB)", "Escala logarítmica de nível. +6 dB ≈ dobro da pressão; +10 dB ≈ “o dobro do volume” percebido."],
  ["Produção", "Série harmônica", "Fundamental + múltiplos inteiros (2f, 3f, 4f…). Define o timbre.", "fx:harm"],
  ["Produção", "ADSR", "Ataque, decaimento, sustentação e relaxamento: o envelope de um som."],
  ["Produção", "Microfone dinâmico", "Robusto, aguenta pressão alta, ótimo ao vivo."],
  ["Produção", "Microfone condensador", "Sensível e detalhado; precisa de phantom power (48 V)."],
  ["Produção", "Padrão cardioide", "Capta bem pela frente e rejeita por trás."],
  ["Produção", "Efeito de proximidade", "Quanto mais perto de um mic direcional, mais grave."],
  ["Produção", "Clipping", "Sinal acima de 0 dBFS: o topo da onda é cortado e distorce.", "fx:clip"],
  ["Produção", "Taxa de amostragem", "Amostras por segundo (44,1 kHz, 48 kHz). Precisa ser mais que o dobro da maior frequência (Nyquist)."],
  ["Produção", "Latência", "Atraso entre tocar e ouvir. Buffer menor = menos atraso e mais CPU."],
  ["Produção", "Direct box (DI)", "Converte sinal de instrumento em sinal balanceado de microfone."],
  ["Produção", "EQ subtrativa", "Cortar o que sobra antes de reforçar o que falta."],
  ["Produção", "Filtro passa-altas", "Corta os graves abaixo de uma frequência; limpa vozes e violões."],
  ["Produção", "Compressor", "Reduz a diferença entre forte e fraco: limiar, razão, ataque, relaxamento."],
  ["Produção", "Mascaramento", "Um som esconde outro na mesma faixa de frequência."],
  ["Produção", "LUFS", "Unidade de loudness (volume percebido). Plataformas normalizam em torno de −14 LUFS."],
  ["Produção", "RT60", "Tempo para o som cair 60 dB numa sala: mede a reverberação."],
  ["Produção", "Microfonia", "Realimentação: o som da caixa volta ao microfone e apita."],
  ["Produção", "Faixa 250 Hz", "Região do “embolado/abafado”.", "fx:eq:250"],
  ["Produção", "Faixa 2–4 kHz", "Presença, clareza e ataque; em excesso, agressivo.", "fx:eq:3000"]
].map((c, i) => ({ id: "k" + i, cat: c[0], f: c[1], b: c[2], p: c[3] }));

function GUIA() {
  const row = (a, b) => `<tr><td>${a}</td><td>${b}</td></tr>`;
  return `<article class="guia stack">
  <header class="phead"><p class="eyebrow">Guia</p><h1 class="display">O que se aprende numa faculdade de música</h1><p class="muted">E como esta escola cobre cada área.</p></header>
  <section class="card stack"><h2>A ideia central</h2><p>A faculdade ensina a música como <b>linguagem, arte, ciência e profissão</b>. Não é só tocar um instrumento ou cantar: é entender como os sons se organizam, como uma composição é construída e como a música é criada, interpretada, gravada e ensinada.</p></section>
  <section class="card stack"><h2>As 6 áreas e onde estudar aqui</h2><div class="map">
    <a class="mapi" href="#l-t1"><b>Teoria musical</b><span class="small muted">Notas, escalas, intervalos, acordes, ritmo, compassos, tonalidades e partitura → Nível 1 (Quenanias)</span></a>
    <a class="mapi" href="#l-h1"><b>Harmonia e composição</b><span class="small muted">Acordes, tensão e resolução, melodia, forma e arranjo → Níveis 4 e 5 (Asafe e Salomão)</span></a>
    <a class="mapi" href="#l-hi1"><b>História da música</b><span class="small muted">Da Idade Média ao jazz, rock, samba e louvor → Nível 6 (Jubal)</span></a>
    <a class="mapi" href="#ouvido"><b>Percepção e treino auditivo</b><span class="small muted">Intervalos, acordes, ritmo, afinação, timbre → Nível 2 (Davi) + Academia de ouvido</span></a>
    <a class="mapi" href="#l-r1"><b>Prática instrumental ou vocal</b><span class="small muted">Técnica, estudo, conjunto, interpretação → Nível 3 (Miriã) + metrônomo</span></a>
    <a class="mapi" href="#l-s1"><b>Tecnologia, acústica e produção</b><span class="small muted">Física do som, gravação, mixagem, masterização → Nível 7 (Bezalel) + mesa de mixagem</span></a>
  </div></section>
  <section class="card stack"><h2>Quais cursos existem?</h2><div class="tbl"><table><thead><tr><th>Curso</th><th>Foco</th></tr></thead><tbody>
    ${row("Bacharelado em Música", "Formação artística e profissional, em geral com ênfase em instrumento, canto ou outra área.")}
    ${row("Licenciatura em Música", "Formação para ensinar música em escolas e outros contextos educativos.")}
    ${row("Composição", "Criação de obras, melodias, arranjos e, conforme o curso, música para audiovisual.")}
    ${row("Regência", "Preparação e condução de corais, conjuntos e orquestras.")}
    ${row("Produção Musical", "Criação, gravação, edição e finalização de músicas e projetos de áudio (formato varia entre instituições).")}
  </tbody></table></div></section>
  <section class="card stack"><h2>Preciso saber tocar antes?</h2><p>Depende da instituição e do curso. Muitos bacharelados exigem <b>prova de habilidades específicas</b> (THE): tocar ou cantar peças, leitura, percepção (ditados, solfejo) e teoria. Outros cursos têm seleção diferente. Consulte o <b>edital</b> e a <b>matriz curricular</b> de cada universidade.</p>
  <p class="small muted">Para a prova: os níveis 1, 2 e 4 deste app e a Academia de ouvido treinam exatamente teoria, ditado e percepção.</p></section>
  <section class="card stack"><h2>O que um formado em música pode fazer</h2><ul class="goals">
    <li>Músico, instrumentista ou cantor profissional</li><li>Compositor de músicas e trilhas (filmes, séries, publicidade, jogos)</li><li>Produção musical, gravação, edição e áudio</li><li>Arranjador e líder de grupos musicais</li><li>Professor de música (conforme formação)</li><li>Regente, pesquisador, projetos culturais e ministério de música</li></ul></section>
  <section class="card stack"><h2>Música também é ciência</h2><p>Uma trilha de suspense provoca tensão com dissonâncias, intervalos, timbres, ritmo, silêncio e escolhas harmônicas. Estudar música ajuda a reconhecer esses recursos. Mas não existe fórmula única para cada emoção: contexto, cultura e percepção individual também contam.</p>
  <div class="row"><button class="pbtn" data-gplay="fx:beats:2"><span class="pi">${IC.play}</span>Ouvir dissonância (batimento)</button></div>
  <p class="small"><a href="#l-c4">Lição: Música e emoção →</a></p></section>
  <section class="card stack"><h2>Plano de estudo semanal (sugestão)</h2><div class="tbl"><table><tbody>
    ${row("Todo dia (15–25 min)", "1 rodada de ouvido (10 exercícios) + cards + 10 min no instrumento com metrônomo")}
    ${row("3× por semana", "1 lição da trilha (stories + prova) e a missão prática")}
    ${row("1× por semana", "Tirar uma música de ouvido inteira (melodia, baixo e acordes) e tocá-la em 2 tons")}
    ${row("1× por mês", "Gravar-se tocando e ouvir como um engenheiro: tempo, afinação, dinâmica")}
  </tbody></table></div></section>
  <div class="callout">Gostar de música, tocar profissionalmente e entender a fundo como ela funciona são interesses diferentes. Podem se cruzar, mas levam a caminhos de estudo diferentes. Próximo passo: compare a matriz curricular, a prova de ingresso e o instrumento exigido de algumas universidades.</div>
  <p class="small muted">Material introdutório. Os nomes das disciplinas variam conforme a instituição.</p>
  </article>`;
}
