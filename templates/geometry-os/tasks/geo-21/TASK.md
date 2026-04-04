---
name: "SPAWN opcode: VM forks a child with its own memory region"
assignee: "rust-engineer"
project: "phase-2-the-machine-speaks"
---

Workdir: ~/zion/projects/geometry_os/geometry_os

Add a SPAWN opcode to the instruction set. When a VM executes SPAWN, it forks a child VM with its own memory region (base_addr..bound_addr) within the 4096x4096 texture.

Steps:
1. Add SPAWN (opcode 17) and YIELD (opcode 18) to opcode enum in lib.rs
2. Implement SPAWN in software_vm.rs: allocate a new VM slot, copy initial state, set child base_addr
3. Implement SPAWN in glyph_vm_scheduler.wgsl: same logic in the compute shader
4. Add memory isolation: each VM gets base_addr and bound_addr; out-of-bounds access faults
5. Write tests: parent spawns child, child executes independently, parent gets completion signal

Success: Two VMs running concurrently, child spawned from parent, both execute without interfering.
