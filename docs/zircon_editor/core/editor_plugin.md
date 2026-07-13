---
related_code:
  - zircon_editor/src/core/editor_plugin.rs
  - zircon_editor/src/core/editor_plugin_catalog_gen.rs
  - zircon_editor/build.rs
  - zircon_editor/Cargo.toml
  - zircon_editor/src/core/editor_extension.rs
  - zircon_editor/src/ui/host/editor_extension_registration.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/enablement/features.rs
  - zircon_runtime_interface/src/plugin_diagnostics.rs
implementation_files:
  - zircon_editor/src/core/editor_plugin.rs
  - zircon_editor/src/core/editor_plugin_catalog_gen.rs
  - zircon_editor/src/ui/host/editor_extension_registration.rs
  - zircon_editor/build.rs
  - zircon_editor/Cargo.toml
  - zircon_runtime_interface/src/plugin_diagnostics.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/enablement/features.rs
plan_sources:
  - user: 2026-06-12 implement docs/plans/zircon_plugins plugin architecture code
  - docs/plans/zircon_plugins/01-plugin-architecture-core.md
  - docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md
tests:
  - zircon_editor/src/tests/editor_plugin_catalog_consistency.rs::builtin_editor_catalog_entries_are_derived_from_plugin_manifests
  - zircon_editor/src/tests/editor_plugin_catalog_consistency.rs::editor_module_plugin_manifests_are_present_in_builtin_catalog
  - zircon_editor/src/tests/editor_plugin_catalog_consistency.rs::editor_plugin_catalog_reports_missing_capabilities_as_structured_diagnostics
  - zircon_editor/src/tests/host/manager/minimal_host_contract/optional_features.rs
  - zircon_editor/src/tests/host/manager/minimal_host_contract/core_contract.rs
  - zircon_editor/src/tests/host/manager/minimal_host_contract/native_plugins.rs
  - cargo check -p zircon_runtime_interface --lib --locked --message-format short
  - cargo check -p zircon_editor --lib --locked --message-format short
  - cargo test -p zircon_editor --lib editor_plugin_catalog_consistency --locked --message-format short -- --nocapture
doc_type: module-detail
---

# Editor Plugin Catalog

`EditorPluginCatalog` registers editor plugin descriptors and packages, collects package manifests, and merges extension registries. Its built-in descriptor list is generated from `zircon_plugins/*/plugin.toml` at editor build time instead of being maintained as a parallel hand-written table. `zircon_editor/build.rs` scans every plugin manifest, extracts `[[modules]]` entries with `kind = "editor"`, writes `editor_plugin_catalog_gen.rs` into `OUT_DIR`, and `zircon_editor/src/core/editor_plugin_catalog_gen.rs` maps those generated rows into `EditorPluginDescriptor` values.

The generated catalog deliberately keeps `plugin.toml` as the source of truth for plugin id, display name, category, editor crate name, and required capabilities. The consistency tests compare the in-process built-in catalog against the current manifest set so a new editor module is visible to editor tooling as soon as the manifest declares it.

`EditorPluginCatalog` also has an explicit `validate_capabilities(...)` pass that checks a caller-provided capability set against every registered editor plugin capability. Missing capabilities are reported as shared `RegistrationDiagnostic` values from `zircon_runtime_interface`, using code `editor.capability.missing` and `Error` severity.

This is intentionally diagnostic-only. `EditorHostEventController::register_editor_plugin_registration(...)` forwards plugin capabilities as required capabilities on the installed `EditorExtensionRegistration`, using the controller's Workbench shell and operation owners rather than the deleted editor-event aggregate. The report gives editor/plugin tooling a structured way to explain why a registration would be disabled instead of relying on a silent boolean capability gate.

## Optional Feature Dependency Enablement

`EditorManager::enable_project_plugin_feature_dependencies(...)` starts from `RuntimePluginCatalog::complete_project_manifest(...)` so owner plugins, feature selections, and external provider-package selections share the catalog's canonical identities. It recursively enables declared plugin dependencies and unique feature providers without enabling the requested feature itself.

An optional feature may carry an explicit `provider_package_id` distinct from its `owner_plugin_id`. After walking the feature's declared dependencies, the editor enablement path also enables that external provider selection and includes it in `enabled_dependency_plugins`. Native-aware catalogs merge `NativePluginLoadReport` package registrations and feature registrations, including standalone `FeatureExtension` providers. The runtime status layer remains authoritative: it still rejects feature enablement if the owner, a declared dependency, or the external provider is disabled. The editor does not add provider aliases, ignore missing providers, or special-case Sound.

The split tests under `minimal_host_contract/optional_features.rs` verify the builtin path: the feature is initially blocked, dependency enablement turns on `sound`, `animation`, and `sound_timeline_animation_track`, status becomes available while the feature remains disabled, and a later explicit feature enable succeeds. `native_plugins.rs` owns native discovery/export and verifies an external native `FeatureExtension` provider. `core_contract.rs` owns the minimal host boundary; the parent/native/core/optional owners are 490/619/46/258 lines.

The review-hardening changes (single projection owner, exact-key lookup, native feature-registration merge, canonical manifest lookup, and native provider regression) were rebuilt from current source in 16m49s. The native external `FeatureExtension` exact passed 1/1 in 0.13s, and the builtin external-provider exact passed 1/1 in 0.06s on the same binary. The canonical record is `docs/plans/zircon_editor/editor/01/fixed-2026-07-11-editor-m1-plugin-provider-lookup.md`; the complete Editor M1 gate remains separate and open.
