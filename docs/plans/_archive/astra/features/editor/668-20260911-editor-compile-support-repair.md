---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/01-retained-ui-architecture-performance-review.md
  - docs/plans/optimize/zircon_editor/633/2026-09-01-bitset-export-stage-planning.md
  - docs/plans/optimize/zircon_editor/633/2026-09-01-preallocated-widget-dependency-closure.md
related_records:
  - docs/plans/astra/features/editor/664-20260911-editor-ui-optimization-batch-completion.md
  - docs/plans/astra/features/editor/667-editor633-capacity-and-membership-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
---

# Editor Compile-Support Repair Completion List

This batch closes the smallest current-source Editor compile blockers needed to
admit the existing retained-UI optimization rows to one managed validation
request. It does not broaden into tooling or alter the existing UI ownership
design.

## Plan completion list

| Area | Repair | Status |
| --- | --- | --- |
| Preview redraw / host health | Restore the explicit `PaneSurfaceHostContext` import and make the health constructor non-`const` because it calls non-const `u32` bounds helpers. | implemented_pending_validation |
| Settings / notifications | Add `Display` for `SettingsKey` through its canonical string view and retain the notification id before moving the notification payload. | implemented_pending_validation |
| Runtime ownership / scheduler | Resolve play-controller imports from the canonical `crate::core::play` owner and make `requires_resync` an ordinary function. | implemented_pending_validation |
| Pointer routing | Give borrowed pane pointer routes an explicit `'static` lifetime at the existing owned boundary in asset-reference and asset-tree producers. | implemented_pending_validation |
| Native registration / geometry | Correct the nested `EditorManager` import and retain the existing finite side-width budget repair without rewriting unrelated dirty work. | implemented_pending_validation |

## Batched local evidence

- The combined focused Runtime/Editor contract batch passed `53/53` in
  `0.165s`, including the Editor play, settings, viewport, and pointer-route
  contracts.
- Rustfmt parse-only validation passed for the ten touched Editor Rust files;
  scoped whitespace checks passed. No Cargo claim is made.
- Existing Editor optimization rows 661–667 remain listed as
  `implemented_pending_validation`; this record supplies their compile-support
  prerequisite, not a product-performance result.

## Managed acceptance gate

The Editor compile/test and Release allocation/latency evidence must be run in
the same owner-attributed Windows batch as the Runtime rows. The coordinator
has not yet admitted that batch because of the external `E:\Git\zr_vm` dirty
worktree and the pre-existing locked-lockfile drift. Tooling remains deferred
per the task request.
