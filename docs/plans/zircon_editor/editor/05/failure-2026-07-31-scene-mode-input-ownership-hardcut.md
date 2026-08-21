---
handoff_kind: failure
status: open
created_at: 2026-07-31
summary_slug: scene-mode-input-ownership-hardcut
origin_plan: docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md
fixing_plan: docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md
origin_child_dir: docs/plans/zircon_editor/editor/05
fixing_child_dir: docs/plans/zircon_editor/editor/05
failure_scope: local
related_code:
  - zircon_editor/src/scene/modes/builtin_scene_mode.rs
  - zircon_editor/src/scene/modes/builtin_scene_mode_registry.rs
  - zircon_editor/src/scene/modes/scene_mode_ctx.rs
  - zircon_editor/src/scene/modes/scene_mode_stack.rs
  - zircon_editor/src/scene/viewport/settings.rs
  - zircon_editor/src/scene/viewport/controller/scene_viewport_controller_accessors.rs
  - zircon_editor/src/scene/viewport/controller/scene_viewport_controller_handle_input.rs
  - zircon_editor/src/scene/viewport/controller/scene_viewport_controller_handle_interaction.rs
  - zircon_editor/src/scene/viewport/controller/scene_viewport_controller_pointer_route.rs
tests:
  - zircon_editor scene-mode input routing and lifecycle tests
  - zircon_editor viewport selection and transform-mode behavior tests
  - source guard rejecting builtin no-op mode implementations and controller tool-enum dispatch
---

# Editor 05: SceneMode 栈未取得 viewport 输入所有权

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md`
- 来源执行切片：M1.1 模式栈与 SelectionModel
- 修复责任计划：`docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md`
- 交接原因：模式栈已经接在输入入口前，但 `scene.select` 与 `scene.transform` 没有行为；旧 controller 继续持有选择、handle 拖拽和 `SceneViewportTool` 分派。

## 失败现象与复现证据

检测快照中，源码已具备通用 `SceneModeStack::replace_base`、pre-dispatch 和一个自定义 consuming-mode 回归测试，但 `BuiltinSceneMode::enter`、`exit` 和 `build_overlay` 为空，`handle_input` 对所有事件返回 `PassThrough`。`SceneModeCtx` 只暴露 selection 与 settings，无法访问受限的场景操作、pointer route、drag 生命周期或反馈输出。

在该快照中，`SceneViewportController::handle_input` 的 `dispatch_scene_mode_input` 永远不会让内建模式消费事件；随后同一函数仍直接路由 pointer、选择、handle drag 和相机操作，且 `handle_left_pressed` 依赖 `self.active_tool() != SceneViewportTool::Drag` 决定 transform 行为。`activate_tool` 还把 `SceneViewportSettings.tool` 写为第二个 base-mode 事实源。这与 M1 的模式栈作为唯一输入 owner 以及“旧枚举开关分派删除”的硬切要求不符。

## 最低共享层根因

模式 trait 只有生命周期外壳，没有面向 viewport 的受限能力上下文。controller 将模式栈作为自身字段时又直接拥有其余交互状态，导致模式实现无法在不借用冲突或直接暴露 controller 内部状态的情况下执行选择、路由、拖拽和反馈。

## 架构修复验收

- 将内建行为拆为实际的 Select 与 Transform mode owner；不保留 `BuiltinSceneMode` 空实现作为兼容基类或后备路径。
- `SceneModeCtx` 只暴露所需的受限 viewport 服务：有效 scene read/write、pointer route、selection mutation、drag 会话、settings、camera/viewport 快照和 feedback sink；不得将完整 controller 或全局 mutable scene 状态泄漏给 mode。
- controller 在分派前暂时移出 stack 或采用等价的无别名调度，使 mode 可以消费输入并写入明确 effect；resize 与通用相机导航保留为独立、明确的系统级输入 owner，而非 mode 失败后的隐式 fallback。
- Select mode 负责点击和框选的 active-domain `SelectionModel` 更新；Transform mode 负责 handle 命中、拖拽生命周期和 M2 transaction adapter。M2 未就绪的事务能力必须 typed reject，不能回退为 controller 直接 `Scene::update_transform` 写入。
- 删除 `SceneViewportSettings.tool`、controller 的 `SceneViewportTool` 条件分派、`active_tool`/`activate_tool` 旧枚举 API 和 PassThrough-only 内建 mode。UI/command 只通过 descriptor-backed `SceneModeId` 请求切换，overlay 只由 active mode 及 overlay stack 贡献。
- 测试覆盖 mode enter/exit、overlay LIFO、Consumed/PassThrough 路由、选择语义、transform typed reject/transaction 提交，以及源码负向守卫。

## 禁止临时方案

- 禁止保留空 `BuiltinSceneMode`、controller 内的模式后备逻辑、枚举分支、双重 pointer route 或 test-only mode hook。
- 禁止为借用问题暴露 `SceneViewportController` 全量可变引用、全局状态、第二 selection truth 或以 `unsafe` 绕过所有权。
- 禁止在 Transform mode 未接通事务前继续直接写 world，或把 `PassThrough` 当作内建行为已完成。

## 修复结果与回传

Open state: `实现完成 / 二次审查与 accepted closeout open`; typed mode-effect、旧 viewport-tool 协议硬切、modifier-aware 点击/框选和基础 M2 preview adapter 已落地。handle 计算只发布 `ViewportTransformPreview`，`EditorState` 在 Editor03 gizmo transaction lane 中 apply/record/finish，模式切换先取消活动 transaction，controller 生产路径不含 `Scene::update_transform`。生产扩展现在只接受可执行 `SceneModeRegistration`，并把 mode/provider overlay 合并到 render/pointer 共用 interaction extract。多选 pivot 与 Esc cancel 属于 M2 后续；受管 Cargo/UI 门和本轮二次独立复审未完成前不能返回 fixed。M2 的 accepted 上游证据仍由 [Editor03 gizmo transaction handoff](../03/failure-2026-07-19-gizmo-transaction-capture-private-interface.md) 跟踪；其 failure 保持 `open` 不影响本计划继续前向实现，但会延迟 accepted closeout。

## 产出记录与时间

| 日期 | 项目 | 状态 | 证据 |
| --- | --- | --- | --- |
| 2026-07-31 | M1.1 SceneMode input ownership hard-cut | open | 检测快照：通用 `replace_base`、pre-dispatch 和 custom consuming-mode 测试已在源码；但 `BuiltinSceneMode` 对所有事件 PassThrough，`SceneModeCtx` 不含交互服务，controller 仍直接处理选择/drag 并按 `SceneViewportTool` 分支。 |
| 2026-07-31 | M1.1 typed mode-effect 输入切片 | in_progress | Select/Transform mode 已改为消费 primary pointer 并发布受限单值 inline effect；PassThrough mode 产生的 effect 在继续分派前丢弃。controller pointer 路径已删除 `SceneViewportTool`/`active_tool` 条件，mode `enter` 在目标 transform settings 下执行；transform 输入发布 preview request，不直接写 world。前两轮独立审查提出 effect 生命周期/settings 切换和 pointer 热路径分配问题，均已前向修复。 |
| 2026-08-01 | M1.2 SceneViewportTool hard cut + review repair | in_progress | `SceneModeActivation` 与 transform-only `TransformHandleKind` 已替换旧 tool enum，UI/event/binding/template 统一为 `ActivateSceneMode`；`SceneViewportTool`、`SetTool`、`settings.tool` scoped source scan 为 0。独立复审首轮 `Critical/Important/Minor=4/4/0`：registry Result、非 Copy toolbar、codec 漏分支、旧测试、模板断链、base/active 双事实、保留 ID 冒充和 custom-mode 热路径 clone 均已前向整改；整改后复审与 managed Cargo/UI 尚待一次性受管触发。 |
| 2026-08-01 | M2.1 transaction preview adapter | in_progress | handle drag 只生成 `ViewportTransformPreview { node_id, transform }`；workbench 在 `begin_gizmo_transaction` 后 apply preview、record step，并在 release 时 finish，失败走既有 rollback/reset。controller 源码负向守卫拒绝直接 `scene.update_transform`。单节点 Move/Rotate/Scale transaction 行为测试已恢复为新 adapter 路径；多选 pivot、Esc cancel、Editor03 accepted gate 与 managed Cargo 仍 open。 |
| 2026-08-01 | M1.2 modifier selection + box selection | implementation_complete / managed_validation_pending | 原生 pointer modifier 已贯穿 callback/host/event/command/mode effect；Shift=`Extend`、Ctrl=`Toggle`、默认=`Replace`。拖拽超过阈值后通过 shared interaction extract 的 screen-rect query 合并 renderable 与 scene-gizmo pick shape 并稳定去重，不新增 `Scene::nodes()` 全扫；SelectionModel 三种 mutation 与 callback 路由测试已落盘。 |
| 2026-08-01 | M1.3 executable extension registry + overlay lifecycle | implementation_complete / rereview_pending | 扩展 hard-cut 为 `register_scene_mode(SceneModeRegistration)` 唯一入口，descriptor 从 registration 派生；host 在写入前原子校验 duplicate/factory id，controller 安装后支持 custom base activate 与 overlay push/pop/update/shutdown。mode/provider gizmo 在 interaction cache 内合并，render/pointer 同消费一份 `Arc`；activate/provider lifecycle 显式失效缓存。 |
| 2026-08-01 | 独立复审整改批次 2 | implementation_complete / rereview_pending | 前次复审 `Critical/Important/Minor=0/4/2` 的 6 项均已前向整改：生产 custom factory 安装与生命周期、模式切换取消 gizmo transaction、overlay 统一拾取快照、custom id 全链保真、默认 Select、删除 orphan controller clone。待同一 reviewer 二次审查后更新最终分级。 |
| 2026-08-01 | 独立复审整改批次 3 | implementation_complete / rereview_pending | 本轮复审发现旧 descriptor 编译残留、cache 测试签名、factory 双调用、input overlay invalidation、默认模式断言、provider/consumer 半提交、插件 callback panic 与无效 overlay owner 风险；均已前向修复并补 host/stack/pointer 生命周期回归。Editor02 consumer 原子性另记对应 failure；最终分级与 managed gate 待回传。 |
| 2026-08-19 | M2.1 active-domain multi-selection frame | implementation_complete / managed_validation_pending | `frame_selection` 已从 primary-only hard-cut 为 active selection AABB；单次遍历有效 world position，中心成为 orbit target，半径驱动 perspective/orthographic 最小 framing 范围，无场景全扫或临时集合。新增两对象且 primary 固定于左侧的回归断言，同时验证 FOV 距离下限。`rustfmt --check`、`git diff --check` 与 `active_primary` 负向搜索通过；Cargo focused gate 仍在依赖解析前因缺少 `image` 缓存退出，未标记通过。 |
| 2026-08-19 | M2 parent-space gizmo + multi-selection transaction | `open / Editor03 cross-plan authority required` | 完成 handle、preview、workbench capture、runtime hierarchy transform 与 Unreal mode/tool routing 的结构复核。当前数据通路只表达一个局部 Transform；非均匀/负缩放父节点还可能生成 TRS 不可表示的 shear。已交接 [`gizmo-world-space-interactive-transaction`](failure-2026-08-19-gizmo-world-space-interactive-transaction.md)：Editor03 必须先提供冻结 root snapshot、world delta、批量 preview/commit/cancel 和 typed non-representable rejection，Editor05 不会以局部坐标补丁伪造完成。 |
| 2026-08-19 | Editor05 focused Cargo 上行门 | `managed_validation_pending / no Cargo process started` | `rustfmt --check`、`git diff --check` 与 HighlightSet/multi-selection static contract 全部通过。受管 test job `fe3ab84c04ee4d5b9c4c8fc8ef80a245` 仅创建预约；首次被 PowerShell 参数解析拒绝，随后一次 health preflight 超时，最后一次把 `+1.94.1` 错置为 runner 的可执行文件并在 `CreateProcessW` 前失败。协调器实现已复核为直接执行 argv，正确重跑形态为 `cargo +1.94.1 test -p zircon_editor --lib viewport --locked --jobs 1 -- --test-threads=1`。`run-status` 确认无 managed run，最终 job 已 released、live PID 为空、`E:\cargo-targets\zircon-engine\pool\editor05-highlight-multiselect-20260819` 未用于编译。不得将此记录为测试通过或代码 exit failure。 |
| 2026-08-19 | Frame Selection 算法与复杂度复核 | `review_complete / spatial-bounds dependency open` | 对 runtime `Scene::world_transform` 的两条读取路径完成复核：derived state 稳定时直接读取缓存 `WorldMatrix`，本次 selection AABB 为 `O(k)`；hierarchy/transform dirty 时每个目标都走祖先链 `Vec + HashSet` 重建，成本为 `O(k*d)` 且有临时分配。当前实现未扫描整场景，但只能覆盖选中节点的位置，不能宣称 mesh bounds framing 完成。 | 后续性能实验必须在 Windows managed profile 中记录 1/1k/10k selected、depth 1/64/5k 的 stable/dirty p50/p95、allocation 与 frame budget；Render04 返回同 generation selectable bounds 后，Frame Selection 改消费该权威产品并覆盖无 bounds/极端 scale/near-far clipping。无实测前不宣称性能数据、功耗或与 Unreal 的比较结论。 |
| 2026-08-19 | M1.3 native focus-loss interaction cancellation | `implementation_complete / focused_managed_validation_passed` | `WindowEvent::Focused(false)` 通过 host callback 统一转为 `EditorViewportEvent::CancelInteraction`；`PointerLeft` 保持非取消路径。`EditorState` 先回滚 active gizmo transaction，再要求 controller 清除 Orbit/Pan/PrimarySelection/Handle drag；Handle 会结束自身 session，取消不重置相机也不直接写 scene。新增 controller、workbench state 与事件路由回归。受管命令 `cargo +1.94.1 test -p zircon_editor --lib viewport --locked --jobs 1 -- --test-threads=1` 终态两次均由 coordinator 确认 `exit_code=0`：job `80604552dfee4496b7f48ba418293a67` 以及改名后确保事件路由回归也命中 filter 的 job `8247ccdb7ee14635b5903b5dfedc0558`；二者均已 released，产物均在 `E:\cargo-targets\zircon-engine\pool\editor05-focus-cancel*`。完整 typed pointer-id、touch/capture-loss carrier 仍为 P1-17 后续，不以本切片宣称输入协议完整；本改动没有热路径算法替换，未采集或声称性能/功耗数据。 |
