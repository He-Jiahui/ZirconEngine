# Terrain Authoring Plugin

`terrain` is a runtime-backed authoring plugin for terrain heightfields and
layer stacks. The package follows the Unreal Landscape split as a lifecycle
reference while staying inside Zircon's plugin manifest, capability, catalog,
and generic editor extension registry contracts.

- Package id: `terrain`
- Runtime capability: `runtime.plugin.terrain`
- Editor capability: `editor.extension.terrain_authoring`
- Runtime crate: `zircon_plugin_terrain_runtime`
- Editor crate: `zircon_plugin_terrain_editor`
- Runtime asset kinds: `terrain.heightfield`, `terrain.layer_stack`

The runtime side contributes the terrain component descriptor, importers, and
runtime package manifest projection. The editor side registers the terrain
authoring view, component drawer, asset creation template, heightfield and
weightmap importers, terrain viewport tool mode, menu-backed operations, and
payload schema ids.

Default import extensions are `raw`, `r16`, and `png`. Runtime export should
link this package only when selected by the project/plugin profile.
