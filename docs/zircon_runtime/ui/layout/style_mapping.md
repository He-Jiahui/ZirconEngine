---
related_code:
  - zircon_runtime/src/ui/layout/mod.rs
  - zircon_runtime/src/ui/layout/style_mapping.rs
  - zircon_runtime/src/ui/layout/taffy_bridge.rs
  - zircon_runtime/src/ui/layout/pass/taffy_arrange.rs
  - zircon_runtime_interface/src/ui/layout/style.rs
  - zircon_runtime_interface/src/ui/layout/debug.rs
  - zircon_runtime_interface/src/ui/layout/mod.rs
  - zircon_runtime_interface/src/tests/ui_layout.rs
  - zircon_runtime/src/ui/tests/style_mapping.rs
  - zircon_runtime/src/ui/tests/taffy_bridge.rs
implementation_files:
  - zircon_runtime/src/ui/layout/mod.rs
  - zircon_runtime/src/ui/layout/style_mapping.rs
  - zircon_runtime/src/ui/layout/taffy_bridge.rs
  - zircon_runtime_interface/src/ui/layout/style.rs
  - zircon_runtime_interface/src/ui/layout/debug.rs
  - zircon_runtime_interface/src/ui/layout/mod.rs
plan_sources:
  - user: 2026-06-12 implement editor UI architecture from docs/plans/zircon_editor/editor_ui
  - docs/plans/zircon_editor/editor_ui/index.md
  - docs/plans/zircon_editor/editor_ui/02-layout-taffy-and-containers.md
tests:
  - zircon_runtime/src/ui/tests/style_mapping.rs
  - zircon_runtime_interface/src/tests/ui_layout.rs
  - 2026-06-12: cargo check -p zircon_runtime_interface --lib --locked (passed)
  - 2026-06-12: cargo test -p zircon_runtime_interface --lib ui_layout_style_and_debug_packet_contracts_round_trip_with_defaults --locked --target-dir target/codex-editor-ui (passed)
  - 2026-06-12: cargo test -p zircon_runtime --lib style_mapping --locked --jobs 1 --target-dir target/codex-editor-ui-runtime --message-format short --color never -- --nocapture --test-threads=1 (passed, 2 passed)
doc_type: module-detail
---

# UI Layout Style Mapping

`zircon_runtime_interface::ui::layout::UiLayoutStyle` is the dependency-free layout style DTO for the editor UI architecture. It records the CSS-like subset that can be shared by `.zui` assets, runtime surfaces, editor diagnostics, and future style/theme layers without exposing Taffy types through the interface crate.

`zircon_runtime::ui::layout::style_mapping` is the runtime conversion boundary. It turns `UiLayoutStyle` into `taffy::style::Style` only for Taffy-owned families and returns `UiLayoutEngineFallbackReason` when values are invalid or when Zircon-owned layout semantics must stay in the retained runtime pass.

## Mapped Style Fields

The mapper currently covers the Stage A layout contract:

- display: flex, grid, block, none
- flex direction and wrap
- justify, align-items, align-self, align-content
- row and column gaps
- flex grow, shrink, and basis
- grid template rows/columns and row/column placement
- size, min-size, max-size, and aspect ratio
- margin, padding, position, inset, and overflow

`UiDimension::Px` and `UiDimension::Percent` map to Taffy length/percent values after finite non-negative validation. `UiDimension::Auto` maps to auto-capable Taffy fields, but is rejected for fields such as gap and padding that require concrete length/percentage values.

## Runtime Boundary

`taffy_style_for_container` now delegates through the new style DTO path:

1. infer a `UiLayoutStyle` from `UiContainerKind` and `BoxConstraints`;
2. validate legacy numeric inputs;
3. map the style DTO into Taffy;
4. preserve `None` for Zircon-owned containers or invalid inputs.

This keeps the old runtime bridge API stable while establishing the style DTO as the single future expansion point for editor layout assets, style sheets, debug packets, and Taffy diagnostics.

## Zircon-Owned Layout

`UiLayoutDisplay::Overlay`, `Canvas`, `Scroll`, and `Virtual` return `UiLayoutEngineFallbackReason::ZirconOwnedSemantics`. These families require retained surface ownership for z ordering, anchor/pivot placement, scroll windows, virtual ranges, hit-grid authority, and frame sharing. They can appear in diagnostics, but the runtime must not silently reinterpret them as generic Taffy flex/grid/block layout.

## Diagnostics

`UiLayoutDebugPacket` and `UiLayoutDebugNode` are neutral debug DTOs for future editor reflector work. A packet records the frame index, engine selection report, and node rows with geometry, constraints, optional backend, optional fallback reason, and style sources. The DTO is interface-only; runtime/editor slices decide how to populate and display it.
