---
related_code:
  - zircon_editor/src/ui/workbench/autolayout/region_binding/mod.rs
  - zircon_editor/src/ui/workbench/autolayout/shell_regions_asset.rs
  - zircon_editor/src/ui/workbench/autolayout/workbench_skeleton.rs
  - zircon_editor/src/ui/workbench/layout/layout_command_error.rs
  - zircon_editor/src/ui/workbench/layout/manager/apply.rs
  - zircon_editor/src/ui/workbench/layout/manager/attach.rs
  - zircon_editor/src/ui/workbench/layout_preset.rs
  - zircon_editor/src/ui/host/layout_persistence.rs
  - zircon_editor/src/ui/host/editor_manager_layout.rs
  - zircon_editor/src/ui/workbench/page_layout_template.rs
  - zircon_editor/src/ui/workbench/floating_window.rs
  - zircon_editor/src/ui/host/layout_commands.rs
  - zircon_editor/src/tests/workbench/layout/layout_preset_persistence.rs
  - zircon_editor/src/tests/workbench/layout/page_layout_templates.rs
  - zircon_editor/assets/ui/editor/layout/shell_regions.toml
  - zircon_editor/assets/ui/editor/layout/presets.toml
  - zircon_editor/assets/ui/editor/layout/page_templates.toml
  - zircon_editor/assets/ui/editor/components/workbench/shell/workbench_skeleton.zui
  - zircon_editor/assets/ui/editor/components/workbench/shell/workbench_main_band.zui
  - zircon_editor/assets/ui/editor/components/workbench/shell/workbench_scene_tree_panel.zui
  - zircon_editor/assets/ui/editor/components/workbench/shell/workbench_inspector_panel.zui
implementation_files:
  - zircon_editor/src/ui/workbench/autolayout/region_binding/mod.rs
  - zircon_editor/src/ui/workbench/autolayout/region_binding/editor_region.rs
  - zircon_editor/src/ui/workbench/autolayout/region_binding/editor_region_role.rs
  - zircon_editor/src/ui/workbench/autolayout/region_binding/region_binding.rs
  - zircon_editor/src/ui/workbench/autolayout/region_binding/region_binding_error.rs
  - zircon_editor/src/ui/workbench/autolayout/region_binding/workbench_constraint_token_name.rs
  - zircon_editor/src/ui/workbench/autolayout/shell_regions_asset.rs
  - zircon_editor/src/ui/workbench/autolayout/workbench_skeleton.rs
  - zircon_editor/src/ui/workbench/layout/layout_command_error.rs
  - zircon_editor/src/ui/workbench/layout/manager/apply.rs
  - zircon_editor/src/ui/workbench/layout/manager/attach.rs
  - zircon_editor/src/ui/workbench/layout_preset.rs
  - zircon_editor/src/ui/host/layout_persistence.rs
  - zircon_editor/src/ui/host/editor_manager_layout.rs
  - zircon_editor/src/ui/workbench/page_layout_template.rs
  - zircon_editor/src/ui/workbench/floating_window.rs
  - zircon_editor/src/ui/host/layout_commands.rs
  - zircon_editor/src/tests/workbench/layout/layout_preset_persistence.rs
  - zircon_editor/src/tests/workbench/layout/page_layout_templates.rs
  - zircon_editor/assets/ui/editor/layout/shell_regions.toml
  - zircon_editor/assets/ui/editor/layout/presets.toml
  - zircon_editor/assets/ui/editor/layout/page_templates.toml
  - zircon_editor/assets/ui/editor/components/workbench/shell/workbench_skeleton.zui
  - zircon_editor/assets/ui/editor/components/workbench/shell/workbench_main_band.zui
  - zircon_editor/assets/ui/editor/components/workbench/shell/workbench_scene_tree_panel.zui
  - zircon_editor/assets/ui/editor/components/workbench/shell/workbench_inspector_panel.zui
  - zircon_editor/assets/ui/editor/components/workbench/floating/workbench_command_palette.zui
  - zircon_editor/assets/ui/editor/components/workbench/floating/workbench_preferences.zui
plan_sources:
  - docs/plans/zircon_editor/editor_layout/02-declarative-layout-interface.md
  - docs/plans/zircon_editor/editor_layout/03-jetbrains-docking-workbench.md
  - docs/plans/zircon_editor/editor_layout/04-layout-presets-and-persistence.md
  - docs/plans/zircon_editor/editor_layout/05-page-layout-templates.md
  - docs/plans/zircon_editor/editor_layout/06-floating-windows-and-design-parity.md
tests:
  - zircon_editor/src/tests/workbench/layout/editor_layout_contracts.rs
doc_type: module-detail
---

# Workbench Skeleton Contract

## Purpose

This document describes the declaration layer for the editor workbench layout. It translates the planned JetBrains-style shell into stable Rust DTOs and `.zui` or `.toml` assets without replacing the existing retained-host pointer, docking, or geometry code.

## Related Files

`region_binding/` owns the mapping from semantic regions to current drawer slots and shell geometry regions. `workbench_skeleton.rs` owns the default six-region shell declaration. `layout_command_error.rs` owns typed docking command failure reasons used by the layout manager. `layout_preset.rs`, `page_layout_template.rs`, and `floating_window.rs` own the planned preset, page, and floating-window declarations.

## Behavior Model

The layout contract uses six semantic regions: left-top, left-bottom, right-top, right-bottom, bottom, and center. The four side regions and bottom map to existing `ActivityDrawerSlot` values. Center maps to `ShellRegionId::Document`.

Each region has one fixed role:

- left-top: placement tools
- left-bottom: project tree
- right-top: hierarchy or structure
- right-bottom: detail inspector
- bottom: console, diagnostics, or timeline
- center: active document

`RegionBinding::new(...)` validates the role before accepting a panel asset. This keeps author-facing layout declarations from putting inspector content into a project-tree slot or console content into the center document slot.

The layout-owned skeleton and floating-window assets import `editor_tokens.zui` and reference `editor.*` token names for their shell chrome colors. That keeps new layout assets aligned with the design-language contract while older shell/module assets are migrated in later token cleanup slices.

`WorkbenchSkeleton::preferred_region_extents_from_tokens(...)` projects each declared region `size_token` into shell-region preferred extents. The result feeds the existing `compute_workbench_shell_geometry(..., transient_region_preferred)` path, so authored layout declarations can influence shell autolayout without adding another layout solver or hard-coded shell dimensions.

`WorkbenchShellRegionsAsset::from_toml_str(...)` parses the dedicated `shell_regions.toml` layout asset into a typed asset header plus validated `RegionBinding` rows. The loader rejects wrong asset kind/id/version, duplicate regions, missing regions, and region-role mismatches before the rows can replace the built-in skeleton regions. `WorkbenchSkeleton::from_shell_regions_asset(...)` keeps the default workbench chrome assets and swaps only the verified region declarations.

`workbench_main_band.zui`, `workbench_scene_tree_panel.zui`, and `workbench_inspector_panel.zui` now import `editor_tokens.zui` and reference `$--left-drawer-width` / `$--right-drawer-width` for fixed drawer widths. The old inline `332.0` and `404.0` drawer sizes are no longer the source for those shell assets.

The docking command model now treats drawer collapse, drawer tab activation, center split creation, and view focus as layout-owned state transitions. Collapsing a drawer keeps its tab stack but clears the active view. Activating a drawer tab restores the drawer to pinned mode and sets both the tab stack active tab and drawer active view. Center splits remain `DocumentNode::SplitNode` trees with active document tabs owned by the tab stack. Expected mutation failures return `LayoutCommandError` instead of an unstructured string inside the layout owner.

Layout preset persistence is scoped by `(user_id, page_id)`. A persisted entry stores a versioned `LayoutPreset`, not a full `WorkbenchLayout`. Capture records drawer modes, drawer extents through the existing drawer-width tokens, and the center split shape. It deliberately does not serialize view instance IDs, tab payloads, or module-owned state. Restore validates the stored version and falls back to the built-in Authoring preset on missing or mismatched entries. The editor host exposes explicit save/restore APIs and uses the default user path when `ActivateMainPage` switches the active page, saving the previous page and restoring the target page at the host boundary.

Page templates are declared in `PageLayoutTemplate::builtin_templates()` and mirrored by the dedicated `page_templates.toml` layout asset. The built-in set now covers 13 editor pages: scene, game, material, material preview, inspector, prefab, UI designer, UI source, animation timeline, animation graph, asset browser, console, and runtime diagnostics. Each template fills the same six semantic regions, chooses a default preset, declares default drawer modes, and optionally defines a center split profile. The templates do not own module data wiring; they only decide which panel asset belongs in which workbench region.

## Design And Rationale

The declaration layer is intentionally thin. Existing layout managers, drawer states, and retained-host surfaces keep runtime authority. The new model gives plans, tests, and future asset loaders a single vocabulary for workbench regions, size token names, built-in presets, page templates, and floating windows.

The code is split into small modules so root `mod.rs` files stay structural and future page or preset work can expand without becoming an umbrella implementation.

## Control Flow Or Data Flow

Authored layout assets provide region names, roles, panel asset IDs, and optional size tokens. Rust tests exercise the same shape through `RegionBinding`, `WorkbenchSkeleton`, `LayoutPreset`, `PageLayoutTemplate`, and `FloatingWindow`. Layout commands then mutate `WorkbenchLayout` through `LayoutManager::apply(...)`, with typed command errors converted to `EditorError::Layout` only at the editor host boundary. Page/user persistence captures the layout-facing preset subset into `LayoutPresetPersistenceStore`, serializes that store through the editor config layer, and reapplies it through `LayoutPreset::apply_to_layout(...)` when the host restores a page. Later slices can parse the assets into these DTOs and feed existing `WorkbenchLayout` and retained host projection code.

For the built-in skeleton path and the authored `shell_regions.toml` path, size-token projection is wired to shell autolayout by passing the extents map into the `transient_region_preferred` argument. The asset loader now proves external layout assets can drive the same DTOs; later integration can decide where the retained host resolves the asset source at runtime.

## Edge Cases And Constraints

Bottom-left and bottom-right are legacy slots in the current layout model. New layout declarations use the canonical bottom region only. Center has no drawer slot and is the only region that maps to `ShellRegionId::Document`.

## Test Coverage

`zircon_editor/src/tests/workbench/layout/editor_layout_contracts.rs` checks region validation, slot mapping, skeleton defaults, built-in presets, core page templates, floating-window modal/layer contracts, token-reference contracts for the new skeleton/floating assets, shell drawer width tokenization, the built-in skeleton-to-shell extents feed, authored `shell_regions.toml` loading, asset-role mismatch rejection, authored asset extents feeding shell geometry, docking collapse/activate/split/focus state, and typed layout command errors. `zircon_editor/src/tests/workbench/layout/layout_preset_persistence.rs` checks page/user scoped persistence, version mismatch fallback, round-trip serialization, drawer width restore, center split restore, and that persisted JSON does not contain view instance IDs. `zircon_editor/src/tests/workbench/layout/page_layout_templates.rs` checks the 13-page built-in template set, region role contracts, default state profiles, center split declarations, and parity with `page_templates.toml`. The first editor red run timed out in dependency compilation before reaching diagnostics. After the render mesh import drift was repaired, `cargo check -p zircon_editor --lib --offline --jobs 1 --target-dir E:\cargo-targets\zircon-editor-layout-editor-0623 --message-format short --color never` passed, `cargo test -p zircon_editor --lib editor_layout_contracts --no-run --offline --jobs 1 --target-dir E:\cargo-targets\zircon-editor-layout-editor-0623 --message-format short --color never` built the test binary, and directly running `editor_layout_contracts --test-threads=1 --nocapture` passed 8/8 tests. The 02.S2 asset-ingestion verification ran `cargo test -p zircon_editor --lib editor_layout_contracts --offline --jobs 1 --target-dir E:\cargo-targets\zircon-editor-layout-editor-0623 --message-format short --color never -- --test-threads=1 --nocapture` and passed 10/10 filtered tests. For 03.S2, earlier focused test/check attempts timed out after about 604 seconds. A later focused test rerun reached the `zircon_editor` lib-test build stage, then failed because Cargo could not write dep-info under the target-dir `.fingerprint` path. A clean target-dir rerun then exposed lower `zircon_runtime::ui::template::asset::compiler::style_apply` import drift from the active slot-contract split; source-level support repair restored the parent `style_apply` re-export for `slot_contract` helpers. The follow-up `zircon_runtime` check timed out after 606 seconds with no diagnostics. The 04.S2 verification ran `cargo test -p zircon_editor --lib layout_preset_persistence --offline --jobs 1 --target-dir E:\cargo-targets\zircon-editor-layout-editor-0623-clean-2309 --message-format short --color never -- --test-threads=1 --nocapture` and passed 2/2 filtered tests after a lower `editor_showcase` helper split import was repaired. The 05.S2 verification ran `cargo test -p zircon_editor --lib page_layout_templates --offline --jobs 1 --target-dir E:\cargo-targets\zircon-editor-layout-editor-0623-clean-2309 --message-format short --color never -- --test-threads=1 --nocapture` and passed 4/4 filtered tests after lower runtime UI surface split imports/visibility were repaired.

## Plan Sources

This module document covers `02.S1`, the verified `02.S2` shell token-extents and authored asset ingestion segments, `03.S1`, the implemented/static-verified `03.S2` docking command segment, `04.S1`, the focused-verified `04.S2` page/user persistence segment, `05.S1`, the focused-verified `05.S2` page template segment, and `06.S1` layout architecture slices.

## Open Issues Or Follow-up

The current work creates the declaration layer and assets, both built-in and authored shell-region declarations feed size-token extents into shell autolayout, core docking commands now expose typed state-transition failures, page/user layout persistence now restores the preset subset of layout state, and all 13 planned page templates now have default region/state declarations. Later slices still need retained-host runtime source resolution for the authored asset, a lower runtime Cargo rerun plus editor-layout Cargo rerun for 03.S2, remaining hard-coded shell dimension cleanup outside these drawer assets, and visual design parity checks against the design references.
