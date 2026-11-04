# Zircon Milestone Validation Policy

This document is the authoritative execution policy for active milestone validation. It replaces any local default that asks contributors to run Cargo compilation or broad tests after every implementation slice. It does not weaken acceptance criteria or remove regression tests.

The local coordinator and its managed matrix, registration, lease, ticket and commit workflows are retired. Older plan commands that require them are historical. Use the current [validation guidance](../../.codex/skills/zircon-dev/validation/guide.md) for independent command evidence and retain every applicable acceptance gate. Jenkins migration acceptance remains pending; do not restore the old service or infer acceptance from a dry run. See [retirement](../tooling/coordinator-retirement.md).

## 0. Foundation Priority

Apply [`minimum-viable-engine-foundation.md`](minimum-viable-engine-foundation.md) before selecting a milestone. Until its F0-F5 gates are accepted, batch validation capacity belongs first to foundation work; deferred advanced rendering, complete text, AI, networking, and plugin-expansion work may run only when it directly unblocks an earlier foundation gate.

## 1. Validation Units

| Unit | Purpose | Required validation |
| --- | --- | --- |
| Implementation slice | Produce one coherent source change. | Formatting, `git diff --check`, structural guards, plus only the focused-test exceptions enumerated in §2.4. No routine Cargo compile. |
| Milestone | Integrate all slices that satisfy one declared milestone. | One batched package compile and the focused regression set for every behavior and boundary changed by the milestone. |
| Execution wave | Close a dependency-complete group of milestones. | Affected-module regressions, product/visual verification where applicable, and declared CI-equivalent checks before declaring the wave accepted. Task/document-required workspace or package-set gates are allowed under §4. |

## 2. Slice Discipline

1. Keep implementation slices small, owner-scoped, and free of speculative test churn.
2. During a normal slice, run only checks that do not rebuild the Rust dependency graph: formatter/parser checks for touched files, `git diff --check`, plan-required static scans, and deterministic scripts that inspect source text or assets.
3. Do not run `cargo check`, `cargo test`, or a product capture after each slice merely because the command appears in a plan. Queue those commands for the milestone test stage.
4. Run an immediate focused test only when fixing a recorded failure, changing an ABI/public contract, changing unsafe or persistence behavior, or when the same focused test previously failed. Record the reason in the milestone evidence.

## 3. Milestone Batch Gate

At a milestone boundary, the owner prepares one validation manifest listing touched packages, test filters, feature profiles, interface boundaries, and required product evidence.

Before every compilation-capable command (`check`, `build`, `test`, including
`--no-run`), explicitly announce its engine modules, Cargo packages, features,
targets/test filters, action/profile, exact command and coverage reason to the
user using [the build declaration](../../.codex/skills/zircon-dev/validation/guide.md#declare-every-build).
For ordinary work, disable default features and name only needed features. Preserve the declared
scope in the Jenkins request and actual command evidence; declare changes before
execution. Required transitive dependencies remain part of the scoped build.

The validation lane then runs the smallest complete batch in this order:

1. One package-level `cargo check` for the affected declared targets and minimal feature selections.
2. One focused `cargo test` batch covering the milestone's changed behavior, contract tests, and known regressions.
3. One interface/package boundary batch when shared DTOs, ABI, manifests, or generated contracts changed.
4. Product, screenshot, RenderDoc, device, or editor-host verification only for milestones that change those observable outputs.

Failures are diagnosed from the lowest shared layer first. Re-run only the affected focused batch after repair; do not restart the full milestone suite until the focused regression is green.

## 4. Wave and Release Gates

1. Explicit `--workspace` or package-set builds called for by the current task or owning validation documentation are allowed under the declared-gate rule below. Blind `--all`, `--all-features`, `--all-targets`, implicit root-wide selection and undeclared all-package loops/matrices remain prohibited unless the user explicitly authorizes that scope. Ordinary wave/release or root manifest/lockfile/toolchain work still uses the minimal affected modules/features/targets; those changes alone do not justify broad coverage. Inspect unclear dependencies and keep uncovered acceptance pending rather than claim a broad pass from narrower evidence.
2. Keep all regression and contract tests. Optimization comes from scheduling and batching them, not from deleting assertions or weakening expected behavior.
3. A wave may be accepted only when its milestone evidence identifies the exact commands, profiles, failures repaired, and any deferred external validation.

### Declared workspace and package-set gates

The current task or owning validation documentation may require a complete
workspace or explicit package set. Such a gate is allowed: declare its manifest,
workspace members/package names, default and additional features, target/test
selection, action/profile, exact command and coverage reason before execution.
Use the documented defaults when they are part of that gate; do not narrow the
required coverage merely to satisfy the ordinary minimal-scope default.

This scope permission does not grant local execution outside Jenkins ownership,
remote Git actions, service startup, alternate output roots or destructive
cleanup. Those actions retain their own authorization. Physical storage, native
locks, source/process preservation and actual terminal evidence still apply.
Ordinary implementation slices continue to use the smallest affected scope.

### Authorized Windows/MSVC U09 regression

John explicitly authorized this policy exception on 2026-11-03 for the complete
local Windows/MSVC U09 regression in `E:\Git\ZirconEngine`, branch
`fix/editor-orbrun-u09-folders-20261027`. Execute every gate in
[Editor CI Windows reproduction](../editor/ci-validation.md#windows-reproduction),
including `cargo +1.94.1 check --manifest-path zircon_plugins/Cargo.toml --workspace --locked --offline`.
The workspace check covers every member and its default check targets and
features; do not replace it with selected packages or disable its default features.
Use the documented feature settings for the Editor host check and all three
test batches. Keep `-RequireMatchingTests` for each test batch and append
`--offline` to the documented `--locked` commands. This exception does not grant
`--all-features`, `--all-targets`, unrelated root-workspace builds or test matrices.

This authorization also permits direct local validation, compatible artifact
reuse, task-owned evidence generation and bounded task-owned temporary cleanup;
do not start Jenkins/Penpot or claim their acceptance. Verify physical D: Cargo,
rustup, target, build and temporary roots before execution; the C: output ban,
35 GiB reserve, native Cargo locks and process/source preservation remain in force.
Reuse the prior local `E:\Git\zr_vm` object stores at locked revision
`3c1352ecb6ddfab02752d46682cc376558d5f481` with child-only Git URL rewrites;
do not depend on a GitHub fetch or mutate the dirty VM worktree.

Declare each actual command, package/feature/target scope and profile before
execution. Bind logs and terminal exit receipts to the current source hashes.
Report full-gate PASS only if every required step actually passes; otherwise
report FAIL and retain the failing evidence. Record policy wording, results and
remaining blockers in `D:\CodexData\ze-drive\msvc-regression-u09-workspace-20261103.md`
and details under `reports/msvc-regression-u09-workspace-20261103/`.
Ordinary slices and other wave/release work retain the minimal-scope default.

## 5. Parallel Development and Build Lanes

1. Development lanes may implement independent slices in parallel, but Cargo-heavy validation is owned by a dedicated validation lane.
2. Coalesce compatible milestones by package and feature profile before compiling. Reuse compatible managed outputs and preserve exclusive writer/Cargo locks; do not start duplicate builds for the same scoped crate graph.
3. Use stable, lane-owned target directories for concurrent package families. A lane must not delete, overwrite, or repurpose another lane's active target directory.
4. Cross-package hard cuts reserve an integration window. Callers, exports, tests, docs, and product checks move together, then receive one combined milestone batch.

## 6. Evidence Format

Each accepted milestone records: changed scope, validation manifest, commands actually run, result summary, repaired failures, deferred external checks, and the next dependency it unlocks. A record must not claim a broad pass from a narrow test run.

## Benchmark interpretation

A microbenchmark isolates a helper or workload; it does not establish a product result when it bypasses the real caller. Keep the product root bound (MAX_PROJECT_ASSET_ROOTS), scope, raw samples and p50/p95/p99 explicit. Product acceptance needs the actual bounded caller and its environment. The missing live Astra plan is not replaced by historical measurements; its archived correction is provenance only.

### Clause provenance

These content identities preserve the source of this transfer. They are not fresh measurements, active-file dependencies or acceptance receipts.

- `01-bounded-hotpaths.md`, SHA-256 `db11719946998a3abd38c6aef7aae5107e6d16a60934ffc18ddaf077da13a1dc`, selected source lines 133-133.
