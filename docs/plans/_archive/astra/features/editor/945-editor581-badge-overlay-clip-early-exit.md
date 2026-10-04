---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/581/2026-08-31-badge-overlay-clip-early-exit.md
related_records:
  - docs/plans/astra/features/editor/946-editor581-asset-projection-full-reuse.md
  - docs/plans/astra/features/runtime/888-runtime581-visible-spatial-hash-index.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/material_primitives/badge/commands/overlay.rs
tests:
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/material_primitives/badge/commands/overlay.rs
---

# Editor945 Editor581 Badge Overlay Clip Early Exit

| 范围 | 修复 | 证据与门禁 |
| --- | --- | --- |
| Badge overlay paint | Fully clipped valid rectangles return before surface/text command construction; visible geometry, dot/text styling, ordering, and opacity remain unchanged. | Behavior contract requires an offscreen badge to emit no commands; the ignored marker is colocated with the production test. |
| 性能门禁 | 8,192 offscreen badge projections avoid backend command and text allocation work. | ignored marker `EDITOR581_BADGE_OVERLAY_CLIP_EARLY_EXIT_BENCH_V1` requires optimized P95 ≤50% of the legacy path; managed Editor Release receipt remains pending. |

- `overlay.rs` contains the Editor581 behavior contract and release marker.
- No tooling changes; the source plan's combined Runtime581/Editor581 handoff remains the managed gate.

### Grouped validation submission (2026-09-25)

Editor581 is included in the shared `optimization_batch_gz` wave: Editor
development PTY `36565` and Editor Release PTY `35753`, paired with Runtime
development PTY `76539` and Runtime02 Release PTY `51493`. The wrappers remain
intentionally unpolled; compiler, functional-test, and P95 receipts are pending.

The exact `optimization_batch_gz` replacement wave was submitted as Runtime
development PTY `81972`, Editor development PTY `21257`, Runtime02 Release PTY
`20043`, and Editor Release PTY `50976`. The earlier broad-prefix wave did not
select this historical marker; all four replacement wrappers remain unpolled.
