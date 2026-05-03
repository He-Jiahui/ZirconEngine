---
related_code:
  - zircon_runtime/src/plugin/package_manifest/plugin_feature_bundle_manifest.rs
  - zircon_runtime/src/plugin/package_manifest/plugin_feature_dependency.rs
  - zircon_runtime/src/plugin/package_manifest/plugin_package_kind.rs
  - zircon_runtime/src/plugin/package_manifest/plugin_package_manifest.rs
  - zircon_runtime/src/plugin/project_plugin_manifest/project_plugin_feature_selection.rs
  - zircon_runtime/src/plugin/project_plugin_manifest/project_plugin_selection.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_feature_registration_report.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog.rs
  - zircon_runtime/src/plugin/native_plugin_loader/candidate_from_manifest.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_load_report.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog.rs
  - zircon_runtime/src/plugin/export_build_plan/from_project_manifest.rs
  - zircon_runtime/src/plugin/export_build_plan/cargo_manifest_template.rs
  - zircon_runtime/src/plugin/export_build_plan/plugin_selection_template.rs
  - zircon_runtime/src/plugin/export_build_plan/main_template.rs
  - zircon_runtime/src/builtin/runtime_modules.rs
  - zircon_app/src/entry/builtin_modules.rs
  - zircon_app/src/entry/engine_entry.rs
  - zircon_app/src/entry/entry_runner/bootstrap.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/reports/editor_plugin_status.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/reports/editor_plugin_feature_selection_update_report.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/enablement/features.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/manifest_completion/native.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/package_projection/native_project_selection.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/status/builtin.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/status/native.rs
  - zircon_editor/src/ui/slint_host/app/host_lifecycle.rs
  - zircon_editor/src/ui/slint_host/app/module_plugin_actions.rs
  - zircon_editor/src/ui/slint_host/ui/pane_data_conversion/mod.rs
  - zircon_plugins/sound/plugin.toml
  - zircon_plugins/sound/features/timeline_animation_track/runtime/src/lib.rs
  - zircon_plugins/sound/features/ray_traced_convolution_reverb/runtime/src/lib.rs
  - zircon_runtime/src/tests/plugin_extensions/manifest_contributions.rs
  - zircon_runtime/src/tests/plugin_extensions/export_build_plan.rs
  - zircon_runtime/src/tests/plugin_extensions/extension_registry.rs
  - zircon_runtime/src/tests/plugin_extensions/native_plugin_loader.rs
  - zircon_editor/src/tests/host/manager/minimal_host_contract.rs
---

# Plugin Optional Feature Bundles

Optional feature bundles model cross-plugin features as children of one owner plugin. The feature is shown and selected under the owner plugin, but its runtime/editor implementation can live in its own crate under `zircon_plugins/<owner>/features/<feature_slug>/...`.

The compatibility mode for independent feature packages keeps the same owner-facing selection model while letting a separate package provide the implementation. A package with `package_kind = "feature_extension"` declares its exported bundles in `feature_extensions`; those bundles are projected under `owner_plugin_id` and recorded in the project selection with `provider_package_id = <feature package id>`.

## Rules

- A feature has exactly one `owner_plugin_id`.
- The owner must also appear in `dependencies` with `primary = true`, and no secondary dependency may be marked primary.
- Dependencies are all-of in v1. They refer to plugin ids and public capabilities, not crate names.
- A feature is available only when the owner plugin is enabled, every dependency plugin is enabled for the target mode, every required capability is present from enabled plugins or earlier enabled features, and the feature target mode supports the export/runtime target.
- A feature-extension package is not registered as a base runtime plugin. It is a provider package for one or more `feature_extensions`, and each selected external feature requires that provider package to be enabled for the target mode.
- `ProjectPluginFeatureSelection.provider_package_id` is omitted for owner-embedded features and set only when an independent provider package owns the runtime/editor link unit.
- Optional blocked features become warnings. Required blocked features become fatal runtime/export diagnostics.
- Runtime code still declares real service dependencies with `DependencySpec`; plugin enablement is only an availability gate.

## Runtime Flow

`PluginPackageManifest.optional_features` carries owner-embedded feature bundles. `PluginPackageManifest.feature_extensions` carries independent feature-package bundles, with `package_kind = FeatureExtension` marking pure provider packages. `RuntimePluginCatalog::complete_project_manifest` mirrors both declaration forms into `ProjectPluginSelection.features` as disabled selections by default, preserving package declaration order so project manifests do not drift across runs. External projections keep the owner row but fill `provider_package_id`.

`RuntimePluginCatalog::feature_dependency_report` then evaluates enabled feature selections, resolves feature-provided capabilities in dependency order, and reports missing plugins, missing provider packages, missing capabilities, target mismatch, duplicate ids, and cycles. Feature definitions are keyed by feature id plus provider package id internally so a runtime registration from an independent provider can match the projected owner feature without colliding with owner-embedded feature registration.

Base plugin registration reports are merged first. Available feature registration reports are merged afterward, so feature modules/managers/components/render extensions can depend on services supplied by their owner and secondary plugins.

The catalog accepts the normal two-part representation where a package manifest declares a feature and the feature crate registers the same feature id at runtime. That pair is treated as one definition as long as the owner, dependencies, modules, capabilities, default packaging, and default enablement match. Two package declarations with the same feature id, two runtime feature registrations with the same feature id, or a package/runtime pair whose core contract differs remain structured diagnostics.

Cycle diagnostics only apply to enabled unresolved features that wait on capabilities declared by each other. A disabled feature that could provide a missing capability is reported as a missing capability instead, so the editor can guide the user to enable dependencies without presenting a false cycle.

`RuntimeExtensionCatalogReport` preserves all runtime registration notes in `diagnostics` and mirrors only hard failures into `fatal_diagnostics`. Blocked optional features are therefore visible to hosts without making the runtime extension report fail. Blocked required features, duplicate/ambiguous feature definitions, and registry merge errors are fatal; `is_success()` checks the fatal list rather than requiring all informational diagnostics to be empty.

Native plugin package manifests participate in the same model. The native loader preserves `optional_features` and `feature_extensions` while merging discovered manifest, descriptor, runtime entry, and editor entry package metadata, then projects runtime-capable native packages and their runtime-capable optional features into registration reports so dependency status can be evaluated with the built-in catalog plus discovered native packages. Discovery can use a runtime/editor module declared inside `feature_extensions` for pure feature-extension packages, and `runtime_plugin_registration_reports()` deliberately skips `package_kind = FeatureExtension` packages so they only contribute feature registrations.

## Editor Status Flow

The Plugin Manager status report projects `optional_features` under the owner plugin. Each feature row records whether the feature is enabled, whether its all-of dependency set is currently available for the editor host target, which runtime/editor crates would be linked, which capabilities it provides, and a dependency checklist with plugin/capability readiness for each required dependency.

The pane payload keeps the summary nested under the owner plugin so UI details can show missing plugins and missing capabilities without pretending that a checked plugin toggle replaces runtime `DependencySpec` service declarations.

Feature actions are explicit. Enabling a feature first asks the runtime catalog whether the candidate feature is available for `editor_host`; if any plugin, capability, owner-primary, or target-mode gate is missing, the action is blocked with a structured diagnostic. The dependency action updates only the dependency selections: it enables required dependency plugins and, when a dependency capability is provided by exactly one optional feature under that dependency plugin, enables that provider feature too. Provider features are resolved recursively so combinations such as `rendering.vfx_graph -> rendering.shader_graph` can be prepared in one dependency action, while cycles and multiple providers remain diagnostics. It does not silently enable the target feature; the user still confirms the feature after dependencies are ready.

Native-aware status reports use the same projection helpers as built-in plugin status. A native plugin discovered only through `plugin.toml` still shows its optional feature rows, dependency checklist, default feature crates, packaging, and target compatibility, while load-state diagnostics such as a missing dynamic library remain attached to the native plugin row.

Native-aware project completion now preserves those optional feature selections in the project manifest projection, including feature crate names, target modes, and native-default packaging. Feature enablement actions use the same built-in plus native catalog as native-aware status, so a feature declared only in `zircon_plugins/<plugin>/plugin.toml` can prepare dependencies and then enable without falling back to the built-in-only catalog.

## Export Flow

`ExportBuildPlan` links only active feature runtime crates. Owner-embedded features link from `zircon_plugins/<owner>/features/<feature_slug>/runtime`; external provider features link from `zircon_plugins/<provider_package_id>/runtime` when selected as `LibraryEmbed`. Generated source exports both:

- `runtime_plugin_registrations()` for base plugins.
- `runtime_plugin_feature_registrations()` for available optional features.

Generated `main.rs` calls `EntryRunner::bootstrap_with_runtime_plugin_and_feature_registrations`, preserving the base-plugin-first, feature-second ordering at runtime.

Blocked optional features remain in `diagnostics` only and are not linked. Blocked required features and structural feature-definition diagnostics are copied into both `diagnostics` and `fatal_diagnostics`, and materialization/editor export reports preserve the fatal list so export hosts can block packaging or surface a hard failure without parsing diagnostic strings.

Native dynamic exports now merge both `runtime_plugin_registration_reports()` and `runtime_plugin_feature_registration_reports()` from the native loader. An owner-embedded feature selected as `NativeDynamic` still travels through its owner plugin's native dynamic package; if the export profile lacks `NativeDynamic`, or the owner plugin is not selected as `NativeDynamic`, export emits a structured diagnostic and treats it as fatal for required features. An external feature selected as `NativeDynamic` exports its `provider_package_id` package independently, so the owner plugin can remain linked or source-templated while the feature provider travels as its own native package.

For catalogs that do not yet know an external feature package at source-generation time, export treats an enabled feature selection with `provider_package_id` and an enabled provider package selection as a deferred external provider. It links or packages that provider and suppresses only the "unknown feature" diagnostic for that deferred case; ordinary owner-embedded unknown features still report normally.

## Current Examples

- `sound.timeline_animation_track`
  - owner: `sound`
  - dependencies: `sound/runtime.plugin.sound`, `animation/runtime.feature.animation.timeline_event_track`
  - provides: `runtime.feature.sound.timeline_animation_track`
  - runtime crate: `zircon_plugin_sound_timeline_animation_runtime`

- `sound.ray_traced_convolution_reverb`
  - owner: `sound`
  - dependencies: `sound/runtime.plugin.sound`, `physics/runtime.plugin.physics`, `physics/runtime.capability.physics.raycast`
  - provides: `runtime.feature.sound.ray_traced_convolution_reverb`
  - runtime crate: `zircon_plugin_sound_ray_traced_convolution_runtime`

- `sound_timeline_animation_track` as an independent provider package
  - package kind: `feature_extension`
  - declares: `feature_extensions = ["sound.timeline_animation_track"]`
  - projected owner row: `sound.features["sound.timeline_animation_track"]`
  - provider gate: project must also enable plugin/package `sound_timeline_animation_track`
  - embedded path: `zircon_plugins/sound_timeline_animation_track/runtime`

## Validation

Added coverage for manifest roundtrip, project manifest nested feature selections, deterministic catalog completion, dependency availability/blocking, target mismatch diagnostics, disabled provider versus feature-cycle diagnostics, package/runtime declaration default mismatches, runtime blocked optional-vs-required fatal semantics, export linking/diagnostics/fatal diagnostics, native feature registration projection, runtime extension merge ordering, editor Plugin Manager status projection, recursive dependency enablement, native manifest optional-feature projection, native-aware project completion, and editor feature/dependency action projection.

Fresh validation on 2026-05-03:

- `cargo fmt -p zircon_runtime -p zircon_editor`
- `cargo metadata --locked --no-deps --format-version 1`
- `cargo check -p zircon_runtime --lib --locked --jobs 1 --message-format short --color never`
- `cargo test -p zircon_runtime --lib plugin_extensions --locked --jobs 1 --message-format short --color never`
- `cargo test -p zircon_runtime --lib native_manifest_merge_preserves_optional_feature_declarations --locked --jobs 1 --message-format short --color never`
- `cargo test -p zircon_editor --lib feature_status_rejects_secondary_primary_dependency --locked --jobs 1 --message-format short --color never`
- `cargo test -p zircon_editor --lib native_plugin_status_uses_manifest_when_library_is_missing --locked --jobs 1 --message-format short --color never`
- `cargo test -p zircon_editor --lib editor_manager_plugin_status_lists_owner_optional_feature_dependencies --locked --jobs 1 --message-format short --color never`
- `cargo test -p zircon_editor --lib editor_manager_feature_dependency_enablement_turns_on_unique_provider_features --locked --jobs 1 --message-format short --color never`
- `cargo test -p zircon_editor --lib shared_menu_pointer_layout --locked --jobs 1 --message-format short --color never`
- `cargo test -p zircon_editor --lib root_menu_popup_scroll_and_dismiss_flow_through_shared_pointer_bridge_in_real_host --locked --jobs 1 --message-format short --color never`
- `$env:CARGO_INCREMENTAL='0'; cargo test -p zircon_runtime --lib runtime_plugin_catalog_reports_target_mismatch_for_optional_feature --locked --jobs 1 --target-dir target-codex-runtime-check --message-format short --color never`
- `$env:CARGO_INCREMENTAL='0'; cargo test -p zircon_runtime --lib runtime_plugin_catalog_reports_feature_capability_cycles --locked --jobs 1 --target-dir target-codex-runtime-check --message-format short --color never`
- `$env:CARGO_INCREMENTAL='0'; cargo test -p zircon_editor --lib native_plugin_status_uses_manifest_when_library_is_missing --locked --jobs 1 --target-dir target-codex-runtime-check --message-format short --color never`
- `$env:CARGO_INCREMENTAL='0'; cargo test -p zircon_editor --lib native_selection_preserves_optional_feature_defaults --locked --jobs 1 --target-dir target-codex-runtime-check --message-format short --color never`
- `cargo fmt -p zircon_runtime -p zircon_app -p zircon_editor`
- `cargo metadata --locked --no-deps --format-version 1`
- `$env:CARGO_INCREMENTAL='0'; cargo test -p zircon_runtime --lib plugin_extensions --locked --jobs 1 --target-dir target-codex-runtime-check --message-format short --color never`
- `$env:CARGO_INCREMENTAL='0'; cargo test -p zircon_runtime --lib plugin_extensions --locked --jobs 1 --target-dir target-codex-runtime-check --message-format short --color never -- --test-threads=1`
- `$env:CARGO_INCREMENTAL='0'; cargo test -p zircon_runtime --lib source_template_links_external_feature_provider_runtime_crates --locked --jobs 1 --target-dir target-codex-runtime-check --message-format short --color never`
- `$env:CARGO_INCREMENTAL='0'; cargo test -p zircon_runtime --lib native_load_report_projects_optional_features_as_runtime_feature_registrations --locked --jobs 1 --target-dir target-codex-runtime-check --message-format short --color never`
- `$env:CARGO_INCREMENTAL='0'; cargo check -p zircon_app --lib --locked --jobs 1 --target-dir target-codex-runtime-check --message-format short --color never`
- `$env:CARGO_INCREMENTAL='0'; cargo test -p zircon_app --lib --locked --jobs 1 --target-dir target-codex-runtime-check --message-format short --color never`
- `git diff --check -- <optional-feature touched files>`

Workspace-wide `cargo build --workspace` / `cargo test --workspace` was not run in this session because the checkout is under active multi-session churn and this milestone used targeted package validation for the optional-feature surfaces.
