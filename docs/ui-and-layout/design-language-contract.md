---
related_code:
  - zircon_runtime_interface/src/ui/design_tokens.rs
  - zircon_runtime_interface/src/ui/style.rs
  - zircon_editor/assets/ui/editor/theme/editor_tokens.zui
  - zircon_editor/assets/ui/editor/components/workbench/shell/workbench_skeleton.zui
  - zircon_editor/assets/ui/editor/components/workbench/shell/workbench_main_band.zui
  - zircon_editor/assets/ui/editor/components/workbench/shell/workbench_scene_tree_panel.zui
  - zircon_editor/assets/ui/editor/components/workbench/shell/workbench_inspector_panel.zui
  - zircon_editor/assets/ui/editor/components/workbench/floating/workbench_command_palette.zui
  - zircon_editor/assets/ui/editor/components/workbench/floating/workbench_preferences.zui
  - zircon_editor/src/ui/workbench/floating_window.rs
  - zircon_editor/tests/integration_contracts/floating_window_design_parity.rs
implementation_files:
  - zircon_runtime_interface/src/ui/design_tokens.rs
  - zircon_editor/assets/ui/editor/theme/editor_tokens.zui
  - zircon_editor/assets/ui/editor/components/workbench/shell/workbench_skeleton.zui
  - zircon_editor/assets/ui/editor/components/workbench/shell/workbench_main_band.zui
  - zircon_editor/assets/ui/editor/components/workbench/shell/workbench_scene_tree_panel.zui
  - zircon_editor/assets/ui/editor/components/workbench/shell/workbench_inspector_panel.zui
  - zircon_editor/assets/ui/editor/components/workbench/floating/workbench_command_palette.zui
  - zircon_editor/assets/ui/editor/components/workbench/floating/workbench_preferences.zui
plan_sources:
  - docs/plans/zircon_editor/editor_layout/01-design-tokens-and-language-contract.md
  - docs/plans/zircon_editor/editor_layout/06-floating-windows-and-design-parity.md
tests:
  - zircon_runtime_interface/src/tests/editor_design_tokens.rs
  - zircon_editor/tests/integration_contracts/floating_window_design_parity.rs
doc_type: module-detail
---

# Editor Design Language Contract

## Purpose

This document defines the editor workbench visual language as a code-facing contract. The contract turns the `editor_layout` plan and the workbench design notes into a token source that runtime UI, retained editor surfaces, `.zui` assets, and validation tests can all reference.

## Related Files

- `zircon_runtime_interface/src/ui/design_tokens.rs` owns the serializable `EditorDesignTokens` DTO and the fixed workbench-dark defaults.
- `zircon_editor/assets/ui/editor/theme/editor_tokens.zui` is the authored asset mirror for the same token groups.
- `zircon_runtime_interface/src/ui/style.rs` remains the painter-state selector and neutral theme document owner.

## Behavior Model

The editor token model has five groups: palette, typography, controls, density, and state roles. Palette stores the near-black surface ladder, the teal accent, border, and text colors. Typography stores logical font families, weights, smoothing, line height, and 96-DPI logical-pixel sizes. Controls store the 28-32 px dense control heights, low radii, and 1 px border width. Density stores gaps, drawer padding, row height, and the shell size tokens for left drawer, right drawer, and bottom output.

The workbench typography defaults preserve Unreal Starship's authored 10/8/14 point scale. Unreal's `SlateFontInfo.h` defines font size in points and converts it to Slate Units at 96 DPI, so the shared Zircon token values are 13.33/10.67/18.67 logical pixels rather than 10/8/14 pixels. DPI scaling happens later at the runtime text raster boundary; controls must not convert these logical values a second time.

State roles map resolved painter states to color roles without changing `UiPainterStyleSelector` priority. Disabled still wins before loading, pressed, selected/focused, hovered, and normal. Tokens only decide what color each already-resolved state uses.

`EditorDesignTokens::resolve_painter_style(...)` is the feed path from semantic painter state to tokenized visual values. It calls `UiPainterStyleSelector::resolved_state_for_family(...)` first, then maps the resolved family/state to palette, foreground, border, radius, border-width, and control-height values. This keeps selector priority in `style.rs` and moves workbench colors/density into the editor token contract.

`EditorDesignTokens::density_value_for_token_name(...)` is the density-side feed path for layout declarations. It resolves both canonical `editor.density.*` names and the shell constraint aliases `--left-drawer-width`, `--right-drawer-width`, and `--bottom-output-height` into the same central density values.

## Design And Rationale

The design keeps token data in `zircon_runtime_interface` because editor, runtime UI, and plugins need a stable DTO. It does not move selector behavior out of `style.rs`; selector priority is already a shared contract and remains separate from theme values.

The token defaults deliberately project into `UiThemeDocument` so existing style consumers can adopt the workbench contract without bypassing current theme plumbing.

## Edge Cases And Constraints

The token asset is allowed to contain literal color values because it is the source of truth. Component `.zui` files should reference token names as later slices remove duplicated naked colors from component definitions.

The S2 hard cutover has started with the layout-owned skeleton/floating assets and the shell drawer width declarations. `workbench_skeleton.zui`, `command_palette.zui`, and `preferences.zui` import `editor_tokens.zui` and use `editor.surface.*`, `editor.text.*`, and `editor.border` token names instead of local hex colors. `workbench_main_band.zui`, `workbench_scene_tree_panel.zui`, and `workbench_inspector_panel.zui` import the same token asset and use `$--left-drawer-width` / `$--right-drawer-width` instead of local drawer pixel widths. Older shell/module workbench assets still contain historical literal colors and remain explicitly open for the wider cleanup slice.

`editor_tokens.zui` mirrors the converted logical typography sizes. The serialized values are already logical pixels, not point values, so asset loading and appearance preferences use the same units as `EditorTypographyTokens`.

## Floating Window Design Parity Checklist

`FloatingWindowDesignContract` is the code-facing checklist for command palette, preferences, and detached editor windows. Every floating window must declare its layer, modality, placement, content layout, and interaction mode instead of relying on ad hoc retained-host placement.

The command palette contract is a top overlay, non-modal, top-center, keyboard-driven vertical layout. Its asset must keep tokenized low chrome, a search input, a first result row, fixed 32 px search height, fixed 28 px row height, and bounded panel height.

The preferences contract is a modal overlay centered over the workbench. Its asset must keep tokenized low chrome and a left navigation plus right content structure, with navigation and content panels using the same surface/border token rules as the shell.

All floating assets must import `editor_tokens.zui`, avoid naked hex colors, avoid gradient/shadow/glow/blur effects, use 1 px `editor.border`, and keep radius at or below 8 px. The focused integration contract parses the real `.zui` assets directly so this checklist fails when the authored assets drift.

## Test Coverage

`zircon_runtime_interface/src/tests/editor_design_tokens.rs` checks palette values, density values, shell constraint token lookup, state-role mapping, projection into `UiThemeDocument`, and the `resolve_painter_style(...)` feed path. The first red run for S1 reached compile and failed on the missing `crate::ui::design_tokens` module before implementation. The S2 focused red run failed on missing `EditorDesignTokens::resolve_painter_style(...)`, then passed after the feed path landed. On 2026-06-23, `cargo test -p zircon_runtime_interface --lib editor_design_tokens --offline --jobs 1 --target-dir E:\cargo-targets\zircon-editor-layout-token-feed-0623 --message-format short --color never` passed 5/5 filtered token tests.

`zircon_editor/src/tests/workbench/layout/editor_layout_contracts.rs` includes static asset contracts for the skeleton/floating assets and shell drawer width tokenization. After the render mesh import drift was repaired, `cargo check -p zircon_editor --lib --offline --jobs 1 --target-dir E:\cargo-targets\zircon-editor-layout-editor-0623 --message-format short --color never` passed, `cargo test -p zircon_editor --lib editor_layout_contracts --no-run --offline --jobs 1 --target-dir E:\cargo-targets\zircon-editor-layout-editor-0623 --message-format short --color never` built the test binary, and directly running `editor_layout_contracts --test-threads=1 --nocapture` passed 8/8 tests.

`zircon_editor/tests/integration_contracts/floating_window_design_parity.rs` checks the 06.S2 floating-window design parity contract. On 2026-06-23, `cargo test -p zircon_editor --test integration_contracts --features integration-contracts floating_window_design_parity --offline --jobs 1 --target-dir E:\cargo-targets\zircon-editor-layout-runtime-state-reducer-0623 --message-format short --color never -- --test-threads=1 --nocapture` passed 4/4 tests after `zircon_runtime` first passed a fresh lower support check in the same target directory.

## Plan Sources

This document implements `01.S1 token 资产骨架 + 契约文档`, records the first `01.S2` token feed, the density token lookup used by `02.S2`, the layout-owned asset tokenization step, and the focused `06.S2` floating-window design parity checklist.

## Open Issues Or Follow-up

`01.S2` still needs the wider hard cutover from historical shell/module component-local naked colors to token references at the retained painter boundary. The current S2 row only closes the runtime-interface feed API and the layout-owned skeleton/floating assets. `06.S2` closes the static/code contract for floating-window design parity; retained-host screenshot or pixel comparison remains pending until the window harness is stable.
