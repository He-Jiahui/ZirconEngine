---
related_code:
  - zircon_runtime_interface/src/ui/layout/mod.rs
  - zircon_runtime_interface/src/ui/layout/engine.rs
  - zircon_runtime_interface/src/ui/layout/constraints.rs
  - zircon_runtime_interface/src/ui/layout/slot.rs
  - zircon_runtime_interface/src/ui/layout/scroll.rs
  - zircon_runtime_interface/src/ui/layout/linear_sizing.rs
  - zircon_runtime_interface/src/tests/layout_engine_contracts.rs
  - dev/bevy/crates/bevy_ui/src/layout/convert.rs
  - docs/ui-and-layout/slate-style-ui-surface-frame.md
implementation_files:
  - zircon_runtime_interface/src/ui/layout/mod.rs
  - zircon_runtime_interface/src/ui/layout/engine.rs
plan_sources:
  - .codex/plans/Bevy-Informed Zircon UI 架构优化里程碑计划.md
  - docs/ui-and-layout/bevy-informed-ui-m0-gap-audit.md
  - user: 2026-05-08 continue M3 layout-engine interface preflight slice
tests:
  - zircon_runtime_interface/src/tests/layout_engine_contracts.rs
  - 2026-05-08: cargo test -p zircon_runtime_interface --lib layout_engine_contracts --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-layout-engine-m3 --message-format short --color never (3 passed; 0 failed; 73 filtered out)
  - 2026-05-08: cargo check -p zircon_runtime_interface --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-layout-engine-m3 --message-format short --color never (passed)
  - 2026-05-08: rustfmt --edition 2021 --check touched M3 layout-engine files (passed)
  - 2026-05-08: git diff --check -- touched M3 layout-engine files and docs (passed with CRLF conversion warnings only)
doc_type: module-detail
---

# UI Layout Contracts

`zircon_runtime_interface::ui::layout` owns neutral layout DTOs shared by runtime, editor, and future layout engines. The M3 preflight slice adds layout-engine capability and selection reports only; it does not run layout, convert styles, add `taffy`, or change runtime/editor behavior.

## Reference Anchors

Bevy is the dominant reference for the M3 layout direction. `dev/bevy/crates/bevy_ui/src/layout/convert.rs` maps UI node fields into `taffy::style::Style` for flex, grid, block, overflow, size, padding, margin, border, gap, and placement. Zircon keeps the interface crate dependency-free by recording which layout family can be routed to a future Taffy-backed engine instead of storing Taffy types in the contract.

The Zircon Slate-style surface-frame contract remains the repository boundary reference. `docs/ui-and-layout/slate-style-ui-surface-frame.md` records parent-owned slot policy, overlay ordering, scroll virtualization, hit-grid authority, and arranged-frame sharing across render and hit testing. Those semantics are preserved as Zircon-owned even when flex/grid/block-compatible subtrees later route through Taffy.

## Engine Capability DTOs

`UiLayoutEngineBackend` identifies the neutral backend choice: `LegacyZircon` for current runtime behavior and `Taffy` for the future flex/grid/block engine. It is a report value, not a dependency on either implementation.

`UiLayoutEngineFamily` classifies layout requests into `Free`, `Container`, `Overlay`, `Flex`, `Grid`, `Block`, `Scrollable`, `Wrap`, and `VirtualizedList`. `Free`, `Container`, `Overlay`, `Scrollable`, and `VirtualizedList` are marked Zircon-owned because they carry retained Slate-style positioning, clip, scroll, or visible-range semantics that must not be hidden behind a generic Taffy conversion.

`UiLayoutEngineCapability` describes one backend's supported families plus whether it can participate in content measurement and DPI scaling. The built-in constructors intentionally model the planned boundary: `taffy_flex_grid_block()` supports flex, grid, block, and wrap; `legacy_zircon()` supports the current shared contract inventory.

`UiLayoutEngineRequest::from_container_kind(...)` maps the current `UiContainerKind` contract into a family for future runtime routing. Horizontal and vertical boxes become `Flex`, grid boxes become `Grid`, wrap boxes become `Wrap`, scroll boxes remain `Scrollable`, and scroll boxes with virtualization become `VirtualizedList`.

`UiLayoutEngineSelection` and `UiLayoutEngineSelectionReport` record whether a request was accepted natively, fell back, or was unsupported. Fallback reasons distinguish unsupported families, missing content measurement, missing DPI scaling, and Zircon-owned semantics. This gives M3 runtime slices a stable diagnostics surface before they wire real engine execution.

## Boundary

This module does not implement `UiLayoutEngine`, Taffy conversion, measure/arrange passes, dirty propagation, or `.ui.toml` schema expansion. Runtime `zircon_runtime::ui::layout` remains the owner of layout execution. Later M3 runtime work should use these DTOs to report engine selection while preserving the existing `UiArrangedTree` and `UiSurfaceFrame` outputs.

The focused tests in `zircon_runtime_interface/src/tests/layout_engine_contracts.rs` cover capability support, current container-to-family mapping, fallback selection, aggregate reporting, and serde round-trips. The 2026-05-08 scoped interface gate passed for the focused contract tests and crate check.
