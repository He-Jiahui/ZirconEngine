---
related_code:
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings
  - zircon_runtime/src/tests/runtime_absorption/structure_convention
plan_sources:
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
  - docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md
output_records:
  - docs/plans/zircon_runtime/runtime/02/2026-07-11-runtime02-current-cargo-baseline.md
  - docs/plans/zircon_runtime/runtime/15/2026-07-11-stable-evidence-owner-hard-cutover.md
status: review_298_passed_structure_stable_evidence_cutover_1297_1304_external_7
---

# Priority Structure and Review Managed Baseline

Date: 2026-07-11

Status: `review_298_passed_structure_stable_evidence_cutover_1297_1304_external_7`

## Fresh managed-lane results

| Filter | Result | Interpretation |
|---|---|---|
| `structure_convention` managed snapshot | 1303 passed / 0 failed | The managed binary was green before later concurrent UI/Text edits. |
| `structure_convention` current package rerun | 1299 passed / 4 failed | Remaining failures are active Render/UI owners: UI render production/test budgets, froxel integrate test budget, and a deferred-lighting shader dispatch anchor. |
| Latest recompiled standalone structure rerun | 1224 passed / 79 failed | Not an acceptance replacement: three concurrently removed Render/Text session-note inputs account for 71 failures, while current Render/UI source and file-budget drift accounts for eight. This run is retained as current-source external-input/owner-drift evidence. |
| Stable-evidence hard-cut current-source rerun | 1297 passed / 7 failed | Removed all 456 `.codex/sessions/` path consumptions from 449 guard files; the initial retired-note subset was 75 reads and 81 tuple consumers across 72 owners. The recursive family guard passes, current artifact-cache paths are synchronized, and no new failure was introduced. |
| Runtime-owned asset pipeline guard follow-up | 1 passed / 0 failed | Updated the fixed 11-test snapshot to the current 12 child tests and anchored the new second-manifest-root watcher test; no production behavior or budget limit changed. |
| `code_review_findings` managed snapshot | 264 passed / 34 failed | The fresh package binary exposed stale parent-overview status ownership. |
| `code_review_findings` current package rerun | 298 passed / 0 failed | Review guards and their nested structure coverage pass in the newly compiled default-feature binary. |
| `code_review_findings` latest standalone source rerun | 80 passed / 0 failed | Direct review-findings owner guards remain green against the current worktree after the Runtime asset-pipeline guard synchronization. |
| `runtime_absorption` | 1555 passed / 76 failed | Aggregate failures include the 34 review cases plus folder-backed/archive-routing and cross-plan status mirrors; production behavior is not inferred green from this result. |
| `plan_status` current standalone rerun | 48 passed / 0 failed | Runtime plan/status/archive routing remains green without changing any `in_progress` lifecycle status. |
| Full runtime-interface structure audit follow-up | monolithic JSON timed out at 124 s; staged run completed in 128.5 s | All 24 Runtime child-boundary audits report `risks = []`. The naming boundary is still `blocked` by four UI test-fixture strings classified as unowned runtime `editor` naming and one graphics test name classified as legacy debt. |

## Review failure groups

The original 34 review failures covered F1/F2/F4/F8/F11/F12/F15/F16/F18/F19,
typed-error convergence, and plugin DX D1/D6/D8/D9/D12/D13. The guards now
separate overview catalog assertions from numbered-record evidence assertions.
They also follow the current FontDocument typed error, `.zshader` v2 migration
wording, `.zui` component import path, and lifecycle fixture builder count.

The worktree contains active, uncommitted Runtime 15 convergence work and new
records named `2026-07-11-priority-review-overview-record-ownership-cleanup.md`
and `2026-07-11-structure-convention-current-source-budget-convergence.md`.
This validation slice therefore records the failures and does not overwrite
that owner’s source-routing changes.

## Decision

The priority review-findings package filter is accepted at 298/298 in a newly
compiled default-feature binary. The same current package binary reports
structure convention 1299/1303. The four remaining failures are active
Render/UI owners: `scene_renderer/ui/render.rs` at 822 lines,
`scene_renderer/ui/render/tests.rs` at 823 lines, froxel integrate tests at 826
lines, and one deferred-lighting built-in dispatch anchor. No parent plan is
repopulated with migrated historical status rows merely to satisfy stale guards.
The latest standalone result is not used to revise that package baseline. Of
its 79 failures, 71 are direct missing-input failures from three now-missing
external session records:
`20260617-0926-render-hzb-progress.md`,
`20260628-0100-runtime-text-implementation.md`, and
`20260628-0141-render-plan08-continuation.md`. The other eight are current
Render/UI owner drift, including two production files above 800 lines, the
render-graph workload owner at 690/680, related test-budget/owner guards, and
the deferred-lighting dispatch anchor. The previously reported
ProjectAssetManager watcher-lock guard passes after recompiling the standalone
harness from current source. No missing session note is recreated as a
compatibility artifact and no budget is raised to make the gate pass.

After that classification, the Runtime-owned asset pipeline manager child-count
guard passes 1/1. A subsequent direct hard cut removed the retired session-note
dependencies, so the current standalone backlog is seven Render/UI owner,
budget, workload, or shader-anchor failures. The two concurrently renamed
artifact-cache asset guards also pass focused 1/1 after current-path and durable
status evidence synchronization.

The monolithic full runtime-interface structure-audit retry did not complete
within 124 seconds and left its redirected JSON output empty. A staged run of
the same audit functions then completed in 128.5 seconds. All 24 child
boundaries report zero explicit risks, but the aggregate naming gate remains
`blocked`: four references are UI test-fixture text in
`ui/surface/render/text_prewarm.rs` and `ui/text/geometry.rs`, and one is the
graphics test name `keeps_legacy_character_advances_when_counts_match`. These
five locations are outside this Runtime 02 session's registered write scope
and are retained as active UI/graphics owner debt rather than rewritten here.

## Stable evidence ownership follow-up

Fresh focused reproductions confirmed that the compiled-graph-cache,
prepared-mesh-queue, and UI-text-layout guards each failed before their real
assertions because one of the three removed `.codex/sessions` inputs could not
be read. The three removed paths occurred in 72 guard files and explained 71
failures; every affected file also read at least one canonical `docs/plans`
source.

This evidence-owner hard cut is now applied to the whole family. All 456
`.codex/sessions/` path consumptions are removed from 449 guard files; the
initial retired-note subset comprised 75 reads and 81 tuple consumers across 72
owners. Existing numbered plan/archive/module/status-row assertions remain.
The recursive guard rejects the entire session-note path family and passes
1/1; representative Render 01, Render 08, and Text 03 guards that previously
failed with `os error 2` each pass 1/1. The full rerun remains 1297/1304, with
the same seven external Render/UI failures and no new regression. Recreating
the notes or mapping missing paths through a fallback remains explicitly
rejected.
