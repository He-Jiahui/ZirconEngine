---
related_code:
  - zircon_runtime/src/ui/text/mod.rs
  - zircon_runtime/src/ui/text/shaper.rs
  - zircon_runtime/src/ui/text/layout_engine.rs
  - zircon_runtime/src/ui/text/grapheme.rs
  - zircon_runtime/src/ui/text/rich_text.rs
  - zircon_runtime/src/ui/text/hit_test.rs
  - zircon_runtime/src/ui/text/edit_state.rs
  - zircon_runtime/src/ui/tests/text_shaper.rs
  - zircon_runtime/src/ui/tests/text_layout.rs
  - zircon_runtime/src/ui/tests/text_hit_testing.rs
  - zircon_runtime_interface/src/ui/surface/render/text_layout.rs
  - zircon_runtime_interface/src/ui/surface/render/text_shape.rs
implementation_files:
  - zircon_runtime/src/ui/text/mod.rs
  - zircon_runtime/src/ui/text/shaper.rs
  - zircon_runtime/src/ui/text/layout_engine.rs
  - zircon_runtime/src/ui/tests/text_shaper.rs
plan_sources:
  - .codex/plans/ZirconEngine UITextInputA11y 缺口收束计划.md
  - dev/bevy/crates/bevy_text/src/lib.rs
  - dev/bevy/crates/bevy_ui_render/src/lib.rs
tests:
  - rustfmt --edition 2021 --config skip_children=true --check zircon_runtime/src/ui/text/mod.rs zircon_runtime/src/ui/text/layout_engine.rs zircon_runtime/src/ui/text/shaper.rs zircon_runtime/src/ui/tests/text_shaper.rs zircon_runtime/src/ui/tests/mod.rs (2026-05-23: passed after UiTextShaper boundary addition)
  - cargo test -p zircon_runtime --lib text_shaper --offline --jobs 1 --target-dir D:\cargo-targets\zircon-text-shaper-20260523 --message-format short --color never (2026-05-23: deferred while unrelated Cargo/rustc processes were active)
doc_type: module-detail
---

# Runtime UI Text

## Purpose

Runtime UI text owns the shared layout, edit, hit-test, and shaping boundary used by render extraction, TextInput behavior, accessibility text geometry, and future SDF/native text backends.

The current backend is intentionally heuristic. It preserves Zircon's existing behavior for grapheme-aware wrapping, rich-run ranges, low-fidelity bidirectional visual order, ellipsis, measurement, caret/selection geometry, and hit testing. It is now accessed through `UiTextShaper` instead of making the public `layout_text(...)` entry point call `layout_engine` directly.

## Shaper Boundary

`UiTextShapeRequest` carries the source text, resolved style, layout frame, and optional clip frame. `UiTextShaper` provides two operations:

- `shape_text(...)` returns `UiResolvedTextLayout`, the shared geometry DTO consumed by render extract and hit testing.
- `measure_text(...)` returns `UiSize`, the same measurement used by layout callbacks.

`UiHeuristicTextShaper` is the current default implementation. It delegates to `layout_engine.rs`, so this slice changes ownership boundaries without changing text geometry. Future Parley/Swash/HarfBuzz integration should implement the same trait and preserve the `UiResolvedTextLayout` contract before diverging into native or SDF raster/cache backends.

## Current Guarantees

`layout_text(...)` now routes through `UiHeuristicTextShaper`, while `measure_text_size(...)` routes through the same backend. This keeps layout measurement and render extraction on one source of text geometry.

The existing `layout_engine` still handles grapheme cluster wrapping, rich text run splitting, ellipsis range preservation, and the low-fidelity BiDi scaffold. `text_hit_testing.rs` consumes the resolved layout rather than re-estimating line geometry.

## Validation

`text_shaper.rs` focused tests assert that the default shaper returns the same layout as the public `layout_text(...)` entry point and the same measurement as `measure_text_size(...)`, including a combining-mark ellipsis case. Runtime Cargo execution is still deferred when unrelated Cargo/Rust compiler processes are active in the shared checkout.
