---
name: evidence-driven-wsl-validation
description: Debug, test, and accept ZirconEngine changes in WSL only when a concrete Linux-specific requirement remains after selecting the environment through `prefer-windows-validation`. Use for Linux-only or WSL-only failures, Linux CI reproduction, required Linux tools such as `valgrind`, `helgrind`, or `heaptrack`, sanitizers unavailable in the Windows path, or explicitly requested Linux platform validation; enforce mounted D/E/F Cargo target roots and auditable evidence.
---

# Evidence-Driven WSL Exception Validation

## Overview

Use this skill only after WSL has a documented Linux-specific purpose. Keep Windows-native validation as the default and use WSL to collect evidence that Windows cannot provide.

## Progressive Disclosure Index

- Start with `../milestone-first-workflow-policy.md` to decide whether the current work is still in an implementation slice or has entered a milestone testing stage.
- Start with `../prefer-windows-validation/SKILL.md`. Stop and return to Windows validation when no explicit WSL reason exists.
- Start with `prefer-tools-over-guessing.md`.
- If you need to place or expand tests under `tests/`, read `plan-tests-under-tests-directory.md`.
- If you need to choose, install, or run WSL debugging and validation tools, read `wsl-tool-selection/SKILL.md`.
- If you need the full acceptance workflow and required documentation format, read `acceptance-and-evidence/SKILL.md`.
- Also apply `../support-first-regression-testing/SKILL.md` when an upper-layer failure may come from a lower shared layer.
- Also apply `../../zircon-dev/SKILL.md` for the repository build matrix and baseline expectations.

## Non-Negotiable Rules

- Do not assert correctness from inspection or intuition when direct evidence can be collected.
- Do not force WSL build/test loops during every implementation slice. Use this skill for milestone testing stages, bug reproduction, deep debugging, or explicit user-requested validation.
- Do not prefer WSL for routine Cargo validation. Record the Linux-specific reason before launching it.
- Put every WSL Cargo target in the coordinator-granted mounted equivalent of one of the nine approved D/E/F roots. Never use an ad-hoc `wsl-*` path, `~`, `$HOME`, or `/home/<user>`.
- Include `platform=wsl` in the complete compatibility document and hold the coordinator lease through a live Windows host wrapper for the full WSL child-process lifetime. Never share one leaf across operating systems.
- Use the strongest appropriate tool for the failure mode before proposing a speculative fix.
- If a required mainstream tool is missing in an already-justified WSL run, install it with `apt` when permissions allow, then record the installed version in the evidence trail.
- Add or update tests under `tests/` for all in-scope behaviors, including edge cases and failure cases. Test code may be written during implementation, but compile/test execution belongs to the milestone testing stage unless earlier evidence is required.
- Do not skip, silence, or hand-wave test failures. Either fix them, prove they are pre-existing baseline failures, or leave the work unaccepted.
- Do not call the work accepted until the acceptance document says exactly what was tested, how it was tested, what failed, what was fixed, and why the remaining state is acceptable.
