---
name: "Font atlas renders in PNG visualization"
project: "phase-1-the-machine-runs"
---

The font atlas exists at 0x00F00000 and CHAR opcode works. But the visualization (visualization.rs + substrate.rs PNG export) doesn't render the font data as readable text. When hello_world program executes and writes 'HELLO' to addresses 5000-5040, the PNG should show those characters as visible glyphs, not just colored pixels. Success: run hello_world, export PNG, visually confirm 'HELLO' is readable. Workdir: ~/zion/projects/geometry_os/geometry_os/
