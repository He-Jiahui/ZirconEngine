---
related_code:
  - zircon_editor/src/ui/host/layout_commands.rs
  - zircon_editor/src/ui/host/workspace_state.rs
implementation_files:
  - zircon_editor/src/ui/host/layout_commands.rs
plan_sources:
  - user: 2026-09-21 optimize Runtime and Editor hot paths and record completion
tests:
  - tools/tests/test_editor867_871_workspace_identity_projection_performance_contract.py
  - zircon_editor/src/ui/host/workspace_state_direct_query_batch_tests.rs
doc_type: milestone-detail
title: Editor867 layout-command floating-window existence query
category: zircon_editor
report_id: Editor867-layout-command-floating-existence-2026-09-21
date: 2026-09-21
session_id: root-runtime-editor-async-optimization-20260921
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Editor867 - layout-command floating-window existence query

## Scope and optimization

Both attach paths cloned the complete `WorkbenchLayout` after a layout command
only to decide whether one floating native window still existed or needed to be
opened. They now reuse the authoritative borrowed `floating_window_exists`
query introduced by Editor865. Attach ordering, native reattach/open behavior,
and the returned `changed` state are unchanged.

## TDD and local evidence

The combined Editor867–871 contract was observed RED with five failures and two
missing-query errors, then GREEN at `7/7`. The focused Editor859–871 plus
Workbench projection batch passes `43/43` in `0.041s`; the widened explicit
performance/contract loader passes `4441/4441` tests across `1127` files in
`290.084s`, with zero load errors, failures, errors, or skips; exact Rustfmt
passes.
The structural target changes complete layout clones from one per applicable
attach path to zero. Editor865's lower exact-identity regression and ignored
`EDITOR865_FLOATING_WINDOW_EXISTS_DIRECT_QUERY_BENCH_V1` marker cover the
shared query.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/ui/host/layout_commands.rs` | `7E168B249BA8795BDBDA6FB9911AACE14BF1E69DC0FFE34DAB44907ED610827F` |
| `tools/tests/test_editor867_871_workspace_identity_projection_performance_contract.py` | `A6E1991F5CE923E7F413AD1E4FC02954074B640681BC4364EE553C2A89049ED2` |

## Acceptance boundary

Keep managed validation pending until current-source Windows compilation,
Release marker, allocator, and attach interaction p50/p95/p99 receipts pass.
