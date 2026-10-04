---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/576/2026-08-31-mesh-sdf-seed-validation-preflight.md
related_records:
  - docs/plans/astra/features/editor/952-editor576-drag-overlay-deferred-label.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_runtime/src/graphics/scene/resources/prepared/prepared_mesh_sdf.rs
tests:
  - zircon_runtime/src/graphics/scene/resources/prepared/prepared_mesh_sdf.rs
---

# Runtime895 Runtime576 Mesh-SDF Seed Validation Preflight

| 范围 | 修复 | 证据与门禁 |
| --- | --- | --- |
| Mesh-SDF seed preparation | Validate every primitive before cloning payloads into the immutable seed; late-invalid input performs zero payload clones while preserving counts, indices, errors, and ready-seed order. | Behavior/source contracts cover late-invalid input and validation-before-clone shape. |
| 性能门禁 | 100 preparations of 16 primitives avoid 15 deep copies of an approximately 8 KiB voxel buffer per preparation. | ignored marker `RUNTIME576_MESH_SDF_SEED_PREFLIGHT_BENCH_V1` requires optimized P95 ≤90% of legacy; managed Runtime Release receipt remains pending. |

- `prepared_mesh_sdf.rs` contains the Runtime576 behavior contract and marker.
- No tooling changes; the combined Runtime576/Editor576 release gate remains pending.

### Grouped validation submission (2026-09-25)

Runtime576 is included in the exact `optimization_batch_gu` replacement wave:
Runtime development PTY `29091`, Editor development PTY `43801`, Runtime02
Release PTY `94742`, and Editor Release PTY `9082`. The wrappers remain
intentionally unpolled; compiler, functional-test, and P95 receipts are pending.
