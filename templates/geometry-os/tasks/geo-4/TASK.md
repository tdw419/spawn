---
name: "Full opcode test suite"
assignee: "rust-engineer"
project: "phase-1-the-machine-runs"
---

Every opcode tested in both software VM and GPU VM.

1. Write a test for each opcode in the software VM and check the result
2. Write a test for each opcode that loads into GPU texture, dispatches, reads back, checks
3. Compare: software VM result == GPU VM result for every opcode
4. Test edge cases: register overflow, out-of-bounds JMP, nested CALL/RET

Success: cargo test runs all opcode tests in both modes, all pass.

Workdir: ~/zion/projects/geometry_os/geometry_os/
