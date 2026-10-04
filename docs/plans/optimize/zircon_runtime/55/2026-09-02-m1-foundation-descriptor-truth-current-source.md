Plan: docs/plans/optimize/zircon_runtime/55-runtime-foundation-module-config-event-service-driver-manager-persistence-lifecycle-product-integration-review.md
Milestone: M1
Status: completed
Files: ["zircon_app/src/entry/tests/profile_bootstrap.rs", "zircon_runtime/src/asset/module.rs", "zircon_runtime/src/core/framework/foundation/event_manager.rs", "zircon_runtime/src/core/framework/foundation/mod.rs", "zircon_runtime/src/core/manager/mod.rs", "zircon_runtime/src/core/manager/resolver.rs", "zircon_runtime/src/core/manager/service_names.rs", "zircon_runtime/src/core/manager/tests.rs", "zircon_runtime/src/foundation/mod.rs", "zircon_runtime/src/foundation/module.rs", "zircon_runtime/src/foundation/runtime/event_manager.rs", "zircon_runtime/src/foundation/runtime/mod.rs", "zircon_runtime/src/foundation/tests.rs", "zircon_runtime/src/platform/module.rs", "zircon_runtime/src/tests/runtime_absorption/service_registry_ownership.rs"]

# Runtime55 M1 Foundation Descriptor Truth

## Scope Delivered

- Foundation now registers only the product-consumed ConfigManager; the test-only EventManager contract, implementation, factory, resolver, handle, and service name are physically removed.
- Asset and Platform no longer declare Foundation module dependencies without consuming a Foundation service. Their real Tasks and local service dependencies remain unchanged.
- The App bootstrap regression exercises the retained Core EventBus directly instead of manufacturing a Foundation service consumer.
- Structure guards reject restoration of the retired files and symbols or the two false dependency edges.

Quantified structural change: Foundation manager descriptors `2 -> 1`, placeholder drivers remain `0`, false Asset/Platform Foundation dependencies `2 -> 0`, compatibility shims `0`, and the scoped source diff is 15 files with 78 insertions and 187 deletions before this record. This is descriptor/API correctness work; no CPU, GPU, power, or cross-engine performance claim is made.

## Fresh Testing Evidence

- Current-source static scan finds no production `EventManager`, `DefaultEventManager`, `EVENT_MANAGER_NAME`, `event_manager_handle`, or `ManagerResolver::event_handle` caller. The only retained symbol strings are negative source guards.
- Current-source static scan finds no Asset or Platform `with_module_dependency(ModuleDependencySpec::named(FOUNDATION_MODULE_NAME))` edge.
- Targeted `rustfmt --edition 2021` completed for all changed Rust files.
- `git diff --check` passed for the working tree; Git emitted only configured LF-to-CRLF notices.
- Coordinator-managed Windows Cargo validation is the required next evidence and must bind this exact manifest snapshot.

## Review

Current-source review confirms the removed facade had zero production resolver consumers, duplicated the Core EventBus authority, silently ignored publish after Runtime loss, and returned a fabricated disconnected subscription. The hard cut leaves Core EventBus behavior intact and introduces no alias, re-export, compatibility bridge, fallback, ordering special case, or replacement facade. Independent coordinator review remains required after managed validation.
