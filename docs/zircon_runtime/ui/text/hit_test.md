---
related_code:
  - zircon_runtime/src/ui/text/hit_test.rs
  - zircon_runtime/src/ui/text/layout_engine.rs
  - zircon_runtime/src/ui/text/grapheme.rs
  - zircon_runtime/src/ui/surface/input/text_pointer.rs
  - zircon_runtime/src/ui/tests/text_hit_testing.rs
  - zircon_runtime/src/ui/tests/widget_text_input_pointer.rs
implementation_files:
  - zircon_runtime/src/ui/text/hit_test.rs
  - zircon_runtime/src/ui/text/layout_engine.rs
  - zircon_runtime/src/ui/surface/input/text_pointer.rs
plan_sources:
  - .codex/plans/ZirconEngine UITextInputA11y 缺口收束计划.md
  - user: 2026-05-21 implement ZirconEngine UI/Text/Input/A11y gap closure plan
  - dev/bevy/crates/bevy_ui_widgets/src/editable_text.rs
  - dev/bevy/crates/bevy_text/src/text_edit.rs
tests:
  - zircon_runtime/src/ui/tests/text_hit_testing.rs
  - zircon_runtime/src/ui/tests/widget_text_input_pointer.rs
  - rustfmt --edition 2021 --check zircon_runtime/src/ui/text/hit_test.rs zircon_runtime/src/ui/text/layout_engine.rs zircon_runtime/src/ui/text/mod.rs zircon_runtime/src/ui/tests/text_hit_testing.rs zircon_runtime/src/ui/tests/mod.rs
  - git diff --check -- zircon_runtime/src/ui/text/hit_test.rs zircon_runtime/src/ui/text/layout_engine.rs zircon_runtime/src/ui/text/mod.rs zircon_runtime/src/ui/tests/text_hit_testing.rs zircon_runtime/src/ui/tests/mod.rs
doc_type: module-detail
---

# Runtime UI Text Hit Testing

`hit_test.rs` maps a point in surface text space back to a source byte caret by consuming `UiResolvedTextLayout`. The important boundary is that hit testing does not re-read raw attributes, recompute wrapping independently, or invent a second measurement model. It uses the same resolved line frames, visual line text, source ranges, direction, and fixed advance produced by `layout_engine.rs`.

This is the geometry foundation for TextInput pointer placement and drag selection. Bevy's editable text widget first transforms pointer press/drag into local text coordinates, then queues `MoveToPoint`, `ShiftClickExtension`, or `ExtendSelectionToPoint`. Zircon now consumes the helper from `surface/input/text_pointer.rs`, so primary press and captured pointer move share the same point-to-caret conversion instead of hiding geometry in widget behavior.

## Behavior

The helper selects the nearest resolved line by y coordinate, clamping above the first line and below the last line. It then converts x into a visual grapheme index using the same fixed advance as the layout scaffold. LTR and mixed lines measure from `line.frame.x`; RTL lines measure from `line.frame.right()`. The resulting visual grapheme boundary is mapped through resolved runs to a source byte offset.

The result also carries line index, visual grapheme index, caret affinity, and whether the point fell inside the chosen line frame. Those fields let future pointer selection distinguish direct text hits from clamped edge hits without changing the source-offset contract.

## Limits

This is still pre-shaper geometry. It is correct for Zircon's current fixed-advance `UiResolvedTextLayout`, but it is not a substitute for HarfBuzz/Parley/Swash glyph cluster hit testing, font fallback, ligatures, or script-specific shaping. When `UiTextShaper` lands, this helper should continue to consume the shared shaped layout DTO rather than letting TextInput pointer handling fork a separate geometry path.

## Tests

`text_hit_testing.rs` covers grapheme midpoint selection, multiline y routing with x clamping, and right-aligned line frames. `widget_text_input_pointer.rs` covers primary press caret placement, Shift+press selection extension, captured drag selection, and empty TextInput layout fallback. Cargo execution is deferred during this implementation slice because unrelated Cargo/Rust compiler jobs are active in the shared checkout; the current local evidence is rustfmt plus scoped diff/whitespace checks.
