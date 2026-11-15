// Motor de som (Web Audio): piano sintetizado, ondas puras, cliques, ruído rosa, EQ e um loop para a mesa de mixagem.
"use strict";
const AU = {
  ctx: null, master: null, bus: null, waves: {}, pink: null, timers: [],
  init() {
    if (!this.ctx) {
      const C = window.AudioContext || window.webkitAudioContext;
      this.ctx = new C();
      this.master = this.ctx.createGain(); this.master.gain.value = 0.9;
      const lim = this.ctx.createDynamicsCompressor();
      lim.threshold.value = -6; lim.ratio.value = 12; lim.attack.value = 0.003; lim.release.value = 0.2;
      this.master.connect(lim); lim.connect(this.ctx.destination);
      this.newBus();
    }
    if (this.ctx.state === "suspended") this.ctx.resume();
    return this.ctx;
  },
  newBus() { this.bus = this.ctx.createGain(); this.bus.connect(this.master); },
  stop() { // corta tudo que estiver tocando
    this.timers.forEach(clearTimeout); this.timers = [];
    if (!this.ctx) return;
    const old = this.bus, t = this.ctx.currentTime;
    old.gain.setTargetAtTime(0, t, 0.02); setTimeout(() => old.disconnect(), 200);
    this.newBus();
  },
  now() { return this.init().currentTime + 0.05; },
  later(fn, sec) { this.timers.push(setTimeout(fn, Math.max(0, sec * 1000))); },
  wave(name) {
    if (this.waves[name]) return this.waves[name];
    const H = { piano: [0, 1, .55, .30, .22, .12, .09, .05, .035, .02, .012], organ: [0, 1, .8, .0, .45, 0, .25, 0, .2], voz: [0, 1, .4, .25, .1, .05] }[name];
    const re = new Float32Array(H.length), im = new Float32Array(H);
    return (this.waves[name] = this.ctx.createPeriodicWave(re, im));
  },
  // uma nota. timbre: piano | organ | sine | triangle | square | sawtooth
  tone(freq, t, dur, o = {}) {
    const c = this.init(), tim = o.w || "piano", vol = (o.v == null ? .8 : o.v) * 0.32, dest = o.dest || this.bus;
    const osc = c.createOscillator(), g = c.createGain();
    if (["sine", "triangle", "square", "sawtooth"].includes(tim)) osc.type = tim; else osc.setPeriodicWave(this.wave(tim));
    osc.frequency.value = freq;
    if (o.cents) osc.detune.value = o.cents;
    let out = osc;
    if (tim === "piano") { // filtro que fecha com o tempo: brilho no ataque, som mais escuro depois
      const f = c.createBiquadFilter(); f.type = "lowpass"; f.Q.value = 0.6;
      f.frequency.setValueAtTime(Math.min(16000, freq * 14), t);
      f.frequency.setTargetAtTime(Math.min(9000, freq * 4), t, 0.25);
      osc.connect(f); out = f;
    }
    out.connect(g); g.connect(dest);
    const sq = tim === "square" || tim === "sawtooth" ? .45 : 1;
    g.gain.setValueAtTime(0, t);
    if (tim === "piano") {
      const decay = Math.max(.35, 1.6 - (freq - 130) / 900);
      g.gain.linearRampToValueAtTime(vol, t + .006);
      g.gain.setTargetAtTime(vol * .0001, t + .01, decay);
      g.gain.setTargetAtTime(0, t + dur, .07);
    } else {
      g.gain.linearRampToValueAtTime(vol * sq, t + .02);
      g.gain.setValueAtTime(vol * sq, t + Math.max(.03, dur - .03));
      g.gain.linearRampToValueAtTime(0, t + dur);
    }
    osc.start(t); osc.stop(t + dur + .6);
    return osc;
  },
  note(m, t, dur, o) { return this.tone(T.hz(m), t, dur, o); },
  chord(ms, t, dur, o = {}) { ms.forEach((m, i) => this.note(m, t + (o.strum || 0) * i, dur, Object.assign({}, o, { v: (o.v || .8) * (ms.length > 3 ? .7 : .85) }))); },
  click(t, accent, dest) {
    const c = this.init(), osc = c.createOscillator(), g = c.createGain();
    osc.type = "sine"; osc.frequency.value = accent ? 1760 : 1180;
    g.gain.setValueAtTime(accent ? .5 : .3, t); g.gain.exponentialRampToValueAtTime(.001, t + .05);
    osc.connect(g); g.connect(dest || this.bus); osc.start(t); osc.stop(t + .06);
  },
  wood(t, accent) { // som de "bloco" para os ritmos
    const c = this.init(), osc = c.createOscillator(), g = c.createGain(), f = c.createBiquadFilter();
    osc.type = "triangle"; osc.frequency.setValueAtTime(accent ? 900 : 780, t); osc.frequency.exponentialRampToValueAtTime(420, t + .08);
    f.type = "bandpass"; f.frequency.value = 900; f.Q.value = 1.2;
    g.gain.setValueAtTime(accent ? .9 : .7, t); g.gain.exponentialRampToValueAtTime(.001, t + .12);
    osc.connect(f); f.connect(g); g.connect(this.bus); osc.start(t); osc.stop(t + .14);
  },
  pinkBuf() {
    if (this.pink) return this.pink;
    const c = this.init(), n = c.sampleRate * 3, b = c.createBuffer(1, n, c.sampleRate), d = b.getChannelData(0);
    let b0 = 0, b1 = 0, b2 = 0, b3 = 0, b4 = 0, b5 = 0, b6 = 0;
    for (let i = 0; i < n; i++) {
      const w = Math.random() * 2 - 1;
      b0 = .99886 * b0 + w * .0555179; b1 = .99332 * b1 + w * .0750759; b2 = .969 * b2 + w * .153852;
      b3 = .8665 * b3 + w * .3104856; b4 = .55 * b4 + w * .5329522; b5 = -.7616 * b5 - w * .016898;
      d[i] = (b0 + b1 + b2 + b3 + b4 + b5 + b6 + w * .5362) * .09; b6 = w * .115926;
    }
    return (this.pink = b);
  },
  // ruído rosa por "dur" segundos, opcionalmente com um filtro (EQ) aplicado
  noise(t, dur, eq, vol = .7) {
    const c = this.init(), s = c.createBufferSource(), g = c.createGain();
    s.buffer = this.pinkBuf(); s.loop = true;
    let node = s;
    if (eq) { const f = c.createBiquadFilter(); f.type = eq.type || "peaking"; f.frequency.value = eq.f; f.Q.value = eq.q || 1.4; f.gain.value = eq.g == null ? 12 : eq.g; s.connect(f); node = f; }
    node.connect(g); g.connect(this.bus);
    g.gain.setValueAtTime(0, t); g.gain.linearRampToValueAtTime(vol, t + .05);
    g.gain.setValueAtTime(vol, t + dur - .06); g.gain.linearRampToValueAtTime(0, t + dur);
    s.start(t, Math.random() * 2); s.stop(t + dur + .05);
  },
  // toca um padrão rítmico em grade (x = toque, X = acento, . = pausa). steps = divisões por tempo.
  rhythm(pat, bpm, o = {}) {
    const t0 = this.now() + .1, sp = 60 / bpm / (o.steps || 4), beat = 60 / bpm, per = o.beats || 4;
    let t = t0;
    if (o.countIn !== false) { for (let i = 0; i < per; i++) this.click(t0 + i * beat, i === 0); t = t0 + per * beat; }
    const clean = pat.replace(/[\s|]/g, "");
    for (let i = 0; i < clean.length; i++) {
      const ch = clean[i];
      if (o.metro) { if (i % (o.steps || 4) === 0) this.click(t + i * sp, i === 0, null); }
      if (ch === "x" || ch === "X") this.wood(t + i * sp, ch === "X");
    }
    return { start: t, step: sp, len: clean.length * sp, total: t - t0 + clean.length * sp + .3, t0 };
  }
};

// ---------- mini linguagem dos botões de som ----------
// "C4 D4 E4" melodia · "[C4 E4 G4]" acorde · "-" pausa · ":2" duração em tempos · "bpm=80" · "w=sine" timbre · "arp" acorde arpejado
function parseSpec(spec) {
  const toks = String(spec).match(/\[[^\]]*\](?::[\d.]+)?|\S+/g) || [];
  const o = { bpm: 92, w: "piano", ev: [], arp: false };
  let beat = 0;
  for (const tk of toks) {
    let m;
    if ((m = /^bpm=(\d+)/.exec(tk))) { o.bpm = +m[1]; continue; }
    if ((m = /^w=(\w+)/.exec(tk))) { o.w = m[1]; continue; }
    if (tk === "arp") { o.arp = true; continue; }
    const [body, d] = tk.split(/:(?=[\d.]+$)/);
    const dur = d ? +d : 1;
    if (body === "-") { beat += dur; continue; }
    const notes = body.replace(/[\[\]]/g, "").trim().split(/\s+/).map(T.parse).filter(Boolean).map(T.midi);
    if (notes.length) o.ev.push({ beat, dur, notes, chord: body.startsWith("[") });
    beat += dur;
  }
  o.beats = beat;
  return o;
}
function playSpec(spec, onNote, opt = {}) {
  AU.stop();
  const p = parseSpec(spec), t0 = AU.now(), b = 60 / p.bpm;
  for (const e of p.ev) {
    const t = t0 + e.beat * b, dur = e.dur * b * (opt.legato || 1.05);
    if (e.chord && (p.arp || opt.arp)) e.notes.forEach((m, i) => AU.note(m, t + i * b * .28, dur, { w: opt.w || p.w }));
    else if (e.chord) AU.chord(e.notes, t, dur, { w: opt.w || p.w });
    else AU.note(e.notes[0], t, dur, { w: opt.w || p.w });
    if (onNote) AU.later(() => onNote(e.notes), t - AU.ctx.currentTime);
  }
  const total = p.beats * b + .4;
  if (onNote) AU.later(() => onNote([]), total);
  return total;
}

// ---------- efeitos especiais usados nas lições ----------
const FX = {
  // batimento: duas frequências próximas
  beats(d = 3) { AU.stop(); const t = AU.now(); AU.tone(440, t, 4, { w: "sine", v: .6 }); AU.tone(440 + d, t, 4, { w: "sine", v: .6 }); return 4.2; },
  // série harmônica sobre Dó2
  harm() { AU.stop(); const t = AU.now(), f = T.hz(36); for (let i = 1; i <= 10; i++) AU.tone(f * i, t + (i - 1) * .55, .9, { w: "sine", v: .7 }); return 6; },
  // as 4 formas de onda em sequência
  waves() { AU.stop(); const t = AU.now(); ["sine", "triangle", "square", "sawtooth"].forEach((w, i) => AU.note(57, t + i * 1.1, 1, { w })); return 4.6; },
  wave(w) { AU.stop(); AU.note(57, AU.now(), 1.6, { w }); return 1.8; },
  // mesma nota, volumes diferentes (dB)
  dyn() { AU.stop(); const t = AU.now(); [.08, .2, .45, .9].forEach((v, i) => AU.note(60, t + i * .9, .8, { v })); return 4; },
  // ruído rosa com reforço numa faixa
  eq(f, g = 12) { AU.stop(); const t = AU.now(); AU.noise(t, 1.6, null); AU.noise(t + 1.8, 1.6, { f, g }); return 3.6; },
  noise() { AU.stop(); AU.noise(AU.now(), 2); return 2.2; },
  // nota em 440 e um pouco desafinada
  tune(c = 20) { AU.stop(); const t = AU.now(); AU.tone(440, t, 1, { w: "triangle" }); AU.tone(440, t + 1.2, 1, { w: "triangle", cents: c }); return 2.4; },
  // distorção digital (clipping)
  clip() {
    AU.stop(); const c = AU.init(), t = AU.now(), ws = c.createWaveShaper(), g = c.createGain(), N = 1024, cur = new Float32Array(N);
    for (let i = 0; i < N; i++) { const x = i / (N - 1) * 2 - 1; cur[i] = Math.max(-.25, Math.min(.25, x)) * 3; }
    ws.curve = cur; g.gain.value = .5; ws.connect(g); g.connect(AU.bus);
    AU.chord([48, 55, 60, 64], t, 1.4, { v: .9 }); AU.chord([48, 55, 60, 64], t + 1.7, 1.4, { v: .9, dest: ws }); return 3.4;
  }
};
function playFx(s) { const [, name, a, b] = s.split(":"); const f = FX[name]; return f ? f(a != null && !isNaN(+a) ? +a : a, b != null ? +b : undefined) : 0; }

// ---------- loop da mesa de mixagem (bateria + baixo + piano) ----------
const MIX = {
  on: false, nodes: null, timer: null, step: 0, next: 0, bpm: 96,
  build() {
    const c = AU.init(), n = {};
    n.in = c.createGain();
    n.hp = c.createBiquadFilter(); n.hp.type = "highpass"; n.hp.frequency.value = 20;
    n.low = c.createBiquadFilter(); n.low.type = "lowshelf"; n.low.frequency.value = 150;
    n.mid = c.createBiquadFilter(); n.mid.type = "peaking"; n.mid.frequency.value = 1000; n.mid.Q.value = 1;
    n.high = c.createBiquadFilter(); n.high.type = "highshelf"; n.high.frequency.value = 6000;
    n.comp = c.createDynamicsCompressor(); n.comp.threshold.value = 0; n.comp.ratio.value = 1;
    n.makeup = c.createGain();
    n.rev = c.createConvolver(); n.rev.buffer = this.impulse(2.2); n.wet = c.createGain(); n.wet.gain.value = 0;
    n.out = c.createGain(); n.ana = c.createAnalyser(); n.ana.fftSize = 2048; n.ana.smoothingTimeConstant = .8;
    n.in.connect(n.hp); n.hp.connect(n.low); n.low.connect(n.mid); n.mid.connect(n.high); n.high.connect(n.comp); n.comp.connect(n.makeup);
    n.makeup.connect(n.out); n.makeup.connect(n.rev); n.rev.connect(n.wet); n.wet.connect(n.out);
    n.out.connect(n.ana); n.ana.connect(AU.master);
    n.drums = c.createGain(); n.bass = c.createGain(); n.keys = c.createGain();
    n.pd = c.createStereoPanner ? c.createStereoPanner() : null;
    [n.drums, n.bass, n.keys].forEach(g => g.connect(n.in));
    this.nodes = n; return n;
  },
  impulse(sec) {
    const c = AU.ctx, len = c.sampleRate * sec, b = c.createBuffer(2, len, c.sampleRate);
    for (let ch = 0; ch < 2; ch++) { const d = b.getChannelData(ch); for (let i = 0; i < len; i++) d[i] = (Math.random() * 2 - 1) * Math.pow(1 - i / len, 3); }
    return b;
  },
  kick(t) { const c = AU.ctx, o = c.createOscillator(), g = c.createGain(); o.frequency.setValueAtTime(140, t); o.frequency.exponentialRampToValueAtTime(42, t + .14); g.gain.setValueAtTime(1, t); g.gain.exponentialRampToValueAtTime(.001, t + .38); o.connect(g); g.connect(this.nodes.drums); o.start(t); o.stop(t + .4); },
  snare(t) { const c = AU.ctx, s = c.createBufferSource(), f = c.createBiquadFilter(), g = c.createGain(); s.buffer = AU.pinkBuf(); f.type = "bandpass"; f.frequency.value = 1900; f.Q.value = .7; g.gain.setValueAtTime(1.3, t); g.gain.exponentialRampToValueAtTime(.001, t + .18); s.connect(f); f.connect(g); g.connect(this.nodes.drums); s.start(t, Math.random()); s.stop(t + .2);
    AU.tone(190, t, .08, { w: "triangle", v: .9, dest: this.nodes.drums }); },
  hat(t, v) { const c = AU.ctx, s = c.createBufferSource(), f = c.createBiquadFilter(), g = c.createGain(); s.buffer = AU.pinkBuf(); f.type = "highpass"; f.frequency.value = 7000; g.gain.setValueAtTime(v, t); g.gain.exponentialRampToValueAtTime(.001, t + .05); s.connect(f); f.connect(g); g.connect(this.nodes.drums); s.start(t, Math.random()); s.stop(t + .06); },
  // progressão I–V–vi–IV em Sol
  prog: [[43, [55, 59, 62, 67]], [38, [54, 57, 62, 66]], [40, [55, 59, 64, 67]], [36, [52, 55, 60, 64]]],
  tick(stepT, s) {
    const bar = Math.floor(s / 16) % 4, i = s % 16, [bn, ch] = this.prog[bar], sp = 60 / this.bpm / 4;
    if (i === 0 || i === 8 || i === 10) this.kick(stepT);
    if (i === 4 || i === 12) this.snare(stepT);
    if (i % 2 === 0) this.hat(stepT, i % 4 === 0 ? .5 : .3);
    if (i === 0 || i === 6 || i === 8 || i === 14) AU.tone(T.hz(bn), stepT, sp * (i === 6 || i === 14 ? 2 : 4), { w: "sawtooth", v: .7, dest: this.bassF });
    if (i === 0 || i === 8) ch.forEach(m => AU.note(m, stepT, sp * 7, { v: .5, dest: this.nodes.keys }));
  },
  start() {
    if (this.on) return; AU.init(); if (!this.nodes) { this.build(); this.bassF = AU.ctx.createBiquadFilter(); this.bassF.type = "lowpass"; this.bassF.frequency.value = 500; this.bassF.connect(this.nodes.bass); }
    this.on = true; this.step = 0; this.next = AU.ctx.currentTime + .1;
    const loop = () => {
      if (!this.on) return;
      const sp = 60 / this.bpm / 4;
      while (this.next < AU.ctx.currentTime + .15) { this.tick(this.next, this.step); this.step++; this.next += sp; }
      this.timer = setTimeout(loop, 30);
    };
    loop();
  },
  stop() { this.on = false; clearTimeout(this.timer); }
};

// ---------- metrônomo ----------
const METRO = {
  on: false, bpm: 80, beats: 4, sub: 1, next: 0, n: 0, timer: null, onBeat: null,
  start() {
    AU.init(); this.on = true; this.n = 0; this.next = AU.ctx.currentTime + .08;
    const loop = () => {
      if (!this.on) return;
      const sp = 60 / this.bpm / this.sub;
      while (this.next < AU.ctx.currentTime + .12) {
        const isBeat = this.n % this.sub === 0, beatN = Math.floor(this.n / this.sub) % this.beats;
        if (isBeat) AU.click(this.next, beatN === 0, AU.master);
        else { const c = AU.ctx, o = c.createOscillator(), g = c.createGain(); o.frequency.value = 900; g.gain.setValueAtTime(.12, this.next); g.gain.exponentialRampToValueAtTime(.001, this.next + .03); o.connect(g); g.connect(AU.master); o.start(this.next); o.stop(this.next + .04); }
        if (isBeat && this.onBeat) { const b = beatN, d = this.next - AU.ctx.currentTime; setTimeout(() => this.onBeat && this.onBeat(b), d * 1000); }
        this.n++; this.next += sp;
      }
      this.timer = setTimeout(loop, 25);
    };
    loop();
  },
  stop() { this.on = false; clearTimeout(this.timer); }
};
