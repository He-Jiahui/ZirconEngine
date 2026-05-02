# Zircon Plugins Workspace

`zircon_plugins` is intentionally separate from the root workspace. Runtime/editor export and plugin CI can compile this workspace when plugin packages are needed, while the root workspace keeps the minimal runtime/editor core fast by default.

Runtime-backed plugin packages use this shape:

- `zircon_plugins/<plugin_id>/plugin.toml`
- `zircon_plugins/<plugin_id>/runtime/Cargo.toml`
- `zircon_plugins/<plugin_id>/editor/Cargo.toml`

Editor-only plugin packages omit the `runtime` crate:

- `zircon_plugins/<plugin_id>/plugin.toml`
- `zircon_plugins/<plugin_id>/editor/Cargo.toml`

Each `plugin.toml` is a serialized `PluginPackageManifest`: it should declare
the package id/version, `sdk_api_version`, category, supported runtime targets
and export platforms, package capabilities, asset/content roots, module entries
for runtime/editor/native/VM code, dependencies/options/events/importers, and
the default packaging strategies the export planner may choose from.

Runtime crates depend only on runtime contracts. Editor crates may depend on
`zircon_editor` plus their matching runtime crate when one exists.

The runtime-backed package set is `physics`, `sound`, `texture`, `net`,
`navigation`, `particles`, `animation`, `terrain`, `tilemap_2d`,
`prefab_tools`, `rendering`, `virtual_geometry`, `hybrid_gi`,
`gltf_importer`, `obj_importer`, `texture_importer`, `audio_importer`,
`shader_wgsl_importer`, and `ui_document_importer`.
The editor-only package set is `material_editor`, `timeline_sequence`,
`animation_graph`, `runtime_diagnostics`, `ui_asset_authoring`, and
`native_window_hosting`, `editor_build_export_desktop`, and
`plugin_sdk_examples`.

`physics` and `animation` are restored as runtime/editor plugin packages in this
workspace. Their current runtime crates expose the shared runtime contracts and
module descriptors from `zircon_runtime::{physics, animation}` while the deeper
behavior move continues; catalog, export, and editor enablement treat them as
external plugin packages.

These packages are deliberately kept outside the root workspace so normal
runtime/editor checks only build the minimal core unless an export profile or
plugin CI job selects this workspace.

The Authoring plugin family follows the same package rules: `terrain`,
`tilemap_2d`, and `prefab_tools` own runtime asset/component descriptors, while
`material_editor`, `timeline_sequence`, and `animation_graph` are editor-only
authoring packages over existing runtime asset contracts.

The importer package split now has root-level runtime packages for
`gltf_importer`, `obj_importer`, `texture_importer`, `audio_importer`,
`shader_wgsl_importer`, and `ui_document_importer`. These packages publish
package manifests, runtime module declarations, capability-gated
`AssetImporterDescriptor` rows, runtime selections, and registration reports.
The older `asset_importers/{model,texture,audio,shader,data}` family crates
remain in the workspace as declaration aggregators until downstream catalog and
project-selection callers are migrated to the new package ids.

`rendering` is an umbrella runtime/editor package. Its optional feature bundles
live under `rendering/features/<feature_id>/{runtime,editor}` and own
`post_process`, `ssao`, `decals`, `reflection_probes`, `baked_lighting`,
`ray_tracing_policy`, `shader_graph`, and `vfx_graph`. The runtime crate stays
metadata-only; individual feature crates register render descriptors, executor
ids, component descriptors, or local graph compiler DTOs.

`editor_build_export_desktop` is an editor-only package for the desktop export
panel, SourceTemplate/LibraryEmbed/NativeDynamic report templates, and the menu
operations that drive host-owned export planning. `plugin_sdk_examples` is the
SDK fixture package: it contributes a sample editor window plus a sample model
importer, inspector, component drawer, and asset creation template without
requiring runtime linkage.
