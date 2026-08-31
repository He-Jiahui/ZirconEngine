---
title: Runtime84 Emoji Shortcode Hash Index
category: zircon_runtime
report_id: Runtime84-emoji-shortcode-hash-index-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime84 Emoji Shortcode Hash Index

## Scope

This slice replaces the parser-local emoji shortcode replacement owner with `HashMap`. Every
normalized `:name:` candidate in rich-text expansion now resolves through expected constant-time
lookup, including built-in and project-registered shortcodes.

The registry exposes no iterator. Name trimming and ASCII case normalization, one-grapheme
replacement validation, duplicate rejection, unknown-shortcode preservation, parser-local
ownership, and output scan order are unchanged.

## Performance Workload

The release workload fills 4,096 long shared-prefix shortcode names and performs 4,096 stable hits
for the final name.

| Work per workload | Before | After |
|---|---:|---:|
| Ordered shortcode lookups | 4,096 | 0 |
| Hash shortcode lookups | 0 | 4,096 |
| Shortcode iteration-policy changes | 0 | 0 |
| Allocations on registry hits | 0 | 0 |

The ignored release gate runs 17 alternating sample pairs and emits
`RUNTIME84_EMOJI_SHORTCODE_HASH_INDEX_BENCH_V1`. Acceptance requires hash lookup P95 to be at least
30% below the legacy `BTreeMap` path. Exact Windows P50/P95 timings remain pending the coordinator
run.

## Acceptance

- `optimization_batch_20260826by_emoji_shortcode_hash_index_preserves_expansion_contract` covers
  normalization, built-in and custom expansion, unknown preservation, and duplicate rejection.
- `optimization_batch_20260826by_emoji_shortcode_hash_index_has_no_ordered_iteration` locks the
  unordered owner contract.
- `optimization_batch_20260826by_emoji_shortcode_hash_index_p95` reports paired release P50/P95
  samples and enforces the 30% P95 reduction gate.

## Remaining Parent-plan Work

Runtime84 still owns parser budgets, diagnostics, decorator isolation, generation retirement,
incremental parsing, semantic projection, rich editing, and product integration. This slice only
converges shortcode lookup inside the existing parser contract.
