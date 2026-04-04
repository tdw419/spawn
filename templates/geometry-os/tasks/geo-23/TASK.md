---
name: "Event queue: external events injected into VMs"
project: "phase-2-the-machine-speaks"
---

Workdir: ~/zion/projects/geometry_os/geometry_os

Create an event injection mechanism so the daemon can push external events (keyboard, timer, network) into running VMs.

Steps:
1. Define Event struct in lib.rs: event_type, param1, param2, timestamp
2. Reserve event buffer region in texture
3. Add INJECT_EVENT to the daemon's API
4. Add WAIT_EVENT opcode (21): VM blocks until an event arrives
5. Test: daemon injects a keyboard event, VM receives it via WAIT_EVENT

Success: Daemon injects event into running VM, VM reads it and branches based on event type.
