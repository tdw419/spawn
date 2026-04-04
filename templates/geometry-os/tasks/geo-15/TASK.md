---
name: "Full opcode test coverage: every instruction tested"
assignee: "rust-engineer"
project: "phase-1-the-machine-runs"
---

Test every opcode in both software VM and GPU VM. Currently 56/56 passing but some opcodes have no tests.

Covered: NOP, LDI, MOV, LOAD, STORE, ADD, SUB, MUL, DIV, JMP, BRANCH (most conditions), CALL, RET, HALT, ENTRY, CHAR

Not tested: DRAW (215), SPAWN (230), YIELD (227), BLIT (16 -- not yet implemented)

Also fix the pre-existing sv_branch_beq_taken failure in opcode_suite.rs (BEQ branch condition).

Success: every opcode has at least one passing test in the software VM. GPU cross-validation passes for all implemented opcodes.

Workdir: ~/zion/projects/geometry_os/geometry_os/
