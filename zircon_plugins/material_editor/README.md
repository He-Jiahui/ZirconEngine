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
Each target pin admits at most one incoming link. Validation and constant graph
evaluation share one borrowed node/link index, so recursive math-node evaluation
does not rebuild node maps or linearly rescan the full link array for every pin.

The output node compiles base color plus optional scalar metallic/roughness,
direct normal textures, and constant or texture-backed emissive into the existing
runtime `MaterialAsset`. Unknown pins, invalid link directions, cycles, mismatched
parameter overrides and non-finite values report errors. Texture arithmetic and
computed normal expressions remain unsupported and report that limitation.

`can_connect_material_graph` checks a proposed link against the prospective
graph without changing the authoring asset. It returns `Connect`, a borrowed
original edge for `Replace`, or `Unchanged`. It removes the occupied input from
the prospective view before checking cycles and downstream value categories;
source fanout remains valid. Compiler validation uses the same type analysis.
Incomplete math inputs can remain `Undetermined` while editing. A successful
connection check is not compile readiness or permission to publish a material.

The host must bind the result to the retained document revision, prepare one
domain delta and inverse, and use the existing permit/history lifecycle. The
current authoring schema has endpoint keys rather than persistent edge UUIDs.
The new connection tests and compiler tests remain pending managed execution;
the current workbench drag handler does not yet call this API.

The UE window reproduction and remaining host/render integration are tracked in
[the material-family record](../../docs/plans/penpot-ue-window-suite-20261024/material-family.md).
Compiler support does not establish an interactive graph or a live material
preview in the current workbench.
