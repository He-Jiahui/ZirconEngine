---
title: Runtime glTF Image Output Capacity
category: zircon_runtime
report_id: Runtime868-gltf-image-output-capacity-2026-09-21
date: 2026-09-21
session_id: root-runtime-editor-async-optimization-20260921
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Runtime868 glTF Image Output Capacity

## Finding

The glTF buffer decoder already reserves its validated document buffer count,
but image decoding published a fallible iterator through `Result<Vec<_>, _>`.
That path did not explicitly retain the already-validated image count as the
owned output capacity, so a dense successful document could geometrically grow
the image vector after every decoded image had already paid its decode cost.

## Optimization

- Read the document image count once and reuse it for both the existing
  cumulative-count admission and output capacity.
- Decode images in document order and push each successful result directly
  into the preallocated output.
- Preserve embedded-buffer range validation, external snapshot reads, data-URI
  accounting, image-format detection, budget charge order, and first-error
  short-circuit behavior.

The empty document still returns an empty zero-capacity vector.

## TDD, test repair, and deterministic evidence

The Runtime868 source/model contract was observed RED at `2/5` and GREEN at
`5/5`. The adjacent glTF snapshot contract initially failed because it matched
the retired repeated `document.images().len()` expression; it now locks both
the cached count source and the unchanged count-limit comparison. The combined
Runtime866–868 and glTF snapshot batch passes `19/19`.

For a 4,096-image output model, zero-capacity growth changes from 11 events to
zero. The ignored 101-pair Release marker
`RUNTIME868_GLTF_IMAGE_OUTPUT_CAPACITY_BENCH_V1` emits alternating p50/p95/p99
samples, asserts the deterministic growth counts, and requires preallocated
p95 to remain within 10% of the fallible collect model. Real image decode and
allocator evidence remain managed gates.

## Local validation boundary

- Exact-file Rustfmt and scoped `git diff --check` pass.
- Runtime866–868 plus the adjacent glTF snapshot contract pass `19/19` in one
  local source/model batch.
- Runtime868 is grouped with Runtime866 and Runtime867 for one managed
  current-source lane; it is not submitted alone.
- Local evidence does not establish Windows compilation, allocator behavior,
  or glTF import product p50/p95/p99 latency.
- Tooling production remains deferred for the later Rust migration.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/asset/importer/ingest/gltf_decode/images.rs` | `80412B2EE2DE9E91D8B50192057A5EAC6DB0B58EC5407867B8CBB6D6AE9A515C` |
| `zircon_runtime/src/asset/importer/ingest/gltf_decode/images/optimization_batch_runtime868_gltf_image_output_capacity_tests.rs` | `6D14001BA748F88DAD48A43E0FD15987B1CCD0422548300D7B5FC32B427494E2` |
| `tools/tests/test_runtime868_gltf_image_output_capacity_performance_contract.py` | `D0C8E93467716C4C5E6F6ACEC1615948B5FC7000A91F495BD76BED37A901DD74` |
| `tools/tests/test_runtime_gltf_snapshot_contract.py` | `EFB305B4C68D0B4942083ECBC3F717D883E1A054075D762FDEBB7F07662C6F9D` |

## Acceptance boundary

Keep this record `implementation_complete` / `managed_validation_pending` until
the combined Windows lane compiles Runtime, executes the lower regression and
ignored Release marker, and supplies allocator plus glTF-import product
p50/p95/p99 evidence. The deterministic growth model is not product acceptance.
