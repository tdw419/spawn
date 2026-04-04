---
name: "Self-replication verified on real GPU dispatch"
assignee: "rust-engineer"
project: "phase-1-the-machine-runs"
---

Run the 18-pixel self-replicator through actual GPU compute dispatch and verify pixels at address 100 match pixels at address 0. Depends on GEO-5 (daemon loop). The CPU-side software VM already verifies this works -- we need the same proof on real hardware. Workdir: ~/zion/projects/geometry_os/geometry_os/
