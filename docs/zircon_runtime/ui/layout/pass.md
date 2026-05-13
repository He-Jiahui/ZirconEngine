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
  - zircon_runtime/src/ui/layout/pass/slot.rs
  - zircon_runtime/src/ui/layout/pass/axis.rs
  - zircon_runtime/src/ui/layout/pass/measure.rs
  - zircon_runtime/src/ui/layout/pass/arrange.rs
  - zircon_runtime/src/ui/layout/pass/child_frame.rs
  - zircon_runtime/src/ui/layout/pass/taffy_arrange.rs
  - zircon_runtime/src/ui/layout/taffy_bridge.rs
  - zircon_runtime/src/ui/tests/layout_slots.rs
  - zircon_runtime/src/ui/tests/template_grid_flow.rs
  - zircon_runtime/src/ui/tests/taffy_bridge.rs
  - zircon_runtime/src/ui/tests/taffy_layout_pass.rs
implementation_files:
  - zircon_runtime_interface/src/ui/layout/scroll.rs
  - zircon_runtime_interface/src/ui/layout/mod.rs
  - zircon_runtime_interface/src/ui/layout/engine.rs
  - zircon_runtime/src/ui/template/build/slot_contract.rs
  - zircon_runtime/src/ui/template/build/container_inference.rs
  - zircon_runtime/src/ui/template/build/parsers.rs
  - zircon_runtime/src/ui/layout/pass/slot.rs
  - zircon_runtime/src/ui/layout/pass/axis.rs
  - zircon_runtime/src/ui/layout/pass/measure.rs
  - zircon_runtime/src/ui/layout/pass/arrange.rs
  - zircon_runtime/src/ui/layout/pass/child_frame.rs
  - zircon_runtime/src/ui/layout/pass/taffy_arrange.rs
  - zircon_runtime/src/ui/layout/taffy_bridge.rs
  - zircon_runtime/src/ui/tests/layout_slots.rs
  - zircon_runtime/src/ui/tests/template_grid_flow.rs
  - zircon_runtime/src/ui/tests/taffy_bridge.rs
  - zircon_runtime/src/ui/tests/taffy_layout_pass.rs
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
- `linear_sizing`: preserved for linear slots and covered by interface serialization, but the current runtime main-axis solver still mostly derives sizing from child constraints and preserved stretch axes.
- `canvas_placement`: preserved for Free/Canvas-like parents by template compilation, but `Free` arrangement still primarily consumes child `anchor`, `pivot`, and `position` unless a slot frame policy is active.
- `order`: consumed for stacked and linear child ordering, including wrap rows.
- `grid_placement`: consumed by `GridBox` for per-child row, column, row-span, and column-span placement. Missing values fall back to stable row-major placement.
- `z_order`: serialized and parsed for overlay slots, but not yet promoted into `UiTreeNode.z_index`; current arranged/render/hit z-order still comes from the node.
- `dirty_revision`: preserved as slot mutation metadata and not yet a standalone rebuild trigger.

`UiContainerKind` currently has runtime arrange support for `Free`, `Container`, `Overlay`, `Space`, `SizeBox`, `HorizontalBox`, `VerticalBox`, `ScrollableBox`, `WrapBox`, and `GridBox`. Template inference maps authored `FlowBox` and `FlexBox` to `WrapBox`, group aliases to the matching horizontal/vertical/grid containers, and `CanvasBox` to `Free`, so v2 component names can stay stable while the shared runtime keeps a compact container enum.

## Taffy Bridge

`zircon_runtime::ui::layout::taffy_bridge` converts only the Taffy-owned subset of `UiContainerKind` into `taffy::style::Style`: horizontal and vertical flex, grid, wrap, and block-style requests. It copies min/preferred/max constraints into Taffy size fields and maps panel gaps to Taffy gap values.

Overlay, scroll, virtualized list, popup-like, canvas/free, `SizeBox`, and editor docking semantics remain Zircon-owned. The bridge returns `None` for these families, and `UiLayoutEngineSelection` reports a Taffy-to-Zircon fallback with `ZirconOwnedSemantics` when Taffy is requested for those containers. `SizeBox` deliberately maps to the `Container` family but still requires Zircon semantics because its child frame is a contain-fit content rectangle, not a flex/grid track.

`taffy_arrange.rs` is the first runtime pass integration point. `arrange.rs` asks it to solve `HorizontalBox`, `VerticalBox`, `WrapBox`, and `GridBox` before falling back to the legacy Zircon arrange code. The helper accepts only simple child contracts: template metadata is allowed because it carries render/event descriptors, but authored slot frame policies, canvas/grid placement overrides, and non-default child anchor/pivot/position still force Zircon arrangement. This lets v2 template-authored component subtrees take the Bevy-style Taffy path while retaining Zircon ownership for absolute placement and parent slot semantics.

The Taffy bridge preserves Zircon's explicit stretch semantics. A child with `StretchMode::Stretch` and an authored preferred extent participates in main-axis `flex_grow` using its constraint weight; on the cross axis it leaves size as `auto` so the parent's stretch alignment can fill the available extent while min/max constraints still clamp the result. Default content-driven children with measured desired size can remain content-sized unless their preserved stretch axis asks to fill.

## Behavior Model

The pass runs in two phases. `measure.rs` walks children first, computes desired content size, and includes slot padding for stacked, linear, and wrap containers. `arrange.rs` then writes each node's `layout_cache.frame`, `clip_frame`, `content_size`, and `virtual_window`.

Stacked panels (`Free`, `Container`, `Overlay`) use `free_child_frame(...)`. When a matching slot carries padding or alignment, the child is arranged inside the padded parent frame. Without a slot frame policy, the legacy node-owned anchor, pivot, and position fields remain the placement source.

`SizeBox` measures stacked child content and, when `aspect_ratio` is positive and finite, expands the desired content box to preserve that ratio. During arrange it computes a centered contain-fit content frame inside the parent frame; children are then placed through the normal container slot path inside that content frame. Invalid or zero ratios degrade to plain container behavior.

Linear panels order children by slot `order`, solve main-axis extents from constraints plus slot padding, and use slot alignment to place each child inside its allocated outer frame. `WrapBox`/`FlowBox` reuses the linear child-frame logic per row after grouping children by available width, horizontal gap, vertical gap, item minimum width, and slot padding.

`GridBox` divides the parent frame into configured rows and columns, subtracts row/column gaps once, and places children from `UiGridSlotPlacement`. Span values expand the outer cell frame before normal slot padding/alignment is applied, so render extraction and hit testing see the same child frame that layout measured.

`ScrollableBox` computes content extent, clamps scroll offset, records `UiScrollState`, and stores `UiVirtualListWindow` when virtualization is configured. Off-window children are hidden from hit testing by zeroing layout frames through `hide_subtree_layout(...)`; visible children keep frames and clips that feed the surface frame.

Runtime layout invalidation uses structured dirty domains. `mark_layout_dirty(...)` bubbles layout invalidation through content-driven or auto-layout ancestors, marking layout, hit-test, and render dirty on affected nodes without setting the legacy `state_flags.dirty` compatibility bit. This keeps `UiSurface::dirty_flags()` diagnostics precise and avoids reporting input dirtiness for pure layout changes.

## Shared Frame Contract

`zircon_runtime/src/ui/tests/layout_slots.rs` now covers four M1.3 focused authority cases:

- `overlay_slot_geometry_feeds_arranged_render_hit_and_z_order_from_one_surface_frame` arranges overlapping overlay children with slot padding/alignment and node z-index, then proves `UiSurfaceFrame.arranged_tree`, `render_extract`, `hit_grid`, and `hit_test_surface_frame(...)` agree on frame, clip, z-order, stacked hit order, and bubble route.
- `scrollable_virtual_window_uses_visible_arranged_child_for_render_and_hit_entries` arranges a virtualized scroller at an offset, verifies the visible window, and proves the visible arranged child is the same frame consumed by render and hit-grid entries while off-window children do not enter hit testing.
- `wrap_flow_slot_padding_alignment_feeds_shared_surface_frame` proves `WrapBox`/flow slot order, padding, and alignment feed arranged, render, and hit evidence from one frame.
- `grid_slot_cell_placement_feeds_arranged_render_hit_from_one_surface_frame` proves configured grid rows/columns/gaps and per-child grid placement feed arranged, render, and hit evidence from one frame.

`zircon_runtime/src/ui/tests/template_grid_flow.rs` adds the matching template compile contract. It proves authored `GridBox` and `FlowBox` nodes produce the expected runtime container config, child slot kinds, grid placement/span metadata, and flow ordering before the layout pass runs.

These tests are intentionally runtime-only. They do not touch Material visual templates, editor host rectangles, or native painter code.

## Accepted Follow-Ups

M1 accepts the shared panel authority for the current retained-tree model. These items remain explicit follow-ups rather than hidden M1 gaps:

- Overlay slot `z_order` should either be promoted into arranged z-order or removed from the runtime DTO if node `z_index` remains the only z authority.
- `canvas_placement` needs a runtime cutover decision for Free/Canvas panels so parent-owned placement can replace child-owned anchor/pivot/position where intended.
- Scroll virtualization currently keeps hidden children in the retained tree and arranged tree with zeroed frames; M1 accepts the hit-grid boundary, but later render/performance work should decide whether zero-area render commands should be suppressed.

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

The SizeBox TOML parser contract passed with:

```powershell
cargo test -p zircon_runtime template_tree_builder_parses_size_box_container_contract -- --nocapture
```

The runtime-interface layout engine contract passed with:

```powershell
cargo test -p zircon_runtime_interface ui_layout_engine_request_maps_current_container_contracts_to_engine_families -- --nocapture
```
