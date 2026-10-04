---
handoff_kind: failure
status: open
created_at: 2026-07-16
summary_slug: viewport-selection-model-consumer-hard-cut
origin_plan: docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md
fixing_plan: docs/plans/zircon_editor/editor/07-domain-editors-and-graph-foundation.md
origin_child_dir: docs/plans/zircon_editor/editor/05
fixing_child_dir: docs/plans/zircon_editor/editor/07
plan_link_mode: child_record_only
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
  - cargo test -p zircon_editor --lib --locked tests::editing::viewport::viewport_controller_has_no_legacy_single_selection_accessors -- --exact --test-threads=1
  - cargo test -p zircon_editor --lib --locked tests::editing::state::selection -- --test-threads=1
  - cargo test -p zircon_editor --lib --locked tests::editing::state::viewport -- --test-threads=1
  - cargo test -p zircon_editor --lib --locked tests::editing::state::play_mode -- --test-threads=1
  - cargo test -p zircon_editor --lib --locked --no-run --message-format short --color never
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

## 2026-09-25 current-source rolling reconciliation

- The current UI consumer scan reports zero matches for
  `viewport_controller.selected_node(...)` and
  `viewport_controller.set_selected_node(...)`. The controller accessor test
  `viewport_controller_has_no_legacy_single_selection_accessors` remains
  present and checks both removed definitions. Workbench state, inspector
  binding, snapshot projection, and viewport render/edit projections consume
  `SelectionModel::active_domain()` / `active_primary()` instead of restoring
  a single-value controller protocol.
- Current key-path SHA-256 manifest:
  `scene/selection/selection_model.rs=50854fa526cead7e6dc7e512a37b94f449549d7b8347aa6e57ccba4b1fa5d3cf`,
  `scene/selection/selection_mutation.rs=a4c705aedf66f1e6913682d18e2b0df2944993eea3d37a5e410908554741c108`,
  `scene/viewport/controller/scene_viewport_controller_accessors.rs=7eb4cdbac5174d2dc232844e5ce2a2946d7bbbb029262f3bf9ddb2046fad7b96`,
  `scene/viewport/controller/scene_viewport_controller_selection.rs=e633aeb23d8318676a1dc0af836ea5ab9e7202b252a63058a11d9b1565981f36`,
  `scene/viewport/controller/scene_viewport_controller_build_edit_mode_projection.rs=205ba6788b1a5e2f8a6ec3df7ca0e4f48f1e93aefc74ad4649dcd376990c9447`,
  `scene/viewport/controller/scene_viewport_controller_build_render_snapshot.rs=65d21314968c5fc83707512258ae1753187593751d5737cff768b4167fece219`,
  `ui/binding_dispatch/inspector/apply.rs=6245265013b81dff34a66d9601a6cc825d5a65e85f2e201b21911cf306e518a0`,
  `ui/binding_dispatch/inspector/subject_path.rs=5195b2d2bc8b660f0e2324dad0a4ec28a8320f255251c113ecb818558b0fa66d`,
  `ui/workbench/snapshot/data/editor_state_snapshot_build.rs=df86e4013bf92ea3d4c937a76ddead93ba79f85e74d4e79ee9986c9441444a1b`,
  `ui/workbench/state/editor_state_selection.rs=5da0191f82eabfc7019989e1e54ec0fd0b844d2cbeca1cd777b4da7126f156b5`,
  `ui/workbench/state/editor_state_viewport.rs=a8c3675859f0f543460d884ce9deae73463ed22127a9053c57db10a3e2543319`,
  `ui/workbench/state/editor_state_apply_intent.rs=4da1c102ab6a830d86c3426f985f82bdc829b7f9600bbef3f09f914019ca5501`,
  `tests/editing/viewport.rs=8841387ab11e5a452e361cf962390f6856d07752c0a2d52cf53b9530bfc0b9c0`,
  `tests/editing/state/selection.rs=c07fc30fa7e1fcbcf5a9b84f70728386a6f0183cbc233cd6eed2097d6a3da0e7`,
  `tests/editing/state/viewport.rs=7df88865d29a82f52b33a3900e35277be885b41cde3537ddc032076118f14754`,
  `tests/editing/state/play_mode.rs=9247b14a6f577e45d4032feacd338f9d8590506ceb26be8aafedac570e0d1fff`.
- Foreign dirty overlays currently include controller module/construction,
  accessors, render snapshot, handle interaction, play-mode tests, editor-state
  snapshot/state files, and an untracked overlay-provider test directory. No
  source line was edited or absorbed by this Session; these overlays remain
  owned by their existing Sessions and are retained as provenance.
- Scoped static checks: the old getter/setter scan is empty, the hard-cut
  exact test and three group filters resolve, and `git diff --check` reports no
  whitespace errors beyond the repository's normal LF-to-CRLF warnings. No
  Cargo command was run directly. The previous managed CPU-lane rejection and
  current foreign overlays cannot be reused as dynamic acceptance.
- This is a current-source/static reconciliation only. A fresh managed Windows
  selection/editing/binding batch, upward Editor M1 gate, independent C/I/M
  review of this refreshed record, canonical fixed return, and closeout remain
  pending; the failure stays `open`.

## 2026-09-25 independent static review receipt

- Reviewer `/root/review_editor03_gizmo_private` re-read the current source and
  this record at snapshot 3792. The supplied document SHA-256 was
  `3da7a52e7f4de370fdd1ab20601e878956466acd8e6cedde68c296ce53e08cbd`.
- Review result: Critical/Important/Moderate = `0/0/0`. All 16 manifest paths
  exist; the legacy `selected_node`/`set_selected_node` consumer scan is empty;
  the guard test is present; SelectionModel consumer usage and foreign-overlay
  provenance are consistent. Scoped `git diff --check` is clean apart from
  normal LF/CRLF warnings.
- This is an independent static review only. No Cargo command was run or
  represented as passing; managed Windows selection/editing/binding, upward M1,
  fixed return, and closeout remain pending.
