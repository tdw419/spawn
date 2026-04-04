---
name: "Geotype Stage 2: Living glyph definitions (programs define new characters)"
assignee: "rust-engineer"
project: "phase-1-the-machine-runs"
---

Define a convention where GPU programs can create new glyphs at runtime and immediately use them. This is the self-hosting milestone: text that creates text.

Design:
1. Reserve memory region 0x00F20000-0x00F2FFFF as the live glyph atlas (user-defined characters, char codes 128-255)
2. Add GLYPH_DEF opcode (18): takes r_charcode, r_bitmap_addr. Reads 8 row bitmasks from r_bitmap_addr and writes them into the live atlas at the standard layout position.
3. CHAR and CHAR_AT treat char codes >= 128 as reads from the live atlas at 0x00F20000 + ((charcode - 128) * 8)
4. This means a program can compute pixel patterns, store them, define them as a glyph, then render them as text

Test:
1. Program defines a custom symbol (e.g. a smiley face) at charcode 128
2. Program uses CHAR with charcode 128
3. Verify the 8 row bitmasks at the target match the defined glyph

Depends on: GEO-18 (CHAR_AT must exist for flexible atlas addressing)

Workdir: ~/zion/projects/geometry_os/geometry_os/
