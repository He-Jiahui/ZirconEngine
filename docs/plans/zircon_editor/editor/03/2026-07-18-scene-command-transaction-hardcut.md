Plan: docs/plans/zircon_editor/editor/03-command-transaction-and-undo.md
Milestone: M2
Status: in_progress
Files: ["docs/plans/zircon_editor/editor/03-command-transaction-and-undo.md", "docs/plans/zircon_editor/editor/03/failure-2026-07-19-gizmo-transaction-capture-private-interface.md", "docs/plans/zircon_editor/editor/16/2026-07-19-m2-editor-manager-context-injection-current-source.md", "docs/plans/zircon_editor/editor/16/failure-2026-07-18-editor-state-context-constructor-hardcut.md", "docs/zircon_editor/core/editing.md", "tools/tests/test_editor03_scene_transaction_hardcut_contract.py", "zircon_app/src/entry/entry_runner/editor.rs", "zircon_app/src/entry/entry_runner/editor/tests/cli_operation.rs", "zircon_app/src/entry/entry_runner/editor/tests/gui_startup.rs", "zircon_app/src/entry/entry_runner/editor/tests/host_config.rs", "zircon_app/src/entry/entry_runner/editor/tests/mod.rs", "zircon_editor/src/core/editing/command.rs", "zircon_editor/src/core/editing/context.rs", "zircon_editor/src/core/editing/engine/history.rs", "zircon_editor/src/core/editing/engine/transaction.rs", "zircon_editor/src/core/editing/history.rs", "zircon_editor/src/core/editing/intent.rs", "zircon_editor/src/core/editing/mod.rs", "zircon_editor/src/scene/viewport/interaction/gizmo_drag_state.rs", "zircon_editor/src/scene/viewport/interaction/mod.rs", "zircon_editor/src/scene/viewport/mod.rs", "zircon_editor/src/tests/editing/history.rs", "zircon_editor/src/tests/editing/reflected_command.rs", "zircon_editor/src/tests/editing/state.rs", "zircon_editor/src/tests/editing/state/play_mode.rs", "zircon_editor/src/tests/editing/state/selection.rs", "zircon_editor/src/tests/editing/state/viewport.rs", "zircon_editor/src/tests/editing/transaction_engine/locking.rs", "zircon_editor/src/tests/ui/boundary/editor_event_cutover.rs", "zircon_editor/src/ui/binding_dispatch/viewport/apply.rs", "zircon_editor/src/ui/host/editor_event_dispatch.rs", "zircon_editor/src/ui/host/editor_event_execution/viewport_event.rs", "zircon_editor/src/ui/host/editor_event_runtime_access.rs", "zircon_editor/src/ui/host/editor_host_event_controller.rs", "zircon_editor/src/ui/retained_host/app/assets/workspace.rs", "zircon_editor/src/ui/retained_host/app/startup.rs", "zircon_editor/src/ui/retained_host/app/welcome_session/session/apply.rs", "zircon_editor/src/ui/workbench/snapshot/data/editor_state_snapshot_build.rs", "zircon_editor/src/ui/workbench/startup/editor_state_construction.rs", "zircon_editor/src/ui/workbench/startup/editor_state_project.rs", "zircon_editor/src/ui/workbench/state/editor_state.rs", "zircon_editor/src/ui/workbench/state/editor_state_apply_intent.rs", "zircon_editor/src/ui/workbench/state/editor_state_play_mode.rs", "zircon_editor/src/ui/workbench/state/editor_state_selection.rs", "zircon_editor/src/ui/workbench/state/editor_state_viewport.rs", "zircon_editor/src/ui/workbench/state/editor_world_slot.rs"]

# Editor03 M2.1 场景命令事务硬切

## 范围与状态

本切片完成场景命令族的源码硬切：旧 `core/editing/history.rs`、`EditorHistory`、`Batch` 变体和命令内 selection 字段已删除；场景编辑统一由 `EditorContext::transactions()` 执行。当前状态是 `implemented_validation_pending`，不是 fixed 或 milestone complete：独立 exact46 终审 C/I/M=0/3/1 已在 r7 以 TDD 整改，增量 source 复审为 0/0/0；受管 Cargo 与真实 workbench 交互仍未完成。

## 已实现合同

- `EditorCommand` 收敛为 create/delete/update/reflected-field 四变体并实现 `EditCommand::apply/revert/try_merge`；构造器只读 `&Scene`，首个写点是 `TransactionScope::push`。
- create 首次 apply 缓存 `NodeRecord`，redo 走 record 插入；delete 纯捕获 subtree 与最后相机守卫，首次 apply 缓存删除后 active camera，redo 确定恢复；update 只应用实际变化字段；reflection capture 同步校验 schema editability，拒绝只读字段而不做探测写入。
- `CoreEditContext` 绑定 `LevelSystem + SceneSelectionSnapshot`；transaction record 统一抓取和恢复 selection，命令不再返回或携带 selection 转移。
- workbench create/delete/rename/reparent/transform/import/inspector、undo、redo 全部走 Global transaction；inspector 以同一 scope 多 push 替代 `Batch`，中途失败由引擎逆序回滚。
- gizmo 拖拽只暂存 `initial/latest Transform`，每帧不创建 command、不克隆节点名；release 只构造并提交一条 applied-transform command，100 帧回归锁定 record/command count 均为 1。
- Play session 保留 edit history，但拒绝 scene mutation/undo/redo，期间隐藏 undo/redo 能力并关闭 gizmo；退出恢复 edit scene、selection、gizmo setting 和原历史。
- update/reflected-field 命令不再重写 selection，rename/reparent/transform/inspector apply/undo/redo 均保留完整 multi-selection；只有 create/delete 等真实改变选择的命令写 selection。
- play 门禁下沉到共享 `execute_scene_commands`，`import_mesh_asset` 等非 intent 入口也无法在 play 期间写 edit world 或污染退出后的 history。
- viewport input 是 gizmo transaction 的唯一 owner；host 不再重复发送 begin/drag/end intent，binding 与 host 传播事务错误。每次拖拽变换前预检 transaction engine，record/commit 失败恢复初始 transform 并重置 drag controller。
- 普通 scene action 在 command capture 前取消 active preview；共享 executor 拒绝泄漏 capture，gizmo release 使用私有 already-applied commit 路径。release/rollback 的统一 cleanup 即使无法恢复已消失目标，也会清空 capture、重置 controller 并回传组合错误。
- `GizmoDragState` 文件、scene re-export、host `Arc` 字段和 `EditorIntent::{Begin,Drag,End}GizmoDrag` 已物理删除；project replace/clear 与 play entry 统一取消 active capture，且 history/context 清理失败时保持原 world 并向调用者返回错误。viewport rollback failure event 带 `RenderChanged`，不会只刷新 presentation 后留下旧画面。
- project replace/clear 与 play enter/exit 从 preview cleanup 到 world mutation 全程持有 exclusive engine transition；project history finalize 与 `CoreEditContext` 清理在同一 operation lane 内执行，旧 world 命令不能插入两个清理步骤之间。
- production `EditorState` 必须显式接收 `Arc<EditorContext>`。retained host 已注入 `EditorManager` Context；CLI operation 的跨计划遗漏由 [Editor16 failure](../16/failure-2026-07-18-editor-state-context-constructor-hardcut.md) 修复源码，不恢复隐式构造兼容层。
- 事件策略旧名已在独立 exact 2-path Session 中硬切为 `DelegatedToTransactionEngine`；生产 Rust 的 `EditorHistory|BatchEditorCommand|EditorCommand::Batch|Self::Batch` 命中为 0。

## 结构与性能

- 删除 167 行 scene-only history owner，不新增 facade、shim、双写或旧 API re-export。
- `command.rs` 低于 1000 行，Context、state orchestration、transaction engine、host construction 与测试仍由既有 leaf owner 承担。
- gizmo 每帧只比较并覆盖一个 `Transform`，不增长 Vec、不分配节点名或 command；history 只保留 release 时生成的一条首末状态 command。scene snapshot 不在每帧复制，reflection 仍按计划保留单字段整值快照。
- 原 1008 行 `tests/editing/state.rs` 按 `state/{selection,play_mode,viewport}.rs` 语义拆分，root 80 行，三个 leaf 分别 187/205/615 行；未使用 part/misc 机械切片。
- 共享 transaction engine 的 clear/finalize API 取代私有栈 clear；project replace/close 不保留指向旧 LevelSystem 的历史记录。

## 产出记录与时间

| 时间 | 状态 | 完成项目 | 证据 |
|---|---|---|---|
| 2026-07-18 16:08 +08:00 | `TDD-RED` | 新增纯捕获、唯一 Context transaction owner、旧 history 物理删除与回归存在性合同。 | `python tools/tests/test_editor03_scene_transaction_hardcut_contract.py`：4 tests / 4 failures。 |
| 2026-07-18 16:36 +08:00 | `实现完成-静态门通过` | 四命令族、Context scene binding、state/inspector/gizmo/undo/redo 接线、旧文件删除和 reflected 往返测试迁移完成。 | Python 合同 4/4；`rustfmt`；scoped `git diff --check` 无错误。 |
| 2026-07-18 16:55 +08:00 | `硬切补强完成-静态门5/5` | 100 帧 gizmo 单事务合并、PIE mutation/undo/redo 隔离、play gizmo 门禁、CLI manager Context 注入、事件策略旧名删除、delete redo camera 确定性与 readonly schema preflight 完成。 | Python 合同 5/5；生产 Rust 旧符号 0；相关生产文件均低于 1000 行；外部 HEAD `41cb500a`/`e2a01927` 不重叠本切片。 |
| 2026-07-18 16:55 +08:00 | `validation_pending` | 未启动受管 Cargo、产品交互、独立 review 或 milestone commit。 | [Coordinator01 full compile-input snapshot failure](../../../zircon_tooling/session_coordinator/01/failure-2026-07-18-full-compile-input-snapshot-barrier-missing.md) 尚未 fixed-return；按门禁保持 M2.2 open，不把 Python/静态证据升级为行为验收。 |
| 2026-07-19 08:04 +08:00 | `private-interface source fix GREEN / upward gate external RED` | [Gizmo transaction capture private interface](failure-2026-07-19-gizmo-transaction-capture-private-interface.md) 以 TDD 把字段与类型共同收窄到 workbench owner；Python 合同由 5/5 增为 6/6，rustfmt/diff guard 通过。 | 受管 job `0873f135c7af481fb8d70080387ceab6` / run `bf674e8ca1f24d26a88e24e40bef572f` 在进入 Editor03 前被 Plugins01 ArcSwap bridge 的 3×E0432 + E0283 + E0282 截断；job exit 101 已释放。M2.2 继续 open，等待 `plugins01-bridge-stable-snapshot-r1-20260719` 与 Text01 Cargo owner 完成后重跑。 |
| 2026-07-19 10:33 +08:00 | `exact31 frozen / managed Cargo nondiagnostic RED` | successor `editor03-editor16-context-hardcut-atomic-closeout-r2-20260719` 补齐拆出的 4 个 CLI 测试模块；snapshot `608`、M2 manifest `4d56cc596a1545c8ade20e56775683c7`、24,608 项 immutable copy 与 31 项 overlay 均零漂移；静态合同 6/6、rustfmt 与 diff guard GREEN。 | failure-bound job `5682435a212f4921b9959edd5609c7f6` 已 released，outer exit 0；inner run `82329a4e961e4ce3ad3894768f9be29c` exit 101 但 stdout/stderr 均空且副本已删除，不能定位或计入行为门。已交接 [Coordinator01 nonzero output failure](../../../zircon_tooling/session_coordinator/01/failure-2026-07-19-validation-copy-nonzero-cargo-output-missing.md)。 |
| 2026-07-19 11:09 +08:00 | `independent review findings fixed / static 9/9 GREEN` | 独立初审 C/I/M=0/3/1 暴露 multi-selection collapse、非 intent play 绕过、gizmo 静默失败与 5/5 文档漂移；按 TDD 先新增 3 条失败合同，再删除非选择命令的 selection 写入、把 play guard 下沉共享 executor、将 viewport 硬切为唯一 fallible gizmo owner并增加 transform rollback；host/binding 改为错误传播。 | Python 合同 9/9、scoped `git diff --check` 与 Rust 格式化通过；successor r3 作用域为 37 路径（33 条当前业务路径 + 4 条 future lifecycle）。受管 Cargo、真实交互和独立复审仍待完成，不声明 M2.2 关闭。 |
| 2026-07-19 11:57 +08:00 | `independent intermediate audit fixed / static 12/12 GREEN` | 中间审计继续发现 dead host gizmo owner、rollback render 未失效、跨 world/play capture 泄漏、`clear_history` 吞错、逐帧完整 command/String 分配及 1008 行测试聚合；successor r6 物理删除旧 state/intents，改 transform-only staging，切换前 fallible cleanup，viewport failure 增加 RenderChanged，并按 selection/play_mode/viewport 拆分测试。 | Python 12/12、旧 gizmo owner 符号 0、rustfmt 与 scoped diff guard GREEN；业务 manifest 为 exact46，r6 write scope 另预留 4 条 lifecycle 路径。新增回归覆盖 enter-play-during-drag、replace-world capture cancel、faulted replace/clear、host viewport failure effects。受管 Cargo 与独立终审待办。 |
| 2026-07-19 12:35 +08:00 | `independent exact46 review 0/3/1 fixed / static 13/13 GREEN` | 终审发现普通命令与 preview 可交错、release/rollback 可遗留 capture、project/play 两步 cleanup 无排他屏障及主计划仍写逐帧 push；successor r7 先补 4 条 RED 回归，再增加 exclusive engine transition、pre-capture cancel、shared executor leak guard、gizmo 私有提交与 finally cleanup，并硬切计划措辞。 | Python 13/13、相关 Rust `rustfmt --check` GREEN；`transaction.rs` 854 行、locking test 188 行、viewport test 670 行。业务 manifest 扩为 exact47，新增 `transaction_engine/locking.rs`；scoped diff、受管 Cargo 与独立复审待办。 |
| 2026-07-19 12:49 +08:00 | `independent incremental source review 0/0/0` | reviewer 逐项确认 preview interlock、release finally cleanup、exclusive project/play transition 与主计划硬切均关闭；未发现预计的 Rust 类型、隐私、生命周期、借用或 move 错误。 | snapshot `623`、run `ef708f852c564a20bb8b46826683d8e2`、exact47 manifest `8bb61cd33fec4a3eafcf14ef23d9040a`（hash `34205889...`）是复审输入；Cargo 未运行且 Coordinator01 failure 仍 open，source-only 0/0/0 不升级为行为验收。 |

## 待办与失败归属

1. Coordinator01 返回 [nonzero Cargo output failure](../../../zircon_tooling/session_coordinator/01/failure-2026-07-19-validation-copy-nonzero-cargo-output-missing.md) 后，基于 exact47 业务 manifest 重建 immutable copy，运行 Editor03 focused scene-edit tests、`cargo test -p zircon_editor --lib --locked` 和带 `target-editor-host` 的 `zircon_app` 门。
2. 完成真实 workbench create/delete/inspector/gizmo/undo/redo 与 Play 禁编辑交互；验证 100 帧拖拽 history 常数增长。
3. 独立 source review 已完成 C/I/M=0/0/0；只有受管 Cargo 与产品交互也通过后，才允许把本记录改为 accepted/fixed 并走受管 milestone commit。
4. 旧架构叙述仍存在于不属于本 exact manifest 的历史设计文档与总索引；由对应文档 owner 后续硬切，不能回流生产符号或作为兼容依据。
