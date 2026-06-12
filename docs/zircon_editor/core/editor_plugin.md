---
related_code:
  - zircon_editor/src/core/editor_plugin.rs
  - zircon_editor/src/core/editor_plugin_catalog_gen.rs
  - zircon_editor/build.rs
  - zircon_editor/Cargo.toml
  - zircon_editor/src/core/editor_extension.rs
  - zircon_editor/src/ui/host/editor_extension_registration.rs
  - zircon_runtime_interface/src/plugin_diagnostics.rs
implementation_files:
  - zircon_editor/src/core/editor_plugin.rs
  - zircon_editor/src/core/editor_plugin_catalog_gen.rs
  - zircon_editor/build.rs
  - zircon_editor/Cargo.toml
  - zircon_runtime_interface/src/plugin_diagnostics.rs
plan_sources:
  - user: 2026-06-12 implement docs/plans/zircon_plugins plugin architecture code
  - docs/plans/zircon_plugins/01-plugin-architecture-core.md
tests:
  - zircon_editor/src/tests/editor_plugin_catalog_consistency.rs::builtin_editor_catalog_entries_are_derived_from_plugin_manifests
  - zircon_editor/src/tests/editor_plugin_catalog_consistency.rs::editor_module_plugin_manifests_are_present_in_builtin_catalog
  - zircon_editor/src/tests/editor_plugin_catalog_consistency.rs::editor_plugin_catalog_reports_missing_capabilities_as_structured_diagnostics
  - cargo check -p zircon_runtime_interface --lib --locked --message-format short
  - cargo check -p zircon_editor --lib --locked --message-format short
  - cargo test -p zircon_editor --lib editor_plugin_catalog_consistency --locked --message-format short -- --nocapture
doc_type: module-detail
---

# Editor Plugin Catalog

`EditorPluginCatalog` registers editor plugin descriptors and packages, collects package manifests, and merges extension registries. Its built-in descriptor list is generated from `zircon_plugins/*/plugin.toml` at editor build time instead of being maintained as a parallel hand-written table. `zircon_editor/build.rs` scans every plugin manifest, extracts `[[modules]]` entries with `kind = "editor"`, writes `editor_plugin_catalog_gen.rs` into `OUT_DIR`, and `zircon_editor/src/core/editor_plugin_catalog_gen.rs` maps those generated rows into `EditorPluginDescriptor` values.

The generated catalog deliberately keeps `plugin.toml` as the source of truth for plugin id, display name, category, editor crate name, and required capabilities. The consistency tests compare the in-process built-in catalog against the current manifest set so a new editor module is visible to editor tooling as soon as the manifest declares it.

`EditorPluginCatalog` also has an explicit `validate_capabilities(...)` pass that checks a caller-provided capability set against every registered editor plugin capability. Missing capabilities are reported as shared `RegistrationDiagnostic` values from `zircon_runtime_interface`, using code `editor.capability.missing` and `Error` severity.

This is intentionally diagnostic-only. Existing editor extension registration behavior is unchanged: `EditorEventRuntime::register_editor_plugin_registration(...)` still forwards plugin capabilities as required capabilities on the installed extension registration. The new report gives editor/plugin tooling a structured way to explain why a registration would be disabled instead of relying on a silent boolean capability gate.

Validation status: `cargo check -p zircon_runtime_interface --lib --locked --message-format short` passes. `cargo check -p zircon_editor --lib --locked --message-format short` passes with existing warnings. `cargo test -p zircon_editor --lib editor_plugin_catalog_consistency --locked --message-format short -- --nocapture` passes 4 focused catalog tests after the Windows link step completes.
