---
related_code:
  - zircon_editor/assets/ui/editor/material_meta_components.ui.toml
  - zircon_editor/assets/ui/theme/editor_material.ui.toml
  - zircon_editor/assets/ui/theme/editor_base.ui.toml
  - zircon_editor/src/tests/ui/boundary/global_material_surface_assets.rs
  - zircon_editor/src/tests/ui/boundary/material_meta_component_contracts.rs
  - zircon_runtime/src/ui/template/asset/compiler/value_normalizer.rs
  - zircon_runtime/src/ui/layout/pass/material.rs
  - zircon_runtime/src/ui/layout/pass/measure.rs
  - zircon_runtime/src/ui/layout/pass/arrange.rs
  - zircon_runtime/src/ui/tests/asset_component_reference_layout.rs
  - zircon_runtime/src/ui/tests/material_layout.rs
  - zircon_runtime_interface/src/ui/surface/render/command.rs
  - zircon_runtime_interface/src/tests/render_contracts.rs
  - zircon_editor/src/ui/slint_host/host_contract/painter/visual_assets.rs
  - zircon_editor/src/ui/slint_host/host_contract/painter/template_nodes.rs
  - zircon_editor/src/ui/slint_host/host_contract/painter/theme.rs
  - zircon_editor/src/ui/slint_host/ui/pane_data_conversion/pane_component_projection/mod.rs
  - zircon_editor/src/ui/layouts/views/view_projection.rs
  - zircon_editor/src/ui/template_runtime/runtime/pane_payload_projection.rs
implementation_files:
  - zircon_editor/assets/ui/editor/material_meta_components.ui.toml
  - zircon_editor/assets/ui/theme/editor_material.ui.toml
  - zircon_editor/src/tests/ui/boundary/global_material_surface_assets.rs
  - zircon_editor/src/tests/ui/boundary/material_meta_component_contracts.rs
  - zircon_runtime/src/ui/template/asset/compiler/value_normalizer.rs
  - zircon_runtime/src/ui/tests/asset_component_reference_layout.rs
  - zircon_runtime/src/ui/layout/pass/material.rs
  - zircon_runtime/src/ui/tests/material_layout.rs
  - zircon_runtime_interface/src/ui/surface/render/command.rs
  - zircon_runtime_interface/src/tests/render_contracts.rs
  - zircon_editor/src/ui/slint_host/host_contract/painter/template_nodes.rs
  - zircon_editor/src/ui/slint_host/host_contract/painter/theme.rs
  - zircon_editor/src/ui/slint_host/ui/pane_data_conversion/pane_component_projection/mod.rs
  - zircon_editor/src/tests/host/slint_window/native_material_painter.rs
plan_sources:
  - .codex/plans/Zircon UI 与 Unreal Slate 差异审计及后续里程碑.md
  - .codex/plans/Material UI 元组件与 .ui.toml 编辑器布局 Slate 化计划.md
  - .codex/plans/Material UI + .ui.toml 全链路 UI 系统推进计划.md
  - docs/superpowers/plans/2026-05-06-material-layout-foundation.md
  - .codex/plans/Material UI 共享组件风格收束计划.md
  - dev/slint/ui-libraries/material/src/material.slint
  - dev/slint/ui-libraries/material/src/ui/styling/material_style_metrics.slint
  - dev/slint/ui-libraries/material/src/ui/styling/material_palette.slint
  - dev/slint/ui-libraries/material/src/ui/components/buttons/base_button.slint
tests:
  - rustfmt --edition 2021 --check zircon_runtime/src/ui/template/asset/compiler/value_normalizer.rs zircon_runtime/src/ui/tests/asset_component_reference_layout.rs zircon_editor/src/tests/ui/boundary/material_meta_component_contracts.rs
  - cargo test -p zircon_runtime --lib ui_document_compiler_resolves_nested_material_role_tokens_in_props_and_styles --locked --jobs 1 --target-dir D:\cargo-targets\zircon-m2-material-token-roles --message-format short --color never -- --nocapture
  - cargo test -p zircon_editor --lib material_theme_declares_m2_role_tokens_and_styles_material_classes --locked --jobs 1 --target-dir D:\cargo-targets\zircon-m2-material-token-roles --message-format short --color never -- --nocapture
  - zircon_runtime/src/ui/tests/material_layout.rs
  - rustfmt --edition 2021 zircon_runtime_interface/src/ui/surface/render/command.rs zircon_runtime_interface/src/tests/render_contracts.rs
  - cargo test -p zircon_runtime_interface --lib render_contracts --locked --jobs 1 --target-dir D:\cargo-targets\zircon-m2-interface --message-format short --color never -- --nocapture
  - cargo test -p zircon_runtime --lib asset_value_nodes_render_as_image_or_icon_not_text --locked --jobs 1 --target-dir D:\cargo-targets\zircon-m1-slot-panel --message-format short --color never -- --nocapture
  - cargo test -p zircon_editor --lib runtime_svg_icon_pixels_follow_requested_target_size --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never -- --nocapture
  - cargo test -p zircon_editor --lib template_svg_icon_pixels_follow_requested_target_size --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never -- --nocapture
  - cargo test -p zircon_editor --lib global_material_surface_assets --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never -- --nocapture
  - cargo test -p zircon_editor --lib template_icon_tint_uses_material_state_priority --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never -- --nocapture
  - cargo test -p zircon_editor --lib material_meta_component_contracts --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never
  - rustfmt --edition 2021 --check zircon_editor/src/tests/ui/boundary/material_meta_component_contracts.rs zircon_runtime/src/ui/tests/asset_component_reference_layout.rs
  - cargo test -p zircon_editor --lib material_meta_component_roots_forward_interaction_accessibility_and_capability_params --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never -- --nocapture
  - cargo test -p zircon_editor --lib material_meta_components --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never -- --nocapture
  - cargo test -p zircon_editor --lib global_material_surface_assets --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never -- --nocapture
  - cargo test -p zircon_runtime --lib ui_document_compiler_preserves_reference_instance_bindings_on_expanded_root --locked --jobs 1 --target-dir D:\cargo-targets\zircon-m1-slot-panel --message-format short --color never -- --nocapture
  - cargo test -p zircon_editor --lib component_showcase_state --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never
  - cargo test -p zircon_editor --lib component_showcase_selection --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never
  - cargo test -p zircon_editor --lib component_showcase_category --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never
  - cargo test -p zircon_runtime --lib material_layout --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never
  - cargo test -p zircon_runtime --lib event_routing --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never
  - cargo test -p zircon_runtime --lib hit_grid --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never
  - cargo check -p zircon_editor --lib --tests --locked --jobs 1 --target-dir D:\cargo-targets\zircon-material-final-check --message-format short --color never
  - cargo test -p zircon_runtime --lib material_layout --locked --jobs 1 --target-dir D:\cargo-targets\zircon-material-final --message-format short --color never -- --nocapture --test-threads=1
  - cargo test -p zircon_editor --lib component_showcase --locked --jobs 1 --target-dir D:\cargo-targets\zircon-material-final-check --message-format short --color never -- --nocapture --test-threads=1
  - cargo test -p zircon_editor --lib native_host_welcome_material --locked --jobs 1 --target-dir D:\cargo-targets\zircon-material-final-check --message-format short --color never -- --nocapture --test-threads=1
  - cargo test -p zircon_editor --lib native_host_pointer_click_routes_projected_material_showcase_button --locked --jobs 1 --target-dir D:\cargo-targets\zircon-material-final-check --message-format short --color never -- --nocapture --test-threads=1
  - cargo test -p zircon_runtime --lib event_routing --locked --jobs 1 --target-dir D:\cargo-targets\zircon-material-final-check --message-format short --color never -- --nocapture --test-threads=1
  - cargo test -p zircon_runtime --lib hit_grid --locked --jobs 1 --target-dir D:\cargo-targets\zircon-material-final-check --message-format short --color never -- --nocapture --test-threads=1
  - cargo test -p zircon_editor --lib material_meta_component_contracts --locked --jobs 1 --target-dir E:\cargo-targets\zircon-material-shared-style-pass --message-format short --color never
  - cargo test -p zircon_editor --lib native_material_painter --locked --jobs 1 --target-dir E:\cargo-targets\zircon-material-shared-style-pass --message-format short --color never
  - cargo test -p zircon_editor --lib runtime_component_projection_preserves_material_visual_metadata --locked --jobs 1 --target-dir E:\cargo-targets\zircon-material-shared-style-pass --message-format short --color never
  - docs-only audit: git diff --check docs/ui-and-layout/material-ui-token-component-audit.md docs/ui-and-layout/index.md .codex/plans/Zircon UI 与 Unreal Slate 差异审计及后续里程碑.md .codex/sessions/20260506-2211-slate-milestone-m0-baseline.md
doc_type: milestone-detail
---

# Material UI Token And Component Audit

## Purpose

本文件记录 M2.1a/M2.1b/M2.2b：先把 `dev/slint` Material 参考与当前 Zircon `.ui.toml` Material token、style class、meta component、runtime layout 支撑放在同一张表里，再把 token role convergence 和 meta component root forwarding 落到 `editor_material.ui.toml`、`material_meta_components.ui.toml`、shared `.ui.toml` compiler token resolver 与 focused tests。

当前审计保持三个边界：

- Slint Material 是编辑器视觉与组件状态参考，不复制 Slint API。
- Zircon 的行为真源仍是 `.ui.toml` meta component + shared `UiSurfaceFrame` + runtime layout/render/input DTO。
- M2.1b/M2.2b 修改 shared token resolver、Material `.ui.toml` token/style assets、focused tests 和文档；不修改 sibling-owned native host 或 runtime layout implementation。

## 2026-05-07 2D Slate Chain Acceptance

Material UI + `.ui.toml` 的 2D editor/runtime 链路已按 focused boundary 收束：runtime `material_layout` 23 / 0，editor `component_showcase` 19 / 0，Material meta export/component asset coverage 通过，Welcome Material text field keyboard input、Welcome Material button callback、projected showcase button click、Inspector Material roots、native Material painter state palette 均通过。Shared Slate core 同步通过 runtime `hit_grid` 12 / 0 和 `event_routing` 23 / 0，其中包含 same-target mouse move idle/no rebuild、keyboard/text/IME、release-inside/outside click、focus/capture、scroll fallback 和 hit-grid visibility/clip/disabled 语义。

本次验收不声明 workspace-wide clean：当前仍保留既有 unused warning 和 unrelated formatting diffs；world-space UI 只保留进入同一 hit grid 的接口约束，不作为本轮 2D editor/runtime Material 完成阻塞项。

## Slint Reference Inventory

| Reference | Covered Material signal | Zircon use |
|---|---|---|
| `material_style_metrics.slint` | `size_*`、`icon_size_*`、`padding_*`、`spacing_*`、`border_radius_*` | M2.1b 需要把当前零散 `material_*` layout tokens 映射到稳定 density/spacing/radius roles |
| `material_palette.slint` | primary/secondary/tertiary/error/surface/outline/shadow/scrim/inverse/fixed roles、state-layer opacity、disabled opacity | M2.1b 需要补足 color role 与 state-layer opacity token，不只保留 editor 暗色主题色名 |
| `components/buttons/base_button.slint` | button padding 24/10、spacing 8、min 40、icon 18、disabled opacity 38% | 当前 `MaterialButton*` 已基本持有同级 layout attributes；仍缺透明度 token 和更完整状态矩阵 |
| `material.slint` exports | AppBar、Badge、CheckBox、Chip、Dialog、Drawer、DropDownMenu、Divider、Card、Button variants、IconButton variants、ListTile/ListView、Navigation、Progress、RadioButton、SearchBar、ScrollView、Slider、SnackBar、StateLayer/Ripple、SegmentedButton、Switch、TabBar、TextField、TimePickerPopup、ToolTip、MenuItem 等 | 当前 Zircon 已覆盖基础 controls 和 editor showcase 所需组件；navigation/surface/dialog/tooltip/snackbar/chip/radio/card 等还不是 M2 第一批闭环组件 |

## Token Comparison

| Domain | Slint Material roles | Current Zircon tokens/classes | Status | M2.1b target |
|---|---|---|---|---|
| Density and size | `size_32/36/40/48/56` 等固定尺寸阶梯 | Shared style pass fixes `material_density_compact_height=32`、`material_density_default_height=40`、`material_density_prominent_height=56`; aliases map compact controls to 32, standard controls/list rows to 40, and field/prominent controls to 56 | Covered for shared component style pass | 后续可以逐步把旧别名折叠到 role token，但当前 role 已有验收 |
| Padding and spacing | `padding_4..56`、`spacing_2..52` | Shared style pass keeps tool UI padding restrained: button `12/6`、field `10/4`、list `10`、gap `6` | Covered for first controls | 保留 compact tooling values while using role names for button content padding, field content padding, list row gap, and menu item gap |
| Icon sizes | `icon_size_18/24/36/90` | `material_button_icon_size=18`、`material_icon_size=30`；M2.3 DTO/painter tests 已接 frame/DPI target size | Covered for M2.3 | 后续只需按真实组件扩展 decorative icon role |
| Radius | `border_radius_2/4/8/12/16/28` | `material_radius=5`，新增 `material_radius_small/medium/large/pill/control` | Covered for M2.1b | 可在 M8 清理 style 内旧孤立半径 |
| Color roles | `primary/on_primary/surface/surface_container_* /outline/error/shadow/scrim/inverse_*` 等 | 保留旧 editor dark token，并新增 `material_color_*` role aliases | Covered for M2.1b | 真实 scheme 切换可后续接主题工具 |
| State layers | hover/focus/press/disabled/drag opacity roles | 新增 `material_state_layer_opacity_*` 和 `material_disabled_opacity` | Covered for M2.1b | 具体 alpha 混合仍由 painter/theme resolve 逐步消费 |
| Focus ring | focus state and state layer | `material_focus_ring`、`material_focus_ring_width`、`material_focus_ring_offset` | Covered for M2.1b | 视觉 offset 细节进入 M3/M7 screenshot |
| Shadow and elevation | `shadow`、`shadow_15`、`shadow_30`、Elevation component | `material_shadow`、`material_shadow_soft`、`material_elevation_level*` | Covered for token contract | 真实 painter/renderer 阴影仍属 M4/M7 |
| Typography | `MaterialTypography` | `material_font_size` 旧别名和 `material_font_size_body/meta/title` roles | Covered for M2.1b | M6 再接真实 shaping 和 text layout |

## M2.1b Token Role Convergence

M2.1b keeps the current editor dark palette values but gives them stable Material role names. `editor_material.ui.toml` now exposes palette aliases such as `material_color_primary`, `material_color_on_primary`, `material_color_surface_container`, `material_color_outline_variant`, `material_color_shadow`, and `material_color_scrim`. It also carries state-layer opacity roles, disabled opacity, focus ring width/offset, shadow/elevation levels, radius roles, border width roles, and title/body/meta typography sizes.

`material_meta_components.ui.toml` now maps legacy component tokens such as `material_control_height`, `material_field_min_height`, `material_button_padding_x`, `material_icon_size`, `material_radius`, and `material_font_size` through density/spacing/icon/radius/typography role tokens. Existing component props can keep their stable token names while new components can target the role tokens directly.

The compiler support for this is `zircon_runtime/src/ui/template/asset/compiler/value_normalizer.rs`: `$token` and `$param.*` values are resolved recursively with a bounded depth. This lets role aliases survive through component props, layout attributes, and imported style rule declarations without duplicating literal values in every rule.

Focused evidence:

- `ui_document_compiler_resolves_nested_material_role_tokens_in_props_and_styles` proves nested role aliases resolve into component props, layout metrics, palette style values, border width, radius, and typography.
- `material_theme_declares_m2_role_tokens_and_styles_material_classes` proves the editor Material theme declares the M2 role tokens and still styles every Material meta component class.
- `material_meta_components_carry_shared_style_defaults_on_root_nodes` freezes the shared component root contract for state props, variants, radius, border width, font size, and min-height metrics on representative Button, text-button, menu-bar item, and GroupBox surfaces.
- Direct Material theme style literals for common radius, border width, typography, and backdrop scrim have been replaced with token references; actual painter/renderer shadow behavior remains an M4/M7 paint/debug concern.

## Current Meta Component Coverage

| Zircon meta component | Runtime/native root | Covered M2 group | Current strengths | Remaining M2 gap |
|---|---|---|---|---|
| `MaterialButtonBase`、`MaterialButton`、`MaterialTextButton` | `Button` | Button | 已有 padding、spacing、min width/height、icon size、interactive/focusable attrs | 需要统一 variant、disabled/hover/pressed/focus/selected/error state matrix 与 accessibility/callback tests |
| `MaterialIconButton` | `IconButton` | IconButton | 已有 square sizing、icon-only measurement、accessibility label 不参与文本测量的 runtime tests；M2.3a/b 已证明 icon 路径解析、frame/DPI target size 和 native/template SVG resize；M2.3c 已覆盖 selected/error/tone metadata 与 disabled/error/warning/active tint priority | 后续只剩 M2.2b accessibility/callback 禁用态行为测试 |
| `MaterialToggleButton`、`MaterialSwitch` | `ToggleButton` | Button/Navigation | checked/selected 和 list-like metrics 已有 | 需要 pressed/disabled/focus semantic tests 与 switch-specific visual token |
| `MaterialCheckboxRow`、`MaterialCheckBox` | `Checkbox` | List/Field | checked/selected、row metrics、interactive attrs 已有 | 需要 error/disabled/focus state matrix 和 keyboard default action |
| `MaterialOutlinedField`、`MaterialLineEdit`、`MaterialTextEdit` | `InputField` / `TextField` | Field | field padding、min height、value/placeholder measurement 已有 | 需要 text edit callback/binding/accessibility、error/focus/disabled states |
| `MaterialComboBox`、`MaterialSliderField`、`MaterialSlider`、`MaterialSpinBox` | `ComboBox` / `RangeField` / `NumberField` | Field/Menu | scalar/object option and numeric value measurement 已有 runtime tests | 需要 popup anchor、value change callback、disabled/focus matrix |
| `MaterialListItem`、`MaterialTableRow`、`MaterialStandardTableView` | `ListRow` / `TableRow` / `VirtualList` | List/Surface | list row/table row metrics and virtual-list support present | 需要 selected/hover/disabled/error semantics in component expansion tests |
| `MaterialMenuBar`、`MaterialMenuBarItem`、`MaterialMenuFrame`、`MaterialMenuItem` | `HorizontalBox` / `Button` / `ContextActionMenu` / `MenuItem` | Menu | popup anchor fields exist for menu/date/time frames; menu item root is semantic `MenuItem` | 需要 nested popup, close behavior, action callback, focus traversal, disabled owner tests |
| `MaterialTabWidget*`、`MaterialTabImpl`、`MaterialTabBar*` | `VerticalBox` / `Tab` / bar containers | Tabs/Navigation | tab semantic root, selected state, horizontal/vertical bar classes present | 需要 tab activation callback, selected/focus/hover disabled state matrix |
| `MaterialProgressIndicator`、`MaterialSpinner`、`MaterialScrollView`、`MaterialGroupBox` | `ProgressBar` / `Spinner` / `ScrollableBox` / `Group` | Surface | common native roles now accepted by runtime material layout support | 需要 surface/elevation tokens and progress/spinner state visuals |
| `MaterialDatePickerPopup`、`MaterialTimePickerPopup` | `ContextActionMenu` | Dialog/Menu | popup frame class and anchor attrs present | 需要 dialog/popup lifecycle, modal/backdrop, keyboard close and accessibility tests |

## M2.2a State Matrix Baseline

状态矩阵按 Zircon 运行时语义而不是 Slint 控件类型命名：组件根节点必须透传状态属性，style/theme 可以选择如何呈现，但测试需要先证明状态没有在 meta component 展开时丢失。

| M2 group | Zircon components | Required states | Current pass-through | Missing state/test work |
|---|---|---|---|---|
| Button | `MaterialButton`、`MaterialButtonBase`、`MaterialTextButton` | default、hovered、pressed、focused、disabled、error | `MaterialButton` 已透传 hovered/pressed/focused/disabled/validation_level/text_tone；base/text button 还偏 layout/visual attrs | 给 base/text variants 补状态参数或明确只由 concrete button 承担；测试 root props、style state、click callback 和 accessibility label |
| IconButton | `MaterialIconButton` | default、hovered、pressed、focused、disabled、selected when toggle-like | hovered/pressed/focused/disabled 已透传；icon-only label 已作为 accessibility text 不参与测量 | M2.3 证明 icon frame/DPI/tint；M2.2b 证明 accessibility label 和 disabled/focus 不触发 click |
| Toggle/Check/Switch | `MaterialToggleButton`、`MaterialCheckboxRow`、`MaterialCheckBox`、`MaterialSwitch` | default、hovered、pressed、focused、disabled、selected/checked | toggle/checkbox row 已透传 hovered/pressed/focused/disabled/checked/selected；switch/check-box 基础 interactive attrs 已有 | 统一 `checked` -> `selected` 语义，测试 keyboard/default action 与 disabled owner |
| Field | `MaterialOutlinedField`、`MaterialLineEdit`、`MaterialTextEdit`、`MaterialComboBox`、`MaterialSlider`、`MaterialSpinBox` | default、hovered、focused、disabled、error、pressed when range/combo | outlined/text edit 透传 hovered/focused/disabled/validation_level；line edit 只有 validation_level；combo 目前以 popup_open 为主 | 补 line edit/combo/range/number 的 disabled/focused/hovered；测试 value binding、commit callback、error visual role 和 focus ownership |
| List | `MaterialListItem`、`MaterialTableRow`、`MaterialStandardTableView` | default、hovered、pressed、focused、disabled、selected | list item/table row 透传 hovered/pressed/focused/disabled/selected；virtual list 有 interactive attrs | 测试 selected row 与 hover/press 优先级、disabled row 不入 action route、virtual visible-window state 不丢失 |
| Menu | `MaterialMenuFrame`、`MaterialMenuItem`、`MaterialMenuBarItem` | default、hovered、pressed、focused、disabled、selected/checked、popup_open | menu frame 透传 popup_open/popup_anchor_x/y；menu item 透传 hovered/pressed/focused/disabled/checked/selected | 测试 nested popup anchor、disabled menu item、checked menu item、close-on-select、escape/blur close |
| Dialog/Popup | `MaterialDatePickerPopup`、`MaterialTimePickerPopup`、future dialog wrapper | default、focused、disabled descendants、modal/backdrop、popup_open、error when form-like | date/time popup 已透传 popup_open 和 anchor；runtime backdrop/dialog classes exist in theme | 需要 named Dialog meta component 或 explicit popup lifecycle tests；测试 modal focus trap/backdrop close/accessibility role |
| Tabs/Navigation | `MaterialTabImpl`、`MaterialTabWidget*`、future navigation controls | default、hovered、pressed、focused、disabled、selected | tab root 是 semantic `Tab`，透传 hovered/pressed/focused/disabled/selected/checked | 测试 tab activation callback、keyboard nav、selected tab style、disabled tab 跳过 |
| Surface | `MaterialGroupBox`、`MaterialScrollView`、`MaterialProgressIndicator`、`MaterialSpinner`、future Card/Dialog surface | default、hovered when interactive、focused when focusable、disabled descendants、error/warning where relevant | group/scroll/progress/spinner 有 style/layout support，但状态语义弱 | M2.1b 先补 radius/elevation/shadow/surface role tokens；M2.2b 再测 focusability/capability/accessibility |

M2.2b 的 focused tests assert three layers for each covered family: authored params are accepted by the component catalog, expanded root metadata carries the state/binding/callback/accessibility values, and runtime dispatch/layout/render consumers read those values from the shared tree rather than from editor host-specific side tables.

## M2.2b Closure Evidence

M2.2b is now covered by `zircon_editor/src/tests/ui/boundary/material_meta_component_contracts.rs`. The split keeps the broad `global_material_surface_assets.rs` responsive/import contract small while giving Material root forwarding its own focused module.

| Contract | Test evidence | Covered controls |
|---|---|---|
| Role tokens and style rules | `material_theme_declares_m2_role_tokens_and_styles_material_classes` | every `material-*` class exported by `material_meta_components.ui.toml` |
| Stable state metadata | `material_meta_components_emit_stable_state_metadata` | Button, IconButton, ToggleButton, CheckboxRow, CheckBox, LineEdit/TextEdit/OutlinedField, ComboBox, Slider/SliderField, SpinBox, Switch, ListItem, TableRow, MenuItem, TabImpl |
| Shared root style defaults | `material_meta_components_carry_shared_style_defaults_on_root_nodes` | base Button, text Button, menu-bar item, and GroupBox roots now carry variants, state props, radius, border width, font size, and min-height metrics |
| Input and popup metadata | `material_meta_components_project_input_and_popup_contracts` | all interactive roots plus ComboBox/MenuFrame/DatePicker/TimePicker popup anchors |
| Binding/callback/capability/accessibility forwarding | `material_meta_component_roots_forward_interaction_accessibility_and_capability_params` | Button/IconButton/Field/List/Menu/Dialog-like Popup/Tabs/Navigation/Surface first-slice roots |
| Runtime expanded-root callback | `ui_document_compiler_preserves_reference_instance_bindings_on_expanded_root` | reference component expansion preserves instance binding id, click route, callback action and payload on the expanded root |
| Editor pane/template usage | `global_material_surface_assets` and `material_meta_components` focused gates | real editor/runtime `.ui.toml` asset scanning plus Material component state, input, popup and export contracts |

The 2026-05-07 shared component style pass keeps the public `TemplatePaneNodeData` shape unchanged. `pane_component_projection` already forwards `surface_variant`, `button_variant`, `text_tone`, `validation_level`, `selected`, `hovered`, `pressed`, `focused`, `disabled`, border, radius, and font metadata; `runtime_component_projection_preserves_material_visual_metadata` now asserts that full style/state set so root visual metadata cannot silently fall out of the native host contract.

Native painter state is intentionally centralized in `host_contract/painter/theme.rs` and consumed by `template_nodes.rs`. The current state priority is `disabled > validation error/warning > pressed > selected/focused > primary/accent > hovered > default`; focused painter tests cover default, hover, pressed, selected/focused border, disabled, primary/accent, warning, error, and disabled text colors. Rounded corners and true elevation are still outside the Rust-owned painter’s current quad primitive, but the token and state-color contract is stable for the shared Material controls.

Implementation follow-up in this slice added root-level forwarding params for the first-slice Button/IconButton/Field/List/Menu/Dialog-like Popup/Tabs/Navigation/Surface representatives, and keeps `MaterialComboBox` forwarding `popup_anchor_x` and `popup_anchor_y` so popup positioning does not need an editor-host side table.

## Slint Export Gap

Current Zircon first-class Material meta components cover the M2 first slice around Button, IconButton, Field, List, Menu, Tabs and basic Surface controls. The following Slint-exported families are still intentionally outside first-slice coverage or only represented by a lower-level Zircon primitive:

| Slint family | Current Zircon position | Suggested owner |
|---|---|---|
| AppBar, BottomAppBar, NavigationBar, NavigationDrawer, ModalNavigationDrawer, NavigationRail | Not first-class Material meta components; editor chrome currently comes through host/template paths | M2.2 for navigation semantics, M3 for editor host cutover |
| Dialog, FullscreenDialog, Modal, ModalBottomSheet, Snackbar, ToolTip | Popup/backdrop classes exist, but dialog lifecycle and modal focus contract are not closed | M2.2 + M5 |
| Badge, Chip, SegmentedButton, RadioButton, RadioButtonTile | Not in current first-slice meta component coverage | M2 follow-up after base controls |
| ElevatedCard, FilledCard, OutlinedCard, Elevation | Elevation/shadow tokens now exist, but no first-class card/elevation component behavior yet | M2.2 surface follow-up, M4/M7 paint/debug support |
| SearchBar, ListTile, Avatar, Divider | Can be composed from fields/list rows/basic primitives, but not frozen as named components | M2.2 or M3 according to real pane usage |

## Runtime Support Boundary

`zircon_runtime/src/ui/layout/pass/material.rs` currently treats a node as Material-layout-aware when it has authored layout attributes and a supported component role. The supported role set already includes:

- controls: `Button`, `IconButton`, `ToggleButton`, `Checkbox`, `InputField`, `TextField`, `ComboBox`, `RangeField`, `NumberField`, `Switch`
- lists and menus: `ListRow`, `MenuItem`, `ContextActionMenu`, `TableRow`, `VirtualList`, `Tab`
- status and generic roles: `ProgressBar`, `Spinner`, `Label`
- editor value roles: `ColorField`, `Vector2Field`, `Vector3Field`, `Vector4Field`

This is enough for M2.1 token unification and M2.2 focused behavior tests without adding another layout path. M2.3 now continues through the existing render/visual asset command path instead of teaching layout about icon pixels: render commands carry `UiVisualAssetRef`, paint conversion writes the frame+DPI target `pixel_size`, and native/template SVG tests prove the actual editor raster path can resize the same SVG at multiple target frames.

## Next Execution Targets

M2.1b is accepted: role tokens now exist in the theme/meta component TOML and are checked by `material_meta_component_contracts`. M2.2b is accepted for state/input/popup root forwarding plus binding, callback, focusable, capability, and accessibility params on the first-slice Material roots. M2.3a/b/c are accepted for SVG/icon path, frame/DPI target sizing, cache-size resource state, and Material state tint priority.

Next execution target is M3.1a: move menu, drawer, toolbar, document pane and floating panel entry points toward `.ui.toml + shared surface` data ownership while preserving the M2 contracts as the component-layer floor.
