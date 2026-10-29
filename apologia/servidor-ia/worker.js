/* IA do Firmes: intermediário (Cloudflare Worker) entre o app e o Gemini.
   A chave do Gemini fica guardada aqui como segredo (GEMINI_API_KEY) e nunca vai para o app.
   Ordem: modelos do Gemini (GEMINI_MODELS); se todos falharem e houver o vínculo "AI",
   usa a IA da própria Cloudflare (CF_MODEL). Guia de instalação: GUIA.md nesta pasta. */

const DEFAULT_ORIGINS = "https://contatojampro5-dot.github.io";
const DEFAULT_MODELS = "gemini-3.5-flash-lite,gemini-flash-lite-latest,gemini-flash-latest";
const DEFAULT_CF_MODEL = "@cf/meta/llama-3.3-70b-instruct-fp8-fast";
const MAX_MESSAGES = 16, MAX_CHARS = 14000, PER_HOUR = 60;

const hits = new Map(); // limite simples por IP (vale por instância do Worker)
function tooMany(ip) {
  const now = Date.now(), h = (hits.get(ip) || []).filter(t => now - t < 3600e3);
  h.push(now); hits.set(ip, h);
  if (hits.size > 5000) hits.clear();
  return h.length > PER_HOUR;
}

function cors(origin, allowed) {
  const ok = allowed.includes(origin) || /^https:\/\/[a-z0-9-]+\.claudeusercontent\.com$/.test(origin);
  return {
    "Access-Control-Allow-Origin": ok ? origin : allowed[0],
    "Access-Control-Allow-Methods": "POST, OPTIONS",
    "Access-Control-Allow-Headers": "Content-Type",
    "Vary": "Origin",
  };
}
const reply = (body, status, h) => new Response(JSON.stringify(body), { status, headers: { ...h, "Content-Type": "application/json" } });

async function gemini(env, model, msgs, json) {
  const r = await fetch(`https://generativelanguage.googleapis.com/v1beta/models/${model}:generateContent`, {
    method: "POST",
    headers: { "Content-Type": "application/json", "x-goog-api-key": env.GEMINI_API_KEY },
    body: JSON.stringify({
      contents: msgs.map(m => ({ role: m.role === "assistant" ? "model" : "user", parts: [{ text: m.content }] })),
      generationConfig: { maxOutputTokens: json ? 3000 : 1500, ...(json ? { responseMimeType: "application/json" } : {}) },
    }),
  });
  if (!r.ok) { const e = new Error("gemini " + r.status); e.status = r.status; throw e; }
  const d = await r.json(), c = d.candidates && d.candidates[0];
  const text = ((c && c.content && c.content.parts) || []).filter(p => !p.thought).map(p => p.text || "").join("").trim();
  if (!text) { const e = new Error(c && c.finishReason === "SAFETY" ? "refused" : "empty"); e.status = 502; throw e; }
  return text;
}

async function cloudflare(env, msgs) {
  const out = await env.AI.run(env.CF_MODEL || DEFAULT_CF_MODEL, { messages: msgs, max_tokens: 1200 });
  const text = (out && (out.response || (out.result && out.result.response)) || "").trim();
  if (!text) throw new Error("empty");
  return text;
}

export default {
  async fetch(request, env) {
    const allowed = (env.ALLOWED_ORIGINS || DEFAULT_ORIGINS).split(",").map(s => s.trim()).filter(Boolean);
    const origin = request.headers.get("Origin") || "", h = cors(origin, allowed);
    if (request.method === "OPTIONS") return new Response(null, { status: 204, headers: h });
    if (request.method === "GET") return reply({ ok: true, app: "Firmes IA" }, 200, h);
    if (request.method !== "POST") return reply({ error: "method" }, 405, h);
    if (!h["Access-Control-Allow-Origin"] || h["Access-Control-Allow-Origin"] !== origin) return reply({ error: "origin" }, 403, h);
    if (tooMany(request.headers.get("CF-Connecting-IP") || "?")) return reply({ error: "rate_limited" }, 429, h);

    let body;
    try { body = await request.json(); } catch (e) { return reply({ error: "bad_request" }, 400, h); }
    let msgs = Array.isArray(body.messages) ? body.messages : [];
    msgs = msgs.slice(-MAX_MESSAGES)
      .filter(m => m && typeof m.content === "string" && m.content.trim())
      .map(m => ({ role: m.role === "assistant" ? "assistant" : "user", content: m.content.slice(0, MAX_CHARS) }));
    // junta falas seguidas do mesmo lado e garante que a conversa comece pelo usuário
    const merged = [];
    for (const m of msgs) {
      const last = merged[merged.length - 1];
      if (last && last.role === m.role) last.content += "\n\n" + m.content; else merged.push({ ...m });
    }
    while (merged.length && merged[0].role !== "user") merged.shift();
    if (!merged.length || merged[merged.length - 1].role !== "user") return reply({ error: "bad_request" }, 400, h);
    const json = !!body.json;

    let lastErr = null;
    if (env.GEMINI_API_KEY) {
      for (const model of (env.GEMINI_MODELS || DEFAULT_MODELS).split(",").map(s => s.trim()).filter(Boolean)) {
        try { return reply({ text: await gemini(env, model, merged, json), via: model }, 200, h); }
        catch (e) { lastErr = e; if (e.message === "refused") return reply({ error: "refused" }, 200, h); }
      }
    }
    if (env.AI) {
      try { return reply({ text: await cloudflare(env, merged), via: "cloudflare" }, 200, h); }
      catch (e) { lastErr = e; }
    }
    const quota = lastErr && lastErr.status === 429;
    return reply({ error: quota ? "quota" : "unavailable" }, quota ? 429 : 503, h);
  },
};
