---
related_code:
  - zircon_runtime/src/core/framework/render/backend_types.rs
  - zircon_runtime/src/core/diagnostics/collect.rs
  - zircon_runtime/src/ui/surface/render/resolve.rs
  - zircon_runtime/src/ui/surface/render/node_visual_data.rs
  - zircon_runtime/src/ui/surface/render/extract.rs
  - zircon_runtime/src/ui/surface/render/dropdowns.rs
  - zircon_runtime/src/ui/surface/render/popup_menu.rs
  - zircon_runtime/src/ui/surface/render/popup_options.rs
  - zircon_runtime/src/ui/surface/render/popup_rows.rs
  - zircon_runtime/src/ui/surface/render/selection_controls.rs
  - zircon_runtime/src/ui/surface/render/sliders.rs
  - zircon_runtime/src/ui/surface/render/text_fields.rs
  - zircon_runtime/src/ui/style.rs
  - zircon_runtime/src/ui/tests/shared_core.rs
  - zircon_runtime/src/ui/tests/render_popup_menu.rs
  - zircon_runtime/src/ui/tests/render_popup_options.rs
  - zircon_runtime/src/ui/tests/render_dropdowns.rs
  - zircon_runtime/src/ui/tests/render_selection_controls.rs
  - zircon_runtime/src/ui/tests/render_sliders.rs
  - zircon_runtime/src/ui/tests/render_text_fields.rs
  - zircon_runtime/src/core/framework/tests.rs
  - zircon_runtime_interface/src/ui/tree/node/template_node_metadata.rs
  - zircon_runtime_interface/src/ui/surface/render/resolved_style.rs
implementation_files:
  - zircon_runtime/src/core/framework/render/backend_types.rs
  - zircon_runtime/src/core/diagnostics/collect.rs
  - zircon_runtime/src/ui/surface/render/resolve.rs
  - zircon_runtime/src/ui/surface/render/node_visual_data.rs
  - zircon_runtime/src/ui/surface/render/extract.rs
  - zircon_runtime/src/ui/surface/render/dropdowns.rs
  - zircon_runtime/src/ui/surface/render/popup_menu.rs
  - zircon_runtime/src/ui/surface/render/popup_options.rs
  - zircon_runtime/src/ui/surface/render/popup_rows.rs
  - zircon_runtime/src/ui/surface/render/selection_controls.rs
  - zircon_runtime/src/ui/surface/render/sliders.rs
  - zircon_runtime/src/ui/surface/render/text_fields.rs
  - zircon_runtime/src/ui/tests/shared_core.rs
  - zircon_runtime/src/ui/tests/render_popup_menu.rs
  - zircon_runtime/src/ui/tests/render_popup_options.rs
  - zircon_runtime/src/ui/tests/render_dropdowns.rs
  - zircon_runtime/src/ui/tests/render_selection_controls.rs
  - zircon_runtime/src/ui/tests/render_sliders.rs
  - zircon_runtime/src/ui/tests/render_text_fields.rs
  - zircon_runtime/src/core/framework/tests.rs
plan_sources:
  - user: 2026-06-01 workbench design recreation and engine implementation
  - user: 2026-06-03 native editor workbench window comparison screenshot request
tests:
  - zircon_runtime/src/tests/runtime_diagnostics/mod.rs::runtime_diagnostics_combines_core_render_contract_and_missing_externalized_plugins
  - rustfmt --edition 2021 --check zircon_runtime/src/ui/surface/render/resolve.rs zircon_runtime/src/ui/tests/shared_core.rs
  - rustfmt --edition 2021 --check zircon_runtime/src/ui/surface/render/extract.rs zircon_runtime/src/ui/surface/render/mod.rs zircon_runtime/src/ui/surface/render/selection_controls.rs zircon_runtime/src/ui/tests/render_selection_controls.rs zircon_runtime/src/ui/tests/mod.rs
  - cargo test -p zircon_runtime --lib render_extract_expands_open_context_action_menu_items --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-workbench-command-feedback -- --nocapture
  - pending focused Cargo rerun: cargo test -p zircon_runtime --lib render_extract_expands_open_dropdown_options --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-workbench-popup-options -- --nocapture
  - pending focused Cargo rerun: cargo test -p zircon_runtime --lib render_extract_expands_dropdown_trigger_primitives --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-workbench-dropdowns --color never -- --nocapture
  - cargo test -p zircon_runtime --lib render_extract_expands_selection_control_indicators --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-workbench-selection-controls --color never -- --nocapture
  - cargo test -p zircon_runtime --lib render_extract_expands_slider_primitives --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-workbench-sliders --color never -- --nocapture
  - pending focused Cargo run: cargo test -p zircon_runtime --lib render_extract_expands_text_field_primitives --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-workbench-text-fields --message-format short --color never -- --nocapture --test-threads=1
doc_type: module-detail
---

# Runtime UI Surface Render

`surface/render` converts arranged `UiTree` nodes into the neutral `UiRenderExtract` consumed by the runtime WGPU UI pass and editor host painters. The extract is authoritative for visible frame, clip frame, z index, style, text, image, opacity, and editable text decorations. It does not invent a second widget state model; it reads the node's `UiTemplateNodeMetadata` plus arranged geometry.

Runtime render submit exposes UI composition scale through `RenderStats.last_ui_*`: command, quad, text payload, image payload, clipped command, and graph-executed pass counts. `collect_runtime_diagnostics(...)` mirrors those values into `DiagnosticStore` under `render.ui.*` paths so tooling can observe UI payload and graph placement scale without reading the retained graph or the concrete WGPU UI pass.

## Style Resolution

`resolve.rs` keeps render extraction aligned with the shared style resolver aliases. Color fields now accept both structured and flat authoring forms:

- background: `background = { color = "#..." }` or `background_color = "#..."`
- foreground: `foreground = { color = "#..." }`, `foreground_color = "#..."`, `fg = "#..."`, or `color = "#..."`
- border: `border = { color = "#...", width = 1.0, radius = 6.0 }`, `border_color = "#..."`, or `outline = "#..."`

This matters for editor workbench assets because the component theme uses flat `background_color`, `foreground_color`, and `border_color` rules for compactness, while the renderer still needs the same values in `UiResolvedStyle` before choosing `Quad`, `Text`, `Image`, or `Group` command kinds.

## Popup Rows

`popup_rows.rs` owns the shared row visual vocabulary for popup surfaces: background, border, selected and hovered row fill, separator lines, selected edge marker, and compact popup text. Menu and dropdown renderers use this file rather than copying row colors and z-order math into each component family.

`popup_menu.rs` expands an open `ContextActionMenu` into additional runtime render commands. The owner node keeps its normal value/text command, then `menu_items` produces a popup background, row highlights, separators, selected markers, and row text with higher z order. This keeps menu visibility in the same `UiRenderExtract` stream used by the native editor screenshot path, instead of relying on a browser sample or a post-process overlay.

`popup_options.rs` applies the same row vocabulary to open `Dropdown`, `ComboBox`, and `Select` nodes. It reads `options`, `value`, `disabled_options`, `special_options`, `focused_options`, `hovered_options`, and `pressed_options` from `UiTemplateNodeMetadata`, then appends option-row commands below the control using the same minimum row height and 4 px popup gap as the native host popup layout. If the arranged clip frame is only the control frame, option rows deliberately render without that self-clip so a dropdown trigger does not clip its own popup.

## Dropdown Triggers

`dropdowns.rs` owns the low-level `Dropdown`, `ComboBox`, and `Select` trigger body. It suppresses the owner node's generic fallback text and emits component-level commands for the trigger surface, optional label, selected value text, chevron icon, and open-state edge marker. Selected value text can come from `value_text`, from the matching `options` label, or from the raw `value` display text. Open option rows remain in `popup_options.rs`, so the trigger chrome and transient popup rows can be validated and tuned independently.

## Selection Controls

`selection_controls.rs` expands low-level `Checkbox`, `Radio`, `Toggle`, and `Switch` nodes into component-level render commands in the same `UiRenderExtract` stream. It reads the authored `checked`, `value`, `selected`, `disabled`, `hovered`, `focused`, and `pressed` props together with runtime `UiStateFlags`, then appends the checkbox mark/tick, radio mark/dot, toggle track/thumb, and inline label commands using the shared Workbench selection-control metrics. For these components, `extract.rs` suppresses the owner node's generic centered text so labels are emitted once, in the component lane after the mark or beside the toggle track.

## Sliders

`sliders.rs` expands `RangeField`, `Slider`, and `RangeSlider` nodes into reusable runtime render commands for the web-prototype slider component family. It reads authored `value`, `min`, `max`, `value_percent`, `value_text`, `label`, `tick_count`, `steps`, `range_min_percent`, hover/focus/pressed/disabled state, and flat color aliases, then appends label text, track, fill span, optional ticks, thumb/halo, range-min value, and value-box commands. The owner node's generic text is suppressed for this family so the component layout, not a centered fallback label, controls the visible text lanes.

## Text Fields

`text_fields.rs` expands low-level `InputField`, `TextField`, `LineEdit`, `TextEdit`, and `NumberField` nodes into component-level render commands. It suppresses the owner node's generic fallback text, emits field chrome through the shared `UiPainterFamily::TextField` state model, then emits the value or placeholder text inside the field's padding lane. Focused enabled fields carry the existing `UiEditableTextState` into their text layout, so caret, selection, and composition decorations remain owned by the runtime text pipeline instead of a separate painter-only overlay. Unfocused placeholder text stays paint-only and does not expose caret or selection decorations.

## Acceptance Notes

`render_extract_accepts_flat_style_color_aliases` covers the workbench path directly: a button-like node with flat color aliases must emit a `Quad` command with matching `UiResolvedStyle` colors, border width, radius, and label text. This prevents future theme cleanup from silently making workbench controls invisible in the render extract.

`render_extract_expands_open_context_action_menu_items` covers the Workbench toolbar-menu path: an open `ContextActionMenu` with `menu_items` must emit additional text commands such as `Simulate` and `Network Preview`, plus a higher z-order popup background command over the menu frame.

`render_extract_expands_open_dropdown_options` covers the shared Workbench dropdown path: an open `Dropdown` with authored `options` must emit visible option text commands such as `Surface`, `Post Process`, and `Volume`, plus a higher z-order popup background command below the trigger even when the trigger's own clip frame matches its control frame.

`render_extract_expands_dropdown_trigger_primitives` covers the low-level dropdown trigger path: an open `Select` with authored label, value, and options must emit the trigger surface, label, selected option label, open chevron, and open-state edge marker through `UiRenderExtract`, while still allowing `popup_options.rs` to render the matching option rows below it.

`render_extract_expands_selection_control_indicators` covers the low-level selection-control path: checked checkbox, radio, and toggle nodes must emit visible mark/tick, dot, track/thumb, and label commands through the runtime render extract instead of relying only on retained-host painter special cases.

`render_extract_expands_slider_primitives` covers the low-level slider path: a `RangeField` node with label, value, tick count, and declared colors must emit label, track, fill, five ticks, thumb, value-box, and single-label commands through `UiRenderExtract`. The first attempt exposed an unrelated lower-layer compile blocker in `zircon_runtime/src/core/framework/tests.rs` where a non-finite render-scale regression used the `Real` time marker as `Real::NAN`; that test now uses `f32::NAN`, which is the actual `RenderDynamicResolutionSettings::fixed_scale(...)` input type.

`render_extract_expands_text_field_primitives` covers the low-level text-field path: a focused `InputField` with a custom value property must emit field chrome, exactly one component text command, focused `TextField` painter state, padded text geometry, and editable text layout with the current caret and selection range. `render_extract_expands_text_field_placeholder_without_unfocused_caret` covers the companion placeholder path: an empty unfocused `TextField` must render placeholder text in the placeholder color without attaching editable caret or selection state to the paint layout.

The native editor screenshot path was refreshed separately from the focused runtime Cargo lane by running the already-built `zircon_editor` test binary for `componentized_workbench_module_dropdown_open_paints_native_preview_pixels`. It wrote `target/editor-workbench-visual-check/editor-workbench-native-module-dropdown-open-popup-options-1672x941.png` and the side-by-side comparison `target/editor-workbench-visual-check/editor-workbench-ai-material-vs-native-module-dropdown-open-popup-options-1672x941.png`.

The selection-control follow-up refreshed the full native workbench screenshot through `componentized_workbench_surface_paints_native_preview_pixels_and_interaction_state` with `ZIRCON_WRITE_WORKBENCH_PREVIEW=1`. It wrote `target/editor-workbench-visual-check/editor-workbench-native-selection-controls-1672x941.png`, and the direct reference-to-native window comparison is `target/editor-workbench-visual-check/editor-workbench-reference-vs-native-selection-controls-1672x941.png`.
