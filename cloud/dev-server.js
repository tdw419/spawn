#!/usr/bin/env node
/**
 * Local dev server for Paperclip Cloud API
 * Simulates the Cloudflare Worker for testing
 *
 * Usage: node dev-server.js
 * Listens on http://localhost:8787
 */

const http = require('http');
const crypto = require('crypto');

const PORT = 8787;

// In-memory token store (resets on restart)
const tokens = new Map();

// Config
const DAILY_LIMIT = 50;
const OPENAI_API_KEY = process.env.OPENAI_API_KEY || '';

const server = http.createServer(async (req, res) => {
  const url = new URL(req.url, `http://localhost:${PORT}`);
  const corsHeaders = {
    'Access-Control-Allow-Origin': '*',
    'Access-Control-Allow-Methods': 'GET, POST, OPTIONS',
    'Access-Control-Allow-Headers': 'Content-Type, Authorization',
  };

  if (req.method === 'OPTIONS') {
    res.writeHead(200, corsHeaders);
    res.end();
    return;
  }

  // ── POST /v1/register
  if (url.pathname === '/v1/register' && req.method === 'POST') {
    const token = crypto.randomUUID();
    const today = new Date().toISOString().split('T')[0];
    tokens.set(token, {
      device_id: 'dev',
      created: today,
      daily_used: 0,
      daily_limit: DAILY_LIMIT,
      daily_reset: today,
    });

    console.log(`[register] Issued token: ${token.slice(0, 8)}...`);
    jsonResponse(res, 200, {
      token,
      proxy_url: `http://localhost:${PORT}/v1`,
      daily_limit: DAILY_LIMIT,
    }, corsHeaders);
    return;
  }

  // ── GET /v1/status
  if (url.pathname === '/v1/status' && req.method === 'GET') {
    const token = extractToken(req);
    if (!token || !tokens.has(token)) {
      jsonResponse(res, 401, { error: 'Invalid token' }, corsHeaders);
      return;
    }
    const data = tokens.get(token);
    jsonResponse(res, 200, { used: data.daily_used, limit: data.daily_limit }, corsHeaders);
    return;
  }

  // ── POST /v1/chat/completions
  if (url.pathname === '/v1/chat/completions' && req.method === 'POST') {
    const token = extractToken(req);
    if (!token || !tokens.has(token)) {
      jsonResponse(res, 401, { error: 'Invalid token' }, corsHeaders);
      return;
    }

    const data = tokens.get(token);
    const today = new Date().toISOString().split('T')[0];
    if (data.daily_reset !== today) {
      data.daily_used = 0;
      data.daily_reset = today;
    }

    if (data.daily_used >= data.daily_limit) {
      jsonResponse(res, 429, { error: 'Daily limit reached' }, corsHeaders);
      return;
    }

    data.daily_used += 1;
    tokens.set(token, data);

    if (!OPENAI_API_KEY) {
      // Dev mode: return a fake response instead of proxying
      console.log(`[chat] Token ${token.slice(0, 8)}... request ${data.daily_used}/${data.daily_limit} (mocked)`);
      const body = await readBody(req);
      const parsed = JSON.parse(body);
      const mockResponse = {
        id: 'chatcmpl-dev-' + crypto.randomUUID().slice(0, 8),
        object: 'chat.completion',
        created: Math.floor(Date.now() / 1000),
        model: parsed.model || 'gpt-4o-mini',
        choices: [{
          index: 0,
          message: {
            role: 'assistant',
            content: `[Dev mode] Paperclip Cloud proxy is working. Token ${token.slice(0, 8)}... request ${data.daily_used}/${data.daily_limit}. Set OPENAI_API_KEY env var to proxy real requests.`,
          },
          finish_reason: 'stop',
        }],
        usage: { prompt_tokens: 10, completion_tokens: 20, total_tokens: 30 },
      };
      jsonResponse(res, 200, mockResponse, corsHeaders);
      return;
    }

    // Proxy to real OpenAI
    const body = await readBody(req);
    try {
      const openaiRes = await fetch('https://api.openai.com/v1/chat/completions', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${OPENAI_API_KEY}`,
        },
        body: body,
      });

      const respBody = await openaiRes.text();
      res.writeHead(openaiRes.status, {
        'Content-Type': 'application/json',
        ...corsHeaders,
      });
      res.end(respBody);
    } catch (e) {
      jsonResponse(res, 502, { error: `OpenAI proxy failed: ${e.message}` }, corsHeaders);
    }
    return;
  }

  res.writeHead(404, corsHeaders);
  res.end('Not found');
});

function jsonResponse(res, status, data, headers = {}) {
  res.writeHead(status, { 'Content-Type': 'application/json', ...headers });
  res.end(JSON.stringify(data));
}

function extractToken(req) {
  const auth = req.headers['authorization'] || '';
  if (auth.startsWith('Bearer cloud:')) return auth.slice('Bearer cloud:'.length);
  if (auth.startsWith('Bearer ')) return auth.slice('Bearer '.length);
  return null;
}

function readBody(req) {
  return new Promise((resolve) => {
    let body = '';
    req.on('data', chunk => body += chunk);
    req.on('end', () => resolve(body));
  });
}

server.listen(PORT, () => {
  console.log(`\n  Paperclip Cloud Dev Server`);
  console.log(`  http://localhost:${PORT}`);
  console.log(`  Daily limit: ${DAILY_LIMIT}`);
  console.log(`  OpenAI proxy: ${OPENAI_API_KEY ? 'enabled' : 'mocked (set OPENAI_API_KEY to enable)'}\n`);
});
