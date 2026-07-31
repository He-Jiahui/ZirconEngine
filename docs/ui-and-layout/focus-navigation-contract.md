---
related_code:
  - zircon_runtime_interface/src/ui/focus.rs
  - zircon_runtime_interface/src/ui/navigation.rs
  - zircon_runtime_interface/src/ui/tree/node/ui_tree.rs
  - docs/plans/zircon_editor/editor_layout/19-focus-and-navigation-model.md
doc_type: architecture-contract
---

# Focus And Navigation Contract

This document defines the shared focus vocabulary and deterministic Tab-chain selection used by editor UI. It keeps focus state independent from pointer hover and does not create a second input dispatcher.

## S1 Contract

`UiFocusContract` retains the legacy `focusable` declaration and adds `UiFocusMode`:

| Mode | Pointer Focus | Tab Focus |
| --- | --- | --- |
| `none` | No | No |
| `click` | Yes | No |
| `all` | Yes | Yes |

Missing serialized `mode` values default to `all`, preserving existing focusable-node behavior. A node must also be enabled and render-visible to participate in a Tab chain.

`UiFocusCause` maps the source of a focus change to `UiFocusVisible`: navigation shows the keyboard focus indicator, while pointer, programmatic, and restore focus do not. This supplies the semantic distinction needed by the style owner without painting a focus ring in the input layer.

`focus_chain(&UiTree)` is the sole S1 Tab ordering helper. It traverses reachable roots in stable tree pre-order, ignores repeated or missing node links, and carries ancestor visibility into descendants. Authored enabled tab indices sort before unindexed candidates; ties retain pre-order. Nodes with `focus: none`, `focus: click`, disabled `UiTabIndex`, disabled state, or invisible ancestry are omitted.

`UiNavigationBoundary` defines the later dispatcher boundary policy: `escape`, `wrap`, `stop`, `explicit(node)`, and `trap`. It is a DTO only in S1; no direction-scoring or modal state machine is introduced here.

## Ownership Boundaries

S2 owns directional geometry scoring and explicit-neighbor resolution in the editor navigation dispatcher. S3 owns modal focus scope trapping, stable restoration, and the rendered `:focus-visible` ring. Pointer hit testing and capture remain owned by Layout18; the focus contract consumes their semantic focus cause rather than recreating pointer routing. Runtime Text owns caret, selection, and IME completion before focus transfer.
