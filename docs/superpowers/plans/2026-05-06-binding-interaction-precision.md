# Binding Interaction Precision Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make `.ui.toml` binding-driven interaction inference set only the capabilities implied by each binding event kind while preserving explicit `input_*` metadata and the temporary legacy component fallback.

**Architecture:** Keep the correction inside `zircon_runtime/src/ui/template/build/interaction.rs`. Replace the single broad `binding_requires_interaction(...)` boolean with an internal capability accumulator so click, hover, focus, and receive-input are inferred independently before explicit metadata and legacy fallback are applied.

**Tech Stack:** Rust, `zircon_runtime` template builder tests, scoped Cargo validation with `--locked`.

---

## File Structure

- Modify `zircon_runtime/src/ui/template/build/interaction.rs`: add a small private `InferredInputCapabilities` helper and event-kind mapping.
- Modify `zircon_runtime/src/ui/tests/template.rs`: add focused regression tests for scroll-only, focus-only, hover-only, and click-binding behavior.
- Modify `docs/ui-and-layout/shared-ui-core-foundation.md`: document precise binding capability inference.
- Modify `tests/acceptance/widget-behavior-closure.md`: replace the residual-risk note with final validation evidence if the slice passes.
- Update `.codex/sessions/20260506-1224-binding-interaction-precision.md`: live coordination and closeout evidence.

## Milestone 1: Binding Capability Inference

### Implementation Slice

- [ ] Add focused tests in `zircon_runtime/src/ui/tests/template.rs`:
  - `template_tree_builder_infers_scroll_binding_as_receive_input_only`
  - `template_tree_builder_infers_focus_binding_as_focusable_only`
  - `template_tree_builder_infers_hover_binding_as_hoverable_only`
  - `template_tree_builder_keeps_click_binding_button_like_by_default`
- [ ] In `zircon_runtime/src/ui/template/build/interaction.rs`, replace `binding_requires_interaction(...) -> bool` with a private helper equivalent to:

```rust
#[derive(Default)]
struct InferredInputCapabilities {
    receives_input: bool,
    clickable: bool,
    hoverable: bool,
    focusable: bool,
}
```

- [ ] Map `UiEventKind` to capabilities:
  - `Click`, `DoubleClick`, `Press`, `Release`, `Submit`, `Toggle`, `Change`: `receives_input`, `clickable`, `hoverable`, `focusable`.
  - `Hover`: `receives_input`, `hoverable`.
  - `Focus`, `Blur`: `receives_input`, `focusable`.
  - `Scroll`: `receives_input` only.
  - `DragBegin`, `DragUpdate`, `DragEnd`, `Drop`: `receives_input`, `hoverable`.
- [ ] Preserve explicit metadata semantics:
  - `input_clickable`, `input_hoverable`, and `input_focusable` override the corresponding inferred flag.
  - `input_interactive = true` remains broad opt-in for all three capability flags.
  - explicit false metadata still suppresses the legacy `Button` / `IconButton` / `TextField` fallback.
- [ ] Keep `legacy_component_interaction_fallback(...)` unchanged except for call-site wiring.

### Testing Stage

- [ ] Run formatting:

```powershell
rustfmt --edition 2021 --check "zircon_runtime/src/ui/template/build/interaction.rs" "zircon_runtime/src/ui/tests/template.rs"
```

Expected: no output and exit 0.

- [ ] Run focused template tests:

```powershell
$env:TMP="E:\tmp\cargo-tmp"; $env:TEMP="E:\tmp\cargo-tmp"; cargo test -p zircon_runtime --lib ui::tests::template --locked --jobs 1 --target-dir "E:\zircon-build\targets\widget-behavior-closure" --message-format short --color never -- --nocapture
```

Expected: existing `ui::tests::template` tests plus the new focused regressions pass.

- [ ] Run scoped runtime type check:

```powershell
$env:TMP="E:\tmp\cargo-tmp"; $env:TEMP="E:\tmp\cargo-tmp"; cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir "E:\zircon-build\targets\widget-behavior-closure" --message-format short --color never
```

Expected: exit 0 with only existing warning noise.

### Docs And Acceptance

- [ ] Update `docs/ui-and-layout/shared-ui-core-foundation.md` to state that binding inference is capability-specific.
- [ ] Update `tests/acceptance/widget-behavior-closure.md` with the focused validation commands and remove the residual binding-precision risk if validation passes.
- [ ] Run `git diff --check` and record whether output contains only LF-to-CRLF warnings or real whitespace errors.

## Self-Review

- Spec coverage: the plan covers the user-approved binding precision design and does not expand into Material/editor host work.
- Placeholder scan: no placeholder tasks remain.
- Type consistency: all mentioned test and helper names are private to the touched files and use existing `UiEventKind` / `UiStateFlags` semantics.
