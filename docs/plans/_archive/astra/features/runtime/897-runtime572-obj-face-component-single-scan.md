---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/02/2026-08-31-runtime572-obj-face-component-single-scan.md
related_records:
  - docs/plans/astra/features/editor/955-editor572-build-export-target-single-buffer.md
  - docs/plans/astra/features/runtime/898-runtime573-feedback-css-color-single-buffer.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_runtime/src/asset/formats/obj/parse_obj_face_vertex.rs
tests:
  - zircon_runtime/src/asset/formats/obj/parse_obj_face_vertex.rs
---

# Runtime897 Runtime572 OBJ Face Component Single Scan

| 范围 | 修复 | 证据与门禁 |
| --- | --- | --- |
| OBJ face-token parsing | Perform one byte traversal, record at most the first three separators, and return borrowed position/UV/normal slices while preserving empty fields and extra-component behavior. | Compatibility contract covers `v`, `v/vt`, `v//vn`, `v/vt/vn`, and legacy extra components. |
| 性能门禁 | 250,000 face-token parses avoid three separate `split('/')` iterator traversals. | ignored marker `RUNTIME572_OBJ_FACE_COMPONENT_SINGLE_SCAN_BENCH_V1` requires at least 5% P95 improvement; managed Runtime Release receipt remains pending. |

- `parse_obj_face_vertex.rs` contains the Runtime572 behavior/source contracts and marker.
- No tooling changes; managed Release evidence remains pending.

### Grouped validation submission (2026-09-25)

Runtime572 is included in the shared broad `57` batch covering the 571–579
Runtime/Editor records: Runtime development PTY `62085`, Editor development PTY
`29906`, Runtime02 Release PTY `6464`, and Editor Release PTY `16118`. All
wrappers remain intentionally unpolled and managed compiler/P95 receipts are pending.
