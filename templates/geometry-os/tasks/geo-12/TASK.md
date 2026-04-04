---
name: "Font atlas + CHAR opcode: pixel-based text rendering"
assignee: "rust-engineer"
project: "phase-1-the-machine-runs"
---

COMPLETED. Shipped in commit 2082357.

What was built:
- font_atlas module: 8x8 PC BIOS bitmap font (printable ASCII 32-126), 1024 bytes stored as ROM at Hilbert address 0xF00000 (0.006 percent of texture)
- CHAR opcode (15): blits one character 8 row bitmasks from font atlas to any substrate address. Programs now render text as pixels.
- Software VM: full CHAR execution + load_font_atlas() for deterministic testing without GPU
- Assembler: char_blit() builder + hello_world() program (26 pixels, 5 chars)
- Visualization: spring green (CHAR) and lime (BLIT) opcode colors
- 56/56 tests passing

Key insight: the engine (Rust) compiles once. Programs (pixels) build forever. The font atlas is the bridge between layers -- characters become data in the texture that any program can read.

Docs: docs/FONT_ATLAS_AND_TEXT.md
