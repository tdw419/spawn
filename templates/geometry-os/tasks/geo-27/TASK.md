---
name: "Fitness function: measure program quality objectively"
project: "phase-4-the-machine-improves-itself"
---

Workdir: ~/zion/projects/geometry_os/geometry_os

Design and implement a fitness function that can objectively measure the quality of a pixel program.

Metrics:
- Execution speed (cycles to complete)
- Correctness (does it produce expected output?)
- Memory efficiency (bytes used vs bytes available)
- Spatial locality (Hilbert distance between related instructions)

Steps:
1. Define FitnessScore struct: speed, correctness, memory, locality, composite
2. Add benchmark harness: load program, run N cycles, read output, compute score
3. Implement each metric
4. Score the existing self-replicator as a baseline
5. Test: two versions of a program get different fitness scores

Success: The self-replicator gets a fitness score. A deliberately worse version scores lower.
