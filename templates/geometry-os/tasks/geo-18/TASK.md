---
name: "CHAR_AT opcode: blit character from arbitrary atlas base address"
assignee: "rust-engineer"
project: "phase-1-the-machine-runs"
---

The existing CHAR opcode (15) hardcodes FONT_BASE (0x00F00000) as the atlas source. For Geotype self-hosting, programs need to blit characters from ANY atlas -- including the derived bold atlas at 0x00F10000.

Add CHAR_AT opcode (21):  NOTE: 17 is taken by SEND, 18 by RECV, 19 by SHR, 20 by OR. Use 21.
  CHAR_AT r_ascii, r_target, r_atlas_base

Like CHAR but reads from atlas_base + (ascii * 8) + row instead of hardcoded FONT_BASE.

Implementation:
1. Add op::CHAR_AT = 21 in src/assembler.rs
2. Add char_at_blit() method to Program
3. Add parser support in parse_gasm: CHAR_AT r0, r1, r2
4. Implement in software_vm.rs (mirror CHAR but use register for base)
5. Implement in shaders/glyph_vm_scheduler.wgsl (case 21u)
6. Keep existing CHAR opcode (15) unchanged for backward compatibility
7. Tests:
   - CHAR_AT using FONT_BASE produces identical results to CHAR
   - CHAR_AT using derived atlas base reads from correct location

Workdir: ~/zion/projects/geometry_os/geometry_os/
