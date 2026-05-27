---
related_code:
  - zircon_runtime/src/plugin/runtime_profile.rs
  - zircon_runtime/src/plugin/core_profiles.rs
  - zircon_runtime/src/prelude.rs
  - zircon_runtime/src/core/runtime/runtime.rs
  - zircon_runtime/src/core/state/mod.rs
  - zircon_runtime/src/core/tasks/mod.rs
  - zircon_runtime/src/core/tasks/pools.rs
  - zircon_runtime/src/core/job_scheduler.rs
  - zircon_runtime/src/core/framework/time/mod.rs
  - zircon_runtime/src/core/modules/diagnostics.rs
  - zircon_runtime/src/core/modules/frame_count.rs
  - zircon_runtime/src/core/modules/log.rs
  - zircon_runtime/src/core/modules/tasks.rs
  - zircon_runtime/src/core/modules/time.rs
  - zircon_runtime/src/builtin/runtime_modules.rs
  - zircon_runtime/src/lib.rs
  - zircon_runtime/src/plugin/project_plugin_manifest/project_plugin_manifest.rs
  - zircon_runtime/src/plugin/project_plugin_manifest/project_plugin_selection.rs
  - zircon_runtime/src/plugin/export_build_plan/export_build_plan.rs
  - zircon_runtime/src/plugin/export_build_plan/export_materialize_report.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog.rs
  - zircon_runtime/src/tests/plugin_extensions/profile_maturity.rs
  - zircon_runtime/src/tests/plugin_extensions/manifest_contributions.rs
  - zircon_runtime/src/tests/plugin_extensions/export_build_plan.rs
  - zircon_app/src/prelude.rs
  - zircon_app/src/plugins/builder.rs
  - zircon_app/src/plugins/groups.rs
  - zircon_app/src/plugins/tests.rs
  - zircon_app/src/tests/prelude.rs
  - zircon_app/src/entry/builtin_modules.rs
  - zircon_app/src/entry/entry_config.rs
  - zircon_app/src/entry/first_party_runtime_plugins.rs
  - zircon_app/src/entry/engine_entry.rs
  - zircon_app/src/entry/entry_runner/bootstrap.rs
  - zircon_plugins/sound/plugin.toml
  - zircon_plugins/sound/runtime/Cargo.toml
  - zircon_plugins/sound/runtime/src/lib.rs
  - zircon_plugins/sound/runtime/src/tests/manifest.rs
  - zircon_plugins/virtual_geometry/plugin.toml
  - zircon_plugins/virtual_geometry/runtime/src/lib.rs
  - zircon_plugins/hybrid_gi/plugin.toml
  - zircon_plugins/hybrid_gi/runtime/src/lib.rs
  - zircon_plugins/solari/plugin.toml
  - zircon_plugins/solari/runtime/src/lib.rs
  - .github/workflows/ci.yml
  - .codex/skills/zircon-dev/scripts/validate-matrix.ps1
  - .codex/skills/zircon-dev/validation/SKILL.md
  - .codex/skills/zircon-dev/reporting.md
  - docs/runtime-plugins/bevy-parity-matrix.md
implementation_files:
  - zircon_runtime/src/plugin/runtime_profile.rs
  - zircon_runtime/src/prelude.rs
  - zircon_runtime/src/core/runtime/runtime.rs
  - zircon_runtime/src/core/state/mod.rs
  - zircon_runtime/src/core/tasks/mod.rs
  - zircon_runtime/src/core/tasks/pools.rs
  - zircon_runtime/src/core/job_scheduler.rs
  - zircon_runtime/src/core/framework/time/mod.rs
  - zircon_runtime/src/core/modules/diagnostics.rs
  - zircon_runtime/src/core/modules/frame_count.rs
  - zircon_runtime/src/core/modules/log.rs
  - zircon_runtime/src/core/modules/tasks.rs
  - zircon_runtime/src/core/modules/time.rs
  - zircon_runtime/src/builtin/runtime_modules.rs
  - zircon_runtime/src/lib.rs
  - zircon_runtime/src/plugin/export_build_plan/export_build_plan.rs
  - zircon_runtime/src/plugin/export_build_plan/export_materialize_report.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog.rs
  - zircon_runtime/src/tests/plugin_extensions/manifest_contributions.rs
  - zircon_app/src/prelude.rs
  - zircon_app/src/plugins/builder.rs
  - zircon_app/src/plugins/groups.rs
  - zircon_app/src/plugins/tests.rs
  - zircon_app/src/tests/prelude.rs
  - zircon_app/src/entry/builtin_modules.rs
  - zircon_app/src/entry/entry_config.rs
  - zircon_app/src/entry/first_party_runtime_plugins.rs
  - zircon_app/src/entry/engine_entry.rs
  - zircon_app/src/entry/entry_runner/bootstrap.rs
  - zircon_plugins/sound/plugin.toml
  - zircon_plugins/sound/runtime/Cargo.toml
  - zircon_plugins/sound/runtime/src/lib.rs
  - zircon_plugins/sound/runtime/src/tests/manifest.rs
  - zircon_plugins/virtual_geometry/plugin.toml
  - zircon_plugins/virtual_geometry/runtime/src/lib.rs
  - zircon_plugins/hybrid_gi/plugin.toml
  - zircon_plugins/hybrid_gi/runtime/src/lib.rs
  - zircon_plugins/solari/plugin.toml
  - zircon_plugins/solari/runtime/src/lib.rs
  - .github/workflows/ci.yml
  - .codex/skills/zircon-dev/scripts/validate-matrix.ps1
  - .codex/skills/zircon-dev/validation/SKILL.md
  - .codex/skills/zircon-dev/reporting.md
  - docs/runtime-plugins/bevy-parity-matrix.md
plan_sources:
  - user: 2026-05-08 实现 ZirconEngine Bevy 级插件完成度里程碑计划
  - user: 2026-05-16 continue Bevy-style MinimalPlugins/runtime profile convergence
  - user: 2026-05-24 继续完善 ZirconEngine 到 Bevy 完成度的详细计划并引用 Bevy 源码
  - .codex/plans/ZirconEngine Bevy 级插件完成度里程碑计划.md
  - .codex/plans/ZirconEngine Bevy 完成度两层路线图.md
tests:
  - zircon_app/src/plugins/tests.rs
  - zircon_app/src/tests/prelude.rs
  - zircon_runtime/src/tests/prelude.rs
  - zircon_runtime/src/tests/state.rs
  - zircon_runtime/src/tests/time.rs
  - zircon_runtime/src/tests/tasks.rs
  - zircon_runtime/src/diagnostic_log/diagnostics.rs
  - zircon_runtime/src/tests/plugin_extensions/profile_maturity.rs
  - zircon_runtime/src/tests/plugin_extensions/export_build_plan.rs
  - zircon_plugins/sound/runtime/src/tests/manifest.rs
  - zircon_app/src/entry/tests/profile_bootstrap.rs
  - cargo test -p zircon_runtime --lib plugin_extensions::profile_maturity --locked -- --nocapture
  - cargo test -p zircon_app --locked profile_bootstrap
  - cargo test -p zircon_app --locked --offline --jobs 1 --features "plugin-ui,first-party-runtime-plugins" first_party_sound_provider_preserves_manifest_maturity_and_capability_status -- --nocapture --test-threads=1
  - cargo test -p zircon_app --locked --offline --jobs 1 --features "plugin-ui,first-party-runtime-plugins" profile_bootstrap -- --nocapture --test-threads=1
  - cargo test -p zircon_app --locked --jobs 1 --no-default-features --features "plugin-ui,first-party-runtime-plugins,first-party-navigation-runtime-plugin" runtime_profile_bootstrap_can_link_navigation_when_native_provider_feature_is_enabled --message-format short -- --nocapture --test-threads=1
  - cargo check -p zircon_runtime --lib --locked
  - cargo test -p zircon_runtime --lib prelude --locked
  - cargo test -p zircon_runtime --lib state --locked
  - cargo test -p zircon_runtime --lib time --locked
  - cargo test -p zircon_runtime --lib tasks --locked
  - cargo test -p zircon_runtime --lib plugin_extensions::export_build_plan --locked
  - cargo test -p zircon_app --locked plugins
  - cargo test -p zircon_app --locked prelude
  - cargo test -p zircon_app --locked --no-default-features --features "plugin-ui,first-party-runtime-plugins,first-party-advanced-render-runtime-plugins" render_profile_runtime_plugins --jobs 1 --message-format short --color never
  - cargo test -p zircon_runtime --lib advanced_render_plugin_manifests_declare_profile_capabilities --locked --jobs 1 --message-format short --color never
  - cargo build -p zircon_app --no-default-features --features target-editor-host --bin zircon_editor --locked --jobs 1 --target-dir D:\cargo-targets\global-ui-m3-validation
  - docs-only M1 review: app plugin groups, stable prelude, state/time/tasks/log/diagnostics gates are sourced from Bevy and Zircon files
  - docs-only M10 review: profile/catalog docs-sync gates are sourced from Bevy CI feature-doc checks and Zircon CI/validator/reporting files
doc_type: module-detail
---

# Profile Selection

`RuntimeProfileDescriptor` is the M1 profile contract for Bevy-grade plugin selection. It sits in `zircon_runtime::plugin` so runtime, app, export, and editor code can share one descriptor shape without making `zircon_runtime` depend on `zircon_plugins`.

## Built-In Profiles

Profiles are exposed by `RuntimeProfileId`:

- `Minimal`: Bevy-style minimal core loop: lifecycle, task, time, frame-count, and diagnostics capabilities only; no platform/input/asset/scene/render/script module and no first-party external plugin requirement.
- `Client2d`: client runtime target with sound, rendering, and texture baseline plus optional tilemap, particles, and animation.
- `Client3d`: client runtime target with sound, rendering, and texture baseline plus optional animation, navigation, particles, virtual geometry, hybrid GI, and Solari.
- `Editor`: editor host target with UI, sound, rendering, texture, and optional authoring-facing runtime plugins.
- `Dev`: editor-host diagnostics/dev profile with networking enabled as optional and advanced rendering/VFX plugins available.
- `Server`: server runtime target; it does not enable UI, rendering, or audio listener defaults.

Each descriptor carries target mode, default plugins, optional plugins, required capabilities, minimum maturity, and whether required externalized plugins are allowed. Stable/default-style profiles use `minimum_maturity = Beta` and do not allow required `Externalized` or `Stub` plugins.

## Bevy Reference Map

This document uses Bevy as the dominant reference for default/minimal composition, while keeping Zircon's runtime/app boundary intact:

| Reference | Behavior used for Zircon planning |
| --- | --- |
| `dev/bevy/crates/bevy_internal/src/default_plugins.rs` | `DefaultPlugins` is the broad app baseline and `MinimalPlugins` is the smallest runnable core group. |
| `dev/bevy/crates/bevy_internal/src/prelude.rs` | Bevy's public prelude re-exports core app/runtime-facing types and feature-gated subsystem preludes. |
| `dev/bevy/crates/bevy_app/src/plugin_group.rs` | `PluginGroupBuilder` owns deterministic plugin ordering, replacement, disable/enable, and anchor-based insertion. |
| `dev/bevy/crates/bevy_app/src/task_pool_plugin.rs` | `TaskPoolPlugin` creates IO, async-compute, and compute task pools as core app infrastructure. |
| `dev/bevy/crates/bevy_time/src/lib.rs` | `TimePlugin` initializes real, virtual, and fixed time resources and fixed-step behavior. |
| `dev/bevy/crates/bevy_state/src/app.rs` | `StatesPlugin` installs transition scheduling and exposes state/next-state semantics. |
| `dev/bevy/crates/bevy_log/src/lib.rs` | `LogPlugin` is part of the default group and can be replaced or disabled from `DefaultPlugins`. |
| `dev/bevy/crates/bevy_diagnostic/src/lib.rs` and `frame_time_diagnostics_plugin.rs` | Diagnostics storage, frame-time metrics, and log diagnostics are separate plugins. |
| `dev/bevy/docs/cargo_features.md` | Bevy separates user-facing feature/profile collections from individual subsystem crates. |
| `dev/bevy/.github/workflows/ci.yml` and `dev/bevy/tools/ci/src/ci.rs` | Bevy keeps feature docs, examples, compile checks, doc checks, and lints in separate CI gates. |

## Manifest Projection

`RuntimeProfileDescriptor::project_manifest()` converts a profile into a deterministic `ProjectPluginManifest` by preserving default plugin order and assigning each selection to the profile target mode. The legacy `default_manifest_for_target()` behavior is preserved for runtime bootstrap compatibility, while `manifest_for_runtime_profile()` exposes the new profile-driven manifest for export/editor surfaces that opt into M1 profile selection.

## App Plugin Groups

`RuntimeProfileDescriptor` owns the runtime-plugin/profile contract. `zircon_app::plugins` owns the Bevy-style composition layer that selects concrete runtime modules for an app entry. This split is intentional: `zircon_runtime` can describe profiles and capabilities without depending on `zircon_plugins`, while `zircon_app` can provide concrete module groups and linked first-party provider registrations.

| Zircon group | Bevy role | Current module families | Profile use |
| --- | --- | --- | --- |
| `MinimalPlugins` | Mirrors Bevy's small runnable core group. | Foundation, tasks, time, frame count, diagnostics core. | Used by `RuntimeProfileId::Minimal`; must stay free of platform/input/asset/render/UI/script modules. |
| `DefaultPlugins` | Mirrors Bevy's broad default app baseline. | Foundation, log, tasks, time, frame count, diagnostics, platform, input, asset, scene, graphics, script, and UI when `plugin-ui` is enabled. | Used by runtime/client/editor profiles that need normal app infrastructure. |
| `DevPlugins` | Bevy-style default plus verbose development diagnostics. | `DefaultPlugins` plus `LogDiagnosticsModule` inserted after `DiagnosticsCoreModule`. | Used by `Dev`; this is the only built-in group that should enable diagnostic-store log cadence by default. |
| `HeadlessPlugins` | Default-style runtime without visual presentation. | Default core/platform/input/asset/scene/script path without graphics or UI. | Used by server/headless entry modes and export validation. |

`PluginGroupBuilder` is Zircon's equivalent of Bevy's group editing surface. It supports `set`, `disable`, `enable`, `add_before`, `add_after`, duplicate detection, missing-key diagnostics, and built-in membership tests. Zircon deliberately keys ordering by `EngineModule::module_name()` strings instead of Bevy `TypeId` values because modules are trait-object descriptors, not concrete `Plugin` types exposed to users.

One G1 decision remains open: Bevy preserves disabled plugin entries as ordering anchors, while Zircon's current builder treats insertion relative to a disabled anchor as a `DisabledAnchor` error. That stricter behavior is acceptable while profile composition is still stabilizing, but it must either become the documented Zircon contract or be aligned to Bevy before broadening default profile membership.

## Stable Prelude Policy

Zircon has two stable-facing preludes:

- `zircon_runtime::prelude` exposes core runtime contracts: `CoreRuntime`, module descriptors, service/lifecycle contracts, state, real/virtual/fixed time, task pools, diagnostic store/log settings, runtime profiles, plugin ids, and runtime module helpers.
- `zircon_app::prelude` layers app entry, profile selection, first-party provider helpers, and `MinimalPlugins` / `DefaultPlugins` / `DevPlugins` / `HeadlessPlugins` on top of the runtime prelude.

Admission rules for M1:

- Core app/runtime foundations may enter stable preludes once they have source-owned tests or smoke tests: state, time, tasks, diagnostics, log settings, profile descriptors, plugin group types, and entry selection.
- Experimental subsystem internals must stay behind their subsystem modules until their milestone promotes them. This includes active render, UI, asset, sound, navigation, and importer implementation details.
- `zircon_runtime::prelude` must not re-export provider crate types from `zircon_plugins`; linked/native provider ownership stays in `zircon_app` or generated export hosts.
- `zircon_app/src/tests/prelude.rs` is the public-surface smoke test. Any later prelude expansion should add a smoke assertion there or in the owning crate's equivalent test.

## Core Foundation Gates

These foundations are not first-party runtime plugins. They are the Bevy-grade app/runtime spine that every later profile depends on.

| Foundation | Bevy precedent | Zircon owner | Completion gate |
| --- | --- | --- | --- |
| State | `StatesPlugin` installs transition scheduling and exposes state/next-state transition behavior. | `zircon_runtime/src/core/state` and `CoreRuntime::{init_state,set_next_state,apply_state_transition}`. | State transitions must be driveable from the runtime schedule/profile path, with hook/event tests and no feature-specific branches. |
| Time and frame count | `TimePlugin` owns real/virtual/fixed time; frame count is part of default/minimal infrastructure. | `core::framework::time`, `TimeModule`, `FrameCountModule`, `CoreRuntime::{tick_time,advance_time_by}`. | Tests must cover pause, speed, max-delta, fixed-step limits, frame count, and diagnostic emission. |
| Tasks | `TaskPoolPlugin` creates IO, async-compute, and compute pools. | `core::tasks`, `TaskPools`, `TaskPoolOptions`, `TaskPoolReport`, and compute-backed `JobScheduler`. | Profile-configurable defaults and report shape must be stable before asset/render/audio rely on the task pools as default infrastructure. |
| Log and diagnostics | `LogPlugin`, `DiagnosticsPlugin`, `FrameTimeDiagnosticsPlugin`, and `LogDiagnosticsPlugin` split logging, storage, metrics, and output cadence. | `diagnostic_log`, `DiagnosticStore`, `LogModule`, `DiagnosticsCoreModule`, `FrameCountModule`, and `LogDiagnosticsModule`. | Default profiles stay quiet; `DevPlugins` enables diagnostic-store log cadence explicitly and remains test-covered. |

## M1 Validation Matrix

M1 is complete only when the profile contract, app group composition, public prelude, and core runtime foundations are all covered. Passing only profile metadata tests is not enough because Bevy's `DefaultPlugins` contract depends on the app-facing composition and foundation services behaving together.

| Validation area | Bevy-derived behavior | Zircon evidence to keep green | Promotion rule |
| --- | --- | --- | --- |
| Profile descriptor and maturity | Bevy separates feature/profile collections from individual plugin crates and treats missing replacement targets as explicit errors. | `zircon_runtime/src/tests/plugin_extensions/profile_maturity.rs` plus `zircon_app/src/entry/tests/profile_bootstrap.rs`. | Every profile id resolves target mode, required capabilities, optional plugins, and minimum maturity; stable/default profiles have no required `Externalized` or `Stub` plugin. |
| Plugin group editing | Bevy tests ordered insertion, missing anchors, re-add, disable/enable, and disabled-anchor ordering. | `zircon_app/src/plugins/tests.rs`. | Order, replacement, duplicate/missing key, missing anchor, disabled-anchor behavior, and minimal/default/dev/headless membership must be covered before adding more default modules. |
| Public prelude | Bevy prelude exports app/runtime basics and feature-gated subsystem preludes without forcing internal paths. | `zircon_app/src/tests/prelude.rs` and `zircon_runtime/src/tests/prelude.rs`. | `zircon_app::prelude::*` must expose entry/profile/plugin-group types and runtime foundations; provider crate types from `zircon_plugins` must not leak into `zircon_runtime::prelude`. |
| State foundation | Bevy `StatesPlugin` installs `StateTransition`, supports init/insert overwrite, and emits transition events. | `zircon_runtime/src/tests/state.rs` plus the app prelude state smoke test. | Init, insert, next-state apply/reset, multi-state independence, transition events, and hooks must remain deterministic. |
| Time and frame foundation | Bevy `TimePlugin` initializes real/virtual/fixed time and tests fixed-update accumulation. | `zircon_runtime/src/tests/time.rs` and frame-count diagnostics from runtime/prelude tests. | Real/virtual/fixed clocks must cover pause, relative speed, max-delta clamp, fixed-step limits, and diagnostic emission. |
| Task pools and scheduler | Bevy `TaskPoolPlugin` creates separate IO, async-compute, and compute pools. | `zircon_runtime/src/tests/tasks.rs`. | Task pool counts, reports, pool lookup, and compute-backed scheduler behavior must be stable enough for asset/render/audio milestones. |
| Log and diagnostics | Bevy separates log setup, diagnostic storage, frame metrics, and log diagnostics output cadence. | `zircon_runtime/src/diagnostic_log/diagnostics.rs`, `zircon_runtime/src/tests/prelude.rs`, and app plugin group tests. | Default profiles must keep diagnostics available but quiet; `DevPlugins` must be the explicit path that adds log diagnostics cadence. |

M1's unresolved choice is limited to disabled-anchor semantics. Either keep Zircon's stricter `DisabledAnchor` error as a documented intentional divergence, or align to Bevy's disabled-anchor ordering before G1 is closed. The decision must be reflected in `zircon_app/src/plugins/tests.rs`; undocumented behavior is not acceptable for profile broadening.

## M1 Candidate Commands

These commands form the local M1 gate when shared Cargo target directories are not blocked by other active sessions:

- `cargo test -p zircon_runtime --lib plugin_extensions::profile_maturity --locked -- --nocapture`
- `cargo test -p zircon_app --locked profile_bootstrap`
- `cargo test -p zircon_app --locked plugins`
- `cargo test -p zircon_app --locked prelude`
- `cargo test -p zircon_runtime --lib prelude --locked`
- `cargo test -p zircon_runtime --lib state --locked`
- `cargo test -p zircon_runtime --lib time --locked`
- `cargo test -p zircon_runtime --lib tasks --locked`
- `cargo check -p zircon_runtime --lib --locked`

If any upper app/profile test fails, debug the lower foundation first: state/time/tasks/diagnostics, then plugin-group composition, then profile bootstrap.

## Availability Report

`RuntimeProfileDescriptor::availability_report()` produces structured buckets:

- `available`: descriptor satisfies target and maturity gates.
- `linked`: runtime registration was supplied by the app/export layer.
- `native_dynamic`: reserved for M2 native provider-chain reporting.
- `externalized_missing`: descriptor is externalized and no registration is linked.
- `stub`: descriptor is a stub.
- `blocked_by_target`: target mode is unsupported.
- `blocked_by_maturity`: maturity is below the profile minimum.
- `missing_required`: required default plugin failed one of the gates.

This is the data model intended to replace broad warning strings in later resolver work. Default and optional profile plugins are evaluated through the same target, linked-registration, externalized, stub, and maturity gates; optional failures populate their warning buckets without entering `missing_required`. M2 will connect the report to the provider chain and native dynamic package resolution.

## Provider-Aware Availability

M2 begins by making provider availability explicit without moving `zircon_runtime` across the `zircon_plugins` crate boundary. `availability_report()` remains the M1 metadata-only gate for legacy callers. `availability_report_with_providers()` adds two provider sets: linked registrations and native dynamic registrations. Required first-party plugins that have catalog descriptors but no provider now land in `externalized_missing` and `missing_required` instead of appearing usable only because their descriptor maturity is `Beta` or `Stable`.

`availability_report_for_registration_reports()` derives those provider sets from `RuntimePluginRegistrationReport` values. `LibraryEmbed` and source-style registration reports count as `linked`; `NativeDynamic` registration reports count as `native_dynamic`. Disabled or wrong-target registration reports are ignored. This keeps the runtime-side contract provider-chain shaped while allowing app/export/editor layers to supply actual first-party or native reports.

`runtime_modules_for_runtime_profile()` and `runtime_modules_for_runtime_profile_with_plugin_registration_reports()` provide runtime-only profile bootstrap helpers. They project the selected profile into a `ProjectPluginManifest` and then reuse the existing target/module loader against that exact profile manifest, so profile helpers do not inherit legacy target defaults such as the default client UI plugin. `RuntimeProfileId::Minimal` is stricter: it returns the Bevy-style minimal core module set directly instead of the wider client baseline. The older `runtime_modules_for_target*` helpers still merge target baselines for compatibility with existing bootstrap callers.

The manifest-specific profile helpers are intentionally available through the `zircon_runtime` crate root as well as `zircon_runtime::builtin` internals. App bootstrap uses those helpers when `EntryConfig.runtime_profile()` is set and runtime plugin or feature-registration reports are present, so profile-aware provider selection does not fall back to target-only defaults. The feature-registration app path clones the optional project manifest before creating a profile fallback manifest, preserving the caller manifest for feature dependency checks while still giving the module resolver a concrete profile manifest.

The 2026-05-25 editor-host closeout reran `cargo build -p zircon_app --no-default-features --features target-editor-host --bin zircon_editor --locked --jobs 1 --target-dir D:\cargo-targets\global-ui-m3-validation` and passed. That build exercises the crate-root helper exports plus the app-side manifest-preservation path used by provider-aware editor bootstrap.

`zircon_app` now owns the linked first-party registration provider for profile bootstrap. `EntryConfig::for_runtime_profile()` maps `RuntimeProfileId` into the appropriate app `EntryProfile`, target mode, and projected profile manifest. `first_party_runtime_plugin_registrations_for_config()` then selects compiled-in first-party provider reports from that manifest and feeds them into `BuiltinEngineEntry::for_config_with_first_party_runtime_plugin_registrations()` or `EntryRunner::bootstrap_with_first_party_runtime_plugin_registrations()`. The app feature `first-party-runtime-plugins` links the non-native profile-provider crates; `first-party-navigation-runtime-plugin` links the Recast-backed navigation crate separately.

Advanced render provider collection is driven by the app render profile, not by target defaults. When `EntryConfig::render_profile` contains `VirtualGeometry`, `HybridGlobalIllumination`, or `Solari`, `first_party_runtime_plugin_registrations_for_config(...)` appends target-scoped project selections for `RuntimePluginId::VirtualGeometry`, `RuntimePluginId::HybridGi`, and `RuntimePluginId::Solari` before collecting linked providers. Those providers are compiled only behind `first-party-advanced-render-runtime-plugins`, so `DefaultRender` and normal `Client3d` profile selection remain lightweight unless a caller explicitly chooses an advanced or Solari render profile. The current Solari provider is a contract-only unavailable provider, not a visual pass implementation.

Profile bootstrap also persists app-owned platform and render config before and after module activation. The second write is intentional: runtime modules may install defaults during activation, but the selected entry profile must remain authoritative for `PLATFORM_CONFIG_KEY` and `RENDER_PROFILE_CONFIG_KEY`. Headless profile validation therefore checks both headless platform features and the `Headless` render bundle, while minimal profile validation still records disabled platform state because `MinimalPlugins` excludes the platform module.

First-party importer capability statuses use explicit package-id mappings rather than deriving capability names from package slugs. This keeps built-in catalog metadata aligned with the authoritative `[[capability_statuses]]` entries in `zircon_plugins/*_importer/plugin.toml`, including category-specific names such as `runtime.asset.importer.model.gltf`, `runtime.asset.importer.model.obj`, and `runtime.asset.importer.audio.wav`.

First-party provider crates must keep their generated `RuntimePluginDescriptor` and `RuntimePluginRegistrationReport::package_manifest` metadata aligned with the static `zircon_plugins/*/plugin.toml` row. Sound is the current M4 sample: `zircon_plugin_sound_runtime` reports `maturity = Beta` and `runtime.plugin.sound = Partial`, matching the static TOML and built-in catalog so linked provider availability cannot silently downgrade a required default-profile plugin.

## M2 Provider Chain Closure Matrix

M2 closes the gap between a profile saying "this plugin is required" and the app/export layer proving that a concrete provider exists. The Bevy precedent is simple: `DefaultPlugins` is a list of real plugin types, and `PluginGroupBuilder` reports missing or duplicate plugin edits directly. Zircon cannot copy that shape because `zircon_runtime` must not depend on `zircon_plugins`, so the equivalent proof is a structured provider chain.

| Chain stage | Bevy-derived behavior | Zircon owner | M2 gate |
| --- | --- | --- | --- |
| Metadata-only profile availability | Bevy's default group contains direct plugin entries, not warning-only placeholders. | `RuntimeProfileDescriptor::availability_report()` and built-in catalog descriptors. | Required plugins below maturity, `Externalized`, or `Stub` enter `missing_required`; optional failures stay warnings and never block `missing_required`. |
| Linked provider classification | Bevy apps add concrete plugin instances into the app. | `availability_report_with_providers()` and `availability_report_for_registration_reports()`. | `LibraryEmbed` and source-style reports classify as `linked`; wrong-target or disabled registrations are ignored; linked providers remove the same id from `externalized_missing`. |
| Native dynamic classification | Bevy plugin presence is explicit; Zircon native packages need the same explicit report bucket. | `RuntimePluginAvailabilityReport::native_dynamic` and export/native loader report merging. | Native dynamic registrations classify separately from linked providers and do not masquerade as source-linked crates. |
| Runtime module load fatality | Bevy duplicate/missing plugin cases become direct app errors or panics. | `RuntimeModuleLoadReport::{effective_required_missing,effective_errors,has_fatal_diagnostics}`. | Structured `missing_required` contributes to fatal diagnostics; no caller should need to parse `warnings` text to decide whether bootstrap failed. |
| App first-party provider injection | Bevy default plugins are available when the crate feature is compiled. | `zircon_app::entry::first_party_runtime_plugin_registrations_for_config()` and `EntryConfig::for_runtime_profile()`. | App profile bootstrap supplies linked first-party providers from the projected profile manifest; `RuntimeProfileId::Minimal` remains provider-free. |
| Feature provider registration | Bevy optional feature collections do not imply unrelated default plugins. | `runtime_modules_for_runtime_profile_with_plugin_and_feature_registration_reports()` and app render profile provider collection. | Advanced render or sound feature providers are selected only by explicit render/profile/feature selection and do not widen normal `Client3d` defaults silently. |
| Export availability | Bevy Cargo features produce a concrete build graph. | `ExportBuildPlan.runtime_plugin_availability`, `linked_runtime_crates`, `native_dynamic_packages`, and materialize reports. | Export plans serialize linked/native/externalized/missing categories; target/profile mismatch or unsupported required native provider is fatal. |
| No warning-string resolver contract | Bevy plugin errors are typed at the API boundary. | `RuntimeModuleLoadReport` plus profile/export tests. | New code must consume `RuntimePluginAvailabilityReport` or `effective_errors()` instead of checking text from `externalized_runtime_plugin_message(...)`. |

## M2 Candidate Commands

These are the M2 closure commands once M1 is green and shared Cargo targets are available:

- `cargo test -p zircon_runtime --lib plugin_extensions::profile_maturity --locked -- --nocapture`
- `cargo test -p zircon_runtime --lib plugin_extensions::export_build_plan --locked`
- `cargo test -p zircon_app --locked profile_bootstrap`
- `cargo test -p zircon_app --locked --offline --jobs 1 --features "plugin-ui,first-party-runtime-plugins" profile_bootstrap -- --nocapture --test-threads=1`
- `cargo test -p zircon_app --locked --jobs 1 --no-default-features --features "plugin-ui,first-party-runtime-plugins,first-party-navigation-runtime-plugin" runtime_profile_bootstrap_can_link_navigation_when_native_provider_feature_is_enabled --message-format short -- --nocapture --test-threads=1`
- `cargo check -p zircon_runtime --lib --locked`

Debug order matters. If a profile bootstrap fails, inspect `RuntimePluginAvailabilityReport` first, then provider report target/maturity, then app provider collection. Do not patch app bootstrap to accept a missing required plugin unless the profile descriptor itself intentionally made that plugin optional.

## Boundaries

`zircon_runtime` owns the descriptor and report contracts. First-party runtime implementations continue to be supplied from `zircon_plugins` through registration reports passed by `zircon_app` or generated export hosts. That preserves the existing boundary: runtime knows plugin ids and capabilities, but not plugin crate implementations.

`zircon_app` is the only root package in this slice that may directly depend on selected first-party runtime plugin crates, and those dependencies are feature-gated. Generated export hosts keep using their generated `zircon_plugins.rs` provider module, so export output does not depend on the app's built-in provider feature set.

## M10 Profile / Catalog Sync Gates

M10 makes profile selection maintainable. Bevy's matching practice is to treat feature docs, examples, doc builds, compile checks, and CI failure artifacts as first-class gates, not as after-the-fact cleanup. In Zircon, this document owns the profile/provider contract, while `bevy-parity-matrix.md` owns the broader subsystem parity matrix.

| Sync area | Bevy-derived behavior | Zircon source of truth | M10 gate |
| --- | --- | --- | --- |
| Profile vocabulary | Bevy exposes public feature/profile collections in `Cargo.toml` and checks generated feature docs for missing updates. | `RuntimeProfileId`, `RuntimeProfileDescriptor`, `core_profiles`, and this document's Built-In Profiles section. | Every profile id, target mode, required capability, optional plugin, and minimum maturity must appear here before a profile is promoted. |
| Catalog and plugin manifests | Bevy feature documentation is synchronized with actual Cargo features rather than manually trusted. | `RuntimePluginDescriptor::builtin_catalog()`, `RuntimePluginId`, and `zircon_plugins/*/plugin.toml` maturity/capability rows. | Catalog additions, new first-party plugin TOML files, or changed capability statuses must update this document or the parity matrix in the same change. |
| Availability categories | Bevy default plugin presence is concrete; missing/duplicate plugin edits fail at the API or CI boundary. | `RuntimePluginAvailabilityReport` buckets, `RuntimeModuleLoadReport` fatal diagnostics, and `ExportBuildPlan.runtime_plugin_availability`. | Stable/default profiles must keep required `Externalized`, `Stub`, below-minimum maturity, blocked-by-target, and missing-provider entries out of `missing_required`; optional failures must remain structured warnings. |
| Provider feature gates | Bevy optional feature collections do not imply unrelated default plugins. | `zircon_app` first-party provider features, generated export providers, and profile bootstrap tests. | New linked/native providers must be selected by explicit profile/feature/export data. App features may provide implementations, but they must not silently widen `Client2d`, `Client3d`, `Editor`, `Dev`, or `Server` defaults. |
| Docs evidence | Bevy CI has missing-feature/missing-example docs checks and all-feature rustdoc/doc-test lanes. | This document, `docs/runtime-plugins/bevy-parity-matrix.md`, and machine-readable doc headers. | The owning docs must list related code, implementation files, plan sources, tests, candidate commands, and validation notes. Docs-only continuations must state that no Cargo was run. |
| Local and CI validation | Bevy's `tools/ci` expands aliases into format, clippy, tests, doc checks, compile-fail, benches, and examples. | `.github/workflows/ci.yml`, `validate-matrix.ps1`, validation skill docs, and reporting rules. | M10 promotion requires a named validation ladder: static docs checks, focused profile/export tests, app/profile bootstrap, plugin workspace checks, export-platform matrix, workspace build/test, then validator or CI evidence when shared targets are available. |

M10 profile/catalog candidate checks:

- Scoped docs check: `git diff --check -- docs/runtime-plugins/profile-selection.md docs/runtime-plugins/bevy-parity-matrix.md ".codex/plans/ZirconEngine Bevy 级插件完成度里程碑计划.md"`.
- Placeholder scan across the profile-selection guide, parity matrix, and main milestone plan using the repository's standard placeholder-pattern set.
- Catalog/profile tests: `cargo test -p zircon_runtime --lib plugin_extensions::profile_maturity --locked -- --nocapture`; `cargo test -p zircon_runtime --lib plugin_extensions::manifest_contributions --locked -- --nocapture`; `cargo test -p zircon_runtime --lib plugin_extensions::export_build_plan --locked -- --nocapture`.
- App/provider tests: `cargo test -p zircon_app --locked --offline --jobs 1 --features "plugin-ui,first-party-runtime-plugins" profile_bootstrap -- --nocapture --test-threads=1`, plus focused provider-feature variants when those feature gates change.
- CI parity: root workspace build/test, plugin workspace check/build/test, and export platform policy match the visible `.github/workflows/ci.yml` command shapes before M10 is considered implementation-complete.

M10 debug rule: when a profile/docs-sync check fails, diagnose in this order: `RuntimePluginId` and profile descriptor drift, built-in catalog or plugin TOML drift, availability bucket semantics, app/export provider selection, stale doc frontmatter or matrix row, then CI command shape. Do not make the check pass by weakening stable/default profile rules or by deleting a doc row.

## Validation Notes

Fresh M1 profile-selection documentation review on 2026-05-25 covered Bevy `DefaultPlugins`/`MinimalPlugins`, `PluginGroupBuilder`, prelude, state, time, task pool, log, and diagnostics sources, then cross-checked Zircon `zircon_app::plugins`, `zircon_app::prelude`, `zircon_runtime::prelude`, `zircon_runtime/src/tests/{prelude,state,time,tasks}.rs`, and diagnostic-log tests. This pass was docs-only and did not run Cargo.

Fresh M2 provider-chain documentation review on 2026-05-25 covered Bevy direct plugin/default group behavior and remote protocol transport separation, then cross-checked Zircon `RuntimeModuleLoadReport`, profile availability reports, app first-party provider collection, profile bootstrap tests, and export build plan tests. This pass was docs-only and did not run Cargo.

Fresh M10 profile/catalog documentation review on 2026-05-25 covered Bevy CI feature-doc/example-doc checks, `tools/ci` command expansion, doc-check/doc-test behavior, Zircon `.github/workflows/ci.yml`, `validate-matrix.ps1`, validation rules, reporting rules, and the M10 matrix in `docs/runtime-plugins/bevy-parity-matrix.md`. This pass was docs-only and did not run Cargo.

Fresh Sound provider metadata validation on 2026-05-26 covered the M4 sample path. `zircon_plugins/sound/runtime/src/tests/manifest.rs` now checks static TOML, generated runtime descriptor, generated package manifest, and the built-in runtime catalog for `maturity = Beta` and `runtime.plugin.sound = Partial`; it also compares runtime module, dependency, event catalog, option, and component descriptor contributions. The focused runtime command `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml manifest --locked --jobs 1 -- --nocapture` passed with 3 tests and 0 failures. The focused app provider command `cargo test -p zircon_app --locked --offline --jobs 1 --features "plugin-ui,first-party-runtime-plugins" first_party_sound_provider_preserves_manifest_maturity_and_capability_status -- --nocapture --test-threads=1` passed after an isolated-target rerun, proving the linked first-party Sound provider preserves maturity, capability status, module, option, and dynamic-event catalog metadata.

The profile tests verify deterministic profile ids, deterministic manifest projection, target scoping, built-in catalog maturity, failure buckets for externalized, stub, and below-minimum required plugins, optional unavailable plugin warnings that do not block `missing_required`, provider-aware linked/native availability, and profile module loading from registration reports. Full native loader provider-chain validation is deferred to the plugin infrastructure lane.

Fresh M1 validation used `cargo test -p zircon_runtime --lib plugin_extensions::profile_maturity --locked -- --nocapture` and passed 8 profile/maturity tests with 0 failures.

Fresh M2 validation after sibling scene ECS updates and review fixes:

- `cargo check -p zircon_runtime --lib --locked` passed with warning-only output.
- `rustfmt --edition 2021 --check <scoped profile/maturity files>` passed after formatting scoped files.
- `git diff --check -- <scoped profile/maturity files and docs>` passed with line-ending normalization warnings only.
- `cargo test -p zircon_runtime --lib tests::plugin_extensions::profile_maturity::builtin_catalog_statuses_match_importer_and_physics_capability_metadata --locked --target-dir target\codex-shared-a -- --nocapture` passed after importer capability-status mappings were aligned with first-party plugin manifests.
- `cargo test -p zircon_runtime --lib plugin_extensions::manifest_contributions::builtin_runtime_catalog_entries_have_matching_plugin_manifests_and_workspace_members --locked -- --nocapture` passed the catalog-to-plugin-manifest matching test with 0 failures.
- `.\.opencode\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_runtime` passed Cargo build and Cargo test for the package.

Fresh M2 app-provider validation on 2026-05-16 used `CARGO_TARGET_DIR=C:\Users\HeJiahui\AppData\Local\Temp\opencode\zircon-profile-provider-target` because other active sessions were using the shared target directories:

- `cargo test -p zircon_app --locked --offline --jobs 1 --features "plugin-ui,first-party-runtime-plugins" entry_config_can_select_headless_render_profile_bundle -- --nocapture --test-threads=1` passed: 1 test, 0 failures.
- `cargo test -p zircon_app --locked --offline --jobs 1 --features "plugin-ui,first-party-runtime-plugins" profile_bootstrap -- --nocapture --test-threads=1` passed: 15 tests, 0 failures.
- `cargo test -p zircon_app --locked --offline --jobs 1 profile_bootstrap -- --nocapture --test-threads=1` passed: 13 tests, 0 failures.
- `CARGO_INCREMENTAL=0 cargo test -p zircon_app --locked --offline --jobs 1 --no-default-features --features "plugin-ui,first-party-runtime-plugins,first-party-navigation-runtime-plugin" profile_bootstrap --message-format short -- --nocapture --test-threads=1` passed on Windows: 18 tests, 0 failures.

Fresh M2 navigation-provider validation on 2026-05-16 found and corrected the Windows D3D12 dependency-edge skew for this path. The root lockfile still keeps `accesskit_windows v0.30.0` from Slint/`zircon_hub` on `windows 0.61.3`, but `gpu-allocator v0.28.0` now resolves to the already-present `windows 0.62.2` package so it matches `wgpu-hal v29.0.3`'s D3D12 bindings. Evidence:

- `CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=C:\Users\HeJiahui\AppData\Local\Temp\opencode\zircon-profile-provider-target cargo test -p zircon_app --locked --offline --jobs 1 --no-default-features --features "plugin-ui,first-party-runtime-plugins,first-party-navigation-runtime-plugin" runtime_profile_bootstrap_can_link_navigation_when_native_provider_feature_is_enabled --message-format short -- --nocapture --test-threads=1` passed on Windows: 1 test, 0 failures.
- `CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=/tmp/opencode/zircon-profile-provider-target cargo test -p zircon_app --locked --jobs 1 --no-default-features --features "plugin-ui,first-party-runtime-plugins,first-party-navigation-runtime-plugin" runtime_profile_bootstrap_can_link_navigation_when_native_provider_feature_is_enabled --message-format short -- --nocapture --test-threads=1` passed: 1 test, 0 failures.

Fresh M9A advanced-provider validation on 2026-05-19:

- `cargo test -p zircon_app --locked --no-default-features --features "plugin-ui,first-party-runtime-plugins,first-party-advanced-render-runtime-plugins" render_profile_runtime_plugins --jobs 1 --message-format short --color never` passed: 3 tests, 0 failures.
- `cargo test -p zircon_runtime --lib advanced_render_plugin_manifests_declare_profile_capabilities --locked --jobs 1 --message-format short --color never` passed: 1 test, 0 failures.
- `cargo test --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_virtual_geometry_runtime --lib virtual_geometry_registration_contributes_render_feature_descriptor --locked --jobs 1` passed: 1 test, 0 failures.
- `cargo test --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_hybrid_gi_runtime --lib hybrid_gi_registration_contributes_render_feature_descriptor --locked --jobs 1 --message-format short --color never` passed: 1 test, 0 failures.
- `cargo check --manifest-path zircon_plugins\Cargo.toml --workspace --locked --all-targets --jobs 1` passed after syncing the shader importer and WGSL importer `ShaderAsset` initializers to the current `texture_slots` field.
