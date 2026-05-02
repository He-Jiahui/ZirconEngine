# UI Productization Editor Binding Parity Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Complete the remaining UI productization milestones after M16 and M15: editor contract diagnostics/root-class authoring, runtime-owned binding semantics, UI Asset Editor recovery/designer behavior, and runtime/editor dual-host parity.

**Architecture:** Keep runtime asset/template/component contracts authoritative in `zircon_runtime::ui`, and keep `zircon_editor` as an authoring/projection consumer. Execute only after active M15/M12/Cargo sessions release overlapping compiler/cache/invalidation files; do not introduce Slint source authority or graphics/plugin changes.

**Tech Stack:** Rust 2021, Serde/TOML, Cargo `--locked`, existing `zircon_runtime::ui::template::asset`, existing `zircon_editor::ui::asset_editor`, and existing Rust-owned host-contract projection.

---

## Execution Gate

- [x] Refresh `.codex/sessions` with `Get-RecentCoordinationContext.ps1` before source edits.
- [ ] Confirm active M15 implementation has either completed or handed off before editing `zircon_runtime/src/ui/template/asset/document.rs`, `zircon_runtime/src/ui/template/asset/compiler/**`, `zircon_runtime/src/ui/template/asset/invalidation/**`, or `zircon_runtime/src/ui/template/asset/resource_ref/**`.
- [ ] Confirm active M12 post-review validation has finished before editing compile cache or invalidation files.
- [x] Use an isolated Cargo target directory on a drive with more than 50 GB free space, or wait for the current Cargo writer to finish before running validation.
- [x] Do not edit graphics, plugin renderer, Runtime UI showcase, SDF font rendering, deleted Slint source, or editor chrome files from this plan.

## File Structure

### Editor Contract Productization

- Create: `zircon_runtime/src/ui/template/asset/component_contract/diagnostic.rs` for stable runtime component-contract diagnostic codes and paths.
- Modify: `zircon_runtime/src/ui/template/asset/component_contract/mod.rs` to re-export diagnostics.
- Modify: `zircon_runtime/src/ui/template/asset/component_contract/validation.rs` to attach structured diagnostic codes before converting to `UiAssetError`.
- Create: `zircon_editor/src/ui/asset_editor/diagnostics/mod.rs` for editor diagnostic DTOs.
- Create: `zircon_editor/src/ui/asset_editor/diagnostics/contract.rs` for mapping runtime component-contract diagnostics into editor diagnostics.
- Modify: `zircon_editor/src/ui/asset_editor/session/ui_asset_editor_session.rs` to retain structured diagnostics alongside existing string diagnostics.
- Modify: `zircon_editor/src/ui/asset_editor/session/lifecycle.rs` and `zircon_editor/src/ui/asset_editor/session/presentation_state.rs` to refresh and project diagnostics.
- Create: `zircon_editor/src/ui/asset_editor/session/root_class_policy_state.rs` for selected component root-class policy lookup and replay-aware authoring.
- Create: `zircon_editor/src/tests/ui/ui_asset_editor/contract_diagnostics.rs` for editor-facing diagnostic tests.

### M18 Runtime Binding Semantics

- Create: `zircon_runtime/src/ui/template/asset/binding/mod.rs` for folder-backed binding schema exports.
- Create: `zircon_runtime/src/ui/template/asset/binding/expression.rs` for restricted expression AST.
- Create: `zircon_runtime/src/ui/template/asset/binding/target.rs` for binding target schema.
- Create: `zircon_runtime/src/ui/template/asset/binding/diagnostic.rs` for binding diagnostic codes.
- Create: `zircon_runtime/src/ui/template/asset/binding/validation.rs` for compile-time validation.
- Modify: `zircon_runtime/src/ui/template/asset/mod.rs` and `zircon_runtime/src/ui/template/mod.rs` to re-export runtime binding schema types.
- Modify: `zircon_runtime/src/ui/template/asset/compiler/compile.rs` to run binding validation before expansion returns a compiled document.
- Create: `zircon_runtime/src/ui/tests/asset_binding.rs` for runtime binding schema tests.
- Modify: `zircon_editor/src/ui/asset_editor/binding/binding_inspector.rs` and `zircon_editor/src/ui/asset_editor/binding/schema_projection.rs` to consume runtime binding diagnostics instead of editor-only inference.
- Create: `zircon_editor/src/tests/ui/ui_asset_editor/binding_semantics.rs` for editor diagnostic/projection tests.

### M5/M6/M24 UI Asset Editor Closure

- Create: `zircon_editor/src/ui/asset_editor/session/shell_state.rs` for `UiAssetEditorShellState` and emergency/last-valid flags.
- Create: `zircon_editor/src/ui/asset_editor/session/designer_tool_state.rs` for select, resize-slot, and preview-interact modes.
- Create: `zircon_editor/src/ui/asset_editor/session/slot_resize.rs` for slot resize transaction commands.
- Create: `zircon_editor/src/ui/asset_editor/session/preview_interaction.rs` for preview interaction dispatch.
- Modify: `zircon_editor/src/ui/asset_editor/session/ui_asset_editor_session.rs` to retain shell and tool state.
- Modify: `zircon_editor/src/ui/asset_editor/session/lifecycle.rs` to preserve last-valid preview state when current source is invalid.
- Modify: `zircon_editor/src/ui/asset_editor/session/presentation_state.rs` and `zircon_editor/src/ui/asset_editor/presentation.rs` to project shell/tool status.
- Modify: `zircon_editor/src/ui/asset_editor/command.rs` to add reload, revert, open asset browser, resize slot, and preview-interact commands.
- Modify: `zircon_editor/src/ui/host/asset_editor_sessions/refresh.rs` to route reload/keep-local/diff/revert actions without duplicating watcher logic.
- Create: `zircon_editor/src/tests/ui/ui_asset_editor/designer_tools.rs`.
- Create: `zircon_editor/src/tests/ui/ui_asset_editor/emergency_shell.rs`.

### M22 Runtime/Editor Dual-Host Parity

- Create: `zircon_editor/src/tests/ui/ui_asset_editor/runtime_editor_parity.rs` for parity fixtures that need editor projection access.
- Create: `zircon_editor/src/tests/support/ui_parity.rs` only if repeated parity extraction helpers exceed one test file.
- Modify: `zircon_editor/src/tests/ui/ui_asset_editor/mod.rs` to register parity tests.
- Modify: `docs/ui-and-layout/ui-asset-documents-and-editor-protocol.md` and `docs/ui-and-layout/shared-ui-template-runtime.md` after parity evidence is available.

## Milestone 1: Editor Contract Productization

**Goal:** Convert runtime component-contract correctness into stable UI Asset Editor diagnostics and source-outline targets.

- [x] Add `UiComponentContractDiagnosticCode` with variants `InvalidPublicPart`, `PrivateSelector`, `ApiMismatch`, `ClosedRootClass`, `PrivateBindingTarget`, and `PrivateFocusTarget`.
- [x] Add `UiComponentContractDiagnostic { code, message, path, target_node_id, target_control_id }` in `component_contract/diagnostic.rs`.
- [x] In `component_contract/validation.rs`, keep existing `UiAssetError::InvalidDocument` behavior but route detail generation through the structured diagnostic type.
- [x] Add editor `UiAssetEditorDiagnostic { code, severity, message, source_path, target_node_id, target_control_id }` in `zircon_editor/src/ui/asset_editor/diagnostics/mod.rs`.
- [x] Store `structured_diagnostics: Vec<UiAssetEditorDiagnostic>` in `UiAssetEditorSession` while preserving `diagnostics: Vec<String>` for compatibility with existing presentation code.
- [x] Add source-outline mapping so a diagnostic with `target_node_id` selects the matching outline node when available.
- [x] Add editor tests asserting private selector, API mismatch, and closed root class produce stable codes and target metadata.
- [x] Add editor session and component-adapter tests for root-class policy projection, editing, undo/redo, descriptor exposure, and adapter commit routing.
- [x] Testing stage:
  - `rustfmt --edition 2021 --check zircon_runtime/src/ui/template/asset/component_contract/*.rs zircon_editor/src/ui/asset_editor/diagnostics/*.rs zircon_editor/src/ui/asset_editor/session/*.rs zircon_editor/src/tests/ui/ui_asset_editor/contract_diagnostics.rs`
  - `cargo test -p zircon_runtime --lib asset_component_contract --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-productization-contract --message-format short --color never`
  - `cargo test -p zircon_editor --lib contract_diagnostics --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-productization-contract --message-format short --color never`
  - `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-productization-contract --message-format short --color never`

### Milestone 1 Evidence: 2026-05-01 22:26 +08:00

- `rustfmt --edition 2021 --check --config skip_children=true zircon_runtime/src/ui/template/asset/component_contract/mod.rs zircon_runtime/src/ui/template/asset/component_contract/diagnostic.rs zircon_runtime/src/ui/template/asset/component_contract/validation.rs zircon_runtime/src/ui/template/asset/mod.rs zircon_runtime/src/ui/template/mod.rs zircon_runtime/src/ui/tests/asset_component_contract.rs zircon_editor/src/ui/asset_editor/diagnostics/mod.rs zircon_editor/src/ui/asset_editor/diagnostics/contract.rs zircon_editor/src/ui/asset_editor/session/ui_asset_editor_session.rs zircon_editor/src/ui/asset_editor/session/lifecycle.rs zircon_editor/src/ui/asset_editor/session/presentation_state.rs zircon_editor/src/ui/asset_editor/mod.rs zircon_editor/src/ui/asset_editor/presentation.rs zircon_editor/src/tests/ui/ui_asset_editor/mod.rs zircon_editor/src/tests/ui/ui_asset_editor/contract_diagnostics.rs` passed with no output. `skip_children=true` was required to avoid formatting active M15 child modules.
- `git diff --check -- ...` over the same source files passed with only repository LF-to-CRLF notices.
- `cargo test -p zircon_runtime --lib asset_component_contract --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-productization-contract --message-format short --color never` passed, `15 passed; 0 failed; 611 filtered out`, with unrelated runtime graphics warnings.
- `cargo test -p zircon_editor --lib contract_diagnostics --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-productization-contract --message-format short --color never` passed, `3 passed; 0 failed; 877 filtered out`, after an earlier cold compile timeout; output still includes unrelated runtime graphics warnings and one unrelated editor test unused-import warning.
- `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-productization-contract --message-format short --color never` passed, with unrelated runtime graphics warnings.
- This evidence closes the diagnostic/productization slice in this plan.
- 2026-05-02 root-class authoring follow-up evidence closed the remaining Milestone 1 UX gap on `E:\cargo-targets\zircon-ui-m10-root-class-authoring`: focused tests `ui_asset_editor_session_projects_and_updates_root_class_policy`, `asset_editor_component_adapter_updates_selected_component_root_class_policy`, `editor_component_adapter_registry_advertises_reflection_and_asset_editor_sources`, and `ui_asset_editor_host_genericizes_detail_event_dispatch` passed; `cargo test -p zircon_editor --lib ui_asset_editor --locked --jobs 1` passed with `204 passed; 0 failed; 675 filtered out`. Broader `cargo check -p zircon_editor --lib --locked` and `cargo test -p zircon_runtime --lib asset_component_contract --locked` were blocked before compilation by unrelated runtime-interface manifest/lock drift, so broad workspace green is still not claimed.

## Milestone 2: M18 Runtime Binding Semantics

**Goal:** Move binding expression and target validation into runtime-owned asset semantics.

- [x] Define `UiBindingExpression` as `Literal(UiValue)`, `ParamRef(String)`, `PropRef(String)`, `Equals`, `NotEquals`, `And`, `Or`, and `Not`.
- [x] Define `UiBindingTargetSchema` for `Prop`, `Class`, `Visibility`, `Enabled`, and `ActionPayload` targets with expected `UiValueKind`.
- [x] Define `UiBindingDiagnosticCode::{InvalidTarget, InvalidValueKind, UnresolvedRef, UnsupportedOperator}`.
- [x] Add `validate_asset_bindings(document, component_registry)` and run it during `UiDocumentCompiler::compile(...)` after document shape/localization/component contract validation.
- [x] Keep expression evaluation preview-only; runtime compiler validates shape and static references, not arbitrary execution.
- [x] Update editor binding inspector/schema projection to display runtime diagnostic codes and retain existing authoring controls after runtime-interface M2 editor cutover releases overlapping editor DTO files.
- [x] Add runtime tests for valid prop binding, invalid target, invalid value kind, unresolved ref, unsupported operator, descriptor-owned prop authority, missing action-payload target rejection, malformed single-character operator diagnostics, and compiler precondition integration.
- [x] Add editor tests for binding diagnostic projection and preview mock compatibility after the runtime-interface/editor cutover blocker clears.
- [x] Testing stage:
  - `rustfmt --edition 2021 --check <M18 binding runtime files>` passed with no output.
  - `cargo test -p zircon_runtime --test ui_asset_binding_contract --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-binding-m18 --message-format short --color never` passed, `16 passed; 0 failed`.
  - `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-binding-m18 --message-format short --color never` passed with unrelated graphics/plugin warnings.
  - `cargo test -p zircon_runtime --lib asset_binding --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-binding-m18 --message-format short --color never` remains blocked before filtered tests by unrelated lib-test UI DTO type-identity errors.
  - `cargo test -p zircon_runtime --lib ui::tests::asset --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-binding-m18 --message-format short --color never -- --nocapture` remains blocked by the same unrelated lib-test compile blocker.

### Editor Projection Evidence: 2026-05-02 11:08 +08:00

- `rustfmt --edition 2021 --check zircon_runtime/src/ui/template/asset/binding/diagnostic.rs zircon_editor/src/ui/asset_editor/diagnostics/mod.rs zircon_editor/src/ui/asset_editor/diagnostics/binding.rs zircon_editor/src/ui/asset_editor/session/lifecycle.rs zircon_editor/src/ui/asset_editor/binding/schema_projection.rs zircon_editor/src/tests/ui/ui_asset_editor/mod.rs zircon_editor/src/tests/ui/ui_asset_editor/binding_semantics.rs` passed with no output.
- `git diff --check -- zircon_runtime/src/ui/template/asset/binding/diagnostic.rs zircon_editor/src/ui/asset_editor/diagnostics/mod.rs zircon_editor/src/ui/asset_editor/diagnostics/binding.rs zircon_editor/src/ui/asset_editor/session/lifecycle.rs zircon_editor/src/ui/asset_editor/binding/schema_projection.rs zircon_editor/src/tests/ui/ui_asset_editor/mod.rs zircon_editor/src/tests/ui/ui_asset_editor/binding_semantics.rs` passed with only LF-to-CRLF notices.
- `cargo test -p zircon_editor --lib ui_asset_editor_projects_runtime_binding_diagnostic_and_schema_items --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-interface-big-cutover --message-format short --color never` passed, `1 passed; 0 failed; 888 filtered out`.
- `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-interface-big-cutover --message-format short --color never` was blocked before compilation by unrelated workspace lock drift: Cargo selected `web-sys 0.3.97` requiring `js-sys 0.3.97`, while `Cargo.lock` selected `js-sys 0.3.95`; active Cargo/Rustc jobs were also present, so this M18 editor projection slice did not mutate the lockfile.
- Editor projection now maps interface-owned `UiBindingDiagnostic` values produced by runtime validation into `UiAssetEditorDiagnostic`, preserves `target_binding_id`, and shows target-assignment diagnostics in the binding inspector schema rows without creating an editor-owned binding semantic fork. After the UI runtime-interface hard cutover, neutral M18 binding target/expression/diagnostic/report declarations are canonical under `zircon_runtime_interface::ui::template::asset::binding`, while `zircon_runtime::ui::template::asset::binding` keeps validation behavior.

## Milestone 3: M5/M24 Recovery And Emergency Shell

**Goal:** Make invalid source and external conflict flows recoverable through explicit session state and editor commands.

- [ ] Define `UiAssetEditorShellState::{Valid, StaleExternal, InvalidSource, Emergency}`.
- [ ] Preserve `last_valid_document`, `last_valid_compiled`, and preview projection when current source fails to compile.
- [ ] Project emergency shell state to existing `EmergencyShellPanel` authored nodes without adding Slint source files.
- [ ] Add commands for reload external, keep local, open diff, revert to last valid, and open asset browser.
- [ ] Route commands through existing session/host manager paths instead of duplicating file watcher policy.
- [ ] Add tests for invalid source -> emergency, emergency -> revert, external conflict -> reload, and external conflict -> keep local.
- [ ] Testing stage:
  - `cargo test -p zircon_editor --lib emergency_shell --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-editor-recovery --message-format short --color never`
  - `cargo test -p zircon_editor --lib ui_asset_workspace_watcher --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-editor-recovery --message-format short --color never`
  - `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-editor-recovery --message-format short --color never`

## Milestone 4: M6 Designer Canvas Tools

**Goal:** Add behavior to authored designer tool rows: slot resize and preview interaction dispatch.

- [ ] Define `UiAssetDesignerToolMode::{Select, ResizeSlot, PreviewInteract}`.
- [ ] Add session command to switch tool mode and project the active mode back into `DesignerToolModeRow`.
- [ ] Add slot resize transaction input `{ node_id, slot, delta_x, delta_y }` and mutate the selected child mount/layout metadata through existing undo/redo command paths.
- [ ] Add preview interaction dispatch that maps a projected preview hit to existing Runtime UI action/component adapter paths when the active mode is `PreviewInteract`.
- [ ] Add tests for tool-mode projection, resize transaction dirty state, undo/redo, and preview interaction dispatch log.
- [ ] Testing stage:
  - `cargo test -p zircon_editor --lib designer_tools --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-designer-m6 --message-format short --color never`
  - `cargo test -p zircon_editor --lib ui_asset_editor_bootstrap --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-designer-m6 --message-format short --color never -- --nocapture`
  - `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-designer-m6 --message-format short --color never`

## Milestone 5: M22 Runtime/Editor Dual-Host Parity

**Goal:** Prove representative `.ui.toml` assets keep equivalent layout-critical and event-critical semantics in runtime extract and editor projection.

- [ ] Pick fixtures: UI Asset Editor shell, component showcase subset, and one runtime HUD fixture.
- [ ] Build one compile path through `UiAssetLoader -> UiDocumentCompiler -> UiTemplateSurfaceBuilder`.
- [ ] Extract editor projection facts: node ids, control ids, bindings, slots, component ids, virtualization metadata, and world-space metadata.
- [ ] Extract runtime surface facts from the same compiled document: node ids, bindings, layout attributes, and render/extract metadata available without RHI.
- [ ] Compare only shared semantics; record intentional editor-only and runtime-only divergences in the test fixture.
- [ ] Add event parity tests for `ValueChanged`, `Commit`, action route, and component adapter mutation result where runtime/editor both expose the route.
- [ ] Testing stage:
  - `cargo test -p zircon_editor --lib runtime_editor_parity --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-dual-host-parity --message-format short --color never`
  - `cargo test -p zircon_runtime --lib ui::tests::asset --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-dual-host-parity --message-format short --color never -- --nocapture`
  - `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-dual-host-parity --message-format short --color never`
  - `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-dual-host-parity --message-format short --color never`

## Documentation And Archive Stage

- [ ] Update `docs/ui-and-layout/ui-asset-documents-and-editor-protocol.md` with M18 and M22 semantics.
- [ ] Update `docs/ui-and-layout/ui-asset-foundation-descriptors-contracts-invalidation.md` if binding/resource/package inputs affect cache or invalidation behavior.
- [ ] Create or update focused docs for editor diagnostics/recovery if the implementation creates new modules under `zircon_editor/src/ui/asset_editor/diagnostics` or `session`.
- [ ] Update `.codex/plans/UI Asset Editor 与共享 Layout 未完成内容归档.md` only after each milestone has focused evidence.
- [ ] Archive active session notes when their milestone closes.

## Plan Self-Review

- Coverage: this plan intentionally starts after M16 and M15 foundation work and covers Editor contract productization, M18, M5/M24, M6, and M22.
- Placeholder scan: no task uses placeholder implementation names; every milestone names concrete files, types, tests, and commands.
- Type consistency: runtime binding and diagnostic names are stable across milestones; editor DTOs consume runtime-owned diagnostics rather than creating a parallel semantic authority.
