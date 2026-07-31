---
handoff_kind: failure
status: open
created_at: 2026-08-01
summary_slug: terrain-tilemap-scene-mode-factories-missing
origin_plan: docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md
fixing_plan: docs/plans/zircon_plugins/10-editor-integration.md
origin_child_dir: docs/plans/zircon_editor/editor/05
fixing_child_dir: docs/plans/zircon_plugins/10
related_code:
  - zircon_plugins/editor_support/src/lib.rs
  - zircon_plugins/terrain/editor/src/plugin.rs
  - zircon_plugins/tilemap_2d/editor/src/plugin.rs
  - zircon_editor/src/scene/modes/scene_mode_registration.rs
tests:
  - Terrain sculpt executable scene-mode input and transaction tests
  - Tilemap paint executable scene-mode input and transaction tests
---

# Plugins10: Terrain and Tilemap publish descriptors without scene-mode factories

## Failure

Terrain sculpt and Tilemap paint were contributed through the shared authoring batch as
descriptor-only scene-mode metadata. Neither plugin owns an executable
`EditorSceneMode` factory, input effects, transaction adapter, or overlay lifecycle. Editor05
hard-cut the batch to `Vec<SceneModeRegistration>` and removed these descriptor-only entries;
inventing passive modes would leave clickable controls that silently perform no authoring.

## Required Repair

- Terrain and Tilemap each contribute a plugin-owned `SceneModeRegistration` with an exact mode id
  and factory.
- Primary input, pointer capture, overlay generation, transaction commit/cancel, and capability
  disable behavior must use the Editor05 mode/effect contracts without direct world mutation.
- Re-add toolbar projection only after focused behavior tests prove the factory output id and
  authoring result.

## Forbidden Workarounds

No descriptor-only batch field, shared no-op mode, PassThrough placeholder, direct viewport/world
mutation, compatibility shim, or test-only factory.

## 产出记录与时间

| 日期 | 项目 | 状态 | 证据 |
| --- | --- | --- | --- |
| 2026-08-01 | descriptor-only Terrain/Tilemap mode removal | open | `EditorAuthoringContributionBatch` now accepts executable `scene_modes`; Terrain and Tilemap retain their commands/views but no longer publish false mode availability. Plugins10 owns both real factories and product behavior gates. |
