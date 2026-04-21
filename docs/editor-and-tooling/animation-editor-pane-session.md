---
related_code:
  - zircon_editor/src/ui/animation_editor/mod.rs
  - zircon_editor/src/ui/animation_editor/presentation.rs
  - zircon_editor/src/ui/animation_editor/session.rs
  - zircon_editor/src/ui/host/animation_editor_sessions/mod.rs
  - zircon_editor/src/ui/host/animation_editor_sessions/lifecycle.rs
  - zircon_editor/src/ui/host/animation_editor_sessions/sync.rs
  - zircon_editor/src/ui/host/animation_editor_sessions/editing.rs
  - zircon_editor/src/ui/host/editor_ui_host.rs
  - zircon_editor/src/core/editor_event/runtime/execution/animation_event.rs
  - zircon_editor/src/core/editor_event/runtime/execution/asset_event.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_projection.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/shell_presentation.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/floating_windows.rs
  - zircon_editor/src/ui/slint_host/ui/apply_presentation.rs
  - zircon_editor/src/ui/slint_host/app/host_lifecycle.rs
  - zircon_editor/ui/workbench/animation_editor_pane.slint
  - zircon_editor/ui/workbench/pane_surface.slint
  - zircon_editor/src/tests/editor_event/animation_runtime.rs
  - zircon_editor/src/tests/host/animation_editor.rs
  - zircon_editor/tests/workbench_animation_editor_shell.rs
implementation_files:
  - zircon_editor/src/ui/animation_editor/mod.rs
  - zircon_editor/src/ui/animation_editor/presentation.rs
  - zircon_editor/src/ui/animation_editor/session.rs
  - zircon_editor/src/ui/host/animation_editor_sessions/mod.rs
  - zircon_editor/src/ui/host/animation_editor_sessions/lifecycle.rs
  - zircon_editor/src/ui/host/animation_editor_sessions/sync.rs
  - zircon_editor/src/ui/host/animation_editor_sessions/editing.rs
  - zircon_editor/src/ui/host/editor_ui_host.rs
  - zircon_editor/src/core/editor_event/runtime/execution/animation_event.rs
  - zircon_editor/src/core/editor_event/runtime/execution/asset_event.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_projection.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/shell_presentation.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/floating_windows.rs
  - zircon_editor/src/ui/slint_host/ui/apply_presentation.rs
  - zircon_editor/src/ui/slint_host/app/host_lifecycle.rs
  - zircon_editor/ui/workbench/animation_editor_pane.slint
  - zircon_editor/ui/workbench/pane_surface.slint
plan_sources:
  - user: 2026-04-20 PLEASE IMPLEMENT THIS PLAN
  - .codex/plans/Physics + Full Animation Support 新计划.md
tests:
  - zircon_editor/src/tests/editor_event/animation_runtime.rs
  - zircon_editor/src/ui/animation_editor/session/tests.rs
  - zircon_editor/src/tests/host/animation_editor.rs
  - zircon_editor/tests/workbench_animation_editor_shell.rs
  - cargo test -p zircon_editor animation_editor --locked
  - cargo test -p zircon_editor --locked tests::editor_event::animation_runtime:: -- --nocapture --test-threads=1
doc_type: module-detail
---

# Animation Editor Pane Session

## Purpose

这份文档记录 `zircon_editor` 当前 animation authoring pane 的真实 owner 边界：

- `ui::animation_editor` 负责资产读取、最小 authoring session model 和 Slint pane DTO
- `ui::host::animation_editor_sessions` 负责把 workbench view instance、workspace payload 和 session 脏状态连起来
- `EditorEventRuntime` 的 animation 分支不再只写状态栏，而是能真正驱动 sequence / graph / state-machine pane 内容变化

## Owner Split

### `ui::animation_editor`

`zircon_editor/src/ui/animation_editor/` 现在是 animation pane 本体的领域 owner：

- `session.rs`
  - 从 `.sequence.zranim`、`.graph.zranim`、`.state_machine.zranim` 直接加载资产
  - 维护 sequence 当前帧、timeline 范围、选中 span、播放状态
  - 维护 graph parameter/node 摘要
  - 维护 state-machine state/transition 摘要
  - 负责 `CreateTrack`、`AddKey`、`CreateTransition` 之类 session 内变更
- `presentation.rs`
  - 定义 `AnimationEditorPanePresentation`
  - 把 sequence / graph / state-machine 三种文档模式收敛到同一份 host/slint DTO
- `mod.rs`
  - 保持对外入口轻量，只导出 pane/session 所需公共面

### `ui::host::animation_editor_sessions`

`zircon_editor/src/ui/host/animation_editor_sessions/` 负责 editor host 侧 orchestration，而不是 animation 领域本体：

- `mod.rs`
  - 定义 `AnimationEditorWorkspaceEntry`
- `lifecycle.rs`
  - 创建或恢复 animation editor session
  - 通过 `serializable_payload["path"]` 惰性恢复资产路径
- `sync.rs`
  - 对外暴露 pane 查询和 host/session 同步入口
- `editing.rs`
  - 把 `EditorAnimationEvent` 路由到 sequence 或 graph/state-machine session
  - 同步 title、dirty bit 和 payload path

`EditorUiHost` 现在持有一份 `BTreeMap<ViewInstanceId, AnimationEditorWorkspaceEntry>`，这让 animation pane 和 UI asset pane 一样，有正式的 host-level session registry，而不是只靠 fallback 描述文本。

## Session Model

当前 session model 刻意保持“最小但真实”：

- sequence
  - track 列表来自 `AnimationSequenceAsset::track_paths()`
  - `duration_seconds + frames_per_second` 会投影成 timeline 终止帧
  - 关键帧增删、track 创建/删除/重绑定、timeline scrub/range/span、playback 设置都直接修改 session 内文档
  - 删除当前已选中的 track 时，会同步清掉 `selected_span`，避免 pane 继续显示已经不存在的 timeline 选区
  - 成功 rebind 当前已选中的 track 时，会把 `selected_span` 迁移到目标 track path
  - 对不存在的 track 执行 timeline selection 时，会保持 no-op，避免 pane 产生 phantom selection
  - `SelectTimelineSpan` 现在会把选区端点 clamp 到当前 timeline 可见范围，而不是把越界端点原样写进 session
  - `SetTimelineRange` 收缩可见范围时，也会同步把已有 `selected_span` clamp 回新范围内，避免 pane 摘要继续悬挂在不可见帧区间
  - `SetPlayback` 现在会拒绝 `NaN` / `Inf` 这类非有限速度，避免 pane playback label 被静默污染成无效数值
- graph
  - 展示 parameter 默认值摘要和 node 列表
  - 支持 node 增删、连接/断开和 parameter 文字值写入
  - `RemoveGraphNode("output")` 现在会真实删除 output 节点，而不是因为 output 没有普通 node id 而残留在 pane 摘要里
  - `SetGraphParameter` 现在会拒绝 `NaN` / `Inf` 标量，以及任何带非有限分量的 vector literal，避免 pane 参数摘要被静默污染
- state machine
  - 展示 entry state、state 列表和 transition 摘要
  - 支持 state 增删、entry state 修改、transition 增删和条件写入
  - `SetTransitionCondition` 现在同样会拒绝 `NaN` / `Inf` 标量和带非有限分量的 vector literal，保持现有条件值不变

这仍然不是完整资产写回系统，但它已经足够让 editor pane 反映真实 authoring 命令结果，而不是永远停留在 fallback 或 placeholder 状态。

## Event Targeting

animation 命令现在有两条目标解析规则：

- sequence 命令
  - 依赖当前 `active_center_tab`
  - 目标必须是 `editor.animation_sequence`
- graph / state-machine 命令
  - 优先按命令里的 `graph_path` / `state_machine_path` 匹配已打开的 `editor.animation_graph` instance
  - 没找到时再回退到当前 `active_center_tab`

`execution::animation_event` 还保留了一个稳定降级面：

- 如果当前没有兼容的 animation editor target，事件层把它视为受控 no-op
- 这样 animation binding 归一化测试、headless dispatch 和 stray UI 事件不会重新变成 hard error
- 一旦存在匹配的 animation session，同一条事件链就会落到真实 session 变更

## Presentation Route

animation pane 现在已经接到正式 workbench/slint 投影链：

- `pane_projection.rs`、`shell_presentation.rs`、`floating_windows.rs`
  - 把 `ViewInstanceId` 映射到 `AnimationEditorPanePresentation`
- `apply_presentation.rs`
  - 把 animation pane 数据写进 Slint host presentation
- `host_lifecycle.rs`
  - 在宿主 tick/recompute 周期里保留 animation pane 数据流
- `pane_surface.slint`
  - 新增 animation branch
- `animation_editor_pane.slint`
  - 提供 sequence / graph / state-machine 三种视图

结果是 animation 资产页签现在不再借用通用 fallback pane。中心文档区和浮动窗口都会显示真实 animation pane 数据。

## Asset Routing And Restore

动画资产打开路由已经固定为：

- `*.sequence.zranim` -> `editor.animation_sequence`
- `*.graph.zranim` -> `editor.animation_graph`
- `*.state_machine.zranim` -> `editor.animation_graph`

恢复语义同样固定：

- workspace 只保存 view instance payload 里的 `"path"`
- host 在真正需要 pane 数据时才恢复 session
- 不引入第二套 inspector-only 或 slint-only animation payload

## Acceptance Evidence

这轮直接验证通过的命令是：

- `cargo test -p zircon_editor animation_editor --locked`
- `cargo test -p zircon_editor --locked tests::editor_event::animation_runtime:: -- --nocapture --test-threads=1`

它覆盖了三类关键验收：

- host 能从 payload path 恢复 sequence/state-machine session
- animation authoring 命令能把 sequence/state-machine session 标记 dirty 并更新 pane 内容
- workbench shell 已经声明 animation pane，而不是只剩 fallback surface
- sequence/graph authoring 的关键防御分支现在也被直接回归覆盖，包括：
  - 重复 rebind 不会删源轨道
  - 删除选中轨道会清掉悬空选区
  - rebind 选中轨道会迁移选区到新路径
  - 选择不存在的轨道不会制造 phantom selection
  - timeline 选区不会再保存超出当前可见范围的起止帧
  - 收缩 timeline range 时，已有选区会跟着收口到新范围
  - playback 状态不会再接受非有限速度并把 pane label 写成 `speed=NaN` 或 `speed=inf`
  - 删除 output 节点会真正从 graph pane 消失
  - graph node 不会再接受 `locomotion -> locomotion` 这种自环连接
  - graph parameter 写入遇到无效 literal 时会保持 no-op，而不是把 typed default 悄悄压成 `false` / `0` / `0.0`
  - graph parameter 也不会再接受 `NaN` / `Inf` 标量或带非有限分量的 vector literal
  - graph/state-machine semantic no-op 会保持文档未脏并回到 ignored status
  - 未知 transition operator 不会再被偷偷降级成 `equal`
  - 无效 transition condition literal 也不会再把现有条件值偷偷改写成兜底数字
  - transition condition 也不会再接受 `NaN` / `Inf` 标量或带非有限分量的 vector literal
  - 已有 transition condition 的 typed value 也不再接受错类型 literal，例如已有 `Scalar` 不会被 `"true"` 改写成 `Bool`

更宽的 `zircon_editor` / `zircon_runtime` 全量验证在这个共享脏树上仍然有残余 blocker：

- `zircon_runtime` 当前还存在与本 slice 无关的 Hybrid GI / graphics 测试面 compile drift
- 针对本轮新增 timeline clamp 回归的 `cargo test -p zircon_editor --locked select_timeline_span_clamps_to_current_timeline_range --target-dir target/codex-asset-icon-validation-b -- --nocapture --test-threads=1` 这次没有进入断言阶段，而是在重新链接 `zircon_editor` 时因为 `no space on device` 失败
- 更宽 `zircon_editor` 回归在这份脏树里仍需要单独的稳定化轮次

这不影响本篇记录的 animation pane/session owner 和定向行为已经落地，但意味着“全仓绿”仍然不是这次 slice 的已完成结论。
