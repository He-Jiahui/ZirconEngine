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
  - docs/plans/zircon_editor/editor/05/failure-2026-07-12-command-eval-scene-mode-selection-projection.md
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
- [ ] 两个 canonical open failure 已进入 fixing priority：command eval projection、world inspection generation projection。
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
- 不要求所有 100k 性能目标；但解除当前 `cfg(test)` 所需的 shared generation、稳定帧零全量重建和声明的规模正确性必须完成。
- 不把 OS 鼠标坐标自动化作为唯一证据；产品 integration harness 可以注入正常 UI binding，但不得跳过 binding/command/transaction。

## 4. M5.1 World inspection production generation

### 目标

Runtime 发布可供生产 Editor 消费的 immutable world inspection generation；Hierarchy/Inspector 对稳定 generation 不重建全量 DTO，当前 `cfg(test)` 限制在规模、Cargo 和 F4 trace 通过后解除。

### 实现切片

- [ ] 以 world inspection open failure 为权威，定义 generation identity、hierarchy rows、focused fields、changed rows/fields 和 removal 的 neutral contract。
- [ ] Runtime world/scene 变更只推进一次 inspection generation；stable frame 复用同一 `Arc` artifact。
- [ ] hierarchy 构建避免 focus 第二遍、重复 parent map、重复全表 sort 和 field-name clone；focused Inspector 只投影 primary selection 所需 fields。
- [ ] Editor 直接借用/共享 runtime artifact，删除第二套完整 DTO copy；Editor 可以保留 UI view model，但其 identity 必须引用 runtime entity/property path。
- [ ] Editor 02 event/message 只发送 generation/delta 通知，不在高频消息中深拷贝整份 inspection。
- [ ] 把 `zircon_editor/src/scene/viewport/mod.rs` 和 controller 中的 edit-mode projection 从 `cfg(test)` 迁入 production owner，仅在 contract/scale tests 与 F4 trace 均就绪后执行 hard cut。
- [ ] 添加 1、1,000、100,000 entity 的结构/计数测试：stable generation full build/scan/clone = 0；单实体 transform 变化只失效对应 row/fields；删除 entity 清理 selection/inspection。

### 测试阶段：F4 Inspection Generation Gate

- [ ] 运行 Runtime world inspection unit/serialization/structure tests。
- [ ] 运行 Editor editing projection、Hierarchy/Inspector projection 和 generation invalidation tests。
- [ ] 运行 1/1,000/100,000 entity correctness/counter batch；时间 p95 只在已有 owner budget 中判定，不为 MVP新增随机器阈值。
- [ ] 运行 package-level Runtime→Editor boundary compile，确认 production build 实际包含 projection module。
- [ ] 失败时先修 Runtime artifact/generation，再修 Editor projection；不得恢复 test-only duplicate。

### 退出证据

- [ ] production editor 能消费 WorldInspection，不再依赖 `cfg(test)`。
- [ ] stable generation 不发生 full inspection rebuild/clone。
- [ ] focused selection 变化产生有界 delta，entity 删除不会留下 stale row/field。

## 5. M5.2 SceneMode/Selection 权威命令投影

### 目标

`CommandEvalCtx` 从真实 `SceneModeStack` 和 active-domain `SelectionModel` 原子生成；menu、keymap、command palette 和 UI binding 看到同一快照。

### 实现切片

- [ ] 修改 `command_eval_projection.rs` 的输入，不再仅接收 `EditorChromeSnapshot`/inspector presence；传入 scene mode 与 selection snapshot/generation。
- [ ] `scene_mode` 来自 `SceneModeStack` active id；`selection_count` 来自 `SelectionModel` 当前 active domain 的集合长度。
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

- [ ] SceneMode/Selection open failure 完成修复和 upward validation。
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

- [ ] 新建 `zircon_app/tests/editor_mvp_authoring.rs`，通过 App editor composition 创建完整 retained host/runtime gateway，不直接构造简化 EditorState。
- [ ] harness 从 F3 canonical project 打开 editor，读取 production WorldInspection，按 Hierarchy control/binding 选择 cube。
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

## Code Review 建议 (2026-07-30)

### 与代码现状不符，需修订

- M5.1 前提「world inspection consumer 仍为 `cfg(test)`」（也见 index §6 风险表 F4 行）已过时。当前 Editor 侧 WorldInspection 消费者是生产模块，非 test-only：`zircon_editor/src/scene/viewport/mod.rs:4` 无条件 `mod edit_mode_projection`，其 `edit_mode_projection/build.rs:1-234` 直接消费 `WorldInspectionField/HierarchyRow/Summary`；`zircon_editor/src/ui/host/mod.rs:47` 无条件 `mod scene_inspection_publication`，`scene_inspection_publication.rs:20` 持有 `Arc<WorldInspectionArtifact>`。建议把「从 `cfg(test)` 迁入 production owner」的切片改为「验证既有生产投影是否满足 generation/delta/规模不重建 contract」。
- M5.2 前提「command eval 仍以 inspector presence 模拟 selection」（index §6 风险表 F4 行同）已不成立。`zircon_editor/src/ui/host/command_eval_projection.rs:11-35` 的 `command_eval_ctx_from_chrome` 已不再从 inspector presence 推导 selection；真实 selection/scene_mode 投影在 `zircon_editor/src/scene/modes/scene_mode_stack.rs:34-42`：`project_command_eval_ctx` 用 `self.active_mode_id()` 和 `selection.active_items().len()` 填充 `with_scene_mode` / `with_selection_count`，经 `scene_viewport_controller_accessors.rs:52-56` → `editor_state_viewport.rs:47-49` 串到 host。`when.rs:62-65` 的 `SceneModeActive`/`SelectionNonEmpty` 已消费这些字段。建议 M5.2 改为「核对 Edit/Play 双域切换、多选和 stale removal 的既有覆盖」，而非「修改 `command_eval_projection.rs` 的输入不再仅接收 inspector presence」。
- M5.4「新建 `zircon_app/tests/editor_mvp_authoring.rs`」——该文件已存在（`zircon_app/tests/editor_mvp_authoring.rs`，335 行），且 `f4_project_authoring_survives_full_application_restart`（`:20`）已通过 `EditorApplicationComposition`、`ProjectAuthority`、`EditorUiBinding`/`SelectionCommand`、`EditorProjectDocument` 实现「App composition 打开→选择→binding 提交→save→重启比较」链路。建议改为「验证既有 integration test 是否覆盖 M5.4 全部断言（截图、transaction/save trace、双 composition 比较）」。

### 验证缺口

- 既然 M5.1/M5.2/M5.4 的核心投影与 integration harness 已在源码存在，应补一条明确任务：以 `milestone-validation-policy.md` 批量门跑 `zircon_editor` 命令/editing/host focused suites 与 `zircon_app` `editor_mvp_authoring` integration test，用当前源码通过证据替换这些过时的「待实现」描述，避免执行者重复搭建已存在的能力。
