---
related_code:
  - zircon_runtime_interface/src/ui/layout/mod.rs
  - zircon_runtime_interface/src/ui/layout/engine.rs
  - zircon_runtime_interface/src/ui/layout/constraints.rs
  - zircon_runtime_interface/src/ui/layout/debug.rs
  - zircon_runtime_interface/src/ui/layout/slot.rs
  - zircon_runtime_interface/src/ui/layout/style.rs
  - zircon_runtime_interface/src/ui/layout/scroll.rs
  - zircon_runtime_interface/src/ui/layout/linear_sizing.rs
  - zircon_runtime_interface/src/tests/ui_layout.rs
  - zircon_runtime_interface/src/tests/layout_engine_contracts.rs
  - dev/bevy/crates/bevy_ui/src/layout/convert.rs
  - docs/ui-and-layout/slate-style-ui-surface-frame.md
implementation_files:
  - zircon_runtime_interface/src/ui/layout/mod.rs
  - zircon_runtime_interface/src/ui/layout/engine.rs
  - zircon_runtime_interface/src/ui/layout/debug.rs
  - zircon_runtime_interface/src/ui/layout/slot.rs
  - zircon_runtime_interface/src/ui/layout/style.rs
  - zircon_runtime_interface/src/ui/layout/scroll.rs
plan_sources:
  - .codex/plans/Bevy-Informed Zircon UI 架构优化里程碑计划.md
  - docs/ui-and-layout/bevy-informed-ui-m0-gap-audit.md
  - user: 2026-05-08 continue M3 layout-engine interface preflight slice
  - user: 2026-05-24 continue ZirconEditor MUI Web parity Masonry layout
  - user: 2026-06-12 implement editor UI architecture from docs/plans/zircon_editor/editor_ui
  - docs/plans/zircon_editor/editor_ui/02-layout-taffy-and-containers.md
  - docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md
  - user: 2026-06-16 implement runtime architecture from docs/plans/zircon_runtime/runtime
tests:
  - zircon_runtime_interface/src/tests/layout_engine_contracts.rs
  - zircon_runtime_interface/src/tests/ui_layout.rs
  - 2026-05-08: cargo test -p zircon_runtime_interface --lib layout_engine_contracts --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-layout-engine-m3 --message-format short --color never (3 passed; 0 failed; 73 filtered out)
  - 2026-05-08: cargo check -p zircon_runtime_interface --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-layout-engine-m3 --message-format short --color never (passed)
  - 2026-05-08: rustfmt --edition 2021 --check touched M3 layout-engine files (passed)
  - 2026-05-08: git diff --check -- touched M3 layout-engine files and docs (passed with CRLF conversion warnings only)
  - 2026-05-24: cargo check -p zircon_runtime_interface --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-masonry-interface --message-format short --color never (passed)
  - 2026-05-25: cargo test -p zircon_runtime_interface --lib ui_layout_engine_request_maps_current_container_contracts_to_engine_families --locked --jobs 1 --target-dir D:\cargo-targets\zircon-masonry-interface --message-format short --color never (1 passed; 0 failed; 114 filtered out)
  - 2026-06-06: cargo check -p zircon_runtime_interface --lib --tests --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-taffy-family-contract-0606 --message-format short --color never (passed)
  - 2026-06-06: cargo test -p zircon_runtime_interface --lib layout_engine_contracts --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-taffy-family-contract-0606 --message-format short --color never -- --nocapture --test-threads=1 (8 passed; 0 failed; 126 filtered out)
  - 2026-06-07: cargo check -p zircon_runtime_interface --lib --tests --locked --jobs 1 --target-dir D:\cargo-targets\zircon-canvas-stretched-interface-0607 --message-format short --color never (passed)
  - 2026-06-12: cargo check -p zircon_runtime_interface --lib --locked (passed)
  - 2026-06-12: cargo test -p zircon_runtime_interface --lib ui_layout_style_and_debug_packet_contracts_round_trip_with_defaults --locked --target-dir target/codex-editor-ui (passed)
  - 2026-06-12: cargo test -p zircon_runtime --lib style_mapping --locked --jobs 1 --target-dir target/codex-editor-ui-runtime --message-format short --color never -- --nocapture --test-threads=1 (passed, 2 passed)
  - 2026-06-16: Runtime 09 layout engine backend name cutover static validation: rustfmt --edition 2021 --check touched layout/editor source files (passed); Python py_compile ui_architecture_boundary.py (passed); direct ui_architecture_boundary_audit risks=[]; standalone runtime_absorption ui_architecture.rs 17/17; standalone status-output 2/2; old backend source-call scan found no production/editor uses. Cargo and broader behavior tests intentionally deferred.
doc_type: module-detail
---

# UI Layout Contracts

`zircon_runtime_interface::ui::layout` owns neutral layout DTOs shared by runtime, editor, and future layout engines. The M3 preflight slice adds layout-engine capability and selection reports only; it does not run layout, convert styles, add `taffy`, or change runtime/editor behavior.

## Reference Anchors

Bevy is the dominant reference for the M3 layout direction. `dev/bevy/crates/bevy_ui/src/layout/convert.rs` maps UI node fields into `taffy::style::Style` for flex, grid, block, overflow, size, padding, margin, border, gap, and placement. Zircon keeps the interface crate dependency-free by recording which layout family can be routed to a future Taffy-backed engine instead of storing Taffy types in the contract.

The Zircon Slate-style surface-frame contract remains the repository boundary reference. `docs/ui-and-layout/slate-style-ui-surface-frame.md` records parent-owned slot policy, overlay ordering, scroll virtualization, hit-grid authority, and arranged-frame sharing across render and hit testing. Those semantics are preserved as Zircon-owned even when flex/grid/wrap/block-compatible subtrees later route through Taffy.

## Slot DTOs

`UiCanvasSlotPlacement` is the parent-owned Free/Canvas placement payload. It stores minimum anchor, optional maximum anchor, pivot/alignment, local position, Slate-style offset margins, and auto-size intent. When `anchor_max` is absent it resolves to the minimum anchor for fixed-anchor placement; when present with a different axis value it represents the stretched anchor range consumed by runtime Free/Canvas arrange before falling back to child node defaults.

## Engine Capability DTOs

`UiLayoutEngineBackend` identifies the neutral backend choice: `Zircon` for current runtime behavior and `Taffy` for the future flex/grid/wrap/block engine. It is a report value, not a dependency on either implementation.

`UiLayoutEngineFamily` classifies layout requests into `Free`, `Container`, `Overlay`, `Flex`, `Grid`, `Block`, `Scrollable`, `Wrap`, `Masonry`, and `VirtualizedList`. `is_taffy_owned()` is the shared predicate for `Flex`, `Grid`, `Wrap`, and explicit `Block`; `is_zircon_owned()` is the shared predicate for `Free`, `Container`, `Overlay`, `Scrollable`, `Masonry`, and `VirtualizedList` because those families carry retained Slate-style positioning, clip, scroll, staggered column placement, or visible-range semantics that must not be hidden behind a generic Taffy conversion.

`UiLayoutEngineCapability` describes one backend's supported families plus whether it can participate in content measurement and DPI scaling. The built-in constructors intentionally model the planned boundary: `taffy_flex_grid_wrap_block()` supports flex, grid, wrap, and explicit block; `zircon()` supports the current shared contract inventory including the Block fallback path and Zircon-owned Masonry. The helper name is intentionally explicit so Wrap cannot drift out of the interface contract while the runtime Taffy bridge still solves it.

`UiLayoutEngineRequest::from_container_kind(...)` maps the current `UiContainerKind` contract into a family for future runtime routing. Horizontal and vertical boxes become `Flex`, grid boxes become `Grid`, wrap boxes become `Wrap`, Masonry boxes become `Masonry`, scroll boxes remain `Scrollable`, and scroll boxes with virtualization become `VirtualizedList`.

`UiLayoutEngineSelection` and `UiLayoutEngineSelectionReport` record whether a request was accepted natively, fell back, or was unsupported. Fallback reasons distinguish unsupported families, missing content measurement, missing DPI scaling, and Zircon-owned semantics. This gives M3 runtime slices a stable diagnostics surface before they wire real engine execution.

## Boundary

This module does not implement `UiLayoutEngine`, Taffy conversion, measure/arrange passes, dirty propagation, or `.ui.toml` schema expansion. Runtime `zircon_runtime::ui::layout` remains the owner of layout execution. Later M3 runtime work should use these DTOs to report engine selection while preserving the existing `UiArrangedTree` and `UiSurfaceFrame` outputs.

The focused tests in `zircon_runtime_interface/src/tests/layout_engine_contracts.rs` cover capability support, current container-to-family mapping, fallback selection, aggregate reporting, and serde round-trips. The 2026-05-24 interface check and 2026-05-25 focused family-mapping test passed after adding `UiMasonryBoxConfig`, `UiContainerKind::MasonryBox`, and the Zircon-owned `Masonry` engine family. The 2026-06-06 contract update hard-renamed the Taffy capability helper to `taffy_flex_grid_wrap_block()`, added `UiLayoutEngineFamily::is_taffy_owned()`, and made the runtime bridge consume that predicate instead of keeping a private duplicate family list. The 2026-06-07 Canvas slot update adds `UiCanvasSlotPlacement.anchor_max` so stretched Slate-style Canvas anchors can be serialized and consumed by runtime Free/Canvas arrange while fixed-anchor payloads keep their older default shape; the interface lib/tests check passed after the DTO documentation refresh.

The 2026-06-16 Runtime 09 layout engine backend name cutover hard-renamed the current runtime backend surface to `UiLayoutEngineBackend::Zircon`, `UiLayoutEngineCapability::zircon()`, and `zircon_selected_count`. The old backend names are intentionally not kept as compatibility aliases.

## Style and Debug DTOs

`UiLayoutStyle` is the editor UI style DTO introduced for the 2026-06-12 editor UI architecture slice. It records the shared layout subset used by flex/grid/block-compatible runtime layout: display, direction, wrap, justify/alignment fields, gap, flex sizing, grid tracks and placement, size constraints, aspect ratio, edges, positioning, inset, and overflow. The interface crate still does not depend on Taffy; conversion lives in `zircon_runtime::ui::layout::style_mapping`.

`UiLayoutDisplay` includes `Overlay`, `Canvas`, `Scroll`, and `Virtual` even though those are not Taffy-owned. Keeping those variants in the DTO lets assets and diagnostics describe mixed trees while runtime routing can explicitly preserve Zircon-owned semantics for retained placement, scroll windows, virtualization, and hit-grid authority.

`UiLayoutDebugPacket` and `UiLayoutDebugNode` give the future editor reflector a neutral payload for layout inspection. A node row can carry final geometry, constraints, chosen backend, fallback reason, and style source references without importing runtime-only structures into the interface crate.
