// Telas no padrão do app Firmes: boas-vindas, início, trilha em caminho, mestres, álbum e teste.
"use strict";
const BIO = {
  quenanias: "Chefe dos levitas no canto no tempo de Davi. A Bíblia diz que ele “ensinava o canto, porque era entendido” (1 Cr 15.22). É o professor da teoria: entender antes de impressionar.",
  davi: "Pastor de ovelhas, ruivo e de bela aparência (1 Sm 16.12). Quando tocava a harpa, Saul se acalmava (1 Sm 16.23). Depois, rei e o grande salmista de Israel. Ensina a ouvir.",
  miria: "Profetisa, irmã de Moisés e Arão. Depois da travessia do mar, pegou o tamborim e conduziu as mulheres em canto e dança (Êx 15.20-21). Ensina ritmo e a tocar em grupo.",
  asafe: "Levita que Davi colocou como chefe dos cantores diante da arca; ele fazia ressoar os címbalos (1 Cr 16.5). Doze salmos levam seu nome (Sl 50; 73–83). Ensina harmonia.",
  salomao: "Rei de Israel, famoso pela sabedoria. “Compôs três mil provérbios, e foram os seus cânticos mil e cinco” (1 Rs 4.32). Ensina composição e arranjo.",
  jubal: "Descendente de Caim, “pai de todos os que tocam harpa e flauta” (Gn 4.21): a primeira menção a músicos na Bíblia. Ensina a história da música.",
  bezalel: "Artesão que Deus encheu “de sabedoria, entendimento, ciência e em todo artifício” para construir o tabernáculo (Êx 31.3). Ensina acústica, tecnologia e produção."
};
const STAGE_TXT = ["Assista aos stories do nível", "4 pontos no nível", "9 pontos no nível", "14 pontos no nível"];

// ---------- boas-vindas ----------
function viewBemvindo() {
  document.body.classList.add("onb"); $("#tabs").hidden = true;
  app().innerHTML = `<section class="onbx">
    <img class="appicon" src="icon.svg" alt="">
    <h1 class="display onbt">Harpa</h1>
    <p class="onbs">Escola de música para quem quer entender, tocar e servir.</p>
    <div class="card versec"><p>“Cantai-lhe um cântico novo; tocai bem e com júbilo.”</p><b>SALMO 33.3</b></div>
    <div class="feats">
      <div><span class="fi" style="--fc:#E8590C">${IC.path}</span><p><b>Trilha da faculdade de música</b><br><span>Teoria, ouvido, prática, harmonia, composição, história e produção.</span></p></div>
      <div><span class="fi" style="--fc:#6D3FE0">${IC.ear}</span><p><b>Academia de ouvido</b><br><span>Exercícios com som de verdade, do iniciante ao avançado.</span></p></div>
      <div><span class="fi" style="--fc:#0B7285">${IC.lab}</span><p><b>Laboratório</b><br><span>Teclado, campo harmônico, metrônomo e mesa de mixagem.</span></p></div>
    </div>
    <label class="small" for="nm"><b>Como podemos te chamar?</b></label>
    <input id="nm" class="inp" placeholder="Seu nome" autocomplete="given-name" maxlength="30">
    <button class="btn pri big" id="go">Continuar</button></section>`;
  const go = () => { const n = $("#nm").value.trim(); if (!n) { $("#nm").focus(); toast("Digite seu nome"); return; } S.name = n; save(); document.body.classList.remove("onb"); location.hash = "#inicio"; route(); };
  $("#go").onclick = go; $("#nm").onkeydown = e => { if (e.key === "Enter") go(); };
}
function greet() { const h = new Date().getHours(); return h < 12 ? "Bom dia" : h < 18 ? "Boa tarde" : "Boa noite"; }

// ---------- figurinhas ----------
let FIGL = null;
function figList() {
  if (FIGL) return FIGL; const L = []; let n = 0;
  TRACKS.forEach(t => { t.mods.forEach(id => FIGS[id].forEach((f, r) => L.push({ id: id + "-" + r, n: ++n, les: id, r, name: f[0], cat: f[1], d: f[2] }))); L.push({ id: "L-" + t.mk, n: ++n, leg: t.mk, tr: t, r: 3, name: MENTORS[t.mk].name + ", o Mestre", cat: "Lendária", d: BIO[t.mk].split(".")[0] + "." }); });
  return (FIGL = L);
}
function figOwned(f) { return f.leg ? mStage(f.tr) >= 3 : f.r === 0 ? storyDone(f.les) : f.r === 1 ? passed(f.les) : perfect(f.les); }
function figNew() { S.figSeen = S.figSeen || {}; return figList().filter(f => figOwned(f) && !S.figSeen[f.id]); }
function figCard(f, o = {}) {
  const own = figOwned(f) || o.force;
  if (!own) return `<div class="fig off"><span class="fnum">${f.n}</span><span class="fq">?</span><p class="small">${esc(FCOND[f.r])}${f.les ? `<br><b>${esc(L(f.les).title)}</b>` : ""}</p></div>`;
  if (f.leg) { const M = MENTORS[f.leg]; return `<div class="fig leg" style="--f1:${M.c1};--f2:${M.c2}"><span class="fnum">${f.n}</span><div class="fart">${mArt(f.leg, 3)}</div><b>${esc(f.name)}</b><span class="fcat">Lendária</span></div>`; }
  const [c1, c2] = FCAT[f.cat] || FCAT.Conceito;
  return `<div class="fig" style="--f1:${c1};--f2:${c2}"><span class="fnum">${f.n}</span><span class="fcat">${esc(f.cat)}</span><b>${esc(f.name)}</b><p>${esc(f.d)}</p></div>`;
}
function packModal() {
  const nw = figNew(); if (!nw.length) return;
  nw.forEach(f => S.figSeen[f.id] = 1); save();
  modal(`<div class="center stack"><p class="eyebrow">Figurinhas novas!</p><h2 class="display">+${nw.length} no álbum</h2><div class="figrow">${nw.slice(0, 3).map(f => figCard(f, { force: true })).join("")}</div>${nw.length > 3 ? `<p class="small muted">e mais ${nw.length - 3}…</p>` : ""}<a class="btn" href="#album">Ver álbum</a></div>`);
}
function viewAlbum() {
  const Lf = figList(), own = Lf.filter(figOwned).length;
  app().innerHTML = `<a class="back" href="#inicio">${IC.back} Início</a><header class="phead"><p class="eyebrow">Álbum de figurinhas</p><h1 class="display">${own} de ${Lf.length} coladas</h1><p class="muted">3 por lição (stories, prova e nota máxima) e uma lendária por mestre.</p><div class="qbar"><span style="width:${own / Lf.length * 100}%"></span></div></header>
  ${TRACKS.map(t => { const M = MENTORS[t.mk], fs = Lf.filter(f => (f.les && trackOf(f.les) === t) || f.leg === t.mk); return `<section class="stack"><div class="row" style="flex-wrap:nowrap">${ava(t.mk, 36, mStage(t))}<div><p class="eyebrow">Nível ${t.n} · ${esc(M.name)}</p><b>${esc(t.title)}</b> <span class="small muted">${fs.filter(figOwned).length}/${fs.length}</span></div></div><div class="figgrid">${fs.map(f => figCard(f)).join("")}</div></section>`; }).join("")}`;
  figNew().forEach(f => S.figSeen[f.id] = 1); save();
}

// ---------- início ----------
function viewHome() {
  const Lv = level(), nx = nextLesson(), tot = ALL().length, done = ALL().filter(passed).length;
  const t = nx ? trackOf(nx) : TRACKS[TRACKS.length - 1], M = MENTORS[t.mk];
  const ear = earSuggest(), earToday = (S.earDays || {})[today()] || 0, due = dueCards().length;
  const ts = { les: Object.values(S.les).some(l => l.day === today()), ear: earToday >= 10, cards: cardsToday() >= 10 };
  const nm = Object.values(ts).filter(Boolean).length, Lf = figList(), ownF = Lf.filter(figOwned).length;
  const q = QUOTES[Math.floor(Date.now() / 864e5) % QUOTES.length], voc = S.voc && VOC.areas[S.voc.res];
  const mastered = Object.values(S.cards).filter(c => c.iv >= 3).length;
  app().innerHTML = `
  <section class="hello"><p class="eyebrow">${esc(greet())}, ${esc(S.name || "músico")}!</p><h1 class="display hbig">Pronto para <span class="grad">tocar bem</span>?</h1><p class="verse">“Cantai-lhe um cântico novo; tocai bem e com júbilo.” <span>Sl 33.3</span></p></section>
  ${voc ? `<a class="card testdone" href="#teste" style="--c1:${voc.c1};--c2:${voc.c2}"><span class="tdi">♪</span><div class="grow"><p class="eyebrow">Seu caminho na música</p><b>${esc(voc.n)}</b><p class="small muted">Toque para ver o resultado ou refazer o teste</p></div></a>`
    : `<section class="card testcard"><div class="row" style="flex-wrap:nowrap;align-items:flex-start"><span class="tplus">♪</span><div class="grow"><p class="eyebrow">Faça o teste</p><h2>Qual caminho na música combina com você?</h2><p class="small muted">10 perguntas. Descubra se você tem mais cara de intérprete, educador, compositor, regente ou produtor, e o que estudar primeiro.</p></div></div><a class="btn pri" href="#teste">Começar</a></section>`}
  <section class="stack"><div class="row" style="justify-content:space-between"><h2 class="sec">Stories dos mestres</h2><a class="small" href="#mentores">Ver mestres ›</a></div>
    <div class="srow">${TRACKS.map(tr => { const un = trackUnlocked(tr), st = mStage(tr), nw = un && tr.mods.some(id => lessonUnlocked(id) && !storyDone(id)); return `<a class="sitem ${un ? "" : "off"}" href="${un ? "#s-" + (tr.mods.find(id => lessonUnlocked(id) && !storyDone(id)) || tr.mods.find(id => !passed(id)) || tr.mods[0]) : "#mentor-" + tr.mk}"><span class="sring ${nw ? "new" : ""}" style="--c1:${MENTORS[tr.mk].c1};--c2:${MENTORS[tr.mk].c2}">${mArt(tr.mk, st, { crop: true })}${un ? (nw ? `<em>NOVO</em>` : "") : `<i>${IC.lock}</i>`}</span><b>${esc(MENTORS[tr.mk].name)}</b><span>${esc(tr.short)}</span></a>`; }).join("")}</div></section>
  <section class="card here" style="--c1:${M.c1};--c2:${M.c2}">
    <div class="row" style="flex-wrap:nowrap">${ava(t.mk, 58, mStage(t))}<div class="grow"><p class="eyebrow">Você está aqui</p><h2>Nível ${t.n} · ${esc(t.title)}</h2><p class="small muted">${nx ? `Lição ${t.n}.${t.mods.indexOf(nx) + 1}: ${esc(L(nx).title)}` : "Trilha completa! Busque nota máxima em tudo."}</p></div></div>
    <div class="segs">${TRACKS.map(tr => `<div class="seggrp ${tr === t ? "cur" : ""}"><span>${tr.n}</span><div>${tr.mods.map(id => `<i class="${passed(id) ? "on" : id === nx ? "now" : ""}"></i>`).join("")}</div></div>`).join("")}</div>
    <div class="row">${nx ? `<a class="btn pri" href="#l-${nx}">Continuar lição</a>` : ""}<a class="btn" href="#trilha">Ver trilha completa</a></div></section>
  <section class="card lvl"><span class="lvn">${Lv.i + 1}</span><div class="grow"><div class="row" style="justify-content:space-between"><b>${esc(Lv.name)}</b><span class="small muted">${S.xp}${Lv.next ? " / " + Lv.next : ""} XP</span></div><div class="qbar"><span style="width:${Lv.next ? (S.xp - Lv.min) / (Lv.next - Lv.min) * 100 : 100}%"></span></div><p class="small muted">${Lv.next ? `Faltam ${Lv.next - S.xp} XP para <b>${esc(Lv.nextName)}</b>` : "Título máximo!"}</p></div></section>
  <div class="stats3"><div class="card"><b class="bign">${streak()}</b><span>dias seguidos</span></div><div class="card"><b class="bign">${done}/${tot}</b><span>lições aprovadas</span></div><div class="card"><b class="bign">${mastered}</b><span>cards dominados</span></div></div>
  <section class="card stack"><div class="row" style="justify-content:space-between"><h2>Missão de hoje</h2><span class="small muted">${nm}/3</span></div>
    <div class="mission"><span class="check ${ts.les ? "done" : ""}">${IC.check}</span><div class="grow"><b>Estudar 1 lição</b><p class="small muted">${nx ? esc(L(nx).title) : "Revise uma lição"}</p></div>${nx ? `<a class="btn pri" href="#l-${nx}">Estudar</a>` : ""}</div>
    <div class="mission"><span class="check ${ts.ear ? "done" : ""}">${IC.check}</span><div class="grow"><b>10 exercícios de ouvido</b><p class="small muted">${earToday}/10 hoje · sugestão: ${esc(EAR[ear].name)}</p></div><a class="btn" href="#o-${ear}">Treinar</a></div>
    <div class="mission"><span class="check ${ts.cards ? "done" : ""}">${IC.check}</span><div class="grow"><b>Revisar 10 cards</b><p class="small muted">${cardsToday()}/10 hoje · ${due} para revisar</p></div><a class="btn" href="#cards">Revisar</a></div></section>
  <a class="card albumc" href="#album"><div class="stackc"><i></i><i></i><i></i></div><div class="grow"><p class="eyebrow">Álbum de figurinhas</p><h2>${ownF} de ${Lf.length} coladas</h2><p class="small muted">Ganhe figurinhas terminando as lições.</p></div></a>
  <div class="grid2">
    <a class="card linkcard" href="#lab"><span class="lc-ic">${IC.lab}</span><div><b>Laboratório</b><p class="small muted">Teclado, campo harmônico, círculo das quintas, metrônomo, ondas e mesa de mixagem</p></div></a>
    <a class="card linkcard" href="#guia"><span class="lc-ic">${IC.book}</span><div><b>Guia da faculdade de música</b><p class="small muted">Disciplinas, cursos, carreiras e plano de estudo</p></div></a>
  </div>
  <section class="card stack"><p class="eyebrow">Frase do dia</p><p class="qday">“${esc(q[0])}”</p><p class="small muted">${esc(q[1])}</p></section>`;
}

// ---------- trilha em caminho ----------
const XS = [50, 74, 50, 26];
function viewTrilha() {
  const nx = nextLesson(), t = nx ? trackOf(nx) : null;
  app().innerHTML = `<header class="phead"><p class="eyebrow">Do ouvinte ao maestro</p><h1 class="display">Sua trilha</h1></header>
  ${nx ? `<section class="card here" style="--c1:${MENTORS[t.mk].c1};--c2:${MENTORS[t.mk].c2}"><div class="row" style="flex-wrap:nowrap">${ava(t.mk, 58, mStage(t))}<div class="grow"><p class="eyebrow">Você está aqui</p><h2>Nível ${t.n} · ${esc(t.short)}</h2><p class="small muted">Lição ${t.n}.${t.mods.indexOf(nx) + 1} de ${ALL().length}: ${esc(L(nx).title)}</p></div></div><a class="btn pri" href="#l-${nx}">Continuar lição</a></section>` : ""}
  <div class="chipsT">${TRACKS.map(tr => { const un = trackUnlocked(tr), d = tr.mods.filter(passed).length; return `<a class="tchip ${tr === t ? "cur" : ""} ${un ? "" : "off"}" href="#trk${tr.n}"><b>${tr.n}. ${esc(tr.short)}</b>${un ? `<span>${d}/4</span>` : IC.lock}<i style="--p:${d / 4 * 100}%"></i></a>`; }).join("")}</div>
  ${TRACKS.map(tr => {
    const un = trackUnlocked(tr), M = MENTORS[tr.mk], H = tr.mods.length * 112 + 30;
    const pts = tr.mods.map((_, i) => [XS[i % 4], 60 + i * 112]);
    const path = pts.map((p, i) => (i ? "L" : "M") + p[0] + " " + p[1]).join(" ");
    return `<section class="card trk ${un ? "" : "locked"}" id="trk${tr.n}" style="--c1:${M.c1};--c2:${M.c2}">
      <div class="row" style="flex-wrap:nowrap">${ava(tr.mk, 46, mStage(tr))}<div class="grow"><p class="eyebrow">Nível ${tr.n} · ${["Iniciante", "Iniciante", "Intermediário", "Intermediário", "Avançado", "Avançado", "Mestre"][tr.n - 1]} · ${esc(M.name)}</p><h2>${esc(tr.title)}</h2><p class="small muted">${esc(tr.desc)}</p></div><span class="pillc">${un ? tr.mods.filter(passed).length + "/4" : IC.lock}</span></div>
      <div class="path" style="height:${H}px"><svg viewBox="0 0 100 ${H}" preserveAspectRatio="none"><path d="${path}" class="dots"/></svg>
      ${tr.mods.map((id, i) => {
        const ok = lessonUnlocked(id), cls = passed(id) ? "done" : ok ? "open" : "lock", [x, y] = pts[i], right = x <= 50;
        return `<a class="pnode ${cls} ${id === nx ? "cur" : ""}" href="${ok ? "#l-" + id : "#trilha"}" style="left:${x}%;top:${y}px">
          ${id === nx ? `<span class="here-pill">${esc((S.name || "V")[0].toUpperCase())} · Você está aqui</span>` : ""}
          <span class="pdot">${passed(id) ? IC.check : ok ? tr.n + "." + (i + 1) : IC.lock}</span>
          <span class="plab ${right ? "r" : "l"}"><b>${esc(L(id).title)}</b><span>${passed(id) ? `Aprovada · ${Math.round(best(id) * 100)}%` : ok ? (id === nx ? "Agora" : "Disponível") + ` · ${Math.max(4, Math.round(L(id).s.length * .45))} min` : "Bloqueada"}</span></span></a>`;
      }).join("")}</div>
      ${un ? "" : `<p class="small muted lockmsg">${IC.lock} Aprove 3 lições de “${esc(TRACKS[tr.n - 2].short)}” para abrir.</p>`}
    </section>`;
  }).join("")}
  <div class="pend">${mArt("davi", 3, { cls: "endart" })}<b>Maestro</b><span class="small muted">Fim da trilha</span></div>
  <p class="small muted center">Acerte 70% da prova para aprovar uma lição. Aprove 3 lições de um nível para abrir o próximo.</p>`;
  if (nx) setTimeout(() => { const c = document.querySelector(".pnode.cur"); if (c && location.hash === "#trilha") c.scrollIntoView({ block: "center", behavior: "smooth" }); }, 350);
  app().querySelectorAll(".tchip").forEach(a => a.onclick = e => { e.preventDefault(); const el = document.querySelector(a.getAttribute("href")); if (el) el.scrollIntoView({ behavior: "smooth", block: "start" }); });
}

// ---------- mestres ----------
function viewMentores() {
  app().innerHTML = `<a class="back" href="#inicio">${IC.back} Início</a><header class="phead"><p class="eyebrow">Quem caminha com você</p><h1 class="display">Seus mestres</h1><p class="muted">Cada nível tem um músico da Bíblia. Você sobe de título com ele, de Ouvinte a Mestre, ao assistir aos stories, aprovar a prova, tirar nota máxima e cumprir a missão de cada lição.</p></header>
  <div class="mgrid">${TRACKS.map(t => { const M = MENTORS[t.mk], st = mStage(t), p = mPts(t), nxt = [4, 9, 14][st]; return `<a class="mcard ${trackUnlocked(t) ? "" : "off"}" href="#mentor-${t.mk}" style="--c1:${M.c1};--c2:${M.c2}">${mArt(t.mk, st)}<b>${esc(M.name)}</b><span>${esc(t.short)} · você: ${RANKS[st]}</span><i><em style="width:${nxt ? Math.min(100, p / nxt * 100) : 100}%"></em></i></a>`; }).join("")}</div>`;
}
function viewMentor(k) {
  const t = TRACKS.find(x => x.mk === k); if (!t) { location.replace("#mentores"); return; }
  const M = MENTORS[k], st = mStage(t), p = mPts(t), un = trackUnlocked(t);
  app().innerHTML = `<a class="back" href="#mentores">${IC.back} Mestres</a>
  <section class="mhero" style="--c1:${M.c1};--c2:${M.c2}">${mArt(k, st)}<p class="eyebrow">Nível ${t.n} · ${esc(t.title)}</p><h1 class="display">${esc(M.name)}</h1><p class="small">${esc(M.ref)} · você: <b>${RANKS[st]}</b> (${p}/16 pontos)</p></section>
  <section class="card stack"><p>${esc(BIO[k])}</p></section>
  <section class="card stack"><p class="eyebrow">Evolução</p><div class="evo">${RANKS.map((r, i) => `<div class="${i <= st ? "on" : ""}">${mArt(k, i, { crop: true })}<b>${r}</b><span>${i === 0 ? "Início" : STAGE_TXT[i]}</span>${i === 2 ? `<em>Dica</em>` : i === 3 ? `<em>Lendária</em>` : ""}</div>`).join("")}</div>
  <p class="small muted">Cada lição vale até 4 pontos: stories, prova aprovada, nota máxima e missão.</p></section>
  <section class="card stack"><p class="eyebrow">Lições</p>${t.mods.map((id, i) => `<a class="mission" href="${lessonUnlocked(id) ? "#l-" + id : "#mentor-" + k}"><span class="check ${passed(id) ? "done" : ""}">${passed(id) ? IC.check : i + 1}</span><div class="grow"><b>${esc(L(id).title)}</b><p class="small muted">${lessonPts(id)}/4 pontos${lessonUnlocked(id) ? "" : " · bloqueada"}</p></div></a>`).join("")}</section>
  ${un ? `<a class="btn pri big" href="#s-${t.mods.find(id => lessonUnlocked(id) && !storyDone(id)) || t.mods[0]}">Ver stories de ${esc(M.name)}</a>` : `<p class="small muted center">${IC.lock} Aprove 3 lições do nível anterior para abrir.</p>`}`;
}

// ---------- teste ----------
let VT = null;
function viewTeste() {
  if (!VT) VT = { i: -1, sc: {} };
  const N = VOC.q.length;
  if (VT.i < 0 && S.voc) return vocResult();
  if (VT.i < 0) {
    app().innerHTML = `<a class="back" href="#inicio">${IC.back} Início</a><section class="card stack center testintro"><span class="tplus big">♪</span><p class="eyebrow">Teste</p><h1 class="display">Qual caminho na música combina com você?</h1><p class="muted">${N} perguntas rápidas. Não há resposta certa: escolha o que é mais <b>você</b>. O resultado mostra os 5 cursos de uma faculdade de música e o que estudar primeiro aqui no app.</p><button class="btn pri big" id="vgo">Começar</button><p class="small muted">É um teste para reflexão, não um diagnóstico. Converse também com professores e músicos que você admira.</p></section>`;
    $("#vgo").onclick = () => { VT = { i: 0, sc: {} }; viewTeste(); }; return;
  }
  if (VT.i >= N) {
    const res = Object.entries(VT.sc).sort((a, b) => b[1] - a[1])[0][0];
    S.voc = { res, sc: VT.sc, day: today() }; save(); gain(10, "teste", true); VT = null; return vocResult();
  }
  const [q, os] = VOC.q[VT.i];
  if (!VT.ord || VT.ordI !== VT.i) { VT.ord = shuf(os.map((_, i) => i)); VT.ordI = VT.i; }
  app().innerHTML = `<a class="back" href="#inicio">${IC.back} Sair</a><section class="card stack quiz"><div class="row" style="justify-content:space-between"><p class="eyebrow">Teste vocacional</p><span class="small muted">${VT.i + 1}/${N}</span></div><div class="qbar"><span style="width:${VT.i / N * 100}%"></span></div><h2>${esc(q)}</h2><div class="opts">${VT.ord.map(i => `<button class="opt" data-a="${os[i][1]}">${esc(os[i][0])}</button>`).join("")}</div>${VT.i ? `<button class="btn" id="vback">${IC.back} Voltar</button>` : ""}</section>`;
  app().querySelectorAll(".opt").forEach(b => b.onclick = () => { (VT.hist = VT.hist || []).push(b.dataset.a); VT.sc[b.dataset.a] = (VT.sc[b.dataset.a] || 0) + 1; VT.i++; viewTeste(); window.scrollTo(0, 0); });
  const vb = $("#vback"); if (vb) vb.onclick = () => { const a = VT.hist.pop(); VT.sc[a]--; VT.i--; viewTeste(); };
}
function vocResult() {
  const v = S.voc, A = VOC.areas[v.res], tot = Object.values(v.sc).reduce((a, b) => a + b, 0) || 1;
  const order = Object.keys(VOC.areas).sort((a, b) => (v.sc[b] || 0) - (v.sc[a] || 0));
  app().innerHTML = `<a class="back" href="#inicio">${IC.back} Início</a>
  <section class="card stack vres" style="--c1:${A.c1};--c2:${A.c2}"><p class="eyebrow">Seu caminho mais forte</p><h1 class="display">${esc(A.n)}</h1><p>${esc(A.d)}</p>
    <p class="small"><b>Onde atua:</b> ${esc(A.prof)}</p></section>
  <section class="card stack"><p class="eyebrow">Seu perfil completo</p>${order.map(k => { const a = VOC.areas[k], p = Math.round((v.sc[k] || 0) / tot * 100); return `<div class="vbar"><span>${esc(a.n)}</span><div><i style="width:${p}%;background:linear-gradient(90deg,${a.c1},${a.c2})"></i></div><b>${p}%</b></div>`; }).join("")}
  <p class="small muted">Quase todo músico mistura caminhos: o produtor que compõe, o regente que ensina. Use o resultado como bússola, não como rótulo.</p></section>
  <section class="card stack"><p class="eyebrow">Comece por aqui no app</p>${A.foco.map(f => `<p>• ${esc(f)}</p>`).join("")}</section>
  <div class="row"><a class="btn pri" href="#guia">Ver os cursos no guia</a><button class="btn" id="vagain">Refazer o teste</button></div>`;
  $("#vagain").onclick = () => { S.voc = null; save(); VT = { i: 0, sc: {} }; viewTeste(); };
}

// ---------- início do app ----------
window.addEventListener("hashchange", route);
document.addEventListener("pointerdown", () => AU.init(), { once: true });
applyTheme(); hdr(); route();
if ("serviceWorker" in navigator && location.protocol === "https:" && !/claude|anthropic/.test(location.host)) navigator.serviceWorker.register("sw.js").catch(() => {});
