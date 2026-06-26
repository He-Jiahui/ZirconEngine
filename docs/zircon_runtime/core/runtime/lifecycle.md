---
related_code:
  - zircon_runtime/src/core/runtime/lifecycle.rs
  - zircon_runtime/src/core/runtime/mod.rs
  - zircon_runtime/src/core/mod.rs
  - zircon_runtime/src/core/runtime/descriptors/registry_name.rs
  - zircon_runtime/src/core/runtime/handle/core_handle.rs
  - zircon_runtime/src/core/runtime/handle/activation.rs
  - zircon_runtime/src/core/runtime/handle/registration/register_module.rs
  - zircon_runtime/src/core/runtime/handle/registration/service_lists/mod.rs
  - zircon_runtime/src/core/runtime/handle/registration/service_lists/types.rs
  - zircon_runtime/src/core/runtime/handle/registration/service_lists/multi.rs
  - zircon_runtime/src/core/runtime/handle/registration/service_lists/specialized.rs
  - zircon_runtime/src/core/runtime/handle/registration/service_lists/shutdown.rs
  - zircon_runtime/src/core/runtime/handle/resolution.rs
  - zircon_runtime/src/core/runtime/handle/runtime_extensions.rs
  - zircon_runtime/src/core/runtime/tests/registration/structure.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/lock_poison_policy.rs
  - zircon_runtime/src/core/runtime/tests/activation/behavior/deactivation/blocked.rs
  - zircon_runtime/src/core/runtime/tests/activation/behavior/deactivation/blocked/external_dependents.rs
  - zircon_runtime/src/core/runtime/tests/activation/behavior/deactivation/blocked/exact_two_three_dependency_matcher.rs
  - zircon_runtime/src/core/runtime/tests/activation/behavior/deactivation/blocked/shutdown_order.rs
  - zircon_runtime/src/core/runtime/tests/activation/behavior/deactivation/blocked/exact_four_dependency_matcher.rs
  - zircon_runtime/src/core/runtime/tests/activation/behavior/deactivation/blocked/exact_five_without_index_map.rs
  - zircon_runtime/src/core/runtime/tests/activation/behavior/deactivation/blocked/exact_five_dependency_matcher.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/core_runtime_deactivation.rs
implementation_files:
  - zircon_runtime/src/core/runtime/lifecycle.rs
  - zircon_runtime/src/core/runtime/mod.rs
  - zircon_runtime/src/core/mod.rs
  - zircon_runtime/src/core/runtime/handle/core_handle.rs
  - zircon_runtime/src/core/runtime/handle/activation.rs
  - zircon_runtime/src/core/runtime/handle/registration/register_module.rs
  - zircon_runtime/src/core/runtime/handle/registration/service_lists/mod.rs
  - zircon_runtime/src/core/runtime/handle/registration/service_lists/types.rs
  - zircon_runtime/src/core/runtime/handle/registration/service_lists/multi.rs
  - zircon_runtime/src/core/runtime/handle/registration/service_lists/specialized.rs
  - zircon_runtime/src/core/runtime/handle/registration/service_lists/shutdown.rs
  - zircon_runtime/src/core/runtime/handle/resolution.rs
  - zircon_runtime/src/core/runtime/handle/runtime_extensions.rs
  - zircon_runtime/src/core/runtime/tests/registration/structure.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/lock_poison_policy.rs
  - zircon_runtime/src/core/runtime/tests/activation/behavior/deactivation/blocked.rs
  - zircon_runtime/src/core/runtime/tests/activation/behavior/deactivation/blocked/external_dependents.rs
  - zircon_runtime/src/core/runtime/tests/activation/behavior/deactivation/blocked/exact_two_three_dependency_matcher.rs
  - zircon_runtime/src/core/runtime/tests/activation/behavior/deactivation/blocked/shutdown_order.rs
  - zircon_runtime/src/core/runtime/tests/activation/behavior/deactivation/blocked/exact_four_dependency_matcher.rs
  - zircon_runtime/src/core/runtime/tests/activation/behavior/deactivation/blocked/exact_five_without_index_map.rs
  - zircon_runtime/src/core/runtime/tests/activation/behavior/deactivation/blocked/exact_five_dependency_matcher.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/core_runtime_deactivation.rs
plan_sources:
  - user: 2026-06-12 runtime architecture implementation from docs/plans/zircon_runtime/runtime
  - docs/plans/zircon_runtime/runtime/02-core-spine-and-root-surface.md
  - docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md
  - .codex/plans/Runtime 吸收层与 Editor_Scene 边界收束计划.md
tests:
  - zircon_runtime/src/tests/runtime_absorption/root_entries.rs
  - zircon_runtime::tests::runtime_absorption::structure_convention::production_file_budget::runtime_15_core_runtime_service_lists_are_folder_backed
  - zircon_runtime/src/core/runtime/handle/core_handle.rs::tests::core_handle_registry_accessors_recover_poisoned_runtime_locks
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/lock_poison_policy.rs::runtime_15_core_handle_registry_lock_poison_recovery_guard_covers_registry_accessors
  - zircon_runtime::tests::runtime_absorption::structure_convention::test_file_budget::core_runtime_deactivation::runtime_15_core_runtime_deactivation_blocked_tests_are_folder_backed
  - rustc --edition 2021 --test zircon_runtime/src/tests/runtime_absorption/root_entries.rs
  - rustfmt --edition 2021 --check zircon_runtime/src/core/runtime/handle/registration/service_lists/types.rs zircon_runtime/src/core/runtime/handle/registration/service_lists/specialized.rs zircon_runtime/src/core/runtime/handle/registration/service_lists/mod.rs zircon_runtime/src/core/runtime/handle/registration/register_module.rs
  - cargo check -p zircon_runtime --lib --locked
doc_type: module-detail
---

# Runtime Lifecycle Vocabulary

`zircon_runtime::core::runtime::lifecycle` owns the service and module lifecycle vocabulary used by runtime registration, activation, deactivation, and resolution. The former `core/lifecycle.rs` root fragment was moved here during runtime plan 02 M2.2.

## Ownership Boundary

- `StartupMode` describes whether a registered service starts immediately or waits for lazy resolution.
- `LifecycleState` describes the runtime state of modules and services: registered, initializing, running, stopping, or unloaded.
- `ServiceKind` is the canonical driver/manager/plugin classifier used by `RegistryName`, dependency validation, and service table logic.
- The curated `zircon_runtime::core::{LifecycleState, StartupMode, ServiceKind}` facade remains because these types are public runtime vocabulary, but the physical owner is now the runtime kernel.

The lifecycle module defines vocabulary only. Registration ordering, dependency validation, activation, deactivation, and resolution behavior stay in their existing runtime handle and descriptor owners.

## Validation

The root-surface guard rejects a revived `mod lifecycle;`, `pub use lifecycle::...`, or retired `src/core/lifecycle.rs` file. Source scans reject `crate::core::lifecycle` and `zircon_runtime::core::lifecycle` imports after the migration.

## Runtime 15 M4 core runtime service-list owner split

`runtime_15_core_runtime_service_lists_folder_split_static_passed_cargo_lock_blocked`

Runtime 15 M4 keeps registration and lifecycle behavior unchanged and splits only the oversized service-list owner. The old flat `core/runtime/handle/registration/service_lists.rs` file is gone. `core/runtime/handle/registration/service_lists/mod.rs` now owns the structural dispatch from pending services to `ModuleServiceLists`; `types.rs` owns the returned service/startup/shutdown name lists; `multi.rs` owns the generic multi-service scan paths; `specialized.rs` owns the one-through-five service paths; `shutdown.rs` owns shutdown list assembly and preserves the inverse plugin, manager, driver lifecycle order.

`runtime_15_core_runtime_service_lists_are_folder_backed` locks the folder-backed layout, prevents the flat file from returning, checks `register_module.rs` still consumes the same narrow service-list entry points, and keeps each service-list owner under the Runtime 15 production-file budget. This is static structure evidence only; Cargo remains deferred by the Runtime 15 milestone testing cadence.

`runtime_15_core_runtime_service_lists_registration_visibility_coremin_check_passed` records the first compile blocker found after the split. Moving the owner into child modules changed `pub(super)` to mean "visible only inside `service_lists`", while `register_module.rs` still legitimately consumes `ModuleServiceLists` and the single-service fast path through the registration owner. The fix exposes only `ModuleServiceLists`, its three returned list fields, and `single_service_module_lists(...)` to `crate::core::runtime::handle::registration`; all multi-service and shutdown helpers stay private to the service-list subtree. This preserves the split's narrow boundary without widening to `pub(crate)` or restoring the deleted flat file. Runtime core-min offline package validation now passes with existing warnings; locked validation is still blocked before compilation by current lockfile drift, and `Cargo.lock` is restored after each Cargo attempt.

## Runtime 15 M3 core handle registry lock poison recovery

`runtime_15_core_handle_registry_lock_poison_recovery_static_passed_cargo_deferred`

Runtime 15 M3 keeps registration, activation, deactivation, lazy resolution, world-extension application, scene-hook ordering, and plugin bridge lifecycle semantics unchanged while moving the shared registry locks behind poison-safe CoreHandle helpers. `core/runtime/handle/core_handle.rs` owns `lock_modules()`, `lock_services()`, `lock_scene_hooks()`, `lock_world_extensions()`, and `lock_plugin_bridge_lifecycle()`. The activation, registration, resolution, and runtime-extension owners consume those helpers instead of directly calling `self.inner.*.lock().unwrap()`.

`core_handle_registry_accessors_recover_poisoned_runtime_locks` deliberately poisons the shared modules, services, scene hooks, world extensions, and plugin bridge lifecycle mutexes, then verifies the helper layer recovers each lock. `runtime_15_core_handle_registry_lock_poison_recovery_guard_covers_registry_accessors` locks the helper shape, checks the four production consumers do not regress to direct lock unwrap or `lock poisoned` panic text, and keeps Runtime 15/status/docs anchors synchronized. This is static structure and module-local recovery evidence only; full `module_convention_gate` and full core runtime handle Cargo sweep remain pending.

## Runtime 15 M3 core runtime registration structure owner split

`runtime_15_core_runtime_registration_structure_owner_split_static_passed_cargo_deferred`

Runtime 15 M3 keeps registration lifecycle behavior unchanged and splits only the near-budget registration structure test owner. The old flat `core/runtime/tests/registration/structure.rs` file is gone. `core/runtime/tests/registration/structure/mod.rs` now owns only child mounting plus the shared `registration_sources()` fixture; `module_layout.rs`, `service_count_paths.rs`, `service_list_caches.rs`, `dependency_fast_paths.rs`, `duplicate_detection.rs`, `cleanup.rs`, and `behavior_layout.rs` own the focused structure guards.

`runtime_15_core_runtime_registration_structure_tests_are_folder_backed` locks the folder-backed layout, checks the helper commit-boundary guard still reads the service-count child, keeps cached service-list and dependency/duplicate structure checks out of the parent module, and keeps each registration structure owner below the Runtime 15 test-file budget. This is static structure evidence only; Cargo remains deferred by the Runtime 15 milestone testing cadence.

## Runtime 15 M3 core runtime deactivation blocked test folder split

`runtime_15_core_runtime_deactivation_blocked_tests_folder_split_static_passed_cargo_deferred`

Runtime 15 M3 keeps deactivation behavior unchanged and splits only the oversized blocked-deactivation test owner. `core/runtime/tests/activation/behavior/deactivation/blocked.rs` now owns only child module mounting. External dependent blockers live in `blocked/external_dependents.rs`; exact two/three dependency matcher coverage lives in `blocked/exact_two_three_dependency_matcher.rs`; shutdown-order coverage lives in `blocked/shutdown_order.rs`; exact four matcher coverage lives in `blocked/exact_four_dependency_matcher.rs`; exact five no-index-map fallback coverage lives in `blocked/exact_five_without_index_map.rs`; the existing exact-five all-dependency matcher remains in `blocked/exact_five_dependency_matcher.rs`.

`runtime_15_core_runtime_deactivation_blocked_tests_are_folder_backed` locks that folder-backed layout, prevents representative moved tests from returning to `blocked.rs`, preserves all 10 blocked-deactivation tests in child owners, and keeps every file in this test family under the Runtime 15 test-file budget. This is static structure evidence only; Cargo remains deferred by the Runtime 15 milestone testing cadence.
