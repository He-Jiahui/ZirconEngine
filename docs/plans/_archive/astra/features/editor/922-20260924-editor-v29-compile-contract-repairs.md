---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/207/2026-08-26-shared-vec-grid-storage.md
  - docs/plans/optimize/zircon_editor/12/2026-08-24-settings-delta-and-reload-hotpaths.md
  - docs/plans/optimize/zircon_editor/07/2026-09-13-pending-edit-page-single-scan.md
  - docs/plans/optimize/zircon_editor/06/2026-08-26-unstable-viewport-overlay-capability-sort.md
related_records:
  - docs/plans/astra/features/editor/664-20260911-editor-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
---

# Editor922 v29 Compile Contract Repairs

## 计划完成列表

| Scope | Completed change | Evidence and remaining gate |
| --- | --- | --- |
| Settings, asset sync and retained geometry | Match `resolve`'s typed result, expose the project sync function to its sibling manager tests, and re-export the existing geometry projection for its retained-host tests. | Five v29 diagnostics on four clean files targeted; grouped library validation pending. |
| Recovery, journal, menus and event tests | Import the actual ledger type, keep the recovery coordinator re-export test-private, scope the journal benchmark constant to its measuring helper, fix nested menu imports and use the existing host plugin-activation method. | Existing lifecycle and source-capacity assertions remain in place; v29 compiler errors targeted. |
| Typed UI, world and gateway fixtures | Carry the play instance in the correct identity field; compare typed activity slots and view IDs; use structured gateway failures, world reads and public test-lock entry points. | v29 type/privacy errors targeted; no claim of runtime acceptance. |
| Detached entity fixture | Construct an exact-ID conflict by detaching a matching node from a second default world and restoring it through the existing public batch API; spawn the 100k unrelated entities through the public `NodeKind::Empty` entry. | Preserves duplicate-ID rejection and large unrelated population without widening Runtime's crate-private spawn API. |
| Localized plugin menu | Give the weather contribution a typed stable path, bound English translations, explicit test keymap bindings and typed capability contexts. | Retains text, menu priority, disabled state and shortcut checks while asserting the capability-gated item is absent until authorized; four v29 stale signature/type errors targeted. |
| Overlay provider state | Add test-only read-only state inspection to session/controller owners; retain checks that both providers enable and the surviving one disables after revocation. | The old boolean command result is now `ViewportFeedback`; explicit state assertions remain, not just a successful-call check. |
| Paired performance markers | Repair mutable measurement closures and borrowed report entries without changing sample counts, pairing order, or thresholds. | Rust source formatting and grouped Release timing still required. |
| Shared grid allocation regression | Adapt the test's `Arc<[T]>` slice view solely to compile, preserving pointer identity, `Arc<Vec<T>>` source guard and p95 `<=70%` limit. | Current foreign `generation.rs` converts `Vec` into `Arc<[T]>` and copies its allocation, contradicting the Editor207 zero-copy plan; expect this test to remain RED until owner-approved storage convergence. **No performance acceptance claimed.** |

The existing `tools/tests/test_editor07_sample_grid_generation_contract.py`
also requires the older `Arc<[SampleGridTick]>` and `Arc<[SampleGridPoint]>`
source text. It passes while current storage is a shared slice but directly
conflicts with Editor207's newer allocation-preserving `Arc<Vec<T>>` contract.
Both tests cannot be true of the same production fields. Do not fake the
source shape to satisfy this outdated test; any approved hard cutover must
reconcile the Editor07 expectation and Editor207's real pointer/p95 gate.

This is a partial repair of the v29 compiler snapshot, not a green Editor
library test or performance gate. Further shared-owner migration errors may
surface on the next grouped check; do not dilute assertions to pass it.

Local Rustfmt `--check` passed for the repaired Editor files, and all six
plan/related links resolve. Two preexisting running Cargo jobs blocked the
one-time grouped validation admission; source-level checks are not Rust tests.
The combined Runtime206/notification/V2-file/Editor asset-index source-contract
run passed `19/19` in one process. It is not Editor207's pointer identity or
ignored Release p95 test; that gate remains explicitly open.
The one-process repository-wide Python performance-source-contract discovery
also passed `2950/2950` (`40.453s`), including unrelated tooling modules; it
neither compiles Editor Rust nor validates the Editor207 pointer/p95 contract.
Eight targeted Runtime/Editor source-contract modules then passed `32/32`
in a single process. This includes the conflicting Editor07 sample-grid
contract above, so its green result is evidence of the old shape, **not**
Editor207 performance acceptance.

The wider grouped Editor `test_editor*contract.py` source-contract discovery
ran `1927` tests and found one unrelated layout failure; the other `1926`
passed. `test_editor_zui_workbench_layout_contract.py` requires two sample-card
columns to fit the 420px ultra window after its 34px activity rail: available
width `386px`, but the current `component_icon_buttons` minimum of `232px`
requires `2 * 232 + 8 = 472px`. The same foreign worktree change increased its
segmented child's fixed width to `216px` and placed this showcase behind a
hidden retired-content owner. Simply lowering the card minimum would clip
its interactive children; neither the ZUI asset nor the old source assertion
was changed here. The layout owner must reconcile the retired showcase and
the still-enforced ultra two-column contract before this Editor suite can be
accepted. ReactBits [Cards](https://pro.reactbits.dev/docs/app-ui/card) is the
gallery reference and [App Dialog](https://pro.reactbits.dev/docs/app-ui/app-dialog)
is the scroll-owner reference; only the public category descriptions were
available, so no variant's hidden responsive behavior is asserted. This
failure is separate from Editor207's shared-storage/p95 blocker.

During the later grouped v30 managed Runtime/Editor library-validation handoff,
the combined Runtime/Editor Python pressure-contract batch passed `264/264`.
Its separate 81-module functional/structure batch passed `231/232`, with the
single failure in Runtime's navigation tooling inventory (recorded in
Runtime881); it does not close the Editor layout failure above. Both are
source/model checks, not v30 compiler, Rust test or Release evidence.

### Layout-contract repair after the grouped static failure (2026-09-25)

The failing workbench layout contract is repaired at its owner instead of weakening
the assertion. `component_icon_buttons` now restores the shrinkable
`min=188/preferred=max=210` card envelope and its segmented child returns to the
`150px` compact width, so two cards plus the `8px` gap fit the `386px` ultra
surface after the `34px` activity rail. The unrelated retired-content and fixed
height changes remain untouched. The focused layout and quiet-action contracts
now pass `25/25`; the refreshed grouped `test_editor*.py` discovery then passes
`2099/2099` in `33.433s`. Grouped Editor Cargo and Release evidence remains
pending.
