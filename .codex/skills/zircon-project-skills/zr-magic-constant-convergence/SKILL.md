---
name: zr-magic-constant-convergence
description: Use when consolidating magic numbers or raw constants in `zirconEngine` production Rust code, auditing bare sentinels or thresholds, or deciding whether a value belongs in a shared crate module, a crate-local constants/config module, a private module constant, or an explicit definition-bound exemption.
---

# Zr Magic Constant Convergence

## Overview

Apply the `zirconEngine` constant-convergence ladder without over-centralizing definition-bound values. The goal is to replace scattered policy, API, protocol, threshold, and helper-control literals with named constants while keeping schema, layout, enum ordinals, and descriptor-bound values local and explicitly exempt.

## Start Here

- Read `references/placement-ladder.md` before promoting any literal into shared scope.
- Use this skill only on production Rust code under `zircon_*`.
- If the change also alters shared runtime or cross-crate behavior, pair this skill with `../../zircon-dev/SKILL.md`.

## Non-Negotiable Rules

- Put real cross-crate runtime, API, protocol, path, version, and shared-threshold constants in the shared crate or shared module that already owns the contract.
- Put crate-scoped reusable configuration in that crate's `constants.rs`, `config.rs`, or other narrow module that already defines related policy.
- Keep helper-local sentinels, compare results, fallback capacities, growth factors, and depth guards as private module-level `const` items unless a real shared contract has already formed.
- Keep `schema/layout/enum ordinal/sizeof/table-count` values explicitly exempt unless the project intentionally promotes them into a dedicated protocol or metadata module.
- Reuse an existing named constant when it already captures the same contract. Do not create synonyms.
- Document the chosen placement in the change summary or nearby comments when the decision is non-obvious.

## Workflow

1. Scan only production Rust code.
- Exclude `tests/`, `target/`, generated outputs, and vendored trees under `dev/`.
- Prefer `rg` or `Select-String` over ad-hoc scanning.

2. Classify each hit before editing.
- Ask whether the value expresses a shared contract, crate policy, helper control flow, or a definition-bound shape.
- If the answer is ambiguous, bias toward local scope first and promote later only when cross-file reuse is proven.

3. Edit to the narrowest correct scope.
- Shared contract: add to the shared crate or module that already owns the boundary.
- Crate policy: add to a crate-local constants/config module.
- Helper-local rule: add a private `const` near the top of the module with a narrow name.
- Exemption: leave the value in place and record why.

4. Verify before reporting completion.
- For code changes under shared runtime, scripting, module wiring, editor, graphics, or other cross-crate engine surfaces, run `.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1`.
- Re-scan the touched Rust files to confirm the raw literal pattern actually disappeared or stayed intentionally exempt.

## Quick Commands

```powershell
rg -n --glob "*.rs" --glob "!**/target/**" --glob "!**/dev/**" "\b4096\b|\b256\b|0x[0-9A-Fa-f]+|usize::MAX|u64::MAX" zircon_*
Get-ChildItem -Path .\zircon_* -Recurse -Filter *.rs | Select-String -Pattern '\b4096\b|\b256\b|0x[0-9A-Fa-f]+|usize::MAX|u64::MAX'
```

## Common Mistakes

- Moving schema or layout values into a shared crate just because they are numeric.
- Promoting a helper-local sentinel into shared scope before it is reused across files.
- Adding a named constant but forgetting to re-check the raw literal in the rest of the touched crate.
- Declaring success from one grep hit without checking the rest of the touched crate.
- Treating `0/1` returns in ordinary boolean helpers as magic numbers when they are just the function's boolean result contract.

## Repository Anchors

- Placement ladder: `references/placement-ladder.md`
- Workspace root: `Cargo.toml`
- Shared-boundary modules: `zircon_runtime::core::{runtime,manager,framework,math,resource}` plus `zircon_runtime::engine_module` when the contract is descriptor-level
- Feature crates: the affected `zircon_*` crate or module
