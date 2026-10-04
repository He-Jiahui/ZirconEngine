---
title: Editor hierarchy control-id Arc backing reuse
category: zircon_editor
report_id: Editor01-hierarchy-control-id-arc-sharing-2026-09-13
date: 2026-09-13
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor01 hierarchy control-id Arc backing reuse

## Scope

The reusable hierarchy projection now has forward entity-to-control and reverse
control-to-entity indexes. Each direction previously owned an independent
`String` for the same control identifier, so a full reflow paid two heap string
allocations per populated row. This follow-up changes only the key ownership;
lookup, ordering, duplicate-key, truncated-control, and stale-row semantics stay
unchanged.

## Implementation

Both control indexes store `Arc<str>`. `replace` converts each incoming control
identifier once, clones the `Arc` handle for the forward index, and moves the
same allocation into the reverse index. The public internal lookup surface still
accepts borrowed `&str`, so callers do not need to allocate or change routing
contracts.

## Deterministic work model

For `N` populated rows with distinct control identifiers, the retired path
materialized `2N` owned string payloads while the current path materializes `N`
payloads plus `N` small reference-count handles. The two hash-table entries per
row remain intentional because both lookup directions are required. Duplicate
identifier overwrite behavior is unchanged; this slice does not claim to
deduplicate repeated input values.

## Validation

- The new control-sharing source/pressure contract was intentionally RED before
  the type and construction change, then GREEN at `3/3`.
- The existing hierarchy generation/index contract was repaired for the new
  `Arc<str>` key types and passes `9/9` together with the new contract.
- The in-file Rust regression checks that the forward and reverse keys are
  `Arc::ptr_eq`, while the existing regression retains lookup, truncation,
  selection, and stale-row coverage.
- The focused source batch passes `33/33` across eight hierarchy, routing, and
  adjacent Runtime hit-grid modules (including control-sharing `3/3` and the
  repaired generation/index contract `9/9`). The single-process non-tooling
  Runtime/Editor performance-plus-pressure loader covers `553` modules and
  passes `2059/2059` tests in `13.480s`.
- Current source fingerprints are:

  | File | SHA-256 |
  | --- | --- |
  | `zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/scene_hierarchy_projection.rs` | `49850F245B6677B5E747997D3ED7385026D595634C507BFC6A4A3DC38A4C70B4` |
  | `tools/tests/test_editor_scene_hierarchy_control_arc_sharing_performance_contract.py` | `1FADA62B892B33A00098F807A18C0F4293F6C38E16F49A901A13E6879673F449` |
  | `tools/tests/test_editor_scene_hierarchy_generation_index_performance_contract.py` | `232BCA3EE7017747683FBBBD98507E41BEA95FC69D785BBBF00E028DAF255554` |

These are local source/model receipts. The final static batch also passed
Python compilation, scoped Rustfmt, Wiki validation (`272/272` pages), scoped
diff checks, and target whitespace checks. Managed Cargo and Windows Release
allocation/latency evidence remain pending under the external
`E:\Git\zr_vm` dirty-worktree admission gate.

## Acceptance boundary

This record is source/contract complete only. Keep
`validation_status: managed_validation_pending` until the owner-attributed
batched Windows Release run verifies hierarchy compilation, routing parity,
allocation/RSS behavior, and the declared p50/p95/p99 latency gates.
