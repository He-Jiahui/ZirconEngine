---
title: Editor01 Showcase Action Key Allocation Free Match
category: zircon_editor
report_id: Editor01-showcase-action-key-allocation-free-match-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor01 Showcase Action Key Allocation Free Match

## Scope

This slice removes repeated heap-backed action-key construction while retained-host component
showcase events are matched against candidate actions. Slash, dot, and colon segment handling;
camel-case conversion; punctuation collapsing; leading and trailing separator trimming; substring
matching; and binding-suffix normalization remain unchanged. It advances Editor01 retained UI event
routing without changing action ownership, event payloads, control selection, or presentation.

## Change

- Stream normalized action-key bytes from a cloneable borrowed iterator.
- Delay normalized separators until a later alphanumeric byte so trailing separators remain trimmed.
- Perform substring matching by cloning iterator state only when the first needle byte matches.
- Stream binding-suffix normalization through the same matcher without constructing a temporary
  `String`.

## Deterministic Performance Evidence

| 64 candidate checks for one showcase event | Before | After |
|---|---:|---:|
| Heap-backed normalized action keys | 64 | 0 |
| Collected segment vectors | 64 | 0 |
| Borrowed normalization streams | 0 | 64 |
| Matching/normalization semantic changes | 0 | 0 |

Deterministic heap-backed key and segment-vector construction falls by 100%. The ignored release
gate runs 17 alternating sample pairs and emits
`EDITOR01_SHOWCASE_ACTION_KEY_ALLOCATION_FREE_MATCH_BENCH_V1`. Acceptance requires streaming match
P95 to be at least 20% below the legacy split-map-collect-join implementation. Exact Windows
P50/P95 timings remain pending the coordinator run.

## Acceptance

- `optimization_batch_20260826bj_showcase_action_key_match_preserves_normalization` compares the
  streaming matcher with the legacy algorithm across camel case, punctuation, empty normalized
  segments, absent needles, empty needles, and binding suffixes.
- `optimization_batch_20260826bj_showcase_action_key_match_eliminates_heap_keys` rejects production
  `String`, segment-vector, and joined-key construction.
- `optimization_batch_20260826bj_showcase_action_key_match_p95` reports paired release P50/P95
  samples and enforces the 20% P95 reduction gate.

## Remaining Parent-plan Work

Editor01 still owns retained presentation invalidation, layout and paint scaling, input routing,
native-window projection, accessibility, and product-scale interaction evidence. This slice only
converges component-showcase action matching.
