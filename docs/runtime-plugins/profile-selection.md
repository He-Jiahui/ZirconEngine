---
related_code:
  - zircon_runtime/src/plugin/runtime_profile.rs
  - zircon_runtime/src/plugin/core_profiles.rs
  - zircon_runtime/src/builtin/runtime_modules.rs
  - zircon_runtime/src/plugin/project_plugin_manifest/project_plugin_manifest.rs
  - zircon_runtime/src/plugin/project_plugin_manifest/project_plugin_selection.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog.rs
  - zircon_runtime/src/tests/plugin_extensions/profile_maturity.rs
  - zircon_app/src/entry/entry_config.rs
  - zircon_app/src/entry/first_party_runtime_plugins.rs
  - zircon_app/src/entry/engine_entry.rs
  - zircon_app/src/entry/entry_runner/bootstrap.rs
implementation_files:
  - zircon_runtime/src/plugin/runtime_profile.rs
  - zircon_runtime/src/builtin/runtime_modules.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog.rs
  - zircon_app/src/entry/entry_config.rs
  - zircon_app/src/entry/first_party_runtime_plugins.rs
  - zircon_app/src/entry/engine_entry.rs
  - zircon_app/src/entry/entry_runner/bootstrap.rs
plan_sources:
  - user: 2026-05-08 实现 ZirconEngine Bevy 级插件完成度里程碑计划
  - .codex/plans/ZirconEngine Bevy 级插件完成度里程碑计划.md
tests:
  - zircon_runtime/src/tests/plugin_extensions/profile_maturity.rs
  - zircon_app/src/entry/tests/profile_bootstrap.rs
  - cargo test -p zircon_runtime --lib plugin_extensions::profile_maturity --locked -- --nocapture
  - cargo test -p zircon_app --locked profile_bootstrap
  - cargo check -p zircon_runtime --lib --locked
doc_type: module-detail
---

# Profile Selection

`RuntimeProfileDescriptor` is the M1 profile contract for Bevy-grade plugin selection. It sits in `zircon_runtime::plugin` so runtime, app, export, and editor code can share one descriptor shape without making `zircon_runtime` depend on `zircon_plugins`.

## Built-In Profiles

Profiles are exposed by `RuntimeProfileId`:

- `Minimal`: core runtime capability list only; no first-party external plugin requirement.
- `Client2d`: client runtime target with sound, rendering, and texture baseline plus optional tilemap, particles, and animation.
- `Client3d`: client runtime target with sound, rendering, and texture baseline plus optional animation, navigation, particles, virtual geometry, and hybrid GI.
- `Editor`: editor host target with UI, sound, rendering, texture, and optional authoring-facing runtime plugins.
- `Dev`: editor-host diagnostics/dev profile with networking enabled as optional and advanced rendering/VFX plugins available.
- `Server`: server runtime target; it does not enable UI, rendering, or audio listener defaults.

Each descriptor carries target mode, default plugins, optional plugins, required capabilities, minimum maturity, and whether required externalized plugins are allowed. Stable/default-style profiles use `minimum_maturity = Beta` and do not allow required `Externalized` or `Stub` plugins.

## Manifest Projection

`RuntimeProfileDescriptor::project_manifest()` converts a profile into a deterministic `ProjectPluginManifest` by preserving default plugin order and assigning each selection to the profile target mode. The legacy `default_manifest_for_target()` behavior is preserved for runtime bootstrap compatibility, while `manifest_for_runtime_profile()` exposes the new profile-driven manifest for export/editor surfaces that opt into M1 profile selection.

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

`runtime_modules_for_runtime_profile()` and `runtime_modules_for_runtime_profile_with_plugin_registration_reports()` provide runtime-only profile bootstrap helpers. They project the selected profile into a `ProjectPluginManifest` and then reuse the existing target/module loader against that exact profile manifest, so profile helpers do not inherit legacy target defaults such as the default client UI plugin. The older `runtime_modules_for_target*` helpers still merge target baselines for compatibility with existing bootstrap callers.

`zircon_app` now owns the linked first-party registration provider for profile bootstrap. `EntryConfig::for_runtime_profile()` maps `RuntimeProfileId` into the appropriate app `EntryProfile`, target mode, and projected profile manifest. `first_party_runtime_plugin_registrations_for_config()` then selects compiled-in first-party provider reports from that manifest and feeds them into `BuiltinEngineEntry::for_config_with_first_party_runtime_plugin_registrations()` or `EntryRunner::bootstrap_with_first_party_runtime_plugin_registrations()`. The app feature `first-party-runtime-plugins` links the non-native profile-provider crates; `first-party-navigation-runtime-plugin` links the Recast-backed navigation crate separately.

First-party importer capability statuses use explicit package-id mappings rather than deriving capability names from package slugs. This keeps built-in catalog metadata aligned with the authoritative `[[capability_statuses]]` entries in `zircon_plugins/*_importer/plugin.toml`, including category-specific names such as `runtime.asset.importer.model.gltf`, `runtime.asset.importer.model.obj`, and `runtime.asset.importer.audio.wav`.

## Boundaries

`zircon_runtime` owns the descriptor and report contracts. First-party runtime implementations continue to be supplied from `zircon_plugins` through registration reports passed by `zircon_app` or generated export hosts. That preserves the existing boundary: runtime knows plugin ids and capabilities, but not plugin crate implementations.

`zircon_app` is the only root package in this slice that may directly depend on selected first-party runtime plugin crates, and those dependencies are feature-gated. Generated export hosts keep using their generated `zircon_plugins.rs` provider module, so export output does not depend on the app's built-in provider feature set.

## Validation Notes

The profile tests verify deterministic profile ids, deterministic manifest projection, target scoping, built-in catalog maturity, failure buckets for externalized, stub, and below-minimum required plugins, optional unavailable plugin warnings that do not block `missing_required`, provider-aware linked/native availability, and profile module loading from registration reports. Full native loader provider-chain validation is deferred to the plugin infrastructure lane.

Fresh M1 validation used `cargo test -p zircon_runtime --lib plugin_extensions::profile_maturity --locked -- --nocapture` and passed 8 profile/maturity tests with 0 failures.

Fresh M2 validation after sibling scene ECS updates and review fixes:

- `cargo check -p zircon_runtime --lib --locked` passed with warning-only output.
- `rustfmt --edition 2021 --check <scoped profile/maturity files>` passed after formatting scoped files.
- `git diff --check -- <scoped profile/maturity files and docs>` passed with line-ending normalization warnings only.
- `cargo test -p zircon_runtime --lib tests::plugin_extensions::profile_maturity::builtin_catalog_statuses_match_importer_and_physics_capability_metadata --locked --target-dir target\codex-shared-a -- --nocapture` passed after importer capability-status mappings were aligned with first-party plugin manifests.
- `cargo test -p zircon_runtime --lib plugin_extensions::manifest_contributions::builtin_runtime_catalog_entries_have_matching_plugin_manifests_and_workspace_members --locked -- --nocapture` passed the catalog-to-plugin-manifest matching test with 0 failures.
- `.\.opencode\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_runtime` passed Cargo build and Cargo test for the package.
