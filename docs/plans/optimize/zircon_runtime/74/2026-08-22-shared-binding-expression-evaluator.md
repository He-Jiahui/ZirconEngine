# Runtime74 Shared Binding Expression Evaluator

Plan: docs/plans/optimize/zircon_runtime/74-runtime-ui-template-component-binding-expression-model-event-command-hot-reload-product-integration-review.md
Milestone: M1
Status: validation_pending
Files: ["docs/plans/optimize/zircon_runtime/74/2026-08-22-shared-binding-expression-evaluator.md","docs/zircon_runtime/ui/template/pipeline.md","zircon_editor/src/ui/template_runtime/runtime/projection.rs","zircon_runtime/src/ui/surface/surface/pointer_component_events.rs","zircon_runtime_interface/src/ui/template/asset/binding/expression.rs","zircon_runtime_interface/src/ui/template/asset/binding/expression/evaluator.rs","zircon_runtime_interface/src/ui/template/asset/binding/mod.rs","zircon_runtime_interface/src/ui/template/asset/mod.rs","zircon_runtime_interface/src/ui/template/mod.rs"]

- Date: 2026-08-22
- Owner: `optimize-runtime74-param-ref-compile-r3-bee4c707-20260822`
- Source items: canonical evaluator subset of `RTB-P1-029`; execution-budget subset of `RTB-P1-013`
- Delivery state: implementation complete; grouped coordinator validation pending

## Problem

Runtime pointer dispatch and Editor template projection each implemented a recursive evaluator for
the same `UiBindingExpression` AST. The copies could drift on short-circuiting, missing values, type
errors, and depth handling. Both also recursed through expression-controlled depth.

## Delivered Contract

- `UiBindingExpression::evaluate_with` is the consumer-neutral evaluator authority in
  `zircon_runtime_interface`.
- Consumers supply param, property, and control-property resolver closures; the evaluator owns all
  literal, equality, boolean, short-circuit, error, node-budget, and depth-budget semantics.
- Evaluation uses explicit frame and value stacks. It rejects expressions above the shared 1,024
  node or 64-level depth ceilings without recursive evaluation.
- Frame and value stacks keep eight entries inline and spill only for deeper valid expressions;
  common shallow payload expressions do not allocate stack-container storage.
- Runtime and Editor template-runtime now call this shared entry point. Their former recursive match
  trees and local boolean coercion helpers are deleted.
- Missing references and non-boolean operands are typed evaluation errors internally. Existing
  Runtime/Editor action projection still maps those errors to its current optional publication
  boundary; operator-visible diagnostics remain owned by `RTB-P1-036`.

## Boundary

The asset-editor preview mock supports functions such as `concat`, `coalesce`, `first`, `join`, and
`count`; that is a separate authoring-preview dialect and is not silently promoted into the runtime
AST. The later `2026-08-22-compiled-action-payload-program.md` slice now compiles standard Runtime
payloads and Editor action-token slots before dispatch. Model/provider evaluation remains a later
Runtime74 milestone.

## Acceptance

- Shared evaluator tests cover param/property resolution, equality, nested boolean operations, and
  proof that an unreachable control lookup is not evaluated.
- A forged expression deeper than the public limit returns a typed budget error without recursion.
- External source contracts require both Runtime and Editor consumers to contain `evaluate_with`
  and reject reintroduction of either local recursive evaluator/helper.
- The grouped Runtime74 superbatch now contains 62 behavior/performance tasks in 31 Cargo groups and
  fourteen performance rows. Cargo results and measured endpoint/payload/control-slot P95 values
  remain pending async coordinator execution; no pass or latency claim is recorded yet.
