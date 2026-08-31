---
title: Runtime07 Linked Plugin Hash Indexes
category: zircon_runtime
report_id: Runtime07-linked-plugin-hash-indexes-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime07 Linked Plugin Hash Indexes

## Scope

This slice removes repeated linear and ordered-set membership work from dynamic-session linked
plugin preparation. It preserves manifest selection order and diagnostics, but does not implement
the parent plan's package resolver, immutable catalog generation, trust policy, or lifecycle lease.

## Change

- Build a borrowed `HashSet<&str>` over existing and registration-owned selection IDs.
- Admit missing registration selections without rescanning the growing effective manifest.
- Build enabled-selection membership as a borrowed hash set instead of an ordered owned set.
- Store prepared package membership in `HashSet<String>` for direct session capability queries.

## Deterministic Performance Evidence

| Representative 16,384 admissions / 2,048 unique selections | Before | After |
|---|---:|---:|
| Registration membership work | 16,783,360 string comparisons | 16,384 average-O(1) hash admissions |
| Extra owned ID clones for admission index | 0 | 0; the index borrows input IDs |
| Enabled-selection membership | ordered owned-string set | borrowed hash set |
| Prepared package lookup | up to `package_count` string comparisons | average O(1) hash lookup |
| Published effective-manifest order | first-seen | first-seen |

The ignored release gate alternates 17 linear-admission and borrowed-hash-admission samples and
emits `RUNTIME07_LINKED_PLUGIN_HASH_INDEXES_BENCH_V1`. Acceptance requires hash-admission P95 to
be at most 35% of linear-admission P95. Exact Windows timings remain pending the coordinator run.

## Acceptance

- `optimization_batch_20260826s_runtime07_borrowed_hash_admission_preserves_first_seen` covers
  duplicate rejection and first-seen publication order through the product admission helper.
- `optimization_batch_20260826s_runtime07_linked_plugins_use_hash_indexes` requires both borrowed
  hash collections and direct prepared-package membership while rejecting production tree/linear
  membership.
- `optimization_batch_20260826s_runtime07_linked_plugin_hash_admission_performance_evidence`
  emits workload and both P95 values, then enforces the 35% threshold.
- Exact-file Rust 1.94.1 formatting, scoped diff checks, and source contracts must pass before
  managed validation submission.

## Remaining Parent-plan Work

Runtime07 still needs one immutable plugin catalog generation, version/source/trust resolution,
dependency and conflict solving, backend lifecycle leases, crash isolation, script budgets,
debug/profiling contracts, and 1/10/100/1,000-package product qualification.
