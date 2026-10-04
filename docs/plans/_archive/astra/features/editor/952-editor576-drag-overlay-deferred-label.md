---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/576/2026-08-31-drag-overlay-deferred-label.md
related_records:
  - docs/plans/astra/features/runtime/895-runtime576-mesh-sdf-seed-preflight.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_drag_overlay/text.rs
tests:
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_drag_overlay/text.rs
---

# Editor952 Editor576 Drag-Overlay Deferred Label

| 范围 | 修复 | 证据与门禁 |
| --- | --- | --- |
| Drag-overlay label paint | Return a trimmed borrowed label and create an owned command payload only after the zero-area geometry gate; collapsed previews preserve priority, trimming, and visible output without the allocation. | Behavior/source contracts cover label priority, trimming, and allocation placement. |
| 性能门禁 | 100,000 collapsed-preview attempts reduce label allocations to zero per sample. | ignored marker `EDITOR576_DEFERRED_DRAG_LABEL_BENCH_V1` requires optimized P95 ≤90% of legacy; managed Editor Release receipt remains pending. |

- `text.rs` contains the Editor576 behavior contract and marker.
- No tooling changes; the combined Runtime576/Editor576 release gate remains pending.

### Grouped validation submission (2026-09-25)

Editor576 is included in the exact `optimization_batch_gu` replacement wave:
Runtime development PTY `29091`, Editor development PTY `43801`, Runtime02
Release PTY `94742`, and Editor Release PTY `9082`. The wrappers remain
intentionally unpolled; compiler, functional-test, and P95 receipts are pending.
