/**
 * Paperclip Cloud API — Cloudflare Worker
 *
 * Two endpoints:
 *   POST /v1/register   — issues a free anonymous token
 *   POST /v1/chat/completions — proxies chat requests to OpenAI (rate-limited)
 *   GET  /v1/status      — returns usage for a token
 *
 * Deploy: wrangler deploy
 * Secrets: wrangler secret put OPENAI_API_KEY
 */

export default {
  async fetch(request, env, ctx) {
    const url = new URL(request.url);
    const corsHeaders = {
      'Access-Control-Allow-Origin': '*',
      'Access-Control-Allow-Methods': 'GET, POST, OPTIONS',
      'Access-Control-Allow-Headers': 'Content-Type, Authorization',
    };

    if (request.method === 'OPTIONS') {
      return new Response(null, { headers: corsHeaders });
    }

    try {
      // ── Register ────────────────────────────────────────────
      if (url.pathname === '/v1/register' && request.method === 'POST') {
        return handleRegister(request, env, corsHeaders);
      }

      // ── Status ──────────────────────────────────────────────
      if (url.pathname === '/v1/status' && request.method === 'GET') {
        return handleStatus(request, env, corsHeaders);
      }

      // ── Chat proxy ──────────────────────────────────────────
      if (url.pathname === '/v1/chat/completions' && request.method === 'POST') {
        return handleChat(request, env, corsHeaders);
      }

      return new Response('Not found', { status: 404, headers: corsHeaders });
    } catch (e) {
      return new Response(JSON.stringify({ error: e.message }), {
        status: 500,
        headers: { 'Content-Type': 'application/json', ...corsHeaders },
      });
    }
  },
};

// ── Register: issue free token ────────────────────────────────────────

async function handleRegister(request, env, cors) {
  const body = await request.json().catch(() => ({}));
  const deviceId = body.device_id || 'unknown';
  const token = crypto.randomUUID();

  // Store token with usage tracking
  const key = `token:${token}`;
  const now = new Date().toISOString().split('T')[0]; // YYYY-MM-DD

  await env.KV.put(key, JSON.stringify({
    device_id: deviceId,
    created: now,
    daily_used: 0,
    daily_limit: 50,
    daily_reset: now,
  }), { expirationTtl: 86400 * 30 }); // 30 day TTL

  return new Response(JSON.stringify({
    token,
    proxy_url: `https://${new URL(request.url).hostname}/v1`,
    daily_limit: 50,
  }), {
    headers: { 'Content-Type': 'application/json', ...cors },
  });
}

// ── Status: check usage ───────────────────────────────────────────────

async function handleStatus(request, env, cors) {
  const token = extractToken(request);
  if (!token) {
    return new Response(JSON.stringify({ error: 'Missing token' }), {
      status: 401,
      headers: { 'Content-Type': 'application/json', ...cors },
    });
  }

  const data = await getTokenData(env, token);
  if (!data) {
    return new Response(JSON.stringify({ error: 'Invalid token' }), {
      status: 401,
      headers: { 'Content-Type': 'application/json', ...cors },
    });
  }

  return new Response(JSON.stringify({
    used: data.daily_used,
    limit: data.daily_limit,
  }), {
    headers: { 'Content-Type': 'application/json', ...cors },
  });
}

// ── Chat: proxy to OpenAI with rate limiting ──────────────────────────

async function handleChat(request, env, cors) {
  const token = extractToken(request);
  if (!token) {
    return new Response(JSON.stringify({ error: 'Missing token' }), {
      status: 401,
      headers: { 'Content-Type': 'application/json', ...cors },
    });
  }

  const data = await getTokenData(env, token);
  if (!data) {
    return new Response(JSON.stringify({ error: 'Invalid token' }), {
      status: 401,
      headers: { 'Content-Type': 'application/json', ...cors },
    });
  }

  // Reset daily counter if new day
  const today = new Date().toISOString().split('T')[0];
  if (data.daily_reset !== today) {
    data.daily_used = 0;
    data.daily_reset = today;
  }

  if (data.daily_used >= data.daily_limit) {
    return new Response(JSON.stringify({
      error: 'Daily limit reached. Try again tomorrow or use your own API key.',
    }), {
      status: 429,
      headers: { 'Content-Type': 'application/json', ...cors },
    });
  }

  // Increment usage
  data.daily_used += 1;
  await env.KV.put(`token:${token}`, JSON.stringify(data));

  // Proxy to OpenAI
  const body = await request.text();
  const openaiResp = await fetch('https://api.openai.com/v1/chat/completions', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'Authorization': `Bearer ${env.OPENAI_API_KEY}`,
    },
    body: body,
  });

  // Stream the response back
  return new Response(openaiResp.body, {
    status: openaiResp.status,
    headers: {
      'Content-Type': openaiResp.headers.get('Content-Type') || 'application/json',
      ...cors,
    },
  });
}

// ── Helpers ───────────────────────────────────────────────────────────

function extractToken(request) {
  const auth = request.headers.get('Authorization') || '';
  if (auth.startsWith('Bearer cloud:')) {
    return auth.slice('Bearer cloud:'.length);
  }
  if (auth.startsWith('Bearer ')) {
    return auth.slice('Bearer '.length);
  }
  return null;
}

async function getTokenData(env, token) {
  const raw = await env.KV.get(`token:${token}`);
  if (!raw) return null;
  return JSON.parse(raw);
}
