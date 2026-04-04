---
name: "Runtime loader: load assembled programs without restart"
assignee: "rust-engineer"
project: "phase-3-the-machine-writes-programs"
---

Workdir: ~/zion/projects/geometry_os/geometry_os

Build a runtime program loader that can inject new assembled programs into the texture while the daemon is running.

Steps:
1. Add load_program() to substrate.rs that writes pixels to a free region of the texture
2. Add allocate_region() that finds contiguous free space using a simple bitmap allocator
3. Expose via daemon API: POST /api/v1/programs with the assembled pixel data
4. Test: assemble a .gasm file, load it at runtime, dispatch VM to execute it
5. Verify loaded program runs correctly alongside existing VMs

Success: Assemble a .gasm file, POST it to the daemon, a new VM picks it up and runs it. No restart.
