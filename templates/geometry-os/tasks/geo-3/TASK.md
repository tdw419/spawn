---
name: "Hilbert visualization"
assignee: "rust-engineer"
project: "phase-1-the-machine-runs"
---

Render the RAM texture so you can SEE programs as colored regions.

1. Take the 4096x4096 texture data and render it as an image
2. Use the Hilbert curve mapping so nearby addresses appear as nearby pixels
3. Color-code: different opcodes get different colors
4. Save as PNG after each frame for inspection
5. Optional: real-time window display using wgpu render pass

Success: After running the self-replicator, you get a PNG where you can see the program as a colored region and its copy at address 100.

Workdir: ~/zion/projects/geometry_os/geometry_os/
