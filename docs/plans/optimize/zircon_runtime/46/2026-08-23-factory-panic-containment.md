# Runtime46 Factory Panic Containment

- Date: 2026-08-23
- Owner: `optimize-runtime46-factory-panic-containment-r2-20260823`
- Source plan: `docs/plans/optimize/zircon_runtime/46-engine-module-service-contract-context-factory-descriptor-snapshot-composition-lifecycle-product-integration-review.md`
- Finding: `MOD-P0-001`
- Status: source implementation complete; managed validation pending

## Current-Source Review

The review was repeated at baseline HEAD
`0e2bdaa9d3f6949e351ce4e77ccf1aca9e7032b1`. The exact existing owner files
were unchanged from the initial `471bb732e3683fd7c12d7b69a9e85a22048efcba`
review when the coordinator rotated the immutable scope to include the existing
source-shape regression.

`resolve_existing_service_inner` changed a service slot to `Initializing`,
stored the current thread as owner, and then called service/plugin author code
directly. The normal `Err` path reset the slot and notified waiters, but a Rust
panic skipped that path. Lazy callers could therefore observe all three of the
following from one factory panic:

- the panic crossed the Core resolution boundary;
- the slot retained `Initializing` with an owner thread that had terminated;
- a concurrent resolver remained on the service-resolution condition variable
  without a terminal notification.

Immediate activation had an outer module-level unwind boundary, but that
reported a misleading module lifecycle callback failure instead of the failing
service identity. Lazy service and plugin resolution had no outer containment.

## Reference-Engine Decision

Unreal's `FModuleManager` makes a module findable before startup so reentrant
lookup works, but keeps `bIsReady` false until `StartupModule` returns. Load
order and the loaded event are committed only after successful startup. Zircon
retains the same essential rule for its finer service slots: provisional
visibility may exist during initialization, but Ready/Running is published only
after the exact factory generation commits.

Zircon cannot copy Unreal's C++ failure mechanics because Rust factory panics
can unwind through host code. The Rust-specific addition is a no-unwind
trampoline plus an owner-checked RAII claim. This preserves the existing
reentrant activation algorithm while giving every exit a terminal state.

## Change

- `ServiceFactoryPanicked { service }` is the typed Core boundary result.
- Service and plugin factories use one `catch_unwind(AssertUnwindSafe(...))`
  trampoline; author-returned errors still map to the existing
  `Initialization(name, detail)` contract.
- `ServiceInitializationClaim` is armed immediately after the slot claim. Its
  Drop path resets only the same index, generation, and owner when the slot is
  still `Initializing` and instance-free, then notifies all waiters.
- Successful commit and successful activation reentry explicitly disarm the
  claim. Concurrent unload/generation replacement cannot be reset by the stale
  guard because the existing lifecycle/owner checks fail closed.

## Acceptance Contract

- Lazy manager and lazy plugin factory panics return the exact service identity
  as `ServiceFactoryPanicked` and never escape Core.
- Immediate activation preserves the same typed error instead of relabeling it
  as a module callback panic.
- A panicked slot returns to `Registered`, clears `initialization_owner`, keeps
  no instance, and can be retried successfully.
- A resolver already waiting on the slot receives a notification and reaches a
  terminal result within a two-second test budget.
- Retry, module deactivation, and runtime drop remain usable after containment.
- No compatibility error alias, panic suppression outside the factory boundary,
  descriptor redesign, scheduler, or new dependency is introduced.

## Performance Scope

The cached resolved-service path is unchanged. The stack-only claim and unwind
boundary execute only when a service instance is first materialized for a
generation; waiters use the existing condition variable and wait graph. This
slice makes no throughput, latency, power, or cross-engine performance claim.
Runtime46 performance qualification remains M7 work after the compiled graph
and reload semantics exist, so measurements can compare the correct workload.

## Status And Output Record

| Milestone | Scope | Status | Date | Evidence |
|---|---|---|---|---|
| M0 review | Slot state machine, Immediate/Lazy service/plugin routes, waiter graph, Unreal module readiness | `review_complete` | 2026-08-23 | One shared factory invocation point; no prior lazy no-unwind boundary |
| M0 RED | Four focused panic/retry/waiter tests | `red_confirmed` | 2026-08-23 | Typed variant and trampoline absent before production edit |
| M0 containment | Typed error, unified trampoline, RAII claim, modular focused tests | `implementation_complete_validation_pending` | 2026-08-23 | Static containment gate, exact rustfmt, and scoped diff check pass; managed Cargo validation and independent review pending |

## Remaining Scope

Runtime46 M1-M7 identity, compiled contract, sealed context, single snapshot,
reload, product migration, and performance qualification remain open. Runtime01
continues to own process shutdown and full module cleanup; Plugins01/Runtime07
continue to own untrusted native/VM isolation. This M0 does not close those
parent findings.
