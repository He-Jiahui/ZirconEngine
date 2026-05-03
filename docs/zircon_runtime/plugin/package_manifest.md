---
related_code:
  - zircon_runtime/src/plugin/mod.rs
  - zircon_runtime/src/plugin/package_manifest/mod.rs
  - zircon_runtime/src/plugin/package_manifest/plugin_package_kind.rs
  - zircon_runtime/src/plugin/package_manifest/plugin_package_manifest.rs
  - zircon_runtime/src/plugin/package_manifest/plugin_dependency_manifest.rs
  - zircon_runtime/src/plugin/package_manifest/plugin_event_manifest.rs
  - zircon_runtime/src/plugin/package_manifest/plugin_option_manifest.rs
  - zircon_runtime/src/plugin/package_manifest/constructors.rs
  - zircon_runtime/src/plugin/extension_registry/runtime_extension_registry.rs
  - zircon_runtime/src/plugin/extension_registry/register.rs
  - zircon_runtime/src/plugin/extension_registry/access.rs
  - zircon_runtime/src/plugin/extension_registry_error.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_registration_report.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog.rs
  - zircon_runtime/src/plugin/export_build_plan/from_project_manifest.rs
implementation_files:
  - zircon_runtime/src/plugin/mod.rs
  - zircon_runtime/src/plugin/package_manifest/plugin_package_kind.rs
  - zircon_runtime/src/plugin/package_manifest/plugin_package_manifest.rs
  - zircon_runtime/src/plugin/package_manifest/plugin_dependency_manifest.rs
  - zircon_runtime/src/plugin/package_manifest/plugin_event_manifest.rs
  - zircon_runtime/src/plugin/package_manifest/plugin_option_manifest.rs
  - zircon_runtime/src/plugin/package_manifest/constructors.rs
  - zircon_runtime/src/plugin/extension_registry/runtime_extension_registry.rs
  - zircon_runtime/src/plugin/extension_registry/register.rs
  - zircon_runtime/src/plugin/extension_registry/access.rs
  - zircon_runtime/src/plugin/extension_registry_error.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_registration_report.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog.rs
  - zircon_runtime/src/plugin/export_build_plan/from_project_manifest.rs
plan_sources:
  - user: 2026-05-03 review follow-up for plugin workspace compile failure
  - user: 2026-05-02 sound plugin mixer/spatial/convolution/timeline core implementation request
  - .codex/plans/Sound 插件核心完善计划.md
  - .codex/plans/ZirconEngine 独立插件补齐计划.md
tests:
  - cargo check --manifest-path zircon_plugins/Cargo.toml --workspace --locked --all-targets --jobs 1
  - cargo check -p zircon_plugin_sound_runtime -p zircon_plugin_sound_editor --locked --message-format short (passed from zircon_plugins workspace with CARGO_TARGET_DIR=E:\Git\ZirconEngine\target\codex-sound-closeout)
  - cargo test -p zircon_plugin_sound_runtime -p zircon_plugin_sound_editor --locked --message-format short (passed from zircon_plugins workspace with CARGO_TARGET_DIR=E:\Git\ZirconEngine\target\codex-sound-closeout; 8 sound tests passed)
  - cargo check -p zircon_runtime --lib --tests --locked --offline --jobs 1 --target-dir E:\cargo-targets\zircon-independent-plugin-physics --color never
  - 2026-05-03: cargo check -p zircon_runtime --lib --tests --locked --offline --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-lib-importer-contract --message-format short --color never (passed with existing runtime warnings after re-exporting PluginPackageKind, preserving feature diagnostics, and restoring external feature export helpers)
  - zircon_runtime/src/tests/plugin_extensions/manifest_contributions.rs
  - zircon_runtime/src/tests/plugin_extensions/extension_registry.rs
doc_type: module-detail
---

# Plugin Package Manifest Extensions

## Purpose

`PluginPackageManifest` describes what a plugin package contributes before any concrete Rust runtime/editor code is activated. It carries SDK/API version, package category, supported targets/platforms, capabilities, asset/content roots, module declarations, component/UI component metadata, asset importer descriptors, optional feature bundles, and packaging metadata. The sound core and independent plugin slices add neutral manifest contribution kinds that are needed by independent plugins:

- dependencies,
- options,
- event catalogs,
- asset importer descriptors.

These fields are generic because sound is not the only plugin that needs optional feature gates, project-visible settings, or future event namespaces.

## Behavior Model

`PluginDependencyManifest` records another plugin/capability that this package expects. `required = true` means the package cannot fully operate without that dependency. `required = false` means the package exposes a gated advanced path when the capability exists.

`PluginOptionManifest` records editor/project-visible configuration metadata. Values are stored as strings in the manifest so the manifest remains a simple TOML contract and does not depend on a runtime value enum. Consumers should parse values according to `value_type`.

`PluginEventCatalogManifest` records a namespaced event catalog with a version and event list. An empty list is valid and intentional: it lets a plugin reserve and expose a future event namespace without shipping event handlers yet.

`RuntimeExtensionRegistry` mirrors options, event catalogs, manifest-declared components, UI components, and asset importer descriptors during linked plugin registration so runtime/editor hosts can discover them alongside modules, managers, render features, pass executors, and runtime providers. If a plugin has already registered a real importer backend with the same importer id, the manifest descriptor is treated as the public descriptor for that backend and the registration report does not add the diagnostic-only placeholder.

Package manifests and feature manifests expose the same `with_default_packaging(...)` builder shape. That lets standalone plugin packages, such as editor-only export plugins, override the package-level default export strategy without reaching into the public struct fields or relying on feature-bundle builders by mistake.

`PluginPackageKind` is part of the top-level `crate::plugin` public surface. Native plugin load
projection and runtime catalog feature-definition logic both consume it through that surface, so
the package-kind enum must be re-exported next to `PluginPackageManifest` rather than only from the
private package-manifest subtree.

External optional-feature providers are resolved during export planning from the completed project
plugin manifest. Enabled owner selections contribute external feature packages only when the feature
is enabled, target-compatible, and carries a provider package id that differs from the owner plugin.
This prevents disabled catalog defaults from leaking extra native or linked feature packages into a
desktop export plan.

## Constraints

- Option keys must be non-empty and trimmed.
- Event catalog namespaces must be non-empty and trimmed.
- Duplicate option keys and event namespaces are rejected by the runtime extension registry.
- Asset importer descriptors must declare at least one source extension or full suffix before they can be registered as diagnostic-only manifest declarations.
- Duplicate importer ids and duplicate importer matchers at the same priority are rejected by the asset importer registry.
- Existing plugin manifests continue to deserialize because the new fields use serde defaults.
- This layer does not resolve dependency graphs yet; it only records declared dependency metadata for package/catalog consumers.

## Test Coverage

The new sound plugin registration test proves a real package can contribute dependencies, options, components, and an empty event catalog through both its manifest and runtime extension registry.

The independent plugin follow-up adds focused runtime coverage proving `RuntimePluginRegistrationReport::from_plugin(...)` collects manifest-declared options, event catalogs, component descriptors, UI component descriptors, and asset importer descriptors, and that `RuntimePluginCatalog::runtime_extensions()` preserves those contributions when merging registration reports.

The review follow-up adds package-manifest coverage for overriding `default_packaging` through the builder API and validates the plugin workspace with `cargo check --manifest-path zircon_plugins/Cargo.toml --workspace --locked --all-targets --jobs 1`.

`cargo check -p zircon_plugin_sound_runtime -p zircon_plugin_sound_editor --locked --message-format short` and `cargo test -p zircon_plugin_sound_runtime -p zircon_plugin_sound_editor --locked --message-format short` now pass from the `zircon_plugins` workspace using `CARGO_TARGET_DIR=E:\Git\ZirconEngine\target\codex-sound-closeout`. The sound test run covered one editor registration test and seven runtime mixer/DSP/manifest tests.
