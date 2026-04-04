---
name: "Hilbert visualization: see programs and text on the texture"
assignee: "rust-engineer"
project: "phase-1-the-machine-runs"
---

Render the substrate as a visual PNG or window where programs appear as colored pixel regions and text rendered by CHAR instructions is visible.

What exists: visualization.rs has opcode_color() and render_hilbert_png(). substrate.png may be stale.

Need:
1. Re-render substrate PNG with latest state (font atlas region, CHAR output)
2. Continuous frame loop: cargo run opens window, renders substrate in real-time
3. Text output visible: if a program wrote HELLO at address 5000, you see the glyph pattern

The Hilbert curve makes programs spatially legible. You look at the texture and SEE where code lives, where data lives, where text was rendered.

Success: render_hilbert_png() produces an image where you can visually distinguish program code (colored by opcode), font atlas data (at 0xF00000), and rendered text output.

Workdir: ~/zion/projects/geometry_os/geometry_os/
