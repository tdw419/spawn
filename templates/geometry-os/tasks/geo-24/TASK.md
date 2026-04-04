---
name: "Standard library: pre-loaded pixel programs for common routines"
project: "phase-2-the-machine-speaks"
---

Workdir: ~/zion/projects/geometry_os/geometry_os

Build a stdlib of common routines as pre-assembled pixel programs that get loaded into every VM at boot.

Programs to include:
- print: write a string to the text buffer using CHAR opcode
- read: read a null-terminated string from memory
- draw_rect: draw a rectangle to a screen region
- memset: fill a memory region with a value
- memcpy: copy a memory region (wraps BLIT)
- strcmp: compare two strings in memory

Steps:
1. Create src/stdlib.rs with Program builders for each routine
2. Reserve stdlib region in texture (e.g., 0x00D00000-0x00DFFFFF)
3. Load stdlib at daemon startup before any user programs
4. Write tests for each routine
5. Document the stdlib API

Success: A user program can CALL a stdlib routine by address and it works.
