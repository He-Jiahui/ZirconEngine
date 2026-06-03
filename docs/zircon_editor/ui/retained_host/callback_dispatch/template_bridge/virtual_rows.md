---
related_code:
  - zircon_runtime_interface/src/ui/v2/repeat.rs
  - zircon_runtime_interface/src/ui/v2/asset.rs
  - zircon_runtime_interface/src/ui/v2/arena.rs
  - zircon_runtime/src/ui/v2/surface_tree/node.rs
  - zircon_editor/assets/ui/editor/components/workbench_scene_tree_panel.zui
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/mod.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/virtual_rows.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/scene_tree_rows.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/data_sync.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/error.rs
  - zircon_runtime/src/ui/surface/node_pool.rs
  - zircon_runtime/src/ui/surface/surface.rs
  - zircon_editor/src/tests/host/retained_callback_dispatch/template_bridge/workbench_projection.rs
implementation_files:
  - zircon_runtime_interface/src/ui/v2/repeat.rs
  - zircon_runtime_interface/src/ui/v2/asset.rs
  - zircon_runtime_interface/src/ui/v2/arena.rs
  - zircon_runtime/src/ui/v2/surface_tree/node.rs
  - zircon_editor/assets/ui/editor/components/workbench_scene_tree_panel.zui
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/virtual_rows.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/scene_tree_rows.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/data_sync.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/error.rs
plan_sources:
  - user: 2026-06-01 Continue approximating the zirconEngine editor from base rendering, input response, Taffy layout, and declared component composition
  - docs/ui-and-layout/workbench.png
tests:
  - zircon_editor/src/tests/host/retained_callback_dispatch/template_bridge/workbench_projection.rs
  - cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never with CARGO_TARGET_DIR=D:\cargo-targets\zircon-editor-workbench-reference-sync-1208 and RUSTFLAGS=-Awarnings (2026-06-01 after helper extraction: passed)
  - cargo test -p zircon_editor --lib componentized_workbench_scene_tree_grows_and_reuses_virtual_rows_for_live_snapshot_state --locked --jobs 1 --message-format short --color never -- --nocapture with CARGO_TARGET_DIR=D:\cargo-targets\zircon-editor-workbench-virtual-rows-1435 and RUSTFLAGS=-Awarnings (2026-06-01 after helper extraction: passed, 1 passed)
  - cargo test -p zircon_editor --lib componentized_workbench --locked --jobs 1 --message-format short --color never -- --nocapture with CARGO_TARGET_DIR=D:\cargo-targets\zircon-editor-workbench-virtual-rows-1435 and RUSTFLAGS=-Awarnings (2026-06-01 after helper extraction: passed, 24 passed)
  - cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never with CARGO_TARGET_DIR=D:\cargo-targets\zircon-editor-workbench-repeat-decl-1603 and RUSTFLAGS=-Awarnings (2026-06-01 after repeat declaration integration: passed)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --message-format short --color never with CARGO_TARGET_DIR=D:\cargo-targets\zircon-editor-workbench-repeat-validation-1755 and RUSTFLAGS=-Awarnings (2026-06-01 after repeat validation/metadata constants: passed)
  - cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never with CARGO_TARGET_DIR=D:\cargo-targets\zircon-editor-workbench-repeat-validation-1755 and RUSTFLAGS=-Awarnings (2026-06-01 after shared repeat constants in bridge parsing: first run timed out after 604 seconds with no compiler diagnostic; rerun after runtime cache completed passed)
  - cargo test -p zircon_runtime --lib repeat_declaration --locked --jobs 1 --message-format short --color never -- --nocapture with CARGO_TARGET_DIR=D:\cargo-targets\zircon-editor-workbench-repeat-validation-1755 and RUSTFLAGS=-Awarnings (2026-06-01 after repeat validation/metadata constants: timed out after 604 seconds while building/linking test binary; no compiler diagnostic returned)
  - cargo test -p zircon_editor --lib componentized_workbench_scene_tree_grows_and_reuses_virtual_rows_for_live_snapshot_state --locked --jobs 1 --message-format short --color never -- --nocapture with CARGO_TARGET_DIR=D:\cargo-targets\zircon-editor-workbench-repeat-decl-1603 and RUSTFLAGS=-Awarnings (2026-06-01 after repeat declaration integration: timed out after 605 seconds during zircon_runtime test binary link; no compiler diagnostic returned)
  - rustfmt --edition 2021 --check over the touched repeat/runtime/editor Rust files (2026-06-01 after repeat declaration integration: passed)
  - Python tomllib parse of zircon_editor/assets/ui/editor/components/workbench_scene_tree_panel.zui (2026-06-01 after repeat declaration integration: passed, repeat table present and 10 authored scene-tree children)
doc_type: module-detail
---

# Template Bridge Virtual Rows

## Purpose

`virtual_rows.rs` is the retained-template bridge utility for overflow row projection. It is the first reusable step between fixed authored component rows and a future declarative repeat or virtual-list primitive. The helper lets a bridge keep a small set of authored rows for pixel-stable first-screen structure, then synthesize additional rows from an authored prototype when live data grows beyond that baseline.

The current consumer is the componentized workbench scene tree. Rows 1-10 remain authored in `workbench_scene_tree_panel.zui`; the `WorkbenchSceneTree` node declares `repeat = { kind = "virtual_rows", ... }`; rows 11 and above are created as `WorkbenchSceneVirtualItemNN` nodes by `scene_tree_rows.rs` through `TemplateBridgeVirtualRowSequence`.

## Behavior

`TemplateBridgeVirtualRowSequence` stores the parent control id, prototype control id, generated control-id prefix, authored row count, and node-path namespace. The sequence can still be constructed directly for focused bridge code, but workbench integration now uses `TemplateBridgeVirtualRowSequence::from_surface_repeat(...)` to read those fields from the built surface's `repeat` metadata. The metadata key and field names come from the shared `zircon_runtime_interface::ui::v2::UI_V2_REPEAT_*` constants, so bridge parsing stays aligned with runtime projection.

`reconcile(...)` receives the current live row count and performs two operations:

- It prunes generated rows whose row number is now beyond the required live count by calling `UiSurface::detach_subtree_to_pool(...)`.
- It ensures every required generated row exists by cloning the authored prototype, assigning a stable generated control id and node path, applying caller-owned metadata customization, and inserting it with `UiSurface::insert_or_reuse_pooled_child(...)`.

The node-pool key comes from the generated node's component, control id, and node path. Generated control ids are therefore deterministic, so a row removed during shrink can be reused when the same row number comes back later. Slot metadata is cloned from the authored prototype slot so linear, grid, or future parent-owned placement settings follow the authored row shape instead of being guessed by the helper.

## Workbench Integration

`scene_tree_rows.rs` owns only workbench-specific policy: the authored row list, default tree-row attributes, and click binding. The generated prefix, row prototype, authored count, and path namespace come from the declarative `repeat` table on `WorkbenchSceneTree`. The bridge asks the helper for virtual control ids when syncing live `SceneEntry` data and when resolving whether a clicked control belongs to the scene tree.

Dynamic workbench rows reuse an authored scene-row binding. This keeps retained host projection route resolution inside the source document's known binding table while the generated row still exposes its own control id and `scene_node_id` state.

## Follow-up

This helper is still a bridge-level materializer, not a runtime-owned virtual-list engine. The schema can now declare prototype rows, but the retained workbench bridge still decides the live row count and writes row-specific metadata. The next layer should move the materialization policy deeper into reusable list, tree, and table components so data sources can drive rows without hand-wiring a bridge helper per surface.
