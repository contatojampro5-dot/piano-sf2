# IA própria do Firmes (grátis): passo a passo

Com isso, o debate com IA e o "Pergunte ao professor" funcionam **dentro do site do GitHub**, sem sair do app.

Como funciona: o app manda a conversa para um pequeno "intermediário" na Cloudflare (o arquivo `worker.js`
desta pasta). Ele guarda a sua chave do Gemini escondida, pergunta ao Gemini e devolve a resposta ao app.
A chave nunca fica dentro do site, então ninguém consegue copiá-la.

Tudo usa as faixas gratuitas: Google Gemini (sem cartão) e Cloudflare Workers (sem cartão).
Se a cota grátis do dia acabar, o app avisa e oferece o modo "No ChatGPT" ou o treino offline. Ninguém é cobrado.

---

## Parte 1: pegar a chave do Gemini (5 minutos)

1. Abra **https://aistudio.google.com** e entre com uma conta Google.
2. Aceite os termos, se aparecerem.
3. Toque em **Get API key** (ou "Obter chave de API") e depois em **Create API key** ("Criar chave").
4. Copie a chave (um texto comprido que começa com `AIza...`) e guarde num lugar seguro.
   **Não mande essa chave para ninguém, nem para mim no chat.**

## Parte 2: criar o intermediário na Cloudflare (10 minutos)

1. Abra **https://dash.cloudflare.com/sign-up** e crie uma conta gratuita (e-mail e senha). Confirme o e-mail.
2. No menu da esquerda, abra **Compute (Workers)** → **Workers & Pages** e toque em **Create** (Criar).
3. Escolha **Start with Hello World!** (começar do zero).
4. No nome, escreva `firmes-ia` e toque em **Deploy**.
5. Toque em **Edit code** (Editar código). Apague todo o código que aparecer.
6. Abra este arquivo no celular ou no computador e copie **tudo**:
   https://raw.githubusercontent.com/contatojampro5-dot/piano-sf2/main/apologia/servidor-ia/worker.js
7. Cole no editor da Cloudflare e toque em **Deploy** (no canto de cima).
8. Volte para a página do `firmes-ia` e abra **Settings** (Configurações) → **Variables and Secrets**.
9. Toque em **Add** (Adicionar):
   - Type (tipo): **Secret**
   - Variable name (nome): `GEMINI_API_KEY`
   - Value (valor): cole a chave do Gemini da Parte 1
   - Toque em **Deploy** (ou "Save and deploy").
10. *(Opcional, recomendado)* IA de reserva da própria Cloudflare, também grátis: em **Settings** → **Bindings**
    → **Add** → **Workers AI** → no nome escreva `AI` → **Deploy**. Se o Gemini estiver fora do ar ou sem cota,
    o app usa essa reserva.

## Parte 3: me mandar o endereço

No topo da página do `firmes-ia` aparece um endereço assim:

`https://firmes-ia.SEU-NOME.workers.dev`

Mande **só esse endereço** para o Zeus (ele não é segredo). Eu coloco no app e envio para o GitHub.

Quer testar antes? Abra o app com o endereço no fim, assim:

`https://contatojampro5-dot.github.io/piano-sf2/apologia/?ia=https://firmes-ia.SEU-NOME.workers.dev`

Entre em **Debate**, escolha um oponente e converse. Se a aba "Conversa com IA" aparecer e o personagem
responder, está funcionando.

---

## Bom saber

- **Privacidade:** na faixa gratuita, o Google pode usar as conversas para melhorar os produtos dele. Oriente os
  jovens a não escrever nome completo, endereço, telefone ou outros dados pessoais nos debates.
- **Limites:** o Google não publica um número fixo de mensagens grátis por dia; ele aparece no AI Studio, no seu
  projeto. Para um grupo de jovens costuma bastar. O intermediário também limita cada aparelho a 60 mensagens por hora.
- **Trocar o modelo:** em Settings → Variables, crie a variável `GEMINI_MODELS` com os nomes separados por vírgula.
  O padrão é `gemini-3.5-flash-lite,gemini-flash-lite-latest,gemini-flash-latest`: se o primeiro esgotar, tenta o próximo.
- **Outros sites:** o intermediário só responde ao site `https://contatojampro5-dot.github.io`. Para liberar outro
  endereço, crie a variável `ALLOWED_ORIGINS` com os endereços separados por vírgula.
- **Desligar:** em Settings, apague o segredo `GEMINI_API_KEY`, ou apague o Worker. O app volta sozinho para o modo
  "No ChatGPT".
