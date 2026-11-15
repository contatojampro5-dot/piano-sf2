// Teoria musical com grafia certa (Dó♯ ≠ Ré♭). Nota = {l: letra 0..6 (C..B), a: acidente, o: oitava}.
"use strict";
const LET = "CDEFGAB", LET_PT = ["Dó","Ré","Mi","Fá","Sol","Lá","Si"], BASE = [0,2,4,5,7,9,11];
const T = {
  N(l, a = 0, o = 4) { return { l, a, o }; },
  parse(s) { // "C#4", "Eb3", "F##", "Bb"
    const m = /^([A-Ga-g])(##|bb|#|b|x)?(-?\d)?$/.exec(String(s).trim());
    if (!m) return null;
    const a = { "#": 1, "##": 2, x: 2, b: -1, bb: -2 }[m[2]] || 0;
    return { l: LET.indexOf(m[1].toUpperCase()), a, o: m[3] != null ? +m[3] : 4 };
  },
  pc(n) { return ((BASE[n.l] + n.a) % 12 + 12) % 12; },
  midi(n) { return 12 * (n.o + 1) + BASE[n.l] + n.a; },
  acc(a) { return a > 0 ? "♯".repeat(a).replace("♯♯", "𝄪") : a < 0 ? "♭".repeat(-a) : ""; },
  cif(n) { return LET[n.l] + (n.a > 0 ? "#".repeat(n.a) : "b".repeat(-n.a)); },
  pt(n) { return LET_PT[n.l] + T.acc(n.a); },
  both(n) { return T.pt(n) + " (" + T.cif(n) + ")"; },
  // sobe "steps" letras e "semis" semitons, mantendo a grafia correta
  up(n, steps, semis) {
    const L = n.l + steps, nl = ((L % 7) + 7) % 7, no = n.o + Math.floor(L / 7);
    const target = T.midi(n) + semis, natural = 12 * (no + 1) + BASE[nl];
    return { l: nl, a: target - natural, o: no };
  },
  build(root, ivs) { return ivs.map(([s, h]) => T.up(root, s, h)); },
  // nome PT a partir do número MIDI (grafia simples com ♯, ou ♭ se pedir)
  midiName(m, flat) {
    const pc = ((m % 12) + 12) % 12, o = Math.floor(m / 12) - 1;
    const S = ["Dó","Dó♯","Ré","Ré♯","Mi","Fá","Fá♯","Sol","Sol♯","Lá","Lá♯","Si"];
    const F = ["Dó","Ré♭","Ré","Mi♭","Mi","Fá","Sol♭","Sol","Lá♭","Lá","Si♭","Si"];
    return (flat ? F : S)[pc] + o;
  },
  hz(m) { return 440 * Math.pow(2, (m - 69) / 12); }
};

// Intervalos: [nome curto, nome longo, passos de letra, semitons]
const IV = [
  ["U", "Uníssono", 0, 0], ["2m", "Segunda menor", 1, 1], ["2M", "Segunda maior", 1, 2],
  ["3m", "Terça menor", 2, 3], ["3M", "Terça maior", 2, 4], ["4J", "Quarta justa", 3, 5],
  ["4A", "Quarta aumentada (trítono)", 3, 6], ["5J", "Quinta justa", 4, 7], ["6m", "Sexta menor", 5, 8],
  ["6M", "Sexta maior", 5, 9], ["7m", "Sétima menor", 6, 10], ["7M", "Sétima maior", 6, 11], ["8J", "Oitava justa", 7, 12]
];
const IVK = Object.fromEntries(IV.map(i => [i[0], i]));

const SCALES = {
  maior:      ["Maior (jônio)",          [[0,0],[1,2],[2,4],[3,5],[4,7],[5,9],[6,11]]],
  menor:      ["Menor natural (eólio)",  [[0,0],[1,2],[2,3],[3,5],[4,7],[5,8],[6,10]]],
  harmonica:  ["Menor harmônica",        [[0,0],[1,2],[2,3],[3,5],[4,7],[5,8],[6,11]]],
  melodica:   ["Menor melódica",         [[0,0],[1,2],[2,3],[3,5],[4,7],[5,9],[6,11]]],
  dorico:     ["Dórico",                 [[0,0],[1,2],[2,3],[3,5],[4,7],[5,9],[6,10]]],
  frigio:     ["Frígio",                 [[0,0],[1,1],[2,3],[3,5],[4,7],[5,8],[6,10]]],
  lidio:      ["Lídio",                  [[0,0],[1,2],[2,4],[3,6],[4,7],[5,9],[6,11]]],
  mixolidio:  ["Mixolídio",              [[0,0],[1,2],[2,4],[3,5],[4,7],[5,9],[6,10]]],
  locrio:     ["Lócrio",                 [[0,0],[1,1],[2,3],[3,5],[4,6],[5,8],[6,10]]],
  pentaM:     ["Pentatônica maior",      [[0,0],[1,2],[2,4],[4,7],[5,9]]],
  pentam:     ["Pentatônica menor",      [[0,0],[2,3],[3,5],[4,7],[6,10]]],
  blues:      ["Blues",                  [[0,0],[2,3],[3,5],[3,6],[4,7],[6,10]]],
  cromatica:  ["Cromática",              [[0,0],[0,1],[1,2],[1,3],[2,4],[3,5],[3,6],[4,7],[4,8],[5,9],[5,10],[6,11]]]
};

const CHORDS = {
  M:     ["Maior", "",        [[0,0],[2,4],[4,7]]],
  m:     ["Menor", "m",       [[0,0],[2,3],[4,7]]],
  dim:   ["Diminuto", "°",    [[0,0],[2,3],[4,6]]],
  aug:   ["Aumentado", "+",   [[0,0],[2,4],[4,8]]],
  sus2:  ["Suspenso 2", "sus2", [[0,0],[1,2],[4,7]]],
  sus4:  ["Suspenso 4", "sus4", [[0,0],[3,5],[4,7]]],
  "7":   ["Dominante (7)", "7", [[0,0],[2,4],[4,7],[6,10]]],
  "7M":  ["Maior com 7ª maior", "7M", [[0,0],[2,4],[4,7],[6,11]]],
  m7:    ["Menor com 7ª", "m7", [[0,0],[2,3],[4,7],[6,10]]],
  m7b5:  ["Meio-diminuto", "m7(♭5)", [[0,0],[2,3],[4,6],[6,10]]],
  dim7:  ["Diminuto com 7ª", "°7", [[0,0],[2,3],[4,6],[6,9]]],
  add9:  ["Maior com 9ª", "add9", [[0,0],[2,4],[4,7],[8,14]]]
};

// qualidade de uma tríade/tétrade a partir dos semitons (para o campo harmônico)
function chordKind(semis) {
  const k = semis.join(",");
  return ({ "0,4,7": "M", "0,3,7": "m", "0,3,6": "dim", "0,4,8": "aug",
    "0,4,7,11": "7M", "0,4,7,10": "7", "0,3,7,10": "m7", "0,3,6,10": "m7b5", "0,3,6,9": "dim7",
    "0,3,7,11": "m7M", "0,4,8,11": "7M#5" })[k] || "?";
}
const ROMAN = ["I","II","III","IV","V","VI","VII"];
// Campo harmônico: escala (chave de SCALES, 7 notas) + tônica => graus com acordes
function campo(root, scaleKey, tetrades) {
  const sc = T.build(root, SCALES[scaleKey][1]);
  return sc.map((n, i) => {
    const idx = tetrades ? [0, 2, 4, 6] : [0, 2, 4];
    const notes = idx.map(j => { const k = i + j, nn = sc[k % 7]; return { l: nn.l, a: nn.a, o: nn.o + Math.floor(k / 7) }; });
    const semis = notes.map(x => T.midi(x) - T.midi(notes[0]));
    const kind = chordKind(semis), suf = kind === "m7M" ? "m(7M)" : kind === "7M#5" ? "7M(♯5)" : (CHORDS[kind] ? CHORDS[kind][1] : "?");
    let rn = ROMAN[i];
    if (kind === "m" || kind === "m7" || kind === "m7b5" || kind === "dim" || kind === "dim7" || kind === "m7M") rn = rn.toLowerCase();
    if (kind === "dim") rn += "°"; if (kind === "aug") rn += "+";
    if (tetrades) rn += kind === "7M" ? "7M" : kind === "7" ? "7" : kind === "m7" ? "7" : kind === "m7b5" ? "ø" : kind === "dim7" ? "°7" : "";
    return { grau: i + 1, rn, notes, kind, nome: T.cif(notes[0]).replace("b", "♭").replace("#", "♯") + suf };
  });
}

// Círculo das quintas (tonalidades maiores, relativas e nº de acidentes na armadura)
const CIRCLE = [
  ["C", "Am", 0], ["G", "Em", 1], ["D", "Bm", 2], ["A", "F#m", 3], ["E", "C#m", 4], ["B", "G#m", 5],
  ["F#", "D#m", 6], ["Db", "Bbm", -5], ["Ab", "Fm", -4], ["Eb", "Cm", -3], ["Bb", "Gm", -2], ["F", "Dm", -1]
];
const SHARPS_ORDER = ["F", "C", "G", "D", "A", "E", "B"]; // Fá Dó Sol Ré Lá Mi Si
const FLATS_ORDER = ["B", "E", "A", "D", "G", "C", "F"];  // Si Mi Lá Ré Sol Dó Fá
function niceCif(s) { return s.replace(/#/g, "♯").replace(/([A-G])b/g, "$1♭"); }
function ptCif(s) { const n = T.parse(s.replace(/m$/, "")); return n ? T.pt(n) + (s.endsWith("m") ? " menor" : " maior") : s; }
