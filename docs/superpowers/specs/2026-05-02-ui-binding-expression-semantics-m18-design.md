---
related_code:
  - zircon_runtime_interface/src/ui/template/asset/binding/mod.rs
  - zircon_runtime_interface/src/ui/template/asset/binding/expression.rs
  - zircon_runtime_interface/src/ui/template/asset/binding/target.rs
  - zircon_runtime_interface/src/ui/template/asset/binding/diagnostic.rs
  - zircon_runtime/src/ui/template/document.rs
  - zircon_runtime/src/ui/template/asset/mod.rs
  - zircon_runtime/src/ui/template/asset/document.rs
  - zircon_runtime/src/ui/template/asset/compiler/compile.rs
  - zircon_runtime/src/ui/template/asset/compiler/ui_document_compiler.rs
  - zircon_runtime/src/ui/component/descriptor/prop_schema.rs
  - zircon_runtime/src/ui/component/value.rs
implementation_files:
  - zircon_runtime_interface/src/ui/template/asset/binding/mod.rs
  - zircon_runtime_interface/src/ui/template/asset/binding/expression.rs
  - zircon_runtime_interface/src/ui/template/asset/binding/target.rs
  - zircon_runtime_interface/src/ui/template/asset/binding/diagnostic.rs
  - zircon_runtime/src/ui/template/asset/binding/mod.rs
  - zircon_runtime/src/ui/template/asset/binding/validation.rs
  - zircon_runtime/src/ui/template/document.rs
  - zircon_runtime/src/ui/template/asset/compiler/compile.rs
  - zircon_runtime/tests/ui_asset_binding_contract.rs
plan_sources:
  - user: 2026-05-02 approve M18 runtime binding expression semantics and execute locally
  - .codex/plans/UI 后续产品化与验证归档计划.md
  - docs/superpowers/plans/2026-05-01-ui-productization-editor-binding-parity.md
tests:
  - zircon_runtime/src/ui/tests/asset_binding.rs
  - zircon_runtime/tests/ui_asset_binding_contract.rs
  - cargo test -p zircon_runtime --test ui_asset_binding_contract --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-binding-m18 --message-format short --color never
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-binding-m18 --message-format short --color never
doc_type: milestone-detail
---

# UI Binding Expression Semantics M18 Design

## Goal

M18 moves UI asset binding expressions from editor-only preview convention into runtime-owned asset semantics. Runtime owns the restricted expression AST, target schema, static diagnostics, and compiler precondition hook; editor remains an authoring and preview consumer.

## Ownership

`zircon_runtime::ui::template::asset::binding` owns the M18 binding model. The module lives under the UI asset subsystem because the schema is part of serialized `.ui.toml` asset validity, not a general scripting VM or editor-only helper.

After the UI runtime-interface hard cutover, the neutral target, expression, diagnostic, and report DTO declarations live under `zircon_runtime_interface::ui::template::asset::binding`. `zircon_runtime::ui::template::asset::binding` keeps validation behavior and imports those DTOs directly instead of preserving runtime-local declaration files.

`zircon_editor` may continue to preview deterministic mock values, but it must not become the authority for accepted operators, target kinds, or unresolved reference behavior. Action side-effect policy remains in `asset::action_policy` and is still reserved for M21.

## Data Model

Bindings keep the existing event/action route shape and gain optional target assignments:

```rust
pub struct UiBindingTargetAssignment {
    pub target: UiBindingTarget,
    pub expression: String,
}
```

`UiBindingTarget` uses a compact serialized schema with `kind` and optional `name`:

- `prop`: binds a component prop named by `name`.
- `class`: toggles a CSS-like class named by `name`.
- `visibility`: controls node visibility and expects a boolean expression.
- `enabled`: controls node enabled state and expects a boolean expression.
- `action_payload`: describes an action payload field named by `name`.

The source model intentionally stores expressions as strings while runtime exposes the parsed AST through `UiBindingExpression`. This keeps TOML authoring compact and avoids a prematurely verbose serialized tree while still giving runtime a canonical parser and validator.

## Expression Language

The restricted AST supports:

- literals: bool, int, float, string, null;
- `param.<name>` references to component parameter schemas;
- `prop.<name>` references to current-node component props;
- equality and inequality: `==`, `!=`;
- boolean combinators: `&&`, `||`, `!`;
- parenthesized subexpressions.

The parser also accepts a leading `=` so existing preview-style expression strings can be statically checked when they appear in action payload values. Arithmetic, method calls, mutation, assignment, host calls, arbitrary script execution, and side-effectful evaluation are out of scope.

## Static Validation

`validate_asset_bindings(document, registry)` walks root and component node trees before compilation expansion. For each binding it validates:

- target names are present where required and absent where invalid;
- prop targets exist in the runtime component descriptor when available, or as authored node props when no descriptor exists;
- class, visibility, and enabled targets evaluate to `Bool`;
- prop target expressions match the descriptor `UiValueKind`;
- `param` and `prop` references resolve in the current node/component context;
- unsupported operators produce stable diagnostics instead of falling through to preview behavior.

The compiler runs binding validation after document shape and component-contract validation and before expansion/cache key construction. That ordering keeps malformed binding semantics from being hidden by later resource, package, or cache behavior.

## Diagnostics

Runtime diagnostics are structured:

- `InvalidTarget`
- `InvalidValueKind`
- `UnresolvedRef`
- `UnsupportedOperator`

Each diagnostic records severity, source path, node id, binding id, and message. `validate_asset_bindings(...)` returns the first error as `UiAssetError::InvalidDocument` for existing compiler callers, while `collect_asset_binding_report(...)` exposes the full report for editor/productization consumers.

## Validation Scope

M18 validation is deterministic and side-effect-free. It does not evaluate live host state, call editor actions, route events, or enforce action side-effect policy. Preview fixtures can use the same parser and target report, but runtime acceptance depends only on asset source, component descriptors, local component parameter schemas, and authored node values.

## Acceptance

Focused public runtime integration tests prove valid prop/class/visibility/enabled/action-payload targets, descriptor-owned prop authority, missing action-payload target rejection, invalid target diagnostics, invalid value kind diagnostics, unresolved reference diagnostics, unsupported operator diagnostics, boolean operator/parentheses parsing, and compiler precondition integration. `cargo check -p zircon_runtime --lib` is the accepted production compile gate for this lane. The crate lib-test filters are currently blocked before M18 assertions run by unrelated UI DTO type-identity errors in asset font and graphics UI renderer test modules, so workspace-wide or full lib-test green remains out of scope while unrelated graphics/plugin and runtime-interface work is active.
