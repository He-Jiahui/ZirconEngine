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
weightmap importers, terrain scene mode, menu-backed operations, and
payload schema ids.

Default import extensions are `raw`, `r16`, and `png`. Runtime export should
link this package only when selected by the project/plugin profile.

Heightfield admission parses those extensions into a typed source format once,
checks the platform-safe sample count, and publishes a canonical extension in
the import plan. The heightfield request deliberately rejects `LayerStack`;
layer stacks require a separate request carrying layer, channel, format, and
endianness semantics instead of reusing a heightfield sample plan.
