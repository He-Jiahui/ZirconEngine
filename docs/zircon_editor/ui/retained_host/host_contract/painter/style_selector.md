---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/painter/style_selector/mod.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/style_selector/workbench_button.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/style_selector/workbench_dropdown.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/style_selector/workbench_icon_button.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/style_selector/workbench_list_row.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/style_selector/workbench_popup_row.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/style_selector/workbench_segmented_control.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/style_selector/workbench_selection_control.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/style_selector/workbench_slider.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/style_selector/workbench_toast.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/style_selector/workbench_tooltip.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/template_alerts.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/template_list_rows.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/template_segmented_controls.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/template_tooltips.rs
  - zircon_runtime_interface/src/ui/style.rs
implementation_files:
  - zircon_editor/src/ui/retained_host/host_contract/painter/style_selector/mod.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/style_selector/workbench_list_row.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/style_selector/workbench_segmented_control.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/style_selector/workbench_toast.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/style_selector/workbench_tooltip.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/template_alerts.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/template_list_rows.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/template_segmented_controls.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/template_tooltips.rs
plan_sources:
  - user: 2026-06-03 componentized editor UI prototype and native replication request
  - .codex/plans/ZirconEngine 宿主编辑器 UI 基础能力计划.md
  - docs/ui-and-layout/ai-workbench-style/component-prototype/README.md
tests:
  - zircon_editor/src/ui/retained_host/host_contract/painter/template_alerts.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/template_list_rows.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/template_segmented_controls.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/template_tooltips.rs
  - docs/ui-and-layout/ai-workbench-style/component-prototype/verify-native-component-contract.mjs
  - rustfmt --edition 2021 --check zircon_editor/src/ui/retained_host/host_contract/painter/style_selector/mod.rs zircon_editor/src/ui/retained_host/host_contract/painter/style_selector/workbench_list_row.rs zircon_editor/src/ui/retained_host/host_contract/painter/style_selector/workbench_segmented_control.rs zircon_editor/src/ui/retained_host/host_contract/painter/style_selector/workbench_toast.rs zircon_editor/src/ui/retained_host/host_contract/painter/style_selector/workbench_tooltip.rs zircon_editor/src/ui/retained_host/host_contract/painter/template_alerts.rs zircon_editor/src/ui/retained_host/host_contract/painter/template_list_rows.rs zircon_editor/src/ui/retained_host/host_contract/painter/template_segmented_controls.rs zircon_editor/src/ui/retained_host/host_contract/painter/template_tooltips.rs
  - node verify-native-component-contract.mjs
  - cargo test -p zircon_editor --lib template_list_rows --locked --jobs 1 --message-format short --color never
  - cargo test -p zircon_editor --lib template_segmented_controls --locked --jobs 1 --message-format short --color never
  - cargo test -p zircon_editor --lib template_alerts --locked --jobs 1 --message-format short --color never
  - cargo test -p zircon_editor --lib template_tooltips --locked --jobs 1 --message-format short --color never
doc_type: module-detail
---

# Workbench Style Selectors

The retained Workbench painter keeps visual state resolution in `style_selector/*` files and leaves the template painters responsible for recognition, geometry, and command emission. Selectors consume the shared `UiPainterState` / `UiPainterResolvedState` priority model, so hover, focus, press, selected, checked, open, disabled, dragging, drop-hover, and loading states do not drift across component families.

## Tabs And Segmented Controls

`workbench_segmented_control.rs` owns the color and state contract for `WorkbenchSegmentedControl` and `WorkbenchTab` primitives:

- segmented controls use the same `UiPainterFamily::Tab` interactive state priority as tabs, then resolve group fill, border, selected segment fill/border, underline, selected text, idle text, and group-label color;
- tabs keep declared idle background support from `.zui` styles, but hover/focus/pressed/disabled states are resolved through the selector instead of ad hoc branches in `template_segmented_controls.rs`;
- selected segment border width, underline height, and underline color remain declaration-driven so icon-toggle segmented controls can suppress the selected border while ordinary segmented controls keep the legacy one-pixel selected border;
- `template_segmented_controls.rs` now delegates state-dependent style choices to the selector while keeping layout offsets, segment splitting, tab underline geometry, text placement, and paint-command ordering local to the painter.

The focused regression `segmented_and_tab_styles_use_shared_state_priority` verifies disabled state wins over pressed/focused/hovered, pressed wins after disabled is removed, and checked tabs still draw selected text while hover controls the tab background. The browser/native component contract also checks for the selector file and the required pressed/disabled state handling so web-to-native component promotion cannot silently drop this family.

## List Rows

`workbench_list_row.rs` owns the style contract for `WorkbenchListRow` collection rows:

- disabled rows suppress their row background and border while still returning disabled text/adornment colors for the painter-owned disabled mark;
- selected or checked rows keep selected surface and focus-ring adornment semantics independent of hover state, matching the existing collection-row selection model;
- pressed, focused, dragging, and drop-hover states produce the shared focus-ring border through `UiPainterResolvedState` instead of duplicating state branches in `template_list_rows.rs`;
- declared row background, text, and icon colors still win where the template author provided them, preserving the component-drawer list samples.

`template_list_rows.rs` now keeps row recognition, label geometry, adornment geometry, and the check/chevron/disabled mark paint commands. The focused regression `list_row_style_uses_shared_state_priority` covers disabled, pressed, and selected state precedence, while `verify-native-component-contract.mjs` checks that the native ListRow selector exists and handles pressed/disabled states.

## Tooltips And Toasts

`workbench_tooltip.rs` owns Workbench tooltip visual resolution:

- tooltip surface, border, title, body, arrow, icon, and shadow colors resolve from `UiPainterFamily::Tooltip` through the shared interactive state priority;
- disabled tooltip state mutes all text/icon/arrow output and lowers shadow strength, while pressed/focused state routes border and icon color through the focus-ring semantics used by the other retained controls;
- author-declared `.zui` style colors still override surface, border, title, body, icon, and arrow colors before paint commands are emitted;
- `template_tooltips.rs` now keeps tooltip detection, bubble placement, arrow geometry, text layout, info-icon drawing, and paint-command ordering.

`workbench_toast.rs` owns Workbench toast visual resolution:

- toast surface, border, text, status mark, action text, and close mark resolve from `UiPainterFamily::Toast`, so disabled, pressed, focused, hovered, dragging, drop-hover, open, and loading state use the same priority model as the rest of the retained-host painter;
- pressed and focused toasts use focus-ring border/action styling, hovered and drag/drop states use a hotter toast surface, and disabled state suppresses declared action/mark colors in favor of disabled text colors;
- `template_alerts.rs` still owns inline alert tone detection and glyph geometry, but standalone Workbench toast visuals now come from the selector instead of local Toast constants and helper branches.

The focused regressions `workbench_tooltip_style_uses_shared_state_priority` and `workbench_toast_style_uses_shared_state_priority` cover disabled-over-pressed/focused/hovered precedence plus the pressed/focused fallthrough used by the shared selector model.
