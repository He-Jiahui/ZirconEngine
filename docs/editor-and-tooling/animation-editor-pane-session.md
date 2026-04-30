---
related_code:
  - zircon_editor/src/ui/animation_editor/mod.rs
  - zircon_editor/src/ui/animation_editor/presentation.rs
  - zircon_editor/src/ui/animation_editor/session.rs
  - zircon_editor/assets/ui/editor/animation_editor.ui.toml
  - zircon_editor/src/ui/layouts/views/animation_editor_shell_layout.rs
  - zircon_editor/src/ui/layouts/views/mod.rs
  - zircon_editor/src/ui/host/animation_editor_sessions/mod.rs
  - zircon_editor/src/ui/host/animation_editor_sessions/lifecycle.rs
  - zircon_editor/src/ui/host/animation_editor_sessions/sync.rs
  - zircon_editor/src/ui/host/animation_editor_sessions/editing.rs
  - zircon_editor/src/ui/host/animation_editor_sessions/save.rs
  - zircon_editor/src/ui/host/editor_ui_host.rs
  - zircon_editor/src/ui/host/editor_manager_animation_editor.rs
  - zircon_editor/src/core/editor_event/runtime/execution/animation_event.rs
  - zircon_editor/src/core/editor_event/runtime/execution/asset_event.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/host_data.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_projection.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/scene_projection.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/shell_presentation.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/floating_windows.rs
  - zircon_editor/src/ui/slint_host/ui/pane_data_conversion/mod.rs
  - zircon_editor/src/ui/slint_host/ui/apply_presentation.rs
  - zircon_editor/src/ui/slint_host/ui/tests.rs
  - zircon_editor/src/ui/slint_host/app/host_lifecycle.rs
  - zircon_editor/ui/workbench/animation_editor_pane_view.slint
  - zircon_editor/ui/workbench/pane_content.slint
  - zircon_editor/ui/workbench/pane_data.slint
  - zircon_editor/ui/workbench/pane_surface.slint
  - zircon_editor/src/tests/ui/animation_editor/bootstrap_assets.rs
  - zircon_editor/src/tests/ui/boundary/template_assets.rs
  - zircon_editor/src/tests/editor_event/animation_runtime.rs
  - zircon_editor/src/tests/host/animation_editor.rs
  - zircon_editor/tests/workbench_animation_editor_shell.rs
implementation_files:
  - zircon_editor/src/ui/animation_editor/mod.rs
  - zircon_editor/src/ui/animation_editor/presentation.rs
  - zircon_editor/src/ui/animation_editor/session.rs
  - zircon_editor/assets/ui/editor/animation_editor.ui.toml
  - zircon_editor/src/ui/layouts/views/animation_editor_shell_layout.rs
  - zircon_editor/src/ui/layouts/views/mod.rs
  - zircon_editor/src/ui/host/animation_editor_sessions/mod.rs
  - zircon_editor/src/ui/host/animation_editor_sessions/lifecycle.rs
  - zircon_editor/src/ui/host/animation_editor_sessions/sync.rs
  - zircon_editor/src/ui/host/animation_editor_sessions/editing.rs
  - zircon_editor/src/ui/host/animation_editor_sessions/save.rs
  - zircon_editor/src/ui/host/editor_ui_host.rs
  - zircon_editor/src/ui/host/editor_manager_animation_editor.rs
  - zircon_editor/src/core/editor_event/runtime/execution/animation_event.rs
  - zircon_editor/src/core/editor_event/runtime/execution/asset_event.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/host_data.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_projection.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/scene_projection.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/shell_presentation.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/floating_windows.rs
  - zircon_editor/src/ui/slint_host/ui/pane_data_conversion/mod.rs
  - zircon_editor/src/ui/slint_host/ui/apply_presentation.rs
  - zircon_editor/src/ui/slint_host/ui/tests.rs
  - zircon_editor/src/ui/slint_host/app/host_lifecycle.rs
  - zircon_editor/ui/workbench/animation_editor_pane_view.slint
  - zircon_editor/ui/workbench/pane_content.slint
  - zircon_editor/ui/workbench/pane_data.slint
  - zircon_editor/ui/workbench/pane_surface.slint
plan_sources:
  - user: 2026-04-20 PLEASE IMPLEMENT THIS PLAN
  - .codex/plans/Physics + Full Animation Support 新计划.md
tests:
  - zircon_editor/src/tests/ui/animation_editor/bootstrap_assets.rs
  - zircon_editor/src/tests/ui/boundary/template_assets.rs
  - zircon_editor/src/ui/slint_host/ui/tests.rs
  - zircon_editor/src/tests/editor_event/animation_runtime.rs
  - zircon_editor/src/ui/animation_editor/session/tests.rs
  - zircon_editor/src/tests/host/animation_editor.rs
  - zircon_editor/tests/workbench_animation_editor_shell.rs
  - cargo test -p zircon_editor --locked tests::ui::animation_editor:: --lib --target-dir F:/cargo-targets/zircon-codex-a -- --nocapture
  - cargo test -p zircon_editor --locked tests::ui::boundary::template_assets:: --lib --target-dir F:/cargo-targets/zircon-codex-a -- --nocapture
  - cargo test -p zircon_editor --locked host_scene_projection_converts_host_owned_panes_to_slint_panes --lib --target-dir F:/cargo-targets/zircon-codex-a -- --nocapture
  - cargo check -p zircon_editor --lib --locked --target-dir F:/cargo-targets/zircon-codex-a
  - cargo test -p zircon_editor animation_editor --locked
  - cargo test -p zircon_editor --locked tests::editor_event::animation_runtime:: -- --nocapture --test-threads=1
  - cargo test -p zircon_editor --lib save_sequence_session_persists_track_changes_and_clears_dirty -- --nocapture
  - cargo test -p zircon_editor --lib save_graph_session_persists_parameter_changes_and_clears_dirty -- --nocapture
  - cargo test -p zircon_editor --lib save_state_machine_session_persists_entry_state_changes_and_clears_dirty -- --nocapture
  - cargo test -p zircon_editor --lib editor_manager_saves_animation_sequence_editor_session_and_clears_dirty_metadata -- --nocapture
doc_type: module-detail
---

# Animation Editor Pane Session

## Purpose

这份文档记录 `zircon_editor` 当前 animation authoring pane 的真实 owner 边界：

- `ui::animation_editor` 负责资产读取、最小 authoring session model 和中性业务 presentation
- `ui::layouts::views::animation_editor_shell_layout` 负责从 bootstrap `.ui.toml` 提取 animation pane 壳层 frame
- `ui::host::animation_editor_sessions` 负责把 workbench view instance、workspace payload 和 session 脏状态连起来
- `ui::host::animation_editor_sessions` 现在也负责 animation asset 的保存、项目内 `res://...` 重新导入和保存后的 metadata 回写
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
- `save.rs`
  - 对外暴露 `save_animation_editor(...)`
  - 先把当前 session 序列化回源资产路径，再按需触发项目内 `res://...` 资产重导入
  - 保存成功后回写 workbench metadata，清掉 pane dirty 状态

`EditorUiHost` 现在持有一份 `BTreeMap<ViewInstanceId, AnimationEditorWorkspaceEntry>`，这让 animation pane 和 UI asset pane 一样，有正式的 host-level session registry，而不是只靠 fallback 描述文本。

## Session Model

当前 session model 刻意保持“最小但真实”：

- sequence
  - track 列表来自 `AnimationSequenceAsset::track_paths()`
  - `duration_seconds + frames_per_second` 会投影成 timeline 终止帧
  - sequence session 在投影 timeline 之前会先清洗资产 timing 元数据：非有限 `frames_per_second` 回退到默认 `30.0`，非有限 `duration_seconds` 收口为 `0.0`
  - 关键帧增删、track 创建/删除/重绑定、timeline scrub/range/span、playback 设置都直接修改 session 内文档
  - `save()` 会把底层 `AnimationSequenceAsset` 序列化回 `asset_path`，并且只在写盘成功后清掉 dirty bit
  - 删除当前已选中的 track 时，会同步清掉 `selected_span`，避免 pane 继续显示已经不存在的 timeline 选区
  - 成功 rebind 当前已选中的 track 时，会把 `selected_span` 迁移到目标 track path
  - 对不存在的 track 执行 timeline selection 时，会保持 no-op，避免 pane 产生 phantom selection
  - `SelectTimelineSpan` 现在会把选区端点 clamp 到当前 timeline 可见范围，而不是把越界端点原样写进 session
  - `SetTimelineRange` 收缩可见范围时，也会同步把已有 `selected_span` clamp 回新范围内，避免 pane 摘要继续悬挂在不可见帧区间
  - `SetPlayback` 现在会拒绝 `NaN` / `Inf` 这类非有限速度，避免 pane playback label 被静默污染成无效数值
- graph
  - 展示 parameter 默认值摘要和 node 列表
  - 支持 node 增删、连接/断开和 parameter 文字值写入
  - `save()` 会把底层 `AnimationGraphAsset` 序列化回 `asset_path`，并且只在写盘成功后清掉 dirty bit
  - `RemoveGraphNode("output")` 现在会真实删除 output 节点，而不是因为 output 没有普通 node id 而残留在 pane 摘要里
  - `SetGraphParameter` 现在会拒绝 `NaN` / `Inf` 标量，以及任何带非有限分量的 vector literal，避免 pane 参数摘要被静默污染
- state machine
  - 展示 entry state、state 列表和 transition 摘要
  - 支持 state 增删、entry state 修改、transition 增删和条件写入
  - `save()` 会把底层 `AnimationStateMachineAsset` 序列化回 `asset_path`，并且只在写盘成功后清掉 dirty bit
  - `SetTransitionCondition` 现在同样会拒绝 `NaN` / `Inf` 标量和带非有限分量的 vector literal，保持现有条件值不变

这已经不再是只读或 dirty-only 的 session stub：底层 sequence / graph / state-machine 资产都可以经由同一 session 模型落盘。但它仍然不是完整会话快照系统，因为当前帧、timeline 选区和 playback 这类 editor-local 状态不会一起序列化。

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
  - 继续负责非 scene/game pane 的外层 surface，并把正文内容委托给 `PaneContent`
- `pane_content.slint`
  - 现在显式 import `AnimationEditorPaneView`
  - 把 `AnimationSequenceEditor` / `AnimationGraphEditor` 两类 pane kind 路由到 animation 视图
- `animation_editor_pane_view.slint`
  - 提供 sequence / graph / state-machine 三种视图
  - 现在消费 host 投影进来的 `shell_layout`，不再继续持有 `64px` header、`12px` inset、`140px/148px` band offset 这类手写壳层真源

结果是 animation 资产页签现在不再借用通用 fallback pane。中心文档区和浮动窗口都会显示真实 animation pane 数据。

## Bootstrap Shell Layout Authority

这轮 cutover 继续把 animation pane 的壳层几何从 `.slint` 手写常量迁回 crate `assets/`：

- [`animation_editor.ui.toml`](../../zircon_editor/assets/ui/editor/animation_editor.ui.toml)
  - 现在固定了 `AnimationEditorHeaderPanel` / `AnimationEditorBodyPanel`
  - 同时把 `HeaderModeRow`、`HeaderPathRow`、`HeaderStatusRow` 以及 sequence / graph / state-machine 三套 mode band 全部落进同一份 bootstrap asset
  - `BodyPanel` 使用 overlay 容器，让三种 mode shell 共享同一块 authoring 内容区域，而不是在 `.slint` 里各自抄一遍 inset/offset 公式
- [`animation_editor_shell_layout.rs`](../../zircon_editor/src/ui/layouts/views/animation_editor_shell_layout.rs)
  - 从 crate `assets/` 读取 tree asset、注册 editor base style、编译 `UiSurface`
  - 把 control frame 萃取成 `AnimationEditorShellLayout`
- [`host_data.rs`](../../zircon_editor/src/ui/layouts/windows/workbench_host_window/host_data.rs)、[`scene_projection.rs`](../../zircon_editor/src/ui/layouts/windows/workbench_host_window/scene_projection.rs)、[`pane_data_conversion/mod.rs`](../../zircon_editor/src/ui/slint_host/ui/pane_data_conversion/mod.rs)
  - 把这份 layout 作为 `AnimationEditorPaneViewData -> AnimationEditorPaneData -> AnimationEditorShellLayoutData` 的正式投影链
- [`pane_content.slint`](../../zircon_editor/ui/workbench/pane_content.slint)
  - 显式导入 `AnimationEditorPaneView`，把 animation pane kind 接回通用 `PaneContent` 路由
- [`animation_editor_pane_view.slint`](../../zircon_editor/ui/workbench/animation_editor_pane_view.slint)
  - 现在只消费 `root.pane.shell_layout.*` frame，再在对应 band 内放 mode text、timeline text、track/state/node 列表
  - 这让 Slint owner 收窄为“消费 frame 并摆叶子控件”，不再自己定义动画 pane 的顶层几何

这一步并没有把 animation 业务 leaf 全部迁出 Slint；track list、parameter list、state list 仍然是 pane 视图 owner。但最外层 pane shell 已经和 `UiAssetEditor` / `AssetBrowserPane` 一样，回到 bootstrap `.ui.toml` authority。

## Asset Routing And Restore

动画资产打开路由已经固定为：

- `*.sequence.zranim` -> `editor.animation_sequence`
- `*.graph.zranim` -> `editor.animation_graph`
- `*.state_machine.zranim` -> `editor.animation_graph`

恢复语义同样固定：

- workspace 只保存 view instance payload 里的 `"path"`
- host 在真正需要 pane 数据时才恢复 session
- 不引入第二套 inspector-only 或 slint-only animation payload

## Save And Persistence

animation editor 现在已经有正式的 host save 链路，而不是只会把 pane 标成 dirty：

- `AnimationEditorSession::save()`
  - 只序列化底层 `AnimationSequenceAsset` / `AnimationGraphAsset` / `AnimationStateMachineAsset`
  - 写回当前 `asset_path`
  - 仅在底层 `std::fs::write(...)` 成功后清掉 dirty bit
- `EditorUiHost::save_animation_editor(...)`
  - 确保目标 `ViewInstanceId` 的 session 已恢复
  - 执行 session save
  - 如果源路径位于 `<project>/assets` 下，就推导成 `res://...` 并触发 `asset_manager.import_asset(...)`
  - 最后同步 workbench metadata，保证标签页 dirty 状态和 payload path 与磁盘一致
- `EditorManager::save_animation_editor(...)`
  - 对外暴露给 host / 调用方的稳定入口

保存语义也保持刻意收紧：

- 目前只有底层动画资产文档会落盘
- 当前帧、timeline 可见范围、选中 span、playback 开关和速度仍然是 editor-local session 状态
- 因此保存解决的是 authoring 文档持久化，不是完整 pane UI 状态恢复

## Acceptance Evidence

当前文档记录的直接验证命令包括：

- `cargo test -p zircon_editor --locked tests::ui::animation_editor:: --lib --target-dir F:/cargo-targets/zircon-codex-a -- --nocapture`
- `cargo test -p zircon_editor --locked tests::ui::boundary::template_assets:: --lib --target-dir F:/cargo-targets/zircon-codex-a -- --nocapture`
- `cargo test -p zircon_editor --locked host_scene_projection_converts_host_owned_panes_to_slint_panes --lib --target-dir F:/cargo-targets/zircon-codex-a -- --nocapture`
- `cargo check -p zircon_editor --lib --locked --target-dir F:/cargo-targets/zircon-codex-a`
- `cargo test -p zircon_editor animation_editor --locked`
- `cargo test -p zircon_editor --locked tests::editor_event::animation_runtime:: -- --nocapture --test-threads=1`
- `cargo test -p zircon_editor --lib save_sequence_session_persists_track_changes_and_clears_dirty -- --nocapture`
- `cargo test -p zircon_editor --lib save_graph_session_persists_parameter_changes_and_clears_dirty -- --nocapture`
- `cargo test -p zircon_editor --lib save_state_machine_session_persists_entry_state_changes_and_clears_dirty -- --nocapture`
- `cargo test -p zircon_editor --lib editor_manager_saves_animation_sequence_editor_session_and_clears_dirty_metadata -- --nocapture`

它覆盖了几类关键验收：

- host 能从 payload path 恢复 sequence/state-machine session
- animation authoring 命令能把 sequence/state-machine session 标记 dirty 并更新 pane 内容
- sequence / graph / state-machine 资产文档现在都能通过同一 session save 链路真正写回磁盘
- host save 会清掉 workbench dirty metadata，而不会破坏原有 payload path
- workbench shell 已经声明 animation pane，而不是只剩 fallback surface
- animation pane bootstrap asset 能导出 header/body 和 sequence / graph / state-machine mode frame
- `pane_content.slint` 现在显式导入 animation pane view，Slint build 不再因为缺失 pane import 而把 animation surface 卡死在 build script
- Slint source guard 已锁住 animation pane 不再退回到 `64px` header、`y: 140px` graph node band、`y: 148px` transition band 这类硬编码壳层公式
- host -> Slint 投影回归已经证明 animation shell layout 会穿过 `host_data.rs` / `pane_data_conversion/mod.rs` 边界，而不是重新把这份几何留在 `.slint`
- sequence/graph authoring 的关键防御分支现在也被直接回归覆盖，包括：
  - 重复 rebind 不会删源轨道
  - 删除选中轨道会清掉悬空选区
  - rebind 选中轨道会迁移选区到新路径
  - 选择不存在的轨道不会制造 phantom selection
  - timeline 选区不会再保存超出当前可见范围的起止帧
  - 收缩 timeline range 时，已有选区会跟着收口到新范围
  - sequence 资产里的非有限 `duration_seconds` / `frames_per_second` 不会再把 pane timeline 终止帧撑成异常大值
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

本轮没有把结论扩大成“整个 `zircon_editor` / `zircon_runtime` 工作区都已经全绿”。当前能够确认的是 animation editor 的保存链路和相关 Slint pane 路由已经在定向验证下跑通；更宽的 workspace 级回归仍需要单独稳定化轮次。
