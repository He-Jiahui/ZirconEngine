---
title: Editor menu pointer parent-path reuse
category: zircon_editor
report_id: Editor01-menu-parent-path-reuse-2026-09-13
date: 2026-09-13
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor01 menu pointer parent-path reuse

## Scope

The retained menu pointer bridge derived a submenu parent path into a fresh
`Vec<usize>` on every menu-item move, scroll, and click. This slice removes only
that event-local allocation; the larger published-row authority and popup-layer
cutover described by the menu interaction review remain separate work.

## Implementation

`menu_item_tree::reuse_parent_path` now copies the borrowed parent prefix into a
caller-owned `Vec<usize>` after clearing it, so the existing state buffer keeps
its capacity. Move and scroll handlers compare the retained path with the
borrowed prefix before copying, preserving the existing rebuild/no-rebuild
decision. Click handling uses the same helper, and the old allocating
`parent_path -> Vec<usize>` helper and its callers are removed.

## Deterministic work model

For a route depth `D`, parent-path comparison and copying remain `O(D)` and keep
the same path values. After the first sufficiently deep route, steady-state
move/scroll/click handling performs no parent-path heap allocation; only genuine
capacity growth can allocate. The route index, popup geometry, submenu rebuild,
action identity, and dismissal semantics are unchanged. This does not claim the
full menu interaction design is allocation-free: row/model materialization and
popup `UiSurface` replacement remain the separate `design_ready_e4` items in the
source review.

## Validation

- The new source contract was intentionally RED while callers used the
  allocating `parent_path` helper, then GREEN after the retained-buffer helper
  and borrowed-prefix guards landed.
- The in-file Rust regression verifies parent-prefix values, single-item/empty
  paths, and pointer/capacity stability across repeated copies.
- Current source SHA-256 values:

  | File | SHA-256 |
  | --- | --- |
  | `zircon_editor/src/ui/retained_host/menu_pointer/menu_item_tree.rs` | `BFCF1D7ED4E2E56BA0BC36D754906B11E0FB8FDC27B9AEFA93762D4A88773B19` |
  | `zircon_editor/src/ui/retained_host/menu_pointer/menu_item_tree/capacity_tests.rs` | `EC57F317118C93ADF28BDBC0689A26F79E0CBCE25C6932EFBF29ADB962F9455C` |
  | `zircon_editor/src/ui/retained_host/menu_pointer/host_menu_pointer_bridge_handle_move.rs` | `FA0E149EA1A628817490842AA02DEB010B104636D0E5AD6237646834FA225237` |
  | `zircon_editor/src/ui/retained_host/menu_pointer/host_menu_pointer_bridge_handle_scroll.rs` | `CE40963D03AC71F5AA25C9171B1F59CA05A193EE0ECFC535D887839C8B4A18AA` |
  | `zircon_editor/src/ui/retained_host/menu_pointer/host_menu_pointer_bridge_handle_click.rs` | `FF514A1CB7A052DC2BBC2086787D4F896B9728B0BA227F8AC4DBD639C2EB22DE` |
  | `tools/tests/test_editor_menu_parent_path_reuse_performance_contract.py` | `E408C98031C4C44367817757B6DD9C6E6FF138A426C312D3E304B31CFAD45B1F` |

- The focused parent-path contract passes `2/2`; scoped Rust parse checks and
  `git diff --check` pass. The refreshed one-process non-tooling Runtime/Editor
  performance-plus-pressure loader covered `352` modules and passed `1349/1349`
  tests in `10.228s`, including this new contract with no load failures.

These are source/contract results only. Managed Cargo, allocator, RSS, and
menu input p50/p95/p99 product evidence remain pending.

## Acceptance boundary

Keep `validation_status: managed_validation_pending` until the existing
owner-attributed Windows Release batch verifies compile, route parity, steady
state allocation behavior, and the declared menu input latency gates. No new
coordinator request or status query is made for this slice.
