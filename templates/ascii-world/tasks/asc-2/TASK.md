---
name: "Add mouse input support for framebuffer"
assignee: "engineer"
project: "pxos-core"
---

Workdir: ~/zion/projects/ascii_world/ascii_world\n\nAdd mouse event handling to the framebuffer so users can interact with ASCII elements via click/hover. The WebSocket layer should forward mouse events.\n\nSuccess: Mouse clicks register in the renderer and trigger formula re-evaluation.
