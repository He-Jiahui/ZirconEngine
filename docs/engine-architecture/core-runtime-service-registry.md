---
related_code:
  - zircon_runtime/src/core/mod.rs
  - zircon_runtime/src/core/runtime/mod.rs
  - zircon_runtime/src/core/runtime/error.rs
  - zircon_runtime/src/core/runtime/lifecycle.rs
  - zircon_runtime/src/core/runtime/runtime.rs
  - zircon_runtime/src/core/runtime/contexts/mod.rs
  - zircon_runtime/src/core/runtime/descriptors/mod.rs
  - zircon_runtime/src/core/runtime/handle/core_handle.rs
  - zircon_runtime/src/core/runtime/handle/registration/mod.rs
  - zircon_runtime/src/core/runtime/handle/activation.rs
  - zircon_runtime/src/core/runtime/handle/activation/startup.rs
  - zircon_runtime/src/core/runtime/handle/activation/blocked_unload.rs
  - zircon_runtime/src/core/runtime/handle/activation/unload_mutation.rs
  - zircon_runtime/src/core/runtime/handle/resolution.rs
  - zircon_runtime/src/core/runtime/tests/activation/behavior/activation.rs
  - zircon_runtime/src/core/runtime/tests/activation/behavior/activation/contention.rs
  - zircon_runtime/src/core/runtime/tests/resolution/behavior.rs
  - zircon_runtime/src/core/runtime/tests/resolution/behavior/dependency_cycles.rs
  - zircon_runtime/src/core/runtime/tests/resolution/behavior/exact_dependency_resolution.rs
  - zircon_runtime/src/core/runtime/tests/resolution/behavior/factory_panics.rs
  - zircon_runtime/src/core/runtime/handle/events.rs
  - zircon_runtime/src/core/runtime/state/core_runtime_state.rs
  - zircon_runtime/src/core/runtime/state/module_entry.rs
  - zircon_runtime/src/core/runtime/state/service_entry.rs
  - zircon_runtime/src/core/runtime/events.rs
  - zircon_runtime/src/core/runtime/events/diagnostics.rs
  - zircon_runtime/src/core/runtime/events/prune.rs
  - zircon_runtime/src/core/runtime/events/publish.rs
  - zircon_runtime/src/core/runtime/events/subscribe.rs
  - zircon_runtime/src/core/runtime/events/subscriber.rs
  - zircon_runtime/src/core/runtime/events/topic.rs
  - zircon_runtime/src/core/framework/events.rs
  - zircon_runtime/src/engine_module/service_factory.rs
  - zircon_runtime/src/foundation/runtime/config_manager.rs
  - zircon_runtime/src/foundation/runtime/event_manager.rs
  - zircon_runtime/src/foundation/module.rs
implementation_files:
  - zircon_runtime/src/core/runtime/runtime.rs
  - zircon_runtime/src/core/runtime/handle/registration/mod.rs
  - zircon_runtime/src/core/runtime/handle/activation.rs
  - zircon_runtime/src/core/runtime/handle/resolution.rs
  - zircon_runtime/src/core/runtime/state/core_runtime_state.rs
  - zircon_runtime/src/core/runtime/state/module_entry.rs
  - zircon_runtime/src/core/runtime/state/service_entry.rs
  - zircon_runtime/src/core/runtime/events.rs
  - zircon_runtime/src/core/runtime/events/diagnostics.rs
  - zircon_runtime/src/core/runtime/events/prune.rs
  - zircon_runtime/src/core/runtime/events/publish.rs
  - zircon_runtime/src/core/runtime/events/subscribe.rs
  - zircon_runtime/src/core/runtime/events/subscriber.rs
  - zircon_runtime/src/core/runtime/events/topic.rs
  - zircon_runtime/src/core/framework/events.rs
plan_sources:
  - docs/plans/zircon_runtime/runtime/02-core-spine-and-root-surface.md
  - docs/plans/zircon_runtime/frameworks/02-module-kernel-and-lifecycle-unification.md
  - docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md
tests:
  - tools/tests/test_frameworks_02_core_error_single_source.py
  - zircon_runtime/src/core/runtime/tests.rs
  - zircon_runtime/src/core/runtime/tests/events
  - zircon_runtime/src/core/runtime/tests/activation
  - zircon_runtime/src/core/runtime/tests/registration
  - zircon_runtime/src/core/runtime/tests/resolution
  - zircon_runtime/src/tests/runtime_absorption/service_registry_lifecycle.rs
  - zircon_runtime/src/tests/runtime_absorption/service_registry_ownership.rs
  - zircon_editor/src/tests/host/manager/runtime_lifecycle.rs
doc_type: module-detail
status: current
---

# Core Runtime Service Registry

## Purpose

`zircon_runtime::core::runtime` owns the engine service registry, module lifecycle, dependency resolution, runtime event delivery, configuration storage and scheduler-facing runtime state. Public DTOs stay in `core::framework`; concrete registry state and behavior stay under `core::runtime`.

This document describes the current ownership boundary. Historical migration attempts and command transcripts belong to the numbered plans and are intentionally not duplicated here.

## Ownership Map

| Concern | Canonical owner | Boundary |
| --- | --- | --- |
| Public runtime facade | `core/runtime/mod.rs` and `core/mod.rs` | Curated exports only; no behavior implementation. |
| Runtime root and handles | `core/runtime/runtime.rs` and `core/runtime/handle/` | Runtime construction, registration, activation, resolution and handle operations. |
| Registry declarations | `core/runtime/descriptors/` and `core/runtime/contexts/` | Service/module descriptors and construction contexts. |
| Authoritative registry state | `core/runtime/state/` | Module and service tables, lifecycle and cached instances. Never a public facade. |
| Event DTOs and policies | `core/framework/events.rs` | Neutral contracts that do not own channels, locks or subscriber storage. |
| Event delivery | `core/runtime/events.rs` and `core/runtime/events/` | Topic membership, queues, fan-out, diagnostics and pruning. |
| Foundation adapters | `foundation/runtime/config_manager.rs` and `foundation/runtime/event_manager.rs` | Registry services over runtime-owned config and event facilities. |

The retired `core::event_bus` and mixed root-file owners must not return as aliases or compatibility modules.

## Registry Contract

`ModuleDescriptor::name` is the canonical module-table key. Registration rejects empty or whitespace-padded module names before mutating the table.

`RegistryName` is the canonical service-table key and has the exact serialized form `Module.Kind.Service`. Module and service segments are non-empty and cannot contain leading/trailing whitespace or additional separators. The kind segment is one of the canonical `ServiceKind` values. Callers use `RegistryName` accessors rather than reparsing the string.

Registration is transactional: descriptor validation and duplicate detection complete before module and service entries become visible. `ModuleEntry` caches owner, immediate-startup and shutdown service-key slices, so activation and deactivation do not rescan the global service table. Small exact-count helpers are private optimizations of this contract; they must preserve the same ordering and rollback behavior as the generic path.

Resolution treats the service table as the single instance authority. A direct resolve may activate the owner module; after activation it rereads the service entry before invoking a factory, preventing duplicate construction of an immediate service. Initialization failure resets lifecycle state so a later resolve can retry. Dependency cycles and service-kind mismatches remain typed `CoreError` results.

## Ownership And Lifecycle

`CoreRuntimeInner.services` strongly owns materialized service objects. A registry-owned service must not strongly retain `CoreHandle`, because that would create the cycle `CoreRuntimeInner -> ServiceEntry -> service -> CoreHandle -> CoreRuntimeInner`. Persistent reverse access uses `CoreWeak` and upgrades only at an operation boundary.

Module deactivation first checks dependents in shutdown order. A blocked deactivation restores the module lifecycle without partially unloading earlier services. Successful unload clears cached instances according to the cached shutdown order. Registration rollback, initialization failure and panic-unwind paths must release temporary owners.

Foundation, Editor and plugin managers that live in the registry follow the same weak-back-reference rule. Caller-owned hosts and resolvers may retain a strong handle only when they are not reachable from a registry-owned service instance.

## Event Boundary

`core::framework::events` owns `EngineEvent`, delivery policies, subscription result types and diagnostics snapshots. `core::runtime::events` owns the concrete `EventBus`.

The runtime bus keeps topic membership separate from per-subscriber queues and per-topic delivery serialization. Publishing snapshots the subscriber set, releases the topic map lock, and then delivers according to each subscription policy. Subscriber removal and empty-topic pruning return through the runtime owner. Poisoned internal mutexes recover their contained state rather than turning an unrelated event operation into a process panic.

## Extension Rules

- Add public declarations to the relevant descriptor, context or framework owner; keep `mod.rs` files navigational.
- Add `CoreHandle` behavior to the matching registration, activation, resolution, event or configuration owner.
- Keep registry state private and derive service identity from the `RegistryName` table key instead of duplicating it in `ServiceEntry`.
- Do not introduce strong runtime back-references in registry-owned services.
- Do not duplicate runtime delivery storage in framework DTO modules or foundation adapters.

## Validation Owners

Runtime-core behavior and structure are covered by the mounted suites under `core/runtime/tests/`. Cross-layer lifetime coverage lives in `runtime_absorption/service_registry_lifecycle.rs`, `runtime_absorption/service_registry_ownership.rs` and the Editor manager lifecycle suite. Current acceptance status and managed Cargo evidence belong to Runtime02/Frameworks02 plan records, not this module document.

Resolution behavior tests are folder-backed by domain: the 631-line root keeps shared fixtures and
10 general behaviors, while dependency cycles, exact 4/5 dependency initialization and factory panic
lifecycle behavior live in 115/217/258-line children. The exact-dependency move preserved the two test
bodies byte-for-byte after whitespace normalization and did not change resolution behavior. Status:
`runtime_02_15_resolution_exact_dependency_test_owner_split_static_passed_cargo_deferred`.

Activation behavior keeps transaction/lifecycle fixtures and 11 general tests in the 756-line parent.
The deterministic 7-joiner gate, release-only 21-sample benchmark and their private sampling helpers
live in the 132-line `activation/contention.rs` child. The physical move retained the existing 750 ms
contract but did not generate new performance evidence. Status:
`runtime_02_15_activation_contention_test_owner_split_static_passed_cargo_profile_deferred`.
