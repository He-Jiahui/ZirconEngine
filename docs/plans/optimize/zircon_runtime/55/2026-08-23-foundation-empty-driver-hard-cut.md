# Runtime55 Foundation Empty Driver Hard Cut

- Date: 2026-08-23
- Owner: `optimize-runtime55-foundation-empty-driver-hard-cut-r1-20260823`
- Source plan: `docs/plans/optimize/zircon_runtime/55-runtime-foundation-module-config-event-service-driver-manager-persistence-lifecycle-product-integration-review.md`
- Findings: `FND-P0-003` (partial), `FND-P1-002`, `FND-P2-010`, `FND-G02` (partial)
- Status: source implementation complete; managed validation pending

## Current-Source Review

The review was repeated at baseline HEAD
`471bb732e3683fd7c12d7b69a9e85a22048efcba` before changing production code.

| Surface | Current source before this cut | Consumer evidence | M1 decision |
|---|---|---|---|
| `ConfigDriver` | Empty public unit struct, immediate descriptor, empty factory | No production resolver, dependency, method, state, health, or external consumer | Delete the type, descriptor, public name, module, and re-export |
| `EventDriver` | Empty public unit struct, immediate descriptor, empty factory | No production resolver, dependency, method, state, health, or external consumer | Delete the type, descriptor, public name, module, and re-export |
| Config/Event managers | Concrete manager factories backed by Core config and event primitives | Config has an Editor host consumer; Event remains test-only | Retain both managers; product event consumption remains open |

Repository-wide symbol search found the two driver types only in their own
declarations, Foundation assembly, and review documentation. No production
consumer requires a migration or compatibility alias. The two managers do not
depend on either driver, so removing the descriptors changes capability
reporting and startup work without changing manager behavior.

The historical architecture note in
`docs/plans/zircon_runtime/runtime/02-core-spine-and-root-surface.md` that lists
the two driver exports is superseded by this hard-cut record. It must not be
used to restore those public placeholders.

## Reference-Engine Decision

- Unreal's module manager calls `StartupModule`, records completion order, and
  broadcasts the loaded transition for an actual module instance. Its
  dependency and shutdown guarantees are tied to behavior that ran, not to an
  empty service name or an allocated marker object.
- Fyrox's resource event surface owns a typed event enum, subscriber handles,
  broadcast behavior, and dead-subscriber reclamation. A type with no behavior
  is not treated as an event provider.

Zircon therefore keeps the existing managers as its current behavioral
providers and publishes no separate driver until a real external file or
transport boundary exists. A future provider must add behavior, ownership,
health, dependency, and consumer evidence; the removed names are not reserved
compatibility points.

## Acceptance Contract

- `FoundationModule` has zero driver descriptors and exactly two manager
  descriptors.
- `ConfigDriver`, `EventDriver`, `CONFIG_DRIVER_NAME`, and
  `EVENT_DRIVER_NAME` are physically absent from the public and internal
  source surface.
- The two legacy driver source files are deleted.
- ConfigManager and EventManager construction and registry names are unchanged.
- No re-export, facade, shim, compatibility alias, Cargo dependency, config
  behavior, event behavior, or persistence behavior is introduced.

## Quantified Impact

The cut removes two false driver descriptors, two service registration/index
slots, two immediate empty factory invocations, and two empty `Arc` service
allocations per Foundation-module activation. It also removes two public types,
two public constants, and two source files.

This is startup and contract cleanup, not a frame-path optimization. No
throughput, latency, power, or cross-engine performance claim is made. Runtime55
M7 measurements remain blocked on the correct typed config/event and scoped
persistence workloads.

## Remaining Scope

`FND-P0-003` and `FND-G02` remain partial because EventManager still has no
production consumer and Asset/Platform/Editor dependencies are not yet compiled
into exact service edges. Boot precedence, durable projection, multi-Runtime
path ownership, typed event/config authority, and shutdown receipts remain open.
The Runtime25 Asset descriptor is frozen in a separate validation ticket, so
this slice intentionally does not modify that foreign candidate.

## Status And Output Record

| Milestone | Scope | Status | Date | Evidence |
|---|---|---|---|---|
| M1 review | Current source, full symbol call graph, Unreal/Fyrox routing | `review_complete` | 2026-08-23 | 0 driver consumers; both managers independent of the drivers |
| M1 hard cut | Descriptor, public exports, runtime modules, focused regression | `implementation_complete_validation_pending` | 2026-08-23 | Expected RED source gate, static capability-truth gate, exact rustfmt check, and scoped diff check; managed Cargo validation and independent review pending |
