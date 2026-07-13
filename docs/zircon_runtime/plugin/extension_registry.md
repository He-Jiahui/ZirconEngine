---
related_code:
  - zircon_runtime/src/plugin/extension_registry/mod.rs
  - zircon_runtime/src/plugin/extension_registry/runtime_extension_registry.rs
  - zircon_runtime/src/plugin/extension_registry/runtime_extension_registry/tests.rs
  - zircon_runtime/src/plugin/extension_registry/typed_extension_point.rs
  - zircon_runtime/src/plugin/extension_registry/ownership.rs
  - zircon_runtime/src/plugin/extension_registry/owner.rs
  - zircon_runtime/src/asset/importer/registry.rs
  - zircon_runtime/src/plugin/extension_registry/register.rs
  - zircon_runtime/src/plugin/extension_registry/register/system_registration.rs
  - zircon_runtime/src/plugin/extension_registry/register/resource_registration.rs
  - zircon_runtime/src/plugin/extension_registry/register/event_registration.rs
  - zircon_runtime/src/plugin/extension_registry/register/runtime_scene_system_registration.rs
  - zircon_runtime/src/plugin/extension_registry/register/bridge_registration.rs
  - zircon_runtime/src/plugin/extension_registry/validation.rs
  - zircon_runtime/src/plugin/extension_registry/validation/token.rs
  - zircon_runtime/src/plugin/extension_registry/validation/component.rs
  - zircon_runtime/src/plugin/extension_registry/validation/scene_hook.rs
  - zircon_runtime/src/plugin/extension_registry/apply_to_world.rs
  - zircon_runtime/src/plugin/extension_registry/apply_to_world/component.rs
  - zircon_runtime/src/plugin/extension_registry/apply_to_asset_manager.rs
  - zircon_runtime/src/plugin/extension_registry/apply_to_module/runtime_core.rs
  - zircon_runtime/src/plugin/extension_registry/apply_to_ui/component.rs
  - zircon_runtime/src/plugin/extension_registry_error.rs
  - zircon_runtime/src/core/framework/bridge/mod.rs
  - zircon_runtime/src/core/framework/bridge/interface_slot.rs
  - zircon_runtime/src/plugin/bridge.rs
  - zircon_runtime/src/plugin/bridge/table.rs
  - zircon_runtime/src/core/framework/bridge/strong.rs
  - zircon_runtime/src/plugin/bridge/weak.rs
  - zircon_runtime/src/core/runtime/state/world_runtime_extensions.rs
  - zircon_runtime/src/core/runtime/state/world_runtime_extensions/tests.rs
  - zircon_runtime/tests/runtime_plugin_world_extensions_contract.rs
  - zircon_runtime/src/core/runtime/handle/runtime_extensions.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/contributions/extension.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/extension_report/runtime.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/project_extension_report.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/project_extension_report/runtime_merge.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/access.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/diagnostics.rs
  - zircon_runtime/src/plugin/runtime_plugin/capability_view.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/registration/constructors.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/registration/order.rs
  - zircon_runtime/src/plugin/runtime_plugin/descriptor.rs
  - zircon_runtime/src/plugin/runtime_plugin/descriptor/package_manifest/runtime_module.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/modules/row/systems.rs
  - zircon_runtime/src/plugin/runtime_plugin/registration_report/validation/system_anchors.rs
  - zircon_runtime/src/plugin/runtime_plugin/registration_report/plugin.rs
  - zircon_runtime/src/plugin/runtime_plugin/feature_registration_report/feature.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin/plugin.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin/feature.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/hot_reload.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/lifecycle.rs
  - zircon_runtime/src/plugin/native_plugin_loader/host_api_adapter.rs
  - zircon_editor/src/core/editor_plugin.rs
  - zircon_editor/src/core/editor_plugin_catalog_gen.rs
  - zircon_editor/build.rs
  - zircon_runtime_interface/src/plugin_diagnostics.rs
  - zircon_runtime_interface/src/plugin_api.rs
  - zircon_runtime_interface/src/buffer.rs
  - zircon_runtime/src/scene/ecs/system_set.rs
  - zircon_runtime/src/scene/ecs/scene_system_descriptor.rs
  - zircon_runtime/src/scene/ecs/schedule_stage_plan.rs
  - zircon_runtime/src/scene/ecs/system/native/scene_system_metadata.rs
  - zircon_runtime/src/scene/ecs/system/native/runtime_scene_system.rs
  - zircon_runtime/src/scene/world/events.rs
  - zircon_runtime/src/scene/world/schedule.rs
  - zircon_plugins/first_party_runtime_catalog/src/tests/provider_snapshot.rs
  - tools/plugin_structure_audits/registration.py
  - tools/audit_plugin_structure.py
implementation_files:
  - zircon_runtime/src/plugin/extension_registry/runtime_extension_registry.rs
  - zircon_runtime/src/plugin/extension_registry/runtime_extension_registry/tests.rs
  - zircon_runtime/src/plugin/extension_registry/typed_extension_point.rs
  - zircon_runtime/src/plugin/extension_registry/ownership.rs
  - zircon_runtime/src/plugin/extension_registry/owner.rs
  - zircon_runtime/src/asset/importer/registry.rs
  - zircon_runtime/src/plugin/extension_registry/register/system_registration.rs
  - zircon_runtime/src/plugin/extension_registry/register/resource_registration.rs
  - zircon_runtime/src/plugin/extension_registry/register/event_registration.rs
  - zircon_runtime/src/plugin/extension_registry/register/runtime_scene_system_registration.rs
  - zircon_runtime/src/plugin/extension_registry/register/bridge_registration.rs
  - zircon_runtime/src/plugin/extension_registry/validation.rs
  - zircon_runtime/src/plugin/extension_registry/validation/token.rs
  - zircon_runtime/src/plugin/extension_registry/validation/component.rs
  - zircon_runtime/src/plugin/extension_registry/validation/scene_hook.rs
  - zircon_runtime/src/plugin/extension_registry/apply_to_world.rs
  - zircon_runtime/src/plugin/extension_registry/apply_to_world/component.rs
  - zircon_runtime/src/plugin/extension_registry/apply_to_asset_manager.rs
  - zircon_runtime/src/plugin/extension_registry/apply_to_module/runtime_core.rs
  - zircon_runtime/src/plugin/extension_registry/apply_to_ui/component.rs
  - zircon_runtime/src/core/framework/bridge/mod.rs
  - zircon_runtime/src/core/framework/bridge/interface_slot.rs
  - zircon_runtime/src/plugin/bridge.rs
  - zircon_runtime/src/plugin/bridge/table.rs
  - zircon_runtime/src/core/framework/bridge/strong.rs
  - zircon_runtime/src/plugin/bridge/weak.rs
  - zircon_runtime/src/core/runtime/state/world_runtime_extensions.rs
  - zircon_runtime/src/core/runtime/state/world_runtime_extensions/tests.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/contributions/extension.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/extension_report/runtime.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/project_extension_report.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/access.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/diagnostics.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/project_extension_report/runtime_merge.rs
  - zircon_runtime/src/plugin/runtime_plugin/capability_view.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/registration/constructors.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/registration/order.rs
  - zircon_runtime/src/plugin/runtime_plugin/descriptor.rs
  - zircon_runtime/src/plugin/runtime_plugin/descriptor/package_manifest/runtime_module.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/modules/row/systems.rs
  - zircon_runtime/src/plugin/runtime_plugin/registration_report/validation/system_anchors.rs
  - zircon_runtime/src/plugin/runtime_plugin/registration_report/plugin.rs
  - zircon_runtime/src/plugin/runtime_plugin/feature_registration_report/feature.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin/plugin.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin/feature.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/hot_reload.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/lifecycle.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/tests.rs
  - zircon_runtime/src/plugin/native_plugin_loader/host_api_adapter.rs
  - zircon_editor/src/core/editor_plugin.rs
  - zircon_editor/src/core/editor_plugin_catalog_gen.rs
  - zircon_editor/build.rs
  - zircon_runtime_interface/src/plugin_diagnostics.rs
  - zircon_runtime_interface/src/plugin_api.rs
  - zircon_runtime_interface/src/buffer.rs
  - zircon_runtime/src/scene/ecs/system_set.rs
  - zircon_runtime/src/scene/ecs/schedule_stage_plan.rs
  - zircon_runtime/src/scene/ecs/system/native/runtime_scene_system.rs
  - zircon_plugins/first_party_runtime_catalog/src/tests/provider_snapshot.rs
  - tools/plugin_structure_audits/registration.py
  - tools/audit_plugin_structure.py
plan_sources:
  - docs/plans/zircon_runtime/frameworks/03-optional-features-and-profile-matrix.md
  - user: 2026-07-10 implement runtime architecture plans and prioritize structure/review findings
  - docs/plans/zircon_runtime/runtime/06-plugin-surface-and-lifecycle.md
  - docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md
  - .codex/plans/Zircon Runtime 架构渐进式 Review 与优化计划.md
  - user: 2026-07-10 execute frameworks architecture hard-cutover refactor and validation
  - docs/plans/zircon_runtime/frameworks/02-module-kernel-and-lifecycle-unification.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
  - user: 2026-06-12 implement docs/plans/zircon_plugins plugin architecture code
  - docs/plans/zircon_plugins/index.md
  - docs/plans/zircon_plugins/01-plugin-architecture-core.md
  - docs/plans/zircon_plugins/04-animation.md
  - docs/plans/zircon_plugins/08-zr-vm.md
  - docs/plans/zircon_plugins/11-plugin-call-bridge.md
tests:
  - tools/tests/test_frameworks_03_server_feature_boundary.py
  - tools/tests/test_plugin_extension_registry_finalize_coverage.py
  - zircon_runtime/src/plugin/extension_registry/typed_extension_point/tests.rs
  - zircon_runtime/src/plugin/extension_registry/runtime_extension_registry/tests.rs
  - zircon_runtime/src/core/runtime/state/world_runtime_extensions/tests.rs
  - zircon_runtime/tests/runtime_plugin_world_extensions_contract.rs
  - zircon_plugins/animation/runtime/tests/runtime_physics_animation_tick_contract/state_machine_interruption.rs::pose_targets_visible_to_physics_step
  - zircon_runtime/src/tests/plugin_extensions/extension_registry_typed_points.rs
  - zircon_runtime/src/tests/plugin_extensions/extension_registry_bridge.rs
  - zircon_runtime/src/tests/plugin_extensions/extension_registry_event_catalogs.rs
  - zircon_runtime/src/tests/plugin_extensions/extension_registry_components.rs
  - zircon_runtime/src/tests/plugin_extensions/extension_registry_scene_hooks.rs
  - zircon_runtime/src/tests/plugin_extensions/runtime_plugin_lifecycle.rs
  - zircon_runtime/src/tests/plugin_extensions/runtime_plugin_lifecycle.rs::registration_report_catalog_orders_runtime_extensions_by_module_descriptor
  - zircon_runtime/src/tests/plugin_extensions/runtime_plugin_lifecycle.rs::registration_report_catalog_rejects_invalid_module_order_before_extension_merge
  - zircon_runtime/src/tests/plugin_extensions/runtime_plugin_lifecycle.rs::project_registration_report_catalog_orders_enabled_runtime_extensions
  - zircon_runtime/src/tests/plugin_extensions/runtime_plugin_lifecycle.rs::project_registration_report_catalog_rejects_invalid_enabled_module_order
  - zircon_runtime/src/tests/plugin_extensions/runtime_plugin_descriptor.rs
  - zircon_plugins/first_party_runtime_catalog/src/tests/provider_snapshot.rs
  - tools/tests/test_plugin_structure_audit_registration.py
  - zircon_runtime/src/tests/plugin_extensions/extension_registry_systems.rs
  - zircon_runtime/src/tests/plugin_extensions/extension_registry_metadata.rs
  - zircon_runtime/src/tests/plugin_extensions/static_manifest_contracts/modules/system_anchors.rs
  - zircon_editor/src/tests/editor_plugin_catalog_consistency.rs::editor_plugin_catalog_reports_missing_capabilities_as_structured_diagnostics
  - zircon_runtime/src/scene/tests/ecs_schedule.rs
  - zircon_runtime/src/tests/plugin_extensions/package_manifest_declarations.rs
  - zircon_editor/src/tests/editor_plugin_catalog_consistency.rs::builtin_editor_catalog_entries_are_derived_from_plugin_manifests
  - zircon_editor/src/tests/editor_plugin_catalog_consistency.rs::editor_module_plugin_manifests_are_present_in_builtin_catalog
  - rustfmt zircon_runtime/src/plugin/extension_registry/typed_extension_point.rs zircon_runtime/src/plugin/extension_registry/ownership.rs zircon_runtime/src/plugin/extension_registry/mod.rs zircon_runtime/src/plugin/extension_registry/runtime_extension_registry.rs zircon_runtime/src/plugin/extension_registry/register.rs zircon_runtime/src/plugin/extension_registry/register/metadata.rs zircon_runtime/src/plugin/extension_registry/register/runtime_core.rs zircon_runtime/src/plugin/extension_registry/register/scene_hook.rs zircon_runtime/src/plugin/extension_registry/register/system_registration.rs zircon_runtime/src/plugin/extension_registry/register/resource_registration.rs zircon_runtime/src/plugin/extension_registry/register/event_registration.rs zircon_runtime/src/plugin/mod.rs zircon_runtime/src/tests/plugin_extensions/extension_registry_systems.rs zircon_runtime/src/tests/plugin_extensions/extension_registry_metadata.rs
  - cargo check -p zircon_runtime --lib --locked --message-format short
  - cargo test -p zircon_runtime --lib plugin_extensions::extension_registry_systems --locked --message-format short -- --nocapture
  - cargo check -p zircon_editor --lib --locked --message-format short
  - cargo test -p zircon_editor --lib editor_plugin_catalog_consistency --locked --message-format short -- --nocapture
  - cargo test -p zircon_runtime_interface --lib plugin_api_contracts --locked --target-dir D:\cargo-targets\zircon-plugin-architecture-0612 --message-format short -- --nocapture
  - cargo test -p zircon_runtime_interface --lib abi_v3_layout_is_stable --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-interface-abi-v3-0613 --message-format short --color never -- --test-threads=1 --nocapture
  - cargo test -p zircon_runtime --lib native_system_enters_schedule_as_conservative_node --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-m5-check-coremin-0613b --message-format short --color never -- --test-threads=1 --nocapture (2026-06-13: passed 1 focused test after earlier non-assertion target/process/render compile blockers)
  - D:\cargo-targets\zircon-plugin-architecture-m5-check-coremin-0613b\debug\deps\zircon_runtime-5d2828c2001649f6.exe failed_registration_revoked_via_ownership --test-threads=1 --nocapture (2026-06-13: passed 1 focused test)
  - cargo test -p zircon_runtime --lib native_hot_reload --locked --target-dir D:\cargo-targets\zircon-plugin-architecture-0612 --message-format short -- --nocapture
  - rustfmt --check zircon_runtime/src/plugin/runtime_plugin/descriptor.rs zircon_runtime/src/plugin/runtime_plugin/descriptor/builder/construction.rs zircon_runtime/src/plugin/runtime_plugin/descriptor/builder/fluent.rs zircon_runtime/src/plugin/runtime_plugin/descriptor/package_manifest/runtime_module.rs zircon_runtime/src/plugin/runtime_plugin/package_validation/modules/row.rs zircon_runtime/src/plugin/runtime_plugin/package_validation/modules/row/systems.rs zircon_runtime/src/plugin/runtime_plugin/registration_report/validation.rs zircon_runtime/src/plugin/runtime_plugin/registration_report/validation/system_anchors.rs zircon_runtime/src/plugin/runtime_plugin/registration_report/plugin.rs zircon_runtime/src/tests/plugin_extensions/runtime_plugin_descriptor.rs zircon_runtime/src/tests/plugin_extensions/runtime_plugin_package_manifest.rs
  - git diff --check -- zircon_runtime/src/plugin/runtime_plugin/descriptor.rs zircon_runtime/src/plugin/runtime_plugin/descriptor/builder/construction.rs zircon_runtime/src/plugin/runtime_plugin/descriptor/builder/fluent.rs zircon_runtime/src/plugin/runtime_plugin/descriptor/package_manifest/runtime_module.rs zircon_runtime/src/plugin/runtime_plugin/package_validation/modules/row.rs zircon_runtime/src/plugin/runtime_plugin/package_validation/modules/row/systems.rs zircon_runtime/src/plugin/runtime_plugin/registration_report/validation.rs zircon_runtime/src/plugin/runtime_plugin/registration_report/validation/system_anchors.rs zircon_runtime/src/plugin/runtime_plugin/registration_report/plugin.rs zircon_runtime/src/tests/plugin_extensions/runtime_plugin_descriptor.rs zircon_runtime/src/tests/plugin_extensions/runtime_plugin_package_manifest.rs
  - cargo test -p zircon_runtime --lib plugin_extensions::runtime_plugin_descriptor --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-0612 --message-format short --color never -- --nocapture (attempted 2026-06-12, timed out after 10 minutes under concurrent runtime Cargo load; no pass claimed)
  - rustfmt --check zircon_runtime/src/asset/importer/registry.rs zircon_runtime/src/plugin/extension_registry/typed_extension_point.rs zircon_runtime/src/plugin/extension_registry/ownership.rs zircon_runtime/src/plugin/extension_registry/runtime_extension_registry.rs zircon_runtime/src/tests/plugin_extensions/extension_registry_metadata.rs
  - git diff --check -- zircon_runtime/src/asset/importer/registry.rs zircon_runtime/src/plugin/extension_registry/typed_extension_point.rs zircon_runtime/src/plugin/extension_registry/ownership.rs zircon_runtime/src/plugin/extension_registry/runtime_extension_registry.rs zircon_runtime/src/tests/plugin_extensions/extension_registry_metadata.rs
  - cargo test -p zircon_runtime --lib runtime_extension_registry_revokes_owner_tracked_contributions --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-0612 --message-format short --color never -- --nocapture
  - rustfmt --edition 2021 --check zircon_runtime/src/plugin/extension_registry/register/system_registration.rs zircon_runtime/src/plugin/extension_registry/register/resource_registration.rs zircon_runtime/src/plugin/extension_registry/register/event_registration.rs zircon_runtime/src/plugin/extension_registry/register/runtime_scene_system_registration.rs zircon_runtime/src/plugin/extension_registry/apply_to_world.rs zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/contributions/extension.rs zircon_runtime/src/tests/plugin_extensions/extension_registry.rs zircon_runtime/src/scene/module/mod.rs
  - git diff --check -- zircon_runtime/src/plugin/extension_registry/register/system_registration.rs zircon_runtime/src/plugin/extension_registry/register/resource_registration.rs zircon_runtime/src/plugin/extension_registry/register/event_registration.rs zircon_runtime/src/plugin/extension_registry/register/runtime_scene_system_registration.rs zircon_runtime/src/plugin/extension_registry/apply_to_world.rs zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/contributions/extension.rs zircon_runtime/src/tests/plugin_extensions/extension_registry.rs zircon_runtime/src/scene/module/mod.rs
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-0612 --message-format short --color never
  - rustfmt --edition 2021 --check zircon_runtime/src/plugin/runtime_plugin/runtime_plugin/plugin.rs zircon_runtime/src/plugin/runtime_plugin/runtime_plugin/feature.rs zircon_runtime/src/plugin/runtime_plugin/registration_report/plugin.rs zircon_runtime/src/plugin/runtime_plugin/feature_registration_report/feature.rs zircon_runtime/src/tests/plugin_extensions/runtime_plugin_lifecycle.rs zircon_runtime/src/tests/plugin_extensions/mod.rs
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-lifecycle-register-cutover-coremin-0613 --message-format short --color never
  - cargo check --manifest-path zircon_plugins/Cargo.toml --workspace --all-targets --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-lifecycle-register-cutover-plugins-0613 --message-format short --color never (2026-06-13: blocked before compile because zircon_plugins/Cargo.lock would need update under --locked; lock file left untouched)
  - rustfmt --edition 2021 --check zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/build.rs zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog.rs zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/lifecycle.rs zircon_runtime/src/tests/plugin_extensions/runtime_plugin_lifecycle.rs zircon_runtime/src/plugin/runtime_plugin/lifecycle_context.rs
  - git diff --check -- zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/build.rs zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog.rs zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/lifecycle.rs zircon_runtime/src/tests/plugin_extensions/runtime_plugin_lifecycle.rs zircon_runtime/src/plugin/runtime_plugin/lifecycle_context.rs docs/plans/zircon_plugins/01-plugin-architecture-core.md docs/zircon_runtime/plugin/extension_registry.md .codex/sessions/20260612-0851-plugin-architecture-implementation.md
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-capability-view-coremin-0613 --message-format short --color never (2026-06-13: passed with existing warnings)
  - cargo test -p zircon_runtime --lib optional_dependency_probe_sees_all_registered_capabilities --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-capability-view-coremin-0613 --message-format short --color never -- --test-threads=1 --nocapture (2026-06-13: passed 1 focused test)
  - cargo test -p zircon_runtime --lib feature_register_runs_before_finish --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-capability-view-coremin-0613 --message-format short --color never -- --test-threads=1 --nocapture (2026-06-13: passed 1 focused test)
  - cargo test -p zircon_runtime --lib runtime_plugin_lifecycle --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-capability-view-coremin-0613 --message-format short --color never -- --test-threads=1 --nocapture (2026-06-13: passed 3 focused tests)
  - rustfmt --edition 2021 --check zircon_runtime/src/tests/plugin_extensions/static_manifest_contracts/modules.rs zircon_runtime/src/tests/plugin_extensions/static_manifest_contracts/modules/system_anchors.rs
  - python static manifest/source scan for system anchors (2026-06-13: checked `animation.evaluate` and `physics.step` against their declaring runtime crates)
  - D:\cargo-targets\zircon-plugin-architecture-m5-check-coremin-0613b\debug\deps\zircon_runtime-5d2828c2001649f6.exe declared_system_anchors_are_registered --test-threads=1 --nocapture (2026-06-13: passed 1 focused test after an earlier shared lib-test compilation timeout)
  - cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_physics_runtime --tests --offline --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-plugin-checks --message-format short --color never (2026-06-12: passed with existing warnings; zircon_plugins/Cargo.lock restored after the protected run)
  - cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_animation_runtime --tests --offline --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-plugin-checks --message-format short --color never (2026-06-12: passed with existing warnings; zircon_plugins/Cargo.lock restored after the protected run)
  - cargo test -p zircon_runtime --lib runtime_plugin_catalog_merges_module_and_render_feature_contributions --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-0612 --message-format short --color never -- --nocapture (attempted 2026-06-12, timed out after 10 minutes during lib-test compile/link; no Rust diagnostic returned, no pass claimed)
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_animation_runtime --lib --offline --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-plugin-checks --message-format short --color never -- --nocapture (attempted 2026-06-12, timed out after 10 minutes during compile/link; no plugin test pass claimed)
doc_type: module-detail
---

# Runtime Extension Registry

Frameworks 03 feature ownership is compile-time: graphics render features, executors, geometry/shading descriptors, prepare collectors, runtime render providers, and their ownership slots are declared only with `graphics`; UI component storage/application is declared only with `ui`. Package-manifest registration and catalog merge call sites use the same gates. A server build therefore has no placeholder render/UI extension tables and no runtime fallback branch that accepts obsolete contributions. Both the server and default WSL nightly library checks pass after this hard cutover.

`RuntimeExtensionRegistry` is the registration-time collection point for runtime plugin contributions. It now records plugin module owners through `PluginModuleId`, stores keyed contributions in `TypedExtensionPoint`, and can produce an `ExtensionOwnership` slot summary for a module. The runtime-visible accessors still expose slices such as `components()`, `modules()`, `scene_hooks()`, and `render_features()`, but their backing storage carries the owner/key metadata needed by unload, hot reload, and diagnostics work.

The registry now has an explicit registration-epoch to runtime-read transition. `RuntimeExtensionRegistry::finalize()` moves all 20 typed extension points from writable staging storage into their frozen representation and marks the non-typed asset importer registry finalized after catalog validation and contribution merging. Keys, descriptors, owners, and slot maps move into the frozen tables instead of being cloned into a parallel cache; only the sorted key lookup index is derived for runtime reads. Catalog reports finalize before they are returned, and every public apply path finalizes idempotently before reading extension rows. Registering, sorting, mutating, or revoking typed rows, and registering or revoking asset importers, clears the finalized state; a later finalize publishes the new epoch. This is intentionally re-finalizable for owner reload while keeping the read-side storage dense.

`ExtensionSlot` is a stable logical id, not a dense-vector index. Each typed extension point keeps a slot-to-dense-row map: sorting or compacting surviving rows updates that map, owner revocation leaves a retired tombstone for removed slots, and new registrations receive monotonically new slots instead of reusing stale ids. This prevents an old slot from silently resolving to another plugin's extension after unload. `FrozenExtensionTable` preserves the same mapping when a typed point is consumed into a read-only table. The generic `TypedExtensionPoint` remains an internal storage owner; the public plugin façade exposes stable slot/table contracts and `RuntimeExtensionRegistry`, not the mutable storage implementation.

Current Runtime 06/15 implementation status is `runtime_extension_registry_stable_slot_finalize_coremin_check_passed_tests_compile_blocked`. The module-local tests cover `frozen_table_dense_lookup_matches_registration`, `duplicate_extension_key_rejected`, stable survivor slots, retired owner slots, and sort stability. The catalog/application tests cover `runtime_extension_catalog_finalizes_dense_tables_before_apply`, `runtime_extension_apply_finalizes_dense_tables`, asset importer registration/revocation epoch invalidation, and the plan-named `owner_unload_revokes_all_slots`. The world extension tests cover finalized default state and transactional failed install. This status closes neither Runtime 06 nor Runtime 15: the core-min library check passes, while the latest focused lib-test build is blocked before the target tests by an unrelated active plugin-bridge test inference error, and the complete runtime architecture remains in progress.

Plugin bridge interfaces are also registered through this owner-tracked path. `export_interface::<T>(owner, Arc<T>)` stores one typed interface export per stable `PluginInterface::INTERFACE_ID`, rejects duplicate providers for the same id, and records the owning `PluginModuleId` so later unload and hot-reload work can revoke interface rows with the rest of the module's contributions. `frozen_bridge_table()` turns the registered interface exports into dense `InterfaceSlot` entries consumed by `StrongBridge` and `WeakBridge`; the detailed call-path behavior is documented in `docs/zircon_runtime/plugin/bridge.md`.

Typed plugin ECS registration is the primary runtime path. Plugins intern their module name, intern any shared `SystemSetId`, register resources and events by Rust type, and register native ECS systems with stage, set membership, order, and before/after constraints. `register_event(...)` derives the public event catalog namespace from the owning runtime module, so `weather.runtime` contributes to `weather.events`; it validates the derived `PluginEventCatalogManifest` through the same catalog validator used by explicit manifest rows. Event ids must live under that derived catalog namespace, and payload schemas must stay under the package namespace with a positive version segment. `apply_to_world(...)` installs components first, then resources, then events, then boxed native systems, and then runtime scene systems. Native systems derive `SystemParamAccess`; runtime scene systems receive `RuntimeSceneSystemContext { core, level, delta_seconds }` and conservatively mark full-world access because they can re-enter the `LevelSystem`.

`RuntimePluginCatalog::runtime_extensions(...)` now merges owner-tracked resources, events, native systems, and runtime scene systems from each registration report into the final registry, not only descriptors/render contributions. Owner ids are remapped by plugin module name during merge so unload diagnostics and `system_anchors` validation remain scoped to the declaring runtime module. Native system and resource registrations use repeatable shared factories, so cloning a registration report or applying a merged catalog does not consume the original contribution.

Owner revocation is exposed through `RuntimeExtensionRegistry::revoke_owner_registrations(...)`. For typed extension points, it removes every row owned by the supplied `PluginModuleId`, rebuilds the remaining dense key/value/owner arrays, and returns an `ExtensionOwnership` summary containing the old removed slots for diagnostics and rollback reporting. Asset importers are not stored in `TypedExtensionPoint`, so they are revoked through `AssetImporterRegistry::remove_by_plugin_id(...)` using the `"<plugin_id>.runtime"` owner module suffix to recover the exact package `plugin_id`; this preserves dotted plugin ids such as `net.rpc` and prevents hot reload from leaving stale importer matchers behind.

Programmatic extension metadata now follows the same package-owner identity shape. Component types, UI components, and scene hooks accept runtime package ids with dotted, non-empty lowercase segments such as `net.rpc` or `weather.layer`, while still rejecting uppercase, empty segments, trailing underscores, and repeated underscores. Manager registration intentionally stays on the legacy single-token contributor id because its plugin id is only used to intern the `"<plugin_id>.runtime"` owner for manager rows, not to mirror package-manifest package identity. The native host API adapter therefore projects `net.rpc.runtime` back to `net.rpc` and registers its component descriptor through the shared registry path without a native-host compatibility branch.

Manifest-declared system anchors are validated against those owner-tracked ECS registrations. A runtime module row can declare `system_sets` and `system_anchors`, and the registration report accepts an anchor when either `plugin_systems()` or `plugin_runtime_systems()` contains a matching system id owned by the same interned module name. This prevents a package from satisfying `weather.runtime`'s `weather.tick` anchor by registering that system from `weather.tools`, and it avoids manifest-only anchors that would not participate in unload, hot reload, or schedule planning.

Static first-party manifest coverage now includes `declared_system_anchors_are_registered`. The guard walks `zircon_plugins/**/plugin.toml`, finds runtime module rows with `system_anchors`, resolves the declaring `crate_name` through Cargo manifests, and requires the crate source to retain both a runtime-system registration path and the declared anchor id. It keeps the physics and animation `plugin.toml` anchors tied to their linked runtime-system code without making `zircon_runtime` depend on plugin crates.

System ordering is compiled by `SceneScheduleStagePlan`. It groups internal, native, and runtime scene systems per `SystemStage`, expands `SystemRef::Set(...)` constraints to member systems, rejects cross-stage system constraints, and reports ordering cycles during registration/cache rebuild instead of deferring them to frame execution. `order` remains a deterministic tie-break inside the topology, not the only ordering contract.

World-executable registrations are projected by `RuntimeExtensionRegistry::world_runtime_extension_plan()` into a scene-owned `WorldRuntimeExtensionPlan`. `WorldDriver` transactionally merges and stores those type-erased commands, then applies them to default and asset-loaded levels. The plan carries component descriptors, resource initializers, typed events, native scene systems, and runtime scene systems; each new World receives fresh values and system instances. CoreRuntime no longer stores a plugin registry or accepts a concrete Scene World.

Frameworks 02 M3 hard-cuts RuntimePlugin lifecycle ownership to the embedded kernel `ModuleDescriptor`. `RuntimePlugin` keeps only descriptor/manifest/selection projection plus the extension `register(...)` hook; `lifecycle()` returns the descriptor's `dyn ModuleLifecycle`. The retired plugin-only `PluginReadyContext`, `PluginFinishContext`, `PluginRuntimeContext`, `ready`, `finish`, `activate`, and `deactivate` APIs and catalog dispatcher were deleted instead of retained as compatibility wrappers. Runtime startup now runs build/ready/finish/cleanup exactly once through `CoreRuntime` after report-contributed module descriptors are selected and registered.

`CapabilityView::from_registration_reports(...)` remains a read-only capability projection for catalog, bridge, editor, and diagnostics consumers. It collects package-level capabilities, module-level capabilities, feature-level capabilities, feature module capabilities, and package `capability_statuses`. The aggregation intentionally ignores optional feature declarations embedded inside a package manifest until those features are materialized as `RuntimePluginFeatureRegistrationReport` rows. It no longer exists to support a second plugin-only finish phase.

The SDK exposes `RuntimePluginDeclaration::with_module_descriptor(...)`, allowing providers to install a lifecycle-bearing descriptor without bypassing the single descriptor source. First-party plugins may use the same builder directly. Native ABI v3 manifests cannot carry Rust lifecycle objects, so native providers project init level and dependencies into a no-op `ModuleLifecycle`; dynamic load/unload state remains owned by `NativePluginLiveHost` and the bridge lifecycle protocol.

`RuntimePluginRegistrationReport::from_plugin(...)` owns registration of that embedded descriptor before it calls the provider's contribution hook. The hook must not register a module again; it is reserved for systems, interfaces, importers, render contributions, components, options, and event catalogs owned by the module. All first-party runtime plugins now bind their real module through `.with_module_descriptor(...)`. The SDK `RuntimePluginRegistrationBuilder::module(module_name)` only interns the contribution owner and has no descriptor argument, so there is no second descriptor path to drift from manifest projection or kernel lifecycle state.

Runtime 01/06 catalog regressions now apply the same rule to test providers and optional-feature fixtures. A catalog built from a plugin descriptor always contains that descriptor's embedded base module before contribution-hook modules; enabling a feature appends its feature module after the selected base modules, while a blocked optional or required feature leaves the selected base module intact and omits only the feature module. The focused `extensions` gate previously exposed four stale count assertions that treated the embedded base module as an unexpected contribution; those guards now assert the exact base-before-feature names instead of weakening production single-source registration.

Frameworks 02 M3 now treats module ordering as an execution gate instead of a best-effort diagnostic. `order_runtime_plugins(...)` and `order_runtime_plugin_descriptors(...)` return the original typed `CoreError`; constructors keep that source in `RuntimePluginCatalog::module_order_error()`, expose the corresponding diagnostic, and leave registration reports empty. Registration/ready/finish hooks therefore never run for an invalid graph. Runtime activation and reverse deactivation return `RuntimeExtensionRegistryError::InvalidPluginModuleOrder(CoreError)` before invoking plugin or feature hooks. The former alphabetical fallback path has been deleted, so missing dependencies, duplicate module names, init-level violations, and dependency cycles cannot be converted into a runnable but semantically invalid order.

The report-based production path uses the same gate. `RuntimePluginRegistrationReport` carries package runtime-module rows generated by the SDK, first-party providers, and native ABI v3 manifests. Before `runtime_extensions()` merges any report registry, `order_runtime_plugin_registration_reports(...)` rebuilds those `ModuleDescriptor` values and calls the kernel sorter. A valid graph determines merge order; an invalid graph returns an empty finalized registry plus one fatal module-order diagnostic, so app/bootstrap cannot consume extensions from a graph that the kernel would reject. Reports without runtime-module rows remain metadata-only and keep their input position after all ordered runtime providers.

The 2026-07-10 hard-cut tests lock trait-surface deletion, kernel build/ready/finish/cleanup execution through a plugin-provided embedded descriptor, descriptor/report ordering, project-filtered ordering, and invalid-graph rejection before extension merge. The plugin structure audit also enumerates all trait-backed runtime declaration owners and rejects a missing/duplicate `.with_module_descriptor(...)` or any production `register_module(...)` parallel path; the current result is 28 roots and zero violations on Windows and WSL/Python 3.10. The feature-enabled first-party provider snapshot executes 13 linked providers and verifies order, empty diagnostics, and exact manifest/runtime module identity. Scoped `rustfmt --check`, retired-hook/fallback scans, and WSL `zircon_runtime --lib core-min` check pass; package-wide gates remain tracked in the acceptance document.

Editor catalog registration now derives its built-in descriptors from `zircon_plugins/*/plugin.toml`. `zircon_editor/build.rs` scans editor modules declared in package manifests and emits generated rows that `EditorPluginDescriptor::builtin_catalog()` consumes through `editor_plugin_catalog_gen.rs`. This keeps runtime package metadata and editor plugin discovery on the same manifest source, including required capabilities.

Editor capability validation now has a structured diagnostic entry point. `EditorPluginCatalog::validate_capabilities(...)` checks registered editor plugin capabilities against a caller-provided enabled capability set and returns shared `RegistrationDiagnostic` rows from `zircon_runtime_interface`. This gives editor tooling an explicit missing-capability report while preserving the existing required-capability gate used when editor extensions are installed into `EditorEventRuntime`.

Native ABI v3 public DTOs now live in `zircon_runtime_interface::plugin_api`. `ZrHostApiV3` exposes ECS, asset, event, bridge, and diagnostics domain tables, and `ZrPluginStateSnapshotApiV1` plus `ZrByteBufferRef` provide the ABI side of hot reload state exchange. The runtime live host uses owned `PluginStateSnapshot` values when replacing stateful runtime native plugins: it saves old state before unload, restores into the replacement only after schema match, and re-inserts/restores the old handle when replacement restore fails. Registration rollback uses the same owner-tracked registry path as unload: a failed native registration can revoke every contribution owned by the failed `PluginModuleId` without touching surviving owners.

The native host API adapter now consumes the public ECS registration table. `NativeHostApiV3RegistrationScope` interns a plugin module owner, exposes a temporary `ZrRuntimePluginHandle`, and maps `ZrSystemRegistrationV1` / `ZrComponentDescV1` callbacks into the same owner-tracked extension registry used by Rust plugins. ABI `register_system` rows use `NativeDynamicAccess` instead of an empty system parameter, so `SystemParamAccess::add_conservative_world_access()` marks the resulting boxed native system as a conservative world writer and the conflict graph reports a `World` conflict. Component catalog ownership is derived from the exact runtime module suffix, preserving dotted package ids such as `net.rpc`. This keeps native plugin registrations unloadable, conservatively scheduled, and diagnosable through the same ownership slots as in-process runtime plugins.

Current validation status: `cargo check -p zircon_runtime_interface --lib --locked --target-dir D:\cargo-targets\zircon-plugin-architecture-0612 --message-format short` passes. `cargo check -p zircon_editor --lib --locked --message-format short` passes with existing warnings. `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-0612 --message-format short --color never` passes with existing warnings after the runtime scene system and catalog-merge slice. File-scoped `rustfmt --edition 2021 --check` and `git diff --check` pass for the touched registry, catalog, scene module, plugin runtime, and extension test files. For the M3-T2/T3 lifecycle slice, `cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-capability-view-coremin-0613 --message-format short --color never` passes with existing warnings, and `cargo test -p zircon_runtime --lib runtime_plugin_lifecycle --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-capability-view-coremin-0613 --message-format short --color never -- --test-threads=1 --nocapture` passes all 3 lifecycle focused tests. The M4-T3 `declared_system_anchors_are_registered` guard passed in the warmed `zircon_runtime` lib-test binary, keeping the physics and animation manifest anchors tied to their runtime-system source. The earlier graphics import blocker in `mesh/build_mesh_draws/build/build.rs` was removed with a minimal absolute lighting import so the shared runtime lib can compile. `cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_physics_runtime --tests --offline --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-plugin-checks --message-format short --color never` and the same command for `zircon_plugin_animation_runtime` pass with existing warnings after the runtime-system migration. `cargo test -p zircon_editor --lib editor_plugin_catalog_consistency --locked --message-format short -- --nocapture` passes 4 focused catalog tests. Full `zircon_plugins` workspace validation under `--locked` remains blocked before compile because `zircon_plugins/Cargo.lock` would need an update, and that lockfile is intentionally untouched in this slice.

Additional M2 typed-event evidence: `cargo test -p zircon_runtime --lib typed_event_registration --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-next-coremin-0613 --message-format short --color never -- --test-threads=1 --nocapture` passes both derived-catalog tests. The refreshed warmed lib-test binary also passes `plugin_extensions::extension_registry_event_catalogs` (12 tests), `plugin_resource_event_and_system_registrations_apply_to_world`, `runtime_plugin_registration_collects_package_manifest_declared_runtime_contributions`, and `runtime_plugin_catalog_merges_module_and_render_feature_contributions`, proving the lower derived catalog fix reaches the catalog merge path.

Additional M5-T1 ABI evidence: `cargo test -p zircon_runtime_interface --lib abi_v3_layout_is_stable --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-interface-abi-v3-0613 --message-format short --color never -- --test-threads=1 --nocapture` passes 1 focused alias test that groups the ZrHostApiV3 table and snapshot/buffer layout assertions required by the plan. The broader `cargo test -p zircon_runtime_interface --lib plugin_api_contracts --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-interface-contracts-0613b --message-format short --color never -- --test-threads=1 --nocapture` rerun passes all 6 contract tests.

Additional M5-T2 native adapter evidence: `rustfmt --edition 2021 --check zircon_runtime/src/plugin/native_plugin_loader/host_api_adapter.rs` passes after adding `NativeDynamicAccess` and the plan-facing `native_system_enters_schedule_as_conservative_node` alias. Earlier focused attempts hit stale target-dir dep-info output, lib-test compile/link timeout, process `-1` exits without Rust diagnostics, and one unrelated render-owned lib-test compile blocker; the later `cargo test -p zircon_runtime --lib native_system_enters_schedule_as_conservative_node --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-m5-check-coremin-0613b --message-format short --color never -- --test-threads=1 --nocapture` passes 1 focused test.

Additional M5-T3 owner rollback evidence: `failed_registration_revoked_via_ownership` now exposes the plan-named failed-registration rollback filter by routing through `runtime_extension_registry_revokes_owner_tracked_contributions`. That underlying test removes owner-tracked components, options, events, resources, native systems, and asset importers for one plugin while preserving a second plugin owner. `rustfmt --edition 2021 --check zircon_runtime/src/tests/plugin_extensions/extension_registry_metadata.rs` passed, and direct execution of the warmed `zircon_runtime` lib-test binary passed `failed_registration_revoked_via_ownership`.

Additional Frameworks02 dotted-owner evidence: `cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-extension-registry-dotted-coremin-0704 --message-format short --color never` passes with existing warning noise after the registry package-id validator split. Focused tests pass for `runtime_extension_registry_accepts_dotted_component_plugin_ids`, `runtime_extension_registry_accepts_dotted_scene_hook_plugin_ids`, `native_host_api_v3_preserves_dotted_plugin_ids`, and the invalid component/UI/scene-hook plugin id filters. Direct module filters from the same lib-test binary pass `tests::plugin_extensions::extension_registry_components` 16/16, `tests::plugin_extensions::extension_registry_scene_hooks` 9/9, and `native_host` 16/16. A default-feature Cargo test attempt for the dotted component filter timed out during Windows lib-test link and is not accepted as evidence; full `zircon_runtime` lib-test and Frameworks02 integration remain separate gates.
