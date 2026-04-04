---
name: "Geotype Stage 1: Atlas builder .gasm program (bold variant)"
assignee: "rust-engineer"
project: "phase-1-the-machine-runs"
---

Write a .gasm program that reads the seed font atlas at FONT_BASE (0x00F00000), creates a bold variant by OR-ing each row bitmask with itself shifted right 1 bit, and writes the result to FONT_BASE + 0x10000 (0x00F10000).

Requirements:
1. Write the program as a Rust function in src/assembler.rs (like self_replicator() or hello_world()) that builds the pixel program
2. The program must use only existing opcodes: LDI, LOAD, STORE, ADD, SUB, MUL, BNE, HALT
3. Loop over all 128 ASCII chars, 8 rows each
4. For each row: LOAD the bitmask, compute bold = row | (row >> 1), STORE the result at the derived atlas address
5. Add a test in tests/ that runs the program in the software VM, then verifies:
   - Derived atlas at 0x00F10000 is populated
   - Blank chars (0-31, 127) are still blank
   - Printable chars have MORE set bits than the original (bold is wider)
   - Spot check: 'H' bold row should be original | (original >> 1)
6. cargo test must pass

Key constants:
- FONT_BASE = 0x00F00000 (seed atlas, already loaded at boot)
- DERIVED_ATLAS_BASE = 0x00F10000 (where bold font gets written)
- FONT_CHARS = 128
- FONT_CHAR_HEIGHT = 8

Workdir: ~/zion/projects/geometry_os/geometry_os/
