---
title: Editor01 Runtime Artifact Font Hash LRU
category: zircon_editor
report_id: Editor01-runtime-artifact-font-hash-lru-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor01 Runtime Artifact Font Hash LRU

## Scope

This slice removes the linear queue search and queue-element movement from runtime artifact font
cache hits in retained-host text paint. Source identity, font generation, face, instance,
collection index, 64-entry capacity, duplicate insertion reuse, and least-recently-used eviction
remain unchanged. It advances Editor01 text paint without changing shaping, rasterization, font
fallback, variation handling, or cache ownership.

## Change

- Store runtime artifact font entries in a `HashMap` keyed by the complete existing font key.
- Update a monotonic access generation on hit and duplicate insert.
- Scan entries only when a full-cache miss must select the least recently used key.
- Rebase generations while preserving order if the generation counter reaches its integer limit.

## Deterministic Performance Evidence

| Stable-order paint of 64 cached runtime faces | Before | After |
|---|---:|---:|
| Linear key comparisons | 4,096 | 0 |
| Hash lookups | 0 | 64 |
| Queue removals/moves | 64 | 0 |
| Capacity/LRU changes | 0 | 0 |

Deterministic lookup work falls by 98.4375%. The ignored release gate runs 17 alternating sample
pairs and emits `EDITOR01_RUNTIME_ARTIFACT_FONT_HASH_LRU_BENCH_V1`. Acceptance requires hash-LRU
paint lookup P95 to be at least 50% below the legacy `VecDeque` implementation. Exact Windows
P50/P95 timings remain pending the coordinator run.

## Acceptance

- `optimization_batch_20260826bk_runtime_artifact_font_hash_lru_preserves_eviction` covers hit
  promotion, duplicate insertion identity reuse, full-cache eviction, and capacity.
- `optimization_batch_20260826bk_runtime_artifact_font_hash_lru_eliminates_hit_scan` locks the
  4,096-comparison model and rejects production queue-position scans.
- `optimization_batch_20260826bk_runtime_artifact_font_hash_lru_p95` reports paired release P50/P95
  samples and enforces the 50% P95 reduction gate.

## Remaining Parent-plan Work

Editor01 still owns retained presentation invalidation, layout and paint scaling, glyph atlas and
GPU upload coordination, accessibility, and product-scale interaction evidence. This slice only
converges runtime artifact font cache lookup.
