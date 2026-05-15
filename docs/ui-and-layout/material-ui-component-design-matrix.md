---
related_code:
  - dev/material-ui/docs/data/material/components
  - dev/material-ui/packages/mui-material/src
  - dev/material-ui/packages/mui-lab/src
  - zircon_editor/assets/ui/theme/editor_material.v2.ui.toml
  - zircon_editor/assets/ui/editor/component_showcase.v2.ui.toml
  - zircon_editor/assets/ui/editor/components/showcase_visual_section.zui
  - zircon_editor/assets/ui/editor/components/showcase_input_section.zui
  - zircon_editor/assets/ui/editor/components/showcase_selection_section.zui
  - zircon_editor/assets/ui/editor/components/showcase_collections_section.zui
  - zircon_editor/src/tests/ui/boundary/material_ui_component_design_matrix.rs
implementation_files:
  - docs/ui-and-layout/material-ui-component-design-matrix.md
  - zircon_editor/src/tests/ui/boundary/material_ui_component_design_matrix.rs
plan_sources:
  - .codex/plans/Material UI 全组件样式设计与验证计划.md
  - user: 2026-05-16 implement Material UI full-component style design and validation matrix
  - https://mui.com/material-ui/all-components/
tests:
  - cargo test -p zircon_editor --lib material_ui_component_design_matrix --locked --jobs 1
  - docs-only validation: git diff --check docs/ui-and-layout/material-ui-component-design-matrix.md zircon_editor/src/tests/ui/boundary/material_ui_component_design_matrix.rs zircon_editor/src/tests/ui/boundary/mod.rs
doc_type: milestone-detail
---

# Material UI Component Design Matrix

## Purpose

本文件是第一轮 Material UI 全组件样式设计表。它以本地 `@mui/material` v9.0.1 源码和 MUI `docs/data/material/components` 公开目录为来源，把 MUI 组件族翻译成 Zircon Editor retained UI 能表达的 dark Material 设计约定。

当前目标是冻结设计和验证口径，不直接生成网页图册、不改 Editor Showcase 布局、不复制 React API。后续实现批次必须从这张表选择组件族，补齐主题 token、painter/command 表达、showcase 示例和视觉 + 交互验证。

## Shared Zircon Material Defaults

| Area | Zircon design default |
|---|---|
| Palette | Editor dark Material：surface `#202830`、inset `#12181e`、hover `#2f4650`、pressed `#103c4a`、selected `#0f6574`、accent `#35c7d0`、focus ring `#80eaff` |
| Shape | default control radius `10px`，surface/paper/dialog/card radius `12px`，large surfaces `18px`，pill controls `999px` |
| State priority | `disabled > error/danger > warning > success/info > pressed > selected/checked/focused/open > primary/accent hover > hover/drop-target > default` |
| Interaction feedback | hover = brighter state layer + accent border；pressed = deeper teal container + focus ring；focus-visible = 2px focus ring；selected/checked/open = selected container |
| Interface reuse | Prefer existing fields: `button_variant`、`surface_variant`、`text_tone`、`validation_level`、`hovered`、`pressed`、`focused`、`selected`、`checked`、`disabled`、`popup_open`、`corner_radius`、`border_width` |
| Unsupported capability | Mark `needs support` in this matrix first; do not add new `.ui.toml` schema in the design-only phase |

## Component Matrix

| MUI key | Source reference | Zircon shape / role | Variants and states to design | Feedback and validation |
|---|---|---|---|---|
| `about-the-lab` | `dev/material-ui/docs/data/material/components/about-the-lab`; `dev/material-ui/packages/mui-lab/src` | Documentation-only source family for Lab components | `lab` status, migration notes, component availability | behavior/utility validation: matrix row only; no drawn control |
| `accordion` | `dev/material-ui/packages/mui-material/src/Accordion*` | `Group` / `Foldout` surface with header, summary, details | expanded/collapsed, disabled, focused, hover, pressed, elevation | visual + interaction: toggle click, focus ring, expanded body visibility, divider/radius pixels |
| `alert` | `dev/material-ui/packages/mui-material/src/Alert*` | status surface / `HelpRow` / banner | severity info/success/warning/error, filled/outlined/standard, icon, close action | visual + interaction: severity tone pixels, close click route, disabled action guard |
| `app-bar` | `dev/material-ui/packages/mui-material/src/AppBar`; `Toolbar` | top toolbar surface / workbench chrome strip | position static/fixed/sticky as layout metadata, color default/primary/transparent, elevation | visual contract: surface/elevation/border; interaction belongs to child buttons/menu |
| `autocomplete` | `dev/material-ui/packages/mui-material/src/Autocomplete`; `useAutocomplete`; lab `useAutocomplete` | `SearchSelect` + `TextField` + anchored menu | open/closed, query, option hover, selected, disabled options, multiple chips | visual + interaction: text input, open popup, option select, chip remove, keyboard focus |
| `avatars` | `dev/material-ui/packages/mui-material/src/Avatar*` | image/icon circle surface; stacked group | image, initials, icon, grouped/overlap, fallback | visual contract: circle clipping, text/icon centering, disabled/muted tone if interactive |
| `backdrop` | `dev/material-ui/packages/mui-material/src/Backdrop` | full-surface scrim layer | open/closed, invisible, modal ownership | visual + interaction: scrim color/alpha, backdrop click route, disabled through modal child |
| `badges` | `dev/material-ui/packages/mui-material/src/Badge` | anchored badge overlay on icon/button/list row | dot/standard, max count, invisible, color tones | visual contract: pill/dot placement, tone pixels, overflow count text |
| `bottom-navigation` | `dev/material-ui/packages/mui-material/src/BottomNavigation*` | compact tab/segmented nav strip | selected item, icon+label, disabled, horizontal density | visual + interaction: selected nav feedback, click route, focus traversal |
| `box` | `dev/material-ui/packages/mui-material/src/Box`; `mui-system` Box | generic layout/surface primitive | none; style carrier only | behavior/utility validation: no standalone rendering beyond child layout and optional surface |
| `breadcrumbs` | `dev/material-ui/packages/mui-material/src/Breadcrumbs` | horizontal link row with separator icons | collapsed, max items, active/current item, disabled link | visual + interaction: link hover/click, collapsed menu open, separator spacing |
| `button-group` | `dev/material-ui/packages/mui-material/src/ButtonGroup` | segmented `HorizontalBox` of `Button`/`ToggleButton` | contained/outlined/text, first/middle/last radius, vertical/horizontal, disabled group | visual + interaction: shared borders, neighbor radius, child click/pressed/focus |
| `buttons` | `dev/material-ui/packages/mui-material/src/Button`; `ButtonBase`; `IconButton`; `LoadingButton` in lab | `Button`, `IconButton`, text button, loading button | contained/primary, secondary, outlined, text, underline, size, icon, loading, disabled, hover, pressed, focus-visible | visual + interaction: click route, hover/press/focus pixels, loading spinner, disabled no-click |
| `cards` | `dev/material-ui/packages/mui-material/src/Card*` | rounded `Paper` / content surface | raised/outlined, action area, media/header/actions, hover if clickable | visual + interaction: card radius/elevation/border, action-area press, media crop |
| `checkboxes` | `dev/material-ui/packages/mui-material/src/Checkbox`; `FormControlLabel`; `FormGroup` | `Checkbox` / checkbox row | checked, unchecked, indeterminate, disabled, error, hover, focus | visual + interaction: toggle route, check glyph/state color, label click, disabled no-toggle |
| `chips` | `dev/material-ui/packages/mui-material/src/Chip` | pill `Button` / tag row | filled/outlined, color tones, icon/avatar, clickable, deletable, disabled, selected | visual + interaction: pill radius, delete click, hover/pressed/selected pixels |
| `click-away-listener` | `dev/material-ui/packages/mui-material/src/ClickAwayListener` | host behavior utility for popups/menus/dialogs | mouse/touch outside, ignored inner targets | behavior/utility validation: close-on-outside route for popup owners; no drawn control |
| `container` | `dev/material-ui/packages/mui-material/src/Container` | responsive content bounds / layout frame | fixed/fluid max width, gutters | behavior/utility validation: measure/arrange constraints, no direct visual unless surface class applied |
| `css-baseline` | `dev/material-ui/packages/mui-material/src/CssBaseline`; `ScopedCssBaseline`; `GlobalStyles` | theme reset and baseline tokens | global/scoped, scrollbar/background/text defaults | behavior/utility validation: theme tokens exist; no drawn component |
| `dialogs` | `dev/material-ui/packages/mui-material/src/Dialog*` | modal `Paper` surface + `Backdrop` | open/closed, full-screen/full-width, title/content/actions, scroll body, error/status actions | visual + interaction: backdrop click, close action, focus route, paper radius/elevation |
| `dividers` | `dev/material-ui/packages/mui-material/src/Divider` | `Separator` / one-pixel surface | horizontal/vertical, inset/middle, text divider | visual contract: line thickness, tone, orientation, label spacing |
| `drawers` | `dev/material-ui/packages/mui-material/src/Drawer`; `SwipeableDrawer` | side/bottom overlay or docked panel | temporary/persistent/permanent, anchor left/right/top/bottom, open, backdrop, elevation | visual + interaction: open/close route, backdrop click, anchor frame, focus containment |
| `floating-action-button` | `dev/material-ui/packages/mui-material/src/Fab` | elevated pill/circle `IconButton` | circular/extended, size, color, disabled, hover/pressed/focus | visual + interaction: circular radius, elevation, click route, icon+label layout |
| `grid` | `dev/material-ui/packages/mui-material/src/Grid` | grid layout primitive | columns, spacing, breakpoints, row/column span | behavior/utility validation: layout constraints and wrapping; no standalone visual |
| `icons` | `dev/material-ui/packages/mui-material/src/Icon`; `SvgIcon` | `Icon` / `SvgIcon` visual asset | font/icon/SVG, color, size, disabled/error/warning/success/info tones | visual contract: raster target size, tint priority, alpha clipping |
| `image-list` | `dev/material-ui/packages/mui-material/src/ImageList*` | image grid / virtual tile collection | woven/standard/masonry-like, tile bar, action icon, loading placeholder | visual + interaction: tile crop, selection hover, action click, scroll bounds |
| `init-color-scheme-script` | `dev/material-ui/packages/mui-material/src/InitColorSchemeScript` | startup theme utility | dark/light/system preference bootstrap | behavior/utility validation: not used as drawn control; theme mode remains host-owned |
| `links` | `dev/material-ui/packages/mui-material/src/Link` | `TextButton` / label with route | underline none/hover/always, color, disabled/current | visual + interaction: hover underline/state layer, click route, focus ring |
| `lists` | `dev/material-ui/packages/mui-material/src/List*` | `ListRow`, `VirtualList`, tree/list item rows | dense, selected, disabled, icon/avatar/secondary action, subheader | visual + interaction: row hover/press/select/focus, disabled no-click, scroll/virtual range |
| `masonry` | `dev/material-ui/packages/mui-lab/src/Masonry` | staggered grid layout | column count, gap, ordered placement | needs support: layout algorithm; visual validation after layout support exists |
| `material-icons` | `dev/material-ui/packages/mui-icons-material`; `dev/material-ui/packages/mui-material/src/SvgIcon` | icon catalog backed by Zircon asset refs | named icon, size, tone, missing asset fallback | visual contract: asset lookup, tint, missing placeholder; no interaction by itself |
| `menubar` | docs composite over `MenuList`, `MenuItem`, `Popover` | workbench menu bar + menu popup | active menu, hover, pressed, checked menu items, shortcuts | visual + interaction: open menu, select item, close on click-away, focus path |
| `menus` | `dev/material-ui/packages/mui-material/src/Menu`; `MenuItem`; `MenuList` | `ContextActionMenu` / popup frame / menu item | open/closed, anchor, selected, checked, disabled, divider, submenu-ready | visual + interaction: item hover/pressed/click, checked glyph, close route, anchor frame |
| `modal` | `dev/material-ui/packages/mui-material/src/Modal` | behavior shell for dialog/drawer/popover | open/closed, keep mounted, backdrop, focus trap, escape close | behavior/utility validation: modal focus and backdrop routes; visual belongs to child/backdrop |
| `no-ssr` | `dev/material-ui/packages/mui-material/src/NoSsr` | not applicable to native retained UI | deferred client render concept only | behavior/utility validation: matrix row only; no Zircon control |
| `number-field` | docs component; Zircon has `NumberField`/`SpinBox` | `NumberField` with stepper/drag edit | value, min/max, step, large step, disabled, error, drag active | visual + interaction: increment/decrement, drag delta, commit, error/focus pixels |
| `pagination` | `dev/material-ui/packages/mui-material/src/Pagination*`; `usePagination` | button group of page items | page, count, boundary/sibling, first/last/prev/next, disabled, selected | visual + interaction: page click route, selected page state, disabled edge buttons |
| `paper` | `dev/material-ui/packages/mui-material/src/Paper` | base `Panel` / `Paper` surface | elevation/outlined, square/rounded, surface variants | visual contract: radius, border/elevation token, dark overlay/elevated surface |
| `popover` | `dev/material-ui/packages/mui-material/src/Popover` | anchored elevated surface | open, anchor origin, transform origin, elevation, backdrop optional | visual + interaction: anchor frame, outside close, focus path, elevated radius |
| `popper` | `dev/material-ui/packages/mui-material/src/Popper` | placement engine for tooltip/select/menu | placement top/right/bottom/left, arrow, flip/prevent overflow | needs support: placement policy; validate anchor/placement once implemented |
| `portal` | `dev/material-ui/packages/mui-material/src/Portal` | host overlay layer routing | target layer, z-order, owner surface | behavior/utility validation: popup/dialog/drawer layer ordering; no drawn control |
| `progress` | `dev/material-ui/packages/mui-material/src/LinearProgress`; `CircularProgress` | `ProgressBar` / `Spinner` | determinate/indeterminate, buffer, color, disabled/muted | visual contract: track/fill, spinner arc or fallback glyph, value pixels |
| `radio-buttons` | `dev/material-ui/packages/mui-material/src/Radio`; `RadioGroup` | single-select `ToggleButton` / radio row | checked/unchecked, group value, disabled, error, hover/focus | visual + interaction: single selection route, group exclusivity, focus/disabled pixels |
| `rating` | `dev/material-ui/packages/mui-material/src/Rating` | icon row with hover preview | precision, empty/filled, read-only, disabled, hover value | visual + interaction: icon fill/tint, hover preview, select value, read-only no-click |
| `selects` | `dev/material-ui/packages/mui-material/src/Select`; `NativeSelect`; `SelectInput` | `ComboBox` / `Dropdown` + popup menu | standard/filled/outlined, multiple, display empty, open, disabled, error | visual + interaction: open/close, option hover/select, selected chips, focus/error pixels |
| `skeleton` | `dev/material-ui/packages/mui-material/src/Skeleton` | placeholder surface/line/avatar | text/rect/circular/rounded, pulse/wave static fallback | visual contract: shape/radius/tone; animation deferred until motion support |
| `slider` | `dev/material-ui/packages/mui-material/src/Slider`; `useSlider` | `Slider` / `RangeField` | value/range, marks, value label, track normal/inverted/false, disabled, drag/focus | visual + interaction: thumb drag, track fill, marks/value label, focus/disabled pixels |
| `snackbars` | `dev/material-ui/packages/mui-material/src/Snackbar*` | transient status/toast surface | open/closed, anchor origin, action, severity via alert child | visual + interaction: action click, close click, timeout state recorded; motion deferred |
| `speed-dial` | `dev/material-ui/packages/mui-material/src/SpeedDial*` | FAB-triggered action menu | open/closed, direction, action tooltip, hidden/disabled | visual + interaction: FAB click opens actions, action click, tooltip/open state |
| `stack` | `dev/material-ui/packages/mui-material/src/Stack` | vertical/horizontal layout primitive | direction, spacing, divider | behavior/utility validation: arrange spacing/divider; no standalone visual |
| `steppers` | `dev/material-ui/packages/mui-material/src/Stepper*` | sequential nav/list of steps | active, completed, disabled, error, optional, horizontal/vertical | visual + interaction: active step click, connector/status tones, focus |
| `switches` | `dev/material-ui/packages/mui-material/src/Switch` | `Switch` track + thumb | checked/unchecked, size, color, disabled, focus, hover/pressed | visual + interaction: toggle click, track/thumb pixels, disabled no-toggle |
| `table` | `dev/material-ui/packages/mui-material/src/Table*` | `Table` / `TableRow` / virtual table | head/body/footer, selected row, hover, sort label, pagination | visual + interaction: row select, sort click, hover/focus pixels, pagination bridge |
| `tabs` | `dev/material-ui/packages/mui-material/src/Tabs`; `Tab`; `TabScrollButton` | tab bar + tab item | selected, disabled, scrollable, indicator, icon+label, orientation | visual + interaction: tab click route, selected indicator/container, focus traversal |
| `text-fields` | `dev/material-ui/packages/mui-material/src/TextField`; `InputBase`; `Input`; `FilledInput`; `OutlinedInput` | `InputField` / `TextField` / outlined or underline field | outlined/filled/standard, label/helper, error, disabled, focused, multiline, select-mode | visual + interaction: focus ring/underline, text edit, submit/change routes, error/helper tone |
| `textarea-autosize` | `dev/material-ui/packages/mui-material/src/TextareaAutosize` | multiline `TextField` / `TextEdit` | min/max rows, resize/autosize, disabled, error, focus | visual + interaction: multiline edit, height constraints, focus/error pixels |
| `timeline` | `dev/material-ui/packages/mui-lab/src/Timeline*` | vertical list with connector/dot/content slots | position, connector, dot color, opposite content | visual contract: connector/dot alignment; interaction belongs to child actions |
| `toggle-button` | `dev/material-ui/packages/mui-material/src/ToggleButton*` | `ToggleButton` / `SegmentedControl` | selected, exclusive/multiple group, disabled, hover/pressed/focus | visual + interaction: toggle route, group exclusivity/multi-select, state pixels |
| `tooltips` | `dev/material-ui/packages/mui-material/src/Tooltip` | anchored lightweight popup | placement, arrow, open/closed, disabled anchor wrapper, touch/hover/focus trigger | visual + interaction: hover/focus opens tooltip, close on leave, placement/arrow pixels |
| `transfer-list` | docs composite over `List`, `Checkbox`, `Button` | dual list selection tool | selected items, move left/right/all, disabled actions | visual + interaction: row select, move buttons, list count feedback |
| `transitions` | `dev/material-ui/packages/mui-material/src/Collapse`; `Fade`; `Grow`; `Slide`; `Zoom` | state transition metadata for open/expanded surfaces | entered/exited, direction, duration, collapsed size | behavior/utility validation: first pass freezes static end states; motion deferred |
| `typography` | `dev/material-ui/packages/mui-material/src/Typography` | `Label` / `RichLabel` | variant title/body/meta/code, color, no-wrap, gutter | visual contract: font size/weight/tone, overflow/elide, line height |
| `use-media-query` | `dev/material-ui/packages/mui-material/src/useMediaQuery` | responsive layout condition utility | query match, breakpoint-driven asset/layout selection | behavior/utility validation: layout choice only; no drawn control |

## Implementation Batches

1. **Core Inputs**: `buttons`, `button-group`, `floating-action-button`, `text-fields`, `textarea-autosize`, `number-field`, `selects`, `autocomplete`, `checkboxes`, `radio-buttons`, `switches`, `slider`, `toggle-button`, `rating`.
2. **Data Display**: `chips`, `dividers`, `icons`, `material-icons`, `avatars`, `badges`, `lists`, `table`, `typography`, `image-list`, `skeleton`, `timeline`.
3. **Feedback And Overlays**: `alert`, `backdrop`, `dialogs`, `modal`, `popover`, `popper`, `tooltips`, `snackbars`, `progress`, `speed-dial`.
4. **Surfaces And Navigation**: `accordion`, `app-bar`, `cards`, `paper`, `drawers`, `breadcrumbs`, `bottom-navigation`, `menubar`, `menus`, `pagination`, `steppers`, `tabs`.
5. **Layout And Utilities**: `box`, `container`, `grid`, `stack`, `masonry`, `transfer-list`, `transitions`, `click-away-listener`, `css-baseline`, `init-color-scheme-script`, `no-ssr`, `portal`, `use-media-query`.

Each batch must keep the same acceptance shape: update theme/style tokens first, update retained painter/command support only when a row is marked `needs support`, add showcase examples, then add visual + interaction tests for interactive components.
