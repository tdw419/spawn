---
name: "Higher-level compiler: expressions and loops to pixel opcodes"
project: "phase-3-the-machine-writes-programs"
---

Workdir: ~/zion/projects/geometry_os/geometry_os

Build a simple higher-level language compiler that targets the pixel VM. The language should support:
- Arithmetic expressions: a = b + c * 2
- Functions: fn add(a, b) { return a + b }
- Loops: while (a > 0) { a = a - 1 }
- Conditionals: if (a == 0) { ... } else { ... }

This is the bridge that makes the VM programmable by AI agents without needing to write raw opcodes.

Steps:
1. Design the language grammar (keep it minimal, LLVM-like SSA is overkill)
2. Build a parser (can use Rust's nom or hand-written recursive descent)
3. Build a code generator that emits pixel opcodes
4. Test: compile a fibonacci program, load it, watch it execute
5. Test: compile a sorting algorithm, verify output in memory

Success: Write a program in the high-level language, compile it to pixels, run it on the GPU VM.
