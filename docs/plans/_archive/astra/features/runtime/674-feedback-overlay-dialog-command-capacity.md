---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/11a-runtime-ui-architecture-tree-layout-input-accessibility-review.md
  - docs/plans/optimize/zircon_runtime/11c-gpu-ui-renderer-atlas-sdf-batch-clip-submit-review.md
  - docs/plans/optimize/zircon_editor/01-retained-ui-architecture-performance-review.md
related_code:
  - zircon_runtime/src/ui/surface/render/progress.rs
  - zircon_runtime/src/ui/surface/render/feedback.rs
  - zircon_runtime/src/ui/surface/render/drag_overlay.rs
  - zircon_runtime/src/ui/surface/render/dialog.rs
tests:
  - zircon_runtime/src/ui/tests/render_progress.rs
  - zircon_runtime/src/ui/tests/render_feedback.rs
  - zircon_runtime/src/ui/tests/render_drag_overlay.rs
  - zircon_runtime/src/ui/tests/render_dialog.rs
---

# Runtime Feedback And Overlay Command Capacity

## Scope

Six Runtime UI leaf command builders had bounded output shapes but started with
single-item literals that could grow while optional visual branches appended.
They now reserve their known upper bounds after the existing classification and
geometry gates. Painter order, command content, clipping, style lookup, text
measurement, z offsets, and invalid-input exits are unchanged.

| Painter | Maximum commands | Bound |
| --- | ---: | --- |
| LinearProgress | 3 | track, optional fill, optional label |
| Alert | 4 | surface, optional icon, optional message, optional action |
| Tooltip | 4 | surface, optional title, optional body, optional icon |
| Toast | 4 | surface, icon, optional message, optional action |
| DragOverlay | 4 | preview surface, optional icon, optional label, optional indicator |
| Dialog / ConfirmDialog | 6 | surface, optional severity mark, optional title/body, one or two actions |

This batch deliberately excludes row- and tick-driven renderers such as
NotificationCenter and Slider. Their output sizes depend on authored runtime
data, so a small fixed reservation would either understate the bound or retain
unnecessary spare space on the common path.

## Plan Completion List

| Batch | Work | Status | Validation evidence |
| --- | --- | --- | --- |
| Runtime11C / RUI-674 | Reserve fixed command capacity for Progress, feedback, DragOverlay, and Dialog leaf painters | implemented_pending_validation | Focused Rust source regressions bind all six bounds and reservations. Scoped Rustfmt, scoped diff check, the six-invariant source probe, and the combined Runtime/Editor static-contract batch passed. Managed Runtime Cargo and Windows Release allocation/time evidence remain pending. |

## Static Evidence

- The four focused render test modules now prevent removal of the Progress,
  Alert, Tooltip, Toast, DragOverlay, and Dialog capacity reservations.
- Existing renderer behavior tests continue to cover labels, icons, actions,
  dialog severity, drag indicators, clipping, and token resolution. They are
  staged for managed Rust validation rather than claimed as locally run.
- `rustfmt --edition 2021 --check --config skip_children=true` passed for all
  four production modules and four focused test modules.
- Scoped `git diff --check` passed. Git reported only repository line-ending
  notices.
- The direct source probe passed all six capacity/reservation invariants.
- The combined Runtime/Editor static-contract batch passed `95/95` in
  `0.139s`.

## Complexity Boundary

Each selected builder remains `O(1)` in emitted commands and owns no
collection-shaped input. The change removes geometric vector growth from
optional command branches by using a fixed upper bound. It does not claim a
product allocation, latency, RSS, or p50/p95/p99 improvement until the managed
release comparison runs.

## Source Snapshot

- `progress.rs`: `DBC5BFB8E5D15BB3FC8932754905E35C5B4F912B6FCCC2DC2B3C3E6D66BFA1DB`
- `feedback.rs`: `7A855FED5F6290776A42E5ACE07F2A7DB326A074A9AEEA5E41A05F413B9FE71B`
- `drag_overlay.rs`: `754657B1CAB756430B098D0FE2A5D14D5C94C9EC6ED09655436A9BA8978D9472`
- `dialog.rs`: `D838DA0C9FF88748EAC81C141C16F002F37FBFA28799AC0533499B28063BC9C4`
- `render_progress.rs`: `35B52E6A4C252793D60A61FE5576CC3222A698024F69EC9FB965AEED4B92024A`
- `render_feedback.rs`: `5C487BC91F0909200162DF2BE414D7672AF22C6FD2BFABD89BD513A4D1422D89`
- `render_drag_overlay.rs`: `B7C0F54B556BD75DF0DD3070F13A395085CC38A7C580A6C808572A242D5D98B2`
- `render_dialog.rs`: `6B65441A62590A1C1090A52EC75422B3CBC3DC370CE22CC29F0ACB0D95DA1DCA`

## Managed Gate

No direct Cargo command or coordinator status query was run for this slice.
The next immutable multi-task Runtime/Editor validation input must compile the
four focused renderer test modules together with the existing batch and collect
a Windows Release allocation/time comparison. Until that batch succeeds, this
record remains `implemented_pending_validation` and makes no product
performance or p50/p95/p99 claim.
