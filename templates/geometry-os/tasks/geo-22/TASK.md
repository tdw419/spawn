---
name: "Message queue: VMs send messages via shared memory mailbox"
project: "phase-2-the-machine-speaks"
---

Workdir: ~/zion/projects/geometry_os/geometry_os

Design and implement an IPC message queue where VMs can send fixed-size messages to each other via a shared memory region in the texture.

Steps:
1. Reserve a mailbox region in the texture (e.g., addresses 0x00E00000-0x00EFFFFF)
2. Define message format: [sender_vm, dest_vm, opcode, arg1, arg2, arg3, arg4]
3. Implement SEND opcode (19): write message to destination mailbox
4. Implement RECV opcode (20): check mailbox, block if empty
5. Add mailbox overflow protection
6. Test: VM 0 sends to VM 1, VM 1 receives and acts on the message

Success: Two VMs communicate. VM 0 sends a value, VM 1 receives it and writes it to memory.
