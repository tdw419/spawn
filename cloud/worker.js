/**
 * Spawn Cloud API -- Cloudflare Worker
 *
 * Endpoints:
 *   POST /v1/register              -- issues a free anonymous token
 *   POST /v1/chat/completions      -- proxies to ZAI (rate-limited)
 *   GET  /v1/status                -- returns usage for a token
 *
 *   POST /v1/mesh/register         -- join the mesh for a project
 *   POST /v1/mesh/heartbeat        -- keep-alive ping
 *   GET  /v1/mesh/peers            -- list online peers
 *   POST /v1/mesh/tasks            -- push/claim/complete a task
 *   GET  /v1/mesh/tasks            -- list open tasks
 *   POST /v1/mesh/context          -- broadcast context to peers
 *   GET  /v1/mesh/context          -- read recent context
 *
 * Deploy: wrangler deploy
 * Secret: wrangler secret put ZAI_API_KEY
 */

const ZAI_BASE = 'https://api.z.ai/api/coding/paas/v4';
const DAILY_LIMIT = 200;
const HEARTBEAT_TTL = 300; // 5 minutes
const TASK_TTL = 86400;    // 24 hours
const CONTEXT_TTL = 3600;  // 1 hour

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
      // ── Auth endpoints ─────────────────────────────────────
      if (url.pathname === '/v1/register' && request.method === 'POST') {
        return handleRegister(request, env, cors);
      }
      if (url.pathname === '/v1/status' && request.method === 'GET') {
        return handleStatus(request, env, cors);
      }
      if (url.pathname === '/v1/chat/completions' && request.method === 'POST') {
        return handleChat(request, env, cors);
      }

      // ── Mesh endpoints ─────────────────────────────────────
      if (url.pathname === '/v1/mesh/register' && request.method === 'POST') {
        return handleMeshRegister(request, env, cors);
      }
      if (url.pathname === '/v1/mesh/heartbeat' && request.method === 'POST') {
        return handleMeshHeartbeat(request, env, cors);
      }
      if (url.pathname === '/v1/mesh/peers' && request.method === 'GET') {
        return handleMeshPeers(request, env, cors);
      }
      if (url.pathname === '/v1/mesh/tasks' && request.method === 'POST') {
        return handleMeshTasks(request, env, cors);
      }
      if (url.pathname === '/v1/mesh/tasks' && request.method === 'GET') {
        return handleMeshTasksList(request, env, cors);
      }
      if (url.pathname === '/v1/mesh/context' && request.method === 'POST') {
        return handleMeshContext(request, env, cors);
      }
      if (url.pathname === '/v1/mesh/context' && request.method === 'GET') {
        return handleMeshContextRead(request, env, cors);
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

  // Override model to GLM if not specified or if it's a common OpenAI model
  if (!parsed.model || parsed.model.startsWith('gpt-')) {
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

  // Clean response: strip reasoning_content
  const respBody = await zaiResp.text();
  let cleaned = respBody;

  try {
    const jsonResp = JSON.parse(respBody);
    if (jsonResp.choices) {
      for (const choice of jsonResp.choices) {
        if (choice.message && 'reasoning_content' in choice.message) {
          delete choice.message.reasoning_content;
        }
      }
      cleaned = JSON.stringify(jsonResp);
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

// ── Mesh: Register node ───────────────────────────────────

async function handleMeshRegister(request, env, cors) {
  const body = await request.json().catch(() => ({}));
  const projectId = body.project_id || 'default';
  const nodeId = `mesh:${crypto.randomUUID().slice(0, 8)}`;

  const nodeData = {
    node_id: nodeId,
    device_id: body.device_id || 'unknown',
    name: body.name || 'unnamed-node',
    capabilities: body.capabilities || {},
    project_id: projectId,
    status: 'idle',
    current_task: null,
    last_seen: Date.now(),
  };

  // Store node with 5-min TTL
  await env.KV.put(`mesh:node:${nodeId}`, JSON.stringify(nodeData), {
    expirationTtl: HEARTBEAT_TTL,
  });

  // Add to project peer list
  const peersKey = `mesh:project:${projectId}:peers`;
  const existing = await env.KV.get(peersKey);
  const peers = existing ? JSON.parse(existing) : [];
  if (!peers.find(p => p.node_id === nodeId)) {
    peers.push({ node_id: nodeId, name: nodeData.name, tier: body.capabilities?.tier || 'unknown' });
    await env.KV.put(peersKey, JSON.stringify(peers), { expirationTtl: HEARTBEAT_TTL * 2 });
  }

  return jsonResponse({
    node_id: nodeId,
    peers: peers.filter(p => p.node_id !== nodeId),
  }, 200, cors);
}

// ── Mesh: Heartbeat ───────────────────────────────────────

async function handleMeshHeartbeat(request, env, cors) {
  const body = await request.json().catch(() => ({}));
  const nodeId = body.node_id;

  if (!nodeId) return jsonResponse({ error: 'Missing node_id' }, 400, cors);

  const raw = await env.KV.get(`mesh:node:${nodeId}`);
  if (!raw) return jsonResponse({ error: 'Node not registered (expired?)' }, 404, cors);

  const nodeData = JSON.parse(raw);
  nodeData.status = body.status || 'idle';
  nodeData.current_task = body.current_task || null;
  nodeData.last_seen = Date.now();

  // Refresh TTL
  await env.KV.put(`mesh:node:${nodeId}`, JSON.stringify(nodeData), {
    expirationTtl: HEARTBEAT_TTL,
  });

  return jsonResponse({ ok: true, ttl: HEARTBEAT_TTL }, 200, cors);
}

// ── Mesh: List peers ──────────────────────────────────────

async function handleMeshPeers(request, env, cors) {
  const url = new URL(request.url);
  const projectId = url.searchParams.get('project') || 'default';

  const peersKey = `mesh:project:${projectId}:peers`;
  const raw = await env.KV.get(peersKey);
  const peers = raw ? JSON.parse(raw) : [];

  // Enrich with live data
  const enriched = [];
  for (const peer of peers) {
    const nodeRaw = await env.KV.get(`mesh:node:${peer.node_id}`);
    if (nodeRaw) {
      const node = JSON.parse(nodeRaw);
      enriched.push({
        ...peer,
        status: node.status,
        current_task: node.current_task,
        capabilities: node.capabilities,
        last_seen_ago: Math.round((Date.now() - node.last_seen) / 1000),
      });
    }
  }

  return jsonResponse({ peers: enriched, count: enriched.length }, 200, cors);
}

// ── Mesh: Tasks ───────────────────────────────────────────

async function handleMeshTasks(request, env, cors) {
  const body = await request.json().catch(() => ({}));
  const action = body.action; // push | claim | complete | release
  const projectId = body.project_id || 'default';

  const tasksKey = `mesh:project:${projectId}:tasks`;
  const raw = await env.KV.get(tasksKey);
  let tasks = raw ? JSON.parse(raw) : [];

  if (action === 'push') {
    const task = {
      id: `task:${crypto.randomUUID().slice(0, 8)}`,
      title: body.task.title || 'Untitled task',
      description: body.task.description || '',
      difficulty: body.task.difficulty || 'easy',
      requires: body.task.requires || [],
      files: body.task.files || [],
      context: body.task.context || '',
      created_by: body.node_id,
      created_at: Date.now(),
      status: 'open',
      claimed_by: null,
      claimed_at: null,
    };
    tasks.push(task);
    await env.KV.put(tasksKey, JSON.stringify(tasks), { expirationTtl: TASK_TTL });
    return jsonResponse({ task }, 200, cors);
  }

  if (action === 'claim') {
    const task = tasks.find(t => t.id === body.task_id && t.status === 'open');
    if (!task) return jsonResponse({ error: 'Task not found or already claimed' }, 404, cors);

    // Check if claimer has required capabilities (basic check)
    task.status = 'claimed';
    task.claimed_by = body.node_id;
    task.claimed_at = Date.now();

    await env.KV.put(tasksKey, JSON.stringify(tasks), { expirationTtl: TASK_TTL });
    return jsonResponse({ task }, 200, cors);
  }

  if (action === 'complete') {
    const task = tasks.find(t => t.id === body.task_id);
    if (!task) return jsonResponse({ error: 'Task not found' }, 404, cors);

    task.status = 'completed';
    task.completed_by = body.node_id;
    task.completed_at = Date.now();
    task.result = body.result || '';

    await env.KV.put(tasksKey, JSON.stringify(tasks), { expirationTtl: TASK_TTL });
    return jsonResponse({ task }, 200, cors);
  }

  if (action === 'release') {
    const task = tasks.find(t => t.id === body.task_id && t.claimed_by === body.node_id);
    if (!task) return jsonResponse({ error: 'Task not found or not yours' }, 404, cors);

    task.status = 'open';
    task.claimed_by = null;
    task.claimed_at = null;

    await env.KV.put(tasksKey, JSON.stringify(tasks), { expirationTtl: TASK_TTL });
    return jsonResponse({ task }, 200, cors);
  }

  return jsonResponse({ error: 'Unknown action. Use: push, claim, complete, release' }, 400, cors);
}

// ── Mesh: Task list ───────────────────────────────────────

async function handleMeshTasksList(request, env, cors) {
  const url = new URL(request.url);
  const projectId = url.searchParams.get('project') || 'default';
  const status = url.searchParams.get('status'); // open, claimed, completed

  const tasksKey = `mesh:project:${projectId}:tasks`;
  const raw = await env.KV.get(tasksKey);
  let tasks = raw ? JSON.parse(raw) : [];

  if (status) {
    tasks = tasks.filter(t => t.status === status);
  }

  return jsonResponse({ tasks, count: tasks.length }, 200, cors);
}

// ── Mesh: Context broadcast ───────────────────────────────

async function handleMeshContext(request, env, cors) {
  const body = await request.json().catch(() => ({}));
  const projectId = body.project_id || 'default';

  const entry = {
    id: `ctx:${crypto.randomUUID().slice(0, 8)}`,
    node_id: body.node_id,
    type: body.type || 'info',
    content: body.content || {},
    created_at: Date.now(),
  };

  const contextKey = `mesh:project:${projectId}:context`;
  const raw = await env.KV.get(contextKey);
  let entries = raw ? JSON.parse(raw) : [];

  entries.push(entry);

  // Keep last 100 entries
  if (entries.length > 100) {
    entries = entries.slice(-100);
  }

  await env.KV.put(contextKey, JSON.stringify(entries), { expirationTtl: CONTEXT_TTL });

  return jsonResponse({ entry }, 200, cors);
}

// ── Mesh: Context read ────────────────────────────────────

async function handleMeshContextRead(request, env, cors) {
  const url = new URL(request.url);
  const projectId = url.searchParams.get('project') || 'default';
  const since = url.searchParams.get('since'); // timestamp

  const contextKey = `mesh:project:${projectId}:context`;
  const raw = await env.KV.get(contextKey);
  let entries = raw ? JSON.parse(raw) : [];

  if (since) {
    entries = entries.filter(e => e.created_at > parseInt(since));
  }

  return jsonResponse({ entries, count: entries.length }, 200, cors);
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
