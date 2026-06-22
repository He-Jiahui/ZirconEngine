---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/mod.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_node_pipeline.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_node_pipeline/clip.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_node_pipeline/draw.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_node_pipeline/test_support.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_nodes.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_nodes/commands.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_nodes/fallback.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_nodes/geometry.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_nodes/ordering.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_nodes/specialized.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/render_command_conversion.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/render_commands.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/mod.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/exports.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/docks/pane/template_nodes.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/docks/pane/template_nodes/selection.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/scene_layers/overlay.rs
implementation_files:
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/mod.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_node_pipeline.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_node_pipeline/clip.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_node_pipeline/draw.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_node_pipeline/test_support.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_nodes.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_nodes/commands.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_nodes/fallback.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_nodes/geometry.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_nodes/ordering.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_nodes/specialized.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/render_command_conversion.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/render_commands.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/mod.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/exports.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/docks/pane/template_nodes/selection.rs
plan_sources:
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - user: 2026-06-18 editor UI architecture implementation, feature first and tests deferred
tests:
  - cargo fmt -p zircon_editor
  - cargo fmt -p zircon_editor --check
  - paint-template-nodes root re-export ownership scan
  - scoped trailing-whitespace scan
  - scoped git diff --check
doc_type: module-detail
---

# Paint Template Nodes

`paint_template_nodes/mod.rs` is the retained-host template-node paint entry. It should stay as a structural module declaration surface plus stable re-exports for Workbench painters and test helpers.

`template_node_pipeline.rs` owns the public draw pipeline entry. Its `draw.rs` child iterates node models, applies clipping, orders runtime commands, and reports whether any visible node was painted. Its `clip.rs` child owns node clip resolution, and `test_support.rs` owns image-buffer helpers used by the template paint regression suite.

`template_nodes.rs` owns command emission for a single template node. The `template_nodes/` child owners keep command production separated into command orchestration, fallback rendering, frame geometry, ordering, and specialized component dispatch. Rendering DTO conversion remains in `render_command_conversion.rs`, while `render_commands.rs` owns the runtime command paint harness used by tests.

`style_selector/mod.rs` is now the structural Workbench style-selector entry. `style_selector/exports.rs` owns the selector re-export surface for the child style modules while each `workbench_*` child keeps the family-specific style resolution.

## Boundary Rules

- Keep `paint_template_nodes/mod.rs` limited to child declarations and re-exports.
- Keep draw iteration, clipping, and test image buffers in `template_node_pipeline/`; do not reintroduce wrapper functions in the root module.
- Keep per-node command construction in `template_nodes/` and low-level replay/test harness behavior in `render_commands.rs`.
- Keep Workbench pane, menu, dock, and overlay painters as consumers of `draw_template_nodes`/`has_template_nodes`; they should not reach into template-node internals.

## Validation Notes

The 2026-06-21 root re-export split reduced `paint_template_nodes/mod.rs` from 118 lines to an 85-line structural entry. The stable entries now re-export `draw_template_nodes`, `has_template_nodes`, `paint_template_nodes_for_test`, and `paint_template_nodes_for_test_with_background` directly from `template_node_pipeline`, while `paint_runtime_render_commands_for_test` remains re-exported from `render_commands`.

Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, a paint-template-nodes root re-export ownership scan, scoped trailing-whitespace scan, and scoped `git diff --check`; package-level Cargo check and full Cargo tests remain deferred per the user's feature-first instruction.

The 2026-06-21 style-selector export split reduced `paint_template_nodes/style_selector/mod.rs` from 101 lines to a 20-line structural declaration/re-export entry. `style_selector/exports.rs` owns the restricted selector export surface for state projection and Workbench family selectors. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, a style-selector export ownership scan, scoped trailing-whitespace scan, and scoped `git diff --check`; package-level Cargo check and full Cargo tests remain deferred per the user's feature-first instruction.
