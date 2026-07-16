---
name: zircon-dev
description: Use when building, testing, refactoring, or debugging the `zirconEngine` Rust workspace, especially for Rust code, Cargo manifests, workspace wiring, or cross-crate boundaries that touch the `zircon_app`/`zircon_runtime`/`zircon_editor` root packages or the runtime-internal `core/{runtime,framework,manager,math,resource}` spine.
---

# Zircon Dev

## Start Here

- Work from the repository root.
- Read `references/main-branch-development-policy.md` before starting. It overrides any generic skill advice about creating worktrees or feature branches.
- Read `../zircon-engineering/SKILL.md` first for the standard delivery loop.
- Read `../zircon-project-skills/milestone-first-workflow-policy.md` when selecting a milestone validation gate or planning a substantial change.
- Read `workflow/SKILL.md` only for Rust workspace editing, refactors, crate boundaries, or test-tree organization.
- Read `../zircon-project-skills/prefer-windows-validation/SKILL.md` and `references/cargo-target-disk-policy.md` immediately before a Cargo command. Windows-native validation is the default; WSL is a Linux-specific exception.
- For cross-crate architecture convergence or crate-boundary changes, also read `../zircon-project-skills/zr-architecture-first-engineering/SKILL.md`.
- Read `validation/SKILL.md` before claiming the workspace is green or entering a milestone testing stage.
- Use `validation/SKILL.md` and `scripts/validate-matrix.ps1` for the selected milestone-stage Cargo batch. Do not run Cargo by default during implementation slices; use it early only for a blocker, public API/ABI change, unsafe code, persistence risk, or explicit user request.
- Read `reporting.md` before closeout so validation and risk statements stay concrete.

## Structure Rules

- Keep this root file short and navigational.
- Treat the fixed repository root packages as `zircon_app`, `zircon_runtime`, and `zircon_editor`.
- Treat `zircon_runtime::core` as the internal architecture spine for shared kernel, `framework`, `manager`, `math`, and `resource` responsibilities.
- Treat script and VM/plugin runtime work as one subsystem inside `zirconEngine`, not as the identity of the whole repository.
- Prefer loading only the child module that matches the current need instead of dragging the full workflow into context.
