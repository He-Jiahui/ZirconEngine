# Layout15 Material State Priority Convergence Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make retained Material state-layer and runtime Button boolean fallback share the explicit priority `disabled > pressed > drag > focus > hover > default`, then prove it with current-source managed tests and a dedicated `docs/tests/editor` screenshot.

**Architecture:** A private retained enum becomes the sole state-priority resolver and the existing opacity function becomes a pure state-to-opacity mapping. Runtime Button style extraction keeps its public `ButtonInteractionState` API and folds boolean dragging to `Hover`, matching the already-canonical shared `UiPainter` selector. A dedicated retained fixture exercises real text measurement, relative layout, painting, and PNG export without adding screenshot coordinates to production code.

**Tech Stack:** Rust 2024 workspace, Zircon retained UI, Runtime UI v2 style values, shared Runtime Text measurement, `image` PNG export, Windows session coordinator, Cargo/rustfmt, Markdown module and plan records.

---

## Execution Context

- Execute from the shared `E:\Git\ZirconEngine` main checkout; repository coordination policy forbids creating an unrelated feature worktree for this shared milestone.
- Register one exact implementation Session against `docs/plans/zircon_editor/editor_layout/15-component-standardization-from-primitives.md` and claim only the files listed in this plan.
- Apply `prefer-windows-validation` before every Cargo command. Cargo output must use a coordinator-managed target rooted below `D:\cargo-targets`, `E:\cargo-targets`, or `F:\cargo-targets`.
- Do not invoke raw build-producing Cargo outside the coordinator and do not hand-stage or hand-commit files.
- Keep the accepted TextField/Dialog M1.1 manifest and all Render18 paths out of this milestone.

## File and Responsibility Map

### Production and contract files

- Modify `zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/material_state_layer/state.rs`
  - Own the private `MaterialStateLayerResolvedState` enum.
  - Resolve node flags into one semantic state.
  - Map that state to the existing opacity tokens.
- Modify `zircon_runtime/src/ui/style.rs`
  - Add boolean `dragging` fallback between pressed and focused.
  - Map dragging to the existing `ButtonInteractionState::Hover` bucket.
- Preserve `zircon_runtime_interface/src/ui/style.rs`
  - No production change expected; its existing `UiPainterStyleSelector::button_interaction_state` is the lower shared contract.

### Test and artifact files

- Modify `zircon_runtime/src/ui/tests/material_button_style.rs`
  - Add focused+dragging+hovered runtime fallback coverage.
  - Preserve explicit-string and pressed/disabled priority coverage.
- Preserve `zircon_runtime_interface/src/tests/ui_painter_style_contracts.rs`
  - Re-run `ui_painter_state_keeps_drag_priority_above_focus`; only edit if current evidence reveals a real lower-layer defect.
- Create `zircon_editor/src/tests/host/retained_menu_pointer/material_state_layer_visual_screenshot.rs`
  - Build four real retained Material state samples.
  - Assert pressed/dragging samples do not collapse to focused-only output.
  - Export the ignored PNG capture.
- Modify `zircon_editor/src/tests/host/retained_menu_pointer/mod.rs`
  - Register `material_state_layer_visual_screenshot` once, alphabetically beside the other Material fixtures.
- Create `docs/tests/editor/editor-components-material-state-layer-900x360.png`
  - Current-source visual acceptance artifact; never write the same file name beneath a target directory.

### Documentation and milestone records

- Modify `docs/zircon_editor/ui/retained_host/host_contract/paint_template_nodes/index.md`
  - Record the enum-owned priority and retained opacity contract.
  - Keep `related_code`, implementation, plan, and test references current.
- Modify `docs/zircon_runtime/ui/v2.md`
  - Record runtime Button boolean dragging folding to Hover.
- Modify `docs/plans/zircon_editor/editor_layout/15-component-standardization-from-primitives.md`
  - Add the accepted slice status only after testing-stage acceptance.
- Create `docs/plans/zircon_editor/editor_layout/15/2026-07-16-material-state-priority-convergence.md`
  - Canonical milestone output record written once after acceptance.
- Preserve `docs/superpowers/specs/2026-07-16-layout15-material-state-priority-design.md`
  - Approved architecture source.

## Milestone M1: State Priority Contract Convergence

### Goal

Retained Material and runtime Button style resolution select the same highest-priority interaction state without expanding the public Button enum.

### In-scope behaviors

- Disabled state layer short-circuits every interactive state.
- Pressed and enter-pressed beat dragging, focus, selection, checked, and hover.
- Dragging beats focus, selection, checked, hover, drop-hover, and active-drag-target.
- Focus, selection, or checked beats hover-class states.
- Runtime boolean dragging resolves to `ButtonInteractionState::Hover` before focused is considered.
- Explicit authored string interaction state remains above all boolean fallbacks.
- Existing opacity values remain hover `0.08`, focus/disabled `0.10`, press `0.10`, drag `0.16`.

### Dependencies

- `UiPainterStyleSelector::button_interaction_state` already maps `UiPainterResolvedState::Dragging` to `ButtonInteractionState::Hover`.
- `TemplatePaneNodeData` already carries `state_layer_enabled`, `disabled`, `pressed`, `enter_pressed`, `dragging`, `focused`, `selected`, `checked`, `hovered`, `drop_hovered`, and `active_drag_target`.
- M1.1 TextField/Dialog files remain a separate candidate and are not modified.

### Implementation Slice M1-A: Write contract tests before production changes

- [ ] Extend `material_state_layer/state.rs` tests with a table-driven resolver contract. The test must construct these cases and assert the exact resolved state:

```rust
#[test]
fn material_state_layer_resolves_exact_interaction_priority() {
    let cases = [
        (
            TemplatePaneNodeData {
                state_layer_enabled: false,
                disabled: true,
                pressed: true,
                dragging: true,
                focused: true,
                hovered: true,
                ..TemplatePaneNodeData::default()
            },
            None,
        ),
        (
            TemplatePaneNodeData {
                state_layer_enabled: true,
                disabled: true,
                pressed: true,
                dragging: true,
                focused: true,
                hovered: true,
                ..TemplatePaneNodeData::default()
            },
            Some(MaterialStateLayerResolvedState::Disabled),
        ),
        (
            TemplatePaneNodeData {
                state_layer_enabled: true,
                pressed: true,
                dragging: true,
                focused: true,
                hovered: true,
                ..TemplatePaneNodeData::default()
            },
            Some(MaterialStateLayerResolvedState::Pressed),
        ),
        (
            TemplatePaneNodeData {
                state_layer_enabled: true,
                dragging: true,
                focused: true,
                selected: true,
                checked: true,
                hovered: true,
                ..TemplatePaneNodeData::default()
            },
            Some(MaterialStateLayerResolvedState::Dragging),
        ),
        (
            TemplatePaneNodeData {
                state_layer_enabled: true,
                focused: true,
                hovered: true,
                ..TemplatePaneNodeData::default()
            },
            Some(MaterialStateLayerResolvedState::Focused),
        ),
        (
            TemplatePaneNodeData {
                state_layer_enabled: true,
                hovered: true,
                ..TemplatePaneNodeData::default()
            },
            Some(MaterialStateLayerResolvedState::Hovered),
        ),
    ];

    for (node, expected) in cases {
        assert_eq!(MaterialStateLayerResolvedState::resolve(&node), expected);
    }
}
```

- [ ] Extend `material_button_style_resolves_slint_state_layer_priority` with the exact runtime drift case:

```rust
let values = BTreeMap::from([
    ("focused".to_string(), Value::Boolean(true)),
    ("dragging".to_string(), Value::Boolean(true)),
    ("hovered".to_string(), Value::Boolean(true)),
]);
let style = resolve_button_style_from_values(&values);
assert_eq!(style.interaction_state, ButtonInteractionState::Hover);
```

- [ ] Do not run Cargo yet. Run only lightweight source checks:

```powershell
rustfmt --edition 2021 --check `
  zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/material_state_layer/state.rs `
  zircon_runtime/src/ui/tests/material_button_style.rs
git diff --check -- `
  zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/material_state_layer/state.rs `
  zircon_runtime/src/ui/tests/material_button_style.rs
```

Expected: formatting and diff checks pass; the new tests remain intentionally unexecuted until the milestone testing stage.

### Implementation Slice M1-B: Introduce the private retained resolver

- [ ] Add this private enum and resolver above `state_layer_opacity` in `material_state_layer/state.rs`:

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MaterialStateLayerResolvedState {
    Disabled,
    Pressed,
    Dragging,
    Focused,
    Hovered,
}

impl MaterialStateLayerResolvedState {
    fn resolve(node: &TemplatePaneNodeData) -> Option<Self> {
        if !node.state_layer_enabled {
            return None;
        }
        if is_button_disabled(node) {
            return Some(Self::Disabled);
        }
        if node.pressed || node.enter_pressed {
            return Some(Self::Pressed);
        }
        if node.dragging {
            return Some(Self::Dragging);
        }
        if node.focused || node.selected || node.checked {
            return Some(Self::Focused);
        }
        if node.hovered || node.drop_hovered || node.active_drag_target {
            return Some(Self::Hovered);
        }
        None
    }

    const fn opacity(self) -> f32 {
        match self {
            Self::Disabled | Self::Focused => MATERIAL_STATE_LAYER_OPACITY_FOCUS,
            Self::Pressed => MATERIAL_STATE_LAYER_OPACITY_PRESS,
            Self::Dragging => MATERIAL_STATE_LAYER_OPACITY_DRAG,
            Self::Hovered => MATERIAL_STATE_LAYER_OPACITY_HOVER,
        }
    }
}
```

- [ ] Replace the current opacity `if` chain with a pure mapping:

```rust
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn state_layer_opacity(
    node: &TemplatePaneNodeData,
) -> Option<f32> {
    MaterialStateLayerResolvedState::resolve(node)
        .map(MaterialStateLayerResolvedState::opacity)
}
```

- [ ] Preserve `state_layer_color`, host palette lookup, declared-color override, and all opacity constants exactly.

### Implementation Slice M1-C: Converge runtime Button boolean fallback

- [ ] In `ButtonInteractionStateProperty::extract`, insert boolean dragging immediately after pressed and before focused:

```rust
.or_else(|| {
    bool_value(sheet, "dragging")
        .filter(|value| *value)
        .map(|_| ButtonInteractionState::Hover)
})
```

- [ ] Do not add `ButtonInteractionState::Dragging`, a compatibility alias, or a runtime/editor special case.

### Implementation Slice M1-D: Update module contracts

- [ ] Update `docs/zircon_editor/ui/retained_host/host_contract/paint_template_nodes/index.md` with:
  - private resolver ownership;
  - exact priority order;
  - unchanged opacity values;
  - test references to the state resolver and visual fixture.
- [ ] Update `docs/zircon_runtime/ui/v2.md` with:
  - explicit string state remains authoritative;
  - boolean order is loading, disabled, pressed, dragging, focused, hovered;
  - Button dragging intentionally folds to Hover to match shared `UiPainter`.
- [ ] Keep machine-readable `related_code`, `implementation_files`, `plan_sources`, and `tests` metadata accurate in each touched module document.

### M1 Testing Stage: Managed contract verification

- [ ] Apply `prefer-windows-validation` and ensure the coordinator reports no foreign blocking Cargo job.
- [ ] Run the shared lower-layer contract first through a coordinator-managed Windows test lane:

```powershell
$env:CODEX_THREAD_ID = 'editor-layout15-material-state-priority-implementation-20260716'
& .\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 `
  -Package zircon_runtime_interface `
  -SkipBuild `
  -VerboseOutput
```

Expected focused evidence: `ui_painter_state_keeps_drag_priority_above_focus` passes and Button resolves Dragging to Hover.

- [ ] Run the runtime contract through the same managed Windows policy:

```powershell
$env:CODEX_THREAD_ID = 'editor-layout15-material-state-priority-implementation-20260716'
& .\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 `
  -Package zircon_runtime `
  -SkipBuild `
  -VerboseOutput
```

Expected focused evidence: `material_button_style_resolves_slint_state_layer_priority` passes with focused+dragging resolving to Hover.

- [ ] Run the retained resolver test from a current-source coordinator-managed `zircon_editor` test binary:

```powershell
& $editorTestBinary material_state_layer_resolves_exact_interaction_priority `
  --nocapture --test-threads=1
```

Expected: `1 passed; 0 failed`.

- [ ] If an upper-layer validation fails, list the lower shared support candidates first and repair the lowest owned layer. For a foreign owner, create/return the canonical failure handoff instead of adding a UI bypass.
- [ ] Promotion gate: all three named contracts pass against the same current source; rustfmt and scoped diff checks pass; no public Button API change exists.

## Milestone M2: Retained Visual Acceptance and Coordinator Closeout

### Goal

Prove the converged state priority through a dedicated real-painter artifact whose layout is relative and whose only output is under `docs/tests/editor`.

### In-scope behaviors

- Four visually comparable samples: hovered/default, focused, pressed+focused, dragging+focused.
- Shared Runtime Text measurement for title, subtitle, sample labels, and state descriptions.
- Tokenized panel insets, gaps, rounded surfaces, and host palette colors.
- Visual and pixel guards distinguish press/drag from focused-only.
- Exact output name: `editor-components-material-state-layer-900x360.png`.

### Dependencies

- M1 contract tests and production changes are green.
- Existing retained test painter and `visual_layout_output_path` remain the sole screenshot route.
- `docs/tests/editor` is the only accepted artifact directory.

### Implementation Slice M2-A: Add the relative visual fixture

- [ ] Register the module in `retained_menu_pointer/mod.rs`:

```rust
mod material_feedback_visual_screenshot;
mod material_state_layer_visual_screenshot;
mod pointer_bridge;
```

- [ ] Create `material_state_layer_visual_screenshot.rs` with these constants and tests:

```rust
const MATERIAL_STATE_LAYER_SCREENSHOT: &str =
    "editor-components-material-state-layer-900x360.png";
const MATERIAL_STATE_LAYER_WIDTH: u32 = 900;
const MATERIAL_STATE_LAYER_HEIGHT: u32 = 360;
const MATERIAL_STATE_LAYER_BACKGROUND: [u8; 4] = [17, 20, 22, 255];
const OUTER_INSET: f32 = 18.0;
const PANEL_GAP: f32 = 12.0;
const PANEL_TOP: f32 = 78.0;
const PANEL_HEIGHT: f32 = 230.0;

#[test]
fn material_state_layer_visual_separates_hover_focus_press_and_drag_priority() {
    let nodes = material_state_layer_nodes();
    let bytes = material_state_layer_bytes_from_nodes(nodes);

    let hovered = sample_center(&bytes, 0);
    let focused = sample_center(&bytes, 1);
    let pressed = sample_center(&bytes, 2);
    let dragging = sample_center(&bytes, 3);

    assert_ne!(hovered, focused, "focus must remain distinct from hover");
    assert_ne!(pressed, focused, "pressed+focused must resolve as pressed");
    assert_ne!(dragging, focused, "dragging+focused must resolve as dragging");
    assert_ne!(dragging, hovered, "drag opacity must remain stronger than hover");
}

#[test]
#[ignore = "writes local Material state-layer screenshot artifact for visual review"]
fn capture_material_state_layer_visual_artifact() {
    let bytes = material_state_layer_bytes();
    let output_path = visual_layout_output_path(MATERIAL_STATE_LAYER_SCREENSHOT);
    image::save_buffer_with_format(
        &output_path,
        &bytes,
        MATERIAL_STATE_LAYER_WIDTH,
        MATERIAL_STATE_LAYER_HEIGHT,
        image::ColorType::Rgba8,
        image::ImageFormat::Png,
    )
    .expect("Material state-layer screenshot should be written as PNG");
    assert!(output_path.exists(), "expected screenshot at {}", output_path.display());
}
```

- [ ] Compute sample geometry from available width, never from four unrelated x coordinates:

```rust
fn sample_frame(index: usize) -> TemplateNodeFrameData {
    let available = MATERIAL_STATE_LAYER_WIDTH as f32 - OUTER_INSET * 2.0;
    let width = (available - PANEL_GAP * 3.0) / 4.0;
    let x = OUTER_INSET + index as f32 * (width + PANEL_GAP);
    frame(x, PANEL_TOP, width, PANEL_HEIGHT)
}
```

- [ ] Define the complete fixture state and helper path below. The sample itself is a real
  outlined Button so focused and pressed still exercise their base-surface split even though
  both state-layer opacity tokens are `0.10`; dragging remains visibly stronger through its
  `0.16` overlay.

```rust
use std::path::{Path, PathBuf};

use crate::ui::layouts::common::model_rc;
use crate::ui::retained_host::{
    paint_template_nodes_for_test_with_background, TemplateNodeFrameData,
    TemplatePaneNodeData,
};

#[derive(Clone, Copy)]
enum MaterialVisualState {
    Hovered,
    Focused,
    PressedFocused,
    DraggingFocused,
}

const MATERIAL_VISUAL_STATES: [MaterialVisualState; 4] = [
    MaterialVisualState::Hovered,
    MaterialVisualState::Focused,
    MaterialVisualState::PressedFocused,
    MaterialVisualState::DraggingFocused,
];

impl MaterialVisualState {
    const fn label(self) -> &'static str {
        match self {
            Self::Hovered => "Hovered",
            Self::Focused => "Focused",
            Self::PressedFocused => "Pressed + Focused",
            Self::DraggingFocused => "Dragging + Focused",
        }
    }

    const fn description(self) -> &'static str {
        match self {
            Self::Hovered => "Hover layer 0.08",
            Self::Focused => "Focus layer 0.10",
            Self::PressedFocused => "Pressed wins, 0.10",
            Self::DraggingFocused => "Drag wins, 0.16",
        }
    }
}

fn material_state_layer_bytes() -> Vec<u8> {
    material_state_layer_bytes_from_nodes(material_state_layer_nodes())
}

fn material_state_layer_bytes_from_nodes(nodes: Vec<TemplatePaneNodeData>) -> Vec<u8> {
    paint_template_nodes_for_test_with_background(
        MATERIAL_STATE_LAYER_WIDTH,
        MATERIAL_STATE_LAYER_HEIGHT,
        MATERIAL_STATE_LAYER_BACKGROUND,
        model_rc(nodes),
    )
}

fn material_state_layer_nodes() -> Vec<TemplatePaneNodeData> {
    let mut nodes = vec![
        surface("MaterialStateLayerRoot", "shell", frame(0.0, 0.0, 900.0, 360.0)),
        label(
            "MaterialStateLayerTitle",
            "Material State Priority",
            frame(22.0, 20.0, 320.0, 22.0),
            13.0,
            "",
        ),
        label(
            "MaterialStateLayerSubtitle",
            "Pressed and dragging stay above keyboard focus",
            frame(22.0, 42.0, 720.0, 18.0),
            10.0,
            "muted",
        ),
    ];

    for (index, state) in MATERIAL_VISUAL_STATES.into_iter().enumerate() {
        let panel = sample_frame(index);
        nodes.push(surface(
            &format!("MaterialStatePanel{index}"),
            "panel",
            panel.clone(),
        ));
        nodes.push(label(
            &format!("MaterialStateLabel{index}"),
            state.label(),
            frame(panel.x + 18.0, panel.y + 18.0, panel.width - 36.0, 18.0),
            11.0,
            "",
        ));
        nodes.push(state_button(
            &format!("MaterialStateButton{index}"),
            "Apply",
            state,
            state_button_frame(index),
        ));
        nodes.push(label(
            &format!("MaterialStateDescription{index}"),
            state.description(),
            frame(panel.x + 18.0, panel.y + 142.0, panel.width - 36.0, 18.0),
            9.0,
            "muted",
        ));
    }

    nodes
}

fn state_button_frame(index: usize) -> TemplateNodeFrameData {
    let panel = sample_frame(index);
    frame(panel.x + 18.0, panel.y + 66.0, panel.width - 36.0, 56.0)
}

fn state_button(
    control_id: &str,
    text: &str,
    state: MaterialVisualState,
    frame: TemplateNodeFrameData,
) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        control_id: control_id.into(),
        role: "Button".into(),
        component_role: "button".into(),
        button_variant: "outlined".into(),
        action_id: "workbench.material.apply".into(),
        text: text.into(),
        state_layer_enabled: true,
        hovered: matches!(state, MaterialVisualState::Hovered),
        focused: matches!(
            state,
            MaterialVisualState::Focused
                | MaterialVisualState::PressedFocused
                | MaterialVisualState::DraggingFocused
        ),
        pressed: matches!(state, MaterialVisualState::PressedFocused),
        dragging: matches!(state, MaterialVisualState::DraggingFocused),
        frame,
        ..TemplatePaneNodeData::default()
    }
}

fn sample_center(bytes: &[u8], index: usize) -> [u8; 4] {
    let button = state_button_frame(index);
    // Sample inside the rounded surface but left of the centered runtime text.
    pixel_at(bytes, (button.x + 14.0) as u32, (button.y + 14.0) as u32)
}

fn surface(
    control_id: &str,
    variant: &str,
    frame: TemplateNodeFrameData,
) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        control_id: control_id.into(),
        role: "Panel".into(),
        surface_variant: variant.into(),
        border_width: 1.0,
        corner_radius: 6.0,
        frame,
        ..TemplatePaneNodeData::default()
    }
}

fn label(
    control_id: &str,
    text: &str,
    frame: TemplateNodeFrameData,
    font_size: f32,
    tone: &str,
) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        control_id: control_id.into(),
        role: "Label".into(),
        text: text.into(),
        font_size,
        text_tone: tone.into(),
        frame,
        ..TemplatePaneNodeData::default()
    }
}

fn frame(x: f32, y: f32, width: f32, height: f32) -> TemplateNodeFrameData {
    TemplateNodeFrameData {
        x,
        y,
        width,
        height,
    }
}

fn pixel_at(bytes: &[u8], x: u32, y: u32) -> [u8; 4] {
    let index = ((y as usize * MATERIAL_STATE_LAYER_WIDTH as usize) + x as usize) * 4;
    [
        bytes[index],
        bytes[index + 1],
        bytes[index + 2],
        bytes[index + 3],
    ]
}

fn visual_layout_output_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("editor crate should live under the repository root")
        .join("docs")
        .join("tests")
        .join("editor")
}

fn visual_layout_output_path(filename: &str) -> PathBuf {
    let output_dir = visual_layout_output_dir();
    std::fs::create_dir_all(&output_dir)
        .expect("visual-layout output directory should exist");
    output_dir.join(filename)
}
```

- [ ] Each sample node sets `state_layer_enabled: true` and only the flags encoded by
  `MaterialVisualState`. All labels use the retained Runtime Text path; do not rasterize
  text locally or name a concrete font family.

### Implementation Slice M2-B: Add artifact and target guards

- [ ] The ignored capture writes via `visual_layout_output_path`, which resolves to `docs/tests/editor`.
- [ ] After capture, compute and record the PNG SHA-256:

```powershell
Get-FileHash -Algorithm SHA256 `
  docs/tests/editor/editor-components-material-state-layer-900x360.png
```

- [ ] Scan repository and managed target roots for the exact file name:

```powershell
$name = 'editor-components-material-state-layer-900x360.png'
$matches = @(
  Get-ChildItem -Path target,D:\cargo-targets,E:\cargo-targets,F:\cargo-targets `
    -Recurse -File -Filter $name -ErrorAction SilentlyContinue
)
if ($matches.Count -ne 0) {
    throw "Material state-layer screenshot leaked into a target root: $($matches.FullName -join ', ')"
}
```

Expected: zero target matches; the only accepted file is under `docs/tests/editor`.

### Implementation Slice M2-C: Document and record the accepted milestone

- [ ] Update the parent Layout15 slice only after testing succeeds. Record that this slice closes the explicit state-priority resolver, runtime dragging fallback, and dedicated local artifact; do not mark parent M1 or full S15.4 complete.
- [ ] Create `docs/plans/zircon_editor/editor_layout/15/2026-07-16-material-state-priority-convergence.md` with exactly one machine-readable `Plan`, `Milestone`, `Status`, and `Files` field plus these headings:
  - `## Scope delivered`
  - `## Fresh testing evidence`
  - `## Review`
- [ ] Include current-source managed job IDs, exact focused test counts, PNG bytes/hash, target scan count, independent review severity counts, and accepted residual risk.

### M2 Testing Stage: Current-source visual acceptance

- [ ] Run rustfmt and scoped diff checks across the exact candidate.
- [ ] Build or refresh a current-source `zircon_editor` test binary using only the coordinator-managed Windows lane.
- [ ] Execute the exact visual guard:

```powershell
& $editorTestBinary material_state_layer_visual_separates_hover_focus_press_and_drag_priority `
  --nocapture --test-threads=1
```

Expected: `1 passed; 0 failed`.

- [ ] Execute the ignored capture:

```powershell
& $editorTestBinary capture_material_state_layer_visual_artifact `
  --ignored --nocapture --test-threads=1
```

Expected: `1 passed; 0 failed`; PNG exists only in `docs/tests/editor`.

- [ ] Visually inspect the PNG at original detail and verify:
  - all four labels are readable;
  - surfaces and gaps are balanced and use the shared rounded style;
  - focused does not impersonate pressed;
  - dragging is visibly stronger than hover/focus;
  - no overlap, clipping, large unexplained empty region, or absolute-position drift appears.
- [ ] Run the output-record audit, scoped plan audit, `git diff --check`, and exact-manifest review.
- [ ] Independent reviewer gate: Critical/Important/Minor must be `0/0/0` before commit.
- [ ] Coordinator milestone gate: validation, review, failure audit, plan output, and exact commit manifest must all bind to the same current HEAD and candidate fingerprint.
- [ ] Commit only through the coordinator; after success, record the commit hash and keep the broader Layout15 goal active.

## Debug and Correction Loop

1. If retained enum tests fail, correct the resolver before touching opacity or visual fixtures.
2. If runtime style tests fail, compare the authored-string override and boolean fallback chain against shared `UiPainter`; do not add a second selector.
3. If the visual guard fails but contract tests pass, inspect state-layer paint color/alpha, inherited clip, measured label frames, and pixel sample positions in that order.
4. If a build fails outside the exact candidate, apply support-first diagnosis and return a canonical cross-plan failure to its owning numbered plan.
5. After every correction, rerun the lowest failed layer, then the dependent contract, then the visual guard/capture.

## Acceptance Manifest

The final exact manifest may contain only:

- `zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/material_state_layer/state.rs`
- `zircon_runtime/src/ui/style.rs`
- `zircon_runtime/src/ui/tests/material_button_style.rs`
- `zircon_editor/src/tests/host/retained_menu_pointer/material_state_layer_visual_screenshot.rs`
- `zircon_editor/src/tests/host/retained_menu_pointer/mod.rs`
- `docs/tests/editor/editor-components-material-state-layer-900x360.png`
- `docs/zircon_editor/ui/retained_host/host_contract/paint_template_nodes/index.md`
- `docs/zircon_runtime/ui/v2.md`
- `docs/plans/zircon_editor/editor_layout/15-component-standardization-from-primitives.md`
- `docs/plans/zircon_editor/editor_layout/15/2026-07-16-material-state-priority-convergence.md`
- `docs/superpowers/specs/2026-07-16-layout15-material-state-priority-design.md`
- `docs/superpowers/plans/2026-07-16-layout15-material-state-priority.md`

Any changed Runtime Text, Render18, M1.1 TextField/Dialog, workspace manifest, lockfile, compatibility facade, or unrelated screenshot path is foreign and must be excluded.

## 状态与产出记录

每个里程碑测试通过后记录一次；实现切片不单独写入产出记录。

| 里程碑 | 范围 | 状态 | 完成日期 | 证据（命令输出 / 文件 / 测试名） |
|---|---|---|---|---|
