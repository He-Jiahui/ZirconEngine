# Runtime74 Compiled Action Payload Program

Plan: docs/plans/optimize/zircon_runtime/74-runtime-ui-template-component-binding-expression-model-event-command-hot-reload-product-integration-review.md
Milestone: M1
Status: validation_pending
Files: ["docs/plans/optimize/zircon_runtime/74/2026-08-22-compiled-action-payload-program.md","docs/plans/optimize/zircon_runtime/74/2026-08-22-shared-binding-expression-evaluator.md","docs/plans/optimize/zircon_runtime/74/2026-08-22-typed-binding-endpoint-program.md","docs/ui-and-layout/ui-asset-documents-and-editor-protocol.md","docs/ui-and-layout/ui-asset-foundation-descriptors-contracts-invalidation.md","docs/zircon_runtime/ui/architecture.md","docs/zircon_runtime/ui/surface/binding_targets.md","docs/zircon_runtime/ui/template/pipeline.md","zircon_editor/src/ui/template_runtime/runtime/compiled_template_action.rs","zircon_editor/src/ui/template_runtime/runtime/mod.rs","zircon_editor/src/ui/template_runtime/runtime/template_action_registry.rs","zircon_editor/src/ui/template_runtime/runtime/template_action_slot.rs","zircon_runtime/src/ui/surface/binding_targets.rs","zircon_runtime/src/ui/surface/surface/default_interactions.rs","zircon_runtime/src/ui/surface/surface/pointer_component_events.rs","zircon_runtime/src/ui/template/asset/compiler/binding_program.rs","zircon_runtime/src/ui/template/asset/compiler/package/artifact.rs","zircon_runtime/src/ui/tests/asset_binding/compiled_program.rs","zircon_runtime/src/ui/tests/asset_package_validation.rs","zircon_runtime/src/ui/tests/event_routing/component_events.rs","zircon_runtime/src/ui/tests/event_routing/component_events/performance.rs","zircon_runtime_interface/src/tests/contracts.rs","zircon_runtime_interface/src/ui/template/asset/compiler/binding_program.rs","zircon_runtime_interface/src/ui/template/asset/compiler/mod.rs","zircon_runtime_interface/src/ui/template/asset/compiler/package/artifact.rs","zircon_runtime_interface/src/ui/template/asset/compiler/package/header.rs","zircon_runtime_interface/src/ui/template/asset/mod.rs","zircon_runtime_interface/src/ui/template/mod.rs"]

- Date: 2026-08-22
- Owner: `optimize-runtime74-param-ref-compile-r3-bee4c707-20260822`
- Source items: `RTB-P1-015`, `RTB-P1-016`
- Delivery state: implementation complete; grouped coordinator validation pending

## Problem

Compiled bindings stored action and route identities plus payload field names, but retained each
payload value only in authoring TOML. Every event parsed the same expression text again, and default
interactions did not attach the compiled binding handle. Editor action-token dispatch repeated the
same parse from its retained source slot.

## Delivered Contract

- Every compiled payload field carries an interned property ID and either a typed `UiValue`, a
  `UiCompiledBindingExpression`, or an explicit `Unavailable` marker.
- The canonical compiler parses standard payload expressions once after component expansion and
  parameter/control qualification. Over-budget expressions fail compilation.
- Runtime pointer events, default interactions, and atomic target execution all resolve payloads
  from the installed program. The compiled path never reads `UiActionRef.payload`.
- A target override replaces the compiled payload value without evaluating it. Prepared overrides
  use dense property IDs and move into the invocation instead of allocating field-name keys and
  cloning resolved values again. Target reporting obtains its previous value from the same field.
- Asset-editor-only functions such as `concat` are not silently treated as strings. Their compiled
  value is unavailable and Runtime action publication fails closed.
- Editor action-token slots compile the standard `UiBindingExpression` AST at bind/rebind time and
  use the shared evaluator on token dispatch. Editor host-model authoring projection remains a
  separate non-cooked path.
- Compiler schema 7, TOML envelope schema 3, and magic `ZRUIA018` invalidate earlier artifacts;
  schema 7 additionally prevents V2 caches from reusing artifacts compiled before product params
  or explicit binding-mode identity.
  Artifact admission validates payload property IDs, uniqueness, and expression budgets.
- Binding IDs are globally contiguous in node/source-slot order, target endpoint indices match
  authored target order, and payload fields are strictly lexical by field name. Artifact admission
  rejects reordered binding slots and payload fields instead of accepting a different execution
  order under the same compiled schema.

## Deterministic Performance Gate

`compiled_action_payload_ir_improves_nearest_rank_p95_by_at_least_twenty_five_percent` runs 21
alternating sample pairs with 4,000 dispatch-equivalent payload resolutions per side. The legacy
side parses and evaluates source text; the compiled side evaluates the typed field directly. It
emits all raw samples plus externally recomputable nearest-rank P95 values:

`PERF-RUNTIME74-COMPILED-PAYLOAD sample_pairs=21 dispatches_per_sample=4000 legacy_samples_us=<21-raw-samples> compiled_samples_us=<21-raw-samples> legacy_p95_us=<measured> compiled_p95_us=<measured> improvement_threshold_percent=25`

Measured values remain pending coordinator execution; no performance pass is claimed.

## Acceptance

- Compiler tests inspect literal, standard-expression, and preview-only payload variants and verify
  artifact round-trip preservation.
- Runtime dispatch mutates authoring TOML after program installation and proves the emitted action
  still uses the compiled value.
- Editor slot tests prove source mutation after compilation cannot change the invocation and that a
  preview-only expression fails closed.
- The external validator runs 26 behavior tests in six Cargo groups and validates four independent
  21-pair performance rows. With the selective binding-transaction, compiled control-slot, and
  payload-move children, the Runtime74 superbatch contains 62 tasks in 31 Cargo groups and fourteen
  performance rows.
- The prior interface-only async batch became orphaned before terminal test output and was released;
  it is not acceptance evidence. Current grouped Cargo and measured P95 values remain pending.
