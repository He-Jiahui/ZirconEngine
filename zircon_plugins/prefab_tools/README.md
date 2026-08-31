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
- Dist crate: `zircon_plugin_prefab_tools_dist`
- Runtime asset kind: `prefab.asset`

The runtime side contributes prefab component/package manifest metadata,
including the NativeDynamic dist contract. The editor side currently contributes
only the prefab authoring view, drawer, and inspector customization surface.
Create, open, apply, revert, and break operations are intentionally absent
until their transaction factories, prefab graph authority, and importer backend
are installed together.

Runtime DTOs must not contain editor-only open state. A prefab instance is
retained losslessly until a future transaction-backed break operation can update
the scene and source authority atomically.

Validation rejects duplicate `(entity_path, property_path)` overrides. Effective
override queries remain deterministic and retain latest-value precedence, but
build their ordered index from borrowed paths and clone only the final values.
This query behavior does not make duplicate paths admissible for mutation.
