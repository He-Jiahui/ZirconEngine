---
related_code:
  - zircon_runtime/src/plugin/extension_registry/mod.rs
  - zircon_runtime/src/plugin/extension_registry/runtime_extension_registry.rs
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
  - zircon_runtime/src/plugin/extension_registry/apply_to_world.rs
  - zircon_runtime/src/plugin/extension_registry/apply_to_world/component.rs
  - zircon_runtime/src/plugin/extension_registry_error.rs
  - zircon_runtime/src/core/framework/bridge.rs
  - zircon_runtime/src/plugin/bridge.rs
  - zircon_runtime/src/plugin/bridge/interface_id.rs
  - zircon_runtime/src/plugin/bridge/table.rs
  - zircon_runtime/src/plugin/bridge/strong.rs
  - zircon_runtime/src/plugin/bridge/weak.rs
  - zircon_runtime/src/core/runtime/state/world_runtime_extensions.rs
  - zircon_runtime/src/core/runtime/handle/runtime_extensions.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/contributions/extension.rs
  - zircon_runtime/src/plugin/runtime_plugin/descriptor.rs
  - zircon_runtime/src/plugin/runtime_plugin/descriptor/package_manifest/runtime_module.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/modules/row/systems.rs
  - zircon_runtime/src/plugin/runtime_plugin/registration_report/validation/system_anchors.rs
  - zircon_runtime/src/plugin/runtime_plugin/lifecycle_context.rs
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
implementation_files:
  - zircon_runtime/src/plugin/extension_registry/runtime_extension_registry.rs
  - zircon_runtime/src/plugin/extension_registry/typed_extension_point.rs
  - zircon_runtime/src/plugin/extension_registry/ownership.rs
  - zircon_runtime/src/plugin/extension_registry/owner.rs
  - zircon_runtime/src/asset/importer/registry.rs
  - zircon_runtime/src/plugin/extension_registry/register/system_registration.rs
  - zircon_runtime/src/plugin/extension_registry/register/resource_registration.rs
  - zircon_runtime/src/plugin/extension_registry/register/event_registration.rs
  - zircon_runtime/src/plugin/extension_registry/register/runtime_scene_system_registration.rs
  - zircon_runtime/src/plugin/extension_registry/register/bridge_registration.rs
  - zircon_runtime/src/plugin/extension_registry/apply_to_world.rs
  - zircon_runtime/src/core/framework/bridge.rs
  - zircon_runtime/src/plugin/bridge.rs
  - zircon_runtime/src/plugin/bridge/interface_id.rs
  - zircon_runtime/src/plugin/bridge/table.rs
  - zircon_runtime/src/plugin/bridge/strong.rs
  - zircon_runtime/src/plugin/bridge/weak.rs
  - zircon_runtime/src/core/runtime/state/world_runtime_extensions.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/contributions/extension.rs
  - zircon_runtime/src/plugin/runtime_plugin/descriptor.rs
  - zircon_runtime/src/plugin/runtime_plugin/descriptor/package_manifest/runtime_module.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/modules/row/systems.rs
  - zircon_runtime/src/plugin/runtime_plugin/registration_report/validation/system_anchors.rs
  - zircon_runtime/src/plugin/runtime_plugin/lifecycle_context.rs
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
plan_sources:
  - user: 2026-06-12 implement docs/plans/zircon_plugins plugin architecture code
  - docs/plans/zircon_plugins/index.md
  - docs/plans/zircon_plugins/01-plugin-architecture-core.md
  - docs/plans/zircon_plugins/08-zr-vm.md
  - docs/plans/zircon_plugins/11-plugin-call-bridge.md
tests:
  - zircon_runtime/src/tests/plugin_extensions/extension_registry_bridge.rs
  - zircon_runtime/src/tests/plugin_extensions/extension_registry_systems.rs
  - zircon_runtime/src/tests/plugin_extensions/extension_registry_metadata.rs
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
  - cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_physics_runtime --tests --offline --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-plugin-checks --message-format short --color never (2026-06-12: passed with existing warnings; zircon_plugins/Cargo.lock restored after the protected run)
  - cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_animation_runtime --tests --offline --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-plugin-checks --message-format short --color never (2026-06-12: passed with existing warnings; zircon_plugins/Cargo.lock restored after the protected run)
  - cargo test -p zircon_runtime --lib runtime_plugin_catalog_merges_module_and_render_feature_contributions --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-0612 --message-format short --color never -- --nocapture (attempted 2026-06-12, timed out after 10 minutes during lib-test compile/link; no Rust diagnostic returned, no pass claimed)
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_animation_runtime --lib --offline --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-plugin-checks --message-format short --color never -- --nocapture (attempted 2026-06-12, timed out after 10 minutes during compile/link; no plugin test pass claimed)
doc_type: module-detail
---

# Runtime Extension Registry

`RuntimeExtensionRegistry` is the registration-time collection point for runtime plugin contributions. It now records plugin module owners through `PluginModuleId`, stores keyed contributions in `TypedExtensionPoint`, and can produce an `ExtensionOwnership` slot summary for a module. The runtime-visible accessors still expose slices such as `components()`, `modules()`, `scene_hooks()`, and `render_features()`, but their backing storage carries the owner/key metadata needed by unload, hot reload, and diagnostics work.

Plugin bridge interfaces are also registered through this owner-tracked path. `export_interface::<T>(owner, Arc<T>)` stores one typed interface export per stable `PluginInterface::INTERFACE_ID`, rejects duplicate providers for the same id, and records the owning `PluginModuleId` so later unload and hot-reload work can revoke interface rows with the rest of the module's contributions. `frozen_bridge_table()` turns the registered interface exports into dense `InterfaceSlot` entries consumed by `StrongBridge` and `WeakBridge`; the detailed call-path behavior is documented in `docs/zircon_runtime/plugin/bridge.md`.

Typed plugin ECS registration is the primary runtime path. Plugins intern their module name, intern any shared `SystemSetId`, register resources and events by Rust type, and register native ECS systems with stage, set membership, order, and before/after constraints. `apply_to_world(...)` installs components first, then resources, then events, then boxed native systems, and then runtime scene systems. Native systems derive `SystemParamAccess`; runtime scene systems receive `RuntimeSceneSystemContext { core, level, delta_seconds }` and conservatively mark full-world access because they can re-enter the `LevelSystem`.

`RuntimePluginCatalog::runtime_extensions(...)` now merges owner-tracked resources, events, native systems, and runtime scene systems from each registration report into the final registry, not only descriptors/render contributions. Owner ids are remapped by plugin module name during merge so unload diagnostics and `system_anchors` validation remain scoped to the declaring runtime module. Native system and resource registrations use repeatable shared factories, so cloning a registration report or applying a merged catalog does not consume the original contribution.

Owner revocation is exposed through `RuntimeExtensionRegistry::revoke_owner_registrations(...)`. For typed extension points, it removes every row owned by the supplied `PluginModuleId`, rebuilds the remaining dense key/value/owner arrays, and returns an `ExtensionOwnership` summary containing the old removed slots for diagnostics and rollback reporting. Asset importers are not stored in `TypedExtensionPoint`, so they are revoked through `AssetImporterRegistry::remove_by_plugin_id(...)` using the `"<plugin_id>.runtime"` owner module suffix to recover the exact package `plugin_id`; this preserves dotted plugin ids such as `net.rpc` and prevents hot reload from leaving stale importer matchers behind.

Manifest-declared system anchors are validated against those owner-tracked ECS registrations. A runtime module row can declare `system_sets` and `system_anchors`, and the registration report accepts an anchor when either `plugin_systems()` or `plugin_runtime_systems()` contains a matching system id owned by the same interned module name. This prevents a package from satisfying `weather.runtime`'s `weather.tick` anchor by registering that system from `weather.tools`, and it avoids manifest-only anchors that would not participate in unload, hot reload, or schedule planning.

System ordering is compiled by `SceneScheduleStagePlan`. It groups internal, native, and runtime scene systems per `SystemStage`, expands `SystemRef::Set(...)` constraints to member systems, rejects cross-stage system constraints, and reports ordering cycles during registration/cache rebuild instead of deferring them to frame execution. `order` remains a deterministic tie-break inside the topology, not the only ordering contract.

World-level runtime extensions are installed on `CoreRuntime` and applied to both default levels and levels loaded from scene assets. The runtime-world extension set currently carries the repeatable runtime scene system subset; one-shot native system/resource/event installation still flows through explicit registry application so it does not get accidentally consumed across multiple worlds.

Lifecycle context is split between registration, finish, and runtime activation. `PluginFinishContext` exposes the mutable registry plus a read-only `CapabilityView`; `PluginRuntimeContext` exposes the ready `World` and `CoreHandle`. The old `register_runtime_extensions(...)` method still forwards into `register(...)` for current in-tree implementors, while new plugin code should implement `register(...)`, `finish(...)`, `activate(...)`, and `deactivate(...)`.

Editor catalog registration now derives its built-in descriptors from `zircon_plugins/*/plugin.toml`. `zircon_editor/build.rs` scans editor modules declared in package manifests and emits generated rows that `EditorPluginDescriptor::builtin_catalog()` consumes through `editor_plugin_catalog_gen.rs`. This keeps runtime package metadata and editor plugin discovery on the same manifest source, including required capabilities.

Editor capability validation now has a structured diagnostic entry point. `EditorPluginCatalog::validate_capabilities(...)` checks registered editor plugin capabilities against a caller-provided enabled capability set and returns shared `RegistrationDiagnostic` rows from `zircon_runtime_interface`. This gives editor tooling an explicit missing-capability report while preserving the existing required-capability gate used when editor extensions are installed into `EditorEventRuntime`.

Native ABI v3 public DTOs now live in `zircon_runtime_interface::plugin_api`. `ZrHostApiV3` exposes ECS, asset, event, and diagnostics domain tables, and `ZrPluginStateSnapshotApiV1` plus `ZrByteBufferRef` provide the ABI side of hot reload state exchange. The runtime live host uses owned `PluginStateSnapshot` values when replacing stateful runtime native plugins: it saves old state before unload, restores into the replacement only after schema match, and re-inserts/restores the old handle when replacement restore fails.

The native host API adapter now consumes the public ECS registration table. `NativeHostApiV3RegistrationScope` interns a plugin module owner, exposes a temporary `ZrRuntimePluginHandle`, and maps `ZrSystemRegistrationV1` / `ZrComponentDescV1` callbacks into the same owner-tracked extension registry used by Rust plugins. Component catalog ownership is derived from the exact runtime module suffix, preserving dotted package ids such as `net.rpc`. This keeps native plugin registrations unloadable and diagnosable through the same ownership slots as in-process runtime plugins.

Current validation status: `cargo check -p zircon_runtime_interface --lib --locked --target-dir D:\cargo-targets\zircon-plugin-architecture-0612 --message-format short` passes. `cargo check -p zircon_editor --lib --locked --message-format short` passes with existing warnings. `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-0612 --message-format short --color never` passes with existing warnings after the runtime scene system and catalog-merge slice. File-scoped `rustfmt --edition 2021 --check` and `git diff --check` pass for the touched registry, catalog, scene module, plugin runtime, and extension test files. `cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_physics_runtime --tests --offline --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-plugin-checks --message-format short --color never` and the same command for `zircon_plugin_animation_runtime` pass with existing warnings after the runtime-system migration. `cargo test -p zircon_editor --lib editor_plugin_catalog_consistency --locked --message-format short -- --nocapture` passes 4 focused catalog tests. Earlier focused runtime extension registry and owner-revocation tests passed before the later runtime scene system slice. Focused runtime/plugin test execution for catalog merge and animation lib tests was attempted on 2026-06-12, but those runs timed out after 10 minutes during compile/link and produced no Rust diagnostics; no fresh test pass is claimed for those focused tests yet.
