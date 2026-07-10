---
related_code:
  - zircon_runtime/src/asset
  - zircon_runtime/src/tests/runtime_absorption/performance_hotspots/scene_project_splits/scene_asset.rs
plan_sources:
  - docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
output_records:
  - docs/plans/zircon_runtime/runtime/04/2026-07-09-asset-pipeline-alignment-output-records.md
---

# Runtime Asset Filter Current Result

Date: 2026-07-10

The successfully compiled default-feature, locked runtime lib-test binary executed the `asset::` filter as 618 tests: 611 passed, 7 failed, and 6820 were filtered out in 81.90 seconds.

One failure belonged to the broad Runtime 07 scene-asset structure guard and was caused by reading concrete status from route-only parent plans. Its current source now reads Runtime 07's numbered output archives and passes standalone 1/1.

The other six failures are in active or distinct owner lanes: font artifact-cache deserialization, the Vampire shader fixture, two render graph/stage expectations, the UI document compiler, and UI v2 persistent cache. They were not changed by this Runtime 04 slice. Therefore the exact `asset::` gate remains failed/pending until a current binary is rebuilt after the guard repair and the external owner failures are resolved.
