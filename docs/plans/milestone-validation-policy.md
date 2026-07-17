# Zircon Milestone Validation Policy

This document is the authoritative execution policy for validation under `docs/plans/`. It replaces any local default that asks contributors to run Cargo compilation or broad tests after every implementation slice. It does not weaken acceptance criteria or remove regression tests.

## 0. Foundation Priority

Apply [`minimum-viable-engine-foundation.md`](minimum-viable-engine-foundation.md) before selecting a milestone. Until its F0-F5 gates are accepted, batch validation capacity belongs first to foundation work; deferred advanced rendering, complete text, AI, networking, and plugin-expansion work may run only when it directly unblocks an earlier foundation gate.

## 1. Validation Units

| Unit | Purpose | Required validation |
| --- | --- | --- |
| Implementation slice | Produce one coherent source change. | Formatting, `git diff --check`, structural guards, plus only the focused-test exceptions enumerated in §2.4. No routine Cargo compile. |
| Milestone | Integrate all slices that satisfy one declared milestone. | One batched package compile and the focused regression set for every behavior and boundary changed by the milestone. |
| Execution wave | Close a dependency-complete group of milestones. | Workspace or multi-package regression, product/visual verification where applicable, and CI-equivalent checks before declaring the wave accepted. |

## 2. Slice Discipline

1. Keep implementation slices small, owner-scoped, and free of speculative test churn.
2. During a normal slice, run only checks that do not rebuild the Rust dependency graph: formatter/parser checks for touched files, `git diff --check`, plan-required static scans, and deterministic scripts that inspect source text or assets.
3. Do not run `cargo check`, `cargo test`, or a product capture after each slice merely because the command appears in a plan. Queue those commands for the milestone test stage.
4. Run an immediate focused test only when fixing a recorded failure, changing an ABI/public contract, changing unsafe or persistence behavior, or when the same focused test previously failed. Record the reason in the milestone evidence.

## 3. Milestone Batch Gate

At a milestone boundary, the owner prepares one validation manifest listing touched packages, test filters, feature profiles, interface boundaries, and required product evidence. The validation lane then runs the smallest complete batch in this order:

1. One package-level `cargo check` covering all changed targets and declared feature profiles.
2. One focused `cargo test` batch covering the milestone's changed behavior, contract tests, and known regressions.
3. One interface/package boundary batch when shared DTOs, ABI, manifests, or generated contracts changed.
4. Product, screenshot, RenderDoc, device, or editor-host verification only for milestones that change those observable outputs.

Failures are diagnosed from the lowest shared layer first. Re-run only the affected focused batch after repair; do not restart the full milestone suite until the focused regression is green.

## 4. Wave and Release Gates

1. Run workspace-wide builds/tests only when a dependency-complete execution wave closes, before a release candidate, or when a shared root manifest/lockfile/toolchain change requires it.
2. Keep all regression and contract tests. Optimization comes from scheduling and batching them, not from deleting assertions or weakening expected behavior.
3. A wave may be accepted only when its milestone evidence identifies the exact commands, profiles, failures repaired, and any deferred external validation.

## 5. Parallel Development and Build Lanes

1. Development lanes may implement independent slices in parallel, but Cargo-heavy validation is owned by a dedicated validation lane.
2. Coalesce compatible milestones by package and feature profile before compiling. Do not start competing full builds for the same crate graph.
3. Use stable, lane-owned target directories for concurrent package families. A lane must not delete, overwrite, or repurpose another lane's active target directory.
4. Cross-package hard cuts reserve an integration window. Callers, exports, tests, docs, and product checks move together, then receive one combined milestone batch.

## 6. Evidence Format

Each accepted milestone records: changed scope, validation manifest, commands actually run, result summary, repaired failures, deferred external checks, and the next dependency it unlocks. A record must not claim a broad pass from a narrow test run.
