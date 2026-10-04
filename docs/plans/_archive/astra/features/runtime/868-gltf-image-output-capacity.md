---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/85/2026-09-21-gltf-image-output-capacity.md
related_records:
  - docs/plans/astra/features/runtime/867-shader-dependency-direct-append.md
  - docs/plans/astra/features/runtime/680-20260911-runtime-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_runtime/src/asset/importer/ingest/gltf_decode/images.rs
tests:
  - zircon_runtime/src/asset/importer/ingest/gltf_decode/images/optimization_batch_runtime868_gltf_image_output_capacity_tests.rs
  - tools/tests/test_runtime868_gltf_image_output_capacity_performance_contract.py
  - tools/tests/test_runtime_gltf_snapshot_contract.py
---

# Runtime868 glTF Image Output Capacity

## 计划完成列表

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Runtime85 glTF image publication | Reuse the validated document image count for output capacity and append decoded images in document order, preserving source/range checks, budget accounting, and first-error behavior. | Intentional RED `2/5` → GREEN `5/5`; the adjacent snapshot contract was repaired for the cached count and the combined Runtime866–868/glTF batch passes `19/19`. The 4,096-image model changes growth `11→0`; ignored `RUNTIME868_GLTF_IMAGE_OUTPUT_CAPACITY_BENCH_V1` is wired. | implemented_pending_validation |

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/asset/importer/ingest/gltf_decode/images.rs` | `80412B2EE2DE9E91D8B50192057A5EAC6DB0B58EC5407867B8CBB6D6AE9A515C` |
| `zircon_runtime/src/asset/importer/ingest/gltf_decode/images/optimization_batch_runtime868_gltf_image_output_capacity_tests.rs` | `6D14001BA748F88DAD48A43E0FD15987B1CCD0422548300D7B5FC32B427494E2` |
| `tools/tests/test_runtime868_gltf_image_output_capacity_performance_contract.py` | `D0C8E93467716C4C5E6F6ACEC1615948B5FC7000A91F495BD76BED37A901DD74` |
| `tools/tests/test_runtime_gltf_snapshot_contract.py` | `EFB305B4C68D0B4942083ECBC3F717D883E1A054075D762FDEBB7F07662C6F9D` |

## Managed gate

Keep Runtime868 pending until the combined Runtime866–868 Windows lane supplies
Runtime compilation, lower/ignored Release execution, allocator evidence, and
glTF-import product p50/p95/p99 percentiles.
