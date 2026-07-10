---
name: evidence-driven-wsl-validation
description: Debug, test, and accept `zirconEngine` changes with WSL-first evidence instead of guesswork. Use when shared runtime, scripting, graphics, editor, platform, networking, or other Rust workspace work needs Linux-first or CI-parity validation with tools such as `cargo`, `gdb`, `lldb`, `valgrind`, `helgrind`, `heaptrack`, or sanitizers, may require installing missing tools with `apt`, and must produce detailed acceptance documentation without ignoring failures or boundary cases.
---

# Evidence-Driven WSL Validation

## Overview

Use this skill to turn `zirconEngine` debugging and test validation into an evidence-first workflow with explicit tooling, boundary coverage, and auditable acceptance records.

## Progressive Disclosure Index

- Start with `../milestone-first-workflow-policy.md` to decide whether the current work is still in an implementation slice or has entered a milestone testing stage.
- Start with `prefer-tools-over-guessing.md`.
- If you need to place or expand tests under `tests/`, read `plan-tests-under-tests-directory.md`.
- If you need to choose, install, or run WSL debugging and validation tools, read `wsl-tool-selection/SKILL.md`.
- If you need the full acceptance workflow and required documentation format, read `acceptance-and-evidence/SKILL.md`.
- Also apply `../support-first-regression-testing/SKILL.md` when an upper-layer failure may come from a lower shared layer.
- Also apply `../../zircon-dev/SKILL.md` for the repository build matrix and baseline expectations.

## Non-Negotiable Rules

- Do not assert correctness from inspection or intuition when direct evidence can be collected.
- Do not force WSL build/test loops during every implementation slice. Use this skill for milestone testing stages, bug reproduction, deep debugging, or explicit user-requested validation.
- Prefer WSL/Linux as the primary environment for deep debugging and validation.
- Use the strongest appropriate tool for the failure mode before proposing a speculative fix.
- If a mainstream tool is missing in WSL, install it with `apt` when permissions allow, then record the installed version in the evidence trail.
- Add or update tests under `tests/` for all in-scope behaviors, including edge cases and failure cases. Test code may be written during implementation, but compile/test execution belongs to the milestone testing stage unless earlier evidence is required.
- Do not skip, silence, or hand-wave test failures. Either fix them, prove they are pre-existing baseline failures, or leave the work unaccepted.
- Do not call the work accepted until the acceptance document says exactly what was tested, how it was tested, what failed, what was fixed, and why the remaining state is acceptable.
