// Arte dos mestres: motor de desenho (personS) trazido do app Firmes, com os mestres da música.
"use strict";
let MUID=0;
const eyeS=(x,y,r,c="#1B140F")=>`<circle cx="${x}" cy="${y}" r="${r}" fill="${c}"/><circle cx="${(x+r*.34).toFixed(1)}" cy="${(y-r*.36).toFixed(1)}" r="${(r*.4).toFixed(1)}" fill="#fff"/><circle cx="${(x-r*.32).toFixed(1)}" cy="${(y+r*.36).toFixed(1)}" r="${(r*.17).toFixed(1)}" fill="#fff" opacity=".85"/>`;
const shadowS=(rx=46,cy=188)=>`<ellipse cx="100" cy="${cy}" rx="${rx}" ry="6.5" fill="#000" opacity=".16"/>`;
const sparkS=(x,y,s,c)=>`<path d="M${x} ${y-s}Q${x} ${y} ${x+s} ${y}Q${x} ${y} ${x} ${y+s}Q${x} ${y} ${x-s} ${y}Q${x} ${y} ${x} ${y-s}Z" fill="${c}"/>`;
function auraS(u,c){
 let rays="";for(let i=0;i<12;i++){const a=i*Math.PI/6,b=.055,r=99,p=(t)=>`${(100+r*Math.cos(t)).toFixed(1)} ${(104+r*Math.sin(t)).toFixed(1)}`;rays+=`<path d="M100 104L${p(a-b)}L${p(a+b)}Z"/>`}
 return`<defs><radialGradient id="${u}a"><stop offset="0" stop-color="${c}" stop-opacity=".75"/><stop offset=".5" stop-color="${c}" stop-opacity=".25"/><stop offset="1" stop-color="${c}" stop-opacity="0"/></radialGradient></defs><g class="m-rays" fill="${c}" opacity=".16">${rays}</g><circle cx="100" cy="104" r="98" fill="url(#${u}a)"/>`;
}
const GOLD="#F2C14E";
function featherS(x,y,ang,l,w,fill,extra=""){return`<path d="M0 0C${(l*.3).toFixed(1)} ${-w} ${(l*.78).toFixed(1)} ${-w} ${l} 0C${(l*.78).toFixed(1)} ${w} ${(l*.3).toFixed(1)} ${w} 0 0Z" transform="translate(${x} ${y}) rotate(${ang})" fill="${fill}"${extra}/>`}
function legendSpark(){return sparkS(34,40,7,"#FFF4C7")+sparkS(166,52,5,"#FFF4C7")+sparkS(172,150,6,"#FFE08A")+sparkS(28,138,4.5,"#FFE08A")+sparkS(146,22,4,"#fff")}

/* Pessoas: figura base comum, com cabelo, barba, roupa e pose de cada mestre */
function personS(u,o){
 const K=o.skin,KS=o.skinSh,g=o.gold,trim=g?GOLD:o.trim;
 const sleeve=o.sleeve||o.tunic;
 const feet=`<ellipse cx="88" cy="185.5" rx="10.5" ry="4.6" fill="#5A3E2B"/><ellipse cx="112" cy="185.5" rx="10.5" ry="4.6" fill="#5A3E2B"/><ellipse cx="88" cy="183.2" rx="6.5" ry="2.8" fill="${K}"/><ellipse cx="112" cy="183.2" rx="6.5" ry="2.8" fill="${K}"/>`;
 const tunic=`<defs><linearGradient id="${u}t" x1="0" y1="0" x2="1" y2="0"><stop offset=".55" stop-color="${o.tunic}"/><stop offset="1" stop-color="${o.tunicSh}"/></linearGradient></defs>
 <path d="M80 102C71 126 65 156 62 182Q100 190 138 182C135 156 129 126 120 102Z" fill="url(#${u}t)"/>`;
 const belt=o.belt?`<path d="M69 142Q100 150 131 142L132.5 150Q100 158 67.5 150Z" fill="${o.belt}"/>`:"";
 const mantle=o.mantle?`<path d="M80 102C71 126 65 156 62 182L80 185C80 156 86 128 98 108Z" fill="${o.mantle}"/>
 <path d="M80 103C98 106 118 124 131 158L137 183L124 185C115 154 100 130 82 118Z" fill="${o.mantle}"/>
 <path d="M82 118C100 130 115 154 124 185" stroke="${trim}" stroke-width="${g?3.4:2.4}" fill="none" stroke-linecap="round" opacity="${g?1:.75}"/>
 <path d="M80 185C80 156 86 128 98 108" stroke="${o.mantleSh}" stroke-width="2.2" fill="none" opacity=".6"/>`:"";
 const arm=o.arms||"down";
 let arms="",hands="";
 if(arm==="down"){
  arms=`<path d="M83 105C70 111 62 129 62 150C64 156 74 156 76 150C76 134 80 122 88 114Z" fill="${sleeve}"/><path d="M117 105C130 111 138 129 138 150C136 156 126 156 124 150C124 134 120 122 112 114Z" fill="${o.sleeveR||sleeve}"/>`;
  hands=`<circle cx="69" cy="155" r="6.6" fill="${K}"/><circle cx="131" cy="155" r="6.6" fill="${K}"/>`;
 }else if(arm==="front"){
  arms=`<path d="M83 105C70 113 65 131 71 146C77 153 88 151 93 145C88 136 86 124 90 113Z" fill="${sleeve}"/><path d="M117 105C130 113 135 131 129 146C123 153 112 151 107 145C112 136 114 124 110 113Z" fill="${o.sleeveR||sleeve}"/>`;
  hands=`<circle cx="87" cy="146" r="6.6" fill="${K}"/><circle cx="113" cy="146" r="6.6" fill="${K}"/>`;
 }else{ /* oração: mãos juntas no peito */
  arms=`<path d="M83 105C71 113 68 129 74 140C80 146 90 143 96 136C90 130 88 122 90 113Z" fill="${sleeve}"/><path d="M117 105C129 113 132 129 126 140C120 146 110 143 104 136C110 130 112 122 110 113Z" fill="${o.sleeveR||sleeve}"/>`;
  hands=`<g transform="rotate(-4 100 128)"><rect x="90.6" y="111" width="9.6" height="29" rx="4.8" fill="${K}"/><rect x="99.8" y="111" width="9.6" height="29" rx="4.8" fill="${K}"/><path d="M100 114V138" stroke="${KS}" stroke-width="1.4" stroke-linecap="round"/><ellipse cx="94" cy="133" rx="3.4" ry="5.4" transform="rotate(18 94 133)" fill="${KS}" opacity=".55"/><ellipse cx="106" cy="133" rx="3.4" ry="5.4" transform="rotate(-18 106 133)" fill="${KS}" opacity=".55"/></g>`;
 }
 const neck=`<path d="M91 92H109V106Q100 111 91 106Z" fill="${KS}"/>`;
 const collar=`<path d="M86 102Q100 112 114 102" stroke="${o.tunicSh}" stroke-width="2.4" fill="none" stroke-linecap="round"/>`;
 const ears=`<ellipse cx="69.5" cy="73" rx="5.6" ry="7.2" fill="${K}"/><ellipse cx="130.5" cy="73" rx="5.6" ry="7.2" fill="${K}"/><path d="M68.5 70Q71 73 69 77M131.5 70Q129 73 131 77" stroke="${KS}" stroke-width="1.8" fill="none" stroke-linecap="round"/>`;
 const head=`<circle cx="100" cy="70" r="31" fill="${K}"/>`;
 const face=`${eyeS(88,74,5.3,"#20150F")}${eyeS(112,74,5.3,"#20150F")}
 <path d="M81.5 64.5Q88 60.5 94.5 63.5M105.5 63.5Q112 60.5 118.5 64.5" stroke="${o.brow}" stroke-width="3.1" fill="none" stroke-linecap="round"/>
 <path d="M100 76Q97 83.5 99.5 85.5Q102.5 86.2 104 84.4" stroke="${KS}" stroke-width="2.4" fill="none" stroke-linecap="round"/>
 <ellipse cx="80.5" cy="84" rx="5.2" ry="3.2" fill="#E07B68" opacity=".33"/><ellipse cx="119.5" cy="84" rx="5.2" ry="3.2" fill="#E07B68" opacity=".33"/>`;
 const mouth=`<path d="M93.5 92.5Q100 97.5 106.5 92.5" stroke="#7A3B2E" stroke-width="2.6" fill="none" stroke-linecap="round"/>`;
 return`${o.behind||""}${feet}${tunic}${belt}${mantle}${o.onBody||""}${arms}${o.held||""}${hands}${o.overHands||""}${neck}${collar}${ears}${head}${o.hairBack||""}${face}${o.beard?o.beard+(o.mouthOver||""):mouth}${o.hair||""}${o.top||""}`;
}

function salomaoS(u,s){
 const g=s===3,hair="#3A2618";
 const hairS=`<path d="M69 72C65 48 81 36 100 36C120 36 135 48 131 72C127 62 120 56 112 54C104 57 96 57 88 54C80 56 73 62 69 72Z" fill="${hair}"/>`;
 const top=`<path d="M78 46L82 30L91 40L100 26L109 40L118 30L122 46Z" fill="${GOLD}"/><path d="M78 46H122" stroke="#C9971F" stroke-width="3"/><circle cx="100" cy="33" r="3" fill="#2F6FD0"/>${s>=1?`<circle cx="86" cy="40" r="2" fill="#D6336C"/><circle cx="114" cy="40" r="2" fill="#2FB67C"/>`:""}`;
 const beard=`<path d="M71 78C72 102 85 118 100 120C115 118 128 102 129 78C125 88 118 92 112 91C108 90 104 89.5 100 89.5C96 89.5 92 90 88 91C82 92 75 88 71 78Z" fill="#3E2A1C"/><path d="M88 89.5Q100 84 112 89.5Q106 94 100 92.5Q94 94 88 89.5Z" fill="#2F2016"/>`;
 const mouthOver=`<path d="M95.5 97Q100 100 104.5 97" stroke="#E2A086" stroke-width="2.2" fill="none" stroke-linecap="round"/>`;
 const book=`<g><path d="M80 130Q90 126 100 130Q110 126 120 130V152Q110 148 100 152Q90 148 80 152Z" fill="#F4E8C8"/><path d="M100 130V152" stroke="#B9A27A" stroke-width="1.6"/><path d="M84 136H96M84 141H96M104 136H116M104 141H116M84 146H94M104 146H114" stroke="#B9A27A" stroke-width="1.4" stroke-linecap="round"/></g>`;
 const o={skin:"#D2A07A",skinSh:"#B07F5A",brow:hair,tunic:"#F4E6C8",tunicSh:"#D9C8A2",mantle:"#8C2F39",mantleSh:"#66202A",trim:GOLD,sleeve:"#F4E6C8",sleeveR:"#D9C8A2",belt:GOLD,gold:true,hair:hairS,top,beard,mouthOver,arms:s===0?"down":"front",held:s>=1?book:""};
 const fig=personS(u,o);
 if(s<2)return`${shadowS(s===0?44:46)}${fig}`;
 const temple=`<g><path d="M140 84L170 66L200 84Z" fill="#E9DCC5"/><rect x="142" y="84" width="58" height="6" fill="#D4C3A6"/>${[146,164,182].map(x=>`<rect x="${x}" y="90" width="10" height="92" rx="2" fill="#EFE4D0"/>`).join("")}<rect x="140" y="180" width="60" height="7" rx="2" fill="#D4C3A6"/></g>`;
 return`${g?auraS(u,"#FFE29A"):""}${shadowS(60,188)}<g transform="translate(-14 0)">${fig}</g>${temple}${g?legendSpark():""}`;
}

/* ---------- Mestres da música (Escola Harpa) ---------- */
const mouthOverS = c => `<path d="M95.5 97.5Q100 100.5 104.5 97.5" stroke="${c || "#C9846F"}" stroke-width="2.2" fill="none" stroke-linecap="round"/>`;
const beardS = (c, sh) => `<path d="M71 78C72 102 85 120 100 123C115 120 128 102 129 78C125 88 118 92 112 91C108 90 104 89.5 100 89.5C96 89.5 92 90 88 91C82 92 75 88 71 78Z" fill="${c}"/><path d="M88 89.5Q100 84 112 89.5Q106 94 100 92.5Q94 94 88 89.5Z" fill="${sh}"/>`;
const capHair = c => `<path d="M69 74C65 48 81 36 100 36C120 36 135 48 131 74C127 62 120 56 112 54C104 57 96 57 88 54C80 56 73 62 69 74Z" fill="${c}"/>`;
// lira/harpa (kinnor) em (x,y), escala k
function kinnorS(x, y, k, gold) {
  const f = gold ? GOLD : "#A86B32", st = gold ? "#FFF4C7" : "#F3E3C3";
  return `<g transform="translate(${x} ${y}) scale(${k})"><path d="M-14 26C-20 6-18-16-10-26M14 26C20 6 18-16 10-26" stroke="${f}" stroke-width="5" fill="none" stroke-linecap="round"/><path d="M-12-24Q0-30 12-24" stroke="${f}" stroke-width="5" fill="none" stroke-linecap="round"/><path d="M-16 26H16" stroke="${f}" stroke-width="6" stroke-linecap="round"/>${[-8, -4, 0, 4, 8].map(sx => `<path d="M${sx} 24L${sx * .8} -24" stroke="${st}" stroke-width="1.2"/>`).join("")}</g>`;
}
function notesFloat(c) { return `<g fill="${c}" opacity=".9"><circle cx="164" cy="64" r="4.5"/><path d="M168 64V44L178 41V60" stroke="${c}" stroke-width="2" fill="none"/><circle cx="174" cy="61" r="4.5"/><circle cx="30" cy="92" r="4"/><path d="M34 92V74" stroke="${c}" stroke-width="2"/></g>`; }
function sceneWrap(u, g, aura, fig, scene) { return `${g ? auraS(u, aura) : ""}${shadowS(60, 188)}<g transform="translate(-14 0)">${fig}</g>${scene}${g ? legendSpark() : ""}`; }

/* Quenanias · chefe dos levitas no canto, o professor (1 Cr 15.22) */
function quenaniasS(u, s) {
  const g = s === 3, hair = "#2E2018";
  const top = `<path d="M68 66C66 44 82 32 100 32C118 32 134 44 132 66C122 58 112 55 100 55C88 55 78 58 68 66Z" fill="#F4F1EA"/><path d="M69 60Q100 44 131 60" stroke="${g ? GOLD : "#3B5BDB"}" stroke-width="4" fill="none"/>`;
  const scroll = `<g><rect x="76" y="128" width="48" height="26" rx="2" fill="#F4E8C8"/>${[133, 138, 143, 148].map(y => `<path d="M81 ${y}H119" stroke="#B9A27A" stroke-width="1"/>`).join("")}<circle cx="88" cy="140.5" r="2.4" fill="#4A3826"/><circle cx="98" cy="136" r="2.4" fill="#4A3826"/><circle cx="108" cy="143" r="2.4" fill="#4A3826"/><rect x="71" y="125" width="8" height="32" rx="4" fill="#3B5BDB"/><rect x="121" y="125" width="8" height="32" rx="4" fill="#3B5BDB"/></g>`;
  const o = { skin: "#C99470", skinSh: "#A77452", brow: hair, tunic: "#F2ECDD", tunicSh: "#D5CCB6", mantle: "#3B5BDB", mantleSh: "#2A3F9E", trim: "#BAC8FF", sleeve: "#F2ECDD", sleeveR: "#D5CCB6", belt: "#3B5BDB", gold: g, top, beard: beardS("#2E2018", "#241811"), mouthOver: mouthOverS(), arms: s === 0 ? "down" : "front", held: s >= 1 ? scroll : "" };
  const fig = personS(u, o);
  if (s < 2) return `${shadowS(s === 0 ? 44 : 46)}${fig}`;
  const stand = `<g><path d="M170 186V120" stroke="#5C4632" stroke-width="4"/><path d="M158 186H182" stroke="#5C4632" stroke-width="4" stroke-linecap="round"/><path d="M146 96H194L188 122H152Z" fill="#F4E8C8" stroke="#5C4632" stroke-width="2"/>${[102, 107, 112, 117].map(y => `<path d="M151 ${y}H189" stroke="#B9A27A" stroke-width="1"/>`).join("")}<circle cx="160" cy="109" r="2.2" fill="#4A3826"/><circle cx="170" cy="104" r="2.2" fill="#4A3826"/><circle cx="180" cy="112" r="2.2" fill="#4A3826"/></g>`;
  return sceneWrap(u, g, "#BAC8FF", fig, stand);
}
/* Davi · jovem, ruivo, pastor e harpista (1 Sm 16.12, 16.23) */
function daviS(u, s) {
  const g = s === 3, hair = "#B5562B";
  const hairS = `<path d="M68 76C62 50 80 34 100 34C122 34 138 50 132 76C130 64 124 56 116 52C110 58 100 56 94 52C86 56 74 62 68 76Z" fill="${hair}"/><path d="M84 40Q92 34 104 38" stroke="#D9774A" stroke-width="3" fill="none" stroke-linecap="round"/>${g ? `<path d="M80 44L84 32L92 40L100 28L108 40L116 32L120 44Z" fill="${GOLD}"/>` : ""}`;
  const o = { skin: "#E3B08A", skinSh: "#C48D68", brow: "#A24A22", tunic: "#F1E3C6", tunicSh: "#D6C7A6", mantle: "#1C7ED6", mantleSh: "#1561A8", trim: "#A5D8FF", sleeve: "#F1E3C6", sleeveR: "#D6C7A6", belt: "#7A5634", gold: g, hair: hairS, arms: s === 0 ? "down" : "front", held: s >= 1 ? kinnorS(100, 136, .78, g) : "" };
  const fig = personS(u, o);
  if (s < 2) return `${shadowS(s === 0 ? 44 : 46)}${fig}`;
  const sheep = `<g><ellipse cx="170" cy="168" rx="22" ry="15" fill="#F8F6F1"/>${[[154, 160], [166, 154], [180, 156], [188, 166], [158, 176], [182, 178]].map(([x, y]) => `<circle cx="${x}" cy="${y}" r="8" fill="#F8F6F1"/>`).join("")}<ellipse cx="150" cy="166" rx="7" ry="9" fill="#3A3330"/><circle cx="148" cy="164" r="1.4" fill="#fff"/><path d="M162 182V188M176 182V188" stroke="#3A3330" stroke-width="3" stroke-linecap="round"/></g>${notesFloat("#A5D8FF")}`;
  return sceneWrap(u, g, "#A5D8FF", fig, sheep);
}
/* Miriã · profetisa com o tamborim (Êx 15.20) */
function miriaS(u, s) {
  const g = s === 3, veil = "#FFE3C2";
  const behind = `<path d="M64 72C58 104 60 134 68 152Q100 158 132 152C140 134 142 104 136 72Z" fill="${veil}"/>`;
  const hairS = `<path d="M73 66C77 54 88 48 100 48C112 48 123 54 127 66C120 60 110 58 100 58C90 58 80 60 73 66Z" fill="#3A2416"/>`;
  const top = `<path d="M64 78C62 46 80 30 100 30C120 30 138 46 136 78C134 68 130 60 124 54C116 48 108 46 100 46C92 46 84 48 76 54C70 60 66 68 64 78Z" fill="${veil}"/><path d="M70 60Q100 44 130 60" stroke="${g ? GOLD : "#E8590C"}" stroke-width="2.6" fill="none"/>`;
  const tamb = `<g><circle cx="100" cy="140" r="17" fill="#F7E7C6" stroke="${g ? GOLD : "#B5651D"}" stroke-width="4"/>${[0, 72, 144, 216, 288].map(a => { const r = a * Math.PI / 180; return `<circle cx="${(100 + 17 * Math.cos(r)).toFixed(1)}" cy="${(140 + 17 * Math.sin(r)).toFixed(1)}" r="3" fill="${GOLD}"/>`; }).join("")}<path d="M92 134Q100 128 108 134" stroke="#D9B98A" stroke-width="1.6" fill="none"/></g>`;
  const o = { skin: "#C48C64", skinSh: "#A36E48", brow: "#3A2416", tunic: "#F6EBDD", tunicSh: "#DCCDB8", mantle: "#E8590C", mantleSh: "#B8460A", trim: "#FFD8A8", sleeve: "#E8590C", sleeveR: "#C94F0B", belt: "#8C5A2A", gold: g, behind, hair: hairS, top, arms: s === 0 ? "down" : "front", held: s >= 1 ? tamb : "" };
  const fig = personS(u, o);
  if (s < 2) return `${shadowS(s === 0 ? 44 : 46)}${fig}`;
  const sea = `<g fill="none" stroke-linecap="round"><path d="M140 120Q152 110 164 120T188 120T200 118" stroke="#4DABF7" stroke-width="7"/><path d="M134 146Q148 134 162 146T190 146T200 144" stroke="#228BE6" stroke-width="8"/><path d="M140 172Q154 160 168 172T196 172" stroke="#1971C2" stroke-width="8"/></g>`;
  return sceneWrap(u, g, "#FFD8A8", fig, sea);
}
/* Asafe · chefe dos cantores, tocava os címbalos (1 Cr 16.5) */
function asafeS(u, s) {
  const g = s === 3;
  const hairS = capHair("#8F8A84");
  const cym = `<g><ellipse cx="80" cy="140" rx="14" ry="6" transform="rotate(-25 80 140)" fill="${g ? GOLD : "#E2B84A"}" stroke="#B88A1E" stroke-width="1.6"/><ellipse cx="120" cy="140" rx="14" ry="6" transform="rotate(25 120 140)" fill="${g ? GOLD : "#E2B84A"}" stroke="#B88A1E" stroke-width="1.6"/><path d="M94 128L98 122M106 128L102 122M100 126V119" stroke="#FFE08A" stroke-width="2" stroke-linecap="round"/></g>`;
  const o = { skin: "#CF9A74", skinSh: "#AE7B57", brow: "#C9C4BD", tunic: "#EDE6D6", tunicSh: "#D1C8B2", mantle: "#0E9F6E", mantleSh: "#0A7853", trim: "#96F2D7", sleeve: "#EDE6D6", sleeveR: "#D1C8B2", belt: "#6B4A2E", gold: g, hair: hairS, beard: beardS("#D9D4CC", "#C2BCB2"), mouthOver: mouthOverS(), arms: s === 0 ? "down" : "front", held: s >= 1 ? cym : "" };
  const fig = personS(u, o);
  if (s < 2) return `${shadowS(s === 0 ? 44 : 46)}${fig}`;
  return sceneWrap(u, g, "#96F2D7", fig, kinnorS(170, 150, 1.25, g) + notesFloat("#96F2D7"));
}
/* Jubal · “pai de todos os que tocam harpa e flauta” (Gn 4.21) */
function jubalS(u, s) {
  const g = s === 3, hair = "#4A3020";
  const hairS = `${capHair(hair)}<path d="M68 72C64 90 66 108 72 118L78 110C74 98 73 86 74 76Z" fill="${hair}"/><path d="M132 72C136 90 134 108 128 118L122 110C126 98 127 86 126 76Z" fill="${hair}"/>`;
  const flute = `<g><path d="M70 152L130 124" stroke="${g ? GOLD : "#C99A5B"}" stroke-width="7" stroke-linecap="round"/>${[0, 1, 2, 3].map(i => `<circle cx="${(96 + i * 7).toFixed(1)}" cy="${(140 - i * 3.3).toFixed(1)}" r="1.6" fill="#6B4A2E"/>`).join("")}</g>`;
  const o = { skin: "#B98058", skinSh: "#98633F", brow: hair, tunic: "#C9A27A", tunicSh: "#A9845E", mantle: "#9C36B5", mantleSh: "#76278A", trim: "#EEBEFA", sleeve: "#C9A27A", sleeveR: "#A9845E", belt: "#5C3A22", gold: g, hair: hairS, beard: beardS("#5A3A26", "#46301E"), mouthOver: mouthOverS(), arms: s === 0 ? "down" : "front", held: s >= 1 ? flute : "",
    onBody: `<g fill="#8A6440" opacity=".55"><circle cx="90" cy="160" r="3"/><circle cx="108" cy="170" r="2.6"/><circle cx="116" cy="154" r="2.4"/><circle cx="84" cy="176" r="2.2"/></g>` };
  const fig = personS(u, o);
  if (s < 2) return `${shadowS(s === 0 ? 44 : 46)}${fig}`;
  const tent = `<g><path d="M140 186L170 116L200 186Z" fill="#B08968"/><path d="M170 116L170 186" stroke="#7F5539" stroke-width="2"/><path d="M162 186L170 150L178 186Z" fill="#5C3A22"/></g>${kinnorS(150, 160, .7, g)}`;
  return sceneWrap(u, g, "#EEBEFA", fig, tent);
}
/* Bezalel · artesão cheio de sabedoria (Êx 31.3) */
function bezalelS(u, s) {
  const g = s === 3, hair = "#5A3A26";
  const top = `<path d="M69 60Q100 46 131 60" stroke="${g ? GOLD : "#0B7285"}" stroke-width="5" fill="none"/>`;
  const apron = `<path d="M84 112H116L120 176Q100 182 80 176Z" fill="#8C6A4A"/><path d="M84 112Q100 118 116 112" stroke="#6B4E34" stroke-width="2" fill="none"/><rect x="92" y="146" width="16" height="12" rx="2" fill="#7A5A3C"/>`;
  const hammer = `<g><path d="M104 148L124 116" stroke="#7A5634" stroke-width="5" stroke-linecap="round"/><rect x="114" y="104" width="22" height="11" rx="2" transform="rotate(32 125 110)" fill="${g ? GOLD : "#868E96"}"/></g>`;
  const o = { skin: "#C99470", skinSh: "#A77452", brow: hair, tunic: "#E9E2D2", tunicSh: "#CDC4AE", mantle: "#0B7285", mantleSh: "#085563", trim: "#99E9F2", sleeve: "#E9E2D2", sleeveR: "#CDC4AE", belt: "#5C3A22", gold: g, hair: capHair(hair), top, beard: beardS(hair, "#46301E"), mouthOver: mouthOverS(), onBody: apron, arms: s === 0 ? "down" : "front", held: s >= 1 ? hammer : "" };
  const fig = personS(u, o);
  if (s < 2) return `${shadowS(s === 0 ? 44 : 46)}${fig}`;
  const men = `<g stroke="${GOLD}" stroke-width="4" fill="none" stroke-linecap="round"><path d="M170 186V112"/><path d="M158 186H182"/><path d="M146 112Q146 140 170 140Q194 140 194 112"/><path d="M156 112Q156 130 170 130Q184 130 184 112"/></g>${[146, 156, 170, 184, 194].map(x => `<path d="M${x} 106Q${x - 3} 100 ${x} 94Q${x + 3} 100 ${x} 106Z" fill="#FFB347"/>`).join("")}`;
  return sceneWrap(u, g, "#99E9F2", fig, men);
}
const ARTMAP = { quenanias: quenaniasS, davi: daviS, miria: miriaS, asafe: asafeS, salomao: salomaoS, jubal: jubalS, bezalel: bezalelS };
const CROP = s => s >= 2 ? "44 30 84 84" : "58 30 84 84";
function mArt(id, stage, o = {}) {
  const u = "m" + (++MUID) + id, f = ARTMAP[id];
  return `<svg viewBox="${o.crop ? CROP(stage) : "0 0 200 200"}" class="mart${o.cls ? " " + o.cls : ""}"${o.label ? ` role="img" aria-label="${o.label}"` : ' aria-hidden="true"'}>${f(u, stage)}</svg>`;
}
