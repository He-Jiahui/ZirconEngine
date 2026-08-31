# Runtime74 Binding Missing-Value Policy

Plan: docs/plans/optimize/zircon_runtime/74-runtime-ui-template-component-binding-expression-model-event-command-hot-reload-product-integration-review.md
Milestone: M1
Status: validation_pending
Files: ["docs/plans/optimize/zircon_runtime/74/2026-08-23-binding-missing-value-policy.md","docs/zircon_runtime/ui/template/pipeline.md","zircon_runtime_interface/src/tests/contracts.rs","zircon_runtime_interface/src/ui/template/asset/binding/mod.rs","zircon_runtime_interface/src/ui/template/asset/binding/target.rs","zircon_runtime_interface/src/ui/template/asset/compiler/binding_program.rs","zircon_runtime_interface/src/ui/template/asset/compiler/package/header.rs","zircon_runtime_interface/src/ui/template/asset/document.rs","zircon_runtime_interface/src/ui/template/asset/mod.rs","zircon_runtime_interface/src/ui/template/mod.rs","zircon_runtime/src/ui/surface/binding_targets.rs","zircon_runtime/src/ui/surface/surface/default_interactions.rs","zircon_runtime/src/ui/surface/surface/pointer_component_events.rs","zircon_runtime/src/ui/template/asset/compiler/binding_program.rs","zircon_runtime/src/ui/tests/asset_binding/compiled_program.rs","zircon_runtime/src/ui/tests/event_routing/component_events.rs","zircon_runtime/src/ui/tests/event_routing/component_events/missing_policy.rs","zircon_runtime/src/ui/tests/event_routing/component_events/performance.rs","zircon_runtime/src/ui/tests/runtime_input_reply_routes/table_pointer_routes.rs","zircon_editor/src/tests/editing/ui_asset_replay.rs","zircon_editor/src/ui/asset_editor/binding/binding_inspector/payload_editing.rs","zircon_editor/src/ui/template_runtime/runtime/compiled_template_action.rs","zircon_editor/src/ui/template_runtime/runtime/projection.rs","zircon_editor/src/ui/template_runtime/runtime/runtime_host.rs","zircon_editor/src/ui/template_runtime/runtime/template_action_registry.rs"]

- Date: 2026-08-23
- Owner: `optimize-runtime74-param-ref-compile-r3-bee4c707-20260822`
- Source item: `RTB-P1-009`
- Delivery state: implementation and static source contract complete; grouped coordinator validation pending

## Scope Delivered

- `UiBindingMissingValuePolicy` gives authored targets and action payloads explicit `Required`,
  `Optional`, `Default`, `Fallback`, and `Error` behavior. Existing serialized assets default to
  `Required`.
- The compiler retains the policy in `UiCompiledBindingTarget` and `UiCompiledBinding`; compiler
  schema 8 invalidates persistent entries that predate the fields. Artifact well-formedness rejects
  non-finite default or fallback values.
- `UiBindingMissingValueResolution` is the shared typed result. Runtime target transactions,
  Runtime raw and compiled action dispatch, Editor source preview, and Editor compiled retained
  actions consume the same value/omit/required-missing/explicit-error outcomes.
- An optional target omits only its unresolved mutation while resolved siblings remain in the
  atomic transaction. Default and fallback substitutes still pass the ordinary target type checks.
  Required and explicit-error targets produce distinct rejection messages.
- Optional action fields are omitted without suppressing the route. Default/fallback fields publish
  typed substitutes; required/error fields suppress the action. The former implicit
  `collect::<Option<_>>()?` short circuit was removed from all four action paths.

## Reference Evidence and Divergence

- `dev/UnrealEngine/Engine/Plugins/Runtime/ModelViewViewModel/Source/ModelViewViewModel/Public/Bindings/MVVMCompiledBindingLibrary.h`
  returns `TValueOrError` from binding execution and names invalid source/destination and conversion
  failures in `EExecutionFailingReason`. This supports preserving a typed missing/error outcome at
  the compiled execution boundary instead of reducing every failure to absence.
- `dev/slint/internal/compiler/expression_tree.rs` keeps `Invalid` and `Uncompiled` expression
  states distinct and maps both into an explicit invalid type before execution. This is supporting
  evidence for fail-closed typed states, not a source for Zircon's five-policy vocabulary.

Zircon adds serialized per-target and per-action policies because its retained surface transaction
and routed action payload have different publication behavior. `Default` and `Fallback` currently
share substitute mechanics but remain distinct serialized intent for later provider-aware fallback.

## Regression and Validation Contract

Ten focused behavior tests cover interface round-trip and finite-value admission, compiler
retention and legacy defaulting, three target-policy cases, three action-policy cases, and both
Editor source/compiled consumers. The Runtime action tests exercise raw authoring and compiled
dispatch through the public event path.

Local non-Cargo evidence on 2026-08-23:

- `rustfmt +1.94.1`: passed for all P1-009 Rust owners and regressions.
- Runtime74 P1 `-SourceContractOnly`: 11/11 checks passed.
- Five Runtime74 child/super-batch PowerShell scripts: AST parse passed.
- `git diff --check` for the P1-009 scope: passed (line-ending conversion warnings only).

The grouped gate is configured for five P1 tasks, 9 Cargo groups, 30 cumulative new behavior tests,
and the parent super-batch's 18 existing performance rows. Cargo has not run for this revision, so
no test pass, performance result, commit, or WeCom delivery is claimed here.

The grouped submission `a2c39ddcdd944d588daa96cd7c99d512` / request
`d92db795584a4c4e8a561e6d3df175e1` is queued asynchronously without waiting under root validator
SHA-256 `D84C8CA2B28C1EE4137D0CCC580FB601ED34F7F4E4084081E1AA0BEC75701ACB`. Its 245-path,
7-tombstone source manifest `6d2edcabe8fb82f2971f30f13d908d13899a148aa747ce75ae863a87c2582063`
excludes coordinator state paths. This is submission evidence only; acceptance remains pending.

## Performance

Resolved values take the same expression-evaluation path plus one typed result match. Missing
values require no string policy parsing, registry lookup, or action-payload retry; they take one
enum branch and, only for default/fallback, clone the authored substitute. RTB-P1-009 adds no
standalone release-performance row. The existing 18 grouped Runtime74 performance rows remain the
release regression gate and are pending coordinator execution.
