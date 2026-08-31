---
title: Runtime07 Native Plugin Traversal Hash Membership
category: zircon_runtime
report_id: Runtime07-native-plugin-traversal-hash-membership-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime07 Native Plugin Traversal Hash Membership

## Scope

This slice removes ordered-tree membership from canonical directory cycle detection during native
plugin manifest discovery. Breadth-first traversal remains owned by `VecDeque`, filesystem entry
inspection order and visitor callbacks are unchanged, and duplicate paths still avoid a second
owned `PathBuf` clone.

## Change

- Replace `BTreeSet<PathBuf>` with `HashSet<PathBuf>` for canonical visited directories.
- Preserve borrowed `contains` before cloning a newly admitted canonical path.
- Preserve canonical-root containment, depth limits, diagnostics, scratch accounting, and BFS.

## Deterministic Performance Evidence

| Representative 65,536 admissions / 8,192 unique directories | Before | After |
|---|---:|---:|
| Membership class | O(log n) | average O(1) |
| Owned path clones | unique directories only | unchanged |
| Traversal/visitor order | queue and filesystem ordered | unchanged |

The ignored release gate runs 17 alternating samples and emits
`RUNTIME07_NATIVE_PLUGIN_TRAVERSAL_HASH_MEMBERSHIP_BENCH_V1`. Acceptance requires hash membership
P95 to be at most 60% of ordered membership P95. Exact Windows timings remain pending the
coordinator run.

## Acceptance

- `optimization_batch_20260826ac_runtime07_hash_traversal_visits_each_directory_once` exercises
  the product traversal across a four-directory tree.
- `optimization_batch_20260826ac_runtime07_native_plugin_traversal_uses_hash_membership` requires
  the hash visited set and clone-on-first-visit boundary.
- `optimization_batch_20260826ac_runtime07_native_plugin_traversal_hash_membership_performance_evidence`
  checks admission equivalence, reports both P95 values, and enforces the 60% threshold.
- Exact-file Rust 1.94.1 formatting, scoped diff checks, and source contracts must pass before
  managed validation submission.

## Remaining Parent-plan Work

Runtime07 still needs signed package trust, lifecycle-safe native unloading, schema-qualified
bridge generation, capability closure, bounded discovery, and product-scale plugin qualification.
This slice only improves canonical traversal membership.
