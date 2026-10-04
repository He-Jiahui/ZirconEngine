---
title: Editor menu popup route scratch reuse
category: zircon_editor
report_id: Editor01-menu-route-scratch-reuse-2026-09-13
date: 2026-09-13
supersedes: docs/plans/optimize/zircon_editor/340/2026-08-30-popup-item-path-capacity.md
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor01 menu popup route scratch reuse

## Scope

The retained menu pointer bridge already reserved the open submenu depth when
projecting a popup route, but it created a new `Vec<usize>` for that reserve on
every pointer event. This slice retains only the recursive working path across
events. Returned route intents still clone their path because they outlive the
projection call; published-row authority, popup surface replacement, and row
materialization remain separate design-ready work.

## Implementation

`HostMenuPointerBridge` now owns `popup_item_path_scratch`. Popup route
projection clears and capacity-prepares that buffer, passes it through the
recursive `PopupProjection`, and clears it after the owned route result is
formed. `project_route_at_point` and its popup helper are mutable only because
they borrow this bridge-owned scratch; dispatch and scroll are the existing
callers. The old `popup_item_path` helper remains test-only for the historical
capacity benchmark and is no longer on the production event path.

## Deterministic work model

For open depth `D`, scratch preparation and recursive path walking remain
`O(D)`. After the first route reaches the retained high-water mark, stable
pointer projection performs no allocation for the recursive working path; only
the required owned route-intent path/action payloads remain. Clearing after the
projection prevents stale indices from crossing events and preserves the
existing child-first hit order, route-index lookup, disabled-item fallback, and
submenu geometry behavior.

## Validation

- The new source contract was intentionally RED before the bridge field and
  mutable projection path existed, then GREEN after the retained scratch was
  wired through construction and both projection callers.
- The in-file Rust regression checks that clearing/preparing a populated path
  preserves its allocation and capacity for repeated depths.
- Rustfmt check (`--config reorder_imports=false`, preserving the pre-existing
  import-order edit) and the focused Editor menu batch pass. The
  one-process non-tooling Runtime/Editor performance-plus-pressure loader
  covered `353` modules and passed `1352/1352` tests in `11.319s`, with zero load
  failures. Python compilation and scoped whitespace/diff checks also pass.

Current source SHA-256 values:

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/ui/retained_host/menu_pointer/host_menu_pointer_bridge.rs` | `565834968942925217C16D3818633072DA42126A84C47D0F78D62339D27006D8` |
| `zircon_editor/src/ui/retained_host/menu_pointer/host_menu_pointer_bridge_new.rs` | `075F8D35F8ACDDFFB038A6B3A634AC30DF22E80C2C4CAB705D0899D985BBE422` |
| `zircon_editor/src/ui/retained_host/menu_pointer/host_menu_pointer_bridge_project_route.rs` | `86589DBFD61836EAD6FE9AC3003F6276DACA995C501C02870F5634848EE8B433` |
| `tools/tests/test_editor_menu_route_scratch_reuse_performance_contract.py` | `84516D6642FEC0E86796325C2E7D0A7405273C9DDBDD1AF2855230EF0F8DEB81` |

The ignored `EDITOR733_POPUP_ROUTE_SCRATCH_BENCH_V1` is authored for the
managed Windows Release batch; no local release timing is presented as a
product gate.

Managed Cargo, allocator, RSS, and menu pointer p50/p95/p99 product evidence
remain pending the existing owner-attributed Windows Release admission. No new
coordinator request or status query is made for this slice.

## Acceptance boundary

Keep `validation_status: managed_validation_pending` until the owner-attributed
batched Windows Release gate verifies compilation, route parity, steady-state
allocation behavior, and the declared menu input latency thresholds.
