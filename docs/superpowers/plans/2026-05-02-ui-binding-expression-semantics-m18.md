# UI Binding Expression Semantics M18 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add runtime-owned UI asset binding target schema, restricted expression AST, static validation, compiler precondition wiring, and editor diagnostic/schema projection for M18.

**Architecture:** Implement a folder-backed `zircon_runtime::ui::template::asset::binding` module. Keep `UiBindingRef` as the serialized event/action owner, add optional target assignments, validate through `UiDocumentCompiler` before expansion, and expose structured runtime diagnostics that the editor consumes by projection only.

**Tech Stack:** Rust 2021, Serde/TOML, existing `UiAssetDocument`, `UiBindingRef`, `UiComponentDescriptorRegistry`, `UiValueKind`, and Cargo focused runtime tests.

---

## Execution Boundary

- Stay on `main`; do not create worktrees or feature branches.
- Do not touch graphics/plugin renderer paths.
- Keep editor behavior changes projection-only; do not add editor-owned binding semantics.
- Do not introduce arbitrary script evaluation, host calls, or action side-effect policy.
- Defer Cargo build/test execution to the testing stage unless a syntax/type blocker requires earlier evidence.

## File Structure

- Create: `zircon_runtime/src/ui/template/asset/binding/mod.rs` for public binding exports.
- Create: `zircon_runtime/src/ui/template/asset/binding/expression.rs` for `UiBindingExpression`, parser, and expression type inference helpers.
- Create: `zircon_runtime/src/ui/template/asset/binding/target.rs` for `UiBindingTargetKind`, `UiBindingTarget`, `UiBindingTargetAssignment`, and `UiBindingTargetSchema`.
- Create: `zircon_runtime/src/ui/template/asset/binding/diagnostic.rs` for diagnostic code/severity/report types.
- Create: `zircon_runtime/src/ui/template/asset/binding/validation.rs` for document traversal and compiler-facing validation.
- Modify: `zircon_runtime/src/ui/template/document.rs` to add optional `targets` to `UiBindingRef`.
- Modify: `zircon_runtime/src/ui/template/asset/mod.rs` and `zircon_runtime/src/ui/template/mod.rs` to re-export runtime binding types.
- Modify: `zircon_runtime/src/ui/template/asset/compiler/ui_document_compiler.rs` only if the compiler needs a registry accessor.
- Modify: `zircon_runtime/src/ui/template/asset/compiler/compile.rs` to run `validate_asset_bindings(...)` after shape/component-contract validation.
- Create: `zircon_runtime/src/ui/tests/asset_binding.rs` for focused M18 lib-test coverage.
- Create: `zircon_runtime/tests/ui_asset_binding_contract.rs` for public integration coverage that can run while unrelated lib-test modules are blocked.
- Modify: `zircon_runtime/src/ui/tests/mod.rs` to register `asset_binding`.
- Update: `docs/ui-and-layout/ui-asset-documents-and-editor-protocol.md` and `docs/ui-and-layout/ui-asset-foundation-descriptors-contracts-invalidation.md` with M18 module ownership and validation evidence.
- Update: `.codex/sessions/20260502-0137-ui-binding-m18-design.md` as live coordination state.
- Modify: `zircon_runtime/src/ui/template/asset/binding/diagnostic.rs` to expose stable diagnostic string codes for editor projection.
- Create: `zircon_editor/src/ui/asset_editor/diagnostics/binding.rs` for runtime binding diagnostic mapping.
- Modify: `zircon_editor/src/ui/asset_editor/diagnostics/mod.rs`, `zircon_editor/src/ui/asset_editor/session/lifecycle.rs`, and `zircon_editor/src/ui/asset_editor/binding/schema_projection.rs` to retain target binding ids, refresh runtime binding diagnostics, and render binding target diagnostic rows.
- Create: `zircon_editor/src/tests/ui/ui_asset_editor/binding_semantics.rs` for focused editor projection coverage.

## Milestone 1: Runtime Binding Schema And Parser

**Goal:** Define the runtime-owned serialized target assignment model and restricted expression AST without compiler behavior changes yet.

- [x] Add `UiBindingTargetKind` with `prop`, `class`, `visibility`, `enabled`, and `action_payload` variants.
- [x] Add `UiBindingTarget`, `UiBindingTargetAssignment`, and `UiBindingTargetSchema` with serde support for compact TOML target tables.
- [x] Add `UiBindingExpression` with literal, param ref, prop ref, equality, inequality, boolean `and`, boolean `or`, and boolean `not` variants.
- [x] Add an expression parser that accepts optional leading `=`, string/bool/int/float/null literals, `param.<name>`, `prop.<name>`, `==`, `!=`, `&&`, `||`, `!`, and parentheses.
- [x] Add structured diagnostics and report types for invalid target, invalid value kind, unresolved ref, and unsupported operator.
- [x] Re-export the new types through `asset/mod.rs` and `template/mod.rs`.

## Milestone 2: Static Validation And Compiler Wiring

**Goal:** Make runtime reject invalid binding target/expression semantics before asset expansion.

- [x] Add `collect_asset_binding_report(document, registry)` to walk document root and component node trees.
- [x] Validate target names and expected kinds: prop descriptor/authored prop, class bool, visibility bool, enabled bool, action payload field.
- [x] Infer expression kinds from literals, component params, node props, boolean operators, and equality operators.
- [x] Validate preview-style action payload expression strings that start with `=` without treating literal strings as expressions.
- [x] Add `validate_asset_bindings(document, registry) -> Result<(), UiAssetError>` that returns the first error diagnostic through `UiAssetError::InvalidDocument`.
- [x] Wire `validate_asset_bindings(...)` into `UiDocumentCompiler::validate_compiler_preconditions(...)` after document shape and component-contract validation.
- [x] Add the optional `targets` field to `UiBindingRef` and update unavoidable struct literals with empty targets.

## Milestone 3: Focused Tests And Docs

**Goal:** Prove the M18 behavior and record the runtime-owned contract.

- [x] Add tests for a valid prop binding expression against the descriptor registry.
- [x] Add tests for valid class, visibility, enabled, and action payload target schemas.
- [x] Add tests for invalid target, invalid value kind, unresolved ref, unsupported operator diagnostics, descriptor-authority unknown prop rejection, missing action-payload target rejection, malformed operator classification, and boolean operator/parenthesis parsing.
- [x] Add a compiler precondition test proving invalid binding semantics fail during compile.
- [x] Update UI asset docs with the new binding owner, serialized examples, diagnostic codes, and validation commands.
- [x] Update the active M18 session note with touched modules, blockers, and testing-stage state.

## Milestone 4: Editor Diagnostic Projection

**Goal:** Let the UI Asset Editor consume runtime binding diagnostics without adding an editor-only semantic model.

- [x] Add a stable `UiBindingDiagnosticCode::as_str()` helper for diagnostic row projection.
- [x] Map runtime `UiBindingDiagnostic` into `UiAssetEditorDiagnostic`, preserving code, severity, source path, target node id, and target binding id.
- [x] Refresh binding diagnostics during preview compile failure alongside existing component-contract diagnostics.
- [x] Project authored binding target assignments and matching target diagnostics into inspector schema rows.
- [x] Add focused editor coverage for `invalid_value_kind` projection and schema diagnostic display.

## Testing Stage

- [x] Run `rustfmt --edition 2021 --check zircon_runtime/src/ui/template/document.rs zircon_runtime/src/ui/template/asset/mod.rs zircon_runtime/src/ui/template/mod.rs zircon_runtime/src/ui/template/asset/binding/*.rs zircon_runtime/src/ui/template/asset/compiler/compile.rs zircon_runtime/src/ui/template/asset/compiler/ui_document_compiler.rs zircon_runtime/src/ui/tests/mod.rs zircon_runtime/src/ui/tests/asset_binding.rs zircon_runtime/tests/ui_asset_binding_contract.rs` (passed; no output after formatting).
- [x] Run `cargo test -p zircon_runtime --test ui_asset_binding_contract --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-binding-m18 --message-format short --color never` (passed after review fixes: 16 tests, 0 failed).
- [x] Run `cargo test -p zircon_runtime --lib asset_binding --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-binding-m18 --message-format short --color never` (blocked before filtered tests by unrelated lib-test UI DTO type-identity errors in asset font and graphics UI renderer tests).
- [x] Run `cargo test -p zircon_runtime --lib ui::tests::asset --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-binding-m18 --message-format short --color never -- --nocapture` (same unrelated lib-test compile blocker before filtered tests).
- [x] Run `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-binding-m18 --message-format short --color never` (passed with unrelated graphics/plugin warnings).
- [x] Diagnose the failed lib-test commands as a lower compile-harness blocker already recorded by active runtime-interface/UI sessions; do not patch graphics/runtime-interface/editor cutover areas from this M18 lane.
- [x] Run `rustfmt --edition 2021 --check zircon_runtime/src/ui/template/asset/binding/diagnostic.rs zircon_editor/src/ui/asset_editor/diagnostics/mod.rs zircon_editor/src/ui/asset_editor/diagnostics/binding.rs zircon_editor/src/ui/asset_editor/session/lifecycle.rs zircon_editor/src/ui/asset_editor/binding/schema_projection.rs zircon_editor/src/tests/ui/ui_asset_editor/mod.rs zircon_editor/src/tests/ui/ui_asset_editor/binding_semantics.rs` (passed with no output).
- [x] Run `git diff --check -- zircon_runtime/src/ui/template/asset/binding/diagnostic.rs zircon_editor/src/ui/asset_editor/diagnostics/mod.rs zircon_editor/src/ui/asset_editor/diagnostics/binding.rs zircon_editor/src/ui/asset_editor/session/lifecycle.rs zircon_editor/src/ui/asset_editor/binding/schema_projection.rs zircon_editor/src/tests/ui/ui_asset_editor/mod.rs zircon_editor/src/tests/ui/ui_asset_editor/binding_semantics.rs` (passed with only LF-to-CRLF notices).
- [x] Run `cargo test -p zircon_editor --lib ui_asset_editor_projects_runtime_binding_diagnostic_and_schema_items --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-interface-big-cutover --message-format short --color never` (passed: `1 passed; 0 failed; 888 filtered out`).
- [x] Run `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-interface-big-cutover --message-format short --color never` (blocked before compilation by unrelated workspace lock drift: `web-sys 0.3.97` requires `js-sys 0.3.97`, while `Cargo.lock` selects `js-sys 0.3.95`; active Cargo/Rustc jobs were present, so this M18 editor projection slice did not mutate the lockfile).

## Acceptance Evidence

- Runtime owns binding target/expression validation behavior under `zircon_runtime::ui::template::asset::binding`, while neutral target/expression/diagnostic/report DTO declarations are canonical under `zircon_runtime_interface::ui::template::asset::binding` after the UI runtime-interface hard cutover.
- Compiler preconditions reject invalid binding targets, invalid value kinds, unresolved refs, and unsupported operators before expansion.
- Review follow-up tightened descriptor-owned prop authority, missing action-payload target rejection, and malformed single-character operator diagnostics.
- Public M18 integration contract tests pass in focused runtime scope.
- Focused editor projection test passes for runtime binding diagnostic mapping and target-schema diagnostic rows.
- Existing UI asset lib-test filters were originally blocked before test execution by unrelated UI DTO type-identity compile errors. The runtime-interface cutover later cleared the focused `asset_binding` execution blocker; the active UI runtime-interface session now owns final DTO hard-cutover acceptance and post-move validation.
- Scoped editor type-check is currently blocked before Rust compilation by unrelated workspace lock drift between `web-sys` and `js-sys`; no editor/workspace green claim is made.
- Docs and session state list exact implementation files and validation commands.
- No workspace-wide green claim is made unless broad validation is explicitly run and passes.
