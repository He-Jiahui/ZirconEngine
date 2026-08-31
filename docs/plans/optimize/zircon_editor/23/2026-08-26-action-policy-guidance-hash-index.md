---
title: Editor23 Action Policy Guidance Hash Index
category: zircon_editor
report_id: Editor23-action-policy-guidance-hash-index-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor23 Action Policy Guidance Hash Index

## Scope

This slice removes the nested runtime/editor diagnostic scan from UI asset runtime-report
projection. Guidance wording, editor-report order, runtime-report order, duplicate editor rows,
and policy precedence remain unchanged.

## Change

- Build one borrowed `HashSet<(&str, &str)>` from editor diagnostic node/binding keys.
- Replace each runtime diagnostic's linear editor-report scan with an average `O(1)` membership
  probe.
- Borrow both key components from the existing report; the index allocates buckets but no key
  strings.

## Deterministic Performance Evidence

| 4,096 runtime diagnostics, 2,048 editor diagnostics | Before | After |
|---|---:|---:|
| Worst-case cross-report comparisons | 8,388,608 | 0 |
| Runtime membership probes | nested `O(R x E)` | 4,096 average `O(1)` |
| Borrowed key string allocations | 0 | 0 |
| Published guidance order | editor then runtime source order | unchanged |

The ignored release gate runs 17 alternating sample pairs and emits
`EDITOR23_ACTION_POLICY_GUIDANCE_HASH_INDEX_BENCH_V1`. Acceptance requires hash-index P95 to be at
least 80% below nested-scan P95. Exact Windows timings remain pending the coordinator run.

## Acceptance

- `optimization_batch_20260826al_action_policy_guidance_hash_index_preserves_rows` compares the
  complete output with the legacy algorithm, including duplicate editor keys.
- `optimization_batch_20260826al_action_policy_guidance_uses_borrowed_hash_index` requires the
  borrowed hash index and rejects the nested `.iter().any(...)` scan.
- `optimization_batch_20260826al_action_policy_guidance_hash_index_p95` reports paired P50/P95
  samples and enforces the 80% P95 reduction gate.

## Remaining Parent-plan Work

Editor23 still owns the complete UI asset authoring surface, previews, bindings, accessibility,
menus, fonts, and runtime-product parity. This slice only converges action-policy report
projection.
