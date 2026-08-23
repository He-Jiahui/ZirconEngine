---
handoff_kind: failure
status: open
created_at: 2026-07-17
summary_slug: editor-selection-state-runtime-session-boundary
origin_plan: docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md
fixing_plan: docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md
origin_child_dir: docs/plans/zircon_editor/editor/01
fixing_child_dir: docs/plans/zircon_runtime/runtime/10
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/dynamic_api/session/state.rs
  - zircon_runtime/src/dynamic_api/session/construction.rs
  - zircon_runtime/src/dynamic_api/session/events.rs
  - zircon_runtime/src/tests/runtime_absorption/dynamic_api_session/event_split.rs
  - zircon_runtime/src/tests/runtime_absorption/performance_hotspots/scene_project_splits/dynamic_session_event.rs
tests:
  - cargo test -p zircon_runtime --lib dynamic_session_event_split --locked --jobs 1 -- --test-threads=1
  - cargo test -p zircon_runtime --lib dynamic_api --locked --jobs 1 -- --test-threads=1
---

# Runtime10: editor selection state remains inside runtime session

## 产出记录与时间

| 时间 | 来源门禁 | 状态 | 失败证据 | 修复责任 |
| --- | --- | --- | --- | --- |
| 2026-07-17 | Editor01 M2.3 `selected_node` 迁出 | `待修复（open）` | 当前源清点确认 `RuntimeDynamicSession.selected_node` 仍在 `state.rs`；`construction.rs` 仅在 session 创建时选择默认 cube，`events.rs` 仅在指针移动与滚轮事件后重新查询其 transform。全 runtime 无写入更新入口，也无选中高亮/overlay 消费，因此该字段既不是有效编辑器选择真相，又让高频指针路径重复持 world 读锁。 | Runtime10 删除 session authoring 命名状态与事件同步 helper；相机只保留 construction 阶段的中性初始 orbit target。Editor05 后续正式 HighlightSet 必须走独立 overlay 输入，不得回填本字段。 |

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md`
- 来源执行切片：Editor01 M2.3 `RuntimeDynamicSession.selected_node` 迁出
- 修复责任计划：`docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md`
- 交接原因：字段所有权、session 构造和 ABI event 路由都属于 Runtime10；Editor01 不应从 UI 调用点局部隐藏 runtime 内的重复 authoring 状态。

## 失败现象与复现证据

当前源静态清点：

```text
zircon_runtime/src/dynamic_api/session/state.rs: selected_node: Option<u64>
zircon_runtime/src/dynamic_api/session/construction.rs: let (selected_node, orbit_target) = ...
zircon_runtime/src/dynamic_api/session/events.rs: sync_orbit_target_from_selection()
```

字段只在 construction 写一次、事件路由的指针移动与滚轮分支读取；仓库中没有编辑器选中集写入该字段的入口。既有两个结构测试还把 `sync_orbit_target_from_selection` 当成必须保留的 event owner anchor，导致旧边界被测试固化。

## 最低共享层根因

早期 preview camera 把“默认 orbit anchor node”命名并存储成 `selected_node`，随后 Editor01 架构把 SelectionModel 定为编辑器唯一事实源，但 Runtime10 session 状态和结构测试未同步硬切。该字段实际不提供 authoring 选择能力，只造成重复状态语义和指针/滚轮热路径的 world 查询。

## 架构修复验收

- `RuntimeDynamicSession` 不再声明 `selected_node`，construction 不再把默认 cube id 写入 session；只计算一次中性 `orbit_target` 初始化 camera controller。
- event 路由删除 `sync_orbit_target_from_selection` 及其调用；mouse wheel 只推进 camera controller 自有 orbit 状态。
- 两个 event-split 结构测试反向守卫：`state.rs`、`construction.rs`、`events.rs` 均不得重新出现 `selected_node` 或 selection-sync helper。
- `cargo test -p zircon_runtime --lib dynamic_session_event_split --locked --jobs 1 -- --test-threads=1` 通过；上行 `dynamic_api` 门通过。
- Editor05 正式高亮输入后使用独立 overlay/HighlightSet 合同；不得用 `selected_node` 字段、别名或兼容 setter 恢复旧边界。

## 禁止临时方案

- 不得把字段改名为 editor selection、增加兼容 getter/setter、从 UI 每帧写入 runtime 私有字段或放宽结构测试。
- 不得把同一 node id 同时保存在 Editor SelectionModel 与 RuntimeDynamicSession。
- 不得删除相机初始 orbit target；本修复只删除错误的持久 authoring 状态，不破坏 preview 初始构图。

## 修复结果与回传

- 当前实现：`RuntimeDynamicSession.selected_node`、construction node-id 保存与 pointer/scroll selection-sync helper 已硬切；默认 cube translation 仍只在 construction 计算一次并初始化 camera orbit target。两个 event-split guard 同时拒绝字段别名、setter 与旧 helper，并正向锁定中性初始 orbit anchor。
- 聚焦验证：Windows managed reservation `dbad6f7125b04f5794d80f6f726400a3`、job `c30156f887b64a27aed8caa42caf7099`、run `88ca441d8d514fcebe1a17deb1a33a5b` 执行 `cargo test -p zircon_runtime --lib dynamic_session_event_split --locked --jobs 1 -- --test-threads=1`，2 passed / 0 failed / 8226 filtered，exit 0。
- 上行验证：reservation `0296a2f98e16478db2463461b53ae4ea`、job `388b5dfbf30245328db1a66d0bb88978`、run `b1a01bc55a2d478d9067ac56a91e489f` 执行 `dynamic_api` 112 项，94 passed / 8 failed / 10 ignored，exit 101。Runtime10 自有的 headless `pub(super)` slice 锚已按 current source 修复；M1.3 守卫已从 01–15 全目录聚合收紧到 Runtime10 canonical output record，并独立断言 current index 的 Runtime10 parent/FFI owner 路由，等待受管重跑。五项 Runtime15 parent/index owner/status 断链已完整交接为 [`../15/failure-2026-07-17-dynamic-api-owner-status-anchor-loss.md`](../15/failure-2026-07-17-dynamic-api-owner-status-anchor-loss.md)；Render01 F2 仍因 fixture asset id `edec23b9-2e44-d3af-dfdc-d1aa941e0614` 缺 artifact URI 失败，等待活跃 Render01 owner 接收对应 failure。
- 独立 current-source 首轮复审：Critical / Important / Minor = 0 / 2 / 0；两项 Important 分别为 Runtime10 状态守卫跨 owner 聚合导致假阳性，以及 Runtime15 failure 漏记 current-index 后续断言。当前已按 exact Runtime10 output/current-index source 收紧守卫，并补全 Runtime15 parent + index 五组断链诊断；等待整改后复审。
- 独立整改复审：Critical / Important / Minor = 0 / 1 / 0；Runtime10 source 收紧已关闭，剩余项是 Runtime15 vampire 组 parent 事实与短路范围记载不准。当前已改为精确缺失矩阵：前四组 parent 缺 3/5、4/5、4/5、5/5 且 index 均缺 5/5，vampire parent 已 5/5、index 缺 4/5；等待最终复审。
- 独立最终复审：Critical / Important / Minor = 0 / 0 / 0。复审确认 Runtime10 直接读取 exact canonical output record 与 current runtime index，M1.3、`| 10 |` parent route、FFI owner route 分别断言；Runtime15 精确矩阵与前四 parent 短路、vampire index 失败的执行顺序一致。
- Runtime10 owner gate：reservation `50af9eb455184495a3457e396da9916e`、job `efd9db13c38a47519f15755e4da225e5`、run `79c2e4d99dbc4538aedb1d9f674689c0` 执行 `cargo test -p zircon_runtime --lib runtime_10_ --locked --jobs 1 -- --test-threads=1`，12 passed / 1 failed / 0 ignored / 8221 filtered，exit 101。selection/event split、headless `pub(super)`、M1.3 exact source 均通过；唯一失败是 Runtime10 UI M2 guard 仍要求 Runtime05 closeout 复制已不存在的 Runtime10 状态锚。逐源扫描确认 canonical 证据只在 Runtime10 exact output record、runtime-interface convergence doc 与 architecture review；Runtime05 parent/child 当前均为 0 occurrence。守卫已硬切删除 Runtime05 第二事实源，保留上述三处 canonical/current 路由，等待聚焦重跑。
- Runtime10 owner-gate 修复独立复审：Critical / Important / Minor = 0 / 0 / 0。复审确认删除 Runtime05 mirror 属于 single-source hard cut；Runtime10 plan/numbered-child、exact output、current index parent/status route、runtime-interface convergence 与 architecture review 的 required anchors 和 forbidden completion claims 均保持完整。
- Runtime10 owner-gate 重跑：reservation `afc8bd5033704b9ab8c7d5fc75c6f794`、job `9e08dd5fa8a24bea8d3a060793c16f0f`、run `f1c4f4c29ad3490f959e1afaf6c0e328` 在进入测试前被当时的 Render01 current source E0425 阻断：`render_frame_with_pipeline.rs:318` 仍使用 `RenderGraphResourceAccessKind`，同一 owner 的 O(1) compiled-pipeline metadata 改造已删除其 import。该历史 blocker 后续由 [Render01 compiled-pipeline frame-derived recomputation fixed return](../../../performance/01/fixed-2026-07-17-compiled-pipeline-frame-derived-recomputation.md) 关闭；本轮 exit 101 仍只证明当时的 compile blocker，不能覆盖前轮 12/13 或声明 Runtime10 tests 通过。
- 跨消费者复现：释放已知会失败且未绑定 job 的 Editor01 队首预约后，Frameworks05 受管 job `fb0000a6ee3b406a81f0917ef3d46f1d`、run `7ea84557bf824db08004d949e1d18c05` 同样在编译 `zircon_runtime` 时于 `render_frame_with_pipeline.rs:318` 得到 E0425，rustc 明确建议导入 `crate::render_graph::RenderGraphResourceAccessKind`，exit 101。该历史证据确认阻塞属于共享 Render01 current source，不是 Runtime10 test filter 或 Editor01 lifecycle 夹具；Render01 fixed return 不替代 Runtime10 自身仍待完成的上行重跑。

Open state: `实现与聚焦门通过，上行跨 owner 失败待回传`; Runtime10 failure return and fixed promotion are not yet claimed.
