---
name: "Headless GPU dispatch daemon"
assignee: "rust-engineer"
project: "phase-1-the-machine-runs"
---

Wire up a headless wgpu dispatch loop that runs programs on actual GPU hardware. substrate.rs and vm.rs exist. Need: 1) headless wgpu device init (no surface) 2) compile glyph_vm_scheduler.wgsl 3) upload program pixels to texture 4) dispatch compute 5) read back results. The self-replicator (18 pixels) should copy itself on real GPU. Success: cargo run -- daemon initializes GPU, dispatches one frame, reads back, verifies copy at address 100. Workdir: ~/zion/projects/geometry_os/geometry_os/
