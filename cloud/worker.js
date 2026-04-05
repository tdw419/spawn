/**
 * Spawn Cloud API -- Cloudflare Worker
 *
 * Proxies chat requests through ZAI (GLM) with rate-limited free tokens.
 * No user-facing API keys needed.
 *
 * Endpoints:
 *   POST /v1/register        -- issues a free anonymous token
 *   POST /v1/chat/completions -- proxies to ZAI (rate-limited)
 *   GET  /v1/status           -- returns usage for a token
 *
 * Deploy: wrangler deploy
 * Secret: wrangler secret put ZAI_API_KEY
 */

const ZAI_BASE = 'https://api.z.ai/api/coding/paas/v4';
const DAILY_LIMIT = 50;

export default {
  async fetch(request, env, ctx) {
    const url = new URL(request.url);
    const cors = {
      'Access-Control-Allow-Origin': '*',
      'Access-Control-Allow-Methods': 'GET, POST, OPTIONS',
      'Access-Control-Allow-Headers': 'Content-Type, Authorization',
    };

    if (request.method === 'OPTIONS') {
      return new Response(null, { headers: cors });
    }

    try {
      if (url.pathname === '/v1/register' && request.method === 'POST') {
        return handleRegister(request, env, cors);
      }
      if (url.pathname === '/v1/status' && request.method === 'GET') {
        return handleStatus(request, env, cors);
      }
      if (url.pathname === '/v1/chat/completions' && request.method === 'POST') {
        return handleChat(request, env, cors);
      }
      return new Response('Not found', { status: 404, headers: cors });
    } catch (e) {
      return jsonResponse({ error: e.message }, 500, cors);
    }
  },
};

// ── Register: issue free token ────────────────────────────

async function handleRegister(request, env, cors) {
  const body = await request.json().catch(() => ({}));
  const deviceId = body.device_id || 'unknown';
  const token = crypto.randomUUID();
  const today = new Date().toISOString().split('T')[0];

  await env.KV.put(`token:${token}`, JSON.stringify({
    device_id: deviceId,
    created: today,
    daily_used: 0,
    daily_limit: DAILY_LIMIT,
    daily_reset: today,
  }), { expirationTtl: 86400 * 30 });

  return jsonResponse({
    token,
    proxy_url: `https://${new URL(request.url).hostname}/v1`,
    daily_limit: DAILY_LIMIT,
  }, 200, cors);
}

// ── Status ────────────────────────────────────────────────

async function handleStatus(request, env, cors) {
  const token = extractToken(request);
  if (!token) return jsonResponse({ error: 'Missing token' }, 401, cors);

  const data = await getTokenData(env, token);
  if (!data) return jsonResponse({ error: 'Invalid token' }, 401, cors);

  return jsonResponse({ used: data.daily_used, limit: data.daily_limit }, 200, cors);
}

// ── Chat: proxy to ZAI with rate limiting ─────────────────

async function handleChat(request, env, cors) {
  const token = extractToken(request);
  if (!token) return jsonResponse({ error: 'Missing token' }, 401, cors);

  const data = await getTokenData(env, token);
  if (!data) return jsonResponse({ error: 'Invalid token' }, 401, cors);

  const today = new Date().toISOString().split('T')[0];
  if (data.daily_reset !== today) {
    data.daily_used = 0;
    data.daily_reset = today;
  }

  if (data.daily_used >= data.daily_limit) {
    return jsonResponse({
      error: 'Daily limit reached. Try again tomorrow or use your own API key.',
    }, 429, cors);
  }

  data.daily_used += 1;
  await env.KV.put(`token:${token}`, JSON.stringify(data));

  // Proxy to ZAI
  const body = await request.text();
  const parsed = JSON.parse(body);

  // Override model to GLM if needed, keep user's choice otherwise
  if (!parsed.model || parsed.model === 'gpt-4o-mini' || parsed.model === 'gpt-4o') {
    parsed.model = 'glm-4.5-air';
  }

  const zaiResp = await fetch(`${ZAI_BASE}/chat/completions`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'Authorization': `Bearer ${env.ZAI_API_KEY}`,
    },
    body: JSON.stringify(parsed),
  });

  // Clean response: strip reasoning_content (Hermes doesn't expect it)
  const respBody = await zaiResp.text();
  let cleaned = respBody;

  // For non-streaming responses, strip reasoning_content from choices
  try {
    const parsed = JSON.parse(respBody);
    if (parsed.choices) {
      for (const choice of parsed.choices) {
        if (choice.message && 'reasoning_content' in choice.message) {
          delete choice.message.reasoning_content;
        }
      }
      cleaned = JSON.stringify(parsed);
    }
  } catch (e) {
    // Streaming or non-JSON, pass through as-is
  }

  return new Response(cleaned, {
    status: zaiResp.status,
    headers: {
      'Content-Type': zaiResp.headers.get('Content-Type') || 'application/json',
      ...cors,
    },
  });
}

// ── Helpers ───────────────────────────────────────────────

function extractToken(request) {
  const auth = (request.headers.get('authorization') || '');
  if (auth.startsWith('Bearer cloud:')) return auth.slice('Bearer cloud:'.length);
  if (auth.startsWith('Bearer ')) return auth.slice('Bearer '.length);
  return null;
}

async function getTokenData(env, token) {
  const raw = await env.KV.get(`token:${token}`);
  if (!raw) return null;
  return JSON.parse(raw);
}

function jsonResponse(data, status, cors) {
  return new Response(JSON.stringify(data), {
    status,
    headers: { 'Content-Type': 'application/json', ...cors },
  });
}
