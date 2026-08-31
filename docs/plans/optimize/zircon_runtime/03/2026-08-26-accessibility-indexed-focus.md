---
title: Runtime03 Accessibility Indexed Focus
category: zircon_runtime
report_id: Runtime03-accessibility-indexed-focus-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime03 Accessibility Indexed Focus

## Scope

This slice removes post-index linear node scans from accessibility focus validation. Focus clearing,
validity rules, duplicate-node first-match behavior, invalid-focus diagnostics, root fallback order,
and focused-state results remain unchanged.

## Change

- Retain the validated focused node index returned by the existing node-ID map.
- Mutate the focused node directly through that index instead of scanning the node array again.
- Carry both root ID and node index through fallback selection so fallback mutation is also direct.

## Deterministic Performance Evidence

| 32,768 nodes, 64 valid-focus validations per sample | Before | After |
|---|---:|---:|
| Node visits per sample | 4,194,304 | 2,097,152 |
| Post-index linear focus scans | 64 | 0 |
| Focus-state clearing passes | 64 | 64 |
| Additional indexes built | 0 | 0 |

The ignored release gate runs 17 alternating sample pairs and emits
`RUNTIME03_ACCESSIBILITY_INDEXED_FOCUS_BENCH_V1`. Acceptance requires indexed focus P95 to be at
least 25% below post-index linear lookup. Exact Windows timings remain pending the coordinator run.

## Acceptance

- `optimization_batch_20260826au_accessibility_focus_preserves_valid_and_fallback_results` covers
  valid focus, previous-state clearing, missing-focus diagnostics, and root fallback.
- `optimization_batch_20260826au_accessibility_focus_uses_existing_node_index` requires direct
  indexed mutation and rejects focused/fallback node-ID scans.
- `optimization_batch_20260826au_accessibility_indexed_focus_p95` reports paired P50/P95 samples
  and enforces the 25% P95 reduction gate.

## Remaining Parent-plan Work

Runtime03 still owns UI pipeline architecture, scheduling, cache reuse, invalidation, diagnostics,
and product-scale performance receipts. This slice only converges accessibility focus validation.
