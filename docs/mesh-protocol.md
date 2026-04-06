# Spawn Mesh Protocol

## Overview

Multiple Spawn installations coordinate through a shared relay
(Spawn Cloud Worker) to divide work, share context, and combine results.

## Node Registration

When a Spawn install connects to the mesh:

```json
POST /v1/mesh/register
{
  "device_id": "uuid-of-device",
  "name": "jericho-laptop",          // optional human-readable
  "capabilities": {
    "tier": "gpu-powerful",
    "model": "qwen2.5-coder:7b",
    "gpu": "NVIDIA RTX 5090",
    "vram_mb": 24576,
    "ram_mb": 65280,
    "has_rust": true,
    "has_node": true,
    "has_git": true
  },
  "project_id": "geometry-os"
}
```

Response:
```json
{
  "node_id": "mesh:abc123",
  "peers": [
    {"node_id": "mesh:def456", "name": "sarah-desktop", "tier": "gpu-basic"},
    {"node_id": "mesh:ghi789", "name": "mike-chromebook", "tier": "cloud-only"}
  ]
}
```

## Channels

Each project gets channels for coordination:

### task-board
Agents post and claim work items.
```json
// Post a task
POST /v1/mesh/channels/task-board
{
  "node_id": "mesh:abc123",
  "action": "push",
  "task": {
    "id": "task-001",
    "title": "Fix Hilbert curve bounds check",
    "difficulty": "medium",
    "requires": ["rust", "gpu"],     // capability requirements
    "files": ["src/hilbert.rs"],
    "context": "The LDB/STB opcodes overflow on coordinates > 255..."
  }
}

// Claim a task
POST /v1/mesh/channels/task-board
{
  "node_id": "mesh:def456",
  "action": "claim",
  "task_id": "task-001"
}
```

### shared-context
Agents broadcast findings, analysis, partial results.
```json
POST /v1/mesh/channels/shared-context
{
  "node_id": "mesh:abc123",
  "type": "analysis",
  "content": {
    "summary": "Found 3 memory safety issues in cpu_stub.rs",
    "details": "...",
    "files_affected": ["src/cpu_stub.rs:112", "src/cpu_stub.rs:445"]
  }
}
```

### heartbeat
Nodes ping periodically so the mesh knows who's online.
```json
POST /v1/mesh/heartbeat
{
  "node_id": "mesh:abc123",
  "status": "idle",            // idle | working | paused
  "current_task": "task-001"   // or null
}
```

## Routing Logic

The relay routes tasks to capable nodes:

1. Task requires `["rust", "gpu"]`
2. Relay checks which online nodes have both capabilities
3. Picks the least-busy node (fewest active tasks)
4. If no capable node online, task stays in queue

## Conflict Resolution

- Each task is claimed by exactly one node at a time
- Claimed tasks have a TTL (15 minutes) -- if no heartbeat, task is released
- Results are submitted as PRs to git, so GitHub handles merge conflicts
- Critical files can be "locked" by a node while working

## Fallback

If the relay is unreachable:
- Each node operates independently (solo mode)
- Next time relay is available, nodes sync their contributions via git
- No work is lost

## Implementation

All state lives in Cloudflare KV with TTLs:
- `mesh:node:{id}` -- node registration + capabilities (TTL: 5 min, refreshed by heartbeat)
- `mesh:project:{id}:tasks` -- task queue for a project (TTL: 24 hours)
- `mesh:project:{id}:context` -- shared context feed (TTL: 1 hour)
- `mesh:project:{id}:peers` -- online nodes list (rebuilt from heartbeats)

No database. No persistent state. Nodes that stop heartbeating disappear
automatically. Tasks that expire get re-queued.
