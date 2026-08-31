# Runtime74 Typed Binding Conversion Lifecycle

Plan: docs/plans/optimize/zircon_runtime/74-runtime-ui-template-component-binding-expression-model-event-command-hot-reload-product-integration-review.md
Milestone: M1
Status: validation_pending
Files: ["docs/plans/optimize/zircon_runtime/74/2026-08-23-typed-binding-conversion-lifecycle.md","docs/zircon_runtime/ui/template/pipeline.md","zircon_runtime/src/ui/binding/conversion_registry.rs","zircon_runtime/src/ui/binding/mod.rs","zircon_runtime/src/ui/tests/binding_conversion_registry.rs","zircon_runtime/src/ui/tests/mod.rs","zircon_runtime_interface/src/tests/binding_conversion_contracts.rs","zircon_runtime_interface/src/tests/mod.rs","zircon_runtime_interface/src/ui/binding/mod.rs","zircon_runtime_interface/src/ui/binding/model/conversion.rs","zircon_runtime_interface/src/ui/binding/model/mod.rs"]

- Date: 2026-08-23
- Owner: `optimize-runtime74-param-ref-compile-r3-bee4c707-20260822`
- Source item: `RTB-P1-008`
- Delivery state: implementation complete; grouped coordinator validation pending

## Scope Delivered

- Conversion descriptors carry a validated serializable ID, exact source/destination
  `UiValueKind` signature, and a non-zero provider generation.
- Registration assigns a dense slot. Handles combine that slot with provider generation, so a
  provider upgrade can retain index locality while invalidating every old handle.
- Same-descriptor registration is idempotent. Same-generation signature changes, generation
  regressions, stale handles, unloaded handles, and slot exhaustion are typed failures.
- Execution validates input kind before the provider call, preserves structured provider failure,
  and validates the provider's output kind before publication.
- Explicit unload removes the active ID and invalidates the handle. Replacing a provider with a
  newer generation increments registry revision and retains its slot.

## Reference Evidence and Divergence

- `dev/UnrealEngine/Engine/Plugins/Runtime/ModelViewViewModel/Source/ModelViewViewModel/Public/Bindings/MVVMCompiledBindingLibrary.h`
  stores conversion paths with compiled bindings, exposes explicit execution failure reasons, and
  makes library load state part of execution validity.
- `dev/UnrealEngine/Engine/Plugins/Runtime/ModelViewViewModel/Source/ModelViewViewModelBlueprint/Private/Bindings/MVVMConversionFunctionHelper.cpp`
  validates conversion function arguments and return properties at the compiler boundary.

Zircon keeps those responsibilities together but uses Rust function providers, serializable typed
IDs, `UiValueKind` signatures, and dense generation-qualified handles. UObject reflection,
Blueprint graphs, and Unreal field paths are intentionally not copied.

## TDD and Validation Contract

Tests were authored before the registry implementation. Positive coverage locks descriptor/handle
round-trip, exact typed execution, idempotency, generation upgrade, and unload. Negative coverage
locks invalid identity/generation, signature mismatch, generation collision/regression, stale
handles, input mismatch, provider failure, and invalid provider output.

The grouped Runtime74 submission `caf7bfeb2eed4e3e9452e78fd45aed36` / request
`a97a2f548668430b997b32ec2891c14b` covered 88 tasks, 62 Cargo groups, 20 new behavior tests, and
18 existing performance rows under validator SHA-256
`E93B9E81B8EFA1225CDA3B5CF5632687E7CA29C1A02C20C4614342A91D6BAFB1`. It failed during
validation-copy `closure_planning` with `validation_copy_state_forbidden`, before Cargo started.
No behavior pass, performance result, or commit is claimed; grouped validation remains pending.

The forward grouped submission `a2c39ddcdd944d588daa96cd7c99d512` / request
`d92db795584a4c4e8a561e6d3df175e1` is queued asynchronously without waiting. It covers 89 tasks,
65 Cargo groups, 30 cumulative new behavior tests, and 18 performance rows under root validator
SHA-256 `D84C8CA2B28C1EE4137D0CCC580FB601ED34F7F4E4084081E1AA0BEC75701ACB`; its 245-path,
7-tombstone source manifest is `6d2edcabe8fb82f2971f30f13d908d13899a148aa747ce75ae863a87c2582063`.
This receipt is submission evidence only; acceptance remains pending.

## Performance

Execution uses one bounds-checked dense-slot lookup and one generation comparison before invoking
the conversion function; it performs no string or registry-tree lookup on the execution path.
This contract slice adds no standalone release benchmark row. Measured integrated conversion P95
remains pending until conversion handles are cooked into compiled binding instructions.
