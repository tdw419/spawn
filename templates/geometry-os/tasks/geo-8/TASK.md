---
name: "Un-ignore the 25 GPU comparison tests"
assignee: "rust-engineer"
project: "phase-1-the-machine-runs"
---

There are 25 tests marked #[ignore] in tests/opcode_tests.rs (edge_nested_call_ret, gpu_add_sub, gpu_branch_loop, etc). These are CPU-GPU cross-validation tests that need a real GPU. Once the headless daemon is working (GEO-7), un-ignore these tests and make them pass. This is the definitive proof that software VM and GPU VM agree. Success: cargo test runs ALL tests including the 25 currently ignored ones, all pass. Workdir: ~/zion/projects/geometry_os/geometry_os/
