---
title: Runtime03 Style Plan Token Map Sharing
category: zircon_runtime
report_id: Runtime03-style-plan-token-map-sharing-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime03 Style Plan Token Map Sharing

## Scope

This slice removes per-rule deep copies of resolved stylesheet token maps during UI style-plan
construction. Selector parsing, specificity/order, declaration ownership, token resolution,
stylesheet isolation, and empty-sheet behavior remain unchanged.

## Change

- Clone each non-empty resolved stylesheet token map once into an `Arc` before visiting its rules.
- Give every parsed rule in that stylesheet a cheap `Arc` clone instead of a full `BTreeMap` clone.
- Keep different stylesheets on distinct token-map allocations and skip allocation for sheets with
  no rules.

## Deterministic Performance Evidence

| One sheet with 256 rules and 512 tokens, one plan build per sample | Before | After |
|---|---:|---:|
| Owned token-map instances | 256 | 1 |
| Token entries deeply cloned | 131,072 | 512 |
| Shared-pointer clones | 0 | 256 |
| Cross-stylesheet token sharing | none | none |

The ignored release gate runs 17 alternating sample pairs and emits
`RUNTIME03_STYLE_PLAN_TOKEN_MAP_SHARING_BENCH_V1`. Acceptance requires shared-map plan construction
P95 to be at least 50% below per-rule deep copies. Exact Windows timings remain pending the
coordinator run.

## Acceptance

- `optimization_batch_20260826at_style_plan_shares_tokens_per_sheet` covers same-sheet sharing,
  cross-sheet isolation, and token contents.
- `optimization_batch_20260826at_style_plan_clones_token_map_once_per_sheet` requires one deep map
  clone outside the rule loop and `Arc` clones inside it.
- `optimization_batch_20260826at_style_plan_token_map_sharing_p95` reports paired P50/P95 samples
  and enforces the 50% P95 reduction gate.

## Remaining Parent-plan Work

Runtime03 still owns UI pipeline architecture, scheduling, cache reuse, invalidation, diagnostics,
and product-scale performance receipts. This slice only converges compiled style-plan token
ownership.
