---
related_code:
  - zircon_runtime/src/core/framework/project/mod.rs
  - zircon_runtime/src/core/framework/project/export_profile.rs
  - zircon_runtime/src/core/framework/project/runtime_profile_id.rs
  - zircon_runtime/src/core/framework/project/project_plugin_manifest/mod.rs
  - zircon_runtime/src/core/framework/project/project_plugin_manifest/project_plugin_manifest.rs
  - zircon_runtime/src/core/framework/project/project_plugin_manifest/project_plugin_selection.rs
  - zircon_runtime/src/core/framework/project/project_plugin_manifest/project_plugin_feature_selection.rs
  - zircon_runtime/src/asset/project/manifest/project_manifest.rs
  - zircon_runtime/src/plugin/runtime_profile/descriptor.rs
implementation_files:
  - zircon_runtime/src/core/framework/project/mod.rs
  - zircon_runtime/src/core/framework/project/export_profile.rs
  - zircon_runtime/src/core/framework/project/runtime_profile_id.rs
  - zircon_runtime/src/core/framework/project/project_plugin_manifest/mod.rs
  - zircon_runtime/src/core/framework/project/project_plugin_manifest/project_plugin_manifest.rs
  - zircon_runtime/src/core/framework/project/project_plugin_manifest/project_plugin_selection.rs
  - zircon_runtime/src/core/framework/project/project_plugin_manifest/project_plugin_feature_selection.rs
  - zircon_runtime/src/core/framework/project/project_plugin_manifest/project_plugin_selection_access.rs
  - zircon_runtime/src/core/framework/project/project_plugin_manifest/project_plugin_selection_builder.rs
  - zircon_runtime/src/core/framework/project/project_plugin_manifest/project_plugin_state.rs
plan_sources:
  - docs/plans/zircon_runtime/frameworks/05-subsystem-decoupling-contracts.md
tests:
  - tools/tests/test_frameworks_05_layer_direction.py
  - zircon_runtime/src/asset/tests/project/manifest.rs
  - zircon_runtime/src/tests/plugin_extensions/project_plugin_manifest.rs
  - zircon_runtime/tests/export_build_plan_contract.rs
---

# Project and Export Contracts

`core::framework::project` is the single neutral owner of serialized project composition and export policy contracts shared by Asset, Plugin, App, Editor, and external runtime plugins. It owns:

- `ExportProfile`, build mode, target platform, host/resource/plugin policy, and packaging strategy;
- `ProjectPluginManifest`, plugin and feature selections, target filtering, crate-name projection, and exact-id overlay state;
- `RuntimeProfileId`, which identifies a project/runtime shape without importing plugin availability/catalog behavior.

Asset still owns the filesystem-backed `ProjectManifest`; its `plugins` and `export_profiles` fields directly use these neutral schema types. Plugin owns catalog resolution, availability, provider matching, export build planning, and `RuntimePluginId` parsing. The neutral schema therefore stores plugin ids as strings and its `runtime_plugin(...)` constructor accepts any displayable canonical key; builtin enum parsing occurs only in upper assembly/catalog code.

The 2026-07-13 hard cut deleted `plugin/export_profile.rs`, the entire `plugin/project_plugin_manifest/` source owner, `RuntimeProfileId` from plugin/runtime_profile, and all Plugin root re-exports. Runtime, App, Editor, SDK, first-party plugins, generated export templates, and tests now import `core::framework::project` directly. No alias or compatibility facade remains.

## Validation

- `test_project_export_and_plugin_selection_schema_has_one_neutral_owner` locks the owner and rejects old files/re-exports/imports.
- The final Frameworks05 audit reports asset→plugin 0 and all tracked forbidden edges 0.
- Frameworks05 19/19 and Frameworks03/audit 41/41 pass; compiled Windows validation remains recorded in the active failure handoff until the managed lane is available.
