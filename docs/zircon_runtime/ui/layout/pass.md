---
related_code:
  - zircon_runtime_interface/src/ui/component/descriptor/slot_schema.rs
  - zircon_runtime_interface/src/ui/layout/slot.rs
  - zircon_runtime_interface/src/ui/layout/linear_sizing.rs
  - zircon_runtime_interface/src/ui/layout/scroll.rs
  - zircon_runtime_interface/src/ui/layout/engine.rs
  - zircon_runtime_interface/src/tests/ui_layout.rs
  - zircon_runtime_interface/src/tests/layout_engine_contracts.rs
  - zircon_runtime/src/ui/template/build/slot_contract.rs
  - zircon_runtime/src/ui/template/build/container_inference.rs
  - zircon_runtime/src/ui/template/build/parsers.rs
  - zircon_runtime/src/ui/layout/pass/mod.rs
  - zircon_runtime/src/ui/layout/pass/engine.rs
  - zircon_runtime/src/ui/layout/pass/incremental.rs
  - zircon_runtime/src/ui/layout/pass/slot.rs
  - zircon_runtime/src/ui/layout/pass/axis.rs
  - zircon_runtime/src/ui/layout/pass/measure.rs
  - zircon_runtime/src/ui/layout/pass/arrange.rs
  - zircon_runtime/src/ui/layout/pass/child_frame.rs
  - zircon_runtime/src/ui/layout/pass/taffy_arrange.rs
  - zircon_runtime/src/ui/layout/taffy_bridge.rs
  - zircon_runtime/src/ui/surface/diagnostics.rs
  - zircon_runtime/src/ui/surface/surface.rs
  - zircon_runtime/src/ui/surface/surface/rebuild.rs
  - zircon_runtime_interface/src/ui/surface/diagnostics.rs
  - zircon_runtime_interface/src/ui/surface/frame.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload_builders/runtime_diagnostics.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/runtime_diagnostics.rs
  - zircon_editor/src/ui/template_runtime/runtime/pane_payload_projection.rs
  - zircon_editor/src/ui/workbench/debug_reflector/mod.rs
  - zircon_editor/src/ui/workbench/debug_reflector/model.rs
  - zircon_editor/src/ui/workbench/debug_reflector/tests.rs
  - zircon_editor/src/tests/host/pane_presentation.rs
  - zircon_editor/src/tests/host/retained_window/ui_debug_reflector.rs
  - zircon_editor/src/tests/host/template_runtime/pane_payload_projection.rs
  - zircon_editor/src/tests/host/retained_callback_dispatch/template_bridge/layout_routes.rs
  - zircon_runtime/src/ui/tests/layout_slots.rs
  - zircon_runtime/src/ui/tests/template_grid_flow.rs
  - zircon_runtime/src/ui/tests/taffy_bridge.rs
  - zircon_runtime/src/ui/tests/taffy_layout_pass.rs
  - zircon_runtime/src/ui/tests/surface_frame_authority.rs
  - zircon_runtime/src/ui/tests/diagnostics.rs
  - zircon_runtime/src/ui/tests/runtime_ui_layout_routes.rs
  - zircon_runtime/src/ui/tests/surface_dirty_domains.rs
implementation_files:
  - zircon_runtime_interface/src/ui/layout/scroll.rs
  - zircon_runtime_interface/src/ui/layout/mod.rs
  - zircon_runtime_interface/src/ui/layout/engine.rs
  - zircon_runtime/src/ui/template/build/slot_contract.rs
  - zircon_runtime/src/ui/template/build/container_inference.rs
  - zircon_runtime/src/ui/template/build/parsers.rs
  - zircon_runtime/src/ui/layout/pass/slot.rs
  - zircon_runtime/src/ui/layout/pass/engine.rs
  - zircon_runtime/src/ui/layout/pass/incremental.rs
  - zircon_runtime/src/ui/layout/pass/axis.rs
  - zircon_runtime/src/ui/layout/pass/measure.rs
  - zircon_runtime/src/ui/layout/pass/arrange.rs
  - zircon_runtime/src/ui/layout/pass/child_frame.rs
  - zircon_runtime/src/ui/layout/pass/taffy_arrange.rs
  - zircon_runtime/src/ui/layout/taffy_bridge.rs
  - zircon_runtime/src/ui/surface/diagnostics.rs
  - zircon_runtime/src/ui/surface/surface.rs
  - zircon_runtime/src/ui/surface/surface/rebuild.rs
  - zircon_runtime_interface/src/ui/surface/diagnostics.rs
  - zircon_runtime_interface/src/ui/surface/frame.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload_builders/runtime_diagnostics.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/runtime_diagnostics.rs
  - zircon_editor/src/ui/template_runtime/runtime/pane_payload_projection.rs
  - zircon_editor/src/ui/workbench/debug_reflector/mod.rs
  - zircon_editor/src/ui/workbench/debug_reflector/model.rs
  - zircon_editor/src/ui/workbench/debug_reflector/tests.rs
  - zircon_editor/src/tests/host/pane_presentation.rs
  - zircon_editor/src/tests/host/retained_window/ui_debug_reflector.rs
  - zircon_editor/src/tests/host/template_runtime/pane_payload_projection.rs
  - zircon_editor/src/tests/host/retained_callback_dispatch/template_bridge/layout_routes.rs
  - zircon_runtime/src/ui/tests/layout_slots.rs
  - zircon_runtime/src/ui/tests/template_grid_flow.rs
  - zircon_runtime/src/ui/tests/taffy_bridge.rs
  - zircon_runtime/src/ui/tests/taffy_layout_pass.rs
  - zircon_runtime/src/ui/tests/surface_frame_authority.rs
  - zircon_runtime/src/ui/tests/diagnostics.rs
  - zircon_runtime/src/ui/tests/runtime_ui_layout_routes.rs
  - zircon_runtime/src/ui/tests/surface_dirty_domains.rs
  - zircon_runtime/src/ui/tests/template.rs
  - zircon_runtime_interface/src/tests/layout_engine_contracts.rs
plan_sources:
  - .codex/plans/Zircon UI 与 Unreal Slate 差异审计及后续里程碑.md
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Widgets/SPanel.h
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Widgets/SBoxPanel.h
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Layout/ArrangedWidget.h
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Layout/ArrangedChildren.h
tests:
  - rustfmt --edition 2021 --check zircon_runtime_interface/src/ui/layout/scroll.rs zircon_runtime_interface/src/ui/layout/mod.rs zircon_runtime_interface/src/ui/layout/slot.rs zircon_runtime/src/ui/layout/pass/slot.rs zircon_runtime/src/ui/layout/pass/arrange.rs zircon_runtime/src/ui/layout/pass/measure.rs zircon_runtime/src/ui/template/build/container_inference.rs zircon_runtime/src/ui/template/build/parsers.rs zircon_runtime/src/ui/template/build/slot_contract.rs zircon_runtime/src/ui/tests/mod.rs zircon_runtime/src/ui/tests/layout_slots.rs zircon_runtime/src/ui/tests/template_grid_flow.rs zircon_runtime/src/ui/tests/surface_dirty_domains.rs
  - cargo test -p zircon_runtime --lib layout_slots --locked --jobs 1 --target-dir D:\cargo-targets\zircon-m1-slot-panel --message-format short --color never -- --nocapture
  - cargo test -p zircon_runtime --lib template_grid_flow --locked --jobs 1 --target-dir D:\cargo-targets\zircon-m1-slot-panel --message-format short --color never -- --nocapture
  - cargo test -p zircon_runtime --lib taffy_bridge --locked --target-dir target/codex-shared-b (2026-05-11: passed, 2 passed)
  - cargo test -p zircon_runtime taffy -- --nocapture (2026-05-11: passed, 5 passed)
  - cargo test -p zircon_runtime size_box_contain_aspect_ratio_stays_zircon_owned -- --nocapture (2026-05-11: passed, 1 passed)
  - cargo test -p zircon_runtime taffy -- --nocapture (2026-05-11: passed, 6 passed)
  - D:\cargo-targets\zircon-layout-impl\debug\deps\zircon_runtime-eb36ca4a90c1b648.exe taffy_layout_pass --nocapture (2026-05-20: passed, 7 passed)
  - D:\cargo-targets\zircon-layout-impl\debug\deps\zircon_runtime-eb36ca4a90c1b648.exe surface_frame_authority --nocapture (2026-05-20: passed, 1 passed)
  - cargo test -p zircon_runtime_interface layout_engine_contracts --locked --jobs 1 --target-dir D:\cargo-targets\zircon-layout-interface --message-format short --color never (2026-05-20: passed, 3 passed)
  - cargo test -p zircon_runtime_interface ui_layout_engine_selection_report_counts_unsupported_routes_separately --locked --jobs 1 --target-dir D:\cargo-targets\zircon-layout-interface-unsupported-20260521 --message-format short --color never (2026-05-21: passed, 1 passed)
  - cargo test -p zircon_runtime_interface ui_surface_debug_snapshot_contract_serializes_reflector_and_batch_stats --locked --jobs 1 --target-dir D:\cargo-targets\zircon-layout-interface-contract --message-format short --color never (2026-05-20: passed, 1 passed)
  - cargo test -p zircon_editor --lib ui_debug_reflector_model_projects_snapshot_rows_and_sections --locked --jobs 1 --target-dir D:\cargo-targets\zircon-layout-editor-route-payload-20260521 --message-format short --color never (2026-05-21: passed, 1 passed after route-detail model update)
  - cargo test -p zircon_editor --lib ui_debug_reflector_model_displays_unsupported_layout_routes --locked --jobs 1 --target-dir D:\cargo-targets\zircon-layout-editor-route-payload-20260521 --message-format short --color never (2026-05-21: passed, 1 passed)
  - cargo test -p zircon_editor --lib runtime_diagnostics_payload_uses_active_ui_debug_snapshot_when_available --locked --jobs 1 --target-dir D:\cargo-targets\zircon-layout-editor --message-format short --color never (2026-05-20: passed, 1 passed)
  - cargo test -p zircon_editor --lib runtime_diagnostics_payload_uses_active_ui_debug_snapshot_when_available --locked --jobs 1 --target-dir D:\cargo-targets\zircon-layout-editor-route-payload-20260521 --message-format short --color never (2026-05-21: passed, 1 passed after warmed rebuild)
  - cargo test -p zircon_editor --lib editor_ui_host_runtime_projects_pane_body_payload_metadata_into_root_attributes --locked --jobs 1 --target-dir D:\cargo-targets\zircon-layout-editor-route-payload-20260521 --message-format short --color never (2026-05-21: passed, 1 passed after route-detail projection update)
  - cargo test -p zircon_editor --lib runtime_diagnostics_host_conversion_keeps_payload_reflector_text_and_overlay --locked --jobs 1 --target-dir D:\cargo-targets\zircon-layout-editor-route-payload-20260521 --message-format short --color never (2026-05-21: passed, 1 passed after route-detail host update)
  - cargo test -p zircon_editor --lib runtime_diagnostics_body_refresh_preserves_active_payload_reflector --locked --jobs 1 --target-dir D:\cargo-targets\zircon-layout-editor-route-payload-20260521 --message-format short --color never (2026-05-21: passed, 1 passed after route-detail refresh update)
  - cargo test -p zircon_editor --lib runtime_diagnostics_live_body_surface_populates_debug_reflector_rows_and_overlays --locked --jobs 1 --target-dir D:\cargo-targets\zircon-layout-editor --message-format short --color never (2026-05-20: passed, 1 passed)
  - cargo test -p zircon_editor --lib builtin_editor_host_templates_export_layout_engine_route_reports --locked --jobs 1 --target-dir D:\cargo-targets\zircon-layout-editor-host-routes-20260521 --message-format short --color never (2026-05-21: passed, 1 passed after real host-template route, frame-stability, render, hit, pointer-route, and no-silent-fallback coverage update)
  - cargo test -p zircon_runtime --lib taffy_native_flex_surface_frame_feeds_render_hit_and_pointer_dispatch --locked --jobs 1 --target-dir D:\cargo-targets\zircon-layout-impl --message-format short --color never (2026-05-20: passed, 1 passed)
  - cargo test -p zircon_runtime --lib taffy_grid_slot_frame_policy_feeds_render_hit_and_pointer_dispatch --locked --jobs 1 --target-dir D:\cargo-targets\zircon-layout-authority-20260521 --message-format short --color never (2026-05-21: passed, 1 passed)
  - cargo test -p zircon_runtime --lib zircon_size_box_fallback_feeds_render_hit_and_pointer_dispatch --locked --jobs 1 --target-dir D:\cargo-targets\zircon-layout-authority-20260521 --message-format short --color never (2026-05-21: passed, 1 passed)
  - cargo test -p zircon_runtime --lib runtime_ --locked --jobs 1 --target-dir D:\cargo-targets\zircon-layout-runtime-fixture-20260521 --message-format short --color never (2026-05-21: passed, 282 passed including both runtime fixture route tests after no-silent-fallback coverage update)
  - cargo test -p zircon_runtime --lib runtime_ui_layout_routes --locked --jobs 1 --target-dir D:\cargo-targets\zircon-layout-runtime-fixture-20260521 --message-format short --color never (2026-05-21: passed, 2 passed after real runtime fixture render, hit, pointer-route, and public-frame render-extract authority update)
  - cargo test -p zircon_runtime --lib surface_dirty_layout_preserves_unvisited_layout_engine_routes --locked --jobs 1 --target-dir D:\cargo-targets\zircon-layout-incremental-routes-20260521 --message-format short --color never (2026-05-21: passed, 1 passed after incremental layout-engine report merge update)
  - cargo test -p zircon_runtime --lib surface_dirty_layout --locked --jobs 1 --target-dir D:\cargo-targets\zircon-layout-incremental-routes-20260521 --message-format short --color never (2026-05-21: passed, 6 passed after incremental layout route replacement, stale route removal, and debug snapshot/JSON export coverage)
  - cargo test -p zircon_runtime --lib runtime_inventory_fixture_reports_virtualized_list_zircon_fallback --locked --jobs 1 --target-dir D:\cargo-targets\zircon-layout-runtime-fixture-20260521 --message-format short --color never (2026-05-21: passed, 1 passed)
  - cargo test -p zircon_runtime --lib runtime_quest_log_fixture_exports_layout_engine_route_report --locked --jobs 1 --target-dir D:\cargo-targets\zircon-layout-runtime-fixture-20260521 --message-format short --color never (2026-05-21: passed, 1 passed after JSON export roundtrip update)
  - cargo test -p zircon_runtime --lib surface_debug_snapshot_json_exports_layout_engine_route_report --locked --jobs 1 --target-dir D:\cargo-targets\zircon-layout-impl --message-format short --color never (2026-05-20: passed, 1 passed)
  - cargo test -p zircon_runtime --lib surface_debug_snapshot_json_exports_zircon_fallback_route_reason --locked --jobs 1 --target-dir D:\cargo-targets\zircon-layout-diagnostics-20260521 --message-format short --color never (2026-05-21: passed, 1 passed)
  - cargo test -p zircon_runtime --lib taffy_layout_pass_arranges_linear_wrap_and_grid_containers --locked --jobs 1 --target-dir D:\cargo-targets\zircon-layout-impl --message-format short --color never (2026-05-20: passed, 1 passed)
  - cargo test -p zircon_runtime --lib taffy_layout_pass_uses_measured_text_and_image_desired_sizes --locked --jobs 1 --target-dir D:\cargo-targets\zircon-layout-measurement --message-format short --color never (2026-05-20: passed, 1 passed)
  - cargo test -p zircon_runtime --lib taffy_layout_pass_maps_linear_slot_padding_without_fallback --locked --jobs 1 --target-dir D:\cargo-targets\zircon-layout-measurement --message-format short --color never (2026-05-20: passed, 1 passed)
  - cargo test -p zircon_runtime --lib taffy_layout_pass_maps_grid_slot_padding_and_alignment_without_fallback --locked --jobs 1 --target-dir D:\cargo-targets\zircon-layout-measurement --message-format short --color never (2026-05-20: passed, 1 passed)
  - cargo test -p zircon_runtime --lib layout_pass_reports_taffy_native_and_zircon_fallback_routes --locked --jobs 1 --target-dir D:\cargo-targets\zircon-layout-measurement --message-format short --color never (2026-05-20: passed, 1 passed)
  - cargo test -p zircon_runtime --lib layout_pass_reports_taffy_native_and_zircon_fallback_routes --locked --jobs 1 --target-dir D:\cargo-targets\zircon-layout-route-20260521 --message-format short --color never (2026-05-21: passed, 1 passed)
  - cargo test -p zircon_runtime --lib size_box_contain_aspect_ratio_stays_zircon_owned --locked --jobs 1 --target-dir D:\cargo-targets\zircon-layout-route-20260521 --message-format short --color never (2026-05-21: passed, 1 passed after final route-test update)
  - cargo test -p zircon_runtime --lib taffy_layout_pass --locked --jobs 1 --target-dir D:\cargo-targets\zircon-layout-measurement --message-format short --color never (2026-05-20: passed, 11 passed)
  - cargo test -p zircon_runtime --lib taffy_bridge_keeps_block_display_explicit_and_container_zircon_owned --locked --jobs 1 --target-dir D:\cargo-targets\zircon-layout-impl --message-format short --color never (2026-05-20: passed, 1 passed)
  - cargo test -p zircon_runtime_interface ui_layout_engine_block_is_explicit_not_implied_by_current_container_contracts --locked --jobs 1 --target-dir D:\cargo-targets\zircon-layout-interface --message-format short --color never (2026-05-20: passed, 1 passed)
  - cargo test -p zircon_runtime template_tree_builder_parses_size_box_container_contract -- --nocapture (2026-05-11: passed, 1 passed)
  - cargo test -p zircon_runtime_interface ui_layout_engine_request_maps_current_container_contracts_to_engine_families -- --nocapture (2026-05-11: passed, 1 passed)
doc_type: module-detail
---

# Runtime UI Layout Pass Slots

## Purpose

The runtime layout pass turns retained `.ui.toml` tree data into concrete frames, clip frames, scroll windows, and slot-driven child placement. This is the M1 shared Slate core layer for panel geometry: editor hosts and runtime renderers must consume the arranged `UiSurfaceFrame` that comes out of this pass instead of creating local coordinate tables.

Unreal Slate keeps panel-owned placement in slot objects and emits `FArrangedWidget` records during `OnArrangeChildren`. Zircon keeps the authored source as `.ui.toml`, preserves parent-owned slot policy in neutral DTOs, and then derives `UiArrangedTree`, render extract, and hit-grid entries from the same runtime layout cache.

## Slot Inventory

`UiSlotSchema` is component-level authoring metadata. It names slots such as `content`, `row`, or `page`, and records whether they are required or repeatable. It does not describe placement.

`UiSlot` is the runtime parent-child placement DTO. Current fields cover:

- `kind`: `Free`, `Container`, `Overlay`, `Linear`, `Grid`, `Flow`, `Canvas`, `Scrollable`, `Splitter`, `Scale`.
- `padding` and `alignment`: consumed by `Free`, `Container`, `Overlay`, `HorizontalBox`, `VerticalBox`, `WrapBox`/`FlowBox`, and `GridBox` layout through `layout/pass/slot.rs`, `axis.rs`, `measure.rs`, `arrange.rs`, and `child_frame.rs`.
- `linear_sizing`: consumed by linear panels and mapped into the Taffy flex path. `Auto` stays content/preferred sized, `Stretch` grows by slot value from a zero/min basis, and `StretchContent` grows from content/preferred basis. The Zircon axis solver also consumes the same slot sizing so fallback and Taffy paths keep the same main-axis intent for the supported growth cases.
- `canvas_placement`: preserved for Free/Canvas-like parents by template compilation, but `Free` arrangement still primarily consumes child `anchor`, `pivot`, and `position` unless a slot frame policy is active.
- `order`: consumed for stacked and linear child ordering, including wrap rows.
- `grid_placement`: consumed by `GridBox` for per-child row, column, row-span, and column-span placement. Missing values fall back to stable row-major placement. The Taffy path maps this metadata into explicit CSS grid lines; finite non-negative grid slot padding maps to child margin, and fixed-axis `Center`/`End` alignment maps to `justify_self`/`align_self` while unsupported stretch-axis alignment still falls back to Zircon.
- `z_order`: serialized and parsed for overlay slots, but not yet promoted into `UiTreeNode.z_index`; current arranged/render/hit z-order still comes from the node.
- `dirty_revision`: preserved as slot mutation metadata and not yet a standalone rebuild trigger.

`UiContainerKind` currently has runtime arrange support for `Free`, `Container`, `Overlay`, `Space`, `SizeBox`, `HorizontalBox`, `VerticalBox`, `ScrollableBox`, `WrapBox`, and `GridBox`. Template inference maps authored `FlowBox` and `FlexBox` to `WrapBox`, group aliases to the matching horizontal/vertical/grid containers, and `CanvasBox` to `Free`, so v2 component names can stay stable while the shared runtime keeps a compact container enum.

## Taffy Bridge

`zircon_runtime::ui::layout::taffy_bridge` converts only the Taffy-owned subset of `UiContainerKind` into `taffy::style::Style`: horizontal and vertical flex, grid, and wrap. The shared engine capability also exposes an explicit `Block` family and maps that family to Taffy `Display::Block`, but no current `UiContainerKind` produces a `Block` request. Plain `Container`, `Space`, and `SizeBox` remain Zircon-owned until a real block container contract exists. The bridge copies min/preferred/max constraints into Taffy size fields and maps panel gaps to Taffy gap values.

Overlay, scroll, virtualized list, popup-like, canvas/free, `SizeBox`, and editor docking semantics remain Zircon-owned. The bridge returns `None` for these families, and `UiLayoutEngineSelection` reports a Taffy-to-Zircon fallback with `ZirconOwnedSemantics` when Taffy is requested for those containers. Runtime `CanvasBox` authoring currently compiles to `UiContainerKind::Free`, so the route diagnostic for canvas-like parents is the `Free` family plus the separate `SlotCanvasPlacement` fallback for slot-authored canvas placement. `SizeBox` deliberately maps to the `Container` family but still requires Zircon semantics because its child frame is a contain-fit content rectangle, not a flex/grid track.

`layout/pass/engine.rs` records the backend decision for every arranged container that crosses the layout-engine boundary. Full and incremental layout both finish with a `UiLayoutEngineSelectionReport`; `UiSurface` stores it, `UiSurfaceFrame` carries it, and the debug snapshot mirrors it, including JSON export through `UiSurface::debug_snapshot_json(...)`. Reports count native Taffy selections, legacy Zircon selections, fallbacks, unsupported requests, and node ids for each selection so editor/runtime diagnostics no longer have to infer silent legacy fallback.

Incremental layout produces route selections only for visited dirty subtrees. `UiSurface::rebuild_dirty(...)` merges that partial report with the previous surface-level report by dropping selections for visited nodes and stale deleted nodes, then appending the new subtree selections. This keeps diagnostics surface-scoped even when a leaf under a non-auto parent is the only visited layout node, so unvisited container route decisions do not disappear from `UiSurfaceFrame`.

The report treats `Fallback` and `Unsupported` as separate states. Runtime containers currently route to either native Taffy or a known Zircon fallback, but the shared DTO can also represent a request that neither the preferred backend nor fallback backend supports. Interface tests cover this path without requiring a live runtime container that intentionally has no solver.

Editor Debug Reflector consumes the same report through a `Layout Engine` section. The section is intentionally read-only: it summarizes request counts and previews per-node routing/fallback reasons, but does not recompute frames, hit entries, or render commands. Runtime Diagnostics carries those section lines through `RuntimeDiagnosticsPanePayload`, pane-template attributes, and retained-host generated text rows, so Taffy-vs-Zircon routing is visible in the workbench instead of being limited to serialized snapshots.

`taffy_arrange.rs` is the first runtime pass integration point. `arrange.rs` asks it to solve `HorizontalBox`, `VerticalBox`, `WrapBox`, and `GridBox` before falling back to the legacy Zircon arrange code. The helper accepts template metadata because it carries render/event descriptors. It now accepts linear slot sizing, grid placement, finite non-negative slot padding, grid fixed-axis alignment, and flex/wrap cross-axis alignment where the child constraint can preserve Zircon semantics. It still rejects collapsed children, canvas placement, non-default child anchor/pivot/position, unsupported slot padding values, and flex/wrap main-axis per-child `Center`/`End` alignment with explicit fallback reasons (`UnsupportedChildVisibility`, `SlotFramePolicy`, `SlotCanvasPlacement`, `ChildPlacementPolicy`). This lets v2 template-authored component subtrees take the Bevy-style Taffy path while retaining Zircon ownership for absolute placement and parent-specific frame policies.

The Taffy bridge preserves Zircon's explicit stretch semantics. A child with `StretchMode::Stretch` and an authored preferred extent participates in main-axis `flex_grow` using its constraint weight; on the cross axis it leaves size as `auto` so the parent's stretch alignment can fill the available extent while min/max constraints still clamp the result. Default content-driven children with measured desired size can remain content-sized unless their preserved stretch axis asks to fill.

## Behavior Model

The pass runs in two phases. `measure.rs` walks children first, computes desired content size, and includes slot padding for stacked, linear, and wrap containers. `arrange.rs` then writes each node's `layout_cache.frame`, `clip_frame`, `content_size`, and `virtual_window`.

Stacked panels (`Free`, `Container`, `Overlay`) use `free_child_frame(...)`. When a matching slot carries padding or alignment, the child is arranged inside the padded parent frame. Without a slot frame policy, the legacy node-owned anchor, pivot, and position fields remain the placement source.

`SizeBox` measures stacked child content and, when `aspect_ratio` is positive and finite, expands the desired content box to preserve that ratio. During arrange it computes a centered contain-fit content frame inside the parent frame; children are then placed through the normal container slot path inside that content frame. Invalid or zero ratios degrade to plain container behavior.

Linear panels order children by slot `order`, solve main-axis extents from constraints plus slot padding and `UiLinearSlotSizing`, and use slot alignment to place each child inside its allocated outer frame. Taffy-native linear panels map the same slot sizing into flex basis/grow/shrink and map finite non-negative slot padding to child margin; flex/wrap cross-axis alignment can stay native when constraints make the aligned extent explicit. Flex main-axis `Center`/`End` remains a Zircon fallback because Taffy flex does not provide per-child main-axis self alignment. `WrapBox`/`FlowBox` reuses the linear child-frame logic per row after grouping children by available width, horizontal gap, vertical gap, item minimum width, and slot padding.

`GridBox` divides the parent frame into configured rows and columns, subtracts row/column gaps once, and places children from `UiGridSlotPlacement`. Span values expand the outer cell frame before normal slot padding/alignment is applied, so render extraction and hit testing see the same child frame that layout measured. Taffy-native grid panels use the same zero-based placement metadata to emit one-based CSS grid line starts/ends, expand the template rows/columns to cover explicit spans, map finite non-negative slot padding to margin, and map fixed-size slot alignment to `justify_self`/`align_self`.

`ScrollableBox` computes content extent, clamps scroll offset, records `UiScrollState`, and stores `UiVirtualListWindow` when virtualization is configured. Off-window children are hidden from hit testing by zeroing layout frames through `hide_subtree_layout(...)`; visible children keep frames and clips that feed the surface frame.

`UiSurfaceFrame` remains the single spatial authority regardless of backend. Taffy and Zircon both write the same retained tree layout cache, then `build_arranged_tree`, render extraction, hit-grid rebuild, pointer routing, and debug reflection consume that output. The layout-engine report is diagnostic metadata attached to this same frame, not a second coordinate source.

Runtime layout invalidation uses structured dirty domains. `mark_layout_dirty(...)` bubbles layout invalidation through content-driven or auto-layout ancestors, marking layout, hit-test, and render dirty on affected nodes without setting the legacy `state_flags.dirty` compatibility bit. This keeps `UiSurface::dirty_flags()` diagnostics precise and avoids reporting input dirtiness for pure layout changes.

## Shared Frame Contract

`zircon_runtime/src/ui/tests/layout_slots.rs` now covers four M1.3 focused authority cases:

- `overlay_slot_geometry_feeds_arranged_render_hit_and_z_order_from_one_surface_frame` arranges overlapping overlay children with slot padding/alignment and node z-index, then proves `UiSurfaceFrame.arranged_tree`, `render_extract`, `hit_grid`, and `hit_test_surface_frame(...)` agree on frame, clip, z-order, stacked hit order, and bubble route.
- `scrollable_virtual_window_uses_visible_arranged_child_for_render_and_hit_entries` arranges a virtualized scroller at an offset, verifies the visible window, and proves the visible arranged child is the same frame consumed by render and hit-grid entries while off-window children do not enter hit testing.
- `wrap_flow_slot_padding_alignment_feeds_shared_surface_frame` proves `WrapBox`/flow slot order, padding, and alignment feed arranged, render, and hit evidence from one frame.
- `grid_slot_cell_placement_feeds_arranged_render_hit_from_one_surface_frame` proves configured grid rows/columns/gaps and per-child grid placement feed arranged, render, and hit evidence from one frame.

`zircon_runtime/src/ui/tests/template_grid_flow.rs` adds the matching template compile contract. It proves authored `GridBox` and `FlowBox` nodes produce the expected runtime container config, child slot kinds, grid placement/span metadata, and flow ordering before the layout pass runs.

`zircon_runtime/src/ui/tests/taffy_layout_pass.rs` now covers the runtime Taffy route contract. It proves simple flex/wrap/grid containers select Taffy natively, free/canvas-like, generic container, space, overlay, scrollable, virtual-list, and `SizeBox` containers report Zircon-owned fallback, flex main-axis alignment, invalid slot padding values, and canvas placement report explicit fallback reasons, grid placement plus linear slot sizing can remain on the Taffy-native path, slot padding can stay Taffy-native for linear panels, grid slot padding/alignment can stay Taffy-native for fixed-size cells, and measured text/image desired sizes from the measure pass feed Taffy-native child frames.

`zircon_runtime/src/ui/tests/surface_frame_authority.rs` now includes backend-specific authority coverage beyond the original overlapping free-frame case. It proves Taffy-native flex and grid containers write the expected frames into `UiSurfaceFrame`, and it proves a Zircon-owned `SizeBox` fallback writes its contain-fit slot-aligned frame into the same authority surface. In each case render extract, hit-grid entries, frame hit testing, and pointer dispatch consume that same arranged frame instead of rebuilding local geometry.

`zircon_runtime/src/ui/tests/diagnostics.rs` covers JSON/debug snapshot export for both sides of the route report. The Taffy-native snapshot test proves `layout_engine_report` survives JSON roundtrip with a native Taffy backend; the Zircon fallback snapshot test proves `legacy_zircon`, `fallback`, and `zircon_owned_semantics` are serialized and deserialize back to the same `UiLayoutEngineSelection` for a `SizeBox` root.

These tests are intentionally runtime-only. They do not touch Material visual templates, editor host rectangles, or native painter code.

## Accepted Follow-Ups

M1 accepts the shared panel authority for the current retained-tree model. These items remain explicit follow-ups rather than hidden M1 gaps:

- Overlay slot `z_order` should either be promoted into arranged z-order or removed from the runtime DTO if node `z_index` remains the only z authority.
- `canvas_placement` needs a runtime cutover decision for Free/Canvas panels so parent-owned placement can replace child-owned anchor/pivot/position where intended. Until then, Canvas-like parent routing is visible as `UiLayoutEngineFamily::Free` with Zircon-owned fallback, and slot-authored canvas placement stays an explicit `SlotCanvasPlacement` fallback.
- Scroll virtualization currently keeps hidden children in the retained tree and arranged tree with zeroed frames; M1 accepts the hit-grid boundary, but later render/performance work should decide whether zero-area render commands should be suppressed.
- A true `BlockBox` or equivalent container should be introduced before claiming runtime Block arrangement. The interface can already represent an explicit `Block` request for backend selection, but the current retained tree deliberately does not reinterpret generic `Container` as block layout.

## Test Coverage

Formatting passed for the changed runtime-interface layout DTOs, runtime layout pass files, template parser/build files, and focused tests.

Focused runtime validation passed with `7 passed; 0 failed; 920 filtered out` for:

```powershell
cargo test -p zircon_runtime --lib layout_slots --locked --jobs 1 --target-dir D:\cargo-targets\zircon-m1-slot-panel --message-format short --color never -- --nocapture
```

Template Grid/Flow validation passed with `1 passed; 0 failed; 926 filtered out` for:

```powershell
cargo test -p zircon_runtime --lib template_grid_flow --locked --jobs 1 --target-dir D:\cargo-targets\zircon-m1-slot-panel --message-format short --color never -- --nocapture
```

The first cold run exceeded the tool timeout while compiling on a machine with concurrent sibling Cargo validations. Warmed focused reruns above completed and are the accepted evidence for this slice. Existing `zircon_runtime` warning noise remains.

The v2 Taffy cutover guard passed with `6 passed; 0 failed; 1251 filtered out` for:

```powershell
cargo test -p zircon_runtime taffy -- --nocapture
```

That run includes `taffy_layout_pass_accepts_template_metadata_from_v2_assets`, proving v2-authored nodes no longer fall back merely because they carry template metadata. It also includes `size_box_contain_aspect_ratio_stays_zircon_owned`, proving `SizeBox` uses Zircon contain-fit semantics and remains outside the Taffy-owned arrange path.

The 2026-05-20 layout-engine report slice passed by running the already-built focused runtime test harness after the Cargo invocation completed:

```powershell
D:\cargo-targets\zircon-layout-impl\debug\deps\zircon_runtime-eb36ca4a90c1b648.exe taffy_layout_pass --nocapture
D:\cargo-targets\zircon-layout-impl\debug\deps\zircon_runtime-eb36ca4a90c1b648.exe surface_frame_authority --nocapture
```

The first command passed `7 passed; 0 failed; 1657 filtered out`, covering backend reports, explicit fallback reasons, grid placement, and linear slot sizing. The second passed `1 passed; 0 failed; 1663 filtered out`, preserving the arranged/render/hit/pointer authority contract.

The backend authority continuation adds `taffy_native_flex_surface_frame_feeds_render_hit_and_pointer_dispatch`, `taffy_grid_slot_frame_policy_feeds_render_hit_and_pointer_dispatch`, and `zircon_size_box_fallback_feeds_render_hit_and_pointer_dispatch` to `surface_frame_authority.rs`. The Taffy tests prove native flex and grid-slot frames are the same frames seen by arranged tree, render extract, hit grid, surface hit testing, and pointer dispatch while the layout-engine report marks the root container as native Taffy. The Zircon fallback test proves `SizeBox` reports `Container` + `ZirconOwnedSemantics`, keeps its contain-fit slot-aligned frame, and still feeds the same downstream authorities. The grid-slot focused command passed on 2026-05-21 with `1 passed; 0 failed; 1750 filtered out` after a `20m 42s` cold build in `D:\cargo-targets\zircon-layout-authority-20260521`. The SizeBox fallback focused command passed on 2026-05-21 with `1 passed; 0 failed; 1760 filtered out` after a `4m 48s` warmed rebuild in the same target dir. Both reported existing `zircon_runtime` warning noise.

The incremental route-report continuation adds `surface_dirty_layout_preserves_unvisited_layout_engine_routes`, `surface_dirty_layout_replaces_visited_layout_engine_routes`, and `surface_dirty_layout_drops_removed_layout_engine_routes` to `surface_dirty_domains.rs`. The first test starts from a `Free` root whose route report is a Zircon-owned fallback, then marks only a leaf child layout-dirty under a non-auto parent. The accepted rebuild visits one node, skips two, preserves the root fallback route in `UiSurface.layout_engine_report`, and confirms `UiSurfaceFrame.layout_engine_report` carries the same surface-level report. The replacement test mutates a visited child container from native Taffy `Flex` to Zircon-owned `Overlay`, requiring the old flex selection to disappear and exactly one overlay fallback selection to remain for that node while the unvisited root fallback stays present. The removal test detaches the previously reported flex subtree and requires the merged report to drop the stale node selection while retaining the root fallback. These incremental tests also verify the merged report is exported through `UiSurfaceFrame`, `UiSurface::debug_snapshot()`, and `UiSurface::debug_snapshot_json(...)` roundtrip, so diagnostics and JSON export do not regress to a partial subtree report. The focused command passed on 2026-05-21 with `1 passed; 0 failed; 1797 filtered out` after a `17m 07s` cold build in `D:\cargo-targets\zircon-layout-incremental-routes-20260521`. The broader `surface_dirty_layout` rerun passed `6 passed; 0 failed; 1794 filtered out` after a `3m 15s` rebuild in the same target dir, covering the existing skip/revisit/marking dirty-domain cases plus route preservation, replacement, and stale-route removal. The debug-export rerun passed the same `6 passed; 0 failed; 1794 filtered out` after a `2m 06s` rebuild in the same target dir. These accepted runs reported existing `zircon_runtime` warning noise.

The runtime fixture route-report continuation adds `runtime_quest_log_fixture_exports_layout_engine_route_report` and `runtime_inventory_fixture_reports_virtualized_list_zircon_fallback`. The quest log test loads the real `runtime.ui.quest_log_dialog` fixture through `RuntimeUiManager`, computes layout, and verifies that the authored `VerticalBox` dialog and `HorizontalBox` actions route to native Taffy while the overlay root remains a Zircon fallback with `ZirconOwnedSemantics`. The inventory test loads `runtime.ui.inventory_list` and verifies that the real virtualized `ScrollableBox` reports `UiLayoutEngineFamily::VirtualizedList` with `LegacyZircon`, `Fallback`, and `ZirconOwnedSemantics`. Both tests assert the report is identical on `UiSurface`, `UiSurfaceFrame`, the debug snapshot, and a JSON-exported debug snapshot roundtrip, so runtime `.v2.ui.toml` assets do not get a separate diagnostic path. They also recompute route counts from `selections`, require every route to identify its node, require native routes to be Taffy without fallback reasons, and require every LegacyZircon fallback or unsupported route to carry a diagnostic reason. This mirrors the Editor host no-silent-fallback contract for real runtime fixture assets. The runtime fixture authority continuation now asserts the same arranged frames feed render command frame/clip/z-index for `QuestLogDialog`, `QuestLogActions`, `InventoryList`, and the first visible virtualized inventory row. It also dispatches pointer events through `RuntimeUiManager` for `TrackQuestButton`, `CloseQuestLogButton`, and `InventoryRow00`, requiring matching hit-grid entries, identical `surface.hit_test(...)` and `hit_test_surface_frame(...)` results, and a successful `UiPointerDispatcher` route through the same hit path. The same tests then call `RuntimeUiManager::build_frame()` and require the public runtime frame `ui` extract to equal both `UiSurfaceFrame.render_extract` and `UiSurface.render_extract`, with non-empty render commands, so the runtime submission boundary consumes the same surface output. The first quest-log focused command passed `1 passed; 0 failed; 1791 filtered out` after a `26m 50s` cold build in `D:\cargo-targets\zircon-layout-runtime-fixture-20260521`. The JSON roundtrip rerun passed `1 passed; 0 failed; 1793 filtered out` after a `5m 19s` warmed rebuild. After the inventory test was added, the broader `runtime_` filter passed `282 passed; 0 failed; 1515 filtered out` after a `3m 58s` warmed rebuild, and the exact inventory and quest-log focused reruns each passed `1 passed; 0 failed; 1796 filtered out` in the same target dir. The final no-silent-fallback rerun passed `282 passed; 0 failed; 1515 filtered out` after a `4m 02s` rebuild in the same target dir. The authority-focused module rerun passed `2 passed; 0 failed; 1795 filtered out` after a `3m 37s` rebuild in the same target dir. The public-frame authority rerun passed `2 passed; 0 failed; 1795 filtered out` after a `3m 06s` rebuild in the same target dir. These runs reported existing `zircon_runtime` warning noise.

The Zircon fallback diagnostics continuation passed with:

```powershell
cargo test -p zircon_runtime --lib surface_debug_snapshot_json_exports_zircon_fallback_route_reason --locked --jobs 1 --target-dir D:\cargo-targets\zircon-layout-diagnostics-20260521 --message-format short --color never
```

The final rerun passed `1 passed; 0 failed; 1764 filtered out` after a `3m 06s` warmed rebuild. An earlier run failed because the test asserted compact JSON while `UiSurface::debug_snapshot_json(...)` intentionally uses pretty JSON; the assertion now matches the exported pretty JSON and still deserializes the snapshot to verify the structured `LegacyZircon`/`Fallback`/`ZirconOwnedSemantics` selection. The passing run used `--locked` and reported existing `zircon_runtime` warning noise.

The unsupported route-report contract is covered by:

```powershell
cargo test -p zircon_runtime_interface ui_layout_engine_selection_report_counts_unsupported_routes_separately --locked --jobs 1 --target-dir D:\cargo-targets\zircon-layout-interface-unsupported-20260521 --message-format short --color never
```

This test constructs explicit empty preferred/fallback capabilities and verifies that a `Block` request records `UnsupportedFamily`, `UiLayoutEngineSupport::Unsupported`, and `unsupported_count=1` while leaving `fallback_count=0`. It is an interface DTO contract, not a claim that any current runtime container is intentionally unsupported.

The focused run passed `1 passed; 0 failed; 102 filtered out` after a `2m 35s` cold compile in `D:\cargo-targets\zircon-layout-interface-unsupported-20260521`.

The runtime-interface DTO and debug snapshot contracts passed with:

```powershell
cargo test -p zircon_runtime_interface layout_engine_contracts --locked --jobs 1 --target-dir D:\cargo-targets\zircon-layout-interface --message-format short --color never
cargo test -p zircon_runtime_interface ui_surface_debug_snapshot_contract_serializes_reflector_and_batch_stats --locked --jobs 1 --target-dir D:\cargo-targets\zircon-layout-interface-contract --message-format short --color never
```

Those runs passed `3 passed; 0 failed; 97 filtered out` and `1 passed; 0 failed; 99 filtered out`.

The editor reflector projection for the same report passed with:

```powershell
cargo test -p zircon_editor --lib ui_debug_reflector_model_projects_snapshot_rows_and_sections --locked --jobs 1 --target-dir D:\cargo-targets\zircon-layout-editor --message-format short --color never
cargo test -p zircon_editor --lib ui_debug_reflector_model_displays_unsupported_layout_routes --locked --jobs 1 --target-dir D:\cargo-targets\zircon-layout-editor-route-payload-20260521 --message-format short --color never
```

The native/fallback projection rerun passed `1 passed; 0 failed; 1416 filtered out` after a `6.30s` warmed rebuild and reported existing `zircon_runtime`/`zircon_editor` warning noise. The unsupported-route continuation passed `1 passed; 0 failed; 1416 filtered out` after a `4m 03s` warmed rebuild; it constructs a report where neither preferred nor fallback capability supports `Block` and requires the Editor `Layout Engine` section to show `fallbacks: 0 unsupported: 1`, `support=Unsupported`, and `reason=UnsupportedFamily`.

The Runtime Diagnostics display continuation passed the focused payload, template-attribute, payload-host, and live-body refresh tests in the header. The first rerun passed after a `12m 23s` rebuild; warmed follow-up tests completed in seconds. These prove the runtime-owned layout-engine report reaches the active snapshot payload, template root attributes, retained-host generated rows, and live body-surface refresh path.

The editor pane payload route-detail continuation tightens `runtime_diagnostics_payload_uses_active_ui_debug_snapshot_when_available` so the payload must include the exact per-node fallback line for a Zircon-owned route (`node=2`, `family=Overlay`, `selected=LegacyZircon`, `reason=ZirconOwnedSemantics`). This proves the workbench-facing `RuntimeDiagnosticsPanePayload` preserves fallback reasons, not only aggregate Taffy/Zircon counts.

The focused editor rerun passed `1 passed; 0 failed; 1415 filtered out` after a `3m 41s` warmed rebuild in `D:\cargo-targets\zircon-layout-editor-route-payload-20260521`. The earlier cold compile exceeded the tool timeout while still building, so only the completed warmed rerun is accepted as evidence.

The retained-host and template-runtime continuation extends the same route-detail requirement beyond the pane payload object. `runtime_diagnostics_host_conversion_keeps_payload_reflector_text_and_overlay` now requires the generated host rows to include the `Overlay`/`LegacyZircon`/`ZirconOwnedSemantics` line, `runtime_diagnostics_body_refresh_preserves_active_payload_reflector` proves an active payload reflector is not replaced by a live body-surface refresh, and `editor_ui_host_runtime_projects_pane_body_payload_metadata_into_root_attributes` proves the pane-body template attributes retain the same fallback reason line. The focused reruns passed `1 passed; 0 failed; 1415 filtered out` each; the first rebuilt in `3m 55s`, and the warmed follow-ups completed in `9.07s` and `4.71s`.

The editor host template route-report continuation adds `builtin_editor_host_templates_export_layout_engine_route_reports`. It loads the real built-in workbench shell, drawer source, floating-window source, and scene viewport toolbar documents through `EditorUiHostRuntime`, builds shared surfaces, computes layout, and checks their engine selections. Workbench flex scaffolding routes to native Taffy while overlay shell nodes remain explicit `LegacyZircon` fallbacks with `ZirconOwnedSemantics`; drawer, floating-window, and viewport-toolbar flex roots also route to native Taffy with no unsupported selections. The same test asserts the report is identical on `UiSurface`, `UiSurfaceFrame`, the debug snapshot, and the JSON debug snapshot roundtrip. It also recomputes the report counts from `selections`, requires every route to identify its node, requires native routes to be Taffy without fallback reasons, and requires every LegacyZircon fallback to carry an explicit diagnostic reason, so Editor host templates cannot regress to silent fallback. It also asserts key arranged frames from the same `UiSurfaceFrame`: workbench document host and pane surface, drawer top/body/status bands, floating-window top/center/document/status bands, and viewport-toolbar visible command controls. Those frame assertions require matching render command frame, clip, and z-index. Interactive controls (`OpenProject`, `FrameSelection`, and `SetProjectionMode`) additionally require matching hit-grid entries, identical `surface.hit_test(...)` and `hit_test_surface_frame(...)` results, and successful `UiPointerDispatcher` routing through the same bubble path. One intermediate assertion incorrectly reused the older host-model left-group width for the shared toolbar surface; the accepted test keeps the visible command frames and right group as the no-drift contract. The final focused rerun passed `1 passed; 0 failed; 1417 filtered out` after a `2m 41s` warmed rebuild in `D:\cargo-targets\zircon-layout-editor-host-routes-20260521`; it reported existing `zircon_runtime` and `zircon_editor` warning noise.

The flex/wrap/grid route tightening passed with:

```powershell
cargo test -p zircon_runtime --lib taffy_layout_pass_arranges_linear_wrap_and_grid_containers --locked --jobs 1 --target-dir D:\cargo-targets\zircon-layout-impl --message-format short --color never
```

That test now proves the same pass that validates child geometry also records native Taffy selections for `Flex`, `Wrap`, and `Grid` root families.

The Zircon-owned route diagnostic continuation passed with:

```powershell
cargo test -p zircon_runtime --lib layout_pass_reports_taffy_native_and_zircon_fallback_routes --locked --jobs 1 --target-dir D:\cargo-targets\zircon-layout-route-20260521 --message-format short --color never
cargo test -p zircon_runtime --lib size_box_contain_aspect_ratio_stays_zircon_owned --locked --jobs 1 --target-dir D:\cargo-targets\zircon-layout-route-20260521 --message-format short --color never
```

The first command passed `1 passed; 0 failed; 1757 filtered out` after a `3m 40s` warmed rebuild and covers `Free`, generic `Container`, and child-bearing `Space` Zircon-owned fallback alongside overlay, scrollable, virtual list, slot frame policy, and slot canvas placement fallback reasons. An intermediate extension failed because childless `Space` containers are intentionally omitted from the route report; the test now gives `Space` a child before asserting the fallback selection. The second command passed `1 passed; 0 failed; 1759 filtered out` after a `4m 06s` warmed rebuild and proves `SizeBox` keeps its contain-fit frame while reporting the `Container` family with `ZirconOwnedSemantics` fallback. Both passing runs used `--locked`; existing `zircon_runtime` warning noise remains.

The measured leaf sizing continuation passed with:

```powershell
cargo test -p zircon_runtime --lib taffy_layout_pass_uses_measured_text_and_image_desired_sizes --locked --jobs 1 --target-dir D:\cargo-targets\zircon-layout-measurement --message-format short --color never
```

That focused test passed `1 passed; 0 failed; 1740 filtered out` and proves Label text measurement plus IconButton image/padding measurement produce desired sizes that the Taffy-native flex arrange path consumes for final child frames. An earlier retry against `D:\cargo-targets\zircon-layout-impl` compiled but failed to write Cargo dep-info after the shared target fingerprint path disappeared; the first cold run in `D:\cargo-targets\zircon-layout-measurement` exceeded the 30-minute tool timeout while continuing to compile, and the warmed rerun above is the accepted evidence.

The Block/Container bridge boundary passed with:

```powershell
cargo test -p zircon_runtime --lib taffy_bridge_keeps_block_display_explicit_and_container_zircon_owned --locked --jobs 1 --target-dir D:\cargo-targets\zircon-layout-impl --message-format short --color never
cargo test -p zircon_runtime_interface ui_layout_engine_block_is_explicit_not_implied_by_current_container_contracts --locked --jobs 1 --target-dir D:\cargo-targets\zircon-layout-interface --message-format short --color never
```

The runtime bridge test passed `1 passed; 0 failed; 1723 filtered out` and proves `Block` maps to Taffy block display while generic `Container` still returns no Taffy style and reports Zircon-owned fallback. The interface test passed `1 passed; 0 failed; 101 filtered out` and proves no current `UiContainerKind` emits `UiLayoutEngineFamily::Block`, while an explicit `Block` request still selects Taffy natively.

The SizeBox TOML parser contract passed with:

```powershell
cargo test -p zircon_runtime template_tree_builder_parses_size_box_container_contract -- --nocapture
```

The runtime-interface layout engine contract passed with:

```powershell
cargo test -p zircon_runtime_interface ui_layout_engine_request_maps_current_container_contracts_to_engine_families -- --nocapture
```
