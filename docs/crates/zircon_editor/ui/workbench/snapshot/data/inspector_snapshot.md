---
related_code:
  - zircon_editor/src/ui/workbench/snapshot/data/inspector_snapshot.rs
  - zircon_editor/src/ui/workbench/snapshot/data/inspector_snapshot/native_fields.rs
  - zircon_editor/src/ui/workbench/snapshot/data/editor_state_snapshot_build.rs
  - zircon_editor/src/ui/host/play_inspector_projection.rs
  - zircon_editor/src/ui/host/editor_host_event_controller/play_inspector.rs
  - zircon_editor/src/ui/workbench/project/asset_workspace_state/inspector_resource_labels.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/inspector_component_source.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/data_sync.rs
  - zircon_editor/src/ui/binding_dispatch/inspector/apply.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/zui_visual_acceptance/product_presentation.rs
  - dev/penpot/plugins/apps/zircon-zui-plugin/tools/zui-layout-workbench-projection.ts
implementation_files:
  - zircon_editor/src/ui/workbench/snapshot/data/inspector_snapshot/native_fields.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/inspector_component_source.rs
  - zircon_editor/src/ui/workbench/project/asset_workspace_state/inspector_resource_labels.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/zui_visual_acceptance/product_presentation/inspector_components.rs
  - dev/penpot/plugins/apps/zircon-zui-plugin/tools/zui-layout-inspector-properties.ts
plan_sources:
  - user: 2026-10-03 genuine selected Camera, Light, Mesh and dynamic component Inspector coverage
tests:
  - zircon_editor/src/ui/workbench/snapshot/data/inspector_snapshot/native_fields/tests.rs
  - zircon_editor/src/tests/host/retained_callback_dispatch/template_bridge/actual_inspector_components.rs
  - zircon_editor/src/ui/host/play_inspector_projection.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/zui_visual_acceptance/product_presentation/inspector_components/tests.rs
  - dev/penpot/plugins/apps/zircon-zui-plugin/src/zui-layout-inspector-components.spec.ts
doc_type: module-detail
---

# Selected component inspection

`InspectorSnapshot.native_fields` holds read-only UI projections of registered native Camera, MeshRenderer, and Light fields. Edit snapshots read `Scene.inspection_fields_artifact`; Play snapshots use the same Runtime inspection rows returned by the world query. Registered component type identities select the native rendering fields. Transform and RenderLayerMask retain their existing typed controls.

Native data has no plugin owner or customization metadata. It keeps stable `type_path.field_name` identities separately from human labels and display values. Resource identities are resolved through the already published asset catalog or resource generation into locators; unavailable references remain read-only and display `Unavailable`. Inspection does not load assets.

The template bridge combines native fields with every dynamic component's properties in the existing virtual property list. Each dynamic component keeps its source identity and independently requires its existing customization admission before a row becomes editable. Labels and search retain component names when several components share the list.

Native property rows disable text editing, click activation and focus. Inspector property batch and draft binding routes reject these native field identities. Existing authoritative reflected commands, Scene property setters and runtime gateway mutation contracts remain unchanged.

The retained production painter, layout and scroll owner remain the authored `WorkbenchInspectorMeshProperties` virtual list and shared property-row primitive. No separate native component layout or writable backend is introduced.

The version 1 neutral ProductPresentation component DTO groups native fields by registered type identity and includes every dynamic component with its own edit admission. Labels and already resolved values come from the same Inspector snapshot as the normal bridge. The Penpot review host materializes field instances from the mounted Slot04 callsite after shared prefab expansion; generated rows retain field identities without replacing authored source ownership. Rotation uses the actual nullable snapshot, displaying degree values or the unavailable dash as read-only.
