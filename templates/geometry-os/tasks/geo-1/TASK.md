---
name: "RAM texture I/O"
assignee: "rust-engineer"
project: "phase-1-the-machine-runs"
---

Load programs into texture, read texture back, verify pixel values.

1. API to write arbitrary pixel data to specific addresses in the texture
2. API to read back pixel data from specific addresses
3. Verify roundtrip: write -> dispatch (no-op) -> read -> assert equal
4. Test with the 18-pixel self-replicator: load at address 0, read back, verify match

Success: Can load a program into the texture, read it back, and verify every pixel.

Workdir: ~/zion/projects/geometry_os/geometry_os/
