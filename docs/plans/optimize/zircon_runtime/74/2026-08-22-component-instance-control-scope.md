# Runtime74 Component Instance Control Scope

Plan: docs/plans/optimize/zircon_runtime/74-runtime-ui-template-component-binding-expression-model-event-command-hot-reload-product-integration-review.md
Milestone: M0
Status: validation_pending
Files: ["docs/plans/optimize/zircon_runtime/74/2026-08-22-component-instance-control-scope.md","docs/zircon_runtime/ui/template/component_control_scope.md","docs/zircon_runtime/ui/template/pipeline.md","zircon_runtime_interface/src/tests/contracts.rs","zircon_runtime_interface/src/ui/template/asset/binding/expression.rs","zircon_runtime_interface/src/ui/template/asset/compiler/package/header.rs","zircon_runtime/src/ui/surface/control_index.rs","zircon_runtime/src/ui/surface/surface/pointer_component_events.rs","zircon_runtime/src/ui/template/asset/compiler/binding_param_resolver.rs","zircon_runtime/src/ui/template/asset/compiler/compile.rs","zircon_runtime/src/ui/template/asset/compiler/component_instance_expander.rs","zircon_runtime/src/ui/template/asset/compiler/control_scope.rs","zircon_runtime/src/ui/template/asset/compiler/mod.rs","zircon_runtime/src/ui/template/asset/compiler/node_expander.rs","zircon_runtime/src/ui/template/asset/compiler/prototype_instancer.rs","zircon_runtime/src/ui/tests/asset_binding.rs","zircon_runtime/src/ui/tests/asset_binding/control_scope.rs","zircon_runtime/src/ui/tests/asset_prototype_store.rs","zircon_runtime/src/ui/tests/asset_prototype_store/control_scope.rs"]

- Date: 2026-08-22
- Owner: `optimize-runtime74-param-ref-compile-r3-bee4c707-20260822`
- Source item: `RTB-P0-004` instance-scope correctness slice
- Delivery state: implementation complete; grouped coordinator validation pending

## Problem

Repeated component instances retained the same definition-local control ids. Runtime
`control.X.prop.Y` evaluation then selected the smallest matching node id across the surface, so a
row, menu item, inspector field, or virtualized cell could read another instance's value.

## Scope Delivered

- A compiler-owned scope derives deterministic private control identities from the nested component
  call path and local control id.
- Root aliases preserve caller-visible instance controls; nested definitions compose child scopes;
  instance-node bindings and slot fills retain caller scope.
- Target expressions and action payloads rewrite `ControlPropRef` leaves in both recursive and flat
  prototype compilers.
- The expression probe ignores quoted `"control.X.prop.Y"` preview text.
- Compiled templates fail closed on remaining duplicate control ids.
- Runtime action evaluation uses only the unique incremental control index. The smallest-id lookup
  and its compatibility tests are removed.
- Compiler artifact schema version `3` prevents reuse of pre-scope artifacts.

## Deterministic Performance Evidence

The scale gate expands 1,000 component instances and requires:

- `unique_scoped_control_ids=1000`;
- `resolved_control_refs=1000`;
- `global_duplicate_fallbacks=0`;
- debug-test compile/qualification wall time below five seconds.

The marker is `PERF-RUNTIME74-CONTROL-SCOPE`. The coordinator validator will relay its measured
`elapsed_us`; no measured value is claimed before that grouped run passes.

## TDD And Static Evidence

- Repeated instances resolve their own values and action payloads.
- A binding authored on the instance node continues to read a caller-scope control.
- Nested component scopes compose without private-control penetration.
- Flat prototypes use the same qualification semantics.
- Seven control-index regressions lock duplicate fail-close behavior and incremental synchronization.
- `rustfmt +1.94.1` syntax/format check and scoped `git diff --check` complete.
- Cargo and grouped external validation are pending. No Cargo pass is claimed.

## Remaining Scope

This slice removes the live smallest-node correctness failure and establishes the instance identity
contract. Generation-qualified durable endpoint handles across tree replacement remain coupled to
the compiled binding program and transactional hot-reload publication work; that later boundary is
not claimed complete here.
