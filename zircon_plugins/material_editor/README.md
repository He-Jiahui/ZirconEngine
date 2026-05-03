# Material Editor Authoring Plugin

`material_editor` is an editor-only authoring plugin for material graph editing.
It uses Unreal MaterialEditor as the authoring-surface reference while keeping
compiled runtime output in the existing Zircon `MaterialAsset` contract.

- Package id: `material_editor`
- Editor capability: `editor.extension.material_editor_authoring`
- Editor crate: `zircon_plugin_material_editor_editor`
- Runtime export: excluded
- Authoring asset kind: `material.graph`

The package registers a material graph editor, material asset editor, node
palette, asset creation template, menu-backed operations, and payload schema
ids. The v1 node palette contains `output`, `texture_sample`,
`scalar_parameter`, `vector_parameter`, `add`, and `multiply`.

Validation covers missing output nodes, duplicate node ids, disconnected
required inputs, parameter defaults, and capability-gated asset editor exposure.
