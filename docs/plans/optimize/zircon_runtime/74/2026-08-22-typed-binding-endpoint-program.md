# Runtime74 Typed Binding Endpoint Program

Plan: docs/plans/optimize/zircon_runtime/74-runtime-ui-template-component-binding-expression-model-event-command-hot-reload-product-integration-review.md
Milestone: M1
Status: validation_pending
Files: ["docs/plans/optimize/zircon_runtime/74/2026-08-22-typed-binding-endpoint-program.md","docs/ui-and-layout/ui-asset-documents-and-editor-protocol.md","docs/ui-and-layout/ui-asset-foundation-descriptors-contracts-invalidation.md","docs/zircon_runtime/ui/architecture.md","docs/zircon_runtime/ui/surface/binding_targets.md","docs/zircon_runtime/ui/template/pipeline.md","zircon_runtime/src/ui/surface/binding_targets.rs","zircon_runtime/src/ui/surface/surface.rs","zircon_runtime/src/ui/surface/surface/pointer_component_events.rs","zircon_runtime/src/ui/template/asset/compiler/binding_param_resolver.rs","zircon_runtime/src/ui/template/asset/compiler/binding_program.rs","zircon_runtime/src/ui/template/asset/compiler/compile.rs","zircon_runtime/src/ui/template/asset/compiler/control_scope.rs","zircon_runtime/src/ui/template/asset/compiler/mod.rs","zircon_runtime/src/ui/template/asset/compiler/package/artifact.rs","zircon_runtime/src/ui/template/asset/compiler/prototype_instancer.rs","zircon_runtime/src/ui/template/asset/mod.rs","zircon_runtime/src/ui/template/build/surface_builder.rs","zircon_runtime/src/ui/template/instance.rs","zircon_runtime/src/ui/template/mod.rs","zircon_runtime/src/ui/tests/asset_binding.rs","zircon_runtime/src/ui/tests/asset_binding/compiled_program.rs","zircon_runtime/src/ui/tests/asset_package_validation.rs","zircon_runtime/src/ui/tests/event_routing/component_events.rs","zircon_runtime/src/ui/tests/template_tree_builder.rs","zircon_runtime_interface/src/tests/contracts.rs","zircon_runtime_interface/src/ui/dispatch/pointer/component_event.rs","zircon_runtime_interface/src/ui/template/asset/binding/expression.rs","zircon_runtime_interface/src/ui/template/asset/binding/mod.rs","zircon_runtime_interface/src/ui/template/asset/compiler/binding_program.rs","zircon_runtime_interface/src/ui/template/asset/compiler/mod.rs","zircon_runtime_interface/src/ui/template/asset/compiler/package/artifact.rs","zircon_runtime_interface/src/ui/template/asset/compiler/package/header.rs","zircon_runtime_interface/src/ui/template/asset/mod.rs","zircon_runtime_interface/src/ui/template/mod.rs"]

- Date: 2026-08-22
- Owner: `optimize-runtime74-param-ref-compile-r3-bee4c707-20260822`
- Source items: `RTB-P1-002`, `RTB-P1-003`, and the compiled-target subset of `RTB-P1-013`
- Delivery state: implementation complete; grouped coordinator validation pending

## Problem

Pointer component dispatch previously searched bindings by string, reparsed every target expression,
and rebuilt target lookup state on each event. Binding and target identity had no artifact generation,
so a delayed event could not prove that its endpoint belonged to the currently installed template.

## Architecture and Scope Delivered

- The canonical asset compiler fingerprints the expanded and styled root and emits a separate
  `UiCompiledBindingProgram` beside the retained authoring tree.
- Binding, property, route, action, control, node, and target identities use separate dense ID
  domains; public handles and endpoints carry the artifact generation.
- Target expressions compile to typed `UiCompiledBindingExpression` nodes. The valid dispatch path
  performs neither `UiBindingExpression::parse` nor construction of a binding `BTreeMap`.
- `UiTemplateSurfaceBuilder` installs the compiled program. Pointer events attach a typed binding
  handle, and target execution verifies generation, node, binding, event, and target endpoint before
  preparing an atomic transaction.
- Missing, stale, or mismatched compiled endpoints fail closed and suppress the outbound action.
  Target-free legacy/direct test surfaces retain their existing raw action path.
- A target-bearing compiled event defers action construction until the atomic target transaction
  commits. Payload fields supplied by target overrides skip evaluation of the value they replace.
- TOML envelope schema 3, compiler schema 7, and magic `ZRUIA018` invalidate artifacts that lack
  the new identity and generation contract, explicit binding mode, or V2 product-param expansion.
- The binary reader validates the complete node/binding/target/property/control index graph with an
  iterative expression-node budget before publishing an artifact. Dispatch repeats endpoint checks
  as a fail-closed guard against a mismatched in-memory program.
- Source parsing, artifact validation, and runtime target evaluation share explicit source-byte,
  token, node, and depth budgets. Runtime evaluation uses an explicit stack, retains boolean
  short-circuit semantics, and rejects rather than recursing through an over-depth program.

The authoring TOML and Editor-facing strings remain readable source. The follow-up
`2026-08-22-compiled-action-payload-program.md` slice now turns standard action payloads into
compiled values. Model/provider subscriptions remain with later Runtime74 items.

## Reference Evidence

- Slint's runtime uses dense typed item/property indices and separates compiled binding kinds from
  authored expressions; this leads the runtime representation.
- Unreal MVVM uses compiled binding handles plus field identities rather than resolving dynamic
  property paths for each delivery; this supports generation-qualified endpoint validation.
- Godot's indexed property-path access fails invalid paths explicitly; this supports fail-closed
  stale or mismatched endpoint behavior.

## Deterministic Performance Gate

`compiled_binding_endpoint_lookup_improves_nearest_rank_p95_by_at_least_twenty_five_percent` runs
21 paired samples. The first-run order alternates per pair. Each side performs 4,000 lookups across
five targets; the legacy side parses source expressions and resolves string names while the compiled
side resolves a generation-qualified handle and dense IDs. The test emits:

`PERF-RUNTIME74-COMPILED-ENDPOINT sample_pairs=21 lookups_per_target=4000 target_count=5 legacy_samples_us=<21-raw-samples> compiled_samples_us=<21-raw-samples> legacy_p95_us=<measured> compiled_p95_us=<measured> improvement_threshold_percent=25`

The external validator independently sorts both raw sample sets, recomputes nearest-rank P95, and
requires compiled P95 to improve legacy P95 by at least 25%. Measurements are pending coordinator
execution; no performance pass is claimed yet.

## Acceptance

- Compiler tests lock interned identity domains, typed target endpoints, direct action identity, and
  artifact round-trip generation preservation.
- Stale-generation and mismatched-target-index regressions prove fail-closed behavior without
  target mutation.
- Oversized/deep source parsing and an over-depth forged compiled expression both fail within the
  shared budget; the latter produces no target mutation or outbound event.
- Existing pointer target tests lock atomic commit, rollback, event suppression, and target-free fast
  path behavior under the compiled program.
- Interface contract tests lock compiler schema 7; artifact tests lock TOML envelope schema 3 and
  `ZRUIA018`.
- Grouped Cargo compile/tests and the 21-pair performance gate are pending asynchronous coordinator
  validation.
