---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/575/2026-08-31-model-projection-metadata.md
related_records:
  - docs/plans/astra/features/runtime/896-runtime575-watch-error-path-move.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/model_projection.rs
tests:
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/model_projection.rs
---

# Editor953 Editor575 Model Projection Metadata

| 范围 | 修复 | 证据与门禁 |
| --- | --- | --- |
| Retained model projection | Use `ModelRc::map_preserving_metadata` directly, preserving retained generation metadata and removing the temporary generic `Rc<VecModel<_>>` wrapper while keeping borrowed rows and one contiguous result allocation. | Behavior/source contracts verify mapped values and pointer-identical metadata sharing. |
| 性能门禁 | 50,000 eight-row projections reduce wrapper allocations from two to one per projection. | ignored marker `EDITOR575_MODEL_PROJECTION_METADATA_BENCH_V1` requires optimized P95 ≤90% of legacy; managed Editor Release receipt remains pending. |

- `model_projection.rs` contains the Editor575 behavior contract and marker.
- No tooling changes; the combined Runtime575/Editor575 release gate remains pending.

### Grouped validation submission (2026-09-25)

Editor575 is included in the exact `optimization_batch_gt` replacement wave:
Runtime development PTY `91895`, Editor development PTY `38484`, Runtime02
Release PTY `63820`, and Editor Release PTY `41285`. The wrappers remain
intentionally unpolled; compiler, functional-test, and P95 receipts are pending.
