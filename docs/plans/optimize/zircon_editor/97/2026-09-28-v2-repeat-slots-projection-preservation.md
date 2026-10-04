---
title: Editor97 V2 Repeat and Node Slot Projection Preservation
category: zircon_editor
date: 2026-09-28
implementation_status: implemented_pending_validation
validation_status: scoped_static_checks_passed_managed_validation_pending
performance_status: product_gate_pending
---

# Editor97 V2 repeat and node slot projection preservation

## Scope

The V2 Designer path projects a document into the legacy authoring model and reconstructs V2
nodes by stable node ID. The reconstruction already carries forward prior pixel-snapping and
state data for an existing node, but previously replaced its `repeat` with `None` and its
node-level `slots` with an empty map. An unrelated Designer edit could therefore erase these
fields when the rebuilt document was serialized and saved.

This slice carries `repeat` and node-level `slots` from the prior V2 node with the same ID. A
new node keeps the V2 defaults, and a node removed from the Designer tree is not resurrected.
The regression loads a V2 document with a reachable repeated node and node-level slot value,
edits another projected value, serializes through the production V2 projection function, and
reloads it through `UiZuiAssetLoader`. A second regression removes a projected node and checks
that serialization does not restore it from the prior document.

## Validation and remaining gates

The focused Rust regressions are pending managed Cargo execution. Edition 2021 exact rustfmt and
scoped diff checks passed on the final Rust source. No live Designer Save/reopen, crash,
performance, or product acceptance run has been performed for this slice.

Editor97 P0-01 remains open. ThemeTokens kind, unreachable prior nodes, unknown fields and TOML
trivia, source-span fidelity, accessibility/navigation schema, and the full V2 golden corpus are
not covered by this field-specific preservation. The canonical Editor97 plan explicitly forbids
claiming a lossless V2 document contract from piecemeal legacy projection fixes.
