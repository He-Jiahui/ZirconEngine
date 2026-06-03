---
related_code:
  - zircon_runtime_interface/src/ui/v2/repeat.rs
  - zircon_runtime_interface/src/ui/v2/asset.rs
  - zircon_runtime_interface/src/ui/v2/arena.rs
  - zircon_runtime_interface/src/ui/v2/mod.rs
  - zircon_runtime/src/ui/v2/compiler.rs
  - zircon_runtime/src/ui/v2/component_instancer.rs
  - zircon_runtime/src/ui/v2/surface_tree/node.rs
  - zircon_runtime/src/ui/tests/v2_asset.rs
  - zircon_editor/assets/ui/editor/components/workbench_scene_tree_panel.zui
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/virtual_rows.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/scene_tree_rows.rs
implementation_files:
  - zircon_runtime_interface/src/ui/v2/repeat.rs
  - zircon_runtime_interface/src/ui/v2/asset.rs
  - zircon_runtime_interface/src/ui/v2/arena.rs
  - zircon_runtime_interface/src/ui/v2/mod.rs
  - zircon_runtime/src/ui/v2/compiler.rs
  - zircon_runtime/src/ui/v2/component_instancer.rs
  - zircon_runtime/src/ui/v2/surface_tree/node.rs
  - zircon_editor/assets/ui/editor/components/workbench_scene_tree_panel.zui
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/virtual_rows.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/scene_tree_rows.rs
plan_sources:
  - user: 2026-06-01 Continue approximating the zirconEngine editor from base rendering, input response, Taffy layout, and declared component composition
  - docs/ui-and-layout/workbench.png
tests:
  - zircon_runtime/src/ui/tests/v2_asset.rs
  - zircon_editor/src/tests/host/retained_callback_dispatch/template_bridge/workbench_projection.rs
  - cargo check -p zircon_runtime_interface --lib --locked --jobs 1 --message-format short --color never with CARGO_TARGET_DIR=D:\cargo-targets\zircon-editor-workbench-repeat-decl-1603 and RUSTFLAGS=-Awarnings (2026-06-01 after repeat declaration type: passed)
  - cargo check -p zircon_runtime_interface --lib --locked --jobs 1 --message-format short --color never with CARGO_TARGET_DIR=D:\cargo-targets\zircon-editor-workbench-repeat-validation-1755 and RUSTFLAGS=-Awarnings (2026-06-01 after repeat validation/metadata constants: passed)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --message-format short --color never with CARGO_TARGET_DIR=D:\cargo-targets\zircon-editor-workbench-repeat-validation-1755 and RUSTFLAGS=-Awarnings (2026-06-01 after repeat validation/metadata constants: passed)
  - cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never with CARGO_TARGET_DIR=D:\cargo-targets\zircon-editor-workbench-repeat-validation-1755 and RUSTFLAGS=-Awarnings (2026-06-01 after repeat validation/metadata constants: first run timed out after 604 seconds with no compiler diagnostic; rerun after runtime cache completed passed)
  - cargo test -p zircon_runtime --lib repeat_declaration --locked --jobs 1 --message-format short --color never -- --nocapture with CARGO_TARGET_DIR=D:\cargo-targets\zircon-editor-workbench-repeat-validation-1755 and RUSTFLAGS=-Awarnings (2026-06-01 after repeat validation/metadata constants: timed out after 604 seconds while building/linking test binary; no compiler diagnostic returned)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --message-format short --color never with CARGO_TARGET_DIR=D:\cargo-targets\zircon-editor-workbench-repeat-decl-1603 and RUSTFLAGS=-Awarnings (2026-06-01 during repeat declaration: timed out after 304 seconds while other Cargo jobs were active; no compiler diagnostic returned)
  - cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never with CARGO_TARGET_DIR=D:\cargo-targets\zircon-editor-workbench-repeat-decl-1603 and RUSTFLAGS=-Awarnings (2026-06-01 after repeat declaration: passed)
  - cargo test -p zircon_runtime --lib ui_v2_repeat_declaration_is_preserved_in_compiled_surface_metadata --locked --jobs 1 --message-format short --color never -- --nocapture with CARGO_TARGET_DIR=D:\cargo-targets\zircon-editor-workbench-repeat-decl-1603 and RUSTFLAGS=-Awarnings (2026-06-01: timed out after 604 seconds during zircon_runtime test binary link; no compiler diagnostic returned)
  - cargo test -p zircon_editor --lib componentized_workbench_scene_tree_grows_and_reuses_virtual_rows_for_live_snapshot_state --locked --jobs 1 --message-format short --color never -- --nocapture with CARGO_TARGET_DIR=D:\cargo-targets\zircon-editor-workbench-repeat-decl-1603 and RUSTFLAGS=-Awarnings (2026-06-01: timed out after 605 seconds during zircon_runtime test binary link; no compiler diagnostic returned)
  - rustfmt --edition 2021 --check over the touched repeat/runtime/editor Rust files (2026-06-01: passed)
  - Python tomllib parse of zircon_editor/assets/ui/editor/components/workbench_scene_tree_panel.zui (2026-06-01: passed, repeat table present and 10 authored scene-tree children)
doc_type: module-detail
---

# UI v2 Repeat Declarations

## Purpose

`UiV2Repeat` is the first schema-level declaration for template-authored repeated UI regions. It lets a v2 node state that overflow rows should be generated from an authored prototype, while keeping the normal authored children in place for first-screen layout stability.

The initial supported repeat kind is `virtual_rows`. It is intentionally narrow: it describes a parent row container, an authored prototype control id, the generated control-id prefix, the authored row count, and the node-path namespace for generated rows. The workbench scene tree uses this to keep ten authored hierarchy rows in `workbench_scene_tree_panel.zui` and generate rows 11+ as retained surface nodes when live scene data exceeds the authored baseline.

## Data Flow

Template assets declare repeat metadata beside regular node fields:

```toml
repeat = { kind = "virtual_rows", prototype = "WorkbenchSceneSlot10Item", virtual_control_prefix = "WorkbenchSceneVirtualItem", authored_count = 10, node_path_namespace = "v2" }
```

`UiV2NodeDefinition` parses the field into `UiV2Repeat`. `UiV2DocumentCompiler` validates reachable repeat declarations before arena construction, then copies valid declarations into `UiV2ArenaNode`. `UiV2ComponentInstancer` preserves or applies repeat metadata when component roots are expanded from component mounts. `surface_tree::node` then serializes the declaration through `UiV2Repeat::metadata_value()` into `UiTemplateNodeMetadata.attributes["repeat"]` so retained-host code can read the declaration from the built `UiSurface` without reloading the source asset.

The runtime layer does not yet synthesize repeated rows by itself. For this slice, it preserves the declaration through compile and projection. The editor workbench bridge consumes that preserved metadata through `TemplateBridgeVirtualRowSequence::from_surface_repeat(...)`.

## Constraints

The `virtual_rows` kind expects all fields to be explicit and non-empty except `node_path_namespace`, which may be empty. `authored_count` must be greater than zero because the current materializer clones from an authored prototype row. Generated control ids remain deterministic by combining `virtual_control_prefix` with a two-digit row number, so removed rows can be detached into the `UiSurface` node pool and reused when the row count grows again.

Field names are exported from `repeat.rs` as `UI_V2_REPEAT_ATTRIBUTE` and `UI_V2_REPEAT_FIELD_*` constants. Runtime surface projection and editor-side bridge parsing use those constants rather than duplicating string literals.

The declaration is stored outside `props` on purpose. Repeated-row intent is structural template metadata, not a widget property, and it needs to survive style resolution without being confused with a renderable attribute such as text, selected, or visibility.

## Test Coverage

`ui_v2_repeat_declaration_is_preserved_in_compiled_surface_metadata` verifies that TOML repeat syntax parses, survives compilation into the arena node, and appears in surface template metadata.

`ui_v2_rejects_invalid_repeat_declaration_before_surface_build` verifies that invalid repeat declarations fail during v2 compilation before a surface or bridge materializer can consume them.

`componentized_workbench_scene_tree_grows_and_reuses_virtual_rows_for_live_snapshot_state` verifies the editor-side consumer: the workbench scene tree exposes the declared repeat table, creates virtual rows beyond the authored ten-row baseline, resolves selection bindings for generated rows, detaches rows when the scene list shrinks, and reuses pooled rows when it grows again.
