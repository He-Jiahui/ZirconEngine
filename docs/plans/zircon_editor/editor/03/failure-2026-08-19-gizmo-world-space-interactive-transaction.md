---
handoff_kind: failure
status: open
created_at: 2026-08-19
summary_slug: gizmo-world-space-interactive-transaction
origin_plan: docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md
fixing_plan: docs/plans/zircon_editor/editor/03-command-transaction-and-undo.md
origin_child_dir: docs/plans/zircon_editor/editor/05
fixing_child_dir: docs/plans/zircon_editor/editor/03
plan_link_mode: child_record_only
related_code:
  - zircon_editor/src/core/editing/interactive_transform/session.rs
  - zircon_editor/src/core/editing/command/batch_transform.rs
  - zircon_editor/src/core/editing/journal_codecs/scene.rs
  - zircon_editor/src/scene/viewport/handles/helpers/selection.rs
  - zircon_editor/src/scene/viewport/handles/transform_handle_drag_session.rs
  - zircon_editor/src/scene/viewport/handles/move_handle_tool_behavior.rs
  - zircon_editor/src/scene/viewport/handles/rotate_handle_tool_behavior.rs
  - zircon_editor/src/scene/viewport/handles/scale_handle_tool_behavior.rs
  - zircon_editor/src/scene/viewport/controller/scene_viewport_controller_handle_input.rs
  - zircon_editor/src/scene/viewport/interaction/viewport_feedback.rs
  - zircon_editor/src/ui/workbench/state/editor_state_viewport.rs
  - zircon_runtime/src/scene/world/query.rs
  - zircon_runtime_interface/src/math.rs
tests:
  - tools/tests/test_editor03_world_space_interactive_transaction_contract.py
  - zircon_editor/src/tests/editing/interactive_transform.rs
  - zircon_editor/src/tests/editing/transaction_engine/journal_scene_commands.rs
  - zircon_editor/src/tests/editing/transaction_engine/journal_scene_replay.rs
  - rotated non-uniform and negative-scale parent matrix cases produce correct world-space handle basis and local writeback, or a typed non-representable-transform rejection
  - selection containing parent and child transforms only root selections once and commits one typed batch command
  - move rotate scale preview 100 pointer updates commit one history entry and cancel/capture-loss restores every affected root
  - 10k selected roots and deep hierarchy profile records begin/update/commit time, allocations, and no full-scene scan
---

# Editor05 -> Editor03: Gizmo 缺少 world-space 批量交互事务

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md`
- 来源执行切片：M2 事务化拖拽与多选操纵
- 修复责任计划：`docs/plans/zircon_editor/editor/03-command-transaction-and-undo.md`
- 交接原因：Editor05 的 handles 是单节点局部 Transform 计算器；修复 parent-space 与 multi-selection 正确性必须先由 Editor03 提供长生命周期、批量原子且可取消的 interactive transaction authority。不能在 controller 或 handle 行为层再建立平行事务机制。

## 失败现象与复现证据

`selected_basis` 与 `begin_transform_session` 都从 `SceneNode.transform` 读取局部
Transform；`HandleBasis.origin`、投影拖拽原点和 Move/Rotate/Scale 的更新也全部使用它。
但同一 editor 的相机、选中定位和 render packet 已使用 `Scene::world_transform`，runtime
还公开了 `world_matrix` 与 `parent_of`。带平移、旋转、缩放父节点时，gizmo 因而画在错误的
世界位置，Global Move 直接把 world 轴增量加到 local translation，Rotate/Scale 也没有经
parent world inverse 写回。

更深一层，`TransformHandleDragSession` 仅保存一个 `node_id` 和一个局部
`initial_transform`，`HandleTool::update_drag` 只能返回一个局部 `Transform`，
`ViewportTransformPreview` 与 `GizmoTransactionCapture` 也都是单节点形状。当前每次 preview
直接 `Scene::update_transform`，release 时才补建 already-applied command。它无法表达：多选根
去重、共同 pivot、world delta、每个对象的 before/after，以及一次性 rollback/commit。

矩阵数据模型还有一个不得掩盖的限制：runtime 当前只存 TRS。对非均匀或负缩放父节点应用任意
world rotation/scale 时，`parent_world_inverse * world_delta * child_world` 可能包含 shear；
单纯 `Mat4::to_scale_rotation_translation` 会投影并丢失该信息。没有残差检查就把分解结果写回，
不属于数学正确的 parent inverse 修复。

## 最低共享层根因

Editor03 现有 `GizmoTransactionCapture` 是 workbench 私有的单对象 pre/post capture，不是
transaction engine 的 interactive session。Editor05 也没有 selection transform graph 或
world/local 快照 DTO，所以无法在不越过 owner 的情况下构造批量、可回滚的变换。

这与 Unreal 的边界一致：`FEditorModeTools::InputDelta` 只向 active mode 分发输入，
`FEdMode::InputDelta` 再交给当前 tool；工具交互和事务不是 controller 对 Scene 的即时写入。
Zircon 应吸收其 mode -> tool -> interactive transaction 责任分离，而不复制历史兼容接口。

## 架构修复验收

- Editor03 提供唯一的 `InteractiveEditSession` 或等价 transaction-engine owner：begin 时冻结
  document/world generation、tool kind、axis/plane、space、snap、pivot、selection roots 和完整
  before snapshots；preview、commit、cancel、capture loss 与 failure rollback 都经过这一 owner。
  autosave/observer 只能按明确 preview generation 或最后 committed generation 读取，不允许
  workbench 私有的 direct `Scene::update_transform` 旁路。
- begin 阶段从 active selection 计算 top-level affected roots，排除被另一个选择祖先包含的节点，
  在任何写入前验证 target 存在、locked/hidden policy、world/parent matrix 有效性和 parent inverse
  可用性。校验失败必须 typed reject，绝不部分 preview。
- 对每个 root 冻结 local transform、world matrix、optional parent world inverse。handle basis 与
  drag delta 在 world space 计算；局部变换通过 `parent_world_inverse * desired_world_matrix`
  写回。Global move/rotate/scale 的 pivot 变换必须应用于每个冻结 world matrix，Local mode 使用
  同一明确的 local-space 规则，不能混合轴空间和写回空间。
- TRS 仍是数据模型时，decompose 后必须以重组矩阵残差和 finite/invertible 检查确认可表示。若
  non-uniform/negative-scale parent 产生不可表示 shear，interactive session 返回 typed
  `NonRepresentableTransform` 并完整 rollback；不得静默丢失 shear。若产品要求该组合可编辑，先
  把 runtime transform contract 升级为可保真 affine representation，再开放该操作。
- preview 载荷升级为 selection-root keyed 批量变换，而不是单个
  `ViewportTransformPreview { node_id, transform }`；commit 只产生一个 typed batch command，
  label/journal metadata 区分 Move、Rotate、Scale。Editor05 的 HandleTool 只负责生成冻结会话
  的 world delta，不拥有事务或直接写 world。
- begin 时间复杂度限于 selected roots 与其祖先链，update/preview/cancel/commit 均为 `O(k)`
  affected roots，不扫描 `Scene::nodes()`、不 clone 完整 scene。实现前后用 10k roots、深层
  hierarchy 的 Windows-native managed profile 记录 p50/p95、allocation、peak memory 和 frame
  budget；Cargo 不可用时不得伪造性能数据或宣称优化有效。

## 禁止临时方案

- 不得只将 `node.transform` 替换为 `world_transform`，再把 world translation 直接写回 local。
- 不得在 handle/controller/workbench 旁路 `InteractiveEditSession` 建立第二个 GizmoDragState、
  one-off batch 或 already-applied command。
- 不得对父子同时选中循环更新每个节点、接受部分 target、用默认 pivot 或 primary-only 回退。
- 不得通过 `to_scale_rotation_translation` 静默吞掉 shear，或用 epsilon/默认 identity inverse
  掩盖不可逆矩阵。

## 修复结果与回传

Open state: `source-implemented / static-contract-green / managed-rust-and-profile-pending`.
Editor03 已落地唯一的 world-space interactive batch authority，并完成 Editor05 viewport 到该
authority 的硬切；当前仍不宣称 fixing plan fixed，因为受管 Windows Rust 行为测试、10k roots /
deep hierarchy profile、allocation 与 frame-budget 数据尚未执行。

## 产出记录与时间

| 时间 | 状态 | 完成项目 | 证据与后续 |
|---|---|---|---|
| 2026-08-19 | `open / Editor03 interactive batch authority required` | 完成 Editor05 handles、preview adapter、workbench transaction capture、runtime world transform query 与 Unreal `FEditorModeTools`/`FEdMode` 输入分发的静态调用图复核；确认 P1-12、P1-13、P1-15 共享同一根因。 | 当前无运行时性能数据：受管 focused Cargo 在依赖解析前缺少 `image` 缓存而退出。Editor03 先交付 interactive batch owner 与数学表示策略，Editor05 再接 world-space HandleTool，并以完整 hierarchy/multi-selection 回归和 profile 验收。 |
| 2026-08-24 | `open / architecture-and-performance-review-complete` | 在 [Gizmo interactive transaction architecture review](../05/2026-08-24-gizmo-interactive-transaction-architecture-review.md) 固化 Editor03 single-owner session、root filter、world-to-local writeback、TRS residual rejection 和 profile matrix。 | 这是静态结构与参考实现复核，不是性能验收；共享 worktree 的 transaction/viewport owners 正在变更，尚未对其重叠写入。下一步必须从 Editor03 interactive session 落码。 |
| 2026-08-28 00:35 +08:00 | `open / source-implemented / static-contract-green / managed-rust-and-profile-pending` | 完成 `InteractiveTransformSession` 单 owner：冻结 `DocumentId`、world generation、tool/axis/space/snap、primary pivot、去重后的 selection roots、local/world/parent-inverse 快照；Static target 在 preview 前 typed reject，world request 校验 document 与 primary root；preview 先对全部目标完成 finite/TRS 重组残差校验再执行 `O(k)` 写入，失败回滚已应用前缀，cancel 失败补偿恢复上一 preview。完成 `ViewportTransformRequest { primary, target_world }` 硬切，controller/workbench 不再直接 `Scene::update_transform`；release 只提交一个 generation/after-snapshot 校验的 `BatchTransformCommand`，并注册 versioned journal codec/replay。事务预览后从权威 `Scene` 重同步 `primary_root` world pivot/orbit-controller target；selection/Inspector 的 local translation 不再覆盖 world orbit target。活动相机 cache 只在权威 camera identity/world transform 实际变化时同步：interactive preview、普通 scene command 及 Undo/Redo 均在事务前后比较活动相机 authority，覆盖直接编辑活动相机、编辑其祖先或次选根带动相机，同时无关对象编辑保留已导航的 editor camera。新增独立 `camera_authority` 行为回归源码，避免继续膨胀 913 行的 viewport 测试 owner。产品回归已从内部事务调用迁移到真实 `ActivateSceneMode -> LeftPressed -> PointerMoved -> LeftReleased/CancelInteraction` 输入链，100 preview 保持一个 command/history record。 | 非 Cargo 证据：`rustfmt --check` 通过；`test_editor03_world_space_interactive_transaction_contract.py` 3/3、`test_editor03_scene_transaction_hardcut_contract.py` 13/13；静态契约已锁定活动相机直接变换、父层级 Undo/Redo 和无关对象保留导航视角三条行为测试入口；生产旧 `GizmoTransactionCapture` / `ViewportTransformPreview` / begin-record-finish-cancel 旧 API 命中 0，workbench/controller direct transform write 命中 0；核心 session/command 为 363/244 行。按当前目标“不因验收队列阻塞”未启动新的 Cargo；新增 Rust 行为测试执行、batch journal replay、Windows managed 10k roots/deep hierarchy p50/p95、allocation、peak memory、frame budget 与功耗经验值比较仍待独立验收，故不回传 fixed、不提交 milestone commit、不发送企微。 |
