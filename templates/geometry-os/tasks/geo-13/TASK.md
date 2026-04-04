---
name: "BLIT opcode (16): bulk pixel region copy"
assignee: "rust-engineer"
project: "phase-1-the-machine-runs"
---

Implement the BLIT opcode for copying arbitrary pixel regions in the substrate.

Spec: BLIT r_src, r_dst [count] -- copies N pixels from source address to destination address. Count is the next pixel after the instruction (2-word). Already defined in assembler (op::BLIT = 16, blit() builder exists). Need: software VM execution, WGSL shader execution, tests.

This is the generic primitive for self-modifying programs. With BLIT you can copy program code, duplicate data structures, or move screen regions.

Success: cargo test blit passes in software VM. BLIT copies N pixels from src to dst correctly.

Workdir: ~/zion/projects/geometry_os/geometry_os/
