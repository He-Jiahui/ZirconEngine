---
handoff_kind: failure
status: open
created_at: 2026-08-05
summary_slug: retained-hierarchy-dirty-refresh-full-snapshot-fallback
origin_plan: docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md
fixing_plan: docs/plans/zircon_editor/editor_layout/09-incremental-message-bus-and-refresh.md
origin_child_dir: docs/plans/zircon_editor/editor/02
fixing_child_dir: docs/plans/zircon_editor/editor_layout/09
plan_link_mode: child_record_only
related_code:
  - zircon_editor/src/ui/host/editor_event_runtime_reflection.rs
  - zircon_editor/src/ui/host/editor_host_event_controller.rs
  - zircon_editor/src/ui/control/service.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/data_sync.rs
tests:
  - cargo test -p zircon_editor --lib --locked --jobs 1 -- editor_message
  - hierarchy dirty-only frame must publish and consume a hierarchy fragment without a complete workbench reflection rebuild
---

# Layout09: retained hierarchy dirty refresh仍回退完整快照

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md`
- 来源执行切片：M2.2 hierarchy anchor increment projection / M3.1 dirty-driven binding audit
- 修复责任计划：`docs/plans/zircon_editor/editor_layout/09-incremental-message-bus-and-refresh.md`
- 交接原因：运行时到 `ViewDirtySet` 的失效投影已由 Editor02 交付；将一个 dirty view 变成 retained UI 的局部数据/树更新，及删除常规完整 snapshot fallback，属于 Layout09 的 `EditorUiControlService` 与 retained presentation owner。
- 生命周期键：`retained-hierarchy-dirty-refresh-full-snapshot-fallback`

## 失败现象与复现证据

2026-08-05 current-source audit：`EditorHostEventController::drain_pending_view_refreshes` 对含
`TREE_STRUCTURE` 的 dirty 集只发布 `SceneInspectionMessage`，但该消息在编辑器内没有消费端；同时任意
非纯 TREE mask 都调用 `refresh_reflection`，而该函数总是构建 complete workbench reflection 并通过
`EditorUiControlService::publish_snapshot -> UiEventManager::replace_tree` 替换整棵树。retained template
bridge 仍只接受完整 `SceneEntries` 并逐行同步。故删除 full refresh 会使 hierarchy 变陈旧，保留它又违反
Layout09 的「脏视图片段发布，非整体 view_model」目标；不能在 Editor02 使用别名、隐式全量回退或特殊调用点掩盖。

同一审计还确认 Editor02 已提供 `watch_edit_world_for_view` / `unwatch_edit_world_for_view` 与 token
projection，但没有任何 view open/close lifecycle caller。故 runtime dirty token 尚不能对应真实 hierarchy
实例；只补局部 tree patch 而不在 Layout09 注册和对称撤销 `WatchKey::WorldStructure`，仍不能证明脏驱动链完整。

## 最低共享层根因

Layout09 的 dirty-set 只有触发入口，缺少局部 reflection/tree patch 发布与 retained consumer 合同。
`SceneInspectionMessage` 提供 entity/parent/depth/subtree-hash anchor，但现有 `UiReflectionSnapshot` API
只能 `replace_tree`，没有由 dirty `ViewInstanceId` 定位并提交 hierarchy fragment 的原子更新路径。

## 架构修复验收

- `EditorUiControlService` / `UiEventManager` 提供显式的 view-scoped fragment/patch 发布合同；无常规路径将 hierarchy dirty 降级为完整 `replace_tree`。
- hierarchy view 实例在打开/恢复时注册其显式 `WatchKey::WorldStructure` 依赖，在关闭/替换时对称 unwatch；Editor02 已保证重复的同 view/key/mask 注册复用同一 token，Layout09 仍须在 gateway replacement 后按新 session 调用重建。
- retained hierarchy consumer 应消费相同 generation 的 `SceneInspectionMessage` 或等价 fragment，并以 anchor/subtree hash 决定受影响子树；对名称、parent、depth 或删除变化取得足够的行内容，不能只缓存 hash 导致显示陈旧。
- hierarchy-only mutation 不构建完整 workbench reflection；同帧其它 view 保持未重算。结构性 view registry 改变才允许显式完整 rebuild。
- 重新执行 Editor02 的 hierarchy dirty 驱动回归和 Layout09 的多窗口/多页签 dirty 隔离门；记录 source-bound managed Cargo 终态与独立复审后再回传。

## 禁止临时方案

- Do not add aliases, compatibility shims, silent fallback, duplicated truth, test-only bypasses, or call-site exceptions.
- 禁止把 `SceneInspectionMessage` 只作为诊断 bus 消息，同时仍将每个 hierarchy mutation 转为 `PresentationChanged`/完整 snapshot。
- 禁止为局部更新维护第二份不带 generation 的 hierarchy truth，或以「当前可见」条件漏掉后台页签的脏化。
- 禁止弱化为只断言 `ViewDirtySet` 已标记；验收必须证明 retained consumer 未触发完整 reflection。

## 修复结果与回传

Open state: `implemented_static / validation_pending`; the forward source repair is
present, but no managed current-source terminal result or `failure return` is
claimed.

### 2026-08-10 前向修复状态

- `EditorEventRuntime::drain_pending_view_refreshes` keeps a pure
  `TREE_STRUCTURE` dirty set on the `SceneInspection` publication path. It avoids
  the complete reflection fallback; wider masks remain explicitly eligible for
  the existing full path.
- Retained-host lifecycle tick code registers `WatchKey::WorldStructure`, consumes
  the retained scene-inspection publication, resolves a generation-matched
  hierarchy fragment, and applies it through the template bridge. Host teardown
  releases the matching world-structure watch token.
- The hierarchy fragment consumer owns its anchor, subtree, and generation checks,
  so it updates the affected rows without creating a second unversioned hierarchy
  truth.
- `src/tests/editor_message/refresh.rs` contains
  `hierarchy_dirty_refresh_publishes_and_consumes_a_fragment_without_snapshot_fallback`:
  it asserts no complete snapshot fallback, resolves the published fragment, and
  verifies that the retained bridge updates rows.
- Static source, formatting, and path checks pass. The 2026-08-10 managed
  `zircon_editor` validation request did not produce a terminal result before the
  command host timeout; it is neither a passing nor failing execution result.
  This handoff remains open until its declared focused/upward tests and independent
  review have managed terminal evidence, after which only the coordinator may
  return it as `fixed-*`.
- Post-repair static review found and forward-fixed a shared-checkout overwrite
  that had removed the retained SceneInspection subscriber and WorldSyncPump
  ownership from `EditorHostEventController`. Construction now registers the
  retained subscriber, host ownership contains both resources, and `Drop`
  unregisters it; a source lifecycle contract guards the complete owner path.
- The retained host row overlay no longer clones a cumulative `BTreeMap` for
  every unique sparse patch. `PersistentRowPatchMap` path-copies only the
  affected binary-trie path, retaining old model versions and O(log row_count)
  lookup/update behavior. The final static repair review reported P0/P1/P2=0
  for controller ownership, hierarchy filtering/transaction ordering, and the
  persistent patch map. This remains static evidence, not managed acceptance.
- 2026-08-10 selection-revision-gap forward repair: after the bridge repairs a
  missing selection revision, the retained host retries the interrupted sparse
  fragment exactly once only when the authoritative revision matches the
  fragment. It combines the selection and row control ids into one sparse host
  publication. A newer snapshot or any retry failure remains an explicit
  authoritative reflow, so an obsolete selection delta is never replayed.
  The focused source guard and Rust formatting pass. The first independent
  review read a stale pre-repair file version; the repeat against the current
  hash reported P0/P1/P2=0. Managed validation remains pending.
- 2026-08-11 static performance boundary audit: a stable selection revision
  retains the previous `Arc<BTreeSet<EntityId>>` while publishing a renamed
  hierarchy row, rather than collecting the complete selection again. The
  focused 10,000-selected-entity regression uses a panicking replacement
  iterator to guard that contract and verifies that the resulting message
  contains one changed anchor and an empty selection delta. This is parse- and
  source-checked evidence only. The current-source topology audit found that
  dynamically reconciled hierarchy/property rows were absent from the retained
  control index. The forward repair now rebuilds that authoritative index only
  after virtual-row topology changes, keeps sparse state writes on cached node
  ids, and adds hierarchy/property virtual-row regressions. Managed
  current-source validation is still required; this entry does not claim a
  fixed return or accepted closeout.
- 2026-08-11 independent delta review additionally found that hierarchy click
  routing must read the authoritative control-to-entity projection rather than
  round-trip the lossy `scene_node_id: i64` display property; a virtual row
  whose entity id exceeds `i64::MAX` otherwise selects the clamped value. The
  same review found the virtual component-property edit predicate scanning the
  full surface tree despite the refreshed control index. The property edit
  entry now checks the cached control id directly while retaining its existing
  binding and prefix constraints. Hierarchy selection now reads the same
  lossless control-to-entity projection used by sparse row updates, with a
  virtual-row regression above `i64::MAX`; both review repairs await managed
  current-source validation.

## 产出记录与时间

| 里程碑 | 状态 | 完成日期 | 完成项目与证据 |
|---|---|---|---|
| Editor02 -> Layout09 retained hierarchy dirty refresh handoff | `open / implemented_static_reviewed / validation_pending` | 2026-08-11 | Layout09 已接通 pure `TREE_STRUCTURE` 的 SceneInspection fragment publication、WorldStructure watch lifecycle、generation-matched retained fragment consumer 与无 snapshot fallback 回归；补齐 retained subscriber/WorldSyncPump owner lifecycle，并将累计稀疏 host-row overlay 改为 persistent path-copy index。selection revision gap 已改为先修复 overlay、同 revision 时重试原 sparse fragment、合并一次 host patch；snapshot 前进或重试异常明确 reflow。10K 稳定选择集的单行重命名复用已发布的选择快照并保持空 selection delta。随后修复虚拟 hierarchy/property 行在拓扑增删后缺失 retained control index 的问题，并新增虚拟第 11 行稀疏改名和真实 entity 点击路由回归。无 managed terminal evidence，不声称 handoff fixed 或已回传。 |
