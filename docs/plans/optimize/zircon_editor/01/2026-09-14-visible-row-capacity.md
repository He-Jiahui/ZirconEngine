---
title: Editor01 AssetContent visible-row bounded capacity
category: zircon_editor
report_id: Editor760-asset-visible-row-capacity-2026-09-14
date: 2026-09-14
session_id: root-astra-optimize-20260914
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Editor01 AssetContent visible-row bounded capacity

## Scope

`AssetContentPaintMetadata` already uses `partition_point` to identify the groups
intersecting a damage clip, but each visible query appended those groups into the
fixed-row clone without admitting the known row bound first. Activity and Asset
Browser scroll/damage repaint therefore paid geometric `Vec` growth whenever the
fixed rows had no spare capacity. This slice addresses that bounded append path
from the Editor01 retained-UI review and is intentionally narrower than the full
PERF-MVP-219 target: it does not claim that the returned visible-row vector or its
sort has been removed.

## Implementation

- Added `reserve_visible_group_rows`, which sums `node_rows.len()` for the clipped
  `[first, last)` group range with saturating arithmetic and reserves that exact
  additional bound before appending.
- Wired the helper into the shared non-virtual `append_visible_group_rows` path.
  `visible_node_rows`, activity projections, and Browser source/reference/Used By
  projections all retain their existing clip intersection, binary group bounds,
  append order, visible-group count, and final sort behavior.
- The virtualized Browser content helper remains unchanged; its materialized-item
  ownership and budget contract are a separate boundary and are not folded into
  this capacity-only slice.

## Deterministic pressure model

For a fixed-row prefix followed by two visible groups containing five node rows,
the legacy append starts with no admitted visible-row bound and requires at least
one geometric growth step when the prefix exhausts spare capacity. The optimized
path admits the five-row bound before the first append, so the modeled visible
append growth count is zero. The lower ignored marker
`EDITOR760_VISIBLE_GROUP_ROWS_CAPACITY_BENCH_V1` records the same comparison for
4,096 synthetic values. These are allocation-shape signals, not CPU, RSS, or
product p50/p95/p99 measurements.

## TDD and local evidence

- The new source contract was intentionally RED before the helper existed, then
  GREEN at `4/4` after the reservation and wiring were implemented.
- `paint_metadata/capacity_tests.rs` covers the exact lower bound, clipping,
  visible-group count, row order, and the ignored Release marker.
- Scoped Rustfmt and `git diff --check` pass for the touched source/test/contract
  files. The focused five-contract capacity batch passes `21/21` in `0.127s`.
  The merged Runtime/Editor performance-contract batch loads `515` modules and
  passes `1844/1844` in `47.661s` in one process; this is local source/model
  evidence only.

## Source fingerprints

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/ui/workbench/asset_content_layout/paint_metadata.rs` | `5B25032109E13C5E5D9F9393D3D0D410D2975066265C2087249CFDC8D8903E64` |
| `zircon_editor/src/ui/workbench/asset_content_layout/paint_metadata/capacity_tests.rs` | `E37DA45E0489FBE72D864A8DA4E31553113C306DF846F6D018EECD82CD0696A1` |
| `tools/tests/test_editor_asset_visible_row_capacity_performance_contract.py` | `A4E1E4E4922E2ACFD1B94C0AA4DFFC8CA4BC7519D8B6D07128EC5D7694B28B85` |

## Managed acceptance gate

Keep this slice at `managed_validation_pending` until the owner-attributed
batched Windows Release lane proves current-source compilation, visible-row
parity, allocation behavior, and Asset Browser/Activity paint p50/p95/p99.
No standalone Cargo command or coordinator status query is used here; tooling
production work remains deferred by request.
