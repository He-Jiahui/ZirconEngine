---
related_code:
  - zircon_runtime_interface/src/ui/design_tokens.rs
  - zircon_runtime_interface/src/ui/style.rs
  - zircon_editor/assets/ui/editor/theme/editor_tokens.zui
  - zircon_editor/assets/ui/editor/components/workbench/shell/workbench_skeleton.zui
  - zircon_editor/assets/ui/editor/components/workbench/shell/workbench_main_band.zui
  - zircon_editor/assets/ui/editor/components/workbench/shell/workbench_scene_tree_panel.zui
  - zircon_editor/assets/ui/editor/components/workbench/shell/workbench_inspector_panel.zui
  - zircon_editor/assets/ui/editor/components/workbench/shell/workbench_viewport_panel.zui
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
  - zircon_editor/assets/ui/editor/components/workbench/shell/workbench_viewport_panel.zui
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
- `zircon_runtime/src/ui/surface/render/dialog.rs` and `zircon_runtime/src/ui/surface/render/notification_center.rs` project token defaults into generic composite render commands.
- `zircon_runtime/src/ui/surface/render/chrome.rs` projects tokenized surface state plus density-derived text, icon, and separator metrics into generic workbench shell, panel, toolbar, status-bar, and viewport commands.
- `zircon_editor/assets/ui/editor/components/workbench/composites/chrome/workbench_panel_header.zui` is the shared tokenized title/action surface for generic workbench panels.
- `zircon_runtime/src/ui/surface/render/feedback.rs` and `feedback/colors.rs` project token defaults into generic Alert, Tooltip, and Toast commands.
- `zircon_runtime/src/ui/surface/render/command_palette.rs` projects token defaults and validated template overrides into the generic command search panel.
- `zircon_runtime/src/ui/surface/render/popup_rows.rs` projects the shared popup, selected, hover, disabled, danger, typography, and density roles into menu, dropdown, and command-list rows.
- `zircon_runtime/src/ui/surface/render/dropdowns.rs` projects the shared token model into generic dropdown and select triggers.
- `zircon_runtime/src/ui/surface/render/collection_rows/` projects one shared tokenized state model into ListRow, TreeRow, and TableRow render commands, while each child keeps only its domain geometry such as tree depth or table column allocation.
- `zircon_runtime/src/ui/surface/render/buttons.rs` projects Starship-style flat button state roles, validated overrides, density geometry, and shared text metrics into generic Button, ToggleButton, and IconButton commands.
- `zircon_runtime/src/ui/surface/render/text_fields.rs` projects tokenized input states and logical typography into InputField, TextField, LineEdit, TextEdit, NumberField, and SearchField while retaining the shared text-layout cache and IME preedit path.
- `zircon_runtime/src/ui/surface/render/selection_controls.rs` projects the shared palette, density, control geometry, and typography into Checkbox, Radio, Toggle, and Switch without a local selection-control palette.
- `zircon_runtime/src/ui/surface/render/drag_overlay.rs` projects drag-preview and drop-indicator colors, radius, density geometry, and overlay typography from the same token model.
- `zircon_editor/assets/ui/editor/components/workbench/primitives/feedback/workbench_dialog.zui`, `workbench_confirm_dialog.zui`, and `workbench_notification_center.zui` declare the authored visual token inputs for those composites.
- `workbench_alert.zui`, `workbench_tooltip.zui`, `workbench_toast.zui`, and `workbench_drag_overlay.zui` declare the authored feedback token inputs.

## Behavior Model

The editor token model has five groups: palette, typography, controls, density, and state roles. Palette stores the near-black surface ladder, the teal accent, border, and text colors. Typography stores logical font families, role weights, smoothing, line height, and 96-DPI logical-pixel sizes. Controls store 28-32 px dense command heights, a 48 px large activity-rail hit target, low radii, and a 1 px border width. Density stores gaps, drawer padding, row height, the 72 px activity-rail shell width, and the shell size tokens for left drawer, right drawer, and bottom output.

The workbench typography defaults preserve Unreal Starship's authored 10/8/9/14 point scale. Unreal's `SlateFontInfo.h` defines font size in points and converts it to Slate Units at 96 DPI, so the shared Zircon body/caption/overlay/title values are 13.33/10.67/12.00/18.67 logical pixels rather than 10/8/9/14 pixels. `medium=500` serves regular labels, `strong=600` serves section text, and `emphasis=700` serves compact viewport overlays. DPI scaling happens later at the runtime text raster boundary; controls must not convert these logical values a second time.

State roles map resolved painter states to color roles without changing `UiPainterStyleSelector` priority. Disabled still wins before loading, pressed, selected/focused, hovered, and normal. Tokens only decide what color each already-resolved state uses.

`EditorDesignTokens::resolve_painter_style(...)` is the feed path from semantic painter state to tokenized visual values. It calls `UiPainterStyleSelector::resolved_state_for_family(...)` first, then maps the resolved family/state to palette, foreground, border, radius, border-width, and control-height values. This keeps selector priority in `style.rs` and moves workbench colors/density into the editor token contract.

`EditorDesignTokens::density_value_for_token_name(...)` is the density-side feed path for layout declarations. It resolves both canonical `editor.density.*` names and the shell constraint aliases `--left-drawer-width`, `--right-drawer-width`, and `--bottom-output-height` into the same central density values.

Generic Dialog, ConfirmDialog, and NotificationCenter composites use a cached workbench token projection as their default visual source. Their authored `.zui` assets pass palette, semantic severity colors, border/radius, typography sizes, and the shared line-height ratio as `$editor.*` values. ConfirmDialog uses the same source for the destructive confirm action, neutral cancel action, severity mark, and shared runtime text output. When a compiled template presents a resolved color or finite numeric override, it takes precedence over the default projection; malformed, unresolved, or non-finite values fall back to the central token value. This keeps theme customization possible without giving a template a second hard-coded palette or allowing invalid values into render extraction.

Generic Chrome uses the same projection for the shell, panel, panel header, activity rail, toolbar, status bar, and viewport defaults. The activity rail is the deliberately stable narrow band: its root width resolves through `editor.density.activity_rail_width`, while every activity action consumes `editor.control.height.large` on both axes. A `PanelHeader` keeps title and action areas as Stretch slots while its surface, soft separator, border width, and dense-to-compact height range resolve through the same tokens. Chrome text inset, icon size/gap, separator thickness, body font size, and line height derive from density, control, and typography tokens instead of local renderer constants; valid authored overrides may adjust those values through `layout_padding_*`, `layout_icon_size`, `layout_spacing`, `separator_thickness`, and logical text metrics. The top toolbar and status bar roots declare the same state surface, separator, foreground, and metric inputs, so they remain flat Chrome variants while retaining their own module/action and responsive-tier data. Generic feedback maps each severity to the central semantic container and foreground colors, while Tooltip and Toast reuse popup, neutral text, accent, disabled, hover, and pressed roles. Feedback accepts the historical `text_color` and `status_mark_color` aliases in addition to the normalized foreground and mark properties, so existing authored assets remain effective while resolving through one visual model.

WorkbenchChip is a compact noninteractive chrome label: its horizontal inset uses `editor.density.gap.medium`, its vertical inset uses `editor.density.gap.small`, and its body text and dense-to-default height range remain token-derived. It retains only content-width bounds locally, so parent layouts can reserve chip space without recreating a per-component spacing scale.

Generic Image commands use the neutral retained-palette border and `HostControlMetrics` border/radius projection rather than a focus-ring outline or local geometry. Raw decoded image-pixel commands remain unframed because asset-thumbnail painters own their preview treatment separately.

Generic CommandPalette is the shared search-and-select composite. Its popup shell, search field, query and placeholder text, panel/search radii, border width, text line height, gaps, and row extent all derive from `EditorDesignTokens`. Authored templates may provide resolved CSS colors and finite metrics for the documented visual inputs; invalid colors, non-finite numbers, negative spacing, and nonpositive extents retain central defaults. The command list then delegates its interaction states to `PopupRow`, keeping focused-only rows neutral, selected or hover rows low-emphasis teal, and disabled/loading rows readable without a second local palette.

Generic Dropdown and Select own only the trigger: its recessed, hovered, pressed, open, and unavailable surfaces, border precedence, caret, compact label/value text, and open indicator all project from the same central token groups. Popup options continue through `PopupRow`, so keyboard focus remains neutral until hover while open selection uses the shared teal role. The trigger accepts validated resolved color and finite metric overrides, including typography line-height ratios; it falls back to the central projection for malformed colors or invalid dimensions.

Generic ListRow, TreeRow, and TableRow share neutral normal and focus-only surfaces, restrained hover, selected, selected-hover, pressed, and unavailable roles. Selection plus hover uses the separate accent-soft role, so selected assets remain legible without treating every hover as primary emphasis. Insets, action reservation, tree indentation, line height, and logical text size derive from density and `EditorTypographyTokens`; only row data, tree depth, and table column ratios remain local runtime inputs. Their emitted text commands carry the tokenized logical metrics into the existing runtime text layout and cache path instead of creating a second editor text measurement route.

Generic Button, ToggleButton, and IconButton use the same flat, low-radius state model: a near-black surface ladder for neutral controls, teal only for primary emphasis, active selection, and focus, and semantic red only for destructive controls. Their text uses `EditorTypographyTokens` logical body size and line-height rather than a local raster-sized constant. `WorkbenchRailButton` uses the central `editor.control.height.large` token on both axes to retain a 48 px activity-rail hit target while generic IconButton extraction continues to calculate its icon from the normal dense-control visual metrics. Authored assets can override documented resolved colors and finite geometry through the normal style-overrides path; malformed colors, non-finite values, negative spacing, and nonpositive sizes keep the centralized default. This mirrors the Starship normal, hovered, pressed, and disabled response without gradients, glow, or per-component palettes.

Generic text fields use recessed normal/focused surfaces, a restrained hover surface, a one-pixel border that becomes teal at focus, and logical body text from `EditorTypographyTokens`. The generic style resolver accepts a positive absolute `line_height` first, then a positive `line_height_ratio` multiplied by the resolved logical font size; nonpositive values retain the default logical line height. This lets primitive labels such as StatusItem and editable fields share the existing Runtime Text layout and cached measurement path while preserving editing, selection, preedit composition, clipping, and DPI-aware text work owned by the runtime text plans. Caption and Label consume the central `editor.density.gap.xsmall` token for their compact vertical inset instead of carrying local padding constants. Search, regular, numeric, and component-property field assets share these roles and only retain domain values such as numeric min/max/step or the property-row label column locally. Valid style overrides can adjust field colors, padding, borders, radius, and line-height; malformed colors and invalid metrics fall back to the central projection.

Checkbox, Radio, Toggle, and Switch use the same low-chrome state vocabulary: a recessed inactive mark, low-emphasis selected teal surface, teal focus/active edge, and disabled surface/text roles. Their mark, radio dot, track, thumb, label inset, and label gap derive from control height, density gaps, and border thickness rather than authored pixel geometry. Labels use the logical body text role. Templates may provide valid resolved colors and finite control metrics, while malformed colors, nonfinite values, negative distances, and nonpositive extents retain the central visual defaults.

DragOverlay uses the shared low-emphasis accent container for an allowed preview and the semantic error container for a rejected preview. Preview icon spacing, cursor offset, indicator thickness, and logical overlay text derive from density, control, and typography tokens. It keeps drop-target coordinates as runtime data while validating authored colors and metrics before they enter the extracted render commands.

SegmentedControl and Tab use the shared surface ladder, accent, low-radius control edge, density gaps, and logical typography instead of a private palette. `WorkbenchTab` is a generic ToggleButton projection: its neutral, hover, pressed, selected, focus, and disabled roles are authored as token inputs, and its default width is Stretch so tab strips and toolbar parents allocate space rather than inheriting a fixed local width. Segment widths are calculated from the available body frame and option count, while label presence only reserves the token-derived label block. Focus edge, selected segment border, and selected underline remain separate visual inputs so existing templates can independently override `focus_border_color`, `selected_border_color` with `selected_border_width`, and `selected_underline_color`; invalid colors or dimensions fall back to the common token projection. Focus alone leaves the surface neutral, while hover, pressed, unavailable, and selected states follow the resolved painter state.

Slider and RangeSlider keep Slate's quiet adjustment language: the input rail is a token-derived 4px neutral surface, the normal fill is `separator.strong`, the thumb is primary text, and focus or drag exposes a weak primary-text halo instead of turning the entire control teal. Validation alone maps the fill to semantic warning or error. Track, value chip, ticks, thumb, disabled roles, logical text, and visual metrics resolve from the shared palette/control/density/typography groups; percent/range endpoints and tick budgets remain runtime data. RangeSlider calculates both thumb locations from the available track and only emits the extra lower value chip once its allocated height can contain it, so compact layouts degrade without pixel-positioned variants.

Linear Progress uses the same restrained workbench language without borrowing Slider interaction chrome: a recessed 4px track, accent fill, soft separator edge, and semantic warning/error only when the supplied status asks for it. Track width is the available frame width after the tokenized horizontal inset, while data `value_percent` or `value/min/max` determines only the fill fraction. Optional labels use the shared runtime text extraction and are emitted above the track only when the allocated height can fit the logical line height and tokenized gap; compact frames retain the centered track instead of clipping or pixel-positioned variants. Disabled roles, finite style overrides, and invalid-value fallback all resolve through the central token projection. Nonlinear variants remain available to the generic fallback until their own renderer is defined.

## Design And Rationale

The design keeps token data in `zircon_runtime_interface` because editor, runtime UI, and plugins need a stable DTO. It does not move selector behavior out of `style.rs`; selector priority is already a shared contract and remains separate from theme values.

The token defaults deliberately project into `UiThemeDocument` so existing style consumers can adopt the workbench contract without bypassing current theme plumbing.

## Edge Cases And Constraints

The token asset is allowed to contain literal color values because it is the source of truth. Component `.zui` files should reference token names as later slices remove duplicated naked colors from component definitions.

The S2 hard cutover has started with the layout-owned skeleton/floating assets and the shell drawer width declarations. `workbench_skeleton.zui`, `command_palette.zui`, and `preferences.zui` import `editor_tokens.zui` and use `editor.surface.*`, `editor.text.*`, and `editor.border` token names instead of local hex colors. `workbench_main_band.zui`, `workbench_scene_tree_panel.zui`, and `workbench_inspector_panel.zui` import the same token asset and use canonical `$editor.density.left_drawer_width` / `$editor.density.right_drawer_width` values instead of local drawer pixel widths; `$--` forms remain runtime-only compatibility aliases. Older shell/module workbench assets still contain historical literal colors and remain explicitly open for the wider cleanup slice.

`editor_tokens.zui` mirrors the converted logical typography sizes and role weights. The serialized values are already logical pixels, not point values, so asset loading and appearance preferences use the same units as `EditorTypographyTokens`. Viewport axis and gizmo labels use `editor.typography.overlay.size` and `editor.typography.emphasis.weight` rather than retaining local `12px/700` values.

## Floating Window Design Parity Checklist

`FloatingWindowDesignContract` is the code-facing checklist for command palette, preferences, and detached editor windows. Every floating window must declare its layer, modality, placement, content layout, and interaction mode instead of relying on ad hoc retained-host placement.

The command palette contract is a top overlay, non-modal, top-center, keyboard-driven vertical layout. Its asset must keep tokenized low chrome, a search input, a first result row, fixed 32 px search height, fixed 28 px row height, and bounded panel height.

The preferences contract is a modal overlay centered over the workbench. Its asset must keep tokenized low chrome and a left navigation plus right content structure, with navigation and content panels using the same surface/border token rules as the shell.

All floating assets must import `editor_tokens.zui`, avoid naked hex colors, avoid gradient/shadow/glow/blur effects, use 1 px `editor.border`, and keep radius at or below 8 px. The focused integration contract parses the real `.zui` assets directly so this checklist fails when the authored assets drift.

## Test Coverage

`zircon_runtime_interface/src/tests/editor_design_tokens.rs` checks palette values, density values, shell constraint token lookup, state-role mapping, projection into `UiThemeDocument`, and the `resolve_painter_style(...)` feed path. It also asserts that the 48 px `editor.control.height.large` activity-rail token reaches both its canonical registry entry and mechanical cascade alias. The first red run for S1 reached compile and failed on the missing `crate::ui::design_tokens` module before implementation. The S2 focused red run failed on missing `EditorDesignTokens::resolve_painter_style(...)`, then passed after the feed path landed. On 2026-06-23, `cargo test -p zircon_runtime_interface --lib editor_design_tokens --offline --jobs 1 --target-dir E:\cargo-targets\zircon-editor-layout-token-feed-0623 --message-format short --color never` passed 5/5 filtered token tests.

`zircon_editor/src/tests/workbench/layout/editor_shell_asset_contracts.rs` owns static contracts for the skeleton/floating assets, shell drawer width tokenization, and viewport overlay typography. `editor_design_token_contracts.rs` owns primitive and composite token contracts, while `editor_layout_contracts.rs` retains geometry and behavior assertions. After the render mesh import drift was repaired, `cargo check -p zircon_editor --lib --offline --jobs 1 --target-dir E:\cargo-targets\zircon-editor-layout-editor-0623 --message-format short --color never` passed, `cargo test -p zircon_editor --lib editor_layout_contracts --no-run --offline --jobs 1 --target-dir E:\cargo-targets\zircon-editor-layout-editor-0623 --message-format short --color never` built the test binary, and directly running `editor_layout_contracts --test-threads=1 --nocapture` passed 8/8 tests.

`zircon_runtime/src/ui/tests/render_dialog.rs` and `render_notification_center.rs` own generic composite render contracts: central defaults, severity/disabled behavior, destructive confirmation, template visual overrides, typography projection, and owner-text suppression. `editor_design_token_contracts.rs` also requires the corresponding authored feedback assets, including ConfirmDialog, to import `editor_tokens.zui`, use the declared semantic token set, and avoid component-local hex colors.

`zircon_runtime/src/ui/tests/render_chrome.rs` and `render_feedback.rs` own equivalent generic render contracts for workbench chrome and feedback primitives. Chrome tests lock central state defaults plus token-derived icon, text, separator, and logical typography metrics while retaining explicit valid template overrides and state precedence; `editor_design_token_contracts.rs` requires the top toolbar and status bar roots to expose the matching token inputs. Feedback contracts also cover the legacy text/mark aliases consumed by existing `.zui` assets.

`zircon_runtime/src/ui/tests/render_command_palette.rs` and `render_popup_menu.rs` own the generic command-search and popup-row contracts. They cover central defaults, tokenized typography/density, focused-only versus hovered/selected states, borrowed command filtering, and valid visual/metric overrides. `editor_design_token_contracts.rs` requires `workbench_command_palette.zui` to import the editor token asset and expose the corresponding popup, surface, text, control, density, and typography inputs.

`zircon_runtime/src/ui/tests/render_dropdowns.rs` owns trigger-state precedence, token defaults, label/value typography, unavailable visuals, and valid visual/metric overrides. The primitive contract requires `workbench_dropdown.zui` to expose the same surface, border, text, control, density, and typography token inputs as the generic trigger.

`zircon_runtime/src/ui/tests/render_collection_rows.rs` owns ListRow, TreeRow, and TableRow extraction contracts: selected versus selected-hover state separation, focus-neutral behavior, unavailable roles, and logical text size/line-height projection. `editor_design_token_contracts.rs` requires the three authored row assets to expose the shared palette, border, density, and typography inputs without local visual constants.

`zircon_runtime/src/ui/tests/render_buttons.rs` owns generic button and icon-button state extraction, tokenized text metrics, unavailable visuals, and centralized color parsing. `editor_design_token_contracts.rs` requires the authored Button, IconButton, and WorkbenchTab assets to declare the same palette, control, density, and typography inputs without component-local hex colors or fixed default tab-width geometry.

`zircon_runtime/src/ui/tests/render_text_fields.rs` owns generic input state extraction, tokenized field typography, editable/placeholder behavior, and valid-versus-invalid style overrides. `resolve/tests.rs` owns the lower shared style contract that maps `line_height_ratio` through the resolved logical font size and rejects nonpositive ratios before any Runtime Text layout request is emitted. `editor_design_token_contracts.rs` requires SearchInput, Field, NumberField, and StatusItem assets to declare their typography inputs while leaving numeric data constraints outside visual-token checks.

`zircon_runtime/src/ui/tests/render_selection_controls.rs` owns the Checkbox, Radio, and Toggle extraction contract, tokenized text metrics, central state colors, and unavailable visuals. `editor_design_token_contracts.rs` requires all three authored selection assets to declare palette, border, density, control, and typography inputs with no local color or layout constants.

`zircon_runtime/src/ui/tests/render_drag_overlay.rs` owns drag-preview, icon, text, and drop-indicator extraction and asserts the shared overlay token projection. It also keeps the closed-overlay paint-silence contract.

`zircon_runtime/src/ui/tests/render_segmented_controls.rs` owns SegmentedControl and Tab extraction: tokenized surface and typography defaults, unavailable visuals, focused-neutral versus hovered behavior, valid style overrides, invalid-value fallback, and independent selected-border compatibility. `editor_design_token_contracts.rs` requires `workbench_segmented_control.zui` to declare the corresponding palette, control, density, and typography token inputs without local colors or fixed layout geometry.

`zircon_runtime/src/ui/tests/render_sliders.rs` owns Slider and RangeSlider extraction: tokenized neutral rail/fill/thumb defaults, halo/focus/pressed behavior, unavailable visuals, tick-budget preservation, valid style overrides, and invalid-value fallback. `sliders/commands.rs` owns extracted quad/text command construction and `sliders/state_colors.rs` owns state-color selection, keeping visual resolution, state semantics, and command serialization separate; `editor_design_token_contracts.rs` requires both slider assets to import the editor token asset and expose their common palette, semantic, control, density, and typography inputs.

`zircon_runtime/src/ui/tests/render_progress.rs` owns linear Progress extraction: data-driven fill fraction, central track/fill/disabled roles, responsive label allocation through the shared text pipeline, and valid-versus-invalid visual override behavior. `editor_design_token_contracts.rs` requires the authored progress asset to import the editor token asset and declare palette, semantic, control, density, and typography inputs without local colors or fixed control geometry.

`zircon_editor/tests/integration_contracts/floating_window_design_parity.rs` checks the 06.S2 floating-window design parity contract. On 2026-06-23, `cargo test -p zircon_editor --test integration_contracts --features integration-contracts floating_window_design_parity --offline --jobs 1 --target-dir E:\cargo-targets\zircon-editor-layout-runtime-state-reducer-0623 --message-format short --color never -- --test-threads=1 --nocapture` passed 4/4 tests after `zircon_runtime` first passed a fresh lower support check in the same target directory.

## Plan Sources

This document implements `01.S1 token 资产骨架 + 契约文档`, records the first `01.S2` token feed, the density token lookup used by `02.S2`, the layout-owned asset tokenization step, and the focused `06.S2` floating-window design parity checklist.

## Open Issues Or Follow-up

`01.S2` still needs the wider hard cutover from historical shell/module component-local naked colors to token references at the retained painter boundary. Generic Chrome, feedback, Dialog, ConfirmDialog, NotificationCenter, CommandPalette, PopupRow, Dropdown, Button, TextField, selection controls, SegmentedControl/Tab, Slider/RangeSlider, Linear Progress, and DragOverlay now consume the shared projection and authored token overrides, but retained-host screenshot or pixel comparison remains pending until the window harness is stable. `06.S2` closes the static/code contract for floating-window design parity.
