---
related_code:
  - zircon_runtime_interface/src/ui/surface/render/text_geometry/source_map/
  - zircon_runtime/src/ui/text/hit_test.rs
  - zircon_runtime/src/ui/text/layout_engine.rs
  - zircon_runtime/src/ui/text/grapheme.rs
  - zircon_runtime/src/ui/surface/input/text_pointer.rs
  - zircon_runtime/src/ui/tests/text_hit_testing.rs
  - zircon_runtime/src/ui/tests/widget_text_input_pointer.rs
implementation_files:
  - zircon_runtime_interface/src/ui/surface/render/text_geometry/source_map/
  - zircon_runtime/src/ui/text/hit_test.rs
  - zircon_runtime/src/ui/text/layout_engine.rs
  - zircon_runtime/src/ui/surface/input/text_pointer.rs
plan_sources:
  - .codex/plans/ZirconEngine UITextInputA11y 缺口收束计划.md
  - user: 2026-05-21 implement ZirconEngine UI/Text/Input/A11y gap closure plan
  - dev/bevy/crates/bevy_ui_widgets/src/editable_text.rs
  - dev/bevy/crates/bevy_text/src/text_edit.rs
  - docs/plans/zircon_runtime/text/03-line-breaking-measure-and-layout.md
  - docs/plans/engine-code-structure-convention.md
tests:
  - zircon_runtime/src/ui/tests/text_hit_testing.rs
  - zircon_runtime/src/ui/tests/widget_text_input_pointer.rs
  - zircon_runtime_interface/src/ui/surface/render/text_geometry/source_map/tests.rs
  - zircon_runtime/src/ui/text/geometry/tests/mixed_bidi.rs
  - rustfmt --edition 2021 --check zircon_runtime/src/ui/text/hit_test.rs zircon_runtime/src/ui/text/layout_engine.rs zircon_runtime/src/ui/text/mod.rs zircon_runtime/src/ui/tests/text_hit_testing.rs zircon_runtime/src/ui/tests/mod.rs
  - git diff --check -- zircon_runtime/src/ui/text/hit_test.rs zircon_runtime/src/ui/text/layout_engine.rs zircon_runtime/src/ui/text/mod.rs zircon_runtime/src/ui/tests/text_hit_testing.rs zircon_runtime/src/ui/tests/mod.rs
doc_type: module-detail
---

# Runtime UI Text Hit Testing

`hit_test.rs` maps a point in surface text space back to a source byte caret by consuming `UiResolvedTextLayout`. The important boundary is that hit testing does not re-read raw attributes, recompute wrapping, or rebuild source clusters. It uses the same resolved line frames and advances as rendering, then delegates visual-edge to source/affinity mapping to the neutral `UiTextLineSourceMap` owner.

This is the geometry foundation for TextInput pointer placement and drag selection. Bevy's editable text widget first transforms pointer press/drag into local text coordinates, then queues `MoveToPoint`, `ShiftClickExtension`, or `ExtendSelectionToPoint`. Zircon now consumes the helper from `surface/input/text_pointer.rs`, so primary press and captured pointer move share the same point-to-caret conversion instead of hiding geometry in widget behavior.

## Behavior

The helper first prefers the full resolved line frame, which is required when rich-table cells share one block coordinate. Horizontal fallback chooses the nearest y line; `VerticalRl` fallback chooses the nearest right-to-left column by x. It then converts the inline-axis point into a visual grapheme boundary using `glyph_advances` (or shaped fallback widths when the DTO is incomplete). `UiTextLineSourceMap` maps that boundary to the authored byte offset and direction-correct affinity: an RTL visual leading edge resolves to logical end/Downstream, and its visual trailing edge resolves to logical start/Upstream.

The result also carries line index, visual grapheme index, caret affinity, and whether the point fell inside the chosen line frame. Those fields let future pointer selection distinguish direct text hits from clamped edge hits without changing the source-offset contract.

## Limits

The map is only as precise as the resolved run contract. Source-isomorphic grapheme runs retain exact byte ranges; a non-isomorphic generated or inline run is conservatively represented by its whole source range. Full backend glyph-to-source reverse maps for ligature interiors remain a later shaping DTO extension, but consumers must extend the shared map rather than reintroducing a local fallback mapper.

## Tests

`text_hit_testing.rs` covers grapheme midpoint selection, tabs, multiline routing, aligned frames, VerticalRl columns and both visual edges of an RTL cluster inside a mixed line. `text_geometry/source_map/tests.rs` covers the inverse caret mapping and discontiguous mixed-BiDi selection spans. `widget_text_input_pointer.rs` covers press/Shift+press/drag selection. Exact execution and product framebuffer evidence are recorded by the owning Text03 child-plan output after its testing stage.
