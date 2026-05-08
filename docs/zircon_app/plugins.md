---
related_code:
  - zircon_app/src/plugins/mod.rs
  - zircon_app/src/plugins/builder.rs
  - zircon_app/src/plugins/groups.rs
  - zircon_app/src/entry/engine_entry.rs
  - zircon_app/src/entry/builtin_modules.rs
  - zircon_app/src/entry/entry_config.rs
  - zircon_app/src/entry/first_party_runtime_plugins.rs
  - zircon_runtime/src/core/modules/mod.rs
implementation_files:
  - zircon_app/src/plugins/mod.rs
  - zircon_app/src/plugins/builder.rs
  - zircon_app/src/plugins/groups.rs
  - zircon_app/src/entry/engine_entry.rs
  - zircon_app/src/entry/entry_config.rs
  - zircon_app/src/entry/first_party_runtime_plugins.rs
  - zircon_runtime/src/core/modules/mod.rs
plan_sources:
  - user: 2026-05-08 implement ZirconEngine Bevy completion roadmap M1 app composition layer
  - .codex/plans/ZirconEngine Bevy 完成度两层路线图.md
  - .codex/plans/ZirconEngine Bevy 参照基础设施收束计划.md
tests:
  - zircon_app/src/plugins/tests.rs
  - zircon_app/src/entry/tests/builtin_engine_entry.rs
  - zircon_app/src/entry/tests/profile_bootstrap.rs
  - cargo test -p zircon_app --locked profile_bootstrap
  - .github/workflows/ci.yml
doc_type: module-detail
---

# Zircon App Plugin Composition

`zircon_app::plugins` is the app-owned composition layer for Bevy-style plugin groups. It does not replace `zircon_runtime::core::CoreRuntime`; it only resolves ordered `EngineModule` values into module descriptors that the runtime still registers and activates through its existing lifecycle path.

## Ownership

- `zircon_app` owns entry profile selection, project plugin manifest input, and app-level plugin group selection.
- `zircon_runtime` owns concrete runtime modules, descriptors, service factories, dependency rules, and activation.
- `zircon_editor` contributes `EditorModule` only when the editor-host feature/profile is active.

This keeps the roadmap boundary intact: app hosts and composes, runtime schedules and owns services, editor remains an optional authoring host.

## Builder Semantics

`PluginGroupBuilder` stores modules by their `EngineModule::module_name()` key and preserves a separate explicit order list. The builder supports:

- `add` for inserting a new module at the end of the group.
- `set` for replacing an existing module without changing its order.
- `disable` and `enable` for toggling an existing module.
- `add_before` and `add_after` for anchor-based insertion.
- `finish` for producing a `ResolvedPluginGroup` that contains only enabled modules.

Errors are explicit instead of panic-driven. Duplicate module keys, missing keys, missing ordering anchors, and ordering relative to disabled anchors produce `PluginGroupError` values. Disabled anchors are rejected so a final group cannot hide ordering dependencies behind modules that will not be registered.

## Built-In Groups

`MinimalPlugins` maps to the available lower shared descriptors needed for a headless core loop: foundation, tasks, time, frame count, and diagnostics core. The task/time/frame-count/diagnostics entries are runtime-owned descriptor modules under `zircon_runtime::core::modules`; they do not install duplicate services because the actual scheduler, frame clock, and diagnostics collection primitives remain owned by `CoreRuntime` and existing runtime modules.

`DefaultPlugins` maps to the current default runtime stack: foundation, platform, input, asset, scene, graphics, script, and feature-gated UI. `HeadlessPlugins` resolves the same lower runtime stack without graphics/UI presentation modules. `DevPlugins` currently aliases the default stack and is reserved for the later diagnostics/log/dev-profile milestone.

## Bootstrap Flow

`BuiltinEngineEntry` still asks `builtin_modules_for_config*` to resolve project manifests, linked runtime plugin reports, native plugin reports, target mode, and optional editor module insertion. It then overlays those resolved modules onto the public built-in group for the entry profile. Existing modules replace matching group entries with `set`; extension modules that are not part of the default group are appended.

- `Runtime` and `Editor` profiles start from `DefaultPlugins`.
- `Headless` profile starts from `HeadlessPlugins`.

`BuiltinEngineEntry::bootstrap` calls `module_descriptors()` on the resolved group, then registers and activates every descriptor through `CoreRuntime`. Service initialization order, duplicate service detection, dependency resolution, and shutdown rules remain runtime-owned.

## Runtime Profile Provider Wiring

`EntryConfig::for_runtime_profile(RuntimeProfileId)` maps the runtime profile contract into the app host shape. `client_2d`, `client_3d`, and `minimal` map to `EntryProfile::Runtime`; `editor` and `dev` map to `EntryProfile::Editor`; `server` maps to `EntryProfile::Headless`. The helper sets `target_mode` from `RuntimeProfileDescriptor` and projects the profile's deterministic `ProjectPluginManifest` into the entry config.

`first_party_runtime_plugin_registrations_for_config` is the M2 app-owned linked provider. It inspects the enabled plugin selections for the entry target, deduplicates runtime plugin ids, and calls first-party `zircon_plugins/*/runtime::plugin_registration()` functions for the crates compiled into `zircon_app`. The default provider feature is `first-party-runtime-plugins`, which covers the non-native profile-provider set: `sound`, `rendering`, `texture`, `animation`, `net`, and `particles`. `navigation` is behind `first-party-navigation-runtime-plugin` because it builds the Recast/Detour native C++ bridge.

`BuiltinEngineEntry::for_config_with_first_party_runtime_plugin_registrations` and `EntryRunner::bootstrap_with_first_party_runtime_plugin_registrations` feed those reports into the existing provider-aware bootstrap path. This keeps the dependency direction explicit: `zircon_app` may link `zircon_plugins`, but `zircon_runtime` receives only `RuntimePluginRegistrationReport` values and never imports plugin implementation crates.

## Validation Coverage

The plugin tests cover ordering, replacement, disabled module omission, duplicate keys, missing anchors, disabled-anchor insertion errors, and built-in group membership. Entry tests verify that runtime entries expose the resolved group name while preserving existing descriptor and bootstrap behavior. Profile bootstrap tests also cover profile-to-entry projection and, when the first-party provider features are enabled, linked registration closure for required `client_2d` providers plus optional animation/net/particles and navigation provider wiring.
