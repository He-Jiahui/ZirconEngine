---
related_code:
  - zircon_editor/src/ui/workbench/layout_persistence_document.rs
  - zircon_editor/src/ui/host/layout_persistence.rs
  - zircon_editor/src/ui/workbench/layout_preset.rs
  - zircon_editor/src/ui/workbench/project/editor_workspace_document.rs
  - zircon_editor/src/ui/workbench/project/editor_workspace_persistence.rs
  - zircon_editor/src/ui/workbench/project/layout_preset_asset_document.rs
  - zircon_editor/src/ui/workbench/project/layout_preset_assets.rs
  - zircon_runtime_interface/src/serialization
---

# Workbench Layout Persistence

Workbench layout state has one in-memory authority and one versioning mechanism. The live
`WorkbenchLayout` remains owned by the editor session. Persistence adapters snapshot that state only
when an explicit save, restore, project open, or project save operation runs; retained UI projection
does not parse layout documents or perform filesystem work.

## Versioned documents

Every persisted layout payload uses the shared `$zircon` text envelope and a distinct schema:

| Payload | Schema | Storage |
| --- | --- | --- |
| Global default layout | `zircon.editor.workbench.default-layout` v1 | Runtime config value |
| Named global presets | `zircon.editor.workbench.named-layout-presets` v1 | Runtime config value |
| User/page presets | `zircon.editor.workbench.page-layout-presets` v1 | Runtime config value |
| Project workspace | `zircon.editor.workbench.project-workspace` v1 | `.zircon/editor-workspace.json` |
| Project layout preset | `zircon.editor.workbench.project-layout-preset` v1 | `editor/layout-presets/*.workbench-layout.json` |

Each schema has an explicit v0 migration step that rejects unversioned input. The retired raw JSON
readers and the nested `format_version`/`layout_version` fields do not survive as compatibility
paths. Future versions and cross-schema payloads fail closed through the typed shared loader.

Invalid project workspace files degrade to the built-in/global layout and produce an
`EditorWorkspaceRestoreDiagnostic`. Invalid config-backed layouts emit a warning and fall back to
the built-in layout or an empty preset store, so retired user configuration cannot prevent the
editor from opening or being overwritten in the new format. Project layout preset assets remain
typed fail-closed data. Encode failures retain a typed `LayoutPersistenceDocumentError`.

## Write boundary

Project workspace and project preset writers serialize borrowed views of the live snapshot, then use
the Runtime atomic file writer. The payload is not cloned solely for serialization, and interrupted
writes cannot expose a partial document. Config-backed documents cross `serde_json::Value` because
that is the ConfigManager contract; this conversion occurs only during explicit layout IO and is not
allowed in frame, pointer, projection, or paint paths.

The ownership shape follows Unreal's application-mode boundary in
`dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/ApplicationMode.cpp`: the tab manager owns the
authoritative layout, while `FLayoutSaveRestore` is the dedicated save/load boundary. Zircon keeps
the same separation without copying Unreal's INI wire format.
