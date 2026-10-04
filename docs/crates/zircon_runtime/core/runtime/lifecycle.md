---
related_code:
  - zircon_runtime/src/core/runtime/lifecycle.rs
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_naming_boundary.py
  - tools/plugin_structure_audits/manifest_schema.py
  - tools/tests/test_runtime_init_level_naming.py
  - tools/tests/test_plugin_structure_audit_manifest_schema_modules.py
  - zircon_runtime/src/core/runtime/mod.rs
  - zircon_runtime/src/core/mod.rs
  - zircon_runtime/src/engine_module/mod.rs
  - zircon_runtime/src/core/runtime/error.rs
  - zircon_runtime/src/core/runtime/descriptors/mod.rs
  - zircon_runtime/src/core/runtime/descriptors/module_descriptor.rs
  - zircon_runtime/src/core/runtime/descriptors/module_dependency_spec.rs
  - zircon_runtime/src/core/runtime/descriptors/module_order.rs
  - zircon_runtime/src/core/runtime/runtime.rs
  - zircon_runtime/src/core/runtime/handle/activation.rs
  - zircon_runtime/src/core/runtime/handle/activation/batch.rs
  - zircon_runtime/src/core/runtime/handle/activation/module_lifecycle.rs
  - zircon_runtime/src/core/runtime/modules/log.rs
  - zircon_runtime/src/core/runtime/modules/tasks.rs
  - zircon_runtime/src/core/runtime/modules/time.rs
  - zircon_runtime/src/core/runtime/modules/frame_count.rs
  - zircon_runtime/src/core/runtime/modules/diagnostics.rs
  - zircon_runtime/src/foundation/module.rs
  - zircon_runtime/src/platform/module.rs
  - zircon_runtime/src/input/module/descriptor.rs
  - zircon_runtime/src/asset/module.rs
  - zircon_runtime/src/asset/module/lifecycle.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/readiness.rs
  - zircon_runtime/src/scene/module/mod.rs
  - zircon_runtime/src/graphics/runtime_builtin_graphics/host/module_host/module_registration/module_descriptor.rs
  - zircon_runtime/src/script/vm/module/module_descriptor.rs
  - zircon_runtime/src/ui/module.rs
  - zircon_runtime/src/builtin/runtime_modules/assembly.rs
  - zircon_runtime/src/builtin/runtime_modules/core_modules.rs
  - zircon_runtime/src/builtin/runtime_modules/assembly/target_modules.rs
  - zircon_runtime/src/builtin/runtime_modules/assembly/profile_modules.rs
  - zircon_runtime/src/builtin/runtime_modules/load_report/report.rs
  - zircon_app/src/plugins/builder.rs
  - zircon_app/src/entry/engine_entry.rs
  - zircon_app/src/entry/builtin_modules.rs
  - zircon_editor/src/ui/host/module.rs
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
  - zircon_runtime/src/core/runtime/tests/registration/structure/mod.rs
  - zircon_runtime/src/core/runtime/tests/registration/behavior.rs
  - zircon_runtime/src/core/runtime/tests/registration/behavior/module_order.rs
  - zircon_runtime/src/core/runtime/tests/activation/behavior.rs
  - zircon_runtime/src/core/runtime/tests/activation/behavior/module_lifecycle.rs
  - zircon_runtime/src/builtin/runtime_modules/tests/registration/behavior.rs
  - zircon_app/src/plugins/tests.rs
  - zircon_runtime/src/core/runtime/tests/registration/structure/service_count_paths.rs
  - zircon_runtime/src/core/runtime/tests/registration/structure/service_list_caches.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/lock_poison_policy.rs
  - zircon_runtime/src/core/runtime/tests/activation/behavior/deactivation/blocked.rs
  - zircon_runtime/src/core/runtime/tests/activation/behavior/deactivation/blocked/external_dependents.rs
  - zircon_runtime/src/core/runtime/tests/activation/behavior/deactivation/blocked/exact_two_three_dependency_matcher.rs
  - zircon_runtime/src/core/runtime/tests/activation/behavior/deactivation/blocked/shutdown_order.rs
  - zircon_runtime/src/core/runtime/tests/activation/behavior/deactivation/blocked/exact_four_dependency_matcher.rs
  - zircon_runtime/src/core/runtime/tests/activation/behavior/deactivation/blocked/exact_five_without_index_map.rs
  - zircon_runtime/src/core/runtime/tests/activation/behavior/deactivation/blocked/exact_five_dependency_matcher.rs
  - zircon_runtime/src/core/runtime/tests/activation/structure/fixture.rs
  - zircon_runtime/src/core/runtime/tests/activation/structure/blocked_dependencies.rs
  - zircon_runtime/src/core/runtime/tests/activation/structure/blocked_unload.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/core_runtime_deactivation.rs
implementation_files:
  - zircon_runtime/src/core/runtime/tests/activation/structure/fixture.rs
  - zircon_runtime/src/core/runtime/lifecycle.rs
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_naming_boundary.py
  - tools/plugin_structure_audits/manifest_schema.py
  - zircon_runtime/src/core/runtime/mod.rs
  - zircon_runtime/src/core/mod.rs
  - zircon_runtime/src/engine_module/mod.rs
  - zircon_runtime/src/core/runtime/error.rs
  - zircon_runtime/src/core/runtime/descriptors/mod.rs
  - zircon_runtime/src/core/runtime/descriptors/module_descriptor.rs
  - zircon_runtime/src/core/runtime/descriptors/module_dependency_spec.rs
  - zircon_runtime/src/core/runtime/descriptors/module_order.rs
  - zircon_runtime/src/core/runtime/runtime.rs
  - zircon_runtime/src/core/runtime/handle/activation.rs
  - zircon_runtime/src/core/runtime/handle/activation/batch.rs
  - zircon_runtime/src/core/runtime/handle/activation/module_lifecycle.rs
  - zircon_runtime/src/core/runtime/modules/log.rs
  - zircon_runtime/src/core/runtime/modules/tasks.rs
  - zircon_runtime/src/core/runtime/modules/time.rs
  - zircon_runtime/src/core/runtime/modules/frame_count.rs
  - zircon_runtime/src/core/runtime/modules/diagnostics.rs
  - zircon_runtime/src/foundation/module.rs
  - zircon_runtime/src/platform/module.rs
  - zircon_runtime/src/input/module/descriptor.rs
  - zircon_runtime/src/asset/module.rs
  - zircon_runtime/src/asset/module/lifecycle.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/readiness.rs
  - zircon_runtime/src/scene/module/mod.rs
  - zircon_runtime/src/graphics/runtime_builtin_graphics/host/module_host/module_registration/module_descriptor.rs
  - zircon_runtime/src/script/vm/module/module_descriptor.rs
  - zircon_runtime/src/ui/module.rs
  - zircon_runtime/src/builtin/runtime_modules/assembly.rs
  - zircon_runtime/src/builtin/runtime_modules/core_modules.rs
  - zircon_runtime/src/builtin/runtime_modules/assembly/target_modules.rs
  - zircon_runtime/src/builtin/runtime_modules/assembly/profile_modules.rs
  - zircon_runtime/src/builtin/runtime_modules/load_report/report.rs
  - zircon_app/src/plugins/builder.rs
  - zircon_app/src/entry/engine_entry.rs
  - zircon_app/src/entry/builtin_modules.rs
  - zircon_editor/src/ui/host/module.rs
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
  - zircon_runtime/src/core/runtime/tests/registration/structure/mod.rs
  - zircon_runtime/src/core/runtime/tests/registration/behavior.rs
  - zircon_runtime/src/core/runtime/tests/registration/behavior/module_order.rs
  - zircon_runtime/src/core/runtime/tests/activation/behavior.rs
  - zircon_runtime/src/core/runtime/tests/activation/behavior/module_lifecycle.rs
  - zircon_runtime/src/core/runtime/tests/registration/structure/service_count_paths.rs
  - zircon_runtime/src/core/runtime/tests/registration/structure/service_list_caches.rs
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
  - docs/plans/zircon_runtime/frameworks/02-module-kernel-and-lifecycle-unification.md
  - .codex/plans/Runtime 吸收层与 Editor_Scene 边界收束计划.md
tests:
  - zircon_runtime/src/core/runtime/tests/activation/structure/blocked_dependencies.rs
  - zircon_runtime/src/core/runtime/tests/activation/structure/blocked_unload.rs
  - zircon_runtime/src/tests/runtime_absorption/root_entries.rs
  - tools/tests/test_runtime_init_level_naming.py
  - tools/tests/test_plugin_structure_audit_manifest_schema_modules.py
  - zircon_runtime::tests::runtime_absorption::structure_convention::production_file_budget::runtime_15_core_runtime_service_lists_are_folder_backed
  - zircon_runtime/src/core/runtime/handle/core_handle.rs::tests::core_handle_registry_accessors_recover_poisoned_runtime_locks
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/lock_poison_policy.rs::runtime_15_core_handle_registry_lock_poison_recovery_guard_covers_registry_accessors
  - zircon_runtime::tests::runtime_absorption::structure_convention::test_file_budget::core_runtime_deactivation::runtime_15_core_runtime_deactivation_blocked_tests_are_folder_backed
  - zircon_runtime::core::runtime::tests::registration::behavior::module_order::module_descriptor_defaults_to_post_without_module_dependencies
  - zircon_runtime::core::runtime::tests::registration::behavior::module_order::module_activation_order_sorts_levels_and_declared_dependencies
  - zircon_runtime::core::runtime::tests::registration::behavior::module_order::module_activation_order_rejects_missing_module_dependency
  - zircon_runtime::core::runtime::tests::registration::behavior::module_order::module_activation_order_rejects_dependency_on_later_init_level
  - zircon_runtime::core::runtime::tests::registration::behavior::module_order::module_activation_order_reports_same_level_cycles
  - zircon_runtime::core::runtime::tests::registration::behavior::module_order::module_lifecycle_default_hooks_are_noop_and_ready
  - zircon_runtime::core::runtime::tests::activation::behavior::module_lifecycle::module_lifecycle_hooks_wrap_activation_and_deactivation
  - zircon_runtime::core::runtime::tests::activation::behavior::module_lifecycle::module_ready_polling_allows_later_ready_result
  - zircon_runtime::core::runtime::tests::activation::behavior::module_lifecycle::module_ready_timeout_resets_module_and_started_services
  - zircon_runtime::core::runtime::tests::activation::behavior::module_lifecycle::module_finish_error_resets_module_and_started_services
  - zircon_runtime::core::runtime::tests::activation::behavior::module_lifecycle::activate_registered_modules_finishes_only_after_all_modules_are_ready
  - zircon_runtime::core::runtime::tests::activation::behavior::module_lifecycle::activate_registered_modules_rolls_back_all_started_modules_on_finish_error
  - zircon_runtime::builtin::runtime_modules::tests::registration::behavior::target_runtime_modules_follow_descriptor_activation_order
  - zircon_app::plugins::tests::builtin_plugin_groups_finish_in_descriptor_activation_order
  - rustc --edition 2021 --test zircon_runtime/src/tests/runtime_absorption/root_entries.rs
  - rustfmt --edition 2021 --check zircon_runtime/src/core/runtime/handle/registration/service_lists/types.rs zircon_runtime/src/core/runtime/handle/registration/service_lists/specialized.rs zircon_runtime/src/core/runtime/handle/registration/service_lists/mod.rs zircon_runtime/src/core/runtime/handle/registration/register_module.rs
  - rustfmt --edition 2021 --check --config skip_children=true zircon_runtime/src/core/runtime/lifecycle.rs zircon_runtime/src/core/runtime/descriptors/module_descriptor.rs zircon_runtime/src/core/runtime/descriptors/module_dependency_spec.rs zircon_runtime/src/core/runtime/descriptors/module_order.rs zircon_runtime/src/core/runtime/descriptors/mod.rs zircon_runtime/src/core/runtime/mod.rs zircon_runtime/src/core/mod.rs zircon_runtime/src/engine_module/mod.rs zircon_runtime/src/core/runtime/error.rs zircon_runtime/src/core/runtime/tests/registration/behavior.rs zircon_runtime/src/core/runtime/tests/registration/behavior/module_order.rs
  - rustfmt --edition 2021 --check --config skip_children=true zircon_runtime/src/core/runtime/lifecycle.rs zircon_runtime/src/core/runtime/descriptors/module_descriptor.rs zircon_runtime/src/core/runtime/handle/activation.rs zircon_runtime/src/core/runtime/handle/activation/module_lifecycle.rs zircon_runtime/src/core/runtime/runtime.rs zircon_runtime/src/core/runtime/mod.rs zircon_runtime/src/core/mod.rs zircon_runtime/src/engine_module/mod.rs zircon_runtime/src/core/runtime/error.rs zircon_runtime/src/core/runtime/tests/activation/behavior.rs zircon_runtime/src/core/runtime/tests/activation/behavior/module_lifecycle.rs
  - rustfmt --edition 2021 --check --config skip_children=true zircon_runtime/src/core/runtime/handle/activation/batch.rs zircon_runtime/src/core/runtime/handle/activation.rs zircon_runtime/src/core/runtime/runtime.rs zircon_runtime/src/core/runtime/tests/activation/behavior/module_lifecycle.rs
  - rustfmt --edition 2021 --check --config skip_children=true zircon_runtime/src/builtin/runtime_modules/core_modules.rs zircon_runtime/src/builtin/runtime_modules/assembly/target_modules.rs zircon_runtime/src/builtin/runtime_modules/assembly/profile_modules.rs zircon_runtime/src/builtin/runtime_modules/load_report/report.rs zircon_runtime/src/builtin/runtime_modules/assembly.rs zircon_app/src/plugins/builder.rs zircon_app/src/entry/engine_entry.rs zircon_app/src/entry/builtin_modules.rs zircon_runtime/src/builtin/runtime_modules/tests/registration/behavior.rs zircon_app/src/plugins/tests.rs
  - cargo check -p zircon_app --lib --locked --no-default-features --features target-server --jobs 1 --target-dir E:/cargo-targets/zircon-runtime-frameworks-m2-0703 --message-format short --color never
  - cargo check -p zircon_runtime --lib --locked --no-default-features --features core-min --jobs 1 --target-dir E:/cargo-targets/zircon-runtime-frameworks-coremin-0703 --message-format short --color never
  - cargo check -p zircon_runtime --lib --locked
  - cargo test -p zircon_runtime module_activation_order --lib --locked --jobs 1 --target-dir E:/cargo-targets/zircon-runtime-frameworks-0702 --message-format short --color never -- --nocapture --test-threads=1
doc_type: module-detail
---

# Runtime Lifecycle Vocabulary

`zircon_runtime::core::runtime::lifecycle` owns the service and module lifecycle vocabulary used by runtime registration, activation, deactivation, and resolution. The former `core/lifecycle.rs` root fragment was moved here during runtime plan 02 M2.2.

## Ownership Boundary

- `StartupMode` describes whether a registered service starts immediately or waits for lazy resolution.
- `InitLevel` describes module activation layers in order: Kernel, Services, Scene, Editor, and Post. New descriptors default to Post until they opt into an earlier level. `Services` owns non-network runtime infrastructure such as platform, input, and assets; the former ambiguous `Servers` name was removed without a compatibility alias.
- `LifecycleState` describes the runtime state of modules and services: registered, initializing, running, stopping, or unloaded.
- `ServiceKind` is the canonical driver/manager/plugin classifier used by `RegistryName`, dependency validation, and service table logic.
- `ModuleLifecycle` defines the shared build/ready/finish/cleanup hook vocabulary. The default implementation is behavior-preserving: build, finish, and cleanup are no-ops, while ready returns true.
- The curated `zircon_runtime::core::{LifecycleState, StartupMode, ServiceKind}` facade remains because these types are public runtime vocabulary, but the physical owner is now the runtime kernel.

The lifecycle module defines vocabulary only. Registration ordering, dependency validation, activation, deactivation, and resolution behavior stay in their existing runtime handle and descriptor owners. Frameworks 02 M1 wires descriptor ordering and lifecycle hooks through those existing owners instead of moving behavior into the root lifecycle vocabulary module.

Frameworks 02 M3 extends that ownership rule through runtime plugins. Every `RuntimePluginDescriptor` embeds the kernel `ModuleDescriptor`, and `RuntimePluginRegistrationReport::from_plugin(...)` registers that value exactly once before collecting non-module extensions. First-party providers bind their concrete descriptor through `.with_module_descriptor(...)`; the SDK module registration builder now accepts only an owner name. Plugin-only ready/finish/activate/deactivate contexts and a second provider-side module registration path are not retained, so build/ready/finish/cleanup and manifest projection cannot diverge from the descriptor activated by `CoreRuntime`.

The same hard cut applies to identity. First-party runtime module descriptors use the manifest-owned `<package>.runtime` namespace, and manager/driver/plugin registry names preserve that full module namespace. `RegistryName` therefore parses the final `.Driver|Manager|Plugin.<service>` suffix from the right instead of assuming an exact three-segment string; module namespaces may contain clean non-empty dot-separated segments, while service names remain dot-free. Old PascalCase module identities are not aliased.

## Runtime 15 non-network layer naming hard cutover

`runtime_15_lifecycle_services_init_level_naming_coremin_check_passed_frameworks_plan_mirror_pending`

The ambiguous `InitLevel::Servers` variant was removed without an alias and replaced by `InitLevel::Services`. All runtime source callers and lifecycle/order tests use the new name, so platform, input, and asset initialization cannot be confused with a network-server layer. Plugin manifest validation accepts the matching `services` serde value and explicitly rejects the retired `servers` value, preventing Rust/tooling schema divergence. The runtime naming audit classifies the legitimate `Editor` init level as a runtime-profile editor-host target instead of migration debt.

The naming and manifest-schema suites pass 8/8, the direct non-network server audit reports zero references, zero migration debt, `classified-and-clear`, and no risks, and the runtime naming audit's unclassified editor references dropped from six to four. The runtime core-min library check passes with existing warnings. Frameworks plan mirrors still contain the historical name and remain assigned to the active Frameworks plan owner; no compatibility source path was retained.

## Frameworks 02 M1 lifecycle/order foundation

`frameworks_02_m1_lifecycle_order_foundation_rustfmt_lib_check_passed_tests_blocked`

Frameworks 02 M1 establishes the new lifecycle vocabulary and ordering contracts without adding compatibility aliases for retired architecture. `InitLevel` is exported through `core`, `runtime`, and `engine_module` as canonical module kernel vocabulary. `ModuleDescriptor` now carries `init_level` and `module_dependencies`, with default `InitLevel::Post` and an empty dependency set so existing descriptors keep their current order until they declare newer semantics.

`ModuleDependencySpec` and `sort_module_activation_order(...)` live under the descriptor owner. The sorter rejects duplicate modules, missing module dependencies, dependencies on later init levels, and same-level dependency cycles with typed `CoreError` variants instead of string diagnostics. Traversal is stable by init level and declaration index, and dependency lookup returns typed errors rather than panicking on an internal map miss.

The focused behavior tests cover descriptor defaults, level/dependency ordering, missing dependencies, later-level dependency rejection, same-level cycle reporting, and default lifecycle hooks. `rustfmt --edition 2021 --check --config skip_children=true` passed for the touched files, and `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-frameworks-0702 --message-format short --color never` passed with existing repository warnings. Direct lib-test execution is not counted as passing because the current test build is blocked before these tests execute by the active `graphics/text/raster/swash.rs` lib-test dependency resolution issue (`could not find swash in the list of imported crates`).

## Frameworks 02 M1 activation lifecycle progression

`frameworks_02_m1_activation_lifecycle_progression_rustfmt_passed_cargo_blocked`

The second M1 slice connects the lifecycle vocabulary to single-module activation without changing builtin profile assembly yet. `ModuleDescriptor` now owns an `Arc<dyn ModuleLifecycle>` and defaults to `NoopModuleLifecycle`, so descriptors that do not opt into hooks keep the previous activation behavior without a compatibility branch. `CoreHandle::activate_module_with_ready_timeout(...)` runs build, resolves immediate services, polls ready within the caller-supplied budget, runs finish, and only then marks the module Running. `activate_module(...)` uses a zero ready budget, which is behavior-preserving for the default ready=true path and returns a typed `ModuleReadyTimeout` for modules that explicitly report not ready.

`core/runtime/handle/activation/module_lifecycle.rs` owns the hook invocation, ready polling, typed timeout construction, and startup-service rollback helper. Activation failures after startup now reset the initializing module and the module's immediate startup service entries to Registered with no instance, avoiding a half-running module after ready timeout or finish failure. Deactivation calls cleanup before plugin bridge deactivation and service unload, keeping cleanup in the PreDeactivation window while services are still available.

Frameworks 02 M1 correction now also treats a successful `build` as an acquired lifecycle resource. If service resolution, `ready`, `finish`, or final activation fails, the single-module path calls `cleanup` before clearing services and state. Batch activation tracks the successfully built prefix and cleans it in reverse dependency order. Cleanup is best-effort across the whole built prefix; typed `ModuleActivationRollback` and `ModuleBatchActivationRollback` errors preserve the primary activation error when cleanup also fails instead of silently discarding either failure.

Focused tests were added for hook order across activation/deactivation, ready polling that becomes true, ready timeout rollback, and finish-error rollback. Scoped rustfmt passed for the touched activation/runtime/test files. Cargo validation is currently blocked by unrelated active shader/material work: `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-frameworks-0702 --message-format short --color never` stops in `asset/artifact/cache_payload.rs` (`ZMaterialQueueOverride` not exported from `asset`) and `graphics/pipeline/declarations/renderer_feature_contract_diagnostic.rs` (new `RenderMaterialValidationError` variants not covered). This section does not claim Cargo or focused test green for the activation lifecycle slice.

## Frameworks 02 M1 batch activation finish barrier

`frameworks_02_m1_batch_activation_finish_barrier_rustfmt_passed_cargo_blocked`

The third M1 slice adds the cross-module activation phase barrier needed before M2 can replace builtin hand ordering. `CoreHandle::activate_registered_modules_with_ready_timeout(...)` snapshots registered module descriptors, sorts them through `sort_module_activation_order(...)`, marks all non-running modules Initializing, then runs build for the full ordered set, startup-service resolution for the full set, ready polling for the full set, and finish for the full set. Modules are marked Running only after every finish hook succeeds, so a module cannot observe another module as Running before the batch has crossed the finish barrier.

`core/runtime/handle/activation/batch.rs` owns this batch progression and rollback. If build, startup-service resolution, ready, or finish fails, every pending module in the batch is reset to Registered and every immediate startup service resolved during the batch is reset to Registered with no instance. The single-module activation path remains available for direct/lazy resolution, while the registered-module batch API is the planned kernel entry for profile assembly and later RuntimePlugin convergence.

Focused tests cover sorted Kernel/Services/Scene batch ordering with no finish before all ready hooks, and finish-failure rollback across multiple modules plus their immediate services. Scoped rustfmt passed for the touched batch/activation/runtime/test files. Cargo validation remains blocked outside this owner by active shader/material drift: the current runtime lib check stops in graphics scene material paths because `MaterialDisabledPasses` is not exported from `graphics::scene::resources`, and `CachedMeshDrawKey` initializers now miss `disabled_passes`. This section does not claim Cargo or focused test green for the batch slice.

## Frameworks 02 M2 descriptor-sorted builtin/profile assembly

`frameworks_02_m2_builtin_profile_descriptor_sorting_rustfmt_app_server_check_passed`

Frameworks 02 M2 switches builtin module and app plugin assembly from handwritten order to descriptor-owned order. Runtime profiles own typed `BuiltinRuntimeModuleId` membership and app plugin groups declare only a profile plus explicit app features. Target assembly builds one builtin candidate registry, profile selection completes the dependency closure, and the target owner calls `sort_module_activation_order(...)` once after plugin modules join the set. `ModuleDescriptor::init_level` plus `ModuleDependencySpec` decide the activation sequence, so missing dependencies or layer violations become typed runtime load diagnostics instead of silent list drift.

The builtin candidate set now includes the kernel foundations directly in `runtime_core_module_candidates_for_target_with_render_features(...)`: Foundation, log, tasks, time, frame count, diagnostics core, platform, input, asset, scene, optional graphics, and script. Minimal's exact five-module membership appears only in its profile descriptor; there is no Minimal-specific constructor or app-side filter. Target/profile load reports convert candidate, closure, and sorting failures into fatal diagnostics, and `zircon_app` checks those diagnostics before bootstrapping instead of continuing with a partially ordered list.

`AssetModule` is the first production built-in to adopt the readiness hook. Its immediate `ProjectAssetManager` already owns the project-catalog generation gate used while a generation is published. `AssetModuleLifecycle::ready(...)` resolves that manager through the runtime registry and probes the gate with `try_read()`: a concurrent generation publication reports not ready, while an idle or poison-recovered gate reports ready. The hook performs no blocking I/O, busy loop, sleep, or parallel readiness state. The focused integration test holds the real generation write guard to prove false, releases it, then proves true through the lifecycle stored in the production descriptor.

App plugin groups gained `try_finish(...)`, which sorts enabled entries by descriptor order and returns a structured `PluginGroupError::ModuleOrder` when group membership violates the kernel contract. `finish(...)` remains as the assertion-style convenience path for built-in groups. `EngineEntry` and `BuiltinEngineEntry` now register all selected descriptors first and activate through `CoreRuntime::activate_registered_modules(...)`, so M2 consumes the M1 batch finish barrier rather than reintroducing per-module Running publication.

Descriptor declarations were added to the builtin modules: Kernel level for foundation/log/tasks/time/frame count/diagnostics, Services level for platform/input/asset, Scene level for scene/graphics/UI, Editor level for the editor module, and Post for script. Dependencies now encode the expected lower-layer owners; for example frame count depends on time, asset depends on foundation and tasks, graphics depends on platform/asset/scene, UI depends on input/scene/graphics, and editor depends on the runtime/editor-facing module set. No fallback list or old-order compatibility path was kept.

Focused tests were added for target runtime module order and built-in app plugin group order. Scoped rustfmt passed for the M2 runtime/app/editor descriptor, assembly, bootstrap, and test files. The first app server check exposed an owned `PluginGroupBuilder::try_finish(...)` return-type bug; the fix wraps the descriptor-sorted group in `Ok(ResolvedPluginGroup { ... })`. After that correction, `cargo check -p zircon_app --lib --locked --no-default-features --features target-server --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-frameworks-m2-0703 --message-format short --color never` passed with existing repository warnings. Focused runtime/app tests are still not counted as passing for M2 because the milestone has not run the declared test stage.

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

## Runtime 02 activation structure-fixture reconciliation

`runtime_02_core_activation_structure_fixture_inventory_static_passed` keeps the structure guards aligned with that folder split. The shared `activation_tests_source()` fixture now includes the exact-two/three matcher owner, exact-four matcher owner, and exact-five no-index-map owner in addition to the pre-existing exact-five dependency matcher source. This restores source-level evidence for both blocked-dependency and blocked-unload guards without moving behavior back into the former oversized parent.

The standalone activation structure harness passes the two affected guards 2/2. The default-feature `core::` package filter executes 641 tests as 629 passed and 12 failed; the two activation structure failures are closed by this slice, while the remaining Render/UI failures belong to their active owners and are not promoted as Runtime 02 success evidence.
