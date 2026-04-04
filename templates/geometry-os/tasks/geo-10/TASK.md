---
name: "Text assembler (.gasm files)"
assignee: "rust-engineer"
project: "phase-1-the-machine-runs"
---

Phase 3 starter. Parse .gasm text files into pixel programs. Example: 'LDI r0, 72\nCHAR r0, r1\nHALT' assembles to the correct pixel sequence. This unlocks writing programs as text instead of Rust. Depends on nothing. Success: parse a .gasm file, produce a Vec<u32> of pixels, load into VM, execute, verify result. Workdir: ~/zion/projects/geometry_os/geometry_os/
