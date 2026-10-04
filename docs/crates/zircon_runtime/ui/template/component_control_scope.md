---
related_code:
  - zircon_runtime_interface/src/ui/template/asset/binding/expression.rs
  - zircon_runtime_interface/src/ui/template/asset/compiler/package/header.rs
  - zircon_runtime/src/ui/template/asset/compiler/control_scope.rs
  - zircon_runtime/src/ui/template/asset/compiler/node_expander.rs
  - zircon_runtime/src/ui/template/asset/compiler/component_instance_expander.rs
  - zircon_runtime/src/ui/template/asset/compiler/prototype_instancer.rs
  - zircon_runtime/src/ui/template/build/build_error.rs
  - zircon_runtime/src/ui/template/build/tree_builder.rs
  - zircon_runtime/src/ui/surface/control_index.rs
  - zircon_runtime/src/ui/surface/surface/pointer_component_events.rs
  - zircon_runtime/src/ui/tests/asset_binding/control_scope.rs
  - zircon_runtime/src/ui/tests/asset_prototype_store/control_scope.rs
  - zircon_runtime/src/ui/tests/template_tree_builder.rs
implementation_files:
  - zircon_runtime/src/ui/template/asset/compiler/control_scope.rs
  - zircon_runtime/src/ui/template/asset/compiler/node_expander.rs
  - zircon_runtime/src/ui/template/asset/compiler/component_instance_expander.rs
  - zircon_runtime/src/ui/template/asset/compiler/prototype_instancer.rs
  - zircon_runtime/src/ui/template/build/build_error.rs
  - zircon_runtime/src/ui/template/build/tree_builder.rs
  - zircon_runtime/src/ui/surface/control_index.rs
  - zircon_runtime/src/ui/surface/surface/pointer_component_events.rs
plan_sources:
  - docs/plans/optimize/zircon_runtime/74-runtime-ui-template-component-binding-expression-model-event-command-hot-reload-product-integration-review.md
tests:
  - cargo test -p zircon_runtime --lib component_control_scope_ --locked -- --test-threads=1 --nocapture
  - cargo test -p zircon_runtime --lib compiler_rejects_duplicate_control_ids_after_expansion --locked -- --exact --test-threads=1 --nocapture
  - cargo test -p zircon_runtime --lib template_tree_builder_rejects_duplicate_control_ids_during_instantiation --locked -- --exact --test-threads=1 --nocapture
  - cargo test -p zircon_runtime --lib ui::surface::control_index::tests --locked -- --test-threads=1 --nocapture
doc_type: module-detail
---

# Component Control Scope

Component definitions may use local `control_id` values and `control.X.prop.Y` expressions. Those
names are private to one component invocation. The compiler converts them to deterministic
instance-qualified identities before publishing a `UiTemplateInstance`.

## Identity

Each component call appends the authored call-node `node_id` to its parent component path. Path and
local-control bytes are hexadecimal encoded into an expression-safe identifier. Repeating the same
component therefore produces different private ids, while compiling the same document produces the
same artifact.

The component root is special. When the instance node declares a `control_id`, references to the
definition's root control resolve to that caller-visible identity. Other definition controls remain
qualified and private. A nested component receives the composed path; an instance-node binding and
slot fill retain the caller's scope.

## Binding Rewrite

The compiler rewrites `ControlPropRef` leaves in target assignments and action payload expressions.
It first uses the quote-aware token probe, then parses and re-emits through the canonical binding
expression renderer. Quoted preview text is not rewritten. An actual reference in an unsupported
expression dialect fails compilation because silently retaining an unqualified endpoint would be
unsafe.

After expansion, duplicate control ids fail compilation. Direct template instantiation applies the
same contract while building the tree and reports the first and duplicate node paths before either
ambiguous node can be published. Runtime property reads use the surface's unique incremental index
and return no value if a later runtime mutation introduces ambiguity; no smallest-node fallback
remains. Compiler schema version 3 invalidates artifacts created before this contract.

## Cost

Qualification is linear in expanded nodes plus binding AST size. Instantiation folds duplicate
detection into the existing tree-build traversal and does not add a second tree scan. The
1,000-instance acceptance test requires 1,000 unique private controls, 1,000 matching compiled
references, zero global duplicate fallbacks, and a five-second debug-test ceiling. Event-time lookup
remains index based.

Generation-qualified durable endpoint handles across hot reload are not owned here. They belong to
the compiled binding program and transactional publication work that follows this scope contract.
