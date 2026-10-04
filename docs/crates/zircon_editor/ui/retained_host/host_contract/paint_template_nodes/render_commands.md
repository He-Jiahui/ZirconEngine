---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/render_commands.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/render_commands/command.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/render_commands/draw.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/render_command_conversion.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/render_command_conversion/brush.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/render_command_conversion/image.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/render_command_conversion/style.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/render_command_conversion/text.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_node_pipeline.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_primitives.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_primitives/image.rs
implementation_files:
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/render_commands.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/render_commands/command.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/render_commands/draw.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/render_command_conversion.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/render_command_conversion/brush.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/render_command_conversion/image.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/render_command_conversion/style.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/render_command_conversion/text.rs
tests:
  - cargo fmt -p zircon_editor --check
  - render command subtree ownership scan
  - render command conversion subtree ownership scan
  - touched-file whitespace scan
  - scoped git diff --check
  - cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never
plan_sources:
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
doc_type: module-overview
---

# Render Commands

`paint_template_nodes/render_commands.rs` is the retained-host software replay entry for template render commands. It converts runtime `UiRenderCommand` values through `render_command_conversion.rs`, then delegates ordered host paint command playback. The root is intentionally small: it declares the folder-backed subtree, re-exports `HostPaintCommand` and `draw_host_paint_commands` at the existing paint-template visibility, and keeps the test helper entry for runtime command replay.

`paint_template_nodes/render_commands/command.rs` owns the host-side command DTO: `HostPaintCommand`, the internal command kind, command constructors, default image placeholder border, text style payload storage, optional RGBA image payloads, opacity, z-index, and clip/frame data. Template primitive modules continue to construct commands through the same `HostPaintCommand::{quad,text,group,image,image_pixels}` API.

`paint_template_nodes/render_commands/draw.rs` owns ordered playback: stable z-index sorting, per-kind dispatch, quad rounded/flat fill and border emission, text emission, image atlas/resource-key fallback, image placeholder color generation, opacity application, and border-width expansion. This module is still part of the retained software bridge and remains a migration target while M3.S2 converges `paint_template_nodes/` toward runtime extract/GPU command stream ownership.

`paint_template_nodes/render_command_conversion.rs` owns the runtime-to-host traversal entry. It walks `UiRenderCommand`, applies command/parent clip inheritance, filters invisible or fully transparent elements, and dispatches payloads to folder-backed children. `render_command_conversion/brush.rs` owns brush fill/border/resource extraction, `image.rs` owns visual asset resolution and image fallback command construction, `text.rs` owns text run/shaped/source fallback conversion plus decoration layering, and `style.rs` owns UI-frame conversion, foreground fallback, alignment, and hex color parsing.

The 2026-06-18 render-command subtree split reduced `render_commands.rs` from 404 lines to 26 lines and created `render_commands/command.rs` at 180 lines plus `render_commands/draw.rs` at 210 lines. Validation used `cargo fmt -p zircon_editor --check`, a render command subtree ownership scan, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only after widening the `draw_host_paint_commands` re-export to match the existing `template_node_pipeline.rs` caller. Full Cargo test expansion remains deferred to the milestone testing stage per the current implementation cadence.

The 2026-06-18 render-command conversion subtree split reduced `render_command_conversion.rs` from 345 lines to 86 lines and created `render_command_conversion/brush.rs` at 29 lines, `image.rs` at 64 lines, `style.rs` at 69 lines, and `text.rs` at 116 lines. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only. Full Cargo test expansion remains deferred to the milestone testing stage per the current implementation cadence.
