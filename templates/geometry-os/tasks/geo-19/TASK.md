---
name: "Boot sequence: run atlas builder before first user program"
assignee: "rust-engineer"
project: "phase-1-the-machine-runs"
---

After load_into_substrate() seeds the base font at boot, automatically load and execute the atlas builder .gasm program to generate the derived bold atlas.

In src/main.rs or src/bin/daemon.rs:
1. After font_atlas::load_into_substrate() call
2. Build the atlas builder program (assembler::atlas_builder_bold() or whatever GEO-17 names it)
3. Load it into the substrate at a reserved boot address (e.g. 0x00001000)
4. Run it through the software VM (no need for GPU dispatch for boot setup)
5. Verify the derived atlas at 0x00F10000 is populated (check a few chars)
6. If verification passes, log success. If fails, log warning but continue (base font still works)

This makes the bold font available automatically -- no manual step needed.

Depends on: GEO-17 (atlas builder program must exist first)

Workdir: ~/zion/projects/geometry_os/geometry_os/
