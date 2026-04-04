---
name: "Software VM -- CPU-side mirror of the shader"
assignee: "rust-engineer"
project: "phase-1-the-machine-runs"
---

CPU-side VM that mirrors the compute shader exactly for testing. SoftwareVm struct with same state layout: 8 VMs, 128 registers, PC, call stack, stratum. Every opcode implemented. Success: Software VM executes self-replicator producing same state as shader. Workdir: ~/zion/projects/geometry_os/geometry_os/
