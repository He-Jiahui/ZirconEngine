# Evidence-Driven WSL Exception Validation

## Overview

Use this skill only after WSL has a documented Linux-specific purpose. Keep Windows-native validation as the default and use WSL to collect evidence that Windows cannot provide.

## Progressive Disclosure Index

- Start with `../milestone-first-workflow-policy.md` to decide whether the current work is still in an implementation slice or has entered a milestone testing stage.
- Start with `../prefer-windows-validation/guide.md`. Stop and return to Windows validation when no explicit WSL reason exists.
- Start with `prefer-tools-over-guessing.md`.
- If you need to choose, install, or run WSL debugging and validation tools, read `wsl-tool-selection/index.md`.
- If you need the full acceptance workflow and required documentation format, read `acceptance-and-evidence/index.md`.
- Also apply `../support-first-regression-testing/guide.md` when an upper-layer failure may come from a lower shared layer.
- Also apply `../../zircon-dev/SKILL.md` for the repository build matrix and baseline expectations.

## Non-Negotiable Rules

- Do not assert correctness from inspection or intuition when direct evidence can be collected.
- Do not force WSL build/test loops during every implementation slice. Use this skill for milestone testing stages, bug reproduction, deep debugging, or explicit user-requested validation.
- Do not prefer WSL for routine Cargo validation. Record the Linux-specific reason before launching it.
- Put every WSL Cargo target and compiler cache in the physical mounted equivalent of `D:\cargo-targets`, `E:\cargo-targets`, or `F:\cargo-targets`. Use `python3 -B -m tools.dev.local_cargo`; never use `~`, `$HOME`, `/home/<user>`, or an aliased root.
- Record the Linux platform, toolchain, target, profile and feature configuration in validation evidence and keep the invoking process alive for the full child-process lifetime. Preserve Cargo's locks; no coordinator lease is required. Never share one leaf across operating systems.
- Use the strongest appropriate tool for the failure mode before proposing a speculative fix.
- If a required mainstream tool is missing in an already-justified WSL run, install it with `apt` when permissions allow, then record the installed version in the evidence trail.
- Cover the milestone's changed behaviors with focused tests scoped per `docs/plans/milestone-validation-policy.md`: representative coverage plus targeted boundary/failure variants for the risks the milestone actually touches. Test code may be written during implementation, but compile/test execution belongs to the milestone testing stage unless earlier evidence is required.
- Do not skip, silence, or hand-wave test failures. Either fix them, prove they are pre-existing baseline failures, or leave the work unaccepted.
- Record milestone evidence in the authorized child-plan output per `write-plan-output-records`: what was tested, what failed, what was fixed, and why the remaining state is acceptable.
