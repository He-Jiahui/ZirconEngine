---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/09f2/2026-08-24-lightmap-slot-index.md
related_records:
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
  - docs/plans/astra/features/runtime/972-runtime09d-to-09h1-completion-list.md
implementation_files:
  - zircon_runtime/src/core/framework/render/environment/lightmap.rs
  - zircon_runtime/src/core/framework/render/environment/lightmap/tests.rs
tests:
  - zircon_runtime/src/core/framework/render/environment/lightmap/tests.rs
---

# Runtime970 · immutable lightmap slot index

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Runtime09F2 lightmap consumption | `LightmapConsumeContract` normalizes and owns an ascending immutable slot table at construction/deserialization, uses partition-point lookup, validates duplicates by adjacent scan, and lets GPU-scene synchronization query the generation-owned table without a per-frame map/copy. | Constructor/deserialization ordering, duplicate first-match compatibility, generation replacement, serde roundtrip, GPU consumer behavior, and the ignored release marker are wired. The modeled 100K-slot/10K-last-slot workload changes `1,000,000,000` comparisons to at most `170,000` and removes stable-frame index allocations. | implemented_pending_validation |

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/core/framework/render/environment/lightmap.rs` | `04D6CE75BC04B8E6C839014C8BABDC1359696EF2CAA597F76127E876F504A13C` |
| `zircon_runtime/src/core/framework/render/environment/lightmap/tests.rs` | `A0538CFA16DB229EFF54B50A9D41F6B59212CB69DAC0B3CB8C062A9577213E11` |

## Validation handoff

The lightmap behavior suite and `RUNTIME09F2_LIGHTMAP_SLOT_BENCH_V1` are part of
the grouped Runtime package validation. Exact elapsed release evidence and
lightmap product gates remain pending.
