# Runtime74 ParamRef Compile-Time Resolution

Plan: docs/plans/optimize/zircon_runtime/74-runtime-ui-template-component-binding-expression-model-event-command-hot-reload-product-integration-review.md
Milestone: M0
Status: validation_pending
Files: ["docs/plans/optimize/zircon_runtime/74/2026-08-22-param-ref-compile-time-resolution.md","docs/zircon_runtime/ui/template/pipeline.md","zircon_runtime_interface/src/tests/contracts.rs","zircon_runtime_interface/src/ui/template/asset/binding/expression.rs","zircon_runtime_interface/src/ui/template/asset/compiler/package/header.rs","zircon_runtime/src/ui/template/asset/binding/mod.rs","zircon_runtime/src/ui/template/asset/binding/validation.rs","zircon_runtime/src/ui/template/asset/compiler/binding_param_resolver.rs","zircon_runtime/src/ui/template/asset/compiler/component_instance_expander.rs","zircon_runtime/src/ui/template/asset/compiler/mod.rs","zircon_runtime/src/ui/template/asset/compiler/prototype_instancer.rs","zircon_runtime/src/ui/tests/asset_binding.rs","zircon_runtime/src/ui/tests/asset_prototype_store.rs"]

- Date: 2026-08-22
- Owner: `optimize-runtime74-param-ref-compile-r3-bee4c707-20260822`
- Source item: `RTB-P0-002`
- Delivery state: implementation complete; grouped coordinator validation pending

## Problem

The asset binding validator accepted `param.*` expressions and inferred their declared component
types, but both component expansion paths copied the authored binding unchanged. The gameplay and
Editor action evaluators have no component-instance parameter scope and therefore returned `None`,
silently dropping an otherwise valid action after compilation and package publication.

## Scope Delivered

- One compiler-owned resolver substitutes component parameters in binding target expressions and
  action payload expressions before a `UiTemplateNode` is retained.
- Default, override, and nested component values converge on the same resolved parameter map already
  used for component props and layout.
- Fully constant payload expressions use typed TOML when TOML preserves the `UiValue` kind. Semantic
  string kinds, vectors, flags, and null use canonical typed literal constructors so artifact
  round-trips and later evaluator parses retain the declared kind. Constant comparison and boolean
  branches are folded; mixed expressions keep dynamic `prop.*` and `control.*` references.
- Parameter kinds reuse the validator's canonical schema mapping, including numeric widening such as
  an authored integer default for a `float` parameter becoming a compiled TOML float.
- Both recursive document expansion and flat prototype expansion call the same resolver. Bindings
  authored on a component instance are resolved in the caller's parameter scope.
- Missing referenced parameters fail compilation explicitly instead of surviving into an evaluator
  that returns `None`.
- Compiler schema version `3` prevents persistent compiler-v1/v2 artifacts from bypassing parameter
  resolution or the subsequent component-control qualification semantics.
- The compiled binary artifact round-trip test verifies that no real `param.*` expression remains and
  that scalar, semantic string, vector, and flags values survive serialization and expression
  reparsing.

## Deterministic Performance Evidence

The performance contract for this correctness slice is structural:

- parameter-only action payload: event-time expression parses `1 -> 0` per dispatch;
- parameter-only action payload: event-time failed `ParamRef` resolution attempts `1 -> 0` per dispatch;
- nested constant comparison: event-time comparison evaluations `1 -> 0` per dispatch;
- compiled artifact retained component parameter references: `1+ -> 0` for the covered binding;
- ordinary bindings without `param.*` retain their authored source and do not add an event-time path.

No wall-clock speedup is claimed. The grouped validator reports these operation-count reductions as
structural performance evidence; Runtime74 scale and P95 gates remain open for later compiled target
program and model transaction milestones.

## TDD And Static Evidence

- `param_ref_compile_resolves_nested_params_and_artifact_roundtrip` covers a caller override flowing
  through a nested component, an instance-owned caller-scope binding, a component default, typed
  float widening, typed color/vector/flags literals, escaped control characters, constant folding,
  a mixed dynamic expression, and binary artifact serialization.
- `param_ref_compile_rejects_a_missing_referenced_component_param` locks the fail-closed compile path.
- `param_ref_compile_resolves_prototype_binding_params` locks parity for the flat prototype compiler.
- `param_ref_compile_preserves_non_param_preview_expressions` locks compatibility with Editor-only
  expression dialects such as `concat(...)`, including quoted `"param.title"` text inside an actual
  component expansion path.
- `typed_binding_literal_parser_preserves_supported_value_kinds_and_escapes` is a table-driven
  interface contract for all eight typed constructors, empty/non-empty flags, and every supported
  string escape.
- `typed_binding_literal_param_probe_requires_a_path_root` distinguishes a real root `param.*` path
  from quoted text and from a nested segment such as `control.param.prop.value`.
- `binding_compile_schema_version_invalidates_unresolved_or_unscoped_artifacts` locks the
  persistent cache boundary to compiler schema version `3`.
- `rustfmt +1.94.1` and scoped `git diff --check` complete.
- Focused Cargo tests and grouped external validation are pending. No Cargo pass is claimed.

## Remaining Scope

This closes only `RTB-P0-002` for compiled asset and prototype product paths. Target assignment
execution, typed component event identity, instance-qualified control references, model/command
transactions, generation-qualified subscriptions, and transactional hot reload remain open.
