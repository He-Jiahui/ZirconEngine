# Timeline Sequence Authoring Plugin

`timeline_sequence` is an editor-only authoring plugin for timeline editing over
existing animation sequence runtime assets. Unreal Sequencer is the structural
reference for track/keyframe/editor shape.

- Package id: `timeline_sequence`
- Required dependency: `animation`
- Required dependency capability: `runtime.feature.animation.timeline_event_track`
- Editor capability: `editor.extension.timeline_sequence_authoring`
- Editor crate: `zircon_plugin_timeline_sequence_editor`
- Runtime export: excluded

The package registers a timeline editor, track descriptors, asset editor,
operation menu items, and payload schema ids. V1 track types are `transform`,
`component_property`, and `event_marker`.

Validation covers time ranges, deterministic track ordering, keyframe bounds,
event payload shape, and dependency gate behavior when the animation package is
disabled.
