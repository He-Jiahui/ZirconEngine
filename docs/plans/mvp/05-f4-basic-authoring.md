---
related_code:
  - zircon_editor/src/scene/selection
  - zircon_editor/src/scene/modes
  - zircon_editor/src/scene/viewport
  - zircon_runtime/src/scene/inspection
  - zircon_editor/src/ui/host/command_eval_projection.rs
  - zircon_editor/src/ui/host/project_access.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch
  - zircon_editor/src/ui/template_runtime/builtin/workbench_window_template_bindings.rs
  - zircon_editor/src/core/commands
  - zircon_editor/src/core/editing
related_tests:
  - zircon_editor/src/tests/commands/when.rs
  - zircon_editor/src/tests/editing/editor_projection.rs
  - zircon_editor/src/tests/editing/viewport.rs
  - zircon_editor/src/tests/host/retained_callback_dispatch/hierarchy.rs
  - zircon_editor/src/tests/host/retained_callback_dispatch/template_bridge/workbench_inspector_property_edit.rs
  - zircon_runtime/src/scene/tests/inspection.rs
plan_sources:
  - docs/plans/mvp/index.md
  - docs/plans/mvp/04-f3-persistence.md
  - docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md
  - docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md
  - docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md
  - docs/plans/zircon_editor/editor/08-tool-orchestration-and-commands.md
  - docs/plans/zircon_editor/editor/11-serialization-and-versioning.md
  - docs/plans/zircon_editor/editor/08/fixed-2026-07-26-command-eval-scene-mode-selection-projection.md
  - docs/plans/zircon_editor/editor/05/failure-2026-07-22-world-inspection-generation-projection.md
status: blocked_by_f3
gate: F4
last_refined: 2026-07-24
---

# F4 基础编辑 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: 使用 `subagent-driven-development`（推荐）或 `executing-plans`。这是跨 Runtime inspection、Editor state、UI binding、command/transaction 和 persistence 的 C3 边界；必须使用 `zr-architecture-first-engineering`、`support-first-regression-testing` 和相应 failure owner。

**Goal:** staged `zircon_editor` 打开 F3 项目，从生产 Hierarchy 选择 cube，通过正常 Inspector UI binding 和 command/transaction 路径修改 transform，保存，销毁并重开后观察修改。

**Architecture:** Runtime 发布 editor-neutral、generation-owned world inspection；Editor 的 `SelectionModel` 和 `SceneModeStack` 是 authoring 权威；UI 只投影 snapshot 并发出 typed binding；command/transaction 修改 editor world，SaveProject 复用 F3 persistence。禁止 Hierarchy、Inspector、menu/command palette 各保留一份选择或模式真相。

**Tech Stack:** WorldInspection、SelectionModel、SceneModeStack、CommandEvalCtx、retained UI bindings、EditorTransactionEngine、EditorProjectDocument。

---

## 1. 入口条件

- [ ] F3 完成，canonical project 能在新进程中精确保留 fixed transform delta 和 asset refs。
- [ ] F4 Session 已按 owner 拆分 lease；Runtime inspection 与 Editor UI/command 不由同一未协调 Session 同时改共享 contract。
- [ ] command eval projection 的 canonical fixed return 已核对；world inspection generation projection 继续按 open failure 进入 fixing priority。
- [ ] F4 validation manifest 列出 Edit/Play 双域、selection generation、inspection generation、command enablement、transaction/save/reopen 风险。

## 2. 固定生产交互链

F4 只接受以下链路：

```text
Project open
  -> Runtime WorldInspection generation
  -> Hierarchy row projection
  -> Hierarchy selection UI binding
  -> SelectionModel(Edit domain)
  -> Inspector projection for primary selection
  -> Transform axis edit/commit UI binding
  -> typed editor operation/command
  -> EditorTransactionEngine
  -> editor world mutation + dirty generation
  -> SaveProject
  -> process teardown
  -> reopen + persisted comparison
```

任何直接调用 `World::set_*`、直接改 `main.scene.toml`、只更新 control preview、只更新 inspector snapshot 或绕过 command descriptor/transaction 的实现均不满足 F4。

## 3. 非目标

- 不要求 viewport gizmo drag；Inspector 单轴 transform 编辑足以关闭 MVP。
- 不要求 plugin overlay provider、框选、复杂多选 UI、undo history panel 或 domain editor。
- 不要求所有 100k 性能目标；但既有 production projection 的 shared generation、稳定帧零全量重建和声明的规模正确性必须验证。
- 不把 OS 鼠标坐标自动化作为唯一证据；产品 integration harness 可以注入正常 UI binding，但不得跳过 binding/command/transaction。

## 4. M5.1 World inspection production generation

### 目标

Runtime 已向 production Editor 发布 immutable world inspection generation；本切片验证 Hierarchy/Inspector 对稳定 generation 不重建全量 DTO，并关闭规模、Cargo 和 F4 trace 缺口。

### 实现切片

- [ ] 以 world inspection open failure 为权威，定义 generation identity、hierarchy rows、focused fields、changed rows/fields 和 removal 的 neutral contract。
- [ ] Runtime world/scene 变更只推进一次 inspection generation；stable frame 复用同一 `Arc` artifact。
- [ ] hierarchy 构建避免 focus 第二遍、重复 parent map、重复全表 sort 和 field-name clone；focused Inspector 只投影 primary selection 所需 fields。
- [ ] Editor 直接借用/共享 runtime artifact，删除第二套完整 DTO copy；Editor 可以保留 UI view model，但其 identity 必须引用 runtime entity/property path。
- [ ] Editor 02 event/message 只发送 generation/delta 通知，不在高频消息中深拷贝整份 inspection。
- [ ] 核对 `zircon_editor/src/scene/viewport/edit_mode_projection` 与 `ui/host/scene_inspection_publication.rs` 的既有 production owner，验证其 generation/delta/规模 contract；不得重复迁移或建立 test-only duplicate。
- [ ] 添加 1、1,000、100,000 entity 的结构/计数测试：stable generation full build/scan/clone = 0；单实体 transform 变化只失效对应 row/fields；删除 entity 清理 selection/inspection。

### 测试阶段：F4 Inspection Generation Gate

- [ ] 运行 Runtime world inspection unit/serialization/structure tests。
- [ ] 运行 Editor editing projection、Hierarchy/Inspector projection 和 generation invalidation tests。
- [ ] 运行 1/1,000/100,000 entity correctness/counter batch；时间 p95 只在已有 owner budget 中判定，不为 MVP新增随机器阈值。
- [ ] 运行 package-level Runtime→Editor boundary compile，确认 production build 实际包含 projection module。
- [ ] 失败时先修 Runtime artifact/generation，再修 Editor projection；不得恢复 test-only duplicate。

### 退出证据

- [ ] production editor 的既有 WorldInspection 消费路径通过 current-source package boundary 与规模验证。
- [ ] stable generation 不发生 full inspection rebuild/clone。
- [ ] focused selection 变化产生有界 delta，entity 删除不会留下 stale row/field。

## 5. M5.2 SceneMode/Selection 权威命令投影

### 目标

`CommandEvalCtx` 从真实 `SceneModeStack` 和 active-domain `SelectionModel` 原子生成；menu、keymap、command palette 和 UI binding 看到同一快照。

### 实现切片

- [ ] 核对既有 `SceneModeStack::project_command_eval_ctx` 权威投影，确认 `command_eval_projection.rs` 不从 chrome/inspector presence 重建 selection truth。
- [ ] 验证 `scene_mode` 来自 `SceneModeStack` active id、`selection_count` 来自 `SelectionModel` 当前 active domain 集合长度，并覆盖 generation 一致性。
- [ ] Edit/Play 切换原子替换 command eval snapshot；Play domain selection 不污染 Edit domain，退出 Play 恢复 Edit selection。
- [ ] Hierarchy、viewport、Inspector 共享 SelectionModel；删除任何 inspector presence→0/1 的最终 selection authority。
- [ ] mode push/pop、selection add/remove/clear、primary selection change 都使 command eval generation 精确失效一次。
- [ ] 扩充 `commands/when.rs` 测试：无选择、单选、多选、Edit、Play、mode change、stale selection removal 和 snapshot consistency。
- [ ] 确保 `SceneModeActive`、`SelectionNonEmpty` 对 menu、palette、keymap 和 binding dispatcher 得出相同 enablement。

### 测试阶段：F4 Command Context Gate

- [ ] 运行 scene modes、selection、commands when、menu/palette/keymap focused suites。
- [ ] 运行 Edit→Play→Edit 双域切换与 selection restore tests。
- [ ] 运行 Hierarchy selection 后 command eval snapshot 更新的 host integration test。
- [ ] 扫描 production 调用，确认 `with_scene_mode` 和真实 selection count 只有权威 projection owner，不从 view visibility 推导。

### 退出证据

- [ ] command eval projection fixed return 的 upward validation 已复验；world inspection open failure 仍独立保持 open。
- [ ] 所有命令消费面共享同一 generation snapshot。
- [ ] 多选和双域状态正确，inspector presence 不再是 selection truth。

## 6. M5.3 Hierarchy → Inspector → Transaction → Save

### 目标

真实 retained host binding 选择 cube 并提交 transform X，产生一条可 undo 的 transaction、dirty transition 和 F3 SaveProject commit。

### 实现切片

- [ ] Hierarchy rows 使用 Runtime inspection entity identity；点击/选择 binding 发出 `SelectionCommand::SelectSceneNode` 并更新 Edit-domain SelectionModel。
- [ ] primary selection 驱动 Inspector transform fields；字段值来自 current editor world，不是 template preview fixture。
- [ ] `TransformPositionXEdit` 只更新 draft/preview；`TransformPositionXCommit` 解析 typed scalar 并发出 editor operation/command。
- [ ] command 进入 `EditorTransactionEngine`，捕获 before/after transform、history context 和 selection snapshot；失败回滚 world/draft/selection。
- [ ] commit 后 world transform、Inspector row、viewport render packet 和 dirty generation 观察同一值。
- [ ] SaveProject 只有在 persistence completion 后清理 dirty；save failure 保持 dirty 和可重试 transaction history。
- [ ] 把现有 `componentized_workbench_transform_axis_edit_updates_field_and_row_preview` 扩展为真实 host/runtime harness，断言 world mutation 和 transaction record，而不只断言 control string。
- [ ] 增加 integration test 覆盖 Hierarchy 选中→Inspector commit→undo→redo→save completion。

### 测试阶段：F4 Authoring Integration Gate

- [ ] 运行 retained callback Hierarchy、workbench inspector property edit、binding dispatch、editing inspector/history/viewport suites。
- [ ] 断言完整链路中每个阶段使用同一 entity identity 和 transform delta。
- [ ] 运行 invalid numeric input、stale entity、read-only project、transaction failure 和 save failure 负例。
- [ ] 运行 undo/redo 后 Inspector/world/dirty 状态一致性。

### 退出证据

- [ ] Hierarchy selection 改变真实 SelectionModel。
- [ ] Inspector commit 改变真实 editor world 并产生一条 transaction。
- [ ] SaveProject completion 清理 dirty；失败不清理。

## 7. M5.4 当前产品 authoring 闭环

### 目标

`zircon_app` 级 integration harness 和 staged editor 共同证明 F4：打开 F3 项目、正常绑定选择/修改/保存、退出、重开并观察持久值。

### 实现切片

- [ ] 复验既有 `zircon_app/tests/editor_mvp_authoring.rs`，确认它通过 App editor composition 创建完整 retained host/runtime gateway，且不直接构造简化 EditorState。
- [ ] harness 从 F3 canonical project 打开 editor，读取 production WorldInspection，按 Hierarchy control/binding 选择 cube。
- [ ] `--project` 与 `--automation` 在读取或创建 composition 前各自解析一次为现有物理路径；
  `--location` 通过 ProjectAuthority 的同一 resolver 在创建前解析，automation report 和启动诊断只发布显示路径。
- [ ] harness 向 `TransformPositionXCommit` 注入与用户提交等价的正常 UI event，等待 operation/transaction/save completion。
- [ ] Drop 完整 app/host/gateway/session，创建第二份 App composition 并重新打开同一 project。
- [ ] 第二份 composition 从 production snapshot 和 parsed persisted document 双重比较 entity/transform/refs。
- [ ] staged editor 实际打开修改前和修改后项目，分别捕获窗口截图；截图必须显示同一 selected entity 和不同的 X value。
- [ ] 产品 trace 记录 binding id、command/operation id、transaction id、save generation 和 reopened generation；不得记录直接 world/file mutation。

### 测试阶段：F4 Product Authoring Gate

- [ ] 运行 `zircon_app` editor MVP authoring integration test，确认使用 current App composition。
- [ ] 启动 staged editor 打开修改前项目，捕获 Hierarchy/Inspector 初始状态。
- [ ] 通过正常 host binding 执行 fixed delta 和 SaveProject，等待明确 completion 后退出。
- [ ] 第二次启动 staged editor 打开同一项目，捕获 selected entity/Inspector X value。
- [ ] 运行 staged runtime，确认修改后 primitive 仍可见且 asset refs 有效。
- [ ] 对比 persisted document、editor snapshot、runtime render diagnostics 三份证据。

### 退出证据

- [ ] current executable composition 完成 F4 全链，不是 isolated UI preview test。
- [ ] 修改前后截图、transaction/save trace 和 persisted comparison 归属同一 project identity。
- [ ] 重开后 editor/runtime 均观察已修改 scene。

## 8. F4 阶段退出清单

- [ ] M5.1、M5.2、M5.3、M5.4 全部通过。
- [ ] production WorldInspection 已启用，stable generation 无 full rebuild/clone。
- [ ] SelectionModel/SceneModeStack 是 command eval 唯一权威。
- [ ] Hierarchy、Inspector、transaction、save/reopen 使用同一 entity identity。
- [ ] UI binding 修改真实 world，而非只修改 preview/control text。
- [ ] product integration、窗口证据和 persisted comparison 全部通过。
- [ ] plugin overlay、gizmo drag 和高级 authoring 继续延期。

## 状态与产出记录

每个里程碑测试通过后记录一次；实现切片不单独写入产出记录。

| 里程碑 | 范围 | 状态 | 完成日期 | 验证批次 / 残余风险 |
|---|---|---|---|---|

## Code Review 收敛结果（2026-08-01）

- 已把 M5.1 从“迁出 `cfg(test)`”改为验证既有 production WorldInspection projection 的 generation/delta/规模 contract。
- 已把 M5.2 从重新接线改为复验既有 `SceneModeStack::project_command_eval_ctx` 权威投影及 Edit/Play、多选、stale removal 覆盖；command eval failure 链接已更新为 canonical fixed return。
- 已把 M5.4 从新建 integration harness 改为复验既有 `editor_mvp_authoring.rs` 及其尚缺的截图、transaction/save trace 和双 composition 证据。
- 当前仍未运行 F4 批量门，因此 `status: blocked_by_f3` 和所有验收复选框保持不变；world inspection generation failure 继续 open。
