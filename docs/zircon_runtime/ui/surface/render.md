---
related_code:
  - zircon_runtime/src/core/framework/render/backend_types.rs
  - zircon_runtime/src/core/diagnostics/collect.rs
  - zircon_runtime/src/ui/surface/component_state.rs
  - zircon_runtime/src/ui/surface/render/resolve.rs
  - zircon_runtime/src/ui/surface/render/node_visual_data.rs
  - zircon_runtime/src/ui/surface/render/painter_state.rs
  - zircon_runtime/src/ui/surface/render/extract.rs
  - zircon_runtime/src/ui/surface/surface/rebuild.rs
  - zircon_runtime/src/ui/surface/render/buttons.rs
  - zircon_runtime/src/ui/surface/render/collection_rows/mod.rs
  - zircon_runtime/src/ui/surface/render/collection_rows/shared.rs
  - zircon_runtime/src/ui/surface/render/collection_rows/list.rs
  - zircon_runtime/src/ui/surface/render/collection_rows/tree.rs
  - zircon_runtime/src/ui/surface/render/collection_rows/table.rs
  - zircon_runtime/src/ui/surface/render/dropdowns.rs
  - zircon_runtime/src/ui/surface/render/feedback.rs
  - zircon_runtime/src/ui/surface/render/popup_menu.rs
  - zircon_runtime/src/ui/surface/render/popup_options.rs
  - zircon_runtime/src/ui/surface/render/popup_rows.rs
  - zircon_runtime/src/ui/surface/render/selection_controls.rs
  - zircon_runtime/src/ui/surface/render/segmented_controls.rs
  - zircon_runtime/src/ui/surface/render/sliders.rs
  - zircon_runtime/src/ui/surface/render/text_fields.rs
  - zircon_runtime/src/ui/style.rs
  - zircon_runtime/src/ui/tests/shared_core.rs
  - zircon_runtime/src/ui/tests/render_atoms.rs
  - zircon_runtime/src/ui/tests/render_buttons.rs
  - zircon_runtime/src/ui/tests/render_collection_rows.rs
  - zircon_runtime/src/ui/tests/render_feedback.rs
  - zircon_runtime/src/ui/tests/render_popup_menu.rs
  - zircon_runtime/src/ui/tests/render_popup_options.rs
  - zircon_runtime/src/ui/tests/render_dropdowns.rs
  - zircon_runtime/src/ui/tests/render_selection_controls.rs
  - zircon_runtime/src/ui/tests/render_segmented_controls.rs
  - zircon_runtime/src/ui/tests/render_sliders.rs
  - zircon_runtime/src/ui/tests/render_text_fields.rs
  - zircon_runtime/src/ui/tests/render_painter_state.rs
  - zircon_runtime/src/ui/tests/runtime_drag_drop_component_state.rs
  - zircon_runtime/src/ui/tests/runtime_loading_component_state.rs
  - zircon_runtime/src/core/framework/tests.rs
  - zircon_runtime_interface/src/ui/tree/node/template_node_metadata.rs
  - zircon_runtime_interface/src/ui/surface/render/resolved_style.rs
implementation_files:
  - zircon_runtime/src/core/framework/render/backend_types.rs
  - zircon_runtime/src/core/diagnostics/collect.rs
  - zircon_runtime/src/ui/surface/component_state.rs
  - zircon_runtime/src/ui/surface/render/resolve.rs
  - zircon_runtime/src/ui/surface/render/node_visual_data.rs
  - zircon_runtime/src/ui/surface/render/painter_state.rs
  - zircon_runtime/src/ui/surface/render/extract.rs
  - zircon_runtime/src/ui/surface/surface/rebuild.rs
  - zircon_runtime/src/ui/surface/render/buttons.rs
  - zircon_runtime/src/ui/surface/render/collection_rows/mod.rs
  - zircon_runtime/src/ui/surface/render/collection_rows/shared.rs
  - zircon_runtime/src/ui/surface/render/collection_rows/list.rs
  - zircon_runtime/src/ui/surface/render/collection_rows/tree.rs
  - zircon_runtime/src/ui/surface/render/collection_rows/table.rs
  - zircon_runtime/src/ui/surface/render/dropdowns.rs
  - zircon_runtime/src/ui/surface/render/feedback.rs
  - zircon_runtime/src/ui/surface/render/popup_menu.rs
  - zircon_runtime/src/ui/surface/render/popup_options.rs
  - zircon_runtime/src/ui/surface/render/popup_rows.rs
  - zircon_runtime/src/ui/surface/render/selection_controls.rs
  - zircon_runtime/src/ui/surface/render/segmented_controls.rs
  - zircon_runtime/src/ui/surface/render/sliders.rs
  - zircon_runtime/src/ui/surface/render/text_fields.rs
  - zircon_runtime/src/ui/tests/shared_core.rs
  - zircon_runtime/src/ui/tests/render_atoms.rs
  - zircon_runtime/src/ui/tests/render_buttons.rs
  - zircon_runtime/src/ui/tests/render_collection_rows.rs
  - zircon_runtime/src/ui/tests/render_feedback.rs
  - zircon_runtime/src/ui/tests/render_popup_menu.rs
  - zircon_runtime/src/ui/tests/render_popup_options.rs
  - zircon_runtime/src/ui/tests/render_dropdowns.rs
  - zircon_runtime/src/ui/tests/render_selection_controls.rs
  - zircon_runtime/src/ui/tests/render_segmented_controls.rs
  - zircon_runtime/src/ui/tests/render_sliders.rs
  - zircon_runtime/src/ui/tests/render_text_fields.rs
  - zircon_runtime/src/ui/tests/render_painter_state.rs
  - zircon_runtime/src/ui/tests/runtime_drag_drop_component_state.rs
  - zircon_runtime/src/ui/tests/runtime_loading_component_state.rs
  - zircon_runtime/src/core/framework/tests.rs
plan_sources:
  - user: 2026-06-01 workbench design recreation and engine implementation
  - user: 2026-06-03 native editor workbench window comparison screenshot request
tests:
  - zircon_runtime/src/tests/runtime_diagnostics/mod.rs::runtime_diagnostics_combines_core_render_contract_and_missing_externalized_plugins
  - rustfmt --edition 2021 --check zircon_runtime/src/ui/surface/render/resolve.rs zircon_runtime/src/ui/tests/shared_core.rs
  - rustfmt --edition 2021 --check zircon_runtime/src/ui/surface/render/extract.rs zircon_runtime/src/ui/surface/render/mod.rs zircon_runtime/src/ui/surface/render/selection_controls.rs zircon_runtime/src/ui/tests/render_selection_controls.rs zircon_runtime/src/ui/tests/mod.rs
  - rustfmt --edition 2021 --check zircon_runtime/src/ui/surface/render/mod.rs zircon_runtime/src/ui/surface/render/extract.rs zircon_runtime/src/ui/surface/render/segmented_controls.rs zircon_runtime/src/ui/tests/render_segmented_controls.rs zircon_runtime/src/ui/tests/mod.rs
  - node --check docs/ui-and-layout/ai-workbench-style/component-prototype/verify-native-component-contract.mjs
  - node docs/ui-and-layout/ai-workbench-style/component-prototype/verify-native-component-contract.mjs
  - git diff --check -- zircon_runtime/src/ui/surface/render/mod.rs zircon_runtime/src/ui/surface/render/extract.rs zircon_runtime/src/ui/surface/render/segmented_controls.rs zircon_runtime/src/ui/tests/render_segmented_controls.rs zircon_runtime/src/ui/tests/mod.rs docs/ui-and-layout/ai-workbench-style/component-prototype/verify-native-component-contract.mjs docs/zircon_runtime/ui/surface/render.md
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-workbench-segmented-controls-check --message-format short --color never
  - cargo test -p zircon_runtime --lib render_extract_expands_tabs_and_segmented_control_primitives --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-workbench-segmented-controls --message-format short --color never -- --nocapture --test-threads=1 timed out after 904 seconds during test-binary compilation/linking with no Rust diagnostics and no zircon_runtime test executable produced
  - cargo test -p zircon_runtime --lib render_extract_expands_tabs_and_segmented_control_primitives --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-workbench-segmented-controls-check --message-format short --color never -- --nocapture --test-threads=1 timed out after 904 seconds during test-binary compilation/linking with no Rust diagnostics and no zircon_runtime test executable produced
  - cargo check -p zircon_runtime --tests --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-workbench-segmented-controls-test-check --message-format short --color never
  - cargo test -p zircon_runtime --lib render_extract_expands_open_context_action_menu_items --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-workbench-command-feedback -- --nocapture
  - pending focused Cargo rerun: cargo test -p zircon_runtime --lib render_extract_expands_open_dropdown_options --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-workbench-popup-options -- --nocapture
  - pending focused Cargo rerun: cargo test -p zircon_runtime --lib render_extract_expands_dropdown_trigger_primitives --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-workbench-dropdowns --color never -- --nocapture
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-render-state-selector-0605 --message-format short --color never
  - cargo test -p zircon_runtime --lib render_extract_dropdown_uses_shared_metadata_painter_state_priority --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-render-state-selector-0605 --message-format short --color never -- --nocapture --test-threads=1
  - cargo test -p zircon_runtime --lib render_extract_expands_selection_control_indicators --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-workbench-selection-controls --color never -- --nocapture
  - cargo test -p zircon_runtime --lib render_extract_expands_slider_primitives --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-workbench-sliders --color never -- --nocapture
  - cargo test -p zircon_runtime --lib render_extract_expands_text_field --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-workbench-text-fields --message-format short --color never -- --nocapture --test-threads=1 timed out during compilation; the produced test binary was then run directly: D:\cargo-targets\zircon-editor-workbench-text-fields\debug\deps\zircon_runtime-b34ee8d8fc52f1fd.exe render_extract_expands_text_field --nocapture --test-threads=1
  - cargo test -p zircon_runtime --lib render_extract_expands_button --locked --jobs 1 --target-dir D:\cargo-targets\zircon-asset-m6-root-0604-fresh --message-format short --color never -- --nocapture --test-threads=1
  - cargo test -p zircon_runtime --lib render_extract_expands_collection_row_primitives --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-workbench-collection-rows --message-format short --color never -- --nocapture --test-threads=1 timed out during compilation; the produced test binary was then run directly: D:\cargo-targets\zircon-editor-workbench-collection-rows\debug\deps\zircon_runtime-b34ee8d8fc52f1fd.exe render_extract_expands_collection_row_primitives --nocapture --test-threads=1
  - rustfmt --edition 2021 --check zircon_runtime/src/ui/surface/render/feedback.rs zircon_runtime/src/ui/surface/render/extract.rs zircon_runtime/src/ui/surface/render/mod.rs zircon_runtime/src/ui/tests/render_feedback.rs zircon_runtime/src/ui/tests/mod.rs
  - git diff --check -- zircon_runtime/src/ui/surface/render/feedback.rs zircon_runtime/src/ui/surface/render/extract.rs zircon_runtime/src/ui/surface/render/mod.rs zircon_runtime/src/ui/tests/render_feedback.rs zircon_runtime/src/ui/tests/mod.rs
  - cargo test -p zircon_runtime --lib render_extract_expands_feedback_primitives --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-workbench-feedback --message-format short --color never -- --nocapture --test-threads=1 timed out while compiling under shared desktop build load; the target lane later produced D:\cargo-targets\zircon-editor-workbench-feedback\debug\deps\zircon_runtime-b34ee8d8fc52f1fd.exe
  - D:\cargo-targets\zircon-editor-workbench-feedback\debug\deps\zircon_runtime-b34ee8d8fc52f1fd.exe render_extract_expands_feedback_primitives --nocapture --test-threads=1
  - rustfmt --edition 2021 --check zircon_runtime/src/ui/tests/render_atoms.rs zircon_runtime/src/ui/tests/mod.rs
  - git diff --check -- zircon_runtime/src/ui/tests/render_atoms.rs zircon_runtime/src/ui/tests/mod.rs docs/zircon_runtime/ui/surface/render.md
  - cargo test -p zircon_runtime --lib render_extract_carries_label_and_icon_atoms_through_generic_path --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-workbench-atoms --message-format short --color never -- --nocapture --test-threads=1 timed out after 124 seconds with no Rust diagnostics, no remaining atoms target-dir cargo/rustc process, and no zircon_runtime test executable produced
  - cargo test -p zircon_runtime --lib render_extract_carries_label_and_icon_atoms_through_generic_path --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-workbench-feedback --message-format short --color never -- --nocapture --test-threads=1
  - rustfmt --edition 2021 --check zircon_runtime/src/ui/surface/render/painter_state.rs zircon_runtime/src/ui/surface/render/resolve.rs zircon_runtime/src/ui/surface/render/node_visual_data.rs zircon_runtime/src/ui/surface/render/extract.rs zircon_runtime/src/ui/surface/render/mod.rs zircon_runtime/src/ui/surface/surface/rebuild.rs zircon_runtime/src/ui/surface/render/buttons.rs zircon_runtime/src/ui/surface/render/selection_controls.rs zircon_runtime/src/ui/surface/render/segmented_controls.rs zircon_runtime/src/ui/surface/render/sliders.rs zircon_runtime/src/ui/surface/render/dropdowns.rs zircon_runtime/src/ui/surface/render/text_fields.rs zircon_runtime/src/ui/surface/render/collection_rows/mod.rs zircon_runtime/src/ui/surface/render/collection_rows/shared.rs zircon_runtime/src/ui/surface/render/feedback.rs zircon_runtime/src/ui/tests/mod.rs zircon_runtime/src/ui/tests/render_painter_state.rs
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-block-box-0605 --message-format short --color never
  - cargo test -p zircon_runtime --lib render_extract_uses_component_state_store_for_shared_painter_priority --locked --jobs 1 --target-dir D:\cargo-targets\zircon-block-box-0605 --message-format short --color never -- --nocapture --test-threads=1
  - rustfmt --edition 2021 --check zircon_runtime/src/ui/tests/render_painter_state.rs
  - git diff --check -- zircon_runtime/src/ui/tests/render_painter_state.rs docs/zircon_runtime/ui/surface/render.md .codex/sessions/20260603-1955-host-editor-ui-foundation.md
  - touched render painter selector files trailing-whitespace scan returned trailingWhitespace=0
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-keyboard-clipboard-extract-0605 --message-format short --color never
  - cargo test -p zircon_runtime --lib render_extract_uses_component_state_store_for_shared_painter_priority --locked --jobs 1 --target-dir D:\cargo-targets\zircon-keyboard-clipboard-extract-0605 --message-format short --color never -- --nocapture --test-threads=1
doc_type: module-detail
---

# Runtime UI Surface Render

`surface/render` converts arranged `UiTree` nodes into the neutral `UiRenderExtract` consumed by the runtime WGPU UI pass and editor host painters. The extract is authoritative for visible frame, clip frame, z index, style, text, image, opacity, and editable text decorations. It does not invent a second widget state model; it reads the node's `UiTemplateNodeMetadata` plus arranged geometry.

Runtime render submit exposes UI composition scale through `RenderStats.last_ui_*`: command, quad, text payload, image payload, clipped command, and graph-executed pass counts. `collect_runtime_diagnostics(...)` mirrors those values into `DiagnosticStore` under `render.ui.*` paths so tooling can observe UI payload and graph placement scale without reading the retained graph or the concrete WGPU UI pass.

## Shared Painter State

`painter_state.rs` is the render-extract-side state adapter for component painters. It converts retained `UiSurfaceComponentStateStore` flags, `UiTemplateNodeMetadata`, and runtime `UiStateFlags` into `UiPainterState`, including shared aliases for `popup_open`, `active_drag_target`, runtime pressed state, runtime checked state, disabled state, and loading state. Component extractors then resolve their own `UiPainterFamily` from that one state object instead of rebuilding hover/focus/pressed/open/drag/drop/loading booleans locally.

`UiSurface::rebuild_render_extract(...)` passes the retained component state store into `extract_ui_render_tree_from_arranged_with_component_states(...)`, while the public tree-only extraction helpers keep their no-store behavior for standalone callers. This makes input-derived hover, focus, press, checked, selected, popup-open, drag, drop-hover, and loading state visible to the runtime render stream even when the authored template metadata remains neutral.

Drag/drop source and target state is now produced by `UiDispatchEffect::DragDrop` instead of authored metadata mutation: accepted `Begin`/`Update`/`Accept` effects reduce `dragging`, `drop_hovered`, and `active_drag_target` into `UiSurfaceComponentStateStore`, while `Reject`, `Complete`, and `Cancel` clear the retained target/source paint flags according to the active session. Render extraction only consumes the retained flags; it does not need to know whether the state came from initial metadata, pointer routing, or shared reply effects.

This keeps the runtime extract aligned with the retained-host selector model: input, retained component flags, and template metadata produce semantic state first, then Button, IconButton, Dropdown, TextField, Slider, Checkbox, Radio, Toggle, ListRow, TreeRow, TableRow, Tab, Alert, Tooltip, and Toast families choose their visual priority through `UiPainterState::resolved_state_for_family(...)`. Popup option rows still build row state from already-normalized row booleans because selected, hovered, pressed, focused, and disabled options are derived from the owning popup arrays rather than directly from the owner node metadata.

## Style Resolution

`resolve.rs` keeps render extraction aligned with the shared style resolver aliases. Color fields now accept both structured and flat authoring forms:

- background: `background = { color = "#..." }` or `background_color = "#..."`
- foreground: `foreground = { color = "#..." }`, `foreground_color = "#..."`, `fg = "#..."`, or `color = "#..."`
- border: `border = { color = "#...", width = 1.0, radius = 6.0 }`, `border_color = "#..."`, or `outline = "#..."`

This matters for editor workbench assets because the component theme uses flat `background_color`, `foreground_color`, and `border_color` rules for compactness, while the renderer still needs the same values in `UiResolvedStyle` before choosing `Quad`, `Text`, `Image`, or `Group` command kinds.

## Generic Atoms

`Label` and `Icon` stay on the generic render path instead of owning a component-specific extraction module. `resolve.rs` maps their text, icon/image, font, opacity, flat color aliases, and text layout metadata into a single `Text` or `Image` command with `UiPainterFamily::Generic`, while `extract.rs` still takes frame, clip, and z order from the arranged tree. This keeps base atoms cheap and predictable for workbench panel composition while reserving component modules for controls with multi-command chrome or state-specific subparts.

## Buttons And Icon Buttons

`buttons.rs` expands low-level `Button`, `ToggleButton`, and `IconButton` nodes into component-level render commands. It suppresses owner fallback text and owner fallback images for those components, then emits the button surface, optional leading icon, and visible label through the shared `UiPainterFamily::Button` or `UiPainterFamily::IconButton` state model. This keeps command buttons and toolbar icon buttons on the same component lane as the web atoms instead of relying on generic centered fallback text or a single image-only node command.

Button visual selection reads authored `button_color`, `button_variant`, `validation_level`, hover/focus/pressed/selected/disabled state, compact padding, icon size, and spacing metadata. Icon buttons keep accessibility labels paint-silent while rendering a centered icon plus selected/focused chrome, so native workbench toolbar controls can mirror the HTML icon-button primitive without showing labels inside icon-only controls.

## Collection Rows

`collection_rows/` expands low-level `ListRow`, `TreeRow`, `Table`, and `TableRow` nodes into component-level render commands. Shared row state and command helpers live in `shared.rs`, while list, tree, and table geometry stay in dedicated files so collection rendering can grow without turning `extract.rs` into a page-specific painter.

The row renderer suppresses generic owner text and owner image payloads for those components, then appends row surfaces, labels, disclosure icons, row adornments, tree guide lines, action icons, and table cell text through `UiPainterFamily::ListRow`, `TreeRow`, or `TableRow`. Shared row state now consumes retained component flags for selected, checked, and expanded before falling back to authored metadata, so a runtime foldout update can flip the `TreeRow` chevron in the native render extract without mutating `.zui` attributes. This gives the native renderer the same reusable list/tree/table grammar as the browser prototype's collection layer instead of treating rows as anonymous labels with incidental background colors.

## Popup Rows

`popup_rows.rs` owns the shared row visual vocabulary for popup surfaces: background, border, selected and hovered row fill, separator lines, selected edge marker, and compact popup text. Menu and dropdown renderers use this file rather than copying row colors and z-order math into each component family.

`popup_menu.rs` expands an open `ContextActionMenu` into additional runtime render commands. The owner node keeps its normal value/text command, then `menu_items` produces a popup background, row highlights, separators, selected markers, and row text with higher z order. This keeps menu visibility in the same `UiRenderExtract` stream used by the native editor screenshot path, instead of relying on a browser sample or a post-process overlay.

`popup_options.rs` applies the same row vocabulary to open `Dropdown`, `ComboBox`, and `Select` nodes. It reads `options`, `value`, `disabled_options`, `special_options`, `focused_options`, `hovered_options`, and `pressed_options` from `UiTemplateNodeMetadata`, then appends option-row commands below the control using the same minimum row height and 4 px popup gap as the native host popup layout. If the arranged clip frame is only the control frame, option rows deliberately render without that self-clip so a dropdown trigger does not clip its own popup.

## Dropdown Triggers

`dropdowns.rs` owns the low-level `Dropdown`, `ComboBox`, and `Select` trigger body. It suppresses the owner node's generic fallback text and emits component-level commands for the trigger surface, optional label, selected value text, chevron icon, and open-state edge marker. Selected value text can come from `value_text`, from the matching `options` label, or from the raw `value` display text. Open option rows remain in `popup_options.rs`, so the trigger chrome and transient popup rows can be validated and tuned independently.

## Feedback Controls

`feedback.rs` expands low-level `Alert`, `AlertTitle`, `Tooltip`, `Toast`, `Snackbar`, and `SnackbarContent` nodes into component-level render commands. The extractor suppresses the owner fallback text and image payload for those feedback components, then appends surface, title/body or message/action text, status/icon, and state-colored chrome through `UiPainterFamily::Alert`, `UiPainterFamily::Tooltip`, or `UiPainterFamily::Toast`. This keeps transient feedback visuals in the same `UiRenderExtract` stream as buttons, popups, fields, and rows instead of relying on retained-host-only drawing or a browser prototype.

Alert rendering reads severity/color/tone aliases, optional icon visibility, message text, title text, and action labels while routing disabled, pressed, focused, and hover state through `UiPainterFamily::Alert`. Tooltip rendering reads title/body/icon aliases plus focus, press, hover, disabled, and open state, while Toast/Snackbar rendering reads message/action/icon aliases and routes hover/focus/pressed/disabled state through the shared interactive painter priority. These families deliberately stay scoped to render extraction; popup timing, auto-hide, and input side effects remain owned by `surface/input` and the default interaction layer.

## Selection Controls

`selection_controls.rs` expands low-level `Checkbox`, `Radio`, `Toggle`, and `Switch` nodes into component-level render commands in the same `UiRenderExtract` stream. It reads the authored `checked`, `value`, `selected`, `disabled`, `hovered`, `focused`, and `pressed` props together with runtime `UiStateFlags`, then appends the checkbox mark/tick, radio mark/dot, toggle track/thumb, and inline label commands using the shared Workbench selection-control metrics. For these components, `extract.rs` suppresses the owner node's generic centered text so labels are emitted once, in the component lane after the mark or beside the toggle track.

## Tabs And Segmented Controls

`segmented_controls.rs` expands low-level `SegmentedControl`, `Segmented`, `Tab`, and `PanelTab` nodes into component-level render commands. Segmented controls suppress generic owner text and emit the optional group label, shared segmented body surface, segment dividers, selected segment surface, optional selected underline, and per-option labels through `UiPainterFamily::Tab`. Tabs use the same family/state lane but render as a lighter primitive: optional state background, selected underline, and a padded label.

This keeps tab strips, panel tabs, and segmented controls on the same bottom-up component path as the browser prototype. The runtime extractor does not need page-specific knowledge of drawer tabs or labs controls; it only consumes component metadata such as `options`, `value`, `selected`, `checked`, `selected_underline_height`, flat color overrides, and standard hover/focus/pressed/disabled state.

## Sliders

`sliders.rs` expands `RangeField`, `Slider`, and `RangeSlider` nodes into reusable runtime render commands for the web-prototype slider component family. It reads authored `value`, `min`, `max`, `value_percent`, `value_text`, `label`, `tick_count`, `steps`, `range_min_percent`, hover/focus/pressed/disabled state, and flat color aliases, then appends label text, track, fill span, optional ticks, thumb/halo, range-min value, and value-box commands. The owner node's generic text is suppressed for this family so the component layout, not a centered fallback label, controls the visible text lanes.

## Text Fields

`text_fields.rs` expands low-level `InputField`, `TextField`, `LineEdit`, `TextEdit`, and `NumberField` nodes into component-level render commands. It suppresses the owner node's generic fallback text, emits field chrome through the shared `UiPainterFamily::TextField` state model, then emits the value or placeholder text inside the field's padding lane. Focused enabled fields carry the existing `UiEditableTextState` into their text layout, so caret, selection, and composition decorations remain owned by the runtime text pipeline instead of a separate painter-only overlay. Unfocused placeholder text stays paint-only and does not expose caret or selection decorations.

## Acceptance Notes

`render_extract_accepts_flat_style_color_aliases` covers the workbench path directly: a button-like node with flat color aliases must emit a `Quad` command with matching `UiResolvedStyle` colors, border width, radius, and label text. This prevents future theme cleanup from silently making workbench controls invisible in the render extract.

`render_extract_carries_label_and_icon_atoms_through_generic_path` covers the low-level atom path: a `Label` must emit one generic text command with authored font/color/text-layout data and arranged frame/clip authority, while an `Icon` must emit one generic image command with its authored icon, color, opacity, and arranged frame. This locks the base workbench atoms without adding per-atom renderer modules.

The first focused atom Cargo command used a cold `D:\cargo-targets\zircon-editor-workbench-atoms` target directory and timed out before producing a test binary. Re-running the same filtered test against the already warmed `D:\cargo-targets\zircon-editor-workbench-feedback` target directory passed: 1 passed, 0 failed, 2623 filtered out, with existing `zircon_runtime` warning noise.

`render_extract_expands_open_context_action_menu_items` covers the Workbench toolbar-menu path: an open `ContextActionMenu` with `menu_items` must emit additional text commands such as `Simulate` and `Network Preview`, plus a higher z-order popup background command over the menu frame.

`render_extract_expands_open_dropdown_options` covers the shared Workbench dropdown path: an open `Dropdown` with authored `options` must emit visible option text commands such as `Surface`, `Post Process`, and `Volume`, plus a higher z-order popup background command below the trigger even when the trigger's own clip frame matches its control frame.

`render_extract_expands_dropdown_trigger_primitives` covers the low-level dropdown trigger path: an open `Select` with authored label, value, and options must emit the trigger surface, label, selected option label, open chevron, and open-state edge marker through `UiRenderExtract`, while still allowing `popup_options.rs` to render the matching option rows below it.

`render_extract_dropdown_uses_shared_metadata_painter_state_priority` covers the shared metadata-state adapter for dropdown triggers: a loading open dropdown must resolve to `UiPainterResolvedState::Loading` even when pressed, focused, and hovered are also present, and an `active_drag_target` select must resolve to `DropHovered` with focus-ring chrome. This prevents individual render modules from silently omitting state aliases that the retained painter selector already understands.

Validation for the shared metadata-state adapter passed after the cold Windows test target finished compiling: `cargo check -p zircon_runtime --lib` completed with existing warning noise only, and the focused `render_extract_dropdown_uses_shared_metadata_painter_state_priority` test ran 1 test, 0 failed, 2782 filtered out. The first two focused test attempts timed out while compiling dependencies before reaching the test binary.

`render_extract_uses_component_state_store_for_shared_painter_priority` covers the retained-state render path: retained hover, focus, popup-open, selected, active-drag-target, and loading flags in `UiSurfaceComponentStateStore` must resolve Button, TextField, Dropdown, Checkbox, Slider, and loading Button commands to the matching shared painter states even when authored metadata stays neutral. This follows Slate's paint split: `SWidget::Paint(...)` / `OnPaint(...)` receives widget state and appends draw elements, while Zircon first normalizes retained state into `UiPainterState` and then lets each family emit `UiRenderCommand` primitives. Earlier validation passed on the warmed `D:\cargo-targets\zircon-block-box-0605` target lane: `cargo check -p zircon_runtime --lib` finished with existing warning noise only, and the focused test ran 1 test, 0 failed, 2810 filtered out. The expanded 2026-06-06 selector-family assertions passed on the warmed `D:\cargo-targets\zircon-keyboard-clipboard-extract-0605` target lane: scoped rustfmt, `git diff --check`, and trailing-whitespace scan passed; `cargo check -p zircon_runtime --lib` finished with existing warning noise only; and the focused test ran 1 test, 0 failed, 2870 filtered out.

`runtime_drag_drop_component_state` is the producer/consumer coverage for retained drag/drop state. It exercises `UiDispatchEffect::DragDrop` reducing source `dragging` and target `drop_hovered`/`active_drag_target` into `UiSurfaceComponentStateStore`, verifies v2 `:dragging`, `:drop_hovered`, and `:active_drag_target` runtime style projection, rebuilds the render extract, and asserts generic interactive commands resolve to `UiPainterResolvedState::Dragging`, `DropHovered`, and then `Normal` after completion. Earlier input-side coverage type-checked on the first `cargo check -p zircon_runtime --lib` run before unrelated active ECS/query and WGPU post-process drift landed. Runtime execution evidence for the v2 style projection is not accepted yet: two focused Cargo attempts on `D:\cargo-targets\zircon-drag-component-state-style-0605` timed out during compilation without Rust diagnostics, no `zircon_runtime-*.exe` test binary was produced, and no same-target build process remained afterward.

`runtime_loading_component_state` is the semantic producer/consumer coverage for retained loading state: it toggles `loading` through `UiSurface::mutate_property(...)`, verifies v2 `:loading` runtime style projection, rebuilds the render extract, and asserts the Button command resolves to `UiPainterResolvedState::Loading` before returning to normal after the flag clears.

`render_extract_expands_selection_control_indicators` covers the low-level selection-control path: checked checkbox, radio, and toggle nodes must emit visible mark/tick, dot, track/thumb, and label commands through the runtime render extract instead of relying only on retained-host painter special cases.

`render_extract_expands_tabs_and_segmented_control_primitives` covers the low-level tabs/segmented path: a `SegmentedControl` with a label, three options, selected middle segment, and underline must emit group label, body surface, selected segment, underline, and one selected option label, while a selected `Tab` must emit its underline and one padded tab label through the shared `Tab` painter family.

The focused tabs/segmented `cargo test` command timed out twice while compiling/linking the `zircon_runtime` test binary under shared desktop build load. Neither run emitted Rust diagnostics or produced a `zircon_runtime-*.exe` test binary that could be run directly. The lower-cost validation gates did complete: `cargo check -p zircon_runtime --lib --locked` type-checked the production render path, and `cargo check -p zircon_runtime --tests --locked` type-checked the test module that registers `render_extract_expands_tabs_and_segmented_control_primitives`. Both checks finished with only pre-existing warning noise.

`render_extract_expands_slider_primitives` covers the low-level slider path: a `RangeField` node with label, value, tick count, and declared colors must emit label, track, fill, five ticks, thumb, value-box, and single-label commands through `UiRenderExtract`. The first attempt exposed an unrelated lower-layer compile blocker in `zircon_runtime/src/core/framework/tests.rs` where a non-finite render-scale regression used the `Real` time marker as `Real::NAN`; that test now uses `f32::NAN`, which is the actual `RenderDynamicResolutionSettings::fixed_scale(...)` input type.

`render_extract_expands_text_field_primitives` covers the low-level text-field path: a focused `InputField` with a custom value property must emit field chrome, exactly one component text command, focused `TextField` painter state, padded text geometry, and editable text layout with the current caret and selection range. `render_extract_expands_text_field_placeholder_without_unfocused_caret` covers the companion placeholder path: an empty unfocused `TextField` must render placeholder text in the placeholder color without attaching editable caret or selection state to the paint layout.

The focused text-field Cargo command exceeded the desktop timeout while linking the `zircon_runtime` test binary under shared build load, but it produced `D:\cargo-targets\zircon-editor-workbench-text-fields\debug\deps\zircon_runtime-b34ee8d8fc52f1fd.exe`. Running that binary directly with the `render_extract_expands_text_field` filter passed both text-field render-extract tests: 2 passed, 0 failed, 2553 filtered out.

`render_extract_expands_button_primitives` and `render_extract_expands_icon_button_state_surface` cover the low-level button path: a primary `Button` with a leading icon must emit one component surface command, one icon command, one text command, and no duplicate owner fallback payload; a selected `IconButton` must emit selected chrome and a centered icon while keeping the accessibility label paint-silent. The current focused button check also guards the shared icon-and-text layout helper against consuming the optional icon while computing text width, because the same icon value is then needed to emit the image command.

`render_extract_expands_collection_row_primitives` covers the low-level collection-row path: selected list rows must emit selected chrome and exactly one row label, selected tree rows must emit disclosure/object/action icons without duplicating an authored icon through owner fallback image output, retained runtime `expanded` state must drive the `TreeRow` disclosure chevron even when authored metadata is false, and pressed table rows must emit table-row surface and text commands with the shared `TableRow` painter family. The focused Cargo command exceeded the desktop timeout while linking the `zircon_runtime` test binary under shared build load, but it produced `D:\cargo-targets\zircon-editor-workbench-collection-rows\debug\deps\zircon_runtime-b34ee8d8fc52f1fd.exe`. Running that binary directly with the `render_extract_expands_collection_row_primitives` filter passed: 1 passed, 0 failed, 2563 filtered out. The 2026-06-06 runtime-expanded follow-up has scoped rustfmt evidence and is expected to reuse the same focused test filter during validation.

`render_extract_expands_feedback_primitives` covers the low-level feedback path: a focused `Tooltip` must emit tooltip surface, title/body text, and info icon commands with the shared `Tooltip` painter family and no duplicate owner title; a hovered `Toast` must emit toast surface, status icon, message text, and action text commands with the shared `Toast` painter family; `Alert` and `AlertTitle` must emit severity-colored chrome/text through `UiPainterFamily::Alert`; and `Snackbar` plus `SnackbarContent` aliases must emit Toast-family surface, icon, message, action, and state-priority commands instead of relying on generic owner text. The focused Cargo command first timed out before producing a test binary, then timed out again while the cargo/rustc lane continued compiling under shared desktop build load. After that lane finished, the produced test binary passed the feedback filter directly: 1 passed, 0 failed, and 2621 filtered out.

The native editor screenshot path was refreshed separately from the focused runtime Cargo lane by running the already-built `zircon_editor` test binary for `componentized_workbench_module_dropdown_open_paints_native_preview_pixels`. It wrote `target/editor-workbench-visual-check/editor-workbench-native-module-dropdown-open-popup-options-1672x941.png` and the side-by-side comparison `target/editor-workbench-visual-check/editor-workbench-ai-material-vs-native-module-dropdown-open-popup-options-1672x941.png`.

The selection-control follow-up refreshed the full native workbench screenshot through `componentized_workbench_surface_paints_native_preview_pixels_and_interaction_state` with `ZIRCON_WRITE_WORKBENCH_PREVIEW=1`. It wrote `target/editor-workbench-visual-check/editor-workbench-native-selection-controls-1672x941.png`, and the direct reference-to-native window comparison is `target/editor-workbench-visual-check/editor-workbench-reference-vs-native-selection-controls-1672x941.png`.
