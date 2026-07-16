---
related_code:
  - zircon_plugins/first_party_editor_catalog/Cargo.toml
  - zircon_plugins/first_party_editor_catalog/src/lib.rs
  - zircon_plugins/Cargo.toml
  - zircon_plugins/Cargo.lock
  - zircon_plugins/navigation/editor/src/plugin.rs
  - zircon_app/Cargo.toml
  - zircon_app/src/entry/entry_runner/editor.rs
  - zircon_app/src/entry/first_party_editor_plugins.rs
  - zircon_editor/src/ui/retained_host/app.rs
  - zircon_editor/src/ui/retained_host/run_config.rs
  - zircon_editor/src/core/editor_plugin.rs
implementation_files:
  - zircon_plugins/first_party_editor_catalog/Cargo.toml
  - zircon_plugins/first_party_editor_catalog/src/lib.rs
  - zircon_plugins/Cargo.toml
  - zircon_plugins/Cargo.lock
plan_sources:
  - docs/plans/zircon_plugins/01-plugin-architecture-core.md
  - docs/plans/zircon_plugins/05-navigation.md
  - docs/plans/zircon_plugins/10-editor-integration.md
  - docs/plans/zircon_plugins/12-plugin-dx-and-structure-framework.md
  - docs/plans/zircon_plugins/12/failure-2026-07-13-plugin-editor-runtime-mirror-consumer-wiring.md
tests:
  - rustfmt --edition 2021 --check zircon_plugins/first_party_editor_catalog/src/lib.rs
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_first_party_editor_catalog --features navigation-editor-plugin --offline --target-dir <coordinator-managed>
doc_type: module-detail
---

# First-Party Editor Catalog

## Purpose

`zircon_first_party_editor_catalog` is the linked provider catalog for first-party editor plugins.
It maps project-selected plugin package ids to `EditorPluginRegistrationReport` values without
making `zircon_editor` depend on concrete plugin crates.

The composition boundary is deliberately symmetric with `zircon_first_party_runtime_catalog`:

- `zircon_runtime` owns project selections and stable plugin ids;
- `zircon_editor` owns editor registration reports and host registries;
- each `zircon_plugins/*/editor` crate owns its concrete provider;
- this catalog owns the optional provider fan-out;
- `zircon_app` remains responsible for selecting a project/profile and installing both runtime and
  editor reports before PIE starts.

This boundary follows the repository-local Fyrox static/dynamic `PluginContainer` precedent and
the Bevy `App`/`PluginGroup` composition model: concrete providers are gathered at the application
composition root before runtime schedules or editor consumers are activated.

## Selection Rules

`first_party_editor_plugin_registrations_for_manifest(...)` accepts the authoritative
`ProjectPluginManifest` and only projects enabled selections that support
`RuntimeTargetMode::EditorHost`. Duplicate package selections are collapsed by stable
`RuntimePluginId` before provider execution.

The initial feature `navigation-editor-plugin` maps `RuntimePluginId::Navigation` to
`zircon_plugin_navigation_editor::plugin_registration()`. The returned report includes the typed
Navigation PIE runtime-event consumer declared through the plugin SDK. Future first-party editor
providers extend the same catalog match rather than adding concrete plugin dependencies to
`zircon_editor` or one-off startup branches.

## Validation Status

The catalog source and its three contract tests are implemented and formatted. Coordinator-managed
job `1affc641dd064d4eb5699564c391b067` passed all three catalog tests plus the empty doc-test target.
The application composition root now projects the selected project manifest into editor reports,
passes them through `EditorHostRunConfig`, and installs them in `RetainedEditorHost` before startup
callbacks execute. The focused `zircon_app` product-composition test remains pending while the
concurrent runtime-interface owner completes its V2 ABI cutover.
