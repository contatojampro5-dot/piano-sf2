// Escola Harpa — app de estudos de música. Estado no aparelho (localStorage), rotas por #hash.
"use strict";
const $ = s => document.querySelector(s), app = () => $("#app");
const LS = "harpa.v1";
const LEVELS = [[0, "Ouvinte"], [150, "Aprendiz"], [400, "Instrumentista"], [800, "Músico"], [1400, "Maestro"], [2200, "Virtuose"]];
const RANKS = ["Ouvinte", "Aprendiz", "Músico", "Mestre"];
const PASS = 0.7, INTERVALS = [1, 2, 4, 8, 16, 32, 64];

function fresh() { return { v: 1, xp: 0, days: {}, les: {}, mis: {}, ear: {}, cards: {}, unlockAll: false, theme: "auto", mstage: {}, earDays: {}, seenGuide: false }; }
let S = fresh();
try { const r = JSON.parse(localStorage.getItem(LS) || "null"); if (r && r.v === 1) S = Object.assign(fresh(), r); } catch (e) {}
function save() { try { localStorage.setItem(LS, JSON.stringify(S)); } catch (e) {} }
function dkey(d) { return d.getFullYear() + "-" + String(d.getMonth() + 1).padStart(2, "0") + "-" + String(d.getDate()).padStart(2, "0"); }
function today() { return dkey(new Date()); }
function addDays(k, n) { const [y, m, d] = k.split("-").map(Number); return dkey(new Date(y, m - 1, d + n)); }
function gain(n, why, quiet) { S.xp += n; const k = today(); S.days[k] = (S.days[k] || 0) + n; save(); if (!quiet) toast("+" + n + " XP · " + why); hdr(); }
function streak() { let k = today(), c = 0; if (!S.days[k]) k = addDays(k, -1); while (S.days[k]) { c++; k = addDays(k, -1); } return c; }
function level() { let i = 0; LEVELS.forEach((l, j) => { if (S.xp >= l[0]) i = j; }); const cur = LEVELS[i], nx = LEVELS[i + 1]; return { i, name: cur[1], min: cur[0], next: nx ? nx[0] : null, nextName: nx ? nx[1] : null }; }
function esc(s) { return String(s).replace(/[&<>"']/g, c => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", "\"": "&quot;", "'": "&#39;" }[c])); }
function fmt(s) { return esc(s).replace(/\*\*(.+?)\*\*/g, "<b>$1</b>").replace(/(^|[\s(“])_(.+?)_(?=[\s.,;:!?)”]|$)/g, "$1<i>$2</i>").replace(/\n/g, "<br>"); }
function shuf(a) { a = a.slice(); for (let i = a.length - 1; i > 0; i--) { const j = Math.floor(Math.random() * (i + 1)); [a[i], a[j]] = [a[j], a[i]]; } return a; }
function pick(a) { return a[Math.floor(Math.random() * a.length)]; }
let tt; function toast(m) { const t = $("#toast"); t.textContent = m; t.classList.add("on"); clearTimeout(tt); tt = setTimeout(() => t.classList.remove("on"), 2200); }

// ---------- lições ----------
const ALL = () => TRACKS.flatMap(t => t.mods);
function L(id) { return LESSONS[id]; }
function trackOf(id) { return TRACKS.find(t => t.mods.includes(id)); }
function storyDone(id) { return !!(S.les[id] && S.les[id].story); }
function best(id) { return (S.les[id] && S.les[id].best) || 0; }
function passed(id) { return best(id) >= PASS; }
function perfect(id) { return best(id) >= 1; }
function misDone(id) { return !!S.mis[id]; }
function lessonPts(id) { return (storyDone(id) ? 1 : 0) + (passed(id) ? 1 : 0) + (perfect(id) ? 1 : 0) + (misDone(id) ? 1 : 0); }
function trackUnlocked(t) { if (S.unlockAll || t.n === 1) return true; const prev = TRACKS[t.n - 2]; return prev.mods.filter(passed).length >= 3; }
function lessonUnlocked(id) { const t = trackOf(id); if (!trackUnlocked(t)) return false; if (S.unlockAll) return true; const i = t.mods.indexOf(id); return i === 0 || storyDone(t.mods[i - 1]) || passed(t.mods[i - 1]); }
function nextLesson() { for (const id of ALL()) if (lessonUnlocked(id) && !passed(id)) return id; return null; }
function mPts(t) { return t.mods.reduce((a, id) => a + lessonPts(id), 0); }
function mStage(t) { const p = mPts(t); return p >= 14 ? 3 : p >= 9 ? 2 : p >= 4 ? 1 : 0; }

// ---------- ícones ----------
const IC = {
  home: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M3 11l9-7 9 7v9a1 1 0 0 1-1 1h-5v-6h-6v6H4a1 1 0 0 1-1-1z"/></svg>',
  path: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="6" cy="19" r="2"/><circle cx="18" cy="5" r="2"/><path d="M8 19h7a3.5 3.5 0 0 0 0-7H9a3.5 3.5 0 0 1 0-7h7"/></svg>',
  ear: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M6 9a6 6 0 1 1 12 0c0 3-2 4-3 5.5S14 18 12 20a3 3 0 0 1-5-2"/><path d="M9.5 9a2.5 2.5 0 0 1 5 0c0 1.5-1.5 2-1.5 3.5"/></svg>',
  lab: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M4 21V14M4 10V3M12 21v-9M12 8V3M20 21v-5M20 12V3M1 14h6M9 8h6M17 16h6"/></svg>',
  cards: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="6" width="14" height="15" rx="2"/><path d="M7 3h12a2 2 0 0 1 2 2v12"/></svg>',
  user: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="8" r="4"/><path d="M4 21a8 8 0 0 1 16 0"/></svg>',
  play: '<svg viewBox="0 0 24 24" fill="currentColor"><path d="M7 4.5v15a1 1 0 0 0 1.5.86l12.5-7.5a1 1 0 0 0 0-1.72L8.5 3.64A1 1 0 0 0 7 4.5z"/></svg>',
  stop: '<svg viewBox="0 0 24 24" fill="currentColor"><rect x="6" y="6" width="12" height="12" rx="2"/></svg>',
  check: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round"><path d="M5 12.5l4.5 4.5L19 7.5"/></svg>',
  x: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.6" stroke-linecap="round"><path d="M6 6l12 12M18 6L6 18"/></svg>',
  lock: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.4" stroke-linecap="round" stroke-linejoin="round"><rect x="5" y="11" width="14" height="10" rx="2.5"/><path d="M8 11V8a4 4 0 0 1 8 0v3"/></svg>',
  back: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.4" stroke-linecap="round" stroke-linejoin="round"><path d="M15 5l-7 7 7 7"/></svg>',
  flame: '<svg viewBox="0 0 24 24" fill="currentColor"><path d="M12 2c1 4-3 6-3 10a3 3 0 0 0 6 0c0-1.5-.6-2.6-1-3.5 2.6 1.4 4 4 4 6.5a6 6 0 0 1-12 0C6 9 11 7 12 2z"/></svg>',
  book: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M4 19.5A2.5 2.5 0 0 1 6.5 17H20V3H6.5A2.5 2.5 0 0 0 4 5.5z"/><path d="M4 19.5A2.5 2.5 0 0 0 6.5 22H20v-5"/></svg>'
};
const btnPlay = (attr, label) => `<button class="pbtn" ${attr}><span class="pi">${IC.play}</span>${esc(label || "Ouvir")}</button>`;

// ---------- cabeçalho e abas ----------
function hdr() { const h = $("#hdrStats"); if (h) h.innerHTML = `<span class="flame">${IC.flame}</span><b>${streak()}</b>d <span>·</span> <b>${S.xp}</b> XP`; }
function tabs(cur) {
  const due = dueCards().length;
  const items = [["inicio", "Início", IC.home], ["trilha", "Trilha", IC.path], ["ouvido", "Ouvido", IC.ear], ["lab", "Lab", IC.lab], ["cards", "Cards", IC.cards]];
  $("#tabs").innerHTML = `<div class="tabs-in">${items.map(([h, n, ic]) => `<a class="tab" href="#${h}" ${cur === h ? 'aria-current="page"' : ""}>${ic}${n}${h === "cards" && due ? `<span class="dot">${due}</span>` : ""}</a>`).join("")}</div>`;
  $("#tabs").hidden = false;
}

// ---------- mentores ----------
function ava(k, size = 46, stage) {
  const M = MENTORS[k]; const st = stage == null ? 0 : stage;
  return `<span class="ava st${st}" style="--c1:${M.c1};--c2:${M.c2};width:${size}px;height:${size}px;font-size:${size * .46}px" aria-hidden="true">${M.emoji}</span>`;
}

// ---------- teclado ----------
const WHITE = [0, 2, 4, 5, 7, 9, 11];
function kbHTML(from, to, id) {
  const whites = []; for (let m = from; m <= to; m++) if (WHITE.includes(((m % 12) + 12) % 12)) whites.push(m);
  const w = 100 / whites.length;
  let html = `<div class="kb" id="${id || "kb"}" data-from="${from}" data-to="${to}">`;
  whites.forEach((m, i) => { html += `<button class="wk" data-m="${m}" style="left:${i * w}%;width:${w}%" aria-label="${T.midiName(m)}"><span class="kl"></span>${m % 12 === 0 ? `<span class="oct">Dó${Math.floor(m / 12) - 1}</span>` : ""}</button>`; });
  for (let m = from; m <= to; m++) {
    const pc = ((m % 12) + 12) % 12; if (WHITE.includes(pc)) continue;
    const idx = whites.indexOf(m - 1); if (idx < 0) continue;
    html += `<button class="bk" data-m="${m}" style="left:${(idx + 1) * w - w * .31}%;width:${w * .62}%" aria-label="${T.midiName(m)}"><span class="kl"></span></button>`;
  }
  return html + "</div>";
}
function kbMark(el, ms, cls = "on", labels) {
  if (!el) return;
  el.querySelectorAll("." + cls).forEach(k => { k.classList.remove(cls); if (cls === "on") k.querySelector(".kl").textContent = ""; });
  ms.forEach((m, i) => { const k = el.querySelector(`[data-m="${m}"]`); if (k) { k.classList.add(cls); if (labels) k.querySelector(".kl").textContent = labels[i] || ""; } });
}
function kbBind(el, onKey) {
  if (!el) return;
  el.querySelectorAll("[data-m]").forEach(k => k.addEventListener("pointerdown", e => {
    e.preventDefault(); const m = +k.dataset.m; AU.note(m, AU.now() - .04, .9);
    k.classList.add("hit"); setTimeout(() => k.classList.remove("hit"), 220);
    if (onKey) onKey(m, k);
  }));
}

// ---------- pauta (partitura simples) ----------
function staffSVG(notesStr, clef = "sol", o = {}) {
  const ns = String(notesStr).trim().split(/\s+/).map(T.parse).filter(Boolean);
  const gap = 10, top = 34, W = Math.max(150, 70 + ns.length * 36), H = 130;
  const bottomLine = top + 4 * gap;
  const ref = clef === "fa" ? 4 + 7 * 2 : 2 + 7 * 4; // Sol2 / Mi4 = linha de baixo
  let s = `<svg class="staff" viewBox="0 0 ${W} ${H}" width="100%" style="max-width:${W * 1.6}px" role="img" aria-label="Pauta com ${ns.map(T.pt).join(", ")}">`;
  for (let i = 0; i < 5; i++) s += `<line x1="6" x2="${W - 6}" y1="${top + i * gap}" y2="${top + i * gap}" class="sl"/>`;
  s += clef === "fa" ? `<text x="10" y="${top + 3 * gap + 3}" class="clef" font-size="44">𝄢</text>` : `<text x="8" y="${top + 4 * gap + 9}" class="clef" font-size="62">𝄞</text>`;
  ns.forEach((n, i) => {
    const idx = n.l + 7 * n.o - ref, x = 66 + i * 36, y = bottomLine - idx * gap / 2;
    for (let k = -2; k >= idx; k -= 2) s += `<line x1="${x - 11}" x2="${x + 11}" y1="${bottomLine - k * gap / 2}" y2="${bottomLine - k * gap / 2}" class="sl"/>`;
    for (let k = 10; k <= idx; k += 2) s += `<line x1="${x - 11}" x2="${x + 11}" y1="${bottomLine - k * gap / 2}" y2="${bottomLine - k * gap / 2}" class="sl"/>`;
    if (n.a) s += `<text x="${x - 20}" y="${y + 5}" class="acc" font-size="17">${n.a > 0 ? "♯" : "♭"}</text>`;
    s += `<ellipse cx="${x}" cy="${y}" rx="6.6" ry="4.9" transform="rotate(-20 ${x} ${y})" class="nh"/>`;
    const up = idx < 4; s += up ? `<line x1="${x + 6}" x2="${x + 6}" y1="${y - 1}" y2="${y - 30}" class="stem"/>` : `<line x1="${x - 6}" x2="${x - 6}" y1="${y + 1}" y2="${y + 30}" class="stem"/>`;
    if (o.labels) s += `<text x="${x}" y="${H - 6}" text-anchor="middle" class="nlab">${T.pt(n)}</text>`;
  });
  return s + "</svg>";
}

// ---------- grade de ritmo ----------
function rhythmHTML(pat, steps = 4, id) {
  const p = pat.replace(/[\s|]/g, ""), beats = Math.ceil(p.length / steps);
  let h = `<div class="rgrid" ${id ? `id="${id}"` : ""} style="--steps:${steps}">`;
  for (let b = 0; b < beats; b++) {
    h += `<div class="rbeat"><span class="rnum">${b + 1}</span><div class="rcells">`;
    for (let i = 0; i < steps; i++) { const ch = p[b * steps + i] || "."; h += `<span class="rc ${ch === "." ? "" : "on"} ${ch === "X" ? "acc" : ""}" data-i="${b * steps + i}"></span>`; }
    h += "</div></div>";
  }
  return h + "</div>";
}
function rhythmAnimate(el, info, len) {
  if (!el) return; const cells = el.querySelectorAll(".rc");
  for (let i = 0; i < len; i++) AU.later(() => { cells.forEach(c => c.classList.remove("now")); if (cells[i]) cells[i].classList.add("now"); }, info.start + i * info.step - AU.ctx.currentTime);
  AU.later(() => cells.forEach(c => c.classList.remove("now")), info.start + len * info.step - AU.ctx.currentTime + .1);
}

// ---------- tela inicial ----------
function ring(p) { const r = 34, c = 2 * Math.PI * r; return `<svg class="ring" viewBox="0 0 84 84" width="84" height="84" role="img" aria-label="${p}% concluído"><circle cx="42" cy="42" r="${r}" class="ring-bg"/><circle cx="42" cy="42" r="${r}" class="ring-fg" stroke-dasharray="${c.toFixed(1)}" stroke-dashoffset="${(c * (1 - p / 100)).toFixed(1)}" transform="rotate(-90 42 42)"/><text x="42" y="48" text-anchor="middle">${p}%</text></svg>`; }
function viewHome() {
  const Lv = level(), nx = nextLesson(), tot = ALL().length, done = ALL().filter(passed).length, pct = Math.round(done / tot * 100);
  const due = dueCards().length, ear = earSuggest(), earToday = (S.earDays || {})[today()] || 0;
  const t = nx ? trackOf(nx) : null, M = t ? MENTORS[t.mk] : null;
  app().innerHTML = `
  <section class="hero card">
    <div class="row" style="flex-wrap:nowrap;gap:16px">
      ${ring(pct)}
      <div class="grow">
        <p class="eyebrow">Escola Harpa · ${esc(Lv.name)}</p>
        <h1 class="display">A música por dentro</h1>
        <p class="small muted">${done}/${tot} lições aprovadas · ${streak()} dia(s) seguidos · ${S.xp} XP${Lv.next ? ` · faltam ${Lv.next - S.xp} para ${Lv.nextName}` : ""}</p>
      </div>
    </div>
  </section>
  ${nx ? `<a class="card next" href="#l-${nx}" style="--c1:${M.c1};--c2:${M.c2}">
    ${ava(t.mk, 56, mStage(t))}
    <div class="grow"><p class="eyebrow">Continuar · Nível ${t.n} · ${esc(t.title)}</p><h2>${esc(L(nx).title)}</h2><p class="small muted">${esc(L(nx).sub)}</p></div>
    <span class="go">${IC.play}</span></a>` : `<div class="card"><h2>Trilha completa!</h2><p class="muted">Você aprovou todas as lições. Busque nota máxima e mantenha o ouvido em dia.</p></div>`}
  <section class="card stack">
    <p class="eyebrow">Treino de hoje</p>
    <div class="mission"><span class="check ${S.les[nx] && S.les[nx].day === today() ? "done" : ""}">${IC.check}</span><div class="grow"><b>1 lição da trilha</b><p class="small muted">Stories + prova: teoria com som de verdade</p></div>${nx ? `<a class="btn pri" href="#l-${nx}">Estudar</a>` : ""}</div>
    <div class="mission"><span class="check ${earToday >= 10 ? "done" : ""}">${IC.check}</span><div class="grow"><b>10 exercícios de ouvido</b><p class="small muted">${earToday}/10 hoje · sugestão: ${esc(EAR[ear].name)} (nível ${earLv(ear)})</p></div><a class="btn" href="#o-${ear}">Treinar</a></div>
    <div class="mission"><span class="check ${cardsToday() >= 10 ? "done" : ""}">${IC.check}</span><div class="grow"><b>Revisar cards</b><p class="small muted">${cardsToday()}/10 hoje · ${due} para revisar</p></div><a class="btn" href="#cards">Revisar</a></div>
  </section>
  <section class="stack">
    <div class="row" style="justify-content:space-between"><p class="eyebrow">Seus mestres</p><a class="small" href="#trilha">Ver trilha</a></div>
    <div class="ments">${TRACKS.map(t => { const st = mStage(t), un = trackUnlocked(t); return `<a class="ment ${un ? "" : "off"}" href="${un ? "#l-" + (t.mods.find(id => !passed(id)) || t.mods[0]) : "#trilha"}">${ava(t.mk, 54, st)}<b>${esc(MENTORS[t.mk].name)}</b><span>${esc(t.short)}</span><em>${RANKS[st]}</em></a>`; }).join("")}</div>
  </section>
  <div class="grid2">
    <a class="card linkcard" href="#guia"><span class="lc-ic">${IC.book}</span><div><b>Guia da faculdade de música</b><p class="small muted">Disciplinas, cursos, carreiras e como este app cobre cada área</p></div></a>
    <a class="card linkcard" href="#lab"><span class="lc-ic">${IC.lab}</span><div><b>Laboratório</b><p class="small muted">Teclado, campo harmônico, círculo das quintas, metrônomo, ondas e mesa de mixagem</p></div></a>
  </div>`;
}

// ---------- trilha ----------
function viewTrilha() {
  app().innerHTML = `<header class="phead"><p class="eyebrow">Trilha</p><h1 class="display">7 níveis, 28 lições</h1><p class="muted">As áreas de uma faculdade de música, em ordem de aprendizado. Aprove 3 lições de um nível para abrir o próximo.</p></header>
  ${TRACKS.map(t => {
    const un = trackUnlocked(t), M = MENTORS[t.mk], st = mStage(t);
    return `<section class="track card ${un ? "" : "locked"}" style="--c1:${M.c1};--c2:${M.c2}">
      <div class="row" style="flex-wrap:nowrap">${ava(t.mk, 52, st)}<div class="grow"><p class="eyebrow">Nível ${t.n} · ${esc(M.name)} · ${RANKS[st]}</p><h2>${esc(t.title)}</h2><p class="small muted">${esc(t.desc)}</p></div></div>
      <div class="nodes">${t.mods.map((id, i) => {
        const ok = lessonUnlocked(id), cls = passed(id) ? "done" : ok ? "open" : "lock";
        return `<a class="node ${cls}" href="${ok ? "#l-" + id : "#trilha"}" style="--off:${[0, 38, 12, 52][i]}%">
          <span class="dotn">${passed(id) ? IC.check : ok ? i + 1 : IC.lock}</span>
          <span class="nlabel"><b>${esc(L(id).title)}</b><span>${[storyDone(id), passed(id), perfect(id), misDone(id)].map(x => `<i class="${x ? "on" : ""}"></i>`).join("")}</span></span></a>`;
      }).join("")}</div>
      ${un ? "" : `<p class="small muted lockmsg">${IC.lock} Aprove 3 lições de “${esc(TRACKS[t.n - 2].title)}” para abrir.</p>`}
    </section>`;
  }).join("")}`;
}

// ---------- página da lição ----------
function viewLesson(id) {
  const l = L(id); if (!l || !lessonUnlocked(id)) { location.replace("#trilha"); return; }
  const t = trackOf(id), M = MENTORS[t.mk], b = best(id);
  app().innerHTML = `<a class="back" href="#trilha">${IC.back} Trilha</a>
  <section class="card lhead" style="--c1:${M.c1};--c2:${M.c2}">
    <div class="row" style="flex-wrap:nowrap">${ava(t.mk, 60, mStage(t))}<div class="grow"><p class="eyebrow">Nível ${t.n} · ${esc(t.title)} · lição ${t.mods.indexOf(id) + 1}</p><h1>${esc(l.title)}</h1><p class="muted">${esc(l.sub)}</p></div></div>
    <ul class="goals">${l.goals.map(g => `<li>${esc(g)}</li>`).join("")}</ul>
    <div class="row"><a class="btn pri big" href="#s-${id}">${IC.play} ${storyDone(id) ? "Rever stories" : "Começar"}</a><a class="btn big" href="#p-${id}">Prova${b ? ` · ${Math.round(b * 100)}%` : ""}</a></div>
    <div class="pts">${[["Stories", storyDone(id)], ["Prova aprovada", passed(id)], ["Nota máxima", perfect(id)], ["Missão", misDone(id)]].map(([n, o]) => `<span class="pt ${o ? "on" : ""}">${o ? IC.check : ""}${n}</span>`).join("")}</div>
  </section>
  <section class="card stack">
    <p class="eyebrow">Missão prática</p><p>${fmt(l.mis)}</p>
    ${misDone(id) ? `<p class="okmsg">${IC.check} Missão cumprida</p>` : `<button class="btn" id="misBtn">Cumpri a missão (+15 XP)</button>`}
  </section>
  ${l.deep ? `<section class="card stack"><p class="eyebrow">Para ir mais fundo</p>${l.deep.map(d => `<p class="small">• ${fmt(d)}</p>`).join("")}</section>` : ""}`;
  const mb = $("#misBtn"); if (mb) mb.onclick = () => { S.mis[id] = today(); gain(15, "missão"); evoCheck(t); viewLesson(id); };
}
function evoCheck(t) { const st = mStage(t), was = S.mstage[t.id] || 0; if (st > was) { S.mstage[t.id] = st; save(); const M = MENTORS[t.mk]; modal(`<div class="center stack">${ava(t.mk, 96, st)}<p class="eyebrow">Novo título com ${esc(M.name)}</p><h2 class="display">${RANKS[st]}</h2><p class="muted">${esc(M.evo[st - 1] || "")}</p></div>`); } }

// ---------- stories ----------
let ST = null;
function openStory(id) {
  const l = L(id); if (!l || !lessonUnlocked(id)) { location.replace("#trilha"); return; }
  ST = { id, i: 0, gate: false, slides: l.s };
  $("#tabs").hidden = true; document.body.classList.add("storying");
  renderStory();
}
function closeStory() { AU.stop(); document.body.classList.remove("storying"); ST = null; }
function renderStory() {
  const l = L(ST.id), t = trackOf(ST.id), M = MENTORS[t.mk], sl = ST.slides[ST.i], n = ST.slides.length;
  AU.stop();
  app().innerHTML = `<div class="story" style="--c1:${M.c1};--c2:${M.c2}">
    <div class="sbars">${ST.slides.map((_, j) => `<span class="${j < ST.i ? "f" : j === ST.i ? "c" : ""}"></span>`).join("")}</div>
    <div class="shead">${ava(t.mk, 30, mStage(t))}<b>${esc(M.name)}</b><span class="muted small">${esc(l.title)}</span><a class="sx" href="#l-${ST.id}" aria-label="Fechar">${IC.x}</a></div>
    <div class="sbody" id="sbody">${slideHTML(sl)}</div>
    <div class="snav"><button class="btn" id="sPrev" ${ST.i === 0 ? "disabled" : ""}>${IC.back}</button><button class="btn pri grow" id="sNext">${ST.i === n - 1 ? "Concluir" : "Continuar"}</button></div>
  </div>`;
  ST.gate = !!GATED[sl.k];
  $("#sNext").disabled = ST.gate;
  $("#sPrev").onclick = () => { if (ST.i > 0) { ST.i--; renderStory(); } };
  $("#sNext").onclick = () => {
    if (ST.i < n - 1) { ST.i++; renderStory(); window.scrollTo(0, 0); return; }
    const id = ST.id, first = !storyDone(id);
    S.les[id] = Object.assign(S.les[id] || {}, { story: true, day: today() }); save();
    if (first) gain(20, "stories concluídos");
    evoCheck(trackOf(id));
    location.hash = "#p-" + id;
  };
  bindSlide(sl, $("#sbody"));
}
function unlock() { if (ST) { ST.gate = false; const b = $("#sNext"); if (b) { b.disabled = false; b.classList.add("pulse"); } } }
const GATED = { tf: 1, pick: 1, order: 1, match: 1, ear: 1, kb: 1, chat: 1, tap: 1 };

function slideHTML(s) {
  switch (s.k) {
    case "mentor": { const t = trackOf(ST.id); return `<div class="smentor">${ava(t.mk, 84, mStage(t))}<div class="bubble">${fmt(s.t)}</div></div>`; }
    case "txt": return `${s.tag ? `<p class="eyebrow">${esc(s.tag)}</p>` : ""}<h2 class="sh">${esc(s.h)}</h2><p class="sp">${fmt(s.p)}</p>${s.staff ? `<div class="staffbox" data-staff="${esc(s.staff)}" data-clef="${s.clef || "sol"}">${staffSVG(s.staff, s.clef, { labels: true })}</div>` : ""}`;
    case "list": return `<h2 class="sh">${esc(s.h)}</h2>${s.p ? `<p class="sp">${fmt(s.p)}</p>` : ""}<ol class="slist">${s.items.map(x => `<li>${fmt(x)}</li>`).join("")}</ol>`;
    case "table": return `<h2 class="sh">${esc(s.h)}</h2>${s.p ? `<p class="sp">${fmt(s.p)}</p>` : ""}<div class="tbl"><table>${s.head ? `<thead><tr>${s.head.map(x => `<th>${esc(x)}</th>`).join("")}</tr></thead>` : ""}<tbody>${s.rows.map((r, i) => `<tr ${s.play ? `data-row="${i}" class="trp"` : ""}>${r.map((x, j) => `<td>${s.play && j === 0 ? `<span class="tdp">${IC.play}</span>` : ""}${fmt(x)}</td>`).join("")}</tr>`).join("")}</tbody></table></div>`;
    case "big": return `<div class="sbig"><span class="bign">${esc(s.n)}</span><p class="sp">${fmt(s.p)}</p></div>`;
    case "quote": return `<blockquote class="squote">“${fmt(s.t)}”<cite>${esc(s.by)}</cite></blockquote>`;
    case "tf": return `<p class="eyebrow">Verdadeiro ou falso?</p><h2 class="sh">${fmt(s.q)}</h2><div class="opts two"><button class="opt" data-v="1">Verdadeiro</button><button class="opt" data-v="0">Falso</button></div><div class="fb" id="fb"></div>`;
    case "pick": { const ord = shuf(s.o.map((_, i) => i)); s._ord = ord; return `<p class="eyebrow">Responda</p><h2 class="sh">${fmt(s.q)}</h2>${s.staff ? staffSVG(s.staff, s.clef) : ""}<div class="opts">${ord.map(i => `<button class="opt" data-i="${i}">${fmt(s.o[i])}</button>`).join("")}</div><div class="fb" id="fb"></div>`; }
    case "ear": { const ord = shuf(s.o.map((_, i) => i)); return `<p class="eyebrow">Ouça e responda</p><h2 class="sh">${fmt(s.q)}</h2><div class="row">${btnPlay('id="earPlay"', "Ouvir")}</div><div class="opts">${ord.map(i => `<button class="opt" data-i="${i}">${fmt(s.o[i])}</button>`).join("")}</div><div class="fb" id="fb"></div>`; }
    case "order": { const it = shuf(s.items.map((x, i) => [x, i])); return `<p class="eyebrow">Toque na ordem certa</p><h2 class="sh">${esc(s.h)}</h2><ol class="built" id="built"></ol><div class="opts">${it.map(([x, i]) => `<button class="opt" data-i="${i}">${fmt(x)}</button>`).join("")}</div><div class="fb" id="fb"></div>`; }
    case "match": { const R = shuf(s.pairs.map((p, i) => [p[1], i])); return `<p class="eyebrow">Ligue os pares</p><h2 class="sh">${esc(s.h)}</h2><div class="match"><div>${s.pairs.map((p, i) => `<button class="opt ml" data-i="${i}">${fmt(p[0])}</button>`).join("")}</div><div>${R.map(([x, i]) => `<button class="opt mr" data-i="${i}">${fmt(x)}</button>`).join("")}</div></div><div class="fb" id="fb"></div>`; }
    case "play": return `${s.tag ? `<p class="eyebrow">${esc(s.tag)}</p>` : `<p class="eyebrow">Ouça</p>`}<h2 class="sh">${esc(s.h)}</h2>${s.p ? `<p class="sp">${fmt(s.p)}</p>` : ""}${s.staff ? `<div class="staffbox">${staffSVG(s.staff, s.clef, { labels: true })}</div>` : ""}<div class="pbtns">${s.b.map((b, i) => btnPlay(`data-b="${i}"`, b[0])).join("")}</div>${s.kb ? kbHTML(s.kb[0], s.kb[1], "skb") : ""}${s.after ? `<p class="small muted" style="margin-top:12px">${fmt(s.after)}</p>` : ""}`;
    case "kb": return `<p class="eyebrow">No teclado</p><h2 class="sh">${esc(s.h)}</h2>${s.p ? `<p class="sp">${fmt(s.p)}</p>` : ""}${kbHTML(s.r[0], s.r[1], "skb")}<p class="small muted" id="kbProg"></p><div class="fb" id="fb"></div>`;
    case "rhythm": return `<p class="eyebrow">Ritmo</p><h2 class="sh">${esc(s.h)}</h2>${s.p ? `<p class="sp">${fmt(s.p)}</p>` : ""}${(s.pats || [[null, s.pat]]).map((p, i) => `<div class="rblock">${p[0] ? `<p class="small"><b>${esc(p[0])}</b></p>` : ""}${rhythmHTML(p[1], s.steps || 4, "rg" + i)}${btnPlay(`data-r="${i}"`, "Ouvir")}</div>`).join("")}`;
    case "tap": return `<p class="eyebrow">Sua vez: toque no ritmo</p><h2 class="sh">${esc(s.h)}</h2>${s.p ? `<p class="sp">${fmt(s.p)}</p>` : ""}${rhythmHTML(s.pat, s.steps || 4, "rgt")}<div class="row">${btnPlay('id="tapListen"', "Ouvir antes")}<button class="btn pri" id="tapGo">Começar</button></div><button class="tappad" id="tapPad" disabled>Toque aqui</button><div class="fb" id="fb"></div>`;
    case "chat": return `<div class="chat"><div class="msg them"><span class="who" style="color:${s.c}">${esc(s.who)}</span>${fmt(s.q)}</div><div id="chatMore"></div></div><p class="eyebrow">Como você responde?</p><div class="opts">${shuf(s.o.map((_, i) => i)).map(i => `<button class="opt" data-i="${i}">${fmt(s.o[i].t)}</button>`).join("")}</div><div class="fb" id="fb"></div>`;
    case "staff": return `<p class="eyebrow">Partitura</p><h2 class="sh">${esc(s.h)}</h2>${s.p ? `<p class="sp">${fmt(s.p)}</p>` : ""}<div class="staffbox">${staffSVG(s.n, s.clef, { labels: s.labels !== false })}</div><div class="pbtns">${btnPlay('id="staffPlay"', "Ouvir")}</div>`;
  }
  return "";
}
function fb(ok, txt) { const f = $("#fb"); if (f) { f.className = "fb show " + (ok ? "ok" : "bad"); f.innerHTML = `<b>${ok ? "Isso!" : "Quase."}</b> ${fmt(txt || "")}`; } }
function bindSlide(s, el) {
  const opts = [...el.querySelectorAll(".opt")];
  const lockAll = () => opts.forEach(o => o.disabled = true);
  if (s.k === "tf") opts.forEach(b => b.onclick = () => { const ok = (b.dataset.v === "1") === s.a; b.classList.add(ok ? "right" : "wrong"); if (!ok) opts.find(o => (o.dataset.v === "1") === s.a).classList.add("right"); lockAll(); fb(ok, s.e); unlock(); });
  if (s.k === "pick" || s.k === "ear") {
    if (s.k === "ear") $("#earPlay").onclick = () => s.play.startsWith("fx:") ? playFx(s.play) : playSpec(s.play);
    opts.forEach(b => b.onclick = () => { const ok = +b.dataset.i === s.c; b.classList.add(ok ? "right" : "wrong"); if (!ok) opts.find(o => +o.dataset.i === s.c).classList.add("right"); lockAll(); fb(ok, s.e); unlock(); });
  }
  if (s.k === "order") { let n = 0, miss = 0; opts.forEach(b => b.onclick = () => { if (+b.dataset.i === n) { b.disabled = true; b.classList.add("used"); $("#built").insertAdjacentHTML("beforeend", `<li>${b.innerHTML}</li>`); n++; if (n === s.items.length) { fb(miss === 0, s.e || "Sequência completa."); unlock(); } } else { miss++; b.classList.add("shake"); setTimeout(() => b.classList.remove("shake"), 400); } }); }
  if (s.k === "match") {
    let selL = null, got = 0, miss = 0;
    el.querySelectorAll(".ml").forEach(b => b.onclick = () => { el.querySelectorAll(".ml").forEach(x => x.classList.remove("sel")); selL = b; b.classList.add("sel"); });
    el.querySelectorAll(".mr").forEach(b => b.onclick = () => {
      if (!selL) { toast("Escolha primeiro um item da esquerda"); return; }
      if (b.dataset.i === selL.dataset.i) { b.disabled = selL.disabled = true; b.classList.add("right"); selL.classList.add("right"); selL.classList.remove("sel"); selL = null; got++; if (got === s.pairs.length) { fb(miss === 0, s.e || "Todos os pares ligados."); unlock(); } }
      else { miss++; b.classList.add("shake"); setTimeout(() => b.classList.remove("shake"), 400); }
    });
  }
  if (s.k === "play") {
    const kb = $("#skb"); kbBind(kb);
    el.querySelectorAll("[data-b]").forEach(b => b.onclick = () => { const sp = s.b[+b.dataset.b][1]; el.querySelectorAll(".pbtn").forEach(x => x.classList.remove("playing")); b.classList.add("playing"); const d = sp.startsWith("fx:") ? playFx(sp) : playSpec(sp, kb ? ms => kbMark(kb, ms, "on", ms.map(m => T.midiName(m, /b/.test(sp)).replace(/-?\d+$/, ""))) : null); setTimeout(() => b.classList.remove("playing"), d * 1000); });
  }
  if (s.k === "kb") {
    const kb = $("#skb"), need = s.t.map(x => T.pc(T.parse(x))), got = new Set(); let miss = 0;
    const prog = () => { $("#kbProg").textContent = need.length > 1 ? `${got.size}/${need.length} notas` : ""; }; prog();
    kbBind(kb, (m, k) => {
      if (!ST || !ST.gate) return;
      const pc = m % 12;
      if (need.includes(pc)) { got.add(pc); k.classList.add("good"); prog(); if (got.size === need.length) { fb(miss < 2, s.e); unlock(); } }
      else { miss++; k.classList.add("badk"); setTimeout(() => k.classList.remove("badk"), 500); }
    });
  }
  if (s.k === "rhythm") el.querySelectorAll("[data-r]").forEach(b => b.onclick = () => { const p = (s.pats || [[null, s.pat]])[+b.dataset.r][1]; AU.stop(); const info = AU.rhythm(p, s.bpm || 80, { steps: s.steps || 4, beats: s.beats || 4, metro: true }); rhythmAnimate($("#rg" + b.dataset.r), info, p.replace(/[\s|]/g, "").length); });
  if (s.k === "tap") bindTap(s);
  if (s.k === "chat") opts.forEach(b => b.onclick = () => { const o = s.o[+b.dataset.i]; $("#chatMore").innerHTML = `<div class="msg me">${fmt(o.t)}</div><div class="msg them"><span class="who" style="color:${s.c}">${esc(s.who)}</span>${fmt(o.r)}</div>`; b.classList.add(o.v === 2 ? "right" : "wrong"); lockAll(); fb(o.v === 2, o.tip); unlock(); });
  if (s.k === "staff") $("#staffPlay").onclick = () => playSpec((s.chord ? "[" + s.n + "]" : s.n) + " bpm=" + (s.bpm || 84));
  if (s.k === "table" && s.play) el.querySelectorAll("[data-row]").forEach(r => r.onclick = () => { const sp = s.play[+r.dataset.row]; if (sp) sp.startsWith("fx:") ? playFx(sp) : playSpec(sp); });
}
function bindTap(s) {
  const pat = s.pat.replace(/[\s|]/g, ""), steps = s.steps || 4, bpm = s.bpm || 80, sp = 60 / bpm / steps;
  const expect = []; for (let i = 0; i < pat.length; i++) if (pat[i] !== ".") expect.push(i * sp);
  $("#tapListen").onclick = () => { AU.stop(); const info = AU.rhythm(s.pat, bpm, { steps, metro: true }); rhythmAnimate($("#rgt"), info, pat.length); };
  let taps = [], start = 0, live = false;
  const pad = $("#tapPad");
  const hit = e => { e.preventDefault(); if (!live) return; taps.push(AU.ctx.currentTime - start); AU.wood(AU.ctx.currentTime, false); pad.classList.add("hit"); setTimeout(() => pad.classList.remove("hit"), 90); };
  pad.addEventListener("pointerdown", hit);
  $("#tapGo").onclick = () => {
    AU.stop(); AU.init(); taps = []; live = false; pad.disabled = false; pad.textContent = "Prepare… 4 batidas";
    const beat = 60 / bpm, t0 = AU.now() + .1;
    for (let i = 0; i < 4; i++) AU.click(t0 + i * beat, i === 0);
    for (let i = 0; i < pat.length / steps; i++) AU.click(t0 + (4 + i) * beat, false); // metrônomo durante o toque
    start = t0 + 4 * beat;
    AU.later(() => { live = true; pad.textContent = "Toque!"; }, start - AU.ctx.currentTime - beat * .4);
    AU.later(() => {
      live = false; pad.disabled = true; pad.textContent = "Toque aqui";
      let err = 0, hits = 0; const used = new Set();
      expect.forEach(e => { let bi = -1, bd = 1; taps.forEach((t, j) => { const d = Math.abs(t - e); if (!used.has(j) && d < bd) { bd = d; bi = j; } }); if (bi >= 0 && bd < sp * 1.6) { used.add(bi); hits++; err += bd; } });
      const extra = taps.length - used.size, avg = hits ? Math.round(err / hits * 1000) : 0;
      const ok = hits === expect.length && extra <= 1 && avg < 90;
      fb(ok, `Você acertou ${hits} de ${expect.length} toques${hits ? `, com desvio médio de ${avg} ms` : ""}${extra > 0 ? ` e deu ${extra} toque(s) a mais` : ""}. ${ok ? "Pulso firme!" : "Tente de novo: conte em voz alta e sinta o tempo no corpo."}`);
      unlock();
    }, start + pat.length * sp + beat * .8 - AU.ctx.currentTime);
  };
}

// ---------- prova ----------
let QZ = null;
function openQuiz(id) {
  const l = L(id); if (!l || !lessonUnlocked(id)) { location.replace("#trilha"); return; }
  closeStory(); tabs("trilha");
  QZ = { id, qs: shuf(l.q).map(q => Object.assign({}, q, { ord: shuf(q.o.map((_, i) => i)) })), i: 0, ok: 0, done: false };
  renderQuiz();
}
function renderQuiz() {
  const q = QZ.qs[QZ.i], n = QZ.qs.length, t = trackOf(QZ.id);
  if (QZ.done) {
    const sc = QZ.ok / n, prev = best(QZ.id), was = passed(QZ.id);
    if (!QZ.saved) {
      QZ.saved = true;
      S.les[QZ.id] = Object.assign(S.les[QZ.id] || {}, { best: Math.max(prev, sc), day: today() }); save();
      let xp = QZ.ok * 4; if (sc >= PASS && !was) xp += 30; if (sc === 1 && prev < 1) xp += 20;
      gain(xp, "prova");
    }
    const nx = nextLesson();
    app().innerHTML = `<section class="card center stack result">
      ${ava(t.mk, 84, mStage(t))}
      <p class="eyebrow">Prova · ${esc(L(QZ.id).title)}</p>
      <h1 class="display">${QZ.ok}/${n}</h1>
      <p>${sc === 1 ? "Nota máxima! " + esc(MENTORS[t.mk].name) + " está orgulhoso." : sc >= PASS ? "Aprovado! Revise o que errou e siga em frente." : "Ainda não. Reveja os stories e tente de novo: errar faz parte do treino."}</p>
      <div class="row" style="justify-content:center"><a class="btn" href="#l-${QZ.id}">Voltar à lição</a>${sc < 1 ? `<button class="btn" id="again">Refazer</button>` : ""}${nx && nx !== QZ.id ? `<a class="btn pri" href="#l-${nx}">Próxima lição</a>` : ""}</div>
    </section>`;
    const a = $("#again"); if (a) a.onclick = () => openQuiz(QZ.id);
    evoCheck(t); return;
  }
  app().innerHTML = `<a class="back" href="#l-${QZ.id}">${IC.back} Sair da prova</a>
  <section class="card stack quiz">
    <div class="row" style="justify-content:space-between"><p class="eyebrow">Prova · ${esc(L(QZ.id).title)}</p><span class="small muted">${QZ.i + 1}/${n}</span></div>
    <div class="qbar"><span style="width:${QZ.i / n * 100}%"></span></div>
    <h2>${fmt(q.q)}</h2>
    ${q.staff ? staffSVG(q.staff, q.clef) : ""}
    ${q.play ? `<div class="row">${btnPlay('id="qPlay"', "Ouvir")}</div>` : ""}
    <div class="opts">${q.ord.map(i => `<button class="opt" data-i="${i}">${fmt(q.o[i])}</button>`).join("")}</div>
    <div class="fb" id="fb"></div>
    <button class="btn pri" id="qNext" hidden>${QZ.i === n - 1 ? "Ver resultado" : "Próxima"}</button>
  </section>`;
  if (q.play) $("#qPlay").onclick = () => q.play.startsWith("fx:") ? playFx(q.play) : playSpec(q.play);
  const opts = [...app().querySelectorAll(".opt")];
  opts.forEach(b => b.onclick = () => { const ok = +b.dataset.i === q.c; if (ok) QZ.ok++; b.classList.add(ok ? "right" : "wrong"); if (!ok) opts.find(o => +o.dataset.i === q.c).classList.add("right"); opts.forEach(o => o.disabled = true); fb(ok, q.e); const nb = $("#qNext"); nb.hidden = false; nb.focus(); });
  $("#qNext").onclick = () => { QZ.i++; if (QZ.i >= n) QZ.done = true; renderQuiz(); window.scrollTo(0, 0); };
}

// ---------- academia de ouvido ----------
function earLv(k) { return (S.ear[k] && S.ear[k].lv) || 1; }
function earStat(k) { return S.ear[k] || { lv: 1, ok: 0, n: 0 }; }
function earSuggest() { const ks = Object.keys(EAR); let bestK = ks[0], low = 2; ks.forEach(k => { const s = earStat(k), acc = s.n ? s.ok / s.n : -1 + Math.random() * .1; const score = acc + s.n / 400; if (score < low) { low = score; bestK = k; } }); return bestK; }
function viewOuvido() {
  app().innerHTML = `<header class="phead"><p class="eyebrow">Academia de ouvido</p><h1 class="display">Percepção musical</h1><p class="muted">Exercícios gerados na hora, como nas aulas de percepção da faculdade. 10 perguntas por rodada; acerte 8 para subir de nível. Use fone de ouvido se puder.</p></header>
  <div class="eargrid">${Object.entries(EAR).map(([k, e]) => { const s = earStat(k), acc = s.n ? Math.round(s.ok / s.n * 100) : null; return `<a class="card earcard" href="#o-${k}" style="--c1:${e.c1};--c2:${e.c2}"><span class="earic">${e.ic}</span><div class="grow"><b>${esc(e.name)}</b><p class="small muted">${esc(e.desc)}</p><p class="small"><span class="lvpill">Nível ${s.lv}/${e.levels.length}</span>${acc != null ? ` <span class="muted">· ${acc}% de acerto em ${s.n}</span>` : ""}</p></div></a>`; }).join("")}</div>`;
}
let ES = null;
function openEar(k) {
  if (!EAR[k]) { location.replace("#ouvido"); return; }
  ES = { k, lv: earLv(k), i: 0, ok: 0, n: 10, q: null, answered: false };
  nextEarQ();
}
function nextEarQ() { ES.q = EAR[ES.k].gen(ES.lv); ES.answered = false; renderEar(); setTimeout(() => { if (ES && !ES.answered) ES.q.play(); }, 350); }
function renderEar() {
  const e = EAR[ES.k], q = ES.q;
  if (ES.i >= ES.n) {
    const st = earStat(ES.k), up = ES.ok >= 8 && st.lv < e.levels.length;
    if (!ES.saved) { ES.saved = true; if (up) st.lv++; S.ear[ES.k] = st; save(); gain(ES.ok * 2 + (up ? 20 : 0), "ouvido"); }
    app().innerHTML = `<section class="card center stack result" style="--c1:${e.c1};--c2:${e.c2}"><span class="earic big">${e.ic}</span><p class="eyebrow">${esc(e.name)} · nível ${ES.lv}</p><h1 class="display">${ES.ok}/10</h1><p>${up ? `Subiu para o nível ${st.lv}: <b>${esc(e.levels[st.lv - 1])}</b>` : ES.ok >= 8 ? "Excelente! Você está no nível máximo desta área." : "Continue: o ouvido aprende com repetição espaçada, um pouco todo dia."}</p><div class="row" style="justify-content:center"><a class="btn" href="#ouvido">Outras áreas</a><button class="btn pri" id="again">Mais uma rodada</button></div></section>`;
    $("#again").onclick = () => openEar(ES.k); return;
  }
  app().innerHTML = `<a class="back" href="#ouvido">${IC.back} Academia</a>
  <section class="card stack quiz" style="--c1:${e.c1};--c2:${e.c2}">
    <div class="row" style="justify-content:space-between"><p class="eyebrow">${esc(e.name)} · nível ${ES.lv}: ${esc(e.levels[ES.lv - 1])}</p><span class="small muted">${ES.i + 1}/${ES.n} · ${ES.ok} certas</span></div>
    <div class="qbar"><span style="width:${ES.i / ES.n * 100}%"></span></div>
    <h2>${fmt(q.prompt)}</h2>
    ${q.visual || ""}
    <div class="row">${btnPlay('id="ePlay"', "Ouvir de novo")}${q.extra ? q.extra : ""}</div>
    <div class="opts ${q.o.length > 4 ? "many" : ""}">${q.o.map((o, i) => `<button class="opt" data-i="${i}">${o.startsWith("<") ? o : fmt(o)}</button>`).join("")}</div>
    <div class="fb" id="fb"></div>
    <button class="btn pri" id="eNext" hidden>${ES.i === ES.n - 1 ? "Ver resultado" : "Próxima"}</button>
  </section>`;
  $("#ePlay").onclick = () => q.play();
  if (q.bindExtra) q.bindExtra();
  const opts = [...app().querySelectorAll(".opt")];
  opts.forEach(b => b.onclick = () => {
    if (ES.answered) { const i = +b.dataset.i; if (q.hear) q.hear(i); return; }
    ES.answered = true;
    const ok = +b.dataset.i === q.c; if (ok) ES.ok++;
    const st = earStat(ES.k); st.n++; if (ok) st.ok++; S.ear[ES.k] = st; S.earDays = S.earDays || {}; S.earDays[today()] = (S.earDays[today()] || 0) + 1; save();
    b.classList.add(ok ? "right" : "wrong"); if (!ok) opts[q.c].classList.add("right");
    fb(ok, (q.e || "") + (q.hear ? " _Toque nas opções para comparar o som de cada uma._" : ""));
    const nb = $("#eNext"); nb.hidden = false;
  });
  $("#eNext").onclick = () => { ES.i++; if (ES.i >= ES.n) renderEar(); else nextEarQ(); window.scrollTo(0, 0); };
}

// geradores de exercício
const rnd = (a, b) => a + Math.floor(Math.random() * (b - a + 1));
function optsFrom(correct, pool, n = 4) { const others = shuf(pool.filter(x => x !== correct)).slice(0, n - 1); const o = shuf([correct, ...others]); return { o, c: o.indexOf(correct) }; }
const KEYS = ["C4", "D4", "Eb4", "E4", "F4", "G3", "A3", "Bb3"];
function cadence(rootMidi, t0, b) { // I – IV – V – I
  const chords = [[0, 4, 7], [5, 9, 12], [7, 11, 14], [0, 4, 7]], bass = [0, 5, -5, 0];
  chords.forEach((c, i) => { AU.chord(c.map(x => rootMidi + x), t0 + i * b, b * .95, { v: .6 }); AU.note(rootMidi - 12 + bass[i], t0 + i * b, b * .95, { v: .6 }); });
  return t0 + 4 * b;
}
const EAR = {
  intervalos: {
    name: "Intervalos", ic: "↕", c1: "#7B5CFF", c2: "#C08CFF", desc: "A distância entre duas notas: a base de toda a percepção.",
    levels: ["Ascendentes fáceis", "Ascendentes com terças", "Todos os ascendentes", "Descendentes", "Harmônicos (juntos)"],
    gen(lv) {
      const sets = [["2M", "3M", "4J", "5J", "8J"], ["2M", "3m", "3M", "4J", "5J", "6M", "8J"], IV.slice(1).map(i => i[0]), IV.slice(1).map(i => i[0]), IV.slice(1).map(i => i[0])];
      const pool = sets[lv - 1], k = pick(pool), iv = IVK[k], root = rnd(55, 67), dir = lv === 4 ? -1 : 1, harm = lv === 5;
      const a = dir > 0 ? root : root + 5, b = a + dir * iv[3];
      const play = () => { AU.stop(); const t = AU.now(); if (harm) { AU.chord([a, b], t, 1.6); } else { AU.note(a, t, .8); AU.note(b, t + .75, 1); } };
      const { o, c } = optsFrom(k, pool, Math.min(pool.length, lv >= 3 ? 6 : 4));
      return { prompt: harm ? "Qual intervalo soa quando as duas notas tocam juntas?" : `Qual a distância entre as duas notas (${dir > 0 ? "subindo" : "descendo"})?`, play, o: o.map(x => IVK[x][1]), c,
        e: `Foi **${iv[1]}** (${iv[3]} semitons): de ${T.midiName(a)} para ${T.midiName(b)}.${REF_IV[k] ? " Referência: " + REF_IV[k] + "." : ""}`,
        hear: i => { AU.stop(); const t = AU.now(), bb = a + dir * IVK[o[i]][3]; if (harm) AU.chord([a, bb], t, 1.4); else { AU.note(a, t, .7); AU.note(bb, t + .65, .9); } } };
    }
  },
  acordes: {
    name: "Acordes", ic: "♯", c1: "#E8590C", c2: "#FFB067", desc: "Maior, menor, diminuto, aumentado e as tétrades com sétima.",
    levels: ["Maior × menor", "+ diminuto e aumentado", "+ suspensos", "Tétrades (7, 7M, m7)", "Todas as tétrades"],
    gen(lv) {
      const sets = [["M", "m"], ["M", "m", "dim", "aug"], ["M", "m", "dim", "aug", "sus2", "sus4"], ["7", "7M", "m7", "M", "m"], ["7", "7M", "m7", "m7b5", "dim7"]];
      const pool = sets[lv - 1], k = pick(pool), root = T.parse(pick(["C4", "D4", "E4", "F4", "G3", "A3", "Bb3", "Eb4"])), ms = T.build(root, CHORDS[k][2]).map(T.midi);
      const play = () => { AU.stop(); const t = AU.now(); AU.chord(ms, t, 1.4); ms.forEach((m, i) => AU.note(m, t + 1.6 + i * .32, .6)); AU.chord(ms, t + 1.6 + ms.length * .32 + .1, 1.2); };
      const { o, c } = optsFrom(k, pool, Math.min(4, pool.length));
      return { prompt: "Qual é o tipo deste acorde?", play, o: o.map(x => CHORDS[x][0]), c, e: `Era **${CHORDS[k][0]}**: ${T.build(root, CHORDS[k][2]).map(T.pt).join(" – ")} (${T.cif(root)}${CHORDS[k][1]}). ${CHORD_FEEL[k] || ""}`,
        hear: i => { AU.stop(); AU.chord(T.build(root, CHORDS[o[i]][2]).map(T.midi), AU.now(), 1.3); } };
    }
  },
  escalas: {
    name: "Escalas e modos", ic: "≋", c1: "#0E9F6E", c2: "#5EEAD4", desc: "O “cardápio” de notas de cada música e o colorido de cada modo.",
    levels: ["Maior × menor", "Menores (natural, harmônica, melódica)", "Pentatônicas e blues", "Modos gregos"],
    gen(lv) {
      const sets = [["maior", "menor"], ["maior", "menor", "harmonica", "melodica"], ["maior", "pentaM", "pentam", "blues"], ["maior", "dorico", "frigio", "lidio", "mixolidio", "menor"]];
      const pool = sets[lv - 1], k = pick(pool), root = T.parse(pick(["C4", "D4", "F4", "G3", "A3"]));
      const notes = n => { const sc = T.build(root, SCALES[n][1]); return [...sc, { l: root.l, a: root.a, o: root.o + 1 }].map(T.midi); };
      const run = (ms) => { AU.stop(); const t = AU.now(); AU.note(ms[0] - 12, t, ms.length * .3 + .4, { v: .4 }); ms.forEach((m, i) => AU.note(m, t + i * .3, .34)); };
      const { o, c } = optsFrom(k, pool, Math.min(4, pool.length));
      return { prompt: "Que escala é esta?", play: () => run(notes(k)), o: o.map(x => SCALES[x][0]), c, e: `Era a **${SCALES[k][0]}** de ${T.pt(root)}: ${T.build(root, SCALES[k][1]).map(T.pt).join(" ")}. ${SCALE_FEEL[k] || ""}`, hear: i => run(notes(o[i])) };
    }
  },
  graus: {
    name: "Graus da tonalidade", ic: "①", c1: "#1C7ED6", c2: "#74C0FC", desc: "Ouvido funcional: depois da cadência, qual grau da escala soou? É assim que se tira música de ouvido.",
    levels: ["Dó, Mi, Sol (1, 3, 5)", "+ Ré e Fá (2, 4)", "Escala maior inteira", "Escala maior em 2 oitavas"],
    gen(lv) {
      const DEG = [["1", "Dó (1)"], ["2", "Ré (2)"], ["3", "Mi (3)"], ["4", "Fá (4)"], ["5", "Sol (5)"], ["6", "Lá (6)"], ["7", "Si (7)"]], SEMI = [0, 2, 4, 5, 7, 9, 11];
      const sets = [[0, 2, 4], [0, 1, 2, 3, 4], [0, 1, 2, 3, 4, 5, 6], [0, 1, 2, 3, 4, 5, 6]], pool = sets[lv - 1];
      const key = T.midi(T.parse(pick(KEYS))), d = pick(pool), oct = lv === 4 ? pick([-12, 0, 12]) : 0, target = key + SEMI[d] + oct;
      const play = () => { AU.stop(); const t = AU.now(), end = cadence(key, t, .62); AU.note(target, end + .35, 1.4, { v: .9 }); };
      const o = pool.map(i => DEG[i][1]), c = pool.indexOf(d);
      return { prompt: "Depois da cadência (I–IV–V–I), qual grau da escala soou? (dó móvel)", play, o, c,
        e: `Era o grau **${DEG[d][1]}**. ${DEG_FEEL[d]}`, hear: i => { AU.stop(); const t = AU.now(); AU.note(key, t, .6, { v: .5 }); AU.note(key + SEMI[pool[i]] + oct, t + .65, 1); } };
    }
  },
  progressoes: {
    name: "Progressões", ic: "⟿", c1: "#C2255C", c2: "#F783AC", desc: "Sequências de acordes que aparecem em milhares de músicas e louvores.",
    levels: ["I–IV–V e companhia", "Com o vi (relativo menor)", "Com ii e cadências do jazz"],
    gen(lv) {
      const P = { "I – IV – V – I": [1, 4, 5, 1], "I – V – I – V": [1, 5, 1, 5], "I – IV – I – V": [1, 4, 1, 5], "I – vi – IV – V": [1, 6, 4, 5], "I – V – vi – IV": [1, 5, 6, 4], "vi – IV – I – V": [6, 4, 1, 5], "I – IV – vi – V": [1, 4, 6, 5], "ii – V – I – I": [2, 5, 1, 1], "I – vi – ii – V": [1, 6, 2, 5], "I – iii – IV – V": [1, 3, 4, 5] };
      const sets = [["I – IV – V – I", "I – V – I – V", "I – IV – I – V"], ["I – IV – V – I", "I – vi – IV – V", "I – V – vi – IV", "vi – IV – I – V", "I – IV – vi – V"], ["ii – V – I – I", "I – vi – ii – V", "I – iii – IV – V", "I – V – vi – IV", "I – vi – IV – V"]];
      const pool = sets[lv - 1], k = pick(pool), key = T.parse(pick(["C4", "D4", "G3", "F3", "A3"])), cf = campo(key, "maior");
      const voice = d => { const ch = cf[d - 1].notes.map(T.midi); let ms = ch.map(m => m); while (Math.min(...ms) > 62) ms = ms.map(m => m - 12); while (Math.min(...ms) < 55) { const lo = Math.min(...ms); ms = ms.map(m => m === lo ? m + 12 : m); } let b = T.midi(cf[d - 1].notes[0]); while (b > 52) b -= 12; while (b < 40) b += 12; return { ms, bass: b }; };
      const run = seq => { AU.stop(); const t = AU.now(), b = .85; seq.forEach((d, i) => { const v = voice(d); AU.chord(v.ms, t + i * b, b * .95, { v: .6 }); AU.note(v.bass, t + i * b, b * .95, { v: .7 }); }); };
      const { o, c } = optsFrom(k, pool, Math.min(4, pool.length));
      return { prompt: `Qual progressão tocou? (tom de ${T.pt(key)} maior)`, play: () => run(P[k]), o, c, e: `Era **${k}**: ${P[k].map(d => cf[d - 1].nome).join(" – ")}.`, hear: i => run(P[o[i]]) };
    }
  },
  ritmo: {
    name: "Ritmo", ic: "♩", c1: "#F08C00", c2: "#FFD43B", desc: "Ditado rítmico: ouça um compasso e reconheça a grade certa.",
    levels: ["Semínimas e colcheias", "+ semicolcheias", "Contratempo e síncope", "Colcheia pontuada e mistura"],
    gen(lv) {
      const cells = [["x...", "x.x.", "...."], ["x...", "x.x.", "xxxx", "x.xx", "xx.x"], ["x...", "x.x.", "..x.", "xxxx", ".xx.", "x.xx"], ["x..x", "x.xx", "xx.x", "..x.", ".x.x", "xxxx", "x..."]][lv - 1];
      const mk = () => { let p; do { p = [pick(cells.filter(c => c[0] === "x")), pick(cells), pick(cells), pick(cells)].join(" "); } while (!p.includes("x")); return p; };
      const k = mk(), set = new Set([k]); while (set.size < 4) set.add(mk());
      const o = shuf([...set]), c = o.indexOf(k), bpm = lv >= 3 ? 76 : 84;
      return { prompt: "Depois da contagem de 4, qual ritmo tocou?", play: () => { AU.stop(); AU.rhythm(k, bpm, { metro: true }); }, o: o.map(p => rhythmHTML(p, 4)), c, e: "Cada quadrado é uma semicolcheia; quadrado aceso = toque. Agrupe de 4 em 4 (um tempo) e conte “1-e-&-a”.",
        hear: i => { AU.stop(); AU.rhythm(o[i], bpm, { metro: true }); } };
    }
  },
  afinacao: {
    name: "Afinação fina", ic: "≈", c1: "#2B8A3E", c2: "#8CE99A", desc: "A segunda nota é mais aguda ou mais grave? Diferenças de poucos cents.",
    levels: ["50 cents (¼ de tom)", "30 cents", "20 cents", "12 cents", "6 cents (ouvido de afinador)"],
    gen(lv) {
      const cents = [50, 30, 20, 12, 6][lv - 1], dir = pick([1, -1]), f = T.hz(rnd(57, 69)), w = pick(["triangle", "sine", "piano"]);
      const play = () => { AU.stop(); const t = AU.now(); AU.tone(f, t, 1, { w }); AU.tone(f, t + 1.2, 1, { w, cents: cents * dir }); };
      return { prompt: "A segunda nota está mais aguda ou mais grave que a primeira?", play, o: ["Mais aguda (alta)", "Mais grave (baixa)"], c: dir > 0 ? 0 : 1,
        e: `Ela estava ${cents} cents ${dir > 0 ? "acima" : "abaixo"}. Um semitom tem 100 cents. Cantores e violinistas corrigem isso o tempo todo, de ouvido.` };
    }
  },
  timbre: {
    name: "Timbre e ondas", ic: "∿", c1: "#5F3DC4", c2: "#B197FC", desc: "A “cor” do som: reconheça a forma de onda pelos harmônicos.",
    levels: ["Senoide × dente de serra", "Quatro formas de onda", "Ondas em notas diferentes"],
    gen(lv) {
      const W = { sine: "Senoide (só a fundamental)", triangle: "Triangular (harmônicos ímpares fracos)", square: "Quadrada (harmônicos ímpares fortes)", sawtooth: "Dente de serra (todos os harmônicos)" };
      const pool = lv === 1 ? ["sine", "sawtooth"] : ["sine", "triangle", "square", "sawtooth"], k = pick(pool), m = lv === 3 ? rnd(45, 72) : 57;
      return { prompt: "Qual forma de onda está tocando?", play: () => { AU.stop(); AU.note(m, AU.now(), 1.6, { w: k }); }, o: pool.map(x => W[x]), c: pool.indexOf(k),
        e: "Mesma nota, mesma altura: só os **harmônicos** mudam. É isso que faz flauta, violino e trompete soarem diferentes.", hear: i => { AU.stop(); AU.note(m, AU.now(), 1.2, { w: pool[i] }); } };
    }
  },
  eq: {
    name: "Ouvido de engenheiro", ic: "◠", c1: "#0B7285", c2: "#66D9E8", desc: "Ruído plano e depois com reforço de +12 dB numa faixa: que frequência subiu?",
    levels: ["Grave, médio ou agudo", "4 faixas (125 Hz a 8 kHz)", "7 faixas em oitavas", "Cortes (−12 dB) em 7 faixas"],
    gen(lv) {
      const sets = [[120, 1000, 8000], [125, 500, 2000, 8000], [125, 250, 500, 1000, 2000, 4000, 8000], [125, 250, 500, 1000, 2000, 4000, 8000]];
      const pool = sets[lv - 1], f = pick(pool), g = lv === 4 ? -12 : 12;
      const lab = x => (x >= 1000 ? x / 1000 + " kHz" : x + " Hz") + " · " + bandName(x);
      return { prompt: `Primeiro o ruído plano, depois com ${g > 0 ? "reforço" : "corte"} numa faixa. Qual faixa?`, play: () => FX.eq(f, g), o: pool.map(lab), c: pool.indexOf(f),
        e: `Era **${lab(f)}**. ${BAND_TIP(f)}`, hear: i => FX.eq(pool[i], g) };
    }
  }
};
function bandName(f) { return f < 200 ? "grave" : f < 600 ? "médio-grave" : f < 1500 ? "médio" : f < 5000 ? "médio-agudo" : "agudo"; }
function BAND_TIP(f) { return f < 200 ? "Graves dão peso: bumbo, baixo, o “corpo” da voz masculina." : f < 400 ? "250 Hz é a região do “embolado/abafado”: excesso aqui deixa a mixagem turva." : f < 800 ? "500 Hz soa “caixa de papelão”: corpo dos instrumentos, mas cansa em excesso." : f < 1500 ? "1 kHz soa “nasal/telefone”: muito presente no ouvido humano." : f < 3000 ? "2 kHz é presença e ataque: clareza das palavras e das guitarras." : f < 6000 ? "4 kHz é brilho agressivo: inteligibilidade, mas fadiga se exagerar." : "8 kHz é o “ar”: sibilância (s), pratos e brilho."; }
const REF_IV = { "2M": "“Pa-ra-béns” (Parabéns a você)", "3M": "início de “Oh When the Saints”", "4J": "início da Marcha Nupcial (“Here Comes the Bride”)", "5J": "“Bri-lha” em “Brilha, brilha, estrelinha”", "8J": "“Some-where” em “Over the Rainbow”", "2m": "o tema do filme Tubarão", "3m": "início de “Greensleeves”", "4A": "início do tema de Os Simpsons", "6M": "início de “My Bonnie”", "7m": "início do tema clássico de Star Trek" };
const CHORD_FEEL = { M: "Soa estável e luminoso.", m: "Soa mais escuro, introspectivo.", dim: "Tenso e instável: pede resolução.", aug: "Flutuante, de suspense, sem chão.", sus2: "Aberto, sem terça: nem maior nem menor.", sus4: "A quarta “quer” descer para a terça.", "7": "Dominante: puxa forte para a tônica.", "7M": "Sofisticado e doce: bossa nova e louvor moderno.", m7: "Menor suave, muito usado no ii grau.", m7b5: "Meio-diminuto: o ii do tom menor.", dim7: "Tensão simétrica, cheia de suspense." };
const SCALE_FEEL = { maior: "Brilhante, “resolvida”.", menor: "Escura, melancólica.", harmonica: "A sensível (7ª elevada) dá o sabor “árabe/clássico”.", melodica: "Sobe com 6ª e 7ª maiores: menor e elegante.", pentaM: "5 notas sem semitons: folk, gospel, música do mundo inteiro.", pentam: "A base dos solos de rock e blues.", blues: "A “blue note” (4ª aumentada) dá o tempero.", dorico: "Menor com 6ª maior: menor “esperançoso”.", frigio: "2ª menor: som espanhol/flamenco, tenso.", lidio: "4ª aumentada: sonhador, trilha de filme.", mixolidio: "Maior com 7ª menor: rock, baião, forró." };
const DEG_FEEL = ["O 1 (tônica) é o “lar”: soa resolvido.", "O 2 soa em movimento, querendo cair no 1 ou subir para o 3.", "O 3 é estável e doce: a terça do acorde da tônica.", "O 4 é tenso, quer descer para o 3.", "O 5 é forte e estável, um “segundo lar”.", "O 6 é suave e um pouco melancólico (é a tônica do relativo menor).", "O 7 é a **sensível**: muito tenso, quer subir para o 1."];

// ---------- cards (revisão espaçada) ----------
function dueCards() { const k = today(); return CARDS.filter(c => S.cards[c.id] && S.cards[c.id].due <= k); }
function cardsToday() { return Object.values(S.cards).filter(c => c.last === today()).length; }
let CQ = null;
function viewCards() {
  const due = dueCards(), nw = CARDS.filter(c => !S.cards[c.id]), cats = [...new Set(CARDS.map(c => c.cat))];
  if (!CQ || !CQ.length) CQ = [...due, ...nw.slice(0, Math.max(0, 10 - Object.values(S.cards).filter(c => c.added === today()).length))];
  const c = CQ[0];
  app().innerHTML = `<header class="phead"><p class="eyebrow">Cards · revisão espaçada</p><h1 class="display">Vocabulário do músico</h1><p class="muted">${CARDS.length} termos · ${Object.keys(S.cards).length} já estudados · ${due.length} para revisar hoje.</p></header>
  ${c ? `<section class="card flash" id="flash"><p class="eyebrow">${esc(c.cat)}</p><h2 class="display">${fmt(c.f)}</h2><div class="back" id="cb" hidden><p>${fmt(c.b)}</p>${c.p ? `<div class="row" style="margin-top:12px">${btnPlay('id="cPlay"', "Ouvir")}</div>` : ""}</div>
    <div class="row" id="cAct"><button class="btn pri big grow" id="cShow">Mostrar resposta</button></div></section>` : `<section class="card center"><h2>Tudo revisado por hoje!</h2><p class="muted">Volte amanhã: revisar no dia certo é o que fixa na memória.</p></section>`}
  <section class="stack"><p class="eyebrow">Por categoria</p><div class="chips">${cats.map(k => { const all = CARDS.filter(c => c.cat === k), seen = all.filter(c => S.cards[c.id]).length; return `<span class="chip">${esc(k)} <b>${seen}/${all.length}</b></span>`; }).join("")}</div></section>`;
  if (!c) return;
  $("#cShow").onclick = () => {
    $("#cb").hidden = false;
    if (c.p) { const pb = $("#cPlay"); pb.onclick = () => c.p.startsWith("fx:") ? playFx(c.p) : playSpec(c.p); }
    $("#cAct").innerHTML = `<button class="btn" data-r="0">Errei</button><button class="btn" data-r="1">Difícil</button><button class="btn pri" data-r="2">Fácil</button>`;
    $("#cAct").querySelectorAll("[data-r]").forEach(b => b.onclick = () => rate(c, +b.dataset.r));
  };
}
function rate(c, r) {
  const cur = S.cards[c.id] || { iv: 0, added: today() };
  let step = r === 0 ? 0 : r === 1 ? Math.max(0, cur.iv) : cur.iv + 1; step = Math.min(step, INTERVALS.length - 1);
  S.cards[c.id] = Object.assign(cur, { iv: step, due: addDays(today(), r === 0 ? 0 : INTERVALS[step]), last: today() });
  save(); gain(r === 0 ? 1 : 3, "card", true); hdr();
  CQ.shift(); if (r === 0) CQ.push(c);
  viewCards();
}

// ---------- laboratório ----------
const LABS = [
  ["teclado", "Teclado, escalas e acordes", "Monte qualquer escala ou acorde com a grafia certa e ouça."],
  ["campo", "Campo harmônico", "Os 7 acordes de cada tom, com função e cifra. Toque para ouvir."],
  ["circulo", "Círculo das quintas", "Armaduras, relativas e tons vizinhos num só desenho."],
  ["metro", "Metrônomo", "Andamento, compasso, subdivisão e “tap tempo”."],
  ["ondas", "Ondas e harmônicos", "Monte um timbre somando harmônicos e veja a onda."],
  ["mesa", "Mesa de mixagem", "EQ, compressor e reverb num loop de banda, com analisador de espectro."]
];
function viewLab(sub) {
  MIX.stop(); METRO.stop(); AU.stop();
  if (!sub) {
    app().innerHTML = `<header class="phead"><p class="eyebrow">Laboratório</p><h1 class="display">Ferramentas de estudo</h1><p class="muted">Tudo com som gerado no próprio aparelho.</p></header>
    <div class="grid2">${LABS.map(([k, n, d]) => `<a class="card linkcard" href="#lab-${k}"><span class="lc-ic">${IC.lab}</span><div><b>${esc(n)}</b><p class="small muted">${esc(d)}</p></div></a>`).join("")}</div>`;
    return;
  }
  const meta = LABS.find(x => x[0] === sub); if (!meta) { location.replace("#lab"); return; }
  app().innerHTML = `<a class="back" href="#lab">${IC.back} Laboratório</a><header class="phead"><h1>${esc(meta[1])}</h1><p class="muted">${esc(meta[2])}</p></header><div id="labBody" class="stack"></div>`;
  ({ teclado: labTeclado, campo: labCampo, circulo: labCirculo, metro: labMetro, ondas: labOndas, mesa: labMesa })[sub]($("#labBody"));
}
const ROOTS = ["C", "C#", "Db", "D", "Eb", "E", "F", "F#", "Gb", "G", "Ab", "A", "Bb", "B"];
function rootSel(id, cur) { return `<select id="${id}" class="sel">${ROOTS.map(r => `<option value="${r}" ${r === cur ? "selected" : ""}>${T.pt(T.parse(r))} (${niceCif(r)})</option>`).join("")}</select>`; }
function labTeclado(el) {
  el.innerHTML = `<section class="card stack"><div class="row">${rootSel("lr", "C")}<select id="lt" class="sel"><optgroup label="Escalas">${Object.entries(SCALES).map(([k, v]) => `<option value="s:${k}">${v[0]}</option>`).join("")}</optgroup><optgroup label="Acordes">${Object.entries(CHORDS).map(([k, v]) => `<option value="c:${k}">${v[0]} (${v[1] || "M"})</option>`).join("")}</optgroup><optgroup label="Intervalos">${IV.slice(1).map(i => `<option value="i:${i[0]}">${i[1]}</option>`).join("")}</optgroup></select></div>
  <div id="lout"></div>${kbHTML(48, 84, "lkb")}<div class="row">${btnPlay('id="lplay"', "Tocar")}<button class="btn" id="lblock">Tocar junto</button></div><p class="small muted">Toque nas teclas para ouvir qualquer nota.</p></section>`;
  const kb = $("#lkb"); kbBind(kb);
  let cur = [];
  const upd = () => {
    const r = T.parse($("#lr").value + "4"), [kind, k] = $("#lt").value.split(":");
    let ivs, name; if (kind === "s") { ivs = SCALES[k][1]; name = SCALES[k][0]; } else if (kind === "c") { ivs = CHORDS[k][2]; name = CHORDS[k][0]; } else { const iv = IVK[k]; ivs = [[0, 0], [iv[2], iv[3]]]; name = iv[1]; }
    const ns = T.build(r, ivs); if (kind === "s") ns.push({ l: r.l, a: r.a, o: r.o + 1 });
    if (T.midi(r) > 66) ns.forEach(n => n.o--);
    cur = ns;
    const weird = ns.some(n => Math.abs(n.a) > 1);
    $("#lout").innerHTML = `<p class="eyebrow">${esc(name)} de ${T.pt(r)}${kind === "c" ? ` · cifra ${niceCif(T.cif(r))}${CHORDS[k][1]}` : ""}</p>
      <div class="notes">${ns.map((n, i) => `<span class="nchip"><b>${T.pt(n)}</b><small>${kind === "s" ? (i === ns.length - 1 ? "8" : degLabel(ivs[i])) : kind === "c" ? degLabel(ivs[i]) : i ? IVK[k][0] : "1"}</small></span>`).join("")}</div>
      ${staffSVG(ns.map(n => T.cif(n) + n.o).join(" "), "sol")}
      ${weird ? `<p class="small muted">Apareceu dobrado sustenido/bemol? É a grafia teórica correta. Na prática, muita gente escolhe a tonalidade enarmônica (ex.: Ré♭ em vez de Dó♯).</p>` : ""}`;
    kbMark(kb, ns.map(T.midi), "on", ns.map(T.pt));
  };
  $("#lr").onchange = upd; $("#lt").onchange = upd; upd();
  $("#lplay").onclick = () => { AU.stop(); const t = AU.now(); cur.forEach((n, i) => AU.note(T.midi(n), t + i * .32, .4)); };
  $("#lblock").onclick = () => { AU.stop(); AU.chord(cur.map(T.midi), AU.now(), 1.6); };
}
function degLabel([s, h]) { const nat = [0, 2, 4, 5, 7, 9, 11][s % 7] + (s >= 7 ? 12 : 0), d = h - nat; return (d > 0 ? "♯".repeat(d) : d < 0 ? "♭".repeat(-d) : "") + (s + 1); }
const FUNC = { 1: "Tônica (T)", 2: "Subdominante (SD)", 3: "Tônica (T)", 4: "Subdominante (SD)", 5: "Dominante (D)", 6: "Tônica relativa (T)", 7: "Dominante (D)" };
function labCampo(el) {
  el.innerHTML = `<section class="card stack"><div class="row">${rootSel("cr", "G")}<select id="cm" class="sel"><option value="maior">Tom maior</option><option value="menor">Menor natural</option><option value="harmonica">Menor harmônica</option></select><label class="tog"><input type="checkbox" id="ct"> Tétrades (com 7ª)</label></div><div id="cout" class="campo"></div><p class="small muted">Toque num acorde para ouvir. As funções (T, SD, D) explicam por que alguns acordes “pedem” outros.</p></section>`;
  const upd = () => {
    const r = T.parse($("#cr").value + "3"), cf = campo(r, $("#cm").value, $("#ct").checked);
    $("#cout").innerHTML = cf.map((c, i) => `<button class="cchord" data-i="${i}"><span class="rn">${c.rn}</span><b>${c.nome}</b><small>${c.notes.map(T.pt).join(" ")}</small><em>${$("#cm").value === "maior" ? FUNC[c.grau] : ""}</em></button>`).join("");
    $("#cout").querySelectorAll(".cchord").forEach(b => b.onclick = () => { const c = cf[+b.dataset.i], ms = c.notes.map(T.midi); AU.stop(); const t = AU.now(); AU.note(ms[0] - 12, t, 1.6, { v: .7 }); AU.chord(ms, t, 1.6); });
  };
  ["cr", "cm", "ct"].forEach(id => $("#" + id).onchange = upd); upd();
}
function labCirculo(el) {
  const R = 130, cx = 160, cy = 160;
  let svg = `<svg viewBox="0 0 320 320" class="circle5" role="img" aria-label="Círculo das quintas">`;
  CIRCLE.forEach(([M, m, n], i) => {
    const a = (i * 30 - 90) * Math.PI / 180, x = cx + R * Math.cos(a), y = cy + R * Math.sin(a), x2 = cx + 80 * Math.cos(a), y2 = cy + 80 * Math.sin(a);
    svg += `<g class="c5" data-i="${i}" tabindex="0"><circle cx="${x}" cy="${y}" r="24"/><text x="${x}" y="${y + 6}" text-anchor="middle" class="c5M">${niceCif(M)}</text><circle cx="${x2}" cy="${y2}" r="18" class="mi"/><text x="${x2}" y="${y2 + 5}" text-anchor="middle" class="c5m">${niceCif(m)}</text></g>`;
  });
  svg += `<text x="160" y="156" text-anchor="middle" class="c5c">sentido horário</text><text x="160" y="174" text-anchor="middle" class="c5c">= sobe uma 5ª</text></svg>`;
  el.innerHTML = `<section class="card stack"><div class="c5wrap">${svg}</div><div id="c5out"></div></section>`;
  const show = i => {
    const [M, m, n] = CIRCLE[i], acc = n > 0 ? SHARPS_ORDER.slice(0, n).map(x => T.pt(T.parse(x)) + "♯") : FLATS_ORDER.slice(0, -n).map(x => T.pt(T.parse(x)) + "♭");
    const r = T.parse(M + "3"), cf = campo(r, "maior"), nb = [CIRCLE[(i + 11) % 12][0], CIRCLE[(i + 1) % 12][0]];
    el.querySelectorAll(".c5").forEach(g => g.classList.toggle("sel", +g.dataset.i === i));
    $("#c5out").innerHTML = `<h2>${T.pt(r)} maior <span class="muted">· relativa: ${ptCif(m)}</span></h2>
      <p><b>Armadura:</b> ${n === 0 ? "nenhum acidente" : Math.abs(n) + (n > 0 ? " sustenido(s): " : " bemol(is): ") + acc.join(", ")}</p>
      <p><b>Tons vizinhos</b> (mudam só 1 acidente): ${nb.map(x => T.pt(T.parse(x)) + " maior").join(" e ")}</p>
      <p><b>Acordes principais:</b> I = ${cf[0].nome}, IV = ${cf[3].nome}, V = ${cf[4].nome}, vi = ${cf[5].nome}</p>
      <div class="row">${btnPlay('id="c5p"', "Ouvir I – IV – V – I")}</div>`;
    $("#c5p").onclick = () => { AU.stop(); const t = AU.now(); [0, 3, 4, 0].forEach((d, j) => { const ms = cf[d].notes.map(T.midi); AU.chord(ms, t + j * .8, .78); AU.note(ms[0] - 12, t + j * .8, .78); }); };
  };
  el.querySelectorAll(".c5").forEach(g => { g.onclick = () => { show(+g.dataset.i); const r = T.midi(T.parse(CIRCLE[+g.dataset.i][0] + "3")); AU.stop(); AU.chord([r, r + 4, r + 7], AU.now(), 1); }; g.onkeydown = e => { if (e.key === "Enter") g.onclick(); }; });
  show(0);
}
function labMetro(el) {
  el.innerHTML = `<section class="card stack center"><div class="bpm"><button class="btn" id="mm">−</button><div><span id="mb" class="bign">${METRO.bpm}</span><p class="small muted" id="mt"></p></div><button class="btn" id="mp">+</button></div>
  <input type="range" id="mr" min="30" max="240" value="${METRO.bpm}" class="range">
  <div class="beats" id="beats"></div>
  <div class="row" style="justify-content:center"><select id="mc" class="sel"><option value="2">2/4</option><option value="3">3/4</option><option value="4" selected>4/4</option><option value="6">6/8 (6 colcheias)</option></select><select id="ms" class="sel"><option value="1">Só o tempo</option><option value="2">Colcheias</option><option value="3">Tercinas</option><option value="4">Semicolcheias</option></select></div>
  <div class="row" style="justify-content:center"><button class="btn pri big" id="mgo">${IC.play} Iniciar</button><button class="btn big" id="mtap">Tap tempo</button></div>
  <p class="small muted">Dica de estudo: toque uma passagem difícil no andamento em que ela sai perfeita, e suba 4 BPM por vez.</p></section>`;
  const TEMPI = [[40, "Grave"], [60, "Largo"], [66, "Adagio"], [76, "Andante"], [108, "Moderato"], [120, "Allegro"], [168, "Presto"], [200, "Prestissimo"]];
  const upd = () => { $("#mb").textContent = METRO.bpm; $("#mr").value = METRO.bpm; let n = TEMPI[0][1]; TEMPI.forEach(([b, x]) => { if (METRO.bpm >= b) n = x; }); $("#mt").textContent = n + " · " + (60000 / METRO.bpm).toFixed(0) + " ms por tempo"; };
  const dots = () => { $("#beats").innerHTML = Array.from({ length: METRO.beats }, (_, i) => `<span class="bd ${i === 0 ? "first" : ""}"></span>`).join(""); };
  METRO.onBeat = b => { const d = $("#beats"); if (!d) return; d.querySelectorAll(".bd").forEach((x, i) => x.classList.toggle("on", i === b)); };
  $("#mm").onclick = () => { METRO.bpm = Math.max(30, METRO.bpm - 1); upd(); };
  $("#mp").onclick = () => { METRO.bpm = Math.min(240, METRO.bpm + 1); upd(); };
  $("#mr").oninput = e => { METRO.bpm = +e.target.value; upd(); };
  $("#mc").onchange = e => { METRO.beats = +e.target.value; dots(); };
  $("#ms").onchange = e => { METRO.sub = +e.target.value; };
  $("#mgo").onclick = () => { if (METRO.on) { METRO.stop(); $("#mgo").innerHTML = IC.play + " Iniciar"; } else { METRO.start(); $("#mgo").innerHTML = IC.stop + " Parar"; } };
  let taps = []; $("#mtap").onclick = () => { const n = performance.now(); taps = taps.filter(t => n - t < 2500); taps.push(n); if (taps.length > 1) { const d = (taps[taps.length - 1] - taps[0]) / (taps.length - 1); METRO.bpm = Math.round(Math.min(240, Math.max(30, 60000 / d))); upd(); } };
  upd(); dots();
}
function labOndas(el) {
  let H = [1, 0, 0, 0, 0, 0, 0, 0];
  const pre = { "Senoide": [1, 0, 0, 0, 0, 0, 0, 0], "Quadrada": [1, 0, 1 / 3, 0, 1 / 5, 0, 1 / 7, 0], "Dente de serra": [1, 1 / 2, 1 / 3, 1 / 4, 1 / 5, 1 / 6, 1 / 7, 1 / 8], "Triangular": [1, 0, 1 / 9, 0, 1 / 25, 0, 1 / 49, 0], "Órgão": [1, .8, 0, .5, 0, .3, 0, .2], "Clarinete": [1, 0, .75, 0, .5, 0, .14, 0] };
  el.innerHTML = `<section class="card stack"><canvas id="wc" width="640" height="200" class="wcanvas"></canvas>
  <div class="chips">${Object.keys(pre).map(k => `<button class="chip btnchip" data-p="${k}">${k}</button>`).join("")}</div>
  <div class="harms">${H.map((v, i) => `<label><span>${i + 1}º</span><input type="range" min="0" max="100" value="${v * 100}" data-h="${i}" class="vr"><small>${(T.hz(45) * (i + 1)).toFixed(0)} Hz</small></label>`).join("")}</div>
  <div class="row">${btnPlay('id="wp"', "Ouvir (Lá2 = 110 Hz)")}<button class="btn" id="ws">Parar</button></div>
  <p class="small muted">Cada controle é um harmônico: múltiplos inteiros da nota fundamental (1×, 2×, 3×…). A “receita” de harmônicos é o que o ouvido reconhece como timbre. Clarinete tem quase só ímpares; por isso soa “oco”.</p></section>`;
  let osc = null, g = null;
  const draw = () => {
    const c = $("#wc"), x = c.getContext("2d"), W = c.width, Hh = c.height; x.clearRect(0, 0, W, Hh);
    const css = getComputedStyle(document.documentElement); x.strokeStyle = css.getPropertyValue("--hair"); x.beginPath(); x.moveTo(0, Hh / 2); x.lineTo(W, Hh / 2); x.stroke();
    const pts = []; let mx = .001; for (let i = 0; i <= W; i++) { const ph = i / W * 2 * 2 * Math.PI; let v = 0; H.forEach((a, k) => v += a * Math.sin((k + 1) * ph)); pts.push(v); mx = Math.max(mx, Math.abs(v)); }
    x.strokeStyle = css.getPropertyValue("--accent"); x.lineWidth = 3; x.beginPath(); pts.forEach((v, i) => { const y = Hh / 2 - v / mx * (Hh * .42); i ? x.lineTo(i, y) : x.moveTo(i, y); }); x.stroke();
  };
  const live = () => { if (!osc) return; const re = new Float32Array(9), im = new Float32Array([0, ...H]); osc.setPeriodicWave(AU.ctx.createPeriodicWave(re, im)); };
  el.querySelectorAll(".vr").forEach(r => r.oninput = () => { H[+r.dataset.h] = r.value / 100; draw(); live(); });
  el.querySelectorAll("[data-p]").forEach(b => b.onclick = () => { H = pre[b.dataset.p].slice(); el.querySelectorAll(".vr").forEach((r, i) => r.value = H[i] * 100); draw(); live(); });
  const stop = () => { if (osc) { g.gain.setTargetAtTime(0, AU.ctx.currentTime, .03); const o = osc; setTimeout(() => o.stop(), 200); osc = null; } };
  $("#wp").onclick = () => { stop(); const c = AU.init(); osc = c.createOscillator(); g = c.createGain(); g.gain.value = 0; g.gain.setTargetAtTime(.22, c.currentTime, .03); osc.frequency.value = 110; live(); osc.connect(g); g.connect(AU.master); osc.start(); };
  $("#ws").onclick = stop;
  window._labStop = stop;
  draw();
}
function labMesa(el) {
  el.innerHTML = `<section class="card stack"><canvas id="sc" width="640" height="180" class="wcanvas"></canvas>
  <div class="row"><button class="btn pri big" id="xgo">${IC.play} Tocar loop</button><span class="small muted">Banda em Sol: I–V–vi–IV, 96 BPM</span></div>
  <div class="mixer">
    ${[["Volume bateria", "vd", 0, 150, 100, "%"], ["Volume baixo", "vb", 0, 150, 100, "%"], ["Volume teclado", "vk", 0, 150, 100, "%"],
       ["Filtro passa-altas", "hp", 20, 400, 20, " Hz"], ["Graves (shelf 150 Hz)", "lo", -15, 15, 0, " dB"], ["Médios (1 kHz)", "mi", -15, 15, 0, " dB"], ["Agudos (shelf 6 kHz)", "hi", -15, 15, 0, " dB"],
       ["Compressor: limiar", "th", -50, 0, 0, " dB"], ["Compressor: razão", "ra", 1, 20, 1, ":1"], ["Ganho de compensação", "mk", 0, 18, 0, " dB"], ["Reverb (mandada)", "rv", 0, 100, 0, "%"]]
      .map(([n, id, a, b, v, u]) => `<label class="fader"><span>${n}</span><input type="range" min="${a}" max="${b}" value="${v}" id="x${id}" data-u="${u}"><output id="o${id}">${v}${u}</output></label>`).join("")}
  </div>
  <p class="small muted" id="xred">Redução do compressor: 0 dB</p>
  <div class="tips"><p class="eyebrow">Experimentos</p>
  <p class="small">1. Suba os graves +12 dB: o bumbo e o baixo “embolam”. Agora corte 3 dB em vez de aumentar o resto: isso é EQ subtrativa.</p>
  <p class="small">2. Passa-altas em 200 Hz: some o peso. Na igreja, use passa-altas nas vozes e violões (80–120 Hz) para limpar o grave.</p>
  <p class="small">3. Limiar em −30 dB e razão 8:1: o som fica “achatado”. Compense com o ganho e compare o volume de verdade.</p>
  <p class="small">4. Reverb em 60%: tudo fica distante. Pouco reverb aproxima; muito afasta.</p></div></section>`;
  const N = () => MIX.nodes;
  const db = v => Math.pow(10, v / 20);
  const apply = () => {
    if (!N()) return; const n = N(), g = id => +$("#x" + id).value;
    n.drums.gain.value = g("vd") / 100; n.bass.gain.value = g("vb") / 100 * 1.4; n.keys.gain.value = g("vk") / 100;
    n.hp.frequency.value = g("hp"); n.low.gain.value = g("lo"); n.mid.gain.value = g("mi"); n.high.gain.value = g("hi");
    n.comp.threshold.value = g("th"); n.comp.ratio.value = g("ra"); n.makeup.gain.value = db(g("mk")); n.wet.gain.value = g("rv") / 100 * .8;
  };
  el.querySelectorAll(".fader input").forEach(r => r.oninput = () => { $("#o" + r.id.slice(1)).textContent = r.value + r.dataset.u; apply(); });
  let raf = 0;
  const draw = () => {
    const c = $("#sc"); if (!c || !MIX.on) { cancelAnimationFrame(raf); return; }
    const x = c.getContext("2d"), W = c.width, H = c.height, a = MIX.nodes.ana, d = new Uint8Array(a.frequencyBinCount); a.getByteFrequencyData(d);
    const css = getComputedStyle(document.documentElement); x.clearRect(0, 0, W, H);
    x.fillStyle = css.getPropertyValue("--muted"); x.font = "20px system-ui";
    [100, 1000, 10000].forEach(f => { const px = Math.log10(f / 20) / Math.log10(1000) * W; x.fillText(f >= 1000 ? f / 1000 + "k" : f, px + 3, H - 6); });
    const ny = AU.ctx.sampleRate / 2; x.fillStyle = css.getPropertyValue("--accent"); x.beginPath(); x.moveTo(0, H);
    for (let px = 0; px < W; px += 2) { const f = 20 * Math.pow(1000, px / W), bi = Math.min(d.length - 1, Math.round(f / ny * d.length)); x.lineTo(px, H - d[bi] / 255 * (H - 20)); }
    x.lineTo(W, H); x.fill();
    const red = MIX.nodes.comp.reduction; $("#xred").textContent = "Redução do compressor: " + (typeof red === "number" ? red.toFixed(1) : (red && red.value || 0).toFixed(1)) + " dB";
    raf = requestAnimationFrame(draw);
  };
  $("#xgo").onclick = () => { if (MIX.on) { MIX.stop(); $("#xgo").innerHTML = IC.play + " Tocar loop"; } else { MIX.start(); apply(); $("#xgo").innerHTML = IC.stop + " Parar"; draw(); } };
}

// ---------- perfil e guia ----------
function viewPerfil() {
  const Lv = level(), les = ALL().filter(passed).length, ear = Object.values(S.ear).reduce((a, s) => a + s.n, 0);
  app().innerHTML = `<header class="phead"><p class="eyebrow">Perfil</p><h1 class="display">${esc(Lv.name)}</h1><p class="muted">${S.xp} XP${Lv.next ? ` · próximo título: ${esc(Lv.nextName)} em ${Lv.next - S.xp} XP` : ""}</p></header>
  <div class="stats"><div class="card"><span class="bign">${streak()}</span><p class="small muted">dias seguidos</p></div><div class="card"><span class="bign">${les}/${ALL().length}</span><p class="small muted">lições aprovadas</p></div><div class="card"><span class="bign">${ear}</span><p class="small muted">exercícios de ouvido</p></div><div class="card"><span class="bign">${Object.keys(S.cards).length}</span><p class="small muted">cards estudados</p></div></div>
  <section class="card stack"><p class="eyebrow">Mestres</p>${TRACKS.map(t => { const M = MENTORS[t.mk], st = mStage(t); return `<div class="row" style="flex-wrap:nowrap">${ava(t.mk, 44, st)}<div class="grow"><b>${esc(M.name)}</b> <span class="small muted">· ${esc(M.ref)}</span><p class="small muted">${esc(t.title)} · ${RANKS[st]} (${mPts(t)}/16 pontos)</p></div></div>`; }).join("")}</section>
  <section class="card stack"><p class="eyebrow">Aparência</p><div class="seg" id="thm">${[["auto", "Automático"], ["light", "Claro"], ["dark", "Escuro"]].map(([v, n]) => `<button data-v="${v}" aria-pressed="${S.theme === v}">${n}</button>`).join("")}</div></section>
  <section class="card stack"><p class="eyebrow">Ajustes</p>
    <label class="tog"><input type="checkbox" id="ua" ${S.unlockAll ? "checked" : ""}> Liberar todas as lições (para quem já tem base)</label>
    <div class="row"><button class="btn" id="exp">Copiar backup</button><button class="btn" id="imp">Restaurar backup</button><button class="btn danger" id="rst">Apagar progresso</button></div>
    <p class="small muted">O progresso fica salvo só neste aparelho. Use o backup para passar para outro.</p></section>`;
  $("#thm").querySelectorAll("button").forEach(b => b.onclick = () => { S.theme = b.dataset.v; save(); applyTheme(); viewPerfil(); });
  $("#ua").onchange = e => { S.unlockAll = e.target.checked; save(); };
  $("#exp").onclick = async () => { const t = btoa(unescape(encodeURIComponent(JSON.stringify(S)))); try { await navigator.clipboard.writeText(t); toast("Backup copiado"); } catch (e) { prompt("Copie o texto:", t); } };
  $("#imp").onclick = () => { const t = prompt("Cole o backup:"); if (!t) return; try { const o = JSON.parse(decodeURIComponent(escape(atob(t.trim())))); if (o.v !== 1) throw 0; S = Object.assign(fresh(), o); save(); toast("Progresso restaurado"); route(); } catch (e) { toast("Backup inválido"); } };
  $("#rst").onclick = () => { if (confirm("Apagar todo o progresso deste aparelho?")) { S = fresh(); save(); route(); } };
}
function viewGuia() {
  app().innerHTML = `<a class="back" href="#inicio">${IC.back} Início</a>${GUIA()}`;
  app().querySelectorAll("[data-gplay]").forEach(b => b.onclick = () => playFx(b.dataset.gplay));
}
function applyTheme() { const r = document.documentElement; if (S.theme === "auto") r.removeAttribute("data-theme"); else r.setAttribute("data-theme", S.theme); }

// ---------- modal ----------
function modal(html) {
  const m = document.createElement("div"); m.className = "modal"; m.innerHTML = `<div class="card mbox">${html}<button class="btn pri" style="margin-top:16px;width:100%">Continuar</button></div>`;
  document.body.appendChild(m); m.querySelector("button").onclick = () => m.remove(); m.onclick = e => { if (e.target === m) m.remove(); };
}

// ---------- rotas ----------
function route() {
  const h = location.hash.slice(1) || "inicio";
  if (window._labStop) { window._labStop(); window._labStop = null; }
  MIX.stop(); METRO.stop();
  if (!h.startsWith("s-")) closeStory();
  const [a, ...r] = h.split("-"), rest = r.join("-");
  window.scrollTo(0, 0);
  if (a === "s") { openStory(rest); return; }
  if (a === "p") { openQuiz(rest); return; }
  if (a === "l") { tabs("trilha"); viewLesson(rest); return; }
  if (a === "o") { tabs("ouvido"); openEar(rest); return; }
  if (a === "lab") { tabs("lab"); viewLab(rest); return; }
  const V = { inicio: viewHome, trilha: viewTrilha, ouvido: viewOuvido, cards: () => { CQ = null; viewCards(); }, perfil: viewPerfil, guia: viewGuia };
  tabs(V[a] ? (a === "perfil" || a === "guia" ? "" : a) : "inicio");
  (V[a] || viewHome)();
}
window.addEventListener("hashchange", route);
document.addEventListener("pointerdown", () => AU.init(), { once: true });
applyTheme(); hdr(); route();
if ("serviceWorker" in navigator && location.protocol === "https:" && !/claude|anthropic/.test(location.host)) navigator.serviceWorker.register("sw.js").catch(() => {});
