---
title: Editor08 Recent Command Hash Membership
category: zircon_editor
report_id: Editor08-recent-command-hash-membership-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor08 Recent Command Hash Membership

## Scope

This slice removes logarithmic membership checks from command-palette recent-command projection.
Recent IDs are parsed and deduplicated into a private set used only by `contains`; option order
continues to come from projected command entries.

## Change

- Return `HashSet<String>` from the private recent-command ID parser.
- Preserve array/string/table parsing, `id|label` normalization, empty-value filtering, and
  duplicate suppression.
- Keep command option publication, focused index, selection, and query-match ordering unchanged.

## Deterministic Performance Evidence

| Representative 8,192 recent IDs / 65,536 option lookups | Before | After |
|---|---:|---:|
| Membership class | O(log n) | average O(1) |
| Published option order | entry order | entry order |
| Recent ID ownership | set-owned | set-owned |

The ignored release gate runs 17 alternating samples and emits
`EDITOR08_RECENT_COMMAND_HASH_MEMBERSHIP_BENCH_V1`. Acceptance requires hash membership P95 to be
at most 60% of ordered membership P95. Exact Windows timings remain pending the coordinator run.

## Acceptance

- `optimization_batch_20260826y_editor08_recent_command_hash_set_preserves_parsing_and_membership`
  covers separator normalization, duplicates, empty values, and membership.
- `optimization_batch_20260826y_editor08_recent_command_ids_use_hash_membership` requires the
  private production hash return type and rejects ordered membership.
- `optimization_batch_20260826y_editor08_recent_command_hash_membership_performance_evidence`
  checks lookup equivalence, reports both P95 values, and enforces the 60% threshold.
- Exact-file Rust 1.94.1 formatting, scoped diff checks, and source contracts must pass before
  managed validation submission.

## Remaining Parent-plan Work

Editor08 still needs one authoritative command/context/keymap/menu graph, indexed context
invalidation, conflict diagnostics, remote automation admission, accessibility, and full product
latency qualification. This slice only improves recent-command projection membership.
