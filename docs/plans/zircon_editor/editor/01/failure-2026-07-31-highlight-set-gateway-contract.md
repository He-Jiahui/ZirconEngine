---
handoff_kind: failure
status: open
created_at: 2026-07-31
summary_slug: highlight-set-gateway-contract
origin_plan: docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md
fixing_plan: docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md
origin_child_dir: docs/plans/zircon_editor/editor/05
fixing_child_dir: docs/plans/zircon_editor/editor/01
plan_link_mode: child_record_only
related_code:
  - zircon_editor/src/core/gateway/contract.rs
  - zircon_editor/src/core/gateway/handle.rs
  - zircon_editor/src/core/gateway/in_process.rs
  - zircon_editor/src/core/gateway/session.rs
  - zircon_editor/src/core/gateway/capabilities.rs
  - zircon_runtime_interface/src/runtime_api/abi/api_table.rs
  - zircon_runtime/src/dynamic_api/session.rs
  - zircon_runtime/src/core/framework/render/overlay.rs
  - zircon_editor/src/scene/viewport/controller/scene_viewport_controller_build_render_snapshot.rs
  - zircon_editor/src/scene/viewport/render_packet.rs
tests:
  - zircon_editor gateway contract tests for in-process, detached, and session implementations
  - zircon_runtime_interface ABI round-trip test for a per-viewport editor-overlay submission
  - zircon_editor viewport projection test proving multi-entity HighlightSet delivery and stale-entity removal
---

# Editor 05 -> Editor 01: HighlightSet 正式 Gateway 契约缺失

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md`
- 来源执行切片：M4.2 `HighlightSet` 正式化
- 修复责任计划：`docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md`
- 交接原因：最底层共享问题是 `EditorRuntimeGateway` 没有向指定 viewport 提交 editor-owned overlay 的中性契约；Editor05 不能通过直接持有 `Scene` 绕过该边界。

## 失败现象与复现证据

M4 计划要求编辑器每帧将 `HighlightSet(Vec<EntityId>)` 经 gateway 推入 runtime，使 runtime 只看见中性实体集合。当前 `EditorRuntimeGateway` 只有 world callback、frame capture、event 和 operation 表面；`EditorRuntimeGatewayHandle`、`InProcessGateway` 与 `SessionGateway` 也没有 overlay 提交 API 或 capability。

同时，`SceneViewportController::build_render_snapshot` 直接接收 `&Scene`，再由 `build_render_packet` 填充 `RenderOverlayExtract.selection: Vec<SelectionHighlightExtract>`。这既保留了 authoring 命名的旧过渡通道，也绕过了 session gateway/ABI。该路径属于全局 F3 的整帧 extract 与直接场景边界风险，不能作为 M4.2 的正式实现。

## 最低共享层根因

Editor core/runtime 边界没有 per-viewport、generation-bound 的 editor overlay 输入通道。因而 in-process 编辑器可以直接构造 runtime extract，而 session editor 无法以相同语义提交 overlay；任何只重命名 `SelectionHighlightExtract` 的改动都会保留双实现和旧架构路径。

## 架构修复验收

- 在 runtime-owned render contract 定义中性 `HighlightSet`，只携带实体集合和渲染属性；runtime contract、ABI 和 capability 名称不得包含 `selection`、`selected` 或其他 authoring 概念。
- `EditorRuntimeGateway` 与其 handle 增加一次性的 per-viewport overlay 提交方法，输入绑定 viewport 与 frame/generation；latest generation 覆盖旧值，不能积累无界队列或跨 viewport 泄漏。
- `InProcessGateway`、`DetachedEditorRuntimeGateway` 和 `SessionGateway` 实现相同失败语义。session 路径必须通过 `zircon_runtime_interface` 的 ABI-safe DTO 与 runtime API 执行，不得回退到 borrowed `World` callback。
- Runtime capability 报告该输入通道。未支持时返回 typed `GatewayError::CapabilityMissing`，不能 panic、静默丢弃或伪造成功。
- 受管测试同时覆盖 in-process 和 session/ABI 路径：多实体集保持稳定顺序，过期 generation 不覆盖新帧，viewport 隔离，detached 失败为 typed error。
- Editor01 回传 fixed 后，Editor05 才删除 `SelectionHighlightExtract` 和 `RenderOverlayExtract.selection`，将有效 active-domain 多选投影为 `HighlightSet`，并保持 anchor、handle 与 gizmo 的 primary-only 交互语义。

## 禁止临时方案

- 禁止保留 `SelectionHighlightExtract`、`overlays.selection`、type alias、兼容字段、双写或旧/新并行 extract。
- 禁止让 Editor05 继续直接传递 `&Scene`、在 UI/viewport 中保存 runtime overlay 全局状态，或用 test-only callback 模拟 session 传输。
- 禁止把 ABI 缺口隐藏为 no-op、silent fallback、全局队列、每帧完整场景 clone 或 call-site exception。
- 禁止降低 M4.2 的多实体、generation、viewport 隔离或 typed-error 验收条件。

## 修复结果与回传

Open state: `待修复`; 当前没有 gateway 实现、受管 Cargo 结果或 accepted closeout 声明。

### 2026-08-15 current-source boundary re-review（未验收）

上面的“没有 gateway 实现”已不再符合共享当前源，不能据此重复建立第二条 overlay 通道。当前 `EditorRuntimeGateway`/stable handle、InProcess、Detached 和 Session gateway 均有 `submit_highlight_set`；`ZrRuntimeApiV6`、runtime FFI、`RuntimeDynamicSession` latest-value storage、app runtime-library loader 与 `EditorCoreProfile` capability 也都接通 `runtime.editor_overlay.highlight_set`。静态现有测试覆盖 canonical entity sort、session ABI viewport 隔离/过期 generation、session capability-missing 和 detached typed error；未在本轮运行 Cargo，故这不是 managed GREEN 或 fixed return。

当前未闭合的旧架构位于 Editor05 viewport consumer：`scene/viewport/render_packet.rs` 仍由 `&Scene` 构造 `RenderOverlayExtract.selection: Vec<SelectionHighlightExtract>`，且没有调用 gateway。该 owner 必须将 active-domain 多选投影为 generation-bound `EditorRuntimeHighlightSet`，经 viewport gateway 一次提交，并物理删除 `SelectionHighlightExtract`、`overlays.selection` 及旧 direct-`Scene` selection extract；不得新增 alias、双写或 fallback。Editor01 不再修改这些 Editor05-owned projection 文件。共享 current-source 仍为脏工作树，所有 lower-contract 文件的 owner/validation/commit 继续以 coordinator transfer 为准。

## 产出记录与时间

| 日期 | 项目 | 状态 | 证据 |
| --- | --- | --- | --- |
| 2026-07-31 | M4.2 HighlightSet gateway failure handoff | open | 已确认 gateway contract/handle/in-process/session 均缺少 editor-overlay 提交能力；旧 `RenderOverlayExtract.selection` 与直接 `Scene` 组包路径仍存在。 |
