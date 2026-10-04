---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/01/2026-08-26-showcase-action-key-allocation-free-match.md
related_records:
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_editor/src/ui/retained_host/app/showcase_event_inputs.rs
  - zircon_editor/src/ui/retained_host/app/showcase_event_inputs/action_key_match_tests.rs
tests:
  - zircon_editor/src/ui/retained_host/app/showcase_event_inputs/action_key_match_tests.rs
---

# Editor980 · allocation-free showcase action-key matching

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Editor01 retained-host showcase input matching | Action-key normalization and substring/suffix matching stream borrowed bytes through cloneable iterators. Separators are delayed until a following alphanumeric byte, preserving punctuation, camel-case, empty-needle, and binding-suffix semantics without normalized `String`, segment vectors, or joined-key temporaries. | Current-source Editor binary completed the focused owner batch `3/3`. `EDITOR01_SHOWCASE_ACTION_KEY_ALLOCATION_FREE_MATCH_BENCH_V1` reports legacy P95 `1,130,800ns` versus optimized P95 `253,700ns` (`77.57%` reduction) and `64→0` heap-backed normalized keys, clearing the plan's 20% gate. Managed Editor Release/allocation/product evidence remains pending. | implemented_pending_validation |

## Deterministic boundary

Normalization still trims leading/trailing separators, collapses punctuation,
splits camel-case boundaries, supports substring matching, and normalizes
binding suffixes. Only temporary heap-backed key construction is removed.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/ui/retained_host/app/showcase_event_inputs.rs` | `200213053737C2CF422830FAD101F687390D211C1A0A50DFB8A1F08C77801930` |
| `zircon_editor/src/ui/retained_host/app/showcase_event_inputs/action_key_match_tests.rs` | `8E953EBDFE34EE3D2E26820EB09623D0B0177CA6988713134E9CFFC19E9C1B51` |

## Validation handoff

The focused current-source Editor binary was run once with
`optimization_batch_20260826bj_showcase_action_key_match --test-threads 1
--include-ignored --nocapture`; normalization, allocation/source, and ignored
performance owners all passed. This is local Debug evidence; managed Editor
Release and product-scale gates remain pending in the grouped lane.
