---
record_kind: milestone
status: implemented_pending_validation
created_at: 2026-09-11
plan: docs/plans/astra/layouts/02-native-minimums-and-layout-restore.md
milestone: ED-A5 native width-tier bridge
session: astra-ed-a5-width-policy-20260911-01a090c1
---

# ED-A5 native width-tier bridge

The native width policy now exposes the intended staged transition for the
default Scene layout: Regular/Wide retain the 640 logical-pixel minimum,
Narrow lowers the minimum to the 480 Ultra entry point, and Ultra lowers it to
the existing 420 compact minimum. This prevents the native 640 minimum from
trapping the shell before Ultra can become active.

## Scope

- `zircon_editor/src/ui/workbench/autolayout/layout_tier.rs`
- `zircon_editor/src/tests/workbench/layout/editor_layout_contracts/breakpoints.rs`
- `zircon_editor/src/tests/workbench/layout/editor_layout_contracts/geometry.rs`

ED-A6 split/product projection is unchanged. The bounded implementation is an
explicit P1 exception authorized by the Astra goal; it does not promote the
repository MVP gate or claim product acceptance.

## Evidence

- Focused policy regressions cover the 640 -> 480 -> 420 transition and reject
  a return to the 640 floor immediately above Narrow.
- Breakpoint and Scene geometry regressions cover DPI scales 1.0, 1.25, and
  2.0, with logical policy values and scaled physical geometry checked at the
  640 and 480 boundaries.
- `geometry.rs` also contains pre-existing finite-extents and tiny-shell matrix
  tests from another dirty-worktree owner; they are preserved verbatim and are
  not part of this ED-A5 change.
- Scoped Rust formatting and whitespace checks pass for the owned changes.

## Remaining gate

Cargo and native-window validation were deliberately not run in this slice.
The record remains `implemented_pending_validation` until managed focused Rust
tests and Windows product proof confirm continuous resize through both native
minimum transitions, including a live DPI change.
