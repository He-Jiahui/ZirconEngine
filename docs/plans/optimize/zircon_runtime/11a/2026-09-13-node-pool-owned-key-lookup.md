---
title: Runtime11A node-pool owned key lookup reuse
category: zircon_runtime
report_id: Runtime11A-node-pool-owned-key-lookup-2026-09-13
date: 2026-09-13
session_id: root-astra-optimize-20260909
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime11A node-pool owned key lookup reuse

## Scope

The retained-child insertion path asks `UiSurfaceNodePool` for a matching pooled node on
every reuse attempt. The borrowed lookup previously cloned the desired component, control ID,
and node path into a temporary `UiSurfaceNodePoolKey`, even though the caller already owned the
desired node and needed those fields unchanged after a hit or miss. This slice removes those
hot-path key clones while retaining the existing bounded buckets and eviction behavior.

## Implementation

- `UiSurfaceNodePool::take_owned` temporarily moves the desired node's template metadata and
  path strings into the lookup key, probes the retained bucket, then restores the fields before
  returning. A cache miss therefore returns the exact desired node, while a hit returns the
  pooled node plus the same desired identity for `merge_reused_node`.
- `insert_or_reuse_pooled_child` uses the owned lookup; the public borrowed `take` API remains
  available for callers that do not transfer ownership. Recycling still constructs the owned
  key from the detached node, and bucket/node capacity limits are unchanged.
- Residency reporting now obtains node and bucket counts through one combined bucket walk before
  publishing a mutation report; the public count accessors and bounded trim semantics are
  unchanged.
- Lower Rust regressions verify pointer identity on miss and hit, matching-bucket removal, and
  resident-count accounting. A source contract rejects the old desired-key clone in the reuse
  path and requires move/restore of all key fields.

## Validation boundary

- The focused node-pool/layout-order/incremental-layout contract batch passed `27/27`.
- The current one-process Runtime/Editor UI performance-contract batch loaded 173 modules and
  passed `776/776` tests in `1.962s`, including this slice and the preceding virtual-list
  reconciliation guards.
- A broader non-tooling Runtime/Editor performance-contract and pressure batch loaded 548
  modules and passed `2057/2057` tests in `15.174s` after this slice.
- Scoped Rustfmt parsing, Python compilation, and source-contract checks are green. Managed
  Windows Cargo/Release compilation, allocation behavior, and node-pool/navigation p50/p95/p99
  evidence remain pending under the existing external-worktree gate.

Current source hashes:

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/ui/surface/node_pool.rs` | `4278AC49103E86877D6D4556918909B0640686C899EF80A2F5DE5E764A59AD3E` |
| `tools/tests/test_runtime_ui_node_pool_lookup_ownership_performance_contract.py` | `B6860AA8545A7B20AFCA7F14AD59C95B0CB35C822F5BC2429A0F0C447098A947` |

## Acceptance boundary

This is implementation-complete static evidence only. It does not close Runtime11A's broader
data-source virtualization, runtime provider integration, resource-generation handling, or
managed performance gates. No coordinator request or status query was made for this follow-up.
