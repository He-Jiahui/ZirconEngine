# Material Layout Foundation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make `.ui.toml` Material meta components produce Slint-inspired intrinsic layout through the shared Runtime UI layout pass and the existing editor host projection path.

**Architecture:** Keep generated Slint UI out of the editor path. Translate the layout-relevant parts of `dev/slint/ui-libraries/material` into `.ui.toml` tokens/attributes, resolve those attributes in `zircon_runtime::ui::layout`, and let `UiSurfaceFrame.arranged_tree` remain the only source of frames for render and hit testing. Editor host projection and the native painter consume the arranged frames plus already-projected visual metadata; they must not recompute Material layout.

**Tech Stack:** Rust, TOML `.ui.toml` assets, existing Runtime UI template metadata, `UiSurfaceFrame` arranged geometry, Slint Material reference files under `dev/slint/ui-libraries/material`, Markdown docs.

---

## Execution Policy

- Work directly on `main` in the existing checkout.
- Do not create worktrees or feature branches.
- Do not commit unless the user explicitly asks for a commit.
- Preserve unrelated dirty work. Do not revert active changes from other sessions.
- During implementation slices, add production code, focused test code, comments, and docs without forcing per-slice build/test loops.
- Run compile/unit-test commands only in each milestone testing stage unless a blocker requires earlier evidence.
- Avoid the active Asset Browser/media task paths unless a shared lower-layer Material layout contract requires the same file:
  - Avoid broad edits to `zircon_editor/assets/ui/editor/asset_browser.ui.toml`.
  - Avoid SVG/image painter changes unless this layout plan exposes a compile blocker in already-touched shared projection code.
  - Coordinate before changing `zircon_editor/assets/ui/theme/editor_material.ui.toml` beyond layout tokens/selectors because `.codex/sessions/20260505-2334-asset-browser-material-svg-fps.md` owns Material/Asset Browser visual styling while active.
- Avoid text/input-owned native painter behavior changes unless the layout contract exposes a lower-layer measurement issue. The active note is `.codex/sessions/20260505-1106-editor-native-text-input-regression.md`.

## Design Source

- User-provided plan: `Material UI + .ui.toml 全链路 UI 系统推进计划.md`.
- User clarification: prioritize shared Material foundation, especially layout content.
- Existing plan: `.codex/plans/Shared Slate-Style UI Layout, Render, And Hit Framework.md`.
- Existing docs: `docs/ui-and-layout/runtime-ui-component-showcase.md`, `docs/ui-and-layout/shared-ui-core-foundation.md`, `docs/ui-and-layout/slate-style-ui-surface-frame.md`.
- Reference files:
  - `dev/slint/ui-libraries/material/src/material.slint`
  - `dev/slint/ui-libraries/material/src/ui/styling/material_style_metrics.slint`
  - `dev/slint/ui-libraries/material/src/ui/styling/material_palette.slint`
  - `dev/slint/ui-libraries/material/src/ui/components/base_button.slint`
  - `dev/slint/ui-libraries/material/src/ui/components/filled_button.slint`
  - `dev/slint/ui-libraries/material/src/ui/components/outline_button.slint`
  - `dev/slint/ui-libraries/material/src/ui/components/text_field.slint`
  - `dev/slint/ui-libraries/material/src/ui/components/state_layer.slint`

## Current Repository Baseline

- `zircon_editor/assets/ui/editor/material_meta_components.ui.toml` already defines Material meta components and maps them to existing runtime roles such as `Button`, `IconButton`, `Checkbox`, `InputField`, `RangeField`, `ComboBox`, `ListRow`, `ContextActionMenu`, and `TextField`.
- `zircon_editor/assets/ui/theme/editor_material.ui.toml` already owns visual Material-ish colors/selectors. This plan should only add layout-adjacent tokens/selectors when needed.
- `UiTemplateNodeMetadata` already carries `attributes`, `style_tokens`, `classes`, `style_overrides`, and `bindings` from `.ui.toml` into runtime layout/render.
- `zircon_runtime/src/ui/layout/pass/measure.rs` currently hard-codes button padding as `18.0` horizontal and `8.0` vertical for `Button` and `IconButton` leaves.
- `zircon_runtime/src/ui/surface/render/resolve.rs` already resolves numeric visual attributes such as `font_size`, `border_width`, and `corner_radius` from metadata.
- `zircon_editor/src/ui/slint_host/ui/pane_data_conversion/pane_component_projection/mod.rs` already projects Material visual fields into `TemplatePaneNodeData`: `surface_variant`, `text_tone`, `button_variant`, `font_size`, `font_weight`, `text_align`, `overflow`, `corner_radius`, and `border_width`.
- `zircon_editor/src/ui/slint_host/host_contract/painter/template_nodes.rs` paints from `TemplatePaneNodeData.frame`; it must continue to trust arranged frames rather than deriving local Material coordinates.

## Affected Files

### Runtime Layout Foundation

- Modify `zircon_runtime/src/ui/layout/pass/measure.rs` for Material-aware leaf measurement.
- Create `zircon_runtime/src/ui/layout/pass/material.rs` if `measure.rs` would otherwise accumulate parsing and token fallback helpers.
- Modify `zircon_runtime/src/ui/layout/pass/mod.rs` only to wire `mod material;` if `material.rs` is created.
- Modify `zircon_runtime/src/ui/tests/shared_core.rs` for shared layout regressions unless test growth becomes too large.
- If new tests are more than a small cluster, create `zircon_runtime/src/ui/tests/material_layout.rs` and modify `zircon_runtime/src/ui/tests/mod.rs` to add `mod material_layout;`.

### Material Meta Component Assets

- Modify `zircon_editor/assets/ui/editor/material_meta_components.ui.toml` for layout tokens and attributes on Material meta component roots.
- Modify `zircon_editor/src/tests/host/template_runtime/pane_body_documents.rs` for projection/layout-attribute coverage of Material meta components.

### Editor Projection And Native Host Contract

- Modify `zircon_editor/src/ui/slint_host/ui/pane_data_conversion/pane_component_projection/mod.rs` only if new layout attributes must be visible in `TemplatePaneNodeData` for painter/test diagnostics.
- Do not add host-side frame computation. Any native host test must assert frames generated by `UiSurfaceFrame.arranged_tree` or the existing host projection with shared surface data.
- Do not modify `zircon_editor/src/ui/slint_host/host_contract/painter/template_nodes.rs` unless a field already projected there needs a minimal layout-related visual guard.

### Documentation And Coordination

- Update `docs/ui-and-layout/runtime-ui-component-showcase.md` with Material layout foundation behavior, reference files, tests, and validation evidence.
- Update `docs/ui-and-layout/shared-ui-core-foundation.md` with the shared layout measurement contract if runtime measurement helpers are added.
- Update or create `tests/acceptance/material-layout-foundation.md` with focused acceptance evidence.
- Update `.codex/sessions/20260505-1502-editor-ui-layout-regression.md` or create a new active session note if implementation continues beyond this planning handoff.

## Material Layout Contract

Use these attribute names consistently. They are intentionally generic and belong to template metadata, not editor-only DTOs:

```text
layout_padding_left
layout_padding_right
layout_padding_top
layout_padding_bottom
layout_spacing
layout_min_width
layout_min_height
layout_icon_size
layout_leading_slot_width
layout_trailing_slot_width
```

The first implementation should support these component families:

```text
Button
IconButton
ToggleButton
Checkbox
InputField
TextField
ListRow
ComboBox
RangeField
NumberField
Switch
```

Do not add animation, floating-label movement, ripple geometry, or popup placement in this plan. The goal is deterministic intrinsic size and arranged-frame stability.

## Milestone 1: Runtime Material Leaf Measurement

- Goal: Runtime layout reads Material layout attributes from template metadata and derives deterministic intrinsic sizes for Material leaf controls.
- In-scope behaviors: numeric layout attribute resolution, fallback defaults inspired by Slint Material metrics, text intrinsic size plus padding, icon-only minimums, list row/checkbox/switch/input min-height behavior, and explicit authored constraints remaining authoritative.
- Dependencies: existing `UiTemplateNodeMetadata`, `measure_text(...)`, `measure_node(...)`, `DesiredSize`, and existing shared layout tests.

### Implementation Slices

- [ ] Add a small internal resolver for numeric Material layout attributes. If this would push `measure.rs` into mixed parsing/layout responsibilities, create `zircon_runtime/src/ui/layout/pass/material.rs` with this shape:

```rust
use toml::Value;
use zircon_runtime_interface::ui::{
    layout::UiSize,
    tree::UiTemplateNodeMetadata,
};

pub(super) struct MaterialLayoutMetrics {
    pub padding_left: f32,
    pub padding_right: f32,
    pub padding_top: f32,
    pub padding_bottom: f32,
    pub spacing: f32,
    pub min_width: f32,
    pub min_height: f32,
    pub icon_size: f32,
    pub leading_slot_width: f32,
    pub trailing_slot_width: f32,
}

impl MaterialLayoutMetrics {
    pub(super) fn resolve(metadata: &UiTemplateNodeMetadata) -> Self {
        Self {
            padding_left: number_attr(metadata, "layout_padding_left").unwrap_or(0.0).max(0.0),
            padding_right: number_attr(metadata, "layout_padding_right").unwrap_or(0.0).max(0.0),
            padding_top: number_attr(metadata, "layout_padding_top").unwrap_or(0.0).max(0.0),
            padding_bottom: number_attr(metadata, "layout_padding_bottom").unwrap_or(0.0).max(0.0),
            spacing: number_attr(metadata, "layout_spacing").unwrap_or(0.0).max(0.0),
            min_width: number_attr(metadata, "layout_min_width").unwrap_or(0.0).max(0.0),
            min_height: number_attr(metadata, "layout_min_height").unwrap_or(0.0).max(0.0),
            icon_size: number_attr(metadata, "layout_icon_size").unwrap_or(0.0).max(0.0),
            leading_slot_width: number_attr(metadata, "layout_leading_slot_width").unwrap_or(0.0).max(0.0),
            trailing_slot_width: number_attr(metadata, "layout_trailing_slot_width").unwrap_or(0.0).max(0.0),
        }
    }

    pub(super) fn apply_to_content(&self, content: UiSize, has_icon: bool) -> UiSize {
        let icon_width = if has_icon { self.icon_size } else { 0.0 };
        let text_spacing = if has_icon && content.width > 0.0 { self.spacing } else { 0.0 };
        UiSize::new(
            (content.width
                + icon_width
                + text_spacing
                + self.leading_slot_width
                + self.trailing_slot_width
                + self.padding_left
                + self.padding_right)
                .max(self.min_width),
            content
                .height
                .max(if has_icon { self.icon_size } else { 0.0 })
                .max(self.min_height)
                + self.padding_top
                + self.padding_bottom,
        )
    }
}

fn number_attr(metadata: &UiTemplateNodeMetadata, key: &str) -> Option<f32> {
    metadata.attributes.get(key).and_then(value_as_f32)
}

fn value_as_f32(value: &Value) -> Option<f32> {
    value
        .as_float()
        .or_else(|| value.as_integer().map(|value| value as f64))
        .map(|value| value as f32)
}
```

- [ ] In `measure.rs`, replace `BUTTON_HORIZONTAL_PADDING` / `BUTTON_VERTICAL_PADDING` hard-coded sizing with a helper that first checks `UiTemplateNodeMetadata` layout attributes.
- [ ] Preserve current behavior for non-Material controls by keeping a default fallback equivalent to existing button behavior when no layout attributes exist.
- [ ] Treat `IconButton` and icon-bearing buttons as text size plus optional icon size plus spacing. A node has an icon if metadata has non-empty `icon`, `image`, `media`, or `source` attributes.
- [ ] Apply `layout_min_width` and `layout_min_height` after content-plus-padding calculation.
- [ ] Add tests covering:
  - Material button text width plus Slint-inspired horizontal/vertical padding.
  - Material button with icon adds icon size and spacing.
  - Material icon button with no text still gets min width/min height.
  - Authored fixed constraints still override the content-driven desired size through existing `desired_axis(...)` behavior.
  - List row/input/switch controls keep Material min height when text is short or absent.

### Testing Stage: Runtime Layout Gate

Run targeted formatting for changed runtime layout/test files:

```powershell
rustfmt --edition 2021 --check "zircon_runtime/src/ui/layout/pass/measure.rs" "zircon_runtime/src/ui/layout/pass/material.rs" "zircon_runtime/src/ui/layout/pass/mod.rs" "zircon_runtime/src/ui/tests/shared_core.rs" "zircon_runtime/src/ui/tests/material_layout.rs" "zircon_runtime/src/ui/tests/mod.rs"
```

If `material.rs` or `material_layout.rs` is not created, omit it from the command rather than creating an empty file.

Run focused runtime layout tests:

```powershell
$env:TMP="E:\tmp\cargo-tmp"; $env:TEMP="E:\tmp\cargo-tmp"; cargo test -p zircon_runtime --lib material_layout --locked --jobs 1 --target-dir "E:\zircon-build\targets\material-layout-foundation" --message-format short --color never -- --nocapture
```

Run scoped runtime type check:

```powershell
$env:TMP="E:\tmp\cargo-tmp"; $env:TEMP="E:\tmp\cargo-tmp"; cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir "E:\zircon-build\targets\material-layout-foundation" --message-format short --color never
```

### Debug / Correction Loop

- If Material tests fail because desired size is too small, inspect `UiTemplateNodeMetadata.attributes` on the test node before changing layout math.
- If non-Material button tests regress, restore the previous fallback sizing in the no-layout-attributes path.
- If authored constraints are ignored, fix the content-size-to-`desired_axis(...)` flow instead of adding component-specific exceptions.

### Exit Evidence

- Focused runtime Material layout tests pass.
- `cargo check -p zircon_runtime --lib` passes or an unrelated active-session blocker is recorded with the exact diagnostic.
- No editor/painter code is needed for this milestone.

## Milestone 2: `.ui.toml` Material Metrics And Projection Coverage

- Goal: Material meta components declare Slint-inspired layout metrics in `.ui.toml`, and template runtime projection preserves those metrics into the shared runtime tree.
- In-scope behaviors: `.ui.toml` tokens/attributes for button, icon button, toggle, checkbox, input/text fields, list rows, combo/range/number/switch controls; projection tests that prove the attributes reach runtime/host projection without new Rust coordinate tables.
- Dependencies: Milestone 1 runtime measurement can consume layout attributes.

### Implementation Slices

- [ ] Update `zircon_editor/assets/ui/editor/material_meta_components.ui.toml` tokens. Use Slint Material metrics as the source for defaults:

```toml
[tokens]
material_control_height = 40.0
material_compact_control_height = 32.0
material_icon_button_size = 40.0
material_button_min_width = 40.0
material_button_padding_x = 24.0
material_button_padding_y = 10.0
material_button_icon_size = 18.0
material_button_spacing = 8.0
material_field_min_height = 56.0
material_field_padding_x = 16.0
material_field_padding_y = 4.0
material_list_item_height = 40.0
material_list_item_padding_x = 16.0
material_list_item_spacing = 8.0
```

- [ ] Add these layout props to `MaterialButtonBase.root`, `MaterialButton.root`, and button variants such as `MaterialTextButton`:

```toml
layout_padding_left = "$material_button_padding_x"
layout_padding_right = "$material_button_padding_x"
layout_padding_top = "$material_button_padding_y"
layout_padding_bottom = "$material_button_padding_y"
layout_spacing = "$material_button_spacing"
layout_min_width = "$material_button_min_width"
layout_min_height = "$material_control_height"
layout_icon_size = "$material_button_icon_size"
```

- [ ] Add icon-button layout props to `MaterialIconButton.root`:

```toml
layout_min_width = "$material_icon_button_size"
layout_min_height = "$material_icon_button_size"
layout_icon_size = "$material_button_icon_size"
layout_padding_left = 0.0
layout_padding_right = 0.0
layout_padding_top = 0.0
layout_padding_bottom = 0.0
```

- [ ] Add field/list/toggle layout props to the corresponding roots:

```toml
layout_min_height = "$material_field_min_height"
layout_padding_left = "$material_field_padding_x"
layout_padding_right = "$material_field_padding_x"
layout_padding_top = "$material_field_padding_y"
layout_padding_bottom = "$material_field_padding_y"
```

```toml
layout_min_height = "$material_list_item_height"
layout_padding_left = "$material_list_item_padding_x"
layout_padding_right = "$material_list_item_padding_x"
layout_spacing = "$material_list_item_spacing"
```

- [ ] Keep existing `layout = { height = ... }` declarations only where a fixed authored height is still intended. Prefer content/min-driven layout for Material buttons so runtime intrinsic measurement can prove the padding contract.
- [ ] Add or update tests in `pane_body_documents.rs` to assert that projected `ButtonDemo`, `InputFieldDemo`, `ListRowDemo`, and `ComboBoxDemo` carry the layout attributes in `SlintUiHostNodeProjection.properties` or shared surface metadata, depending on the existing helper shape.
- [ ] Add a test that builds the shared surface for `editor.window.ui_component_showcase` and asserts the arranged `ButtonDemo` frame width is at least text intrinsic + Material horizontal padding and height is at least `material_control_height`.

### Testing Stage: Material Asset Projection Gate

Run targeted formatting/checks for touched TOML and Rust tests:

```powershell
rustfmt --edition 2021 --check "zircon_editor/src/tests/host/template_runtime/pane_body_documents.rs"
```

Run focused template runtime tests:

```powershell
$env:TMP="E:\tmp\cargo-tmp"; $env:TEMP="E:\tmp\cargo-tmp"; cargo test -p zircon_editor --lib component_showcase_projection_carries_runtime_component_semantics --locked --jobs 1 --target-dir "E:\zircon-build\targets\material-layout-foundation" --message-format short --color never -- --nocapture
```

Run the broader template-runtime component-showcase filter if the focused test passes:

```powershell
$env:TMP="E:\tmp\cargo-tmp"; $env:TEMP="E:\tmp\cargo-tmp"; cargo test -p zircon_editor --lib component_showcase --locked --jobs 1 --target-dir "E:\zircon-build\targets\material-layout-foundation" --message-format short --color never -- --nocapture
```

Run scoped editor type check:

```powershell
$env:TMP="E:\tmp\cargo-tmp"; $env:TEMP="E:\tmp\cargo-tmp"; cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir "E:\zircon-build\targets\material-layout-foundation" --message-format short --color never
```

### Debug / Correction Loop

- If TOML projection tests fail because a token remains unresolved, inspect `zircon_runtime/src/ui/template/asset/compiler/component_props.rs` and `node_expander.rs` before adding editor-side fallbacks.
- If shared surface arranged frames do not reflect Material min sizes, return to Milestone 1 measurement and confirm the metadata reached `UiTemplateNodeMetadata.attributes`.
- If Asset Browser tests or media paths fail, check the active asset-browser session before editing those files.

### Exit Evidence

- Component Showcase Material nodes expose layout metrics through generic projection.
- Shared surface arranged frames reflect Material intrinsic layout.
- Editor check passes or an unrelated active-session blocker is recorded with exact diagnostics.

## Milestone 3: Native Host Layout Consumption And Hit Coherence

- Goal: Prove native editor host consumes Material arranged frames through `UiSurfaceFrame` and hit-grid routing without reintroducing local coordinate tables.
- In-scope behaviors: native host tests for Component Showcase or pane template Material button frames, hit-grid coherence for padded Material button frames, and no native painter layout math.
- Dependencies: Milestones 1 and 2 arranged frames are stable.

### Implementation Slices

- [ ] Add a focused editor host test in `zircon_editor/src/tests/host/slint_window/native_host_contract.rs` or a new `zircon_editor/src/tests/host/slint_window/native_material_layout.rs` if the contract file is already too broad.
- [ ] The test should build a pane/body surface containing a Material button or use an existing Component Showcase pane projection, then click inside the padded part of the arranged frame outside the raw text bounds.
- [ ] Assert the route dispatches through the existing `PaneSurfaceHostContext.surface_control_clicked` or Component Showcase binding path.
- [ ] Assert no Rust coordinate table is used. Prefer deriving the click point from `PaneData.body_surface_frame.arranged_tree` or the projected `TemplatePaneNodeData.frame` already built from the shared surface.
- [ ] Do not modify `surface_hit_test` unless the test exposes an actual shared hit-grid bug.

### Testing Stage: Native Material Hit Gate

Run targeted formatting for touched editor host tests:

```powershell
rustfmt --edition 2021 --check "zircon_editor/src/tests/host/slint_window/native_host_contract.rs" "zircon_editor/src/tests/host/slint_window/native_material_layout.rs" "zircon_editor/src/tests/host/slint_window/mod.rs"
```

If `native_material_layout.rs` is not created, omit it from the command rather than creating an empty file.

Run focused native host tests:

```powershell
$env:TMP="E:\tmp\cargo-tmp"; $env:TEMP="E:\tmp\cargo-tmp"; cargo test -p zircon_editor --lib native_host_material --locked --jobs 1 --target-dir "E:\zircon-build\targets\material-layout-foundation" --message-format short --color never -- --nocapture
```

If the test is added to the existing contract filter, run:

```powershell
$env:TMP="E:\tmp\cargo-tmp"; $env:TEMP="E:\tmp\cargo-tmp"; cargo test -p zircon_editor --lib native_host_contract --locked --jobs 1 --target-dir "E:\zircon-build\targets\material-layout-foundation" --message-format short --color never -- --nocapture
```

### Debug / Correction Loop

- If click routing misses only in the padded area, inspect the `UiSurfaceFrame.hit_grid` entry frame before editing native pointer code.
- If hit-grid entry frame is correct but native route misses, inspect `surface_hit_test/template_node.rs` and `native_pointer.rs` with the hard-cutover rule: do not add a local Material coordinate shim.
- If the native painter output looks visually clipped but hit routing is correct, defer visual styling to the active Material/SVG/painter session unless the frame itself is wrong.

### Exit Evidence

- Native host click coverage proves Material padded arranged frames are hit-testable.
- No new local coordinate table or Material-specific native hit-test shim exists.

## Milestone 4: Documentation, Acceptance, And Closeout

- Goal: Record the Material layout foundation contract, reference evidence, validation commands, and remaining deferred Material visual/interaction work.
- In-scope behaviors: docs update, acceptance evidence, coordination note update, stale/overlap caveats, and final risk report.
- Dependencies: Milestones 1 through 3 implementation slices.

### Implementation Slices

- [ ] Update `docs/ui-and-layout/runtime-ui-component-showcase.md` frontmatter and body with:
  - `material_meta_components.ui.toml`
  - runtime layout measurement files
  - Component Showcase projection/native tests
  - Slint Material reference files
  - validation commands and outcomes
- [ ] Update `docs/ui-and-layout/shared-ui-core-foundation.md` with the shared Material intrinsic measurement contract.
- [ ] Create `tests/acceptance/material-layout-foundation.md` with scope, baseline, test inventory, commands, results, known non-goals, and acceptance decision.
- [ ] Update the relevant active session note under `.codex/sessions/` with current step, touched modules, test evidence, overlap warnings, and next update.

### Testing Stage: Documentation And Final Gate

Run stale/scope searches:

```powershell
rg "MaterialButtonBase|layout_padding_left|layout_min_height|material_button_padding_x" zircon_editor/assets/ui/editor/material_meta_components.ui.toml zircon_runtime/src/ui zircon_editor/src/tests docs tests/acceptance
```

Run whitespace checks for touched files:

```powershell
git diff --check -- "zircon_runtime/src/ui/layout/pass/measure.rs" "zircon_runtime/src/ui/layout/pass/material.rs" "zircon_runtime/src/ui/layout/pass/mod.rs" "zircon_runtime/src/ui/tests/shared_core.rs" "zircon_runtime/src/ui/tests/material_layout.rs" "zircon_runtime/src/ui/tests/mod.rs" "zircon_editor/assets/ui/editor/material_meta_components.ui.toml" "zircon_editor/src/tests/host/template_runtime/pane_body_documents.rs" "zircon_editor/src/tests/host/slint_window/native_host_contract.rs" "zircon_editor/src/tests/host/slint_window/native_material_layout.rs" "zircon_editor/src/tests/host/slint_window/mod.rs" "docs/ui-and-layout/runtime-ui-component-showcase.md" "docs/ui-and-layout/shared-ui-core-foundation.md" "tests/acceptance/material-layout-foundation.md"
```

Run final scoped checks on the shared target:

```powershell
$env:TMP="E:\tmp\cargo-tmp"; $env:TEMP="E:\tmp\cargo-tmp"; cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir "E:\zircon-build\targets\material-layout-foundation" --message-format short --color never
```

```powershell
$env:TMP="E:\tmp\cargo-tmp"; $env:TEMP="E:\tmp\cargo-tmp"; cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir "E:\zircon-build\targets\material-layout-foundation" --message-format short --color never
```

### Debug / Correction Loop

- If documentation references a test not run in the current validation stage, remove the pass claim or run the test before closeout.
- If `git diff --check` reports only LF-to-CRLF warnings, record them as line-ending warnings rather than source whitespace errors.
- If Cargo fails outside Runtime UI/editor Material layout paths, read active `.codex/sessions` notes before editing unrelated modules.

### Exit Evidence

- Acceptance doc lists every focused command and its result.
- Runtime/editor docs record the Material layout contract and Slint reference files.
- Final response reports no workspace-wide green claim unless full workspace validation was actually run.

## Out Of Scope For This Plan

- Full Material color palette parity and dynamic light/dark scheme switching.
- Ripple animation timing, state-layer opacity animation, text cursor animation, and floating-label animation.
- SVG/image drawing and media preview fixes, except where existing shared projection fields are needed by layout tests.
- Asset Browser-specific responsive layout.
- World-space UI Material rendering.
- Restoring generated `.slint` UI modules.

## Self-Review

- Spec coverage: The user asked to proceed with the recommended Material shared foundation and then clarified layout emphasis. Milestone 1 covers shared runtime measurement, Milestone 2 covers `.ui.toml` Material metrics/projection, Milestone 3 covers native hit coherence from arranged frames, and Milestone 4 covers docs/acceptance.
- Placeholder scan: No placeholder markers or vague "write tests" steps remain. Every test slice names concrete behaviors and files.
- Type consistency: The plan uses existing `UiTemplateNodeMetadata`, `UiSize`, `TemplatePaneNodeData`, `UiSurfaceFrame`, and `PaneData.body_surface_frame` terminology. New layout attribute names are declared once in `Material Layout Contract` and reused consistently.
