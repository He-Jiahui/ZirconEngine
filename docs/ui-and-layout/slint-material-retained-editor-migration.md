---
related_code:
  - dev/material-rust-template/material-1.0/material.slint
  - dev/material-rust-template/material-1.0/ui/styling/material_palette.slint
  - dev/material-rust-template/material-1.0/ui/styling/material_schemes.slint
  - dev/material-rust-template/material-1.0/ui/styling/material_style_metrics.slint
  - dev/material-rust-template/material-1.0/ui/styling/material_typography.slint
  - dev/material-rust-template/material-1.0/ui/styling/material_animations.slint
  - dev/material-rust-template/material-1.0/ui/components/state_layer.slint
  - dev/material-rust-template/material-1.0/ui/components/elevation.slint
  - dev/material-rust-template/material-1.0/ui/components/base_button.slint
  - dev/material-rust-template/material-1.0/ui/components/filled_button.slint
  - dev/material-rust-template/material-1.0/ui/components/outline_button.slint
  - dev/material-rust-template/material-1.0/ui/components/text_button.slint
  - dev/material-rust-template/material-1.0/ui/components/tonal_button.slint
  - dev/material-rust-template/material-1.0/ui/components/elevated_button.slint
  - dev/material-rust-template/material-1.0/ui/components/icon_button.slint
  - dev/material-rust-template/material-1.0/ui/components/floating_action_button.slint
  - dev/material-rust-template/material-1.0/ui/components/text_field.slint
  - dev/material-rust-template/material-1.0/ui/components/check_box.slint
  - dev/material-rust-template/material-1.0/ui/components/radio_button.slint
  - dev/material-rust-template/material-1.0/ui/components/switch.slint
  - dev/material-rust-template/material-1.0/ui/components/slider.slint
  - dev/material-rust-template/material-1.0/ui/components/menu.slint
  - dev/material-rust-template/material-1.0/ui/components/dialog.slint
  - dev/material-rust-template/material-1.0/ui/components/drawer.slint
  - dev/material-rust-template/material-1.0/ui/components/navigation_bar.slint
  - dev/material-rust-template/material-1.0/ui/components/tab_bar.slint
  - zircon_editor/Cargo.toml
  - zircon_editor/assets/ui/theme/editor_material.v2.ui.toml
  - zircon_editor/src/tests/ui/boundary/slint_material_retained_editor_migration.rs
  - zircon_editor/src/tests/host/retained_window/native_material_painter.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/template_nodes.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/material_state_layer.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/mod.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/template_nodes.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/mod.rs
  - zircon_editor/src/ui/retained_host/ui/template_node_conversion.rs
  - zircon_runtime/src/ui/style.rs
  - zircon_runtime/src/ui/tests/material_button_style.rs
implementation_files:
  - docs/ui-and-layout/slint-material-retained-editor-migration.md
  - docs/ui-and-layout/index.md
  - zircon_editor/assets/ui/theme/editor_material.v2.ui.toml
  - zircon_editor/src/tests/ui/boundary/slint_material_retained_editor_migration.rs
  - zircon_editor/src/tests/ui/boundary/mod.rs
  - zircon_editor/src/tests/host/retained_window/native_material_painter.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/template_nodes.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/material_state_layer.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/mod.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/template_nodes.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/mod.rs
  - zircon_editor/src/ui/retained_host/ui/template_node_conversion.rs
  - zircon_runtime/src/ui/style.rs
  - zircon_runtime/src/ui/tests/material_button_style.rs
plan_sources:
  - user: 2026-05-20 migrate Slint Material component behavior into retained Editor UI without direct Slint runtime
  - docs/superpowers/specs/2026-05-20-slint-material-retained-editor-migration-design.md
  - docs/superpowers/plans/2026-05-20-slint-material-retained-editor-migration.md
  - .codex/plans/Material UI + .ui.toml 全链路 UI 系统推进计划.md
  - .codex/plans/ZirconEngine UI 组件化与 Material 样式器重构计划.md
  - .codex/plans/Runtime UI 组件库与 Slint Material Showcase Cutover 计划.md
tests:
  - zircon_editor/src/tests/ui/boundary/slint_material_retained_editor_migration.rs
  - zircon_editor/src/tests/host/retained_window/native_material_painter.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/material_state_layer.rs inline unit tests
  - zircon_runtime/src/ui/tests/material_button_style.rs
  - rustfmt --edition 2021 --check zircon_editor/src/tests/ui/boundary/slint_material_retained_editor_migration.rs
  - rustfmt --edition 2021 --check zircon_editor/src/tests/ui/boundary/slint_material_retained_editor_migration.rs zircon_editor/src/tests/host/retained_window/native_material_painter.rs zircon_editor/src/ui/retained_host/host_contract/data/template_nodes.rs zircon_editor/src/ui/retained_host/host_contract/painter/template_nodes.rs zircon_editor/src/ui/retained_host/host_contract/painter/material_state_layer.rs zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/mod.rs zircon_editor/src/ui/retained_host/ui/template_node_conversion.rs zircon_runtime/src/ui/style.rs zircon_runtime/src/ui/tests/material_button_style.rs
  - cargo metadata --locked --no-deps --format-version 1
  - git diff --check -- docs/ui-and-layout/slint-material-retained-editor-migration.md docs/ui-and-layout/index.md zircon_editor/assets/ui/theme/editor_material.v2.ui.toml zircon_editor/src/tests/ui/boundary/mod.rs zircon_editor/src/tests/ui/boundary/slint_material_retained_editor_migration.rs
doc_type: milestone-detail
---

# Slint Material Retained Editor Migration

## Purpose

This document is the M0/M1/M2 control record for migrating `dev/material-rust-template/material-1.0` into Zircon Editor retained UI. It records the source inventory, retained owner for each Slint Material export, the no-direct-Slint Editor fence, the foundation tokens now present in `editor_material.v2.ui.toml`, and the retained state-layer/ripple/elevation metadata contract.

The accepted behavior target is a retained reproduction of Slint Material behavior. `zircon_editor` must keep `.ui.toml` / `.zui` / Rust retained host contracts as the business UI truth and must not link Slint runtime or import `@material`.

## Boundary Rules

| Rule | Decision |
|---|---|
| Editor business UI source | `.ui.toml`, `.zui`, runtime UI component descriptors, and retained host projection. |
| Slint Material role | Reference implementation and behavior evidence only. |
| Forbidden Editor dependency | No direct `slint`, `slint-build`, `i-slint-*`, `material.slint`, or `@material` dependency/import in `zircon_editor`. |
| Hub exception | `zircon_hub` may use Slint Material directly; that path is outside this Editor retained migration. |
| Runtime/editor ownership | Shared UI contracts and behavior belong in `zircon_runtime_interface::ui` and `zircon_runtime::ui`; authoring host presentation belongs in `zircon_editor`. |
| Implementation cadence | M0/M1/M2 use focused docs/theme/test validation plus lightweight Rust gates; broad Cargo build/test belongs to later milestone testing stages. |

## Source Inventory

| Source file | Covered signal | Retained target |
|---|---|---|
| `material.slint` | Full export inventory for components, items, and styling globals. | M0 mapping and boundary test export coverage. |
| `ui/styling/material_palette.slint` | MD3 palette roles, light/dark schemes, state-layer opacities, disabled opacity, modal background, shadow alpha roles. | `editor_material.v2.ui.toml` `slint_material_*` palette and state tokens. |
| `ui/styling/material_schemes.slint` | Scheme field names for primary/secondary/tertiary/error/surface/outline/fixed/inverse roles. | Retained theme token naming and future scheme-switch support. |
| `ui/styling/material_style_metrics.slint` | Size ladder, icon sizes, padding, spacing, radius ladder. | Retained density/metric tokens and component layout defaults. |
| `ui/styling/material_typography.slint` | Text style scale and font weights. | Retained typography token scale. |
| `ui/styling/material_animations.slint` | Emphasized/standard/ripple/opacity durations and easing names. | Retained motion metadata; animation implementation remains later work. |
| `ui/components/state_layer.slint` | StateLayer, FocusTouchArea, StateLayerArea, Ripple behavior. | Shared input/focus/state metadata and retained painter ripple/state-layer rendering. |
| `ui/components/elevation.slint` | Elevation levels 1..5 and dark/light shadow offsets/blur/alpha. | Retained elevation tokens and painter/renderer shadow support. |
| `ui/components/base_button.slint` | Button layout, padding, icon/avatar support, label style, disabled opacity. | Button typed style, Material Lab prototypes, native painter. |
| `ui/components/text_field.slint` | TextInput behavior, floating label, active indicator, supporting text, error/focus states. | Text field descriptors, text edit events, retained field painter. |
| `ui/components/check_box.slint` | CheckState enum, tristate toggle, error/disabled styling, tile forwarding. | Checkbox descriptors, checked/indeterminate behavior tests, painter glyphs. |
| `ui/components/radio_button.slint` | Radio selected dot, disabled/highlighted/checked state, tile forwarding. | Radio group descriptor, selection behavior, painter glyphs. |
| `ui/components/switch.slint` | Track/thumb geometry, checked toggle, focus/hover/pressed state, disabled colors. | Switch descriptor, local checked mutation, track/thumb painter. |
| `ui/components/slider.slint` | Numeric value mapping, track/fill/thumb, stops, pointer and keyboard updates. | Slider/range descriptor, drag/key behavior, retained track/thumb painter. |
| `ui/components/menu.slint` | Menu rows, selected/disabled state, popup menu close policy, elevation. | Context menu descriptors, popup surface behavior, retained menu painter. |
| `ui/components/dialog.slint` | Modal surface, actions, default action, escape close, backdrop click close, fullscreen variant. | Dialog/modal descriptors, overlay routing, focus/default-action behavior. |
| `ui/components/drawer.slint` | Drawer and ModalDrawer, title header, slide-in animation, modal backdrop close. | Drawer retained assets, overlay surface lifecycle, shell adoption. |
| `ui/components/navigation_bar.slint` | Selected nav item, badges, icon/label layout, state layer/ripple. | Navigation bar descriptors and retained samples. |
| `ui/components/tab_bar.slint` | Selected tab, indicator animation, primary/secondary variants. | Tab descriptors, selected indicator painter, navigation tests. |

## Export Mapping

| Slint export | Source | Retained owner | Milestone | Notes |
|---|---|---|---|---|
| `AppBar`, `SmallAppBar`, `MediumAppBar`, `LargeAppBar` | `ui/components/app_bar.slint` | Composite retained asset and workbench shell chrome | M4 | Toolbar children own interaction. |
| `Badge` | `ui/components/badge.slint` | Badge overlay component/painter | M3 | Anchor placement may need painter support. |
| `BottomAppBar` | `ui/components/bottom_app_bar.slint` | Composite retained app bar surface | M4 | Editor shell adoption later. |
| `CheckState`, `CheckBox`, `CheckBoxTile` | `ui/components/check_box.slint` | Checkbox descriptor and Material Lab input prototype | M3 | Tristate behavior maps to indeterminate metadata. |
| `ActionChip`, `FilterChip`, `InputChip` | `ui/components/chip.slint` | Chip/tag components | M3 | Delete/filter actions stay typed events. |
| `DatePickerPopup`, `DatePickerAdapter` | `ui/components/date_picker.slint` | Date picker popup retained overlay | M4 | Calendar math/timezone remain explicit support gaps until implemented. |
| `Dialog`, `FullscreenDialog` | `ui/components/dialog.slint` | Dialog/modal retained overlay | M4 | Escape/default action/backdrop close via shared routing. |
| `Drawer`, `ModalDrawer` | `ui/components/drawer.slint` | Drawer retained surface | M4 | Slide motion metadata first; shell adoption in M5. |
| `DropDownMenu` | `ui/components/drop_down_menu.slint` | Select/dropdown popup component | M3 | Reuse context menu popup routing. |
| `VerticalDivider`, `HorizontalDivider` | `ui/components/divider.slint` | Divider primitive | M3 | Static visual/layout primitive. |
| `ModalBottomSheet` | `ui/components/bottom_sheet.slint` | Bottom-sheet overlay | M4 | Modal owner and backdrop rules shared with Dialog. |
| `ElevatedCard`, `FilledCard`, `OutlinedCard` | `ui/components/card.slint` | Card/Paper retained surfaces | M4 | Elevation depends on M2 painter support. |
| `ElevatedButton`, `FilledButton`, `OutlineButton`, `TextButton`, `TonalButton` | button variant sources | Button typed style and painter | M3 | All variants reuse a common retained button style contract. |
| `Elevation` | `ui/components/elevation.slint` | Elevation token/painter contract | M1/M2 | M1 tokens, M2 rendering behavior. |
| `ExtendedTouchArea` | `ui/components/extended_touch_area.slint` | Shared hit/input support | M2 | No fake Editor-only event branch. |
| `FilledIconButton`, `IconButton`, `OutlineIconButton`, `TonalIconButton` | icon button sources | IconButton typed style | M3 | Icon-only size/tint/focus state. |
| `FloatingActionButton`, `FABStyle` | `ui/components/floating_action_button.slint` | FAB retained Button/IconButton composite | M3 | Uses elevation tokens and Button behavior. |
| `Grid`, `Horizontal`, `Vertical` | layout helper sources | Retained layout primitives | M4 | Route to runtime layout support, not painter hacks. |
| `Icon` | `ui/components/icon.slint` | Icon primitive/painter | M3 | Source/image/tint/size mapping. |
| `Avatar`, `ListTile` | `ui/components/list.slint` | Avatar/List row descriptors | M3 | ListTile click forwards through state layer. |
| `ListView` | `ui/components/list_view.slint` | List/virtual-list retained component | M4 | Virtualization remains runtime-owned. |
| `MaterialText` | `ui/components/material_text.slint` | Typography/text primitive | M1/M3 | M1 typography tokens, M3 retained text style use. |
| `MaterialWindow`, `MaterialWindowAdapter` | `ui/components/material_window.slint` | Host adapter policy only | M5 | Do not import Slint window into Editor. |
| `PopupMenu` | `ui/components/menu.slint` | Context popup menu | M4 | Close policy and elevation via retained overlay. |
| `NavigationBar`, `NavigationDrawer`, `ModalNavigationDrawer`, `NavigationRail` | navigation sources | Navigation retained components | M4 | Selected item state and focus traversal. |
| `CircularProgressIndicator`, `LinearProgressIndicator` | `ui/components/progress_indicator.slint` | Progress components | M3 | Spinner/arc support may require painter work. |
| `RadioButton`, `RadioButtonTile` | `ui/components/radio_button.slint` | Radio group descriptors | M3 | Selection remains group-owned. |
| `SearchBar` | `ui/components/search_bar.slint` | Search field + popup list composite | M3 | Query edit and popup owner are typed events. |
| `ScrollView` | `ui/components/scroll_view.slint` | ScrollableBox/viewport component | M4 | Runtime layout/scroll owner. |
| `Slider` | `ui/components/slider.slint` | Slider/range descriptor | M3 | Pointer drag and arrow keys. |
| `SnackBar` | `ui/components/snack_bar.slint` | Toast/status overlay | M4 | Timeout/motion can be deferred with metadata. |
| `StateLayerArea`, `StateLayer`, `Ripple` | `ui/components/state_layer.slint` | Shared state metadata and painter behavior | M2 | M1 carries source-derived opacities and durations. |
| `SegmentedButton` | `ui/components/segmented_button.slint` | ToggleButtonGroup/ButtonGroup retained component | M3 | Group exclusivity and attached radius. |
| `Switch` | `ui/components/switch.slint` | Switch descriptor and painter | M3 | Track/thumb geometry and disabled no-toggle. |
| `TabBar`, `SecondaryTabBar` | `ui/components/tab_bar.slint` | Tab descriptors and indicator painter | M4 | Indicator animation metadata. |
| `TextField` | `ui/components/text_field.slint` | TextField/Input retained component | M3 | Floating label and supporting text. |
| `TimePickerPopup`, `Time` | `ui/components/time_picker.slint` | Time picker popup retained overlay | M4 | Parsing/time model support gap until shared date/time exists. |
| `ToolTip` | `ui/components/tooltip.slint` | Tooltip popup | M4 | Hover/focus open and placement behavior. |
| `Modal` | `ui/components/modal.slint` | Modal overlay behavior | M4 | Backdrop/focus/escape rules are runtime-owned. |
| `ListItem`, `NavigationItem`, `NavigationGroup`, `MenuItem` | `ui/items/*.slint` | DTO-like retained row/item data | M3/M4 | Translate as typed props, not Rust object sharing. |
| `MaterialAnimations`, `MaterialScheme`, `MaterialSchemes`, `MaterialStyleMetrics`, `MaterialPalette`, `MaterialTypography` | `ui/styling/*.slint` | Theme/tokens and motion metadata | M1 | These are the foundation token authority for retained migration. |

## M1 Foundation Token Landing

`editor_material.v2.ui.toml` keeps existing compact Editor tokens and now also carries source-aligned `slint_material_*` tokens. These names are intentionally separate from older `material_*` tokens so later milestones can migrate selectors gradually without breaking current Editor density.

The token groups are:

- palette and scheme aliases: `slint_material_primary`, `slint_material_on_primary`, `slint_material_primary_container`, `slint_material_surface`, `slint_material_surface_container_highest`, `slint_material_outline`, `slint_material_shadow`, `slint_material_scrim`, inverse/fixed roles, and modal background;
- state roles: `slint_material_state_layer_opacity_hover = 0.08`, focus/press `0.10`, disabled `0.12`, drag `0.16`, and disabled opacity `0.38`;
- metric ladder: `slint_material_size_*`, `slint_material_icon_size_*`, `slint_material_padding_*`, `slint_material_spacing_*`, and `slint_material_border_radius_*` from `material_style_metrics.slint`;
- typography scale: display/headline/title/label/body size and weight tokens from `material_typography.slint`;
- animation metadata: emphasized, standard, ripple, and opacity easing/duration tokens from `material_animations.slint`;
- elevation metadata: level 1 through 5 light/dark outer/inner shadow offsets and blur values plus `slint_material_shadow_15` / `slint_material_shadow_30`.

Representative exact source values now guarded by `slint_material_retained_editor_migration.rs` include:

| Source area | Token | Value |
|---|---|---|
| Dark palette | `slint_material_primary` | `#adc6ff` |
| Dark palette | `slint_material_surface` | `#111318` |
| Dark palette | `slint_material_surface_container_highest` | `#33353a` |
| Modal/background | `slint_material_background_modal` | `#00000080` |
| State layer | `slint_material_state_layer_opacity_hover` | `0.08` |
| State layer | `slint_material_state_layer_opacity_press` | `0.10` |
| Metrics | `slint_material_size_90` | `96.0` |
| Metrics | `slint_material_icon_size_90` | `27.0` |
| Typography | `slint_material_typography_display_large_size` | `57.0` |
| Typography | `slint_material_typography_title_medium_size` | `16.0` |
| Motion | `slint_material_emphasized_duration_ms` | `500.0` |
| Motion | `slint_material_ripple_duration_ms` | `2000.0` |
| Elevation | `slint_material_elevation_level2_light_inner_blur` | `6.0` |
| Elevation | `slint_material_elevation_level1_dark_outer_blur` | `3.0` |

The retained Editor may continue using compact tokens such as `material_density_compact_height = 32.0` and `material_icon_size_button = 16.0`. Those are Editor UX decisions; the new source-aligned tokens are the evidence-backed foundation for later component parity.

## M2 State Layer, Ripple, And Elevation Behavior

M2 translates the `state_layer.slint` / `elevation.slint` behavior into retained metadata and native painter commands without importing Slint into `zircon_editor`.

Source facts preserved from `state_layer.slint`:

| Source behavior | Retained contract |
|---|---|
| `root.state_layer_opacity: MaterialPalette.state_layer_opacity_focus` for disabled display background and focus. | Disabled and focused retained state layers use focus opacity `0.10`; disabled remains the highest priority. |
| `root.state_layer_opacity: MaterialPalette.state_layer_opacity_press` for pressed. | Pressed and `enter_pressed` use press opacity `0.10`. |
| Hover is gated by `!MaterialWindowAdapter.disable_hover`. | `slint_material_hover_disable_token = "disable_hover"` records the source gate; retained hover activation is still runtime/host input policy. |
| `root.pressed || root.enter_pressed` activates both state layer and ripple. | `TemplatePaneNodeData.enter_pressed` carries keyboard activation metadata without moving key routing into the editor painter. |
| `pressed_x: root.pressed_x` and `pressed_y: root.pressed_y`. | `ripple_pressed_x` and `ripple_pressed_y` are projected from either explicit `ripple_pressed_*` attrs or the source-compatible `pressed_x` / `pressed_y` attrs. |
| `clip_ripple: root.clip_ripple`. | Projection keeps external `clip_ripple` semantics and stores `ripple_unclipped = !clip_ripple` for the painter. |
| Ripple width initializes to `root.width * 2 * 1.4142`, opacity is `MaterialPalette.state_layer_opacity_press`, and animation uses `MaterialAnimations.ripple_duration` / `MaterialAnimations.ripple_easing`. | The retained native painter draws a width-derived static press pulse circle using the same press opacity and source-derived duration/easing tokens; animated ripple expansion remains a later motion-layer gap. |

The retained priority is encoded in `slint_material_state_layer_priority = "disabled>focus>pressed>drag>hover>default"`. Production code applies that rule in two places:

| Layer | Implementation |
|---|---|
| Runtime style fallback | `zircon_runtime/src/ui/style.rs` resolves bool-only interaction state as disabled, focused, pressed, hovered after explicit `button_interaction_state` / `interaction_state` strings. |
| Editor host painter | `zircon_editor/src/ui/retained_host/host_contract/painter/material_state_layer.rs` resolves disabled, focused/selected/checked, pressed/`enter_pressed`, dragging, hover/drop/active-drag-target, default. |

M2 metadata fields on `TemplatePaneNodeData` are:

| Field | Source / use |
|---|---|
| `enter_pressed` | Source `FocusTouchArea.enter_pressed`; runtime input owns when this is true. |
| `state_layer_enabled` | Retained opt-in for drawing state-layer overlays. |
| `state_layer_color` | Source `StateLayerArea.color` / `Ripple.color`; falls back to the host focus-ring color. |
| `ripple_enabled` | Retained opt-in for static press ripple commands. |
| `ripple_pressed_x`, `ripple_pressed_y` | Source-compatible press origin metadata. |
| `ripple_unclipped` | Internal inverse of source `clip_ripple`. |

`ripple_enabled` does not imply `state_layer_enabled`: a control can request only the static ripple command without also drawing the full-rect state-layer overlay. Press coordinates are literal source coordinates, so `0.0` is a valid top/left edge origin and only non-finite values fall back to the center.

`elevation.slint` remains the source authority for level 1 through 5 shadow token values. The boundary tokens include the exact light/dark offsets and blurs, including `drop_shadow_offset_y: 8px` at level 5. The retained painter consumes an `elevation` numeric metadata field and emits a host shadow command through `template_nodes.rs`; the current painter keeps this as a static retained shadow, while exact two-layer light/dark Slint shadow parity remains part of later card/surface visual refinement.

M2 metadata tokens now guarded in `editor_material.v2.ui.toml` are:

| Token | Value |
|---|---|
| `slint_material_state_layer_priority` | `disabled>focus>pressed>drag>hover>default` |
| `slint_material_state_layer_disabled_uses_focus_opacity` | `true` |
| `slint_material_hover_disable_token` | `disable_hover` |
| `slint_material_ripple_pressed_x_attr` | `pressed_x` |
| `slint_material_ripple_pressed_y_attr` | `pressed_y` |
| `slint_material_ripple_clip_attr` | `clip_ripple` |
| `slint_material_ripple_static_pulse_contract` | `pressed_or_enter_pressed_static_circle` |
| `slint_material_elevation_retained_painter_contract` | `retained_template_elevation_shadow` |

M2 focused coverage is split so the existing Material meta component boundary test does not keep growing:

| Test file | Coverage |
|---|---|
| `zircon_editor/src/tests/ui/boundary/slint_material_retained_editor_migration.rs` | Source-symbol documentation guard, M2 metadata tokens, no direct Slint dependency fence. |
| `zircon_editor/src/tests/host/retained_window/native_material_painter.rs` | Retained painter priority, state-layer overlay pixels, static ripple clipping/origin, elevation shadow. |
| `zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/mod.rs` | Projection of state-layer/ripple/elevation metadata into `TemplatePaneNodeData`. |
| `zircon_runtime/src/ui/tests/material_button_style.rs` | Runtime bool fallback priority for disabled/focused/pressed/hovered. |

## M3 Button, Icon Button, And FAB Source Facts

M3 extends the retained control contract from the shared state-layer/elevation metadata into source-derived button, icon button, and floating action button defaults. The source files are `dev/material-rust-template/material-1.0/ui/components/base_button.slint`, `dev/material-rust-template/material-1.0/ui/components/filled_button.slint`, `dev/material-rust-template/material-1.0/ui/components/outline_button.slint`, `dev/material-rust-template/material-1.0/ui/components/text_button.slint`, `dev/material-rust-template/material-1.0/ui/components/tonal_button.slint`, `dev/material-rust-template/material-1.0/ui/components/elevated_button.slint`, `dev/material-rust-template/material-1.0/ui/components/icon_button.slint`, and `dev/material-rust-template/material-1.0/ui/components/floating_action_button.slint`.

Source behavior strings preserved for static review:

| Source behavior | Retained token or metadata |
|---|---|
| `button_horizontal_padding: MaterialStyleMetrics.padding_24` | `slint_material_base_button_horizontal_padding = 24.0` |
| `button_vertical_padding: MaterialStyleMetrics.padding_10` | `slint_material_base_button_vertical_padding = 10.0` |
| `min_layout_width: MaterialStyleMetrics.size_40` | `slint_material_base_button_min_layout_width = 40.0` |
| `icon_size: MaterialStyleMetrics.icon_size_18` | `slint_material_base_button_icon_size = 18.0` |
| `hover when base.has_hover && !base.pressed && !base.enter_pressed` | `slint_material_button_hover_elevations = "filled=1,tonal=2,elevated=3"` |
| `root.inline ? MaterialStyleMetrics.icon_size_18 : MaterialStyleMetrics.icon_size_24` | inline/default icon button size tokens |
| `clip_ripple: !root.inline` | `slint_material_icon_button_clip_ripple_contract = "clip_unless_inline"` |
| `export enum FABStyle` | `slint_material_fab_styles = "small,standard,large"` |
| `level: 3` | `slint_material_fab_elevation_level = 3.0` |
| `root.style == FABStyle.small ? MaterialStyleMetrics.border_radius_12` | `slint_material_fab_small_radius = 12.0` |

The retained token landing includes:

| Token | Value |
|---|---|
| `slint_material_button_variants` | `filled,outlined,text,tonal,elevated` |
| `slint_material_button_radius_policy` | `height_half` |
| `slint_material_button_hover_elevations` | `filled=1,tonal=2,elevated=3` |
| `slint_material_icon_button_display_background` | `false` |
| `slint_material_icon_button_clip_ripple_contract` | `clip_unless_inline` |
| `slint_material_fab_styles` | `small,standard,large` |
| `slint_material_base_button_horizontal_padding` | `24.0` |
| `slint_material_base_button_vertical_padding` | `10.0` |
| `slint_material_base_button_spacing` | `8.0` |
| `slint_material_base_button_min_layout_width` | `40.0` |
| `slint_material_base_button_min_layout_height` | `40.0` |
| `slint_material_base_button_icon_size` | `18.0` |
| `slint_material_icon_button_inline_icon_size` | `18.0` |
| `slint_material_icon_button_default_icon_size` | `24.0` |
| `slint_material_icon_button_inline_min_layout` | `18.0` |
| `slint_material_icon_button_default_min_layout` | `40.0` |
| `slint_material_fab_small_min_layout` | `40.0` |
| `slint_material_fab_standard_min_layout` | `56.0` |
| `slint_material_fab_large_min_layout` | `96.0` |
| `slint_material_fab_small_radius` | `12.0` |
| `slint_material_fab_standard_radius` | `16.0` |
| `slint_material_fab_large_radius` | `28.0` |
| `slint_material_fab_elevation_level` | `3.0` |
| `slint_material_fab_hover_elevation_level` | `4.0` |

## Validation Policy

The focused M0/M1/M2 boundary test checks these facts:

- every export in `material.slint` appears in this mapping doc;
- this document and the spec/plan name the no-direct-Slint Editor fence;
- `zircon_editor/Cargo.toml` does not declare Slint runtime/build dependencies;
- `editor_material.v2.ui.toml` includes source-derived Slint Material foundation tokens with exact values for representative palette, metric, typography, state, animation, and elevation roles.
- M2 state-layer/ripple/elevation metadata tokens are present and documented;
- source behavior strings from `state_layer.slint` and `elevation.slint` remain visible for static review.

Full `zircon_editor` and workspace Cargo tests are intentionally deferred to later milestone testing stages because the current workspace has unrelated active dirty lanes and known broad compile blockers.
