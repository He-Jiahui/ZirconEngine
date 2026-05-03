# Animation Graph Authoring Plugin

`animation_graph` is an editor-only authoring plugin for animation graphs and
state machines. Unreal AnimGraph is the authoring-structure reference, while the
runtime data contract remains the existing Zircon animation graph/state machine
asset family.

- Package id: `animation_graph`
- Required dependency: `animation`
- Required dependency capability: `runtime.plugin.animation`
- Editor capability: `editor.extension.animation_graph_authoring`
- Editor crate: `zircon_plugin_animation_graph_editor`
- Runtime export: excluded

The package registers graph editors, a graph node palette, animation player
component drawer, open/validate/compile operations, menu items, and payload
schema ids. V1 palette node ids are `clip`, `blend`, `output`, `state`,
`transition`, and `condition`.

Validation covers missing output nodes, duplicate node ids, missing clip/state
references, invalid transitions, and dependency gate behavior when animation is
disabled.
