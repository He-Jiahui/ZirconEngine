---
handoff_kind: failure
status: open
created_at: 2026-07-16
summary_slug: viewport-selection-model-consumer-hard-cut
origin_plan: docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md
fixing_plan: docs/plans/zircon_editor/editor/07-domain-editors-and-graph-foundation.md
origin_child_dir: docs/plans/zircon_editor/editor/05
fixing_child_dir: docs/plans/zircon_editor/editor/07
related_code:
  - zircon_editor/src/scene/viewport/controller/scene_viewport_controller_accessors.rs
  - zircon_editor/src/ui/binding_dispatch/inspector
  - zircon_editor/src/ui/workbench/snapshot
  - zircon_editor/src/ui/workbench/startup
  - zircon_editor/src/ui/workbench/state
tests:
  - cargo test -p zircon_editor --lib --locked scene::selection
  - cargo test -p zircon_editor --lib --locked tests::editing
  - cargo test -p zircon_editor --lib --locked tests::host::binding_dispatch
---

# Editor07: viewport SelectionModel consumer hard cut

## 产出记录与时间

| 状态 | 记录日期 | 完成项目与当前门禁 |
|---|---|---|
| `FIX IMPLEMENTED / MANAGED VALIDATION BLOCKED` | 2026-07-16 | 28 处生产 consumer 与 controller/test 调用已整体迁移，旧 controller API 已删除；多选、删除、PIE 双域和 history 完整选择快照覆盖已落盘。静态旧符号扫描为 0 且 `git diff --check` 通过。TDD RED/当前源码 GREEN 尚未获得受管 CPU lane：coordinator 以 foreign Render18 reservation `39d9c5788f09464fb20ea4c761164db4` 拒绝 acquire，因此本 failure 继续保持 open，不提前生成 fixed 回传。 |
| `INDEPENDENT REVIEW P0/P1/P2 = 0/0/0` | 2026-07-16 | 最终树独立复审确认：旧 controller API 无生产定义/调用，多选 hierarchy 与 primary 再点击收束正确，普通编辑和 Undo/Redo 不改选择，create/delete/import 恢复完整有序快照，PIE Play 域隔离与双域恢复闭环；未把静态审查冒充 Cargo pass。 |

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md`
- 来源执行切片：M1.1 双域 `SelectionModel` 与 viewport 权威状态
- 修复责任计划：`docs/plans/zircon_editor/editor/07-domain-editors-and-graph-foundation.md`
- 交接原因：剩余调用全部位于 Editor07 当前 `ui/binding_dispatch`、`ui/workbench`、host tests owner 范围，Editor05 不得越权形成跨会话半迁移。

## 失败现象与复现证据

viewport 的旧 `selected: Option<u64>` 字段已经删除，真实数据只存在于
`SelectionModel`。但 workbench 仍经单值 getter/setter 读取或覆盖当前域，
使多选集合、primary、Edit/Play 域与 generation 无法成为端到端唯一合同。
若只删除 controller 方法，当前源码会产生 28 处生产编译错误；若保留并
提交，则违反新版架构不兼容旧入口的硬切要求。

静态复现：

```powershell
git grep -n -E 'viewport_controller\.(selected_node|set_selected_node)\(' -- zircon_editor/src/ui
```

当前输出精确为 28 行；不包含名称相同但类型不同的 widget-reflector API。

## 最低共享层根因

Editor05 已替换存储 owner，但 Editor07 的 workbench projection、intent 与
binding consumer 尚未切到集合模型，形成“新存储 + 旧单值协议”的半迁移。
修复必须从这组共享 consumer 一次性向下收束，不能在 controller 继续保留
旧协议以隐藏上层迁移债务。

## 架构修复验收

- binding、snapshot、startup、intent、play-mode、selection 与 viewport state
  直接消费 `SelectionModel` 的 active-domain items/primary/mutation API。
- 单选 intent 必须显式表达 replace/select-only 语义；不得用 `Option<u64>`
  覆盖多选集合，Edit/Play 切换不得串值。
- 删除 `SceneViewportController::selected_node` 与 `set_selected_node`，并增加
  源码守卫，禁止同名兼容入口恢复。
- 回跑 selection、editing、binding-dispatch 与 command-eval 投影测试；三视图
  必须读取同一 selection revision。

## 禁止临时方案

- 禁止 alias、compatibility shim、deprecated wrapper、silent fallback、双份
  selection truth 或逐调用点例外。
- 禁止由 Editor05 在 Editor07 活跃 owner 未释放时直接修改 workbench 文件。
- 禁止只改测试或只删方法而留下无法编译的生产调用。

## 修复结果与回传

实现结果：consumer 已直接读取 `SelectionModel::active_primary`，所有选择写入
均使用显式 active-domain mutation；PIE session 保存完整模型，legacy history
仅为选择型命令保存有序 before/after snapshot，普通编辑不会压扁多选。等待 managed current-source
selection/editing/binding gate 后再通过 lifecycle key 返回 fixed；当前不声明 pass。
