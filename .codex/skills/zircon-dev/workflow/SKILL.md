---
name: zircon-dev-workflow
description: Use when `zircon-dev` work needs the repository-specific editing, refactor, and workspace-boundary rules for the zirconEngine Rust workspace.
---

# Zircon Dev Workflow

## Start Here

- Read `../references/main-branch-development-policy.md` first.
- Read `../../zircon-project-skills/milestone-first-workflow-policy.md` before choosing implementation or validation cadence.
- Read `../../zircon-project-skills/zr-architecture-first-engineering/references/system-architecture-baseline.md` before reorganizing shared crate boundaries or public architecture names.
- Read `structure/module-layering.md` before creating or reorganizing non-test source modules.
- Read `structure/root-entry-files.md` before touching `binding.rs`, `lib.rs`, `main.rs`, or production `mod.rs` files, or before creating a new root wiring file.
- Read `refactor-rules.md` before changing behavior or deleting compatibility paths.
- Read `workspace-map.md` before crossing crate boundaries or editing shared runtime contracts.
- Read `testing/mod-rs-map.md` before reorganizing crate unit tests, replacing inline `mod tests`, or scaffolding new test trees.
- If a touched implementation file is approaching or exceeding roughly 1000 lines, also apply `../../zircon-project-skills/modularize-large-files/SKILL.md`.

## Non-Negotiable Rules

- Treat this repository as a Rust workspace rooted at `Cargo.toml`, not as a single-crate project.
- Follow the converged package spine first:
  - `zircon_app` for entry, profile selection, and main-loop hosting
  - `zircon_runtime` for the absorbed runtime business logic plus the internal `core/{runtime,framework,manager,math,resource}` spine
  - `zircon_editor` for editor host and authoring logic
- Treat `zircon_runtime::core::runtime` as the lifecycle, dependency, registration, and scheduling kernel.
- Treat `zircon_runtime::core::manager` as the stable service-name, resolver, handle, activation, and access-entry layer.
- Treat `zircon_runtime::core::framework` as the shared contracts and neutral DTO layer consumed by runtime and editor code.
- Treat `zircon_runtime::core::math` and `zircon_runtime::core::resource` as the canonical shared math and resource foundations.
- Do not treat deleted standalone pre-absorption crates for core, manager, framework, math, or resource responsibilities as valid destinations or validation targets for new work.
- Reserve `server` naming for true network or service-host semantics. Do not introduce or preserve non-network architecture names that use `server`.
- Prefer direct replacement over keeping old and new paths alive at the same time. Do not keep shims, alias crates, or legacy-path re-exports unless the user explicitly requires temporary coexistence.
- If the current task is running inside a directory, crate, or module tree that the convergence rules already mark as the wrong home, stop expanding that area and migrate the work to the compliant destination first. Only continue the task after the move.
- Scope milestone-stage validation per `docs/plans/milestone-validation-policy.md`: package-level batches for the changed crates, expanding to multi-package or workspace validation only when shared crates, workspace manifests, cross-cutting types, or public APIs consumed by multiple crates actually moved. During implementation slices, prefer scoped Rust syntax/type checks unless a blocker requires earlier evidence.
- Prefer hierarchical source trees for both production code and tests. Keep crate roots and `mod.rs` files navigational, and move behavior into focused child modules grouped by subsystem.
- Keep `binding.rs`, `lib.rs`, `main.rs`, and production `mod.rs` files simple, short, and structural. If they start owning parsing, routing, state mutation, orchestration, or multi-domain declarations, push that behavior into child modules immediately.
- Prefer `src/tests/` trees for crate-level and public-surface unit tests. Use module-local `tests.rs` or `tests/mod.rs` only when a source module has private helpers that need direct coverage.
- Add succinct comments to key data structures, invariants, and non-obvious control flow. Avoid comments that restate obvious code.
