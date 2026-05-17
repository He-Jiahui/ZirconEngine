# MUI All Components Detailed Design Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Turn the current MUI all-components matrix into a per-component, Button-quality design and implementation sequence for Zircon retained UI.

**Architecture:** Keep the existing `docs/ui-and-layout/material-ui-component-design-matrix.md`, `zircon_editor/assets/ui/editor/material_components/material_*.zui`, UI v2 selector/token cascade, typed style bridge, runtime component catalog, editor projection, and retained painter paths. Each component first receives a detailed plan block, then implementation proceeds bottom-up from shared style/behavior support into prototype assets, painter/projection, tests, and docs.

**Tech Stack:** Rust Cargo workspace (`zircon_runtime_interface`, `zircon_runtime`, `zircon_editor`), UI v2 `.ui.toml`/`.zui`, TOML style tokens, retained editor host painter, WSL-focused Cargo validation.

---

## Scope And Sources

- User-selected scope: option 3, meaning preserve the existing matrix scope and add current-page gaps.
- Source page: <https://mui.com/material-ui/all-components/> fetched on 2026-05-17.
- Repository baseline: `docs/ui-and-layout/material-ui-component-design-matrix.md` already covers 63 MUI Core/Utils/Lab rows plus 10 MUI X extension rows.
- Required scope correction: add a first-class MUI X `Date and Time Pickers` row and prototype plan. Current local references exist under `dev/material-ui/packages/mui-lab/src/*Date*Picker*` and current Zircon docs mention `MaterialDatePickerPopup` / `MaterialTimePickerPopup`, but the design matrix does not yet carry the MUI X row.
- Do not remove existing prototype assets. Use the current `material_*.zui` files as the landing zone and only add/reshape files when the per-component plan says so.

## Shared Per-Component Design Template

Every component plan below follows the Button slice shape:

1. **MUI Source:** exact MUI docs/source family to inspect before implementation.
2. **Zircon Shape:** retained UI component(s), layout roles, and owner crate/module.
3. **Typed Contract:** props, style fields, enums, state fields, and whether new shared DTOs are needed.
4. **Runtime Behavior:** pointer/keyboard/accessibility/default action behavior, dirty domains, and `needs support` notes.
5. **Editor Projection/Painter:** how `ViewTemplateNodeData` / `TemplatePaneNodeData` and `host_contract/painter` consume the resolved state.
6. **Prototype Asset:** `.zui` file and `editor_material.v2.ui.toml` selectors/tokens to touch.
7. **Validation:** focused runtime/editor tests plus docs checks. Compile/build commands run in milestone testing stages, not after each small slice.
8. **Docs:** update matrix, token audit, module docs, and any source-path docs when code modules change.

## File Responsibility Map

- `docs/ui-and-layout/material-ui-component-design-matrix.md`: canonical component inventory, per-component design state, `.zui` mapping, validation promise.
- `docs/ui-and-layout/material-ui-token-component-audit.md`: theme/style token rationale, painter priority, validation evidence, known gaps.
- `docs/zircon_runtime/ui/v2.md`: selector cascade, runtime pseudo-state, dirty-domain, typed style bridge behavior.
- `docs/zircon_runtime_interface/ui/contract-spine.md`: shared DTO/style contract additions when new typed style families are created.
- `zircon_runtime_interface/src/ui/style.rs`: shared typed style values only when a component needs reusable non-Button typed style DTOs.
- `zircon_runtime/src/ui/style.rs`: generic typed style resolver additions; keep component-specific parsing coherent and reusable.
- `zircon_runtime/src/ui/v2/style.rs`: selector/runtime pseudo-state support and dirty-domain classification; avoid component-only special cases unless the key is a true visual style field.
- `zircon_runtime/src/ui/component/catalog/material_foundation/**`: component descriptor props/default classes split by family.
- `zircon_editor/assets/ui/theme/editor_material.v2.ui.toml`: shared dark Material tokens/selectors and state priority.
- `zircon_editor/assets/ui/editor/material_components/material_*.zui`: one visible prototype per component family, with state strip and representative feedback route where interactive.
- `zircon_editor/src/ui/layouts/views/**`: editor/runtime projection only when new typed data must reach retained host models.
- `zircon_editor/src/ui/retained_host/host_contract/**`: host DTO/painter/presenter support only when prototype metadata needs native rendering beyond existing quad/text/image primitives.
- `zircon_runtime/src/ui/tests/**`: runtime style, v2 asset, event routing, component catalog, and support behavior tests.
- `zircon_editor/src/tests/ui/boundary/material_component_lab/**`: prototype inventory, projection, feedback, and shell tests.
- `zircon_editor/src/tests/host/retained_window/native_material_painter.rs`: native painter pixel/state tests for visual components.

## Milestone 0: Matrix Reconciliation And Planning Artifacts

### Task 0.1: Freeze Current MUI Inventory

**Files:**
- Modify: `docs/ui-and-layout/material-ui-component-design-matrix.md`
- Modify: `docs/ui-and-layout/material-ui-token-component-audit.md`

- [ ] Add a short dated note that the current authoritative page is MUI v9.0.1 all-components.
- [ ] Keep existing 63 Core/Utils/Lab rows and 10 MUI X extension rows.
- [ ] Add `Date and Time Pickers` as an MUI X row with source `<https://mui.com/x/react-date-pickers/>` and local fallback references `dev/material-ui/packages/mui-lab/src/DatePicker`, `DateTimePicker`, `DesktopDatePicker`, `MobileDatePicker`, and `StaticDatePicker`.
- [ ] Map the first Zircon prototype to a new `material_mui_x_date_time_pickers.zui` unless an implementation review proves an existing date/time popup prototype should own it.
- [ ] Record that this row is prototype-first and may reuse `MaterialDatePickerPopup` / `MaterialTimePickerPopup` semantics from existing Zircon docs.

### Task 0.2: Add Plan-State Markers To Existing Matrix Rows

**Files:**
- Modify: `docs/ui-and-layout/material-ui-component-design-matrix.md`

- [ ] For each component row, add or update a concise planning marker using this vocabulary: `planned`, `prototype exists`, `typed style needed`, `runtime support needed`, `painter support needed`, `accepted`.
- [ ] Mark `buttons` as `accepted` for the typed Button slice.
- [ ] Mark layout/utility rows as `prototype exists` or `runtime support needed` rather than forcing fake visual controls.
- [ ] Mark `masonry`, `popper`, and advanced chart/date picker behavior as `runtime support needed` where current layout/placement/data engines are missing.

### Task 0.3: Milestone 0 Testing Stage

**Files:**
- Test: `zircon_editor/src/tests/ui/boundary/material_ui_component_design_matrix.rs`

- [ ] Update the matrix boundary test so every explicit `material_*.zui` filename in the matrix exists.
- [ ] Add a static assertion that the matrix includes `Date and Time Pickers` and its prototype mapping.
- [ ] Run focused validation in WSL after the docs/test update:
  - `CARGO_TARGET_DIR=/mnt/e/cargo-targets/zircon-shared/wsl-mui-plan cargo test -p zircon_editor --lib material_ui_component_design_matrix --locked --jobs 1`
  - `cargo fmt --all --check`
  - `git diff --check -- docs/ui-and-layout/material-ui-component-design-matrix.md docs/ui-and-layout/material-ui-token-component-audit.md zircon_editor/src/tests/ui/boundary/material_ui_component_design_matrix.rs`

## Milestone 1: Inputs Component Plans

### Button Group

- **MUI Source:** `dev/material-ui/packages/mui-material/src/ButtonGroup`, Button/ButtonBase sources.
- **Zircon Shape:** segmented `HorizontalBox` or `VerticalBox` containing `Button`/`ToggleButton` children.
- **Typed Contract:** reuse `ResolvedButtonStyle`; add group metadata for orientation, attached radius role (`first`, `middle`, `last`, `only`) only if current props cannot express it.
- **Runtime Behavior:** group itself is structural; child buttons own click/press/focus. Disabled group propagates disabled state to children through prototype props.
- **Editor Projection/Painter:** painter must show shared borders and clipped interior radii without changing Button text/icon layout.
- **Prototype Asset:** `material_button_group.zui`; theme selectors for grouped first/middle/last border/radius.
- **Validation:** group radius/border pixel test, child click route test, disabled group no-action test, catalog props test.
- **Docs:** update design matrix row and token audit with group radius semantics.

### Floating Action Button

- **MUI Source:** `dev/material-ui/packages/mui-material/src/Fab` plus ButtonBase.
- **Zircon Shape:** elevated circular or extended `IconButton`/`Button` with high-emphasis action style.
- **Typed Contract:** reuse Button typed style; add `button_shape = circular|extended|pill` only if radius/size tokens are insufficient.
- **Runtime Behavior:** click/hover/press/focus match Button; disabled no-click.
- **Editor Projection/Painter:** circle/pill radius and elevation/shadow token must reach native painter.
- **Prototype Asset:** `material_floating_action_button.zui` with circular, small, and extended samples.
- **Validation:** circular radius pixel sample, extended icon+label layout, click/disabled guards.
- **Docs:** record whether elevation remains token-only or painter-backed.

### Toggle Button

- **MUI Source:** `dev/material-ui/packages/mui-material/src/ToggleButton`, ToggleButtonGroup.
- **Zircon Shape:** `ToggleButton` / segmented selectable button.
- **Typed Contract:** reuse Button typed style plus `selected`/`checked`; group exclusivity via existing selection state where possible.
- **Runtime Behavior:** click toggles selected/checked; exclusive groups clear siblings; multi-select groups preserve independent checked state.
- **Editor Projection/Painter:** selected state uses selected container and focus/pressed priority.
- **Prototype Asset:** `material_toggle_button.zui`.
- **Validation:** toggle click, exclusive group, multi-select, disabled no-toggle, selected pixel.
- **Docs:** clarify selected vs checked mapping.

### Text Field

- **MUI Source:** `TextField`, `InputBase`, `Input`, `FilledInput`, `OutlinedInput`.
- **Zircon Shape:** `InputField` / `TextField` with label, helper, border/underline frame.
- **Typed Contract:** define reusable field style fields only if existing `validation_level`, `text_tone`, `focused`, `disabled`, `value`, `placeholder` are insufficient.
- **Runtime Behavior:** focus, edit, commit, disabled, error/helper text; text measurement remains in text layout support.
- **Editor Projection/Painter:** focused ring/underline, error border/helper tone, disabled text and caret rules.
- **Prototype Asset:** `material_text_fields.zui`.
- **Validation:** focus ring, edit/commit event, helper/error tone, disabled no-edit, multiline/select-mode compatibility.
- **Docs:** update input-event and token audit docs.

### Textarea Autosize

- **MUI Source:** `TextareaAutosize`.
- **Zircon Shape:** multiline `TextEdit`/`TextField` with min/max row constraints.
- **Typed Contract:** row constraints as layout metadata; no new shared style unless needed.
- **Runtime Behavior:** multiline edit, commit, focus, disabled, height clamp.
- **Editor Projection/Painter:** multi-line text frame and focused/error border.
- **Prototype Asset:** `material_textarea_autosize.zui`.
- **Validation:** min/max row layout, multiline edit, focus/error pixels.
- **Docs:** record that live autosize is layout-affecting and not render-only.

### Number Field

- **MUI Source:** MUI docs `Number Field`; Zircon `NumberField`/`SpinBox` support.
- **Zircon Shape:** numeric input with stepper and optional drag edit.
- **Typed Contract:** `value`, `min`, `max`, `step`, `large_step`, `value_number`; no string-only numeric state.
- **Runtime Behavior:** increment/decrement, drag delta, clamp, commit, disabled/error.
- **Editor Projection/Painter:** stepper buttons and focused/error border.
- **Prototype Asset:** `material_number_field.zui`.
- **Validation:** step/clamp, drag, commit, disabled no-change, error tone.
- **Docs:** update component state numeric behavior docs.

### Select

- **MUI Source:** `Select`, `NativeSelect`, `SelectInput`.
- **Zircon Shape:** `ComboBox`/`Dropdown` plus popup menu.
- **Typed Contract:** selected option id(s), option labels, `popup_open`, disabled options; use typed option rows where available.
- **Runtime Behavior:** open/close, option hover/select, multiple chips, disabled option rejection, focus/escape close.
- **Editor Projection/Painter:** outlined/filled/standard field frame and popup row states.
- **Prototype Asset:** `material_selects.zui`.
- **Validation:** open route, select option, disabled option, multiple chip display, focus/error pixels.
- **Docs:** update popup/default interaction docs.

### Autocomplete

- **MUI Source:** `Autocomplete`, `useAutocomplete`, lab hook.
- **Zircon Shape:** `SearchSelect` with text input, option popup, selected chips.
- **Typed Contract:** query string, filtered options, selected ids, multiple/free-solo flags if supported.
- **Runtime Behavior:** query edit opens/filter popup; option select; chip remove; keyboard focus path.
- **Editor Projection/Painter:** input and popup row projection; matched text can remain visual placeholder until rich text support.
- **Prototype Asset:** `material_autocomplete.zui`.
- **Validation:** edit query, open popup, select option, remove chip, disabled option.
- **Docs:** mark free-solo and async loading as future support if not implemented.

### Checkbox

- **MUI Source:** `Checkbox`, `FormControlLabel`, `FormGroup`.
- **Zircon Shape:** checkbox row with glyph and label.
- **Typed Contract:** checked, indeterminate, disabled, validation/error.
- **Runtime Behavior:** click label or box toggles; disabled no-toggle; indeterminate resolves to checked on click per chosen policy.
- **Editor Projection/Painter:** check/indeterminate glyph and state layer.
- **Prototype Asset:** `material_checkboxes.zui`.
- **Validation:** checked/unchecked/indeterminate, label click, disabled no-toggle, error/focus pixels.
- **Docs:** define indeterminate transition explicitly.

### Radio Group

- **MUI Source:** `Radio`, `RadioGroup`.
- **Zircon Shape:** single-select radio row group.
- **Typed Contract:** group value, option ids, checked, disabled, error.
- **Runtime Behavior:** selecting one option clears previous; keyboard navigation follows focus plan.
- **Editor Projection/Painter:** radio circle/dot glyph and label state.
- **Prototype Asset:** `material_radio_buttons.zui`.
- **Validation:** exclusive selection, disabled option rejection, focus/error pixels.
- **Docs:** connect to selection state contract.

### Switch

- **MUI Source:** `Switch`.
- **Zircon Shape:** switch track + thumb control.
- **Typed Contract:** checked, disabled, size/color if needed.
- **Runtime Behavior:** click toggles, disabled no-toggle, focus/hover/press states.
- **Editor Projection/Painter:** track/thumb geometry; may need painter support if current quads cannot express thumb overlap cleanly.
- **Prototype Asset:** `material_switches.zui`.
- **Validation:** toggle, disabled no-toggle, checked/unchecked pixels, focus ring.
- **Docs:** record whether switch thumb is asset/image or painter primitive.

### Slider

- **MUI Source:** `Slider`, `useSlider`.
- **Zircon Shape:** `Slider`/`RangeField` with track, fill, thumb, marks.
- **Typed Contract:** numeric value/range, min/max/step, marks, disabled.
- **Runtime Behavior:** click/drag updates value; keyboard increments; a11y increment/decrement; render-only value changes.
- **Editor Projection/Painter:** track/fill/thumb/mark geometry and optional value label.
- **Prototype Asset:** `material_slider.zui`.
- **Validation:** drag, keyboard step, marks/value label, disabled, render-only rebuild.
- **Docs:** update range behavior docs when extended.

### Rating

- **MUI Source:** `Rating`.
- **Zircon Shape:** icon row with filled/empty/hover preview icons.
- **Typed Contract:** value, max, precision, read-only, disabled, hover value.
- **Runtime Behavior:** hover previews value; click commits; read-only no-click.
- **Editor Projection/Painter:** icon tint and partial fill. Partial precision may need painter/image mask support.
- **Prototype Asset:** `material_rating.zui`.
- **Validation:** hover preview, select value, read-only/disabled, half-step visual if supported.
- **Docs:** mark partial icon fill as `needs support` if not implemented.

### Transfer List

- **MUI Source:** docs composite over List, Checkbox, Button.
- **Zircon Shape:** dual list selection tool.
- **Typed Contract:** source/target item arrays, selected ids, move actions.
- **Runtime Behavior:** select rows, move selected/all left/right, disabled move buttons.
- **Editor Projection/Painter:** dual panels, row check state, move buttons.
- **Prototype Asset:** `material_transfer_list.zui`.
- **Validation:** row select, move selected/all, disabled no-op, list counts.
- **Docs:** classify as composite built from List/Checkbox/Button.

## Milestone 1 Testing Stage

- [ ] Run focused runtime tests after all Inputs slices in the milestone are implemented:
  - `CARGO_TARGET_DIR=/mnt/e/cargo-targets/zircon-shared/wsl-mui-inputs cargo test -p zircon_runtime --lib component_catalog --locked --jobs 1`
  - `CARGO_TARGET_DIR=/mnt/e/cargo-targets/zircon-shared/wsl-mui-inputs cargo test -p zircon_runtime --lib v2_asset --locked --jobs 1`
  - `CARGO_TARGET_DIR=/mnt/e/cargo-targets/zircon-shared/wsl-mui-inputs cargo test -p zircon_runtime --lib event_routing --locked --jobs 1`
- [ ] Run focused editor tests:
  - `CARGO_TARGET_DIR=/mnt/e/cargo-targets/zircon-shared/wsl-mui-inputs cargo test -p zircon_editor --lib material_component_lab --locked --jobs 1`
  - `CARGO_TARGET_DIR=/mnt/e/cargo-targets/zircon-shared/wsl-mui-inputs cargo test -p zircon_editor --lib native_material_painter --locked --jobs 1`
  - `CARGO_TARGET_DIR=/mnt/e/cargo-targets/zircon-shared/wsl-mui-inputs cargo test -p zircon_editor --lib template_assets --locked --jobs 1`
- [ ] Run `cargo fmt --all --check` and focused `git diff --check` for touched docs/assets/tests.
- [ ] Debug failures from lower shared support upward before changing individual prototypes.

## Milestone 2: Data Display Component Plans

### Chip

- **MUI Source:** `Chip`.
- **Zircon Shape:** pill tag/button row with optional icon/avatar/delete action.
- **Typed Contract:** filled/outlined, color tone, clickable, deletable, selected, disabled.
- **Runtime Behavior:** chip click emits commit; delete icon emits delete action; disabled blocks both.
- **Editor Projection/Painter:** pill radius, icon/avatar inset, delete affordance.
- **Prototype Asset:** `material_chips.zui`.
- **Validation:** click/delete route, disabled guard, selected/outlined pixels.
- **Docs:** document delete action routing separately from chip activation.

### Divider

- **MUI Source:** `Divider`.
- **Zircon Shape:** one-pixel separator with optional text.
- **Typed Contract:** orientation, inset/middle, text align.
- **Runtime Behavior:** static visual only.
- **Editor Projection/Painter:** horizontal/vertical line and text spacing.
- **Prototype Asset:** `material_dividers.zui`.
- **Validation:** thickness/tone/orientation/text spacing pixels.
- **Docs:** no runtime behavior beyond layout.

### Icons And Material Icons

- **MUI Source:** `Icon`, `SvgIcon`, `mui-icons-material`.
- **Zircon Shape:** `Icon`/`SvgIcon` visual asset plus catalog lookup row.
- **Typed Contract:** icon name/source, size, tone, disabled/error/warning/success/info.
- **Runtime Behavior:** static unless embedded in button/list action.
- **Editor Projection/Painter:** asset lookup, tint, missing placeholder.
- **Prototype Asset:** `material_icons.zui`, `material_material_icons.zui`.
- **Validation:** tint/size/missing fallback; no direct interaction.
- **Docs:** distinguish generic SvgIcon from named catalog icons.

### Typography

- **MUI Source:** `Typography`.
- **Zircon Shape:** `Label`/`RichLabel` text roles.
- **Typed Contract:** variant, tone, nowrap, gutter, overflow.
- **Runtime Behavior:** static text; layout/text dirty when content/font changes.
- **Editor Projection/Painter:** font size/weight/line height/tone.
- **Prototype Asset:** `material_typography.zui`.
- **Validation:** variant font metrics, tone, nowrap/ellipsis if supported.
- **Docs:** map Material variants to Zircon text roles.

### Avatar And Badge

- **MUI Source:** `Avatar`, `AvatarGroup`, `Badge`.
- **Zircon Shape:** circular image/initial/icon and anchored badge overlay.
- **Typed Contract:** image source, initials, overlap count, badge content/dot/max/invisible/color.
- **Runtime Behavior:** static unless embedded in action row.
- **Editor Projection/Painter:** circle clip, text center, badge anchor placement.
- **Prototype Asset:** `material_avatars.zui`, `material_badges.zui`.
- **Validation:** clipping, fallback initials, group overlap, dot/count placement.
- **Docs:** record badge overlay limitations if anchor placement is approximate.

### List And Image List

- **MUI Source:** `List*`, `ImageList*`.
- **Zircon Shape:** `ListRow`, `VirtualList`, image grid/tile collection.
- **Typed Contract:** dense, selected, disabled, icon/avatar, secondary action, tile image/bar/action.
- **Runtime Behavior:** row hover/press/select; virtual range; tile action click.
- **Editor Projection/Painter:** row state layers, tile crop, tile bar overlay.
- **Prototype Asset:** `material_lists.zui`, `material_image_list.zui`.
- **Validation:** row hover/select/disabled, virtual visible range, tile crop/action click.
- **Docs:** list and image-list share row/tile state vocabulary.

### Table

- **MUI Source:** `Table*`.
- **Zircon Shape:** table header/body/footer with rows/cells and optional virtual table.
- **Typed Contract:** selected rows, hover, sort direction, pagination bridge, density.
- **Runtime Behavior:** row select, header sort click, pagination action.
- **Editor Projection/Painter:** header/body/footer lines, selected/hover row fill, sort glyph.
- **Prototype Asset:** `material_table.zui`.
- **Validation:** row hover/select, sort click, pagination bridge, selected pixels.
- **Docs:** connect to Data Grid where advanced virtualization begins.

### Timeline

- **MUI Source:** `mui-lab/src/Timeline*`.
- **Zircon Shape:** vertical list with connector, dot, opposite/content slots.
- **Typed Contract:** position, dot color, connector presence.
- **Runtime Behavior:** static; child actions own interaction.
- **Editor Projection/Painter:** connector/dot alignment and tone.
- **Prototype Asset:** `material_timeline.zui`.
- **Validation:** connector/dot pixels, alternating layout if supported.
- **Docs:** classify as Lab visual structure.

## Milestone 2 Testing Stage

- [ ] Run runtime `component_catalog`, `v2_asset`, and any list/table-specific tests added during the milestone.
- [ ] Run editor `material_component_lab`, `native_material_painter`, and `template_assets` focused gates.
- [ ] Run `cargo fmt --all --check` and focused `git diff --check`.

## Milestone 3: Feedback And Overlay Component Plans

### Alert

- **MUI Source:** `Alert`, `AlertTitle`.
- **Zircon Shape:** status banner/help row with optional icon/close action.
- **Typed Contract:** severity, variant, closeable, disabled action.
- **Runtime Behavior:** close emits action or hides row; severity is visual.
- **Editor Projection/Painter:** severity fill/border/text/icon.
- **Prototype Asset:** `material_alert.zui`.
- **Validation:** severity tones, close click, disabled close guard.
- **Docs:** severity token matrix.

### Backdrop, Modal, Dialog

- **MUI Source:** `Backdrop`, `Modal`, `Dialog*`.
- **Zircon Shape:** overlay scrim, modal behavior shell, centered/docked paper dialog.
- **Typed Contract:** open, backdrop alpha/invisible, modal owner, full-screen/full-width, scroll body.
- **Runtime Behavior:** backdrop click close, escape close, focus containment, disabled descendant rules.
- **Editor Projection/Painter:** scrim layer, paper radius/elevation, title/content/actions layout.
- **Prototype Asset:** `material_backdrop.zui`, `material_modal.zui`, `material_dialogs.zui`.
- **Validation:** open/close, backdrop click, focus route, a11y role snapshot, paper pixels.
- **Docs:** modal focus/backdrop behavior belongs to shared runtime support.

### Popover, Popper, Tooltip

- **MUI Source:** `Popover`, `Popper`, `Tooltip`.
- **Zircon Shape:** anchored overlay surface and lightweight tooltip popup.
- **Typed Contract:** anchor id/frame, placement, arrow, open, transform origin.
- **Runtime Behavior:** hover/focus opens tooltip; click-away/outside close; popper flip/prevent-overflow is `needs support` until placement policy exists.
- **Editor Projection/Painter:** anchored surface, arrow placeholder, elevated radius.
- **Prototype Asset:** `material_popover.zui`, `material_popper.zui`, `material_tooltips.zui`.
- **Validation:** open/close, anchor frame, placement placeholder, tooltip hover/focus.
- **Docs:** separate Popover behavior from Popper placement engine.

### Snackbar

- **MUI Source:** `Snackbar`, `SnackbarContent`.
- **Zircon Shape:** transient toast/status surface.
- **Typed Contract:** open, anchor origin, action, severity via Alert child, timeout metadata.
- **Runtime Behavior:** action click, close click, timeout recorded but motion/timer may be deferred.
- **Editor Projection/Painter:** toast surface, action button, severity child.
- **Prototype Asset:** `material_snackbars.zui`.
- **Validation:** close/action routes, anchor layout, severity child pixels.
- **Docs:** timeout semantics explicit if not active.

### Progress And Skeleton

- **MUI Source:** `LinearProgress`, `CircularProgress`, `Skeleton`.
- **Zircon Shape:** progress bar/spinner and placeholder surfaces.
- **Typed Contract:** determinate/indeterminate, value/buffer, shape variant, animation mode.
- **Runtime Behavior:** progress value is render-only; skeleton animation deferred unless motion support exists.
- **Editor Projection/Painter:** track/fill, spinner arc or fallback glyph, skeleton shape/radius.
- **Prototype Asset:** `material_progress.zui`, `material_skeleton.zui`.
- **Validation:** progress value pixels, buffer, shape variants, render-only rebuild.
- **Docs:** mark animation gaps.

### Speed Dial

- **MUI Source:** `SpeedDial*`.
- **Zircon Shape:** FAB-triggered action fan/menu.
- **Typed Contract:** open, direction, actions, tooltip labels, hidden/disabled.
- **Runtime Behavior:** FAB toggles open; action click emits action; outside close.
- **Editor Projection/Painter:** circular FAB and action positions.
- **Prototype Asset:** `material_speed_dial.zui`.
- **Validation:** open/close, action click, direction layout, disabled action.
- **Docs:** classify as composite over FAB/menu/tooltip.

## Milestone 3 Testing Stage

- [ ] Run runtime overlay/popup/default interaction tests plus `v2_asset`.
- [ ] Run editor Material Lab/projection/native painter tests.
- [ ] Run `cargo fmt --all --check` and focused diff checks.

## Milestone 4: Surfaces And Navigation Component Plans

### Accordion

- **MUI Source:** `Accordion*`.
- **Zircon Shape:** `Foldout`/`Group` header + details surface.
- **Typed Contract:** expanded, disabled, elevation, summary/details slots.
- **Runtime Behavior:** click toggles expanded; disabled no-toggle; focus/keyboard path.
- **Editor Projection/Painter:** header state layer, divider, expanded body visibility.
- **Prototype Asset:** `material_accordion.zui`.
- **Validation:** expand/collapse, disabled, focus, body visibility.
- **Docs:** connect to Foldout default interaction.

### App Bar, Card, Paper, Drawer

- **MUI Source:** `AppBar`, `Toolbar`, `Card*`, `Paper`, `Drawer`.
- **Zircon Shape:** toolbar strip, rounded/elevated surfaces, side/bottom overlay or docked panel.
- **Typed Contract:** surface variant, elevation, square/rounded, drawer anchor/open/mode.
- **Runtime Behavior:** surfaces static unless clickable; drawer open/close/backdrop/focus.
- **Editor Projection/Painter:** radius/elevation/border, media crop, action area, drawer anchor frame.
- **Prototype Asset:** `material_app_bar.zui`, `material_cards.zui`, `material_paper.zui`, `material_drawers.zui`.
- **Validation:** surface pixels, card action press, drawer open/close/backdrop.
- **Docs:** record elevation support status.

### Breadcrumbs, Bottom Navigation, Link

- **MUI Source:** `Breadcrumbs`, `BottomNavigation*`, `Link`.
- **Zircon Shape:** horizontal link row, bottom nav strip, text/link button.
- **Typed Contract:** current/selected, disabled, collapsed items, underline mode.
- **Runtime Behavior:** click route, collapsed menu open, selected nav state, disabled no-action.
- **Editor Projection/Painter:** separator icons, selected nav fill, underline/focus ring.
- **Prototype Asset:** `material_breadcrumbs.zui`, `material_bottom_navigation.zui`, `material_links.zui`.
- **Validation:** link click/hover, collapsed breadcrumbs, selected bottom nav.
- **Docs:** current item accessibility state.

### Menubar And Menu

- **MUI Source:** `Menu`, `MenuItem`, `MenuList`; current docs Menubar composite.
- **Zircon Shape:** workbench menu bar and popup menu frame/items.
- **Typed Contract:** open menu id, checked item, disabled, shortcut, separator.
- **Runtime Behavior:** hover/open menu, click item, close on click-away/escape, checked glyph.
- **Editor Projection/Painter:** menu bar active state, popup frame, item rows.
- **Prototype Asset:** `material_menubar.zui`, `material_menus.zui`.
- **Validation:** open/select/close, disabled item, checked item, shortcut text.
- **Docs:** align with widget menu behavior plans.

### Pagination, Stepper, Tabs

- **MUI Source:** `Pagination*`, `Stepper*`, `Tabs`, `Tab`, `TabScrollButton`.
- **Zircon Shape:** page button row, sequential step list, tab bar/item.
- **Typed Contract:** selected page/tab/step, completed/error/disabled, orientation, indicator.
- **Runtime Behavior:** click changes page/tab/step; disabled edge buttons no-op; focus traversal.
- **Editor Projection/Painter:** selected container/indicator, connector/status tones.
- **Prototype Asset:** `material_pagination.zui`, `material_steppers.zui`, `material_tabs.zui`.
- **Validation:** selection route, disabled edges, indicator pixels, step completion/error.
- **Docs:** connect to selection/default action contracts.

## Milestone 4 Testing Stage

- [ ] Run runtime component/default interaction tests and editor Material Lab/painter tests.
- [ ] Run `cargo fmt --all --check` and focused diff checks.

## Milestone 5: Layout, Utils, And Lab Component Plans

### Box, Container, Grid, Stack

- **MUI Source:** `Box`, `Container`, `Grid`, `Stack`, `mui-system`.
- **Zircon Shape:** style carrier, constrained content frame, grid/flex/stack layout primitives.
- **Typed Contract:** no standalone visual DTO unless optional surface style is used; layout metadata owns sizing/gaps.
- **Runtime Behavior:** layout/arrange only.
- **Editor Projection/Painter:** optional surface only when class applies.
- **Prototype Asset:** `material_box.zui`, `material_container.zui`, `material_grid.zui`, `material_stack.zui`.
- **Validation:** layout bounds, gaps, grid columns/spans, no fake action routes.
- **Docs:** classify as layout primitives.

### Masonry

- **MUI Source:** `mui-lab/src/Masonry`.
- **Zircon Shape:** staggered grid layout.
- **Typed Contract:** column count, gap, ordered placement.
- **Runtime Behavior:** `needs support` layout algorithm before full implementation.
- **Editor Projection/Painter:** placeholder prototype until layout support exists.
- **Prototype Asset:** `material_masonry.zui`.
- **Validation:** static placeholder now; layout algorithm tests later.
- **Docs:** keep `needs support` explicit.

### Portal, No SSR, Click-Away Listener, CSS Baseline, InitColorSchemeScript, useMediaQuery

- **MUI Source:** matching MUI utility sources.
- **Zircon Shape:** behavior/host/theme utility rows, not fake controls.
- **Typed Contract:** portal target/layer, click-away owner, theme baseline tokens, responsive query metadata where applicable.
- **Runtime Behavior:** portal/layer routing and click-away behavior only when shared support exists; No SSR is documentation-only in native retained UI.
- **Editor Projection/Painter:** compact placeholder rows for utilities.
- **Prototype Asset:** `material_portal.zui`, `material_no_ssr.zui`, `material_click_away_listener.zui`, `material_css_baseline.zui`, `material_init_color_scheme_script.zui`, `material_use_media_query.zui`.
- **Validation:** no dispatchable fake controls unless route-bearing support exists; token existence for CSS baseline/theme utilities.
- **Docs:** document native divergence from web-only utilities.

### Transitions

- **MUI Source:** `Collapse`, `Fade`, `Grow`, `Slide`, `Zoom`.
- **Zircon Shape:** transition metadata and static end-state prototypes.
- **Typed Contract:** entered/exited, direction, duration, collapsed size.
- **Runtime Behavior:** motion support deferred; static end-state validation first.
- **Editor Projection/Painter:** show entered/exited examples without animation.
- **Prototype Asset:** `material_transitions.zui`.
- **Validation:** state metadata and static visuals; no timing claims until motion engine exists.
- **Docs:** record motion gap.

## Milestone 5 Testing Stage

- [ ] Run layout-focused runtime tests for Grid/Stack/Container changes.
- [ ] Run Material Lab inventory/projection tests for utility placeholders.
- [ ] Run `cargo fmt --all --check` and focused diff checks.

## Milestone 6: MUI X Component Plans

### Tree View

- **MUI Source:** <https://mui.com/x/react-tree-view/>.
- **Zircon Shape:** `TreeView`/`TreeRow` with indentation and expansion.
- **Typed Contract:** expanded ids, selected ids, disabled ids, editable label.
- **Runtime Behavior:** expand/collapse, select, edit commit, focus traversal.
- **Editor Projection/Painter:** indentation, disclosure glyph, selected/focus row.
- **Prototype Asset:** `material_mui_x_tree_view.zui`.
- **Validation:** expand/select/edit, disabled, focus pixels.
- **Docs:** MUI X Community-visible prototype only.

### Data Grid

- **MUI Source:** <https://mui.com/x/react-data-grid/>.
- **Zircon Shape:** virtual table/data grid prototype.
- **Typed Contract:** columns, rows, visible range, selected rows, sort state, edit value.
- **Runtime Behavior:** virtual range, row select, header sort, edit commit.
- **Editor Projection/Painter:** sticky header/footer, hover/selected rows, sort glyph.
- **Prototype Asset:** `material_mui_x_data_grid.zui`.
- **Validation:** virtual range, sort, select, edit commit.
- **Docs:** Pro/Premium server adapters remain out of scope.

### Charts Family

- **MUI Source:** <https://mui.com/x/react-charts/> and line/bar/pie/sparkline/gauge pages.
- **Zircon Shape:** dashboard chart cards and placeholder chart primitives.
- **Typed Contract:** series, categories, selected/hover point, empty/error state.
- **Runtime Behavior:** hover highlight and select where simple; full chart engine is `needs support`.
- **Editor Projection/Painter:** line/bar/pie/sparkline/gauge static visuals with hover marker.
- **Prototype Asset:** `material_mui_x_charts.zui`, `material_mui_x_line_chart.zui`, `material_mui_x_bar_chart.zui`, `material_mui_x_pie_chart.zui`, `material_mui_x_sparkline.zui`, `material_mui_x_gauge.zui`.
- **Validation:** static geometry/tone, hover feedback routes, empty/error placeholder.
- **Docs:** chart engine limitations explicit.

### AgentChat And Chat Composer

- **MUI Source:** <https://mui.com/x/react-chat/>.
- **Zircon Shape:** conversation list, message list, composer.
- **Typed Contract:** messages, author role, streaming/typing/error, composer value, send disabled.
- **Runtime Behavior:** edit composer, send commit, close error; stop/cancel is `needs support`.
- **Editor Projection/Painter:** message bubbles, composer focus/error, send button state.
- **Prototype Asset:** `material_mui_x_agent_chat.zui`, `material_mui_x_chat_composer.zui`.
- **Validation:** composer edit/send, disabled send, error banner, typing/streaming visual.
- **Docs:** no network/LLM runtime behavior in Material prototype.

### Date And Time Pickers

- **MUI Source:** <https://mui.com/x/react-date-pickers/> plus local `dev/material-ui/packages/mui-lab/src/DatePicker`, `DateTimePicker`, `DateRangePicker`, `DesktopDatePicker`, `MobileDatePicker`, `StaticDatePicker`, `MobileDateTimePicker`, `StaticDateTimePicker`.
- **Zircon Shape:** date/time field with anchored calendar/time popup and static picker panel prototype.
- **Typed Contract:** selected date/time value, open popup, min/max date, disabled dates, range start/end, focused day/time cell, view mode (`day|month|year|hours|minutes`) if needed.
- **Runtime Behavior:** open/close popup, select day/time, keyboard navigation, disabled date rejection, range selection. Full calendar math/timezone behavior is `needs support` unless a shared date-time model already exists.
- **Editor Projection/Painter:** field frame, popup paper, calendar grid cells, selected/today/focused/disabled tones, time list rows.
- **Prototype Asset:** create `zircon_editor/assets/ui/editor/material_components/material_mui_x_date_time_pickers.zui` or deliberately bind to existing date/time popup assets if implementation chooses that owner.
- **Validation:** popup open/close, select date/time, disabled date no-op, selected/focused/today pixels, docs/static matrix row existence.
- **Docs:** update matrix, token audit, and popup/dialog lifecycle docs; explicitly state timezone/parsing limits.

## Milestone 6 Testing Stage

- [ ] Run MUI X Material Lab inventory/projection/feedback tests.
- [ ] Run runtime `component_catalog`, `v2_asset`, and added data-grid/tree/date-picker tests.
- [ ] Run editor `native_material_painter` if chart/date picker painter primitives changed.
- [ ] Run `cargo fmt --all --check` and focused diff checks.

## Cross-Milestone Acceptance Rules

- Keep root wiring files thin; new behavior belongs in owner modules.
- Do not add compatibility shims for old component names unless persisted/shipped data requires them.
- Do not introduce new stringly typed style helpers when a reusable typed enum/value is warranted.
- Do not make a component pass by special-casing a top-level prototype while lower shared support remains broken.
- For visual-only style changes, confirm dirty domains remain render-only where expected.
- For layout-affecting controls, explicitly accept layout dirty behavior rather than hiding it.
- For utilities, avoid fake clickable samples unless a real retained behavior route exists.

## Full Milestone Closeout Validation

At the end of each implemented milestone, run the focused commands named in that milestone. At the end of the full all-components pass, run:

```powershell
cargo fmt --all --check
wsl.exe --cd /mnt/e/Git/ZirconEngine bash -lc 'export CARGO_TARGET_DIR=/mnt/e/cargo-targets/zircon-shared/wsl-mui-all-components && cargo test -p zircon_runtime_interface --lib ui_contract_spine --locked --jobs 1 && cargo test -p zircon_runtime --lib component_catalog --locked --jobs 1 && cargo test -p zircon_runtime --lib v2_asset --locked --jobs 1 && cargo test -p zircon_runtime --lib event_routing --locked --jobs 1 && cargo test -p zircon_editor --lib material_ui_component_design_matrix --locked --jobs 1 && cargo test -p zircon_editor --lib material_component_lab --locked --jobs 1 && cargo test -p zircon_editor --lib native_material_painter --locked --jobs 1 && cargo test -p zircon_editor --lib template_assets --locked --jobs 1'
```

If Windows native Cargo remains blocked by the known `wgpu-hal` DX12 / `windows` crate mismatch, record the blocker and use WSL evidence as the CI-parity baseline.

## Plan Self-Review

- Spec coverage: the plan covers the existing matrix scope, preserves the Button accepted slice, names each MUI all-components group, and adds the missing `Date and Time Pickers` MUI X row.
- Placeholder scan: no `TBD`/`TODO` placeholders are intentionally left; `needs support` entries are explicit accepted capability gaps with validation behavior.
- Type consistency: component fields reuse the established names `button_variant`, `button_color`, `button_size`, `icon_placement`, `button_interaction_state`, `surface_variant`, `text_tone`, `validation_level`, `hovered`, `pressed`, `focused`, `selected`, `checked`, `disabled`, and `popup_open` unless a milestone explicitly adds a typed contract.
- Scope check: this remains a multi-milestone plan; each milestone has implementation slices and a named testing stage per `zirconEngine` milestone-first policy.
