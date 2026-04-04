---
name: "VM message passing (shared memory IPC)"
assignee: "rust-engineer"
project: "phase-1-the-machine-runs"
---

Phase 2 starter. Two VMs communicate by reading/writing a shared memory region. Define a message format (header pixel + data pixels), a message queue address region, and SEND/RECV opcodes or memory-mapped conventions. Success: VM 0 writes a message, VM 1 reads it, both verify content. Workdir: ~/zion/projects/geometry_os/geometry_os/
