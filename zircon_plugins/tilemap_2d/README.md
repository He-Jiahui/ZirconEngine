# Tilemap 2D Authoring Plugin

`tilemap_2d` is a runtime-backed authoring plugin for tilesets, tilemaps, and
tilemap layers. The package uses Unreal Paper2D as the structural reference for
asset/editor responsibilities, but all registration goes through Zircon's
plugin manifest and generic editor extension descriptors.

- Package id: `tilemap_2d`
- Runtime capability: `runtime.plugin.tilemap_2d`
- Editor capability: `editor.extension.tilemap_2d_authoring`
- Runtime crate: `zircon_plugin_tilemap_2d_runtime`
- Editor crate: `zircon_plugin_tilemap_2d_editor`
- Dist crate: `zircon_plugin_tilemap_2d_dist`
- Runtime asset kinds: `tilemap_2d.tileset`, `tilemap_2d.tilemap`

The runtime side contributes tilemap component/importer metadata and runtime
package manifest projection, including the NativeDynamic dist contract. The
editor side registers the tilemap authoring view, tilemap asset editor, asset
creation templates, component drawer, Tiled importer, menu-backed operations,
and payload schema ids.

Supported projections are `orthogonal`, `isometric_diamond`,
`isometric_staggered`, and `hexagonal_staggered`. Default import extensions are
`tmx`, `tsx`, and `json`.

Editor paint requests resolve a validated unique layer identity instead of a
layer array index. A stroke is limited to 4,096 unique cells, preflights every
address before mutation, and commits atomically. Stroke statistics scan the
tilemap once and then update occupied/empty counts from cell-state deltas rather
than rescanning every layer after every painted cell.
