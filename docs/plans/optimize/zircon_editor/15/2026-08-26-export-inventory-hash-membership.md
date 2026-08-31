---
title: Editor15 Export Inventory Hash Membership
category: zircon_editor
report_id: Editor15-export-inventory-hash-membership-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor15 Export Inventory Hash Membership

## Scope

This slice removes ordered-tree membership from the export generation inventory's recursion guard
and seen-file index. Persisted files, tool identities, canonical digest cache entries, and sorted
directory children keep their deterministic ordering. Neither optimized set is iterated to form
an artifact, digest, cache file, or UI result.

## Change

- Replace `visiting_directories: BTreeSet<PathBuf>` with `HashSet<PathBuf>` for cycle membership.
- Replace `seen_file_paths: BTreeSet<PathBuf>` with `HashSet<PathBuf>` for prune membership.
- Preserve `BTreeMap` persistence order and the explicit directory-child sort used by digesting.
- Keep optimization tests in a dedicated child module so the production file remains below the
  large-file warning threshold.

## Deterministic Performance Evidence

| Representative 8,192 paths / 65,536 membership lookups | Before | After |
|---|---:|---:|
| Membership class | O(log n) | average O(1) |
| Path ownership | owned once per set entry | owned once per set entry |
| Digest and persisted order | deterministic | unchanged |

The ignored release gate runs 17 alternating samples and emits
`EDITOR15_EXPORT_INVENTORY_HASH_MEMBERSHIP_BENCH_V1`. Acceptance requires hash membership P95 to
be at most 60% of ordered membership P95. Exact Windows timings remain pending the coordinator run.

## Acceptance

- `optimization_batch_20260826aa_editor15_inventory_hash_membership_preserves_digest_cache`
  proves repeated product digest requests retain one seen path and one content read.
- `optimization_batch_20260826aa_editor15_inventory_uses_hash_membership_without_reordering_output`
  requires both hash sets while retaining `BTreeMap` and explicit child sorting.
- `optimization_batch_20260826aa_editor15_export_inventory_hash_membership_performance_evidence`
  checks lookup equivalence, reports both P95 values, and enforces the 60% threshold.
- Exact-file Rust 1.94.1 formatting, scoped diff checks, and source contracts must pass before
  managed validation submission.

## Remaining Parent-plan Work

Editor15 still requires capability truth, operation factories, canonical graph schemas,
transactional save, semantic compilation, immutable artifacts, live runtime preview, typed
diagnostics, jobs, plugin lifecycle, large-asset qualification, and accessibility coverage. This
slice only improves export inventory membership.
