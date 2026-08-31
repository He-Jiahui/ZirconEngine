---
title: Runtime Interface 03 Focus-chain Candidate Partition
category: zircon_runtime_interface
report_id: RuntimeInterface03-focus-chain-partition-2026-08-25
date: 2026-08-25
session_id: optimize-runtime-interface03-focus-chain-partition-r1-20260825
implementation_status: implementation_complete
validation_status: managed_validation_passed
---

# Runtime Interface 03 Focus-chain Candidate Partition

## Scope

This batch advances the retained UI focus-navigation path described by Runtime Interface 03. It
preserves reachability, render visibility, enabled/focusable filtering, disabled tab-index
exclusion, explicit tab-index priority, stable equal-index pre-order, and default pre-order. It does
not close the parent plan's accessibility bridge, input dispatch, diagnostics, authoring schema, or
status-surface work.

## Change

`focus_chain` previously represented every accepted node as a candidate carrying an optional tab
index and pre-order number, then sorted the complete candidate vector by optional-index state,
index order, and pre-order. Default candidates are already produced in the required traversal
order, so sorting them repeats work without changing the result.

The traversal now writes default candidates directly to a `Vec<UiNodeId>` and explicit tab-index
candidates to a separate vector. Finalization sorts only the explicit subset and prepends those
IDs to the default pre-order vector. If no explicit tab index is present, the default vector is
returned directly without another allocation or sort. For the 10,000-default-candidate release
fixture, sorted candidates are `10,000 -> 0`.

The ignored release gate measures only candidate finalization over 21 alternating sample pairs so
tree lookup and reachability costs cannot hide the removed sort. Optimized P95 must be at most 35%
of the legacy full-candidate finalization P95. Managed Windows-native ticket
`235eba0a940e4ab28e1cc3d9bfe2a415` measured `32,600 ns -> 100 ns` P95, a `99.693%`
reduction; sorted candidates changed from `10,000 -> 0`.

## Validation

- TDD red state: the new source performance contract failed 3/3 against the full-vector sort.
- Source performance contract after implementation: 3/3 passed.
- `rustfmt --check` and scoped whitespace validation: passed.
- Managed Windows-native validation passed the exact existing focus-order regression, all 3 source
  contracts, `rustfmt --check`, and the ignored release performance gate.
- No local Cargo lane or Cargo dry-run was launched, polled, or terminated.

## Remaining Parent-plan Work

Runtime Interface 03 still requires the full UI authoring and accessibility contract review,
input-routing ownership, stable diagnostics/status publication, compatibility evidence, and the
remaining product-level performance budgets.
