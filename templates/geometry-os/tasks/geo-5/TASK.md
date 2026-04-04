---
name: "GPU daemon: headless wgpu dispatch loop"
assignee: "rust-engineer"
project: "phase-1-the-machine-runs"
---

Wire up the headless wgpu dispatch loop so the self-replicator runs on actual GPU hardware. The substrate.rs and vm.rs exist, the shader exists -- need a clean daemon entry that: 1) initializes wgpu headless 2) compiles shader 3) uploads texture 4) dispatches compute 5) reads back results. This is the bridge between Phase 0 (proven on CPU) and Phase 1 (running on real GPU). Workdir: ~/zion/projects/geometry_os/geometry_os/
