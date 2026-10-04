---
title: Runtime200 Valid Input Owner Route Single Pass
category: zircon_runtime
report_id: Runtime200-valid-input-owner-route-2026-09-26
date: 2026-09-26
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime200 valid input owner route single pass

## Scope

Keyboard and focused owner dispatch previously validated the target through
every ancestor, then traversed the same parent chain again to build the bubble
route written to focus diagnostics. Both passes looked up each node in the UI
tree. The shared validation owner now supports a visitor during the validity
walk. The bool-only route keeps its no-allocation query, while consumers that
need the route collect the validated IDs in target-to-root order in one pass.

Keyboard dispatch and focused owner dispatch now consume that validated route.
Non-focused events still use the bool-only query. Invalid targets preserve the
unhandled reply and route-rejection diagnostics. A 64-node inline route buffer
avoids heap allocation until the complete short route is validated. A deeper
route moves the accumulated IDs into a Vec and continues the same traversal;
routes of up to 64 nodes allocate their output once after validation. Keyboard
rejection reuses its existing dispatch result instead of validating the
invalid owner a second time.

## Evidence and target

- Focused tests check exact route order against `UiTree::bubble_route`, a
  route longer than the 64-node inline buffer, rejection of a hidden ancestor
  and a missing node, and actual keyboard dispatch through a newly hidden
  ancestor. The dispatch regression checks the unhandled reply, rejection
  note, and absence of a recorded focused input. Existing focused-route
  integration tests cover the published `UiFocusedInput.route` contract.
- For a valid route of depth H, the changed dispatch paths perform one owner
  validity walk and one node lookup per ancestor instead of a validity walk
  plus an independent bubble-route walk. They still allocate the one output
  route required by the diagnostic contract. For invalid routes of at most
  64 valid nodes, the collector performs no route heap allocation; keyboard
  rejection also avoids a duplicate validation walk.
- `RUNTIME200_VALID_INPUT_OWNER_ROUTE_BENCH_V1` compares a test-local exact
  copy of the HEAD `is_valid_input_owner` parent loop followed by the existing
  `bubble_route` with the fused operation at depth 64. It checks validator
  behavior and route equality before timing, then uses four warmups, 17 paired
  samples, and 4,096 routes per sample. The release P95 target is at least
  20% below that original two-pass baseline.

## Grouped validation manifest

Combine this slice with other Runtime package work in one managed Windows
validation lane:

1. `cargo check -p zircon_runtime --lib` for the package batch.
2. `cargo test -p zircon_runtime --lib valid_input_owner_route`,
   `cargo test -p zircon_runtime --lib hidden_ancestor_rejects_keyboard_dispatch_without_recording_focused_input`,
   `cargo test -p zircon_runtime --lib unified_focus_and_capture_dispatch_report_phase_route_steps`,
   and `cargo test -p zircon_runtime --lib unified_keyboard_default_activation_reports_focus_route_steps_and_component_event`
   as focused filters in the same package batch.
3. `cargo test --release -p zircon_runtime --lib valid_input_owner_route_release_p95 -- --ignored --nocapture`
   grouped with the milestone's release evidence.
4. `rustfmt --check --edition 2021` and `git diff --check` on the four source
   files plus the two plan records.

This implementation slice has no Cargo or release measurement. The traversal
count follows from the source; behavior and P95 acceptance require terminal
managed validation. It does not close the broader Runtime200 transaction,
multi-window, or product input-latency gates.
