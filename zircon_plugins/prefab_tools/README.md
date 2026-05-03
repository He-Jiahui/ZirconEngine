# Prefab Tools Authoring Plugin

`prefab_tools` is a runtime-backed authoring plugin for prefab assets and prefab
instances. Unreal LevelInstance and BlueprintGraph are the lifecycle references,
with Zircon's runtime DTOs and generic editor operation descriptors defining the
actual package boundary.

- Package id: `prefab_tools`
- Runtime capability: `runtime.plugin.prefab_tools`
- Editor capability: `editor.extension.prefab_tools_authoring`
- Runtime crate: `zircon_plugin_prefab_tools_runtime`
- Editor crate: `zircon_plugin_prefab_tools_editor`
- Runtime asset kind: `prefab.asset`

The runtime side contributes prefab component/package manifest metadata. The
editor side registers create-from-selection, open, apply overrides, revert
overrides, and break instance operations, plus the prefab authoring view, drawer,
component drawer, templates, menu items, and payload schema ids.

Runtime DTOs must not contain editor-only open state. Breaking an instance
should remove the prefab link and leave ordinary scene state behind.
