# Button Visual Slice Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Polish the retained-host `Button` component as the first focused UI component quality slice.

**Architecture:** Keep Button semantics in the existing retained UI metadata and native painter path. `.zui` files expose representative Button states, `template_nodes.rs` resolves deterministic Button colors/borders, and editor tests verify projection, painting, click routing, and disabled behavior.

**Tech Stack:** Rust, Zircon retained host, `.zui`/`.ui.toml` assets, native painter byte-buffer tests, focused Cargo gates.

---

## Milestone 1: Button Visual And Interaction Slice

### Files

- Modify: `zircon_editor/assets/ui/editor/components/showcase_input_section.zui`
- Modify: `zircon_editor/assets/ui/editor/material_components/material_buttons.zui`
- Modify: `zircon_editor/src/ui/retained_host/host_contract/painter/template_nodes.rs`
- Modify: `zircon_editor/src/tests/host/retained_window/native_material_painter.rs`
- Modify: `zircon_editor/src/tests/host/retained_window/native_host_contract.rs`
- Modify: `zircon_editor/src/tests/host/template_runtime/pane_body_documents.rs`
- Modify: `docs/ui-and-layout/material-ui-token-component-audit.md`

### Implementation Slices

- [x] Add focused Button painter tests for contained, outlined/text, hover, pressed, focus, disabled, and danger/error states.
- [x] Adjust Button color and border resolution in `template_nodes.rs` so variant and state priority are deterministic and visibly distinct.
- [x] Expand the Button showcase/material lab assets so visible examples cover primary, secondary/outlined, text, danger, disabled, hover, pressed, and focus without changing public schema.
- [x] Add projection assertions that `ButtonDemo` keeps `button_variant`, `surface_variant`, state flags, padding, and input metadata.
- [x] Add or extend pointer-routing assertions so projected Button hit frames match layout frames and disabled Button nodes do not dispatch.
- [x] Update Material UI documentation with the Button slice contract, changed files, and validation commands.

### Testing Stage

Run focused editor gates first:

```powershell
$env:CARGO_TARGET_DIR='target\button-visual-slice'
cargo test -p zircon_editor --lib native_template_painter --locked --jobs 1
cargo test -p zircon_editor --lib component_showcase_projection_carries_runtime_component_semantics --locked --jobs 1
cargo test -p zircon_editor --lib native_host_pointer_click_routes_projected_material_showcase_button --locked --jobs 1
```

Then run a scoped type gate:

```powershell
$env:CARGO_TARGET_DIR='target\button-visual-slice'
cargo check -p zircon_editor --lib --locked --jobs 1
```

Optional live evidence after the code gate:

```powershell
.\tools\ui-profile-capture.ps1 -Scenario material_lab_click -AutoInteract -RequireScenarioEvidence -AutoCloseSeconds 8
```

### Acceptance

- Button states are visually distinguishable in native painter tests.
- The projected Button frame is the click target frame.
- Disabled Button does not activate.
- The slice does not touch `zircon_hub` or Slint.
- Validation evidence is recorded in the closeout summary and the UI docs.

### 2026-05-17 Validation Closeout

- `cargo metadata --locked --no-deps --format-version 1` passed after restoring the `gpu-allocator 0.28.0 -> windows 0.62.2` lockfile edge required by `wgpu-hal 29.0.3` on Windows DX12.
- `cargo test -p zircon_runtime_interface --lib ui_contract_spine --locked --jobs 1` passed, `6 passed`.
- `cargo test -p zircon_runtime --lib material_button_style --locked --jobs 1` passed, `4 passed`.
- `cargo test -p zircon_runtime --lib component_catalog --locked --jobs 1` passed, `45 passed`.
- `cargo test -p zircon_editor --lib native_material_painter --locked --jobs 1` passed, `5 passed`.
- `cargo test -p zircon_editor --lib native_host_contract --locked --jobs 1` passed, `40 passed`.
- `cargo test -p zircon_editor --lib pane_body_documents --locked --jobs 1` passed, `11 passed`.
- `cargo check -p zircon_app --features target-editor-host --locked --jobs 1` passed with the existing `RuntimeSession::create` dead-code warning.
- `rustfmt --edition 2021 --check` passed for the touched Rust files. `git diff --check` passed for the touched slice, reporting only existing LF-to-CRLF warnings.
- Live strict Material Lab click profile passed at `target/zircon-profiles/20260517-175506-material_lab_click`: `1324` visible draw items batched into `146` GPU draw calls, `batch_success_rate=0.890`, `draw_reduction_ratio=9.068`, `dependency_density=0.015`, `layer_density=12.259`, `hit_consistency_samples=15 failed=0`, `software_fallback_present_count=0`.
- Screenshot/fallback evidence was captured at `target/zircon-profiles/20260517-175549-material_lab_click` and direct Button showcase screenshots at `target/button-visual-slice/button-showcase-visual/`. `UI Component Showcase` shows primary, outlined, text, danger, and disabled Button rows in the first viewport. GPU-vs-softbuffer screenshot parity is not accepted yet: the showcase diff reports `differing_sample_ratio=0.9992`, `average_channel_delta=68.03`, matching the broader known GPU live-capture brightness/parity gap.
