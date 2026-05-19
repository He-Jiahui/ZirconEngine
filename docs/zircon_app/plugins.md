---
related_code:
  - zircon_app/Cargo.toml
  - zircon_app/src/plugins/mod.rs
  - zircon_app/src/plugins/builder.rs
  - zircon_app/src/plugins/groups.rs
  - zircon_app/src/entry/engine_entry.rs
  - zircon_app/src/entry/entry_runner/bootstrap.rs
  - zircon_app/src/entry/entry_runner/runtime.rs
  - zircon_app/src/entry/entry_runner/runtime_session_args.rs
  - zircon_app/src/entry/builtin_modules.rs
  - zircon_app/src/entry/entry_config.rs
  - zircon_app/src/entry/first_party_runtime_plugins.rs
  - zircon_app/src/entry/tests/builtin_engine_entry.rs
  - zircon_app/src/entry/tests/profile_bootstrap.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog.rs
  - zircon_runtime/src/tests/plugin_extensions/manifest_contributions.rs
  - zircon_plugins/virtual_geometry/plugin.toml
  - zircon_plugins/virtual_geometry/runtime/src/lib.rs
  - zircon_plugins/hybrid_gi/plugin.toml
  - zircon_plugins/hybrid_gi/runtime/src/lib.rs
  - zircon_plugins/solari/plugin.toml
  - zircon_plugins/solari/runtime/src/lib.rs
  - zircon_runtime/src/core/modules/mod.rs
  - zircon_runtime/src/core/modules/log.rs
  - zircon_runtime/src/core/framework/window/descriptor.rs
  - zircon_runtime/src/core/framework/window/constants.rs
  - zircon_runtime/src/core/time.rs
  - zircon_runtime/src/core/diagnostics/store.rs
  - zircon_runtime/src/platform/mod.rs
  - zircon_runtime/src/input/mod.rs
implementation_files:
  - zircon_app/Cargo.toml
  - zircon_app/src/plugins/mod.rs
  - zircon_app/src/plugins/builder.rs
  - zircon_app/src/plugins/groups.rs
  - zircon_app/src/entry/engine_entry.rs
  - zircon_app/src/entry/entry_runner/bootstrap.rs
  - zircon_app/src/entry/entry_runner/runtime.rs
  - zircon_app/src/entry/entry_runner/runtime_session_args.rs
  - zircon_app/src/entry/entry_config.rs
  - zircon_app/src/entry/first_party_runtime_plugins.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog.rs
  - zircon_runtime/src/tests/plugin_extensions/manifest_contributions.rs
  - zircon_plugins/virtual_geometry/plugin.toml
  - zircon_plugins/virtual_geometry/runtime/src/lib.rs
  - zircon_plugins/hybrid_gi/plugin.toml
  - zircon_plugins/hybrid_gi/runtime/src/lib.rs
  - zircon_plugins/solari/plugin.toml
  - zircon_plugins/solari/runtime/src/lib.rs
  - zircon_runtime/src/core/framework/window/descriptor.rs
  - zircon_runtime/src/core/framework/window/constants.rs
  - zircon_runtime/src/core/modules/mod.rs
  - zircon_runtime/src/core/modules/log.rs
plan_sources:
  - user: 2026-05-08 implement ZirconEngine Bevy completion roadmap M1 app composition layer
  - user: 2026-05-16 continue Bevy-style default log diagnostics and dev profile completion
  - user: 2026-05-16 continue Bevy-style platform/window/input default composition completion
  - user: 2026-05-16 continue Bevy-style runtime profile plugin group selection completion
  - user: 2026-05-16 continue Bevy-style profile/module group diagnostics completion
  - user: 2026-05-16 continue Bevy-style app bootstrap platform config completion
  - .codex/plans/ZirconEngine Bevy 完成度两层路线图.md
  - .codex/plans/ZirconEngine Bevy 参照基础设施收束计划.md
tests:
  - zircon_app/src/plugins/tests.rs
  - zircon_app/src/entry/entry_runner/runtime_session_args.rs
  - zircon_app/src/entry/tests/mod.rs
  - zircon_app/src/entry/tests/builtin_engine_entry.rs
  - zircon_app/src/entry/tests/profile_bootstrap.rs
  - cargo test -p zircon_app --locked profile_bootstrap
  - cargo test -p zircon_app --locked --offline --jobs 1 profile_bootstrap -- --nocapture --test-threads=1
  - cargo test -p zircon_app --locked --offline --jobs 1 --features "plugin-ui,first-party-runtime-plugins" profile_bootstrap -- --nocapture --test-threads=1
  - cargo test -p zircon_app --locked --jobs 1 --no-default-features --features "plugin-ui,first-party-runtime-plugins,first-party-navigation-runtime-plugin" runtime_profile_bootstrap_can_link_navigation_when_native_provider_feature_is_enabled --message-format short -- --nocapture --test-threads=1
  - cargo test -p zircon_app --locked --no-default-features --features "plugin-ui,first-party-advanced-render-runtime-plugins" render_profile_runtime_plugins --jobs 1 --message-format short --color never
  - cargo test -p zircon_app --locked --no-default-features --features "plugin-ui,first-party-runtime-plugins,first-party-advanced-render-runtime-plugins" render_profile_runtime_plugins --jobs 1 --message-format short --color never
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

`MinimalPlugins` maps to the available lower shared descriptors needed for a headless core loop: foundation, tasks, time, frame count, and diagnostics core. The task/time/frame-count/diagnostics entries are runtime-owned descriptor modules under `zircon_runtime::core::modules`; they do not install duplicate services because the actual scheduler, runtime clock bundle, frame clock, diagnostic store, and diagnostics collection primitives remain owned by `CoreRuntime` and existing runtime modules.

`DefaultPlugins` maps to the current default runtime stack: foundation, log, platform, input, asset, scene, graphics, script, and feature-gated UI. `HeadlessPlugins` resolves the same lower runtime stack without graphics/UI presentation modules, but keeps platform/input descriptors for capability reporting and synthetic input state plus the default log descriptor so headless tools retain the same diagnostic policy surface.

`DevPlugins` starts from `DefaultPlugins` and inserts `LogDiagnosticsModule` after `DiagnosticsCoreModule`. This mirrors Bevy's pattern where detailed log diagnostics are a development add-on layered over the default app stack instead of being part of the minimal core loop. The runtime diagnostic store and `diagnostic_log::write_diagnostic_store_snapshot(...)` provide the lower-level data/logging surface; app runners still own when to emit those lines.

## Bootstrap Flow

`BuiltinEngineEntry` still asks `builtin_modules_for_config*` to resolve project manifests, linked runtime plugin reports, native plugin reports, target mode, and optional editor module insertion. It then overlays those resolved modules onto the public built-in group for the entry profile. Existing modules replace matching group entries with `set`; extension modules that are not part of the default group are appended.

- `Runtime` and `Editor` profiles start from `DefaultPlugins`.
- `RuntimeProfileId::Minimal` starts from `MinimalPlugins` even though it still maps to the runtime entry run mode.
- `RuntimeProfileId::Dev` starts from `DevPlugins`.
- `Headless` profile starts from `HeadlessPlugins`.

For default, dev, editor, runtime, and headless groups, modules returned by project manifests, linked runtime plugin registrations, native plugin reports, and optional editor insertion are appended when they are not already present in the selected group. For `RuntimeProfileId::Minimal`, unmatched modules are intentionally ignored: the runtime modules returned by the wider resolver may still describe the full client baseline, but the minimal profile keeps only modules already present in `MinimalPlugins`.

`BuiltinEngineEntry::bootstrap` calls `module_descriptors()` on the resolved group, stores app-owned bootstrap config, then registers and activates every descriptor through `CoreRuntime`. It stores the app-owned config again after activation so modules that install default runtime config cannot overwrite the selected entry render/platform profile. Service initialization order, duplicate service detection, dependency resolution, and shutdown rules remain runtime-owned.

`BuiltinEngineEntry::module_selection_report()` exposes the selected entry profile, run mode, optional runtime profile, target mode, plugin group name, and module descriptor counts. The report is read-only diagnostics for tooling and tests; it does not become a second bootstrap path or bypass runtime descriptor registration. `EntryModuleSelectionReport::diagnostic_lines()` and `format_diagnostics()` render the same data into stable text lines so profile/module choices can be captured in logs, CLI output, or test artifacts without reaching into the plugin builder internals. `EntryRunner::module_selection_report(...)` and `EntryRunner::module_selection_diagnostics(...)` expose the same report at the runner boundary before `CoreRuntime` bootstrap, so tools can explain a profile's module composition without registering or activating services.

The provider-aware runner diagnostics mirror the provider-aware bootstrap surface: first-party, linked runtime-plugin, and linked runtime-plugin-plus-feature registration variants all construct the same `BuiltinEngineEntry` variant that their bootstrap sibling would use, but stop at the immutable report. This closes the diagnostic gap where a tool could previously inspect the base profile while the real startup path would also include linked first-party, embedded, native, or feature registration modules.

Bootstrap also stores `PlatformConfig` under `PLATFORM_CONFIG_KEY` before activation. Runtime/editor entries use the current host target and compiled platform feature snapshot, headless entries store a headless target/features selection, and `RuntimeProfileId::Minimal` stores the config as disabled because the minimal group deliberately excludes `PlatformModule`.

Bootstrap stores `RenderProfileBundle` under `RENDER_PROFILE_CONFIG_KEY` with the same before/after activation policy. Runtime/editor entries default to `DefaultRender`, headless entries default to `Headless`, and callers can override the bundle through `EntryConfig::with_render_profile(...)`.

Bootstrap also stores the selected primary-window `WindowDescriptor` under `PRIMARY_WINDOW_DESCRIPTOR_CONFIG_KEY` (`runtime.window.primary_descriptor`) with the same before/after activation policy. Runtime/editor entries default to the standard visible 1280x720 primary window, callers can override it through `EntryConfig::with_window_descriptor(...)`, and headless/minimal/server-target profiles record `WindowDescriptor::without_primary_window()` so diagnostics can state that no app-owned primary window should be created.

`EntryModuleSelectionReport::diagnostic_lines()` now includes the selected window descriptor beside platform diagnostics. The stable window lines report primary-window identity or `none`, title, present/window mode, position, physical/logical size, scale-factor policy, resize constraints, and visible/resizable/decorated/focused booleans. This remains read-only bootstrap diagnostics; real OS window creation is still owned by the runtime-preview app host and its `WindowDescriptor` to winit `WindowAttributes` conversion.

## Bevy Reference Alignment

This composition layer follows Bevy's split between app-level grouping and runtime execution rather than copying Bevy's ECS internals directly. The local Bevy source references used for this slice are:

- `dev/bevy/crates/bevy_app/src/plugin_group.rs`: `PluginGroup::build`, `PluginGroupBuilder::{add,set,disable,enable,add_before,add_after}`, and `finish` define a read/build composition phase before plugins are installed into an `App`.
- `dev/bevy/crates/bevy_app/src/app.rs`: `App::add_plugins` documents `DefaultPlugins` and `MinimalPlugins` as built-in grouping surfaces, while `plugins_state`, `finish`, and `cleanup` keep plugin lifecycle observation separate from app execution.
- `dev/bevy/crates/bevy_app/src/schedule_runner.rs`: the schedule runner waits for plugin state readiness before executing the app loop, which is the precedent for Zircon's diagnose-composition-first and bootstrap-runtime-second split.

Zircon intentionally maps that model onto module descriptors instead of Bevy `Plugin` objects: `PluginGroupBuilder::finish()` resolves ordered `EngineModule` values, provider-aware diagnostics render the exact descriptor set, and only `bootstrap*` methods register/activate those descriptors in `CoreRuntime`.

## Runtime Profile Provider Wiring

`EntryConfig::for_runtime_profile(RuntimeProfileId)` maps the runtime profile contract into the app host shape. `client_2d`, `client_3d`, and `minimal` map to `EntryProfile::Runtime`; `editor` and `dev` map to `EntryProfile::Editor`; `server` maps to `EntryProfile::Headless`. The helper sets `target_mode` from `RuntimeProfileDescriptor` and projects the profile's deterministic `ProjectPluginManifest` into the entry config. `BuiltinEngineEntry` then chooses the concrete group preset, so `minimal` receives the minimal core loop instead of the full runtime default stack.

The runtime-preview binary has a second, narrower profile surface for dynamic cdylib sessions: `--runtime-session-profile <runtime|editor|dev|minimal|headless>` or `--runtime-session-profile=dev`. That argument is not a `PluginGroup` resolver and does not install app-side modules. It is parsed after diagnostic log startup arguments and forwarded as `ZrRuntimeSessionConfigV1.profile` so the loaded `zircon_runtime` library can select session policy internally. `-h` and `--help` list this distinction before dynamic loading, including the available session profiles and process log controls. `RuntimeProfileId::Dev` selecting `DevPlugins` remains the app bootstrap composition path; `--runtime-session-profile dev` is the dynamic runtime preview policy path. Both converge on dev diagnostics, but ownership stays split: app composition chooses module descriptors, while the runtime cdylib owns clocks, diagnostics, and diagnostic-store log cadence.

`first_party_runtime_plugin_registrations_for_config` is the M2 app-owned linked provider. It inspects the enabled plugin selections for the entry target, deduplicates runtime plugin ids, and calls first-party `zircon_plugins/*/runtime::plugin_registration()` functions for the crates compiled into `zircon_app`. The default provider feature is `first-party-runtime-plugins`, which covers the non-native profile-provider set: `sound`, `rendering`, `texture`, `animation`, `net`, and `particles`. `navigation` is behind `first-party-navigation-runtime-plugin` because it builds the Recast/Detour native C++ bridge.

The advanced render provider feature is intentionally separate: `first-party-advanced-render-runtime-plugins` links the `virtual_geometry`, `hybrid_gi`, and `solari` runtime provider crates. `first_party_runtime_plugin_registrations_for_config(...)` first builds the config's project manifest, then adds transient render-provider selections from `EntryConfig::render_profile` when the selected bundle contains `RenderProductFeature::VirtualGeometry`, `RenderProductFeature::HybridGlobalIllumination`, or `RenderProductFeature::Solari`. `DefaultRender` therefore does not link advanced providers, `AdvancedRender` can collect VG/HGI, and `SolariExperimental` can collect the Solari provider contract. Existing explicit manifest selections win; the render-profile helper only appends missing target-scoped selections before the normal provider-aware bootstrap path runs.

`BuiltinEngineEntry::for_config_with_first_party_runtime_plugin_registrations` and `EntryRunner::bootstrap_with_first_party_runtime_plugin_registrations` feed those reports into the existing provider-aware bootstrap path. The matching runner diagnostics method feeds the same reports into `BuiltinEngineEntry` without bootstrapping, so command-line tools, tests, and dev diagnostics can show the real first-party/runtime-provider module set before `CoreRuntime` registration. This keeps the dependency direction explicit: `zircon_app` may link `zircon_plugins`, but `zircon_runtime` receives only `RuntimePluginRegistrationReport` values and never imports plugin implementation crates.

## Validation Coverage

The plugin tests cover ordering, replacement, disabled module omission, duplicate keys, missing anchors, disabled-anchor insertion errors, and built-in group membership. Built-in group membership now explicitly checks that `DefaultPlugins`, `DevPlugins`, and `HeadlessPlugins` include platform/input descriptors while `MinimalPlugins` stays core-only. Entry tests verify that runtime entries expose the resolved group name and module selection report while preserving existing descriptor and bootstrap behavior, including the `RuntimeProfileId::Minimal` special case that selects `MinimalPlugins`, the `RuntimeProfileId::Dev` special case that selects `DevPlugins`, the formatted module-selection diagnostic summary with selected window descriptor lines, base runner-level diagnostics before bootstrap, first-party provider-aware runner diagnostics, and linked runtime-plugin registration diagnostics that include externally contributed module descriptors. Profile bootstrap tests also cover profile-to-entry projection, platform/render/window config persistence for runtime/headless/minimal entries, and, when the first-party provider features are enabled, linked registration closure for required `client_2d` providers plus optional animation/net/particles and navigation provider wiring.

Latest scoped validation on 2026-05-16 used `CARGO_TARGET_DIR=C:\Users\HeJiahui\AppData\Local\Temp\opencode\zircon-profile-provider-target` because other active sessions were using the shared Cargo target directories:

- `cargo test -p zircon_app --locked --offline --jobs 1 --features "plugin-ui,first-party-runtime-plugins" entry_config_can_select_headless_render_profile_bundle -- --nocapture --test-threads=1` passed: 1 test, 0 failures.
- `cargo test -p zircon_app --locked --offline --jobs 1 --features "plugin-ui,first-party-runtime-plugins" profile_bootstrap -- --nocapture --test-threads=1` passed: 15 tests, 0 failures.
- `cargo test -p zircon_app --locked --offline --jobs 1 profile_bootstrap -- --nocapture --test-threads=1` passed: 13 tests, 0 failures.

Native navigation-provider validation initially exposed a Windows-only D3D12 dependency version skew in the root lockfile: `wgpu-hal v29.0.3` uses `windows 0.62.2`, while its `gpu-allocator v0.28.0` dependency had been resolved to `windows 0.61.3`. The accepted lockfile alignment keeps Slint/`zircon_hub`'s `accesskit_windows v0.30.0` on `windows 0.61.3`, but resolves only `gpu-allocator v0.28.0` to the already-present `windows 0.62.2` package so `wgpu-hal` and its allocator share the same D3D12 ABI types.

Windows navigation-provider validation used `CARGO_TARGET_DIR=C:\Users\HeJiahui\AppData\Local\Temp\opencode\zircon-profile-provider-target`, `CARGO_INCREMENTAL=0`, and disabled default app platform/gamepad features because the profile-provider tests do not need them:

- `cargo test -p zircon_app --locked --offline --jobs 1 --no-default-features --features "plugin-ui,first-party-runtime-plugins,first-party-navigation-runtime-plugin" runtime_profile_bootstrap_can_link_navigation_when_native_provider_feature_is_enabled --message-format short -- --nocapture --test-threads=1` passed: 1 test, 0 failures.
- `cargo test -p zircon_app --locked --offline --jobs 1 --no-default-features --features "plugin-ui,first-party-runtime-plugins,first-party-navigation-runtime-plugin" profile_bootstrap --message-format short -- --nocapture --test-threads=1` passed: 18 tests, 0 failures.

WSL/Linux was also used as corroborating evidence with `CARGO_TARGET_DIR=/tmp/opencode/zircon-profile-provider-target`:

- `CARGO_INCREMENTAL=0 cargo test -p zircon_app --locked --jobs 1 --no-default-features --features "plugin-ui,first-party-runtime-plugins,first-party-navigation-runtime-plugin" runtime_profile_bootstrap_can_link_navigation_when_native_provider_feature_is_enabled --message-format short -- --nocapture --test-threads=1` passed: 1 test, 0 failures.

Fresh M9A app-provider validation on 2026-05-19 used `CARGO_TARGET_DIR=E:\Git\ZirconEngine\target\codex-render-m9a-advanced`:

- `cargo check -p zircon_app --lib --locked --no-default-features --features "plugin-ui,first-party-advanced-render-runtime-plugins" --jobs 1 --color never` passed after the lockfile included the new optional provider crates.
- `cargo test -p zircon_app --locked --no-default-features --features "plugin-ui,first-party-advanced-render-runtime-plugins" render_profile_runtime_plugins --jobs 1 --message-format short --color never` passed: 2 tests, 0 failures.
- `cargo test -p zircon_app --locked --no-default-features --features "plugin-ui,first-party-runtime-plugins,first-party-advanced-render-runtime-plugins" render_profile_runtime_plugins --jobs 1 --message-format short --color never` passed: 3 tests, 0 failures.
- `cargo check --manifest-path zircon_plugins\Cargo.toml --workspace --locked --all-targets --jobs 1` passed for the linked first-party plugin workspace after shader importer schema-sync fixes.
