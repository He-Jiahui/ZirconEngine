---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/02/2026-08-31-editor571-sprite-atlas-dense-placement-index.md
related_records:
  - docs/plans/astra/features/editor/955-editor572-build-export-target-single-buffer.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_editor/src/ui/host/editor_asset_manager/manager/sprite_atlas/packer.rs
tests:
  - zircon_editor/src/ui/host/editor_asset_manager/manager/sprite_atlas/packer.rs
---

# Editor954 Editor571 Sprite-Atlas Dense Placement Index

| 范围 | 修复 | 证据与门禁 |
| --- | --- | --- |
| Sprite-atlas placement lookup | Dense source IDs now project packed locations into a `Vec<PackedSourceLocation>` in source-index order, replacing a private `BTreeMap` lookup while preserving packing, pixels, UVs, padding, order, and failures. | Behavior/source contracts cover deterministic placement, row copies, UVs, padding, and failed packing. |
| 性能门禁 | 4,096 dense locations and 128 lookup rounds change private placement probes from ordered-tree lookup to O(1) indexing. | ignored marker `EDITOR571_DENSE_PLACEMENT_LOOKUP_BENCH_V1` requires at least 75% P95 improvement; managed Editor Release receipt remains pending. |

- `packer.rs` contains the Editor571 behavior/source contracts and marker.
- No tooling changes; this record preserves the managed Release gate without claiming the standalone calibration as acceptance.

### Grouped validation submission (2026-09-25)

Editor571 is included in the shared broad `57` batch covering the 571–579
Runtime/Editor records: Runtime development PTY `62085`, Editor development PTY
`29906`, Runtime02 Release PTY `6464`, and Editor Release PTY `16118`. All
wrappers remain intentionally unpolled and managed compiler/P95 receipts are pending.
