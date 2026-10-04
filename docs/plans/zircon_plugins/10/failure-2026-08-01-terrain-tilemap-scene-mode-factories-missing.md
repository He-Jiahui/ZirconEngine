---
handoff_kind: failure
status: open
created_at: 2026-08-01
summary_slug: terrain-tilemap-scene-mode-factories-missing
origin_plan: docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md
fixing_plan: docs/plans/zircon_plugins/10-editor-integration.md
origin_child_dir: docs/plans/zircon_editor/editor/05
fixing_child_dir: docs/plans/zircon_plugins/10
plan_link_mode: child_record_only
related_code:
  - zircon_plugins/editor_support/src/lib.rs
  - zircon_plugins/terrain/editor/src/plugin.rs
  - zircon_plugins/tilemap_2d/editor/src/plugin.rs
  - zircon_editor/src/scene/modes/scene_mode_registration.rs
related_failures:
  - docs/plans/zircon_editor/editor/05/failure-2026-07-31-scene-mode-input-ownership-hardcut.md
tests:
  - Terrain sculpt executable scene-mode input and transaction tests
  - Tilemap paint executable scene-mode input and transaction tests
---

# Plugins10: Terrain and Tilemap publish descriptors without scene-mode factories

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md`
- 来源执行切片：Editor05 executable scene-mode hard cut
- 修复责任计划：`docs/plans/zircon_plugins/10-editor-integration.md`
- 交接原因：Editor05 已把 scene-mode 注册硬切为 factory-backed contract；Terrain 与 Tilemap 的真实 authoring mode、事务和 overlay 生命周期属于插件编辑器集成责任。

## 失败现象与复现证据

Terrain sculpt and Tilemap paint were contributed through the shared authoring batch as
descriptor-only scene-mode metadata. Neither plugin owns an executable
`EditorSceneMode` factory, input effects, transaction adapter, or overlay lifecycle. Editor05
hard-cut the batch to `Vec<SceneModeRegistration>` and removed these descriptor-only entries;
inventing passive modes would leave clickable controls that silently perform no authoring.

## 最低共享层根因

The retired batch accepted presentation descriptors without requiring an `EditorSceneMode`
factory. Terrain and Tilemap therefore projected availability without owning input effects,
transactions, capability teardown, or overlay output.

## 架构修复验收

- Terrain and Tilemap each contribute a plugin-owned `SceneModeRegistration` with an exact mode id
  and factory.
- Primary input, pointer capture, overlay generation, transaction commit/cancel, and capability
  disable behavior must use the Editor05 mode/effect contracts without direct world mutation.
- Re-add toolbar projection only after focused behavior tests prove the factory output id and
  authoring result.

## 禁止临时方案

No descriptor-only batch field, shared no-op mode, PassThrough placeholder, direct viewport/world
mutation, compatibility shim, or test-only factory.

## 修复结果与回传

Open state: `blocked_on_lower_editor05_contract / plugin factories and operations still open`.
Descriptor-only Terrain/Tilemap entries have been removed, but neither plugin has delivered a
production factory, authoring transaction proof, capability-disable lifecycle, or focused Cargo
GREEN. The toolbar projection must remain absent until those product contracts pass.

The 2026-09-21 current-source audit confirmed that this failure cannot be repaired honestly by
adding plugin-local factory shells first. `SceneModeCtx` exposes selection, settings and overlay
invalidation publicly, while its input-effect sink and the four built-in pointer/selection/transform
effects remain crate-private. `ViewportFeedback` carries only the built-in transform request. A
Terrain or Tilemap mode therefore has no supported way to own/release/cancel primary-pointer
capture, query the immutable camera/viewport/pointer projection needed for a spatial brush, or
route a commit request into the host operation/transaction authority. Terrain sculpt and Tilemap
paint command descriptors also do not register an executable authoring operation factory, so
emitting their current operation ids would end in `MissingFactory`, not an authoring result.

The lower dependency is now explicitly tracked by
[Editor05 scene-mode input ownership hard-cut](../../zircon_editor/editor/05/failure-2026-07-31-scene-mode-input-ownership-hardcut.md).
Plugins10 remains responsible, after that contract lands, for the two production mode factories,
their plugin-owned operation factories and inverse/undo data, spatial overlays, exact activation
ids, and capability retirement tests. No `PassThrough` mode, in-memory-only success sink, direct
world write, or toolbar re-addition is accepted as interim completion.

| 日期 | 项目 | 状态 | 证据 |
| --- | --- | --- | --- |
| 2026-08-01 | descriptor-only Terrain/Tilemap mode removal | open | `EditorAuthoringContributionBatch` now accepts executable `scene_modes`; Terrain and Tilemap retain their commands/views but no longer publish false mode availability. Plugins10 owns both real factories and product behavior gates. |
| 2026-09-21 | Editor05 authoring-effect dependency audit | `waiting_dependency` | Current public mode contract has no plugin-safe capture/query/operation effect path, and current sculpt/paint descriptors have no operation factories. Linked the existing lower Editor05 failure and preserved the absent toolbar; no placeholder source was added. |
