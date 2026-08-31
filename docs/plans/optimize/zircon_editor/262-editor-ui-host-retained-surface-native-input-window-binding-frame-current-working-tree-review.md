---
title: Editor UI Host、Retained Surface、Native Input、Window、Binding 与 Frame Authority 当前工作树复审
category: zircon_editor
report_id: Editor262
review_date: 2026-08-31
baseline_head: 14c89f9776bed828cc85e05e4b9914b3f8d1e784
verification_head: 14c89f9776bed828cc85e05e4b9914b3f8d1e784
supersedes_currentness_of:
  - docs/plans/optimize/zircon_editor/23-ui-asset-hud-widget-binding-theme-icon-accessibility-menu-flow-font-atlas-authoring-review.md
  - docs/plans/optimize/zircon_editor/253-editor-scene-viewport-host-render-product-surface-lifecycle-frame-currentness-multi-viewport-current-working-tree-review.md
  - docs/plans/optimize/zircon_editor/255-editor-interactive-tool-scheduler-resource-lease-input-capture-scene-mode-modal-extension-lifecycle-current-working-tree-review.md
canonical_parent_owners:
  - docs/plans/optimize/zircon_runtime/200-runtime-ui-surface-input-focus-pointer-capture-ime-accessibility-frame-authority-current-working-tree-review.md
  - docs/plans/optimize/zircon_editor/253-editor-scene-viewport-host-render-product-surface-lifecycle-frame-currentness-multi-viewport-current-working-tree-review.md
  - docs/plans/optimize/zircon_editor/255-editor-interactive-tool-scheduler-resource-lease-input-capture-scene-mode-modal-extension-lifecycle-current-working-tree-review.md
related_code:
  - zircon_editor/src/ui/retained_host/app.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle
  - zircon_editor/src/ui/retained_host/app/callback_wiring
  - zircon_editor/src/ui/retained_host/app/native_windows
  - zircon_editor/src/ui/retained_host/host_contract/window
  - zircon_editor/src/ui/retained_host/host_contract/globals
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer
  - zircon_editor/src/ui/retained_host/event_bridge.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/common
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge
  - zircon_editor/src/ui/template_runtime
  - zircon_editor/src/ui/binding/core
  - zircon_editor/src/ui/binding_dispatch/editor_event_normalization.rs
  - zircon_editor/src/ui/control
  - zircon_editor/src/ui/host/editor_event_dispatch.rs
  - zircon_editor/src/ui/host/editor_event_runtime_access/component_dispatch.rs
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Application/SlateApplication.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Application/SlateUser.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Framework/Application/SlateUser.h
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Input/Reply.h
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Widgets/Text/SlateEditableTextLayout.cpp
  - dev/godot/scene/main/viewport.cpp
  - dev/godot/scene/gui/control.cpp
  - dev/godot/scene/main/window.cpp
  - dev/bevy/crates/bevy_input_focus/src/lib.rs
  - dev/bevy/crates/bevy_input_focus/src/directional_navigation.rs
  - dev/bevy/crates/bevy_input_focus/src/tab_navigation.rs
  - dev/bevy/crates/bevy_winit/src/accessibility.rs
  - dev/Fyrox/fyrox-ui/src/lib.rs
  - dev/Fyrox/fyrox-ui/src/message.rs
  - dev/Fyrox/fyrox-ui/src/widget.rs
  - dev/Fyrox/fyrox-ui/src/text_box.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Debugging/DebugManager.cs
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 1. 结论

当前 retained host 已经包含真实的 winit 窗口生命周期、native 输入翻译、布局/命中索引、浮动窗口 presenter、IME 开关、模板投影、局部重绘和 callback wiring，不能再按“空壳 UI”描述。然而这些部件仍然由一个超大的 `RetainedEditorHost`、一个共享可变 `HostContractState`、多组临时桥接器和大量直接 callback 拼接在一起。它可以在单主窗口、内建 workbench 和少量浮动窗口场景下工作，但没有达到 Unreal Slate、Godot Viewport/Control、Fyrox UserInterface 或 Bevy focus/accessibility 所要求的可组合、可代际验证、可撤销、可诊断的工程级 UI host。

本轮没有发现新的唯一 P0；Runtime200、Editor253、Editor255 和 Editor23 的 canonical P0 继续由原 owner 追踪，不在本报告重复计数。当前选择集新增或重判 32 项 P1、12 项 P2 和 28 个工程闸门：

| 等级 | Open/Fail | Partial | Closed/Pass | 合计 |
|---|---:|---:|---:|---:|
| P0（继承 owner，不重复计数） | 0 | 0 | 0 | 0 |
| P1 | 37 | 5 | 0 | 42 |
| P2 | 10 | 2 | 0 | 12 |
| Gate | 25 | 3 | 0 | 28 |

最危险的差异是以下六条链路同时存在但没有统一事实源：

1. 窗口事件循环、`RetainedEditorHost::tick` 和 Runtime UI frame 提交分别拥有生命周期；它们通过 `Rc<RefCell>`、`Weak` 和闭包互相调用，没有 session/window/frame authority。
2. `UiHostCallbacks` 与 `PaneSurfaceCallbacks` 是单槽 `Option<Rc<dyn Fn...>>` 表；注册会覆盖旧 callback，调用直接重入，无法表达订阅、优先级、取消、背压或 terminal receipt。
3. native pointer route 和 workbench/runtime pointer callback 是两次独立调用，事件顺序由调用方决定，返回值不是同一个 typed `Reply`，因此 capture、consume、bubble、cancel 和失败补偿无法统一。
4. retained projection 在多个桥接器内复制并重新构建；`node_by_control_id`、binding route 和属性 mutation 仍大量线性查找/字符串寻址，没有稳定 node handle、结构代际和 scene diff/patch transaction。
5. 结构、几何、交互、viewport、命中测试、diagnostics 各自递增 generation；输入只带 sequence/timestamp，渲染、a11y、focus、hit-test 不能共同 pin 一个 committed frame token。
6. 浮动窗口有独立 `UiHostWindow`，但 presenter store 当前只同步壳、viewport chrome、UI Asset 等内容；没有按 `MainPageId + surface tree + viewport product + frame generation` 发布 Scene/Simulate/Game 图像的完整链路。

因此下一阶段应先收敛 authority 和协议，再扩张模板组件、面板数量或输入功能。目标架构应是：

```text
NativeWindowRegistry
        |
        v
UiRuntimeDriver -> UiWindowSession(s) -> UiFrameAuthority
        |                    |
        |                    +-> InputRouter -> Focus/Capture/IME/A11y
        |                    +-> RetainedSceneStore -> Diff/Patch/HitIndex
        |                    +-> SurfacePresenter -> PresentReceipt
        v
EditorUiControlService -> Typed Editor Event / Operation Receipt
```

`RetainedEditorHost` 应退化为 Editor composition root；窗口、输入、retained scene、presenter、binding/event 和 Runtime session 都必须拥有可验证的 owner、generation、错误阶段与关闭状态。

# 2. 审查边界与冻结指标

## 2.1 选择集

本轮逐文件扫描以下闭包，排除 Tooling（按用户要求未来迁移到 Rust，另立报告）：

- `retained_host/host_contract/window`：winit application handler、native window/presenter、metadata、redraw、IME、focus、input outcome。
- `retained_host/host_contract/globals`：presentation、generation、hit index、interaction/menu/focus state、callback tables。
- `retained_host/host_contract/native_pointer`：capture、move/button/wheel、overlay/body routing、drag/resize/text focus。
- `retained_host/app.rs`、`app/host_lifecycle`、`app/callback_wiring`、`event_bridge.rs`：composition root、tick/recompute/render、native child windows、callback dispatch、effect merge。
- `retained_host/callback_dispatch/common` 与 `template_bridge`：binding/event normalization、template projection、surface bridge、property/data sync、virtual rows。
- `ui/template_runtime`、`ui/binding/core`、`ui/binding_dispatch/editor_event_normalization.rs`、`ui/control`、`ui/host/editor_event_dispatch.rs` 及 component dispatch：模板 runtime、route service 和 Editor event receipt。

当前磁盘选择集指标如下。指标包含工作树中未提交的文件，按规范化相对路径排序；fingerprint 使用 `lowercase path + NUL + raw bytes + NUL` 的 SHA-256。

| 范围 | files | lines | non-empty | bytes | test markers | ignored | fingerprint |
|---|---:|---:|---:|---:|---:|---:|---|
| retained host/window/input/pointer/callback/template/binding/control/editor event closure | **651** | **67,210** | **62,767** | **2,472,319** | **359** | **18** | `995e8668e7bf57b072024a60783927412021c643176aca68371494a75f676f8c` |
| Unreal/Godot/Bevy/Fyrox/Unity Graphics references | selected local slices | not re-counted | not re-counted | not re-counted | n/a | n/a | paths recorded above |

本轮只进行静态源码、生产反查、已有测试声明和本地参考源码核对；没有运行 Cargo、真实 Editor、winit 多窗口、IME、screen reader、GUI automation、fault injection、scale/soak 或 benchmark。因此不能据此声称性能、帧时间或表现优于 Unreal。

## 2.2 与既有 owner 的边界

| 主题 | 本报告处理 | 不重复计数的 owner |
|---|---|---|
| Runtime UI driver、Runtime input/focus/a11y 底座 | 只记录 Editor 接入断点 | Runtime200、Runtime11A/77/78/82 |
| Scene viewport product、multi-viewport、surface lifecycle | 只记录 retained host 如何丢失 window/product/frame identity | Editor253 |
| Tool scheduler、modal/input capture lease | 只记录 native host 没有接入该 authority 的调用面 | Editor255 |
| Command registry、keymap、palette/operation semantics | 只记录 callback/binding 边界如何丢失命令 receipt/generation | Editor261 |
| Widget/asset/theme/font/a11y 具体产品面 | 只记录其 host contract 和 retained projection依赖 | Editor23 |

# 3. 当前 owner 拓扑

## 3.1 启动与关闭

`run_editor_with_config` 创建 `UiHostWindow`，构造 `RetainedEditorHost`，加载 plugin/template/startup scene/layout，将 host 包装进 `Rc<RefCell<_>>`，调用 `wire_callbacks`，刷新 UI 后进入 `ui.run()`。关闭时又由 host 收集 diagnostics、停止 runtime/autosave/settings 并提交 project close。窗口退出、Runtime shutdown、autosave drain、plugin watch 和 native child close 没有一个统一的 session state machine；各路径依靠闭包顺序和布尔状态协调。

`RetainedEditorHost` 同时持有 runtime lease/controller/manager、shell geometry、多个 surface bridge、pointer bridge、native presenter store、floating projection、cache、generation 和 callback source window。这使它成为跨层 God object：任何一个子系统想要刷新 frame、处理 pointer 或关闭窗口，都可以借助共享可变借用进入其它域。

## 3.2 窗口与 presenter

`UiHostWindow::new` 创建 `HostContractState`、wake channel、focus observer、presenter factory 和 job binding；`run` 创建 winit event loop 并安装 `UiHostWindowEventLoop`。`HostWindowHandle` 直接读写共享状态的 position/size/scale/visibility/maximized，snapshot 也直接从 host presentation 绘制成 RGBA。

浮动窗口目标由 `NativeFloatingWindowTarget { MainPageId, title, bounds, surface_tree_id }` 产生。`NativeWindowPresenterStore` 为每个目标创建独立 `UiHostWindow`，记录 `HostPresentationGenerationCursor`，并支持 patch viewport chrome/UI Asset。当前 `sync_native_window_presenters` 的生产调用只为这些 presenter 应用壳、pane payload、模板和 chrome；没有携带 viewport image/product identity。每个新 `HostContractState` 初始化独立空 `HostViewportImageSet`，因此“浮动窗口有自己的窗口”不能等同于“浮动 viewport 具有独立 live frame”。

## 3.3 输入与 callback

winit event loop 将 platform event 翻译成 runtime `UiWindowInputContext`，并在 pointer button/wheel/move、keyboard、IME、focus 等路径调用 host contract。pointer button/wheel 先调用 workbench pointer callback，再调用 native route；pointer move 先 native route，再 dispatch runtime/workbench callback。两条路径没有统一的 event id、capture owner、Reply/disposition 或提交 receipt。

`UiHostCallbacks` 和 `PaneSurfaceCallbacks` 由宏生成注册/调用方法。每个字段最多保存一个 `Option<Rc<dyn Fn...>>`；后注册者覆盖先注册者，没有 subscriber id、remove token、generation、priority 或 backpressure。callback closure 再通过 `Weak<RefCell<RetainedEditorHost>>` 借用 host，直接调用 `tick`、`commit_interactive_frame_update` 或 mutation，重入顺序由 winit 事件和闭包实现细节决定。

## 3.4 retained projection 与 binding

`EditorUiHostRuntime` 同时管理 component catalog、template registry、plugin docs、action registry、projection cache 和 template instance cache。`RetainedUiHostAdapter` 将整个投影复制为 `Vec<RetainedUiHostNodeModel>`；`node_by_control_id` 线性搜索。`BuiltinHostWindowTemplateBridge::recompute_layout_with_workbench_model_at_scale` 重新构建 surface 并重新投影，property/data sync 再逐个 control/property 调用 mutation。

`EditorUiRouter` 是 `HashMap<UiEventPath, Vec<Handler<T>>>`，只支持 exact route；dispatch 依次执行全部 handler 并返回 `Vec<T>`，没有优先级、consume/terminal、capture/bubble、取消或 handler generation。projection support 还会为一次投影创建新的 `EditorUiControlService` 并重新注册所有 route，这与长期运行的 control service 形成两个事实源。

## 3.5 frame 与 effect

`HostContractState` 将 structure、geometry、interaction、viewport、hit-test、diagnostics 分开递增 generation；`replace_host_presentation` 会重置菜单、pane interaction、text focus 和 viewport images。`recompute_if_dirty` 根据多个 dirty flag 选择 shell/layout/window-metrics fast path，`submit_render_frame_if_dirty` 再把 Runtime render extract 写入 viewport。

`UiHostEventEffects` 以布尔 OR、可选 shell scope 和 toast vector 合并。它适合把多个局部 mutation 粗略地合并成一次 refresh，却不能表达哪个输入产生了哪些 write set、哪个副作用已提交、哪个副作用需要补偿、哪个结果因 stale generation 被拒绝。

# 3.6 可复核源码锚点

以下是本轮判定使用的关键行号锚点；行号以当前工作树冻结时的文件为准，实施前需随 fingerprint 重新核对。

1. `zircon_editor/src/ui/retained_host/app.rs:223-327`：`run_editor_with_config` 创建窗口、host、callback wiring 并进入 `ui.run()`；同一启动函数还负责 shutdown 收尾。
2. `zircon_editor/src/ui/retained_host/app.rs:660-777`：`RetainedEditorHost` 聚合 runtime lease/controller、UI bridge、pointer、presenter、cache、generation 和 callback source 状态。
3. `zircon_editor/src/ui/retained_host/host_contract/globals/callback_methods.rs:3-12`：宏生成的 `$on_*` 直接覆盖单个 callback slot，`$invoke_*` clone 后同步调用。
4. `zircon_editor/src/ui/retained_host/host_contract/globals/callbacks/host.rs:10` 与 `callbacks/pane.rs:11`：Host/Panes callback table 分别以大量 `Option<Callback...>` 保存事件。
5. `zircon_editor/src/ui/retained_host/host_contract/window/event_loop/events.rs:28-75`：platform event 翻译、workbench pointer callback 与 native dispatch 分散在事件分支中。
6. `zircon_editor/src/ui/retained_host/host_contract/window/event_loop/input.rs:18-30`：input metadata sequence 使用 `saturating_add`，没有 epoch 或 terminal receipt。
7. `zircon_editor/src/ui/retained_host/host_contract/window/metadata.rs:1-35`：timestamp 来源是 `SystemTime`，window id 通过固定 `NATIVE_HOST_WINDOW_ID` 附加。
8. `zircon_editor/src/ui/retained_host/host_contract/globals/state.rs:49-69,143-231`：多组 presentation generation 与 `HostViewportImageSet` 位于同一可变 state，但不是单一 committed frame token。
9. `zircon_editor/src/ui/retained_host/app/host_lifecycle/tick.rs:9-129`：tick、interactive commit、pending frame update 由 host lifecycle 直接驱动。
10. `zircon_editor/src/ui/retained_host/app/host_lifecycle/recompute.rs:13-245` 与 `render_submission.rs:8-71`：dirty fast path、recompute 和 render dirty 消费是分散布尔协议。
11. `zircon_editor/src/ui/retained_host/event_bridge.rs:38-183`：`UiHostEventEffects` 以 dirty mask/scope/toast merge，缺少 typed write-set receipt。
12. `zircon_editor/src/ui/binding/core/router_dispatch.rs:5-18`：router 只有 exact `HashMap<Path, Vec<Handler>>`，dispatch 返回全部 handler 结果。
13. `zircon_editor/src/ui/template_runtime/retained_adapter.rs:134-143`：retained projection 使用 `Vec`，`node_by_control_id` 线性搜索。
14. `zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/projection_support.rs:61,110-111`：projection support 先线性查节点，再创建临时 `EditorUiControlService` 注册 routes。
15. `zircon_editor/src/ui/retained_host/app/host_lifecycle/native_window_presenters/sync.rs:14-62` 与 `app/native_windows/store.rs:48-221`：native child presenter 按 source generation 同步并 patch chrome/UI Asset，没有 viewport image/product 参数。
16. `zircon_editor/src/ui/retained_host/host_contract/window/focus_observer.rs:7-19`：focus observer 是单一 `Option<Rc<dyn Fn()>>`，重复注册只能返回 `AlreadyRegistered`。
17. `zircon_editor/src/ui/retained_host/host_contract/window/event_loop/input_outcome.rs:105-167`：input outcome tracker 只有 active/pending profiling batch，不是产品级输入 receipt store。

# 4. 直接源码证据与差距

以下条目是本轮新登记或对当前工作的重判，共 42 项 P1、12 项 P2 和 28 个 Gate。每项都给出事实、工程风险和要求的重构方向；状态 `Open` 表示尚无工程底座，`Partial` 表示局部机制存在但不能作为产品 authority。

## 4.1 P1：owner、生命周期与多窗口

| ID | 状态 | 当前证据 | 工程级差异 / 重构要求 |
|---|---|---|---|
| EUIH-P1-001 | Open | `app.rs` 在一个 `RetainedEditorHost` 中保存 runtime、UI、bridge、cache、presenter、pointer、generation 和 lifecycle 状态。 | 拆为 `EditorHostComposition`、`UiWindowRegistry`、`UiRuntimeSession`、`RetainedSceneStore`、`InputRouter`、`SurfacePresenter`、`EditorEventGateway`；root 只组装并转发 typed service。 |
| EUIH-P1-002 | Open | `run_editor_with_config` 以 `Rc<RefCell<RetainedEditorHost>>` 连接 event loop、callbacks、runtime tick、close 和 shutdown。 | 建立 `EditorUiSessionId`、`UiWindowSessionId` 和显式 lifecycle state；禁止跨 owner 通过共享可变借用直接调用 tick/mutation。 |
| EUIH-P1-003 | Open | `UiHostWindow` 自己拥有 winit loop/wake/presenter；`RetainedEditorHost` 自己拥有 Runtime heartbeat、jobs、autosave、render submit。 | 由 `UiRuntimeDriver`/`UiWindowRegistry` 统一 pump、wake、frame commit 和 shutdown barrier；Editor 只消费 receipt。 |
| EUIH-P1-004 | Open | native main/floating close、runtime shutdown、autosave、plugin watch、settings 和 project close 分散在多个闭包/模块。 | 引入可枚举的 `Closing -> Draining -> Closed/Degraded` 状态与 owner barrier；每个 late callback 必须得到 `Closed`/`Rejected` receipt。 |
| EUIH-P1-005 | Partial | `NativeWindowPresenterStore` 有 `MainPageId` key、stale target 删除、applied generation 和独立 `UiHostWindow`。 | 窗口 identity 仍不是 Runtime-qualified `WindowId + SurfaceLease + session`；把 `MainPageId`、native handle、surface tree、document/view instance、presenter generation 收敛为不可伪造 `WindowSessionToken`。 |
| EUIH-P1-006 | Open | `NATIVE_HOST_WINDOW_ID` 是固定常量；reserved input 甚至可不带 window id，callback source 通过 `with_callback_source_window` 推断。 | 每个事件必须携带 registry-issued window/session token；禁止从当前 callback source 或固定常量推断 source window。 |
| EUIH-P1-007 | Open | `NativeFloatingWindowTarget` 只有 title/bounds/tree；`sync_native_window_presenters` 没有 viewport image/product/frame receipt 参数。 | 建立 per-window `SurfaceProductBinding`，明确 Scene/Simulate/Game product、camera/view instance、frame generation、format、present status 和 fallback。 |
| EUIH-P1-008 | Open | child `HostContractState::new` 默认 `HostViewportImageSet::default()`；store 生产 patch 覆盖 chrome/UI Asset，未见 per-window viewport image publication。 | 将图像发布纳入同一 retained frame transaction；主窗口和浮动窗口都只能显示其 pinned product，禁止空槽/全局 kind fallback。 |

## 4.2 P1：callback、输入、capture、IME、focus

| ID | 状态 | 当前证据 | 工程级差异 / 重构要求 |
|---|---|---|---|
| EUIH-P1-009 | Open | callback macro 对每个事件只保存一个 `Option<Rc<dyn Fn...>>`，注册没有 subscriber id，后者覆盖前者。 | 使用 typed event channel/subscription token；注册返回可撤销 handle，按 owner/generation 管理，支持多个消费者和关闭时自动撤销。 |
| EUIH-P1-010 | Open | callback invocation clone callback 后直接调用，host closure 可重新进入 `tick`/mutation。 | 引入 non-reentrant dispatch queue；callback 只产生 `UiEventIntent`，统一 router 在 transaction 边界执行。 |
| EUIH-P1-011 | Open | pane callbacks 有数十个字符串、布尔、`Vec<String>` primitive payload。 | 以 schema-generated `PaneEvent`/`ControlEvent` 取代 tuple/string，携带 control handle、surface generation、source window、input id 和 capability。 |
| EUIH-P1-012 | Open | event loop 对同一 pointer 事件分开调用 workbench callback 与 native route；button/wheel/move 顺序不同。 | 单一 `InputRouter::dispatch` 返回结构化 `Reply { disposition, capture, focus, effects, retry }`；native/workbench/runtime 只消费同一事件上下文。 |
| EUIH-P1-013 | Open | native pointer route 根据 committed presentation hit index；workbench bridge 另行修改 hovered template hit；两者没有共同 terminal outcome。 | 把 hit-test、capture、hover、drag、focus 和 editor tool lease 绑定到一个 `InputTransaction`，支持 capture/bubble/consume/reject/cancel。 |
| EUIH-P1-014 | Partial | `move_dispatch`/`button_dispatch` 已有 capture、overlay、resize、text-focus 和异常清理分支。 | capture owner 仍是各桥接器局部 state，未进入 Editor255 的 scheduler/modal authority；需要 lease id、owner generation、steal/revoke 和 terminal release receipt。 |
| EUIH-P1-015 | Open | mouse pressed count 是单个 `u32`/饱和计数；touch-like pointer 被过滤或拒绝，没有 device/pointer table。 | 建立 per-device/per-pointer state（device id、pointer id、buttons、contact phase、capture owner、window/session）；支持多触点、pen、touch cancel 和重放。 |
| EUIH-P1-016 | Open | `next_input_metadata` 与 `reserve_input_metadata` 使用 `saturating_add`，没有 input epoch/deadline/idempotency。 | 使用 registry-issued `InputId { window, epoch, sequence }`，溢出进入新 epoch 并产生诊断；receipt 校验 stale/duplicate/deadline。 |
| EUIH-P1-017 | Open | timestamp 来自 `SystemTime::now().duration_since(UNIX_EPOCH)`，window id 是固定 `NATIVE_HOST_WINDOW_ID`。 | 事件应同时保存 monotonic arrival time、platform timestamp、window token、device id 和 clock provenance；跨线程排序不能依赖 wall clock。 |
| EUIH-P1-018 | Open | focus observer 只有一个 `Option<Rc<dyn Fn()>>`，focus loss 直接调用 callback；focus state 分散在 host/text/pane。 | 建立 per-window/per-user focus manager，支持 focus path、reason、restore target、nested modal 和 a11y focus；与 Runtime200 的 focus authority 对接。 |
| EUIH-P1-019 | Open | IME 只根据 `text_input_focus_accepts_text()` 调用 native `set_ime_allowed`。 | IME context 必须有 text session id、document/selection revision、composition update、ack/cancel、window token 和 shutdown behavior；不能只有一个 bool。 |
| EUIH-P1-020 | Partial | `input_outcome.rs` 记录 active input/pending present batch，新的输入会中断旧 profiling scope。 | 这是 profiling tracker，不是产品 receipt/backpressure；建立 bounded input queue、drop/coalesce policy、terminal outcome 和 present dependency。 |

## 4.3 P1：retained scene、projection、binding 与 schema

| ID | 状态 | 当前证据 | 工程级差异 / 重构要求 |
|---|---|---|---|
| EUIH-P1-021 | Open | `RetainedUiHostProjection.nodes` 是 Vec，`node_by_control_id` 线性 find；binding_for_control 先找 node 再查 route。 | 每个 surface 使用 stable `NodeHandle`/slot map、control id index、parent/children adjacency 和 generation；热路径禁止线性全树查找。 |
| EUIH-P1-022 | Open | `project_instance` 递归复制 authored nodes/attributes/style/bindings；workbench bridge recompute 会重建 surface 和 projection。 | 实现 immutable scene snapshot + keyed structural diff/patch；layout-only、style-only、state-only 更新必须不重建整个 surface。 |
| EUIH-P1-023 | Open | projection support 每次 project 可能创建新的 `EditorUiControlService` 并注册全部 routes。 | `EditorUiControlService` 必须是 window/session scope 的唯一 route authority；投影只提交 route delta，并返回 generation。 |
| EUIH-P1-024 | Open | `RetainedUiHostComponentKind` 是有限 enum，未知组件降为 `Unknown`；插件节点通过 runtime/property maps 承载。 | 组件 ABI 应是 versioned capability/schema，未知节点要有可诊断的 incompatible state，而不是静默 Unknown；提供 migration/feature negotiation。 |
| EUIH-P1-025 | Open | `UiPropertyMutationRequest` 按字符串 control/property 逐个 mutation；缺少 control 时多个路径返回 `Ok(())` 或 fallback label/value。 | 生成 typed control schema，批量 mutation 必须返回 changed/missing/stale/invalid；禁止把缺节点当成功。 |
| EUIH-P1-026 | Open | binding payload 通过 string symbol、TOML/JSON `UiBindingValue` 转换；非 finite float 在 JSON conversion 中变为 `null`。 | 采用 versioned binary/typed payload codec，明确 NaN/Inf/decimal policy；schema mismatch、lossy conversion、unknown symbol 都必须 typed error。 |
| EUIH-P1-027 | Open | `EditorUiRouter` 只支持 exact path，Vec handler 全部执行并返回 Vec<T>。 | 支持 route hierarchy、capture/target/bubble、priority、consume/stop、handler identity/removal、route generation、deadline 和 error isolation。 |
| EUIH-P1-028 | Open | `EditorUiEventNormalization` 将多个 payload family 拼成 `EditorEvent`，不支持项以 `UnsupportedBinding`。 | normalization 必须携带 source schema/version、capability、unknown-field policy 和 route generation；不能让新插件只能落入 String/Custom。 |
| EUIH-P1-029 | Partial | `EditorHostEventController` 已记录 event id/sequence/source/binding path/operation/transaction/save generation/effects/result。 | 该 receipt 位于 host event 层，native callback 和 present 层没有引用它；需要贯穿 input -> route -> operation -> frame -> present 的 correlation id。 |
| EUIH-P1-030 | Open | `UiHostEventEffects` 以 bool OR、scope replace 和 toast vector 合并。 | 使用 typed write-set/effect journal，列出 applied/rejected/deferred/compensation，提交后生成 `UiCommitReceipt`；merge 不能丢失 scope 或失败阶段。 |

## 4.4 P1：frame、dirty、render 与可观测性

| ID | 状态 | 当前证据 | 工程级差异 / 重构要求 |
|---|---|---|---|
| EUIH-P1-031 | Open | structure、geometry、interaction、viewport、hit-test、diagnostics 各自 generation；`presentation_generation()` 只是打包多个数字。 | 建立单一 `UiFrameToken { session, structure, layout, interaction, viewport, a11y, product }`；输入、focus、hit-test、render、a11y 都必须 pin 同一 token。 |
| EUIH-P1-032 | Open | `replace_host_presentation` 会重置 menu/pane/text focus/viewport images；geometry patch 另行更新 hit index。 | 全量替换不能悄悄清除交互和产品状态；用 versioned frame transaction 明确保留/重置字段，并支持 stale commit rejection。 |
| EUIH-P1-033 | Partial | recompute 有 presentation/layout/window-metrics fast path 与 committed shell state；render submit 有 dirty reason。 | fast path 仍是 host 私有布尔协议；定义 invalidation DAG、dependency token、frame budget 和 bounded retry，不能靠调用顺序隐式正确。 |
| EUIH-P1-034 | Open | redraw retry 采用指数退避；`submit_render_frame_if_dirty` 失败、fallback presenter upgrade 和 deferred present 分散处理。 | 建立 presenter state machine（Starting/Ready/Degraded/Lost/Closing）和 terminal receipt；失败必须保留 last-good frame、原因、重试上限和恢复动作。 |
| EUIH-P1-035 | Open | redraw/input outcome 只有 profiling counters，无法按 input id 查询最终呈现或拒绝原因。 | 建立 bounded receipt store/cursor，覆盖 input, event, frame commit, GPU submit, present, a11y update；支持 resync 与 drop 诊断。 |
| EUIH-P1-036 | Open | diagnostics、theme、viewport images 使用独立 generation，跨 domain 没有一致 snapshot。 | diagnostics 必须引用 frame/session/product token；性能计数器应区分 coalesced、dropped、stale、fallback、device-loss。 |
| EUIH-P1-037 | Open | layout/model 变化后可能同时触发 main host 与所有 native child patch，缺乏共享 frame budget。 | 建立 per-window damage region、priority、budget and deadline；presenter 只能消费与自身 token 匹配的 patch。 |
| EUIH-P1-038 | Open | `take_snapshot` 直接从 host presentation paint RGBA，未说明 source frame/product/scale provenance。 | capture 必须绑定 `UiFrameToken + WindowSessionToken + color/scale/format`，并区分 debug snapshot、test capture、用户导出 artifact。 |

## 4.5 P1：Runtime、Editor event、插件和 a11y 边界

| ID | 状态 | 当前证据 | 工程级差异 / 重构要求 |
|---|---|---|---|
| EUIH-P1-039 | Open | retained host callback 可以直接 `tick`、调用 runtime shell、刷新 surface；Runtime200 仍指出 `UiRuntimeDriver` 是空壳。 | Editor 必须只依赖 Runtime UI driver facade/lease；禁止私有 Dynamic surface set 或 Editor host 直接成为 Runtime UI owner。 |
| EUIH-P1-040 | Open | component dispatch 使用 registry/lock/property mutation；event result 和 UI effects 分离。 | 建立 cross-crate neutral `UiEventEnvelope`/`UiCommitReceipt`，保留 authority/generation/principal/session，错误不能在 editor boundary 压成 String。 |
| EUIH-P1-041 | Open | callback payload 多为 string control id/primitive，plugin contribution 通过模板/runtime maps 注入。 | 插件 UI 需要 capability-scoped schema、stable control identity、version negotiation、revoke epoch 和 route teardown；未知插件节点不得静默显示。 |
| EUIH-P1-042 | Open | host state 有 theme/diagnostics overlay，但未见每窗口 AccessKit adapter、action queue 与 teardown。 | 每个 `UiWindowSession` 建立 a11y tree adapter，引用同一 frame token，处理 action/ack/focus/rebuild/close；与 Runtime200 的 a11y owner 对齐。 |

## 4.6 P2：可维护性、性能和产品完整性

| ID | 状态 | 当前证据 | 重构要求 |
|---|---|---|---|
| EUIH-P2-001 | Open | 多模块使用 `String` control id、property name、symbol namespace。 | 生成 interned ids/schema ids；保留字符串只用于边界解析和 diagnostics。 |
| EUIH-P2-002 | Open | BTreeMap/HashMap/Vec 在 projection、binding、pane data sync 中重复复制。 | 以 arena/slot map、Arc immutable snapshot、incremental indexes 和 copy-on-write patch 控制分配；用 profile 证明收益。 |
| EUIH-P2-003 | Open | 每次 scale/layout recompute 可重建整个 builtin host surface。 | 引入 layout cache key（document/style/scale/constraint generation）和局部 invalidation；禁止无变化重投影。 |
| EUIH-P2-004 | Open | route dispatch exact-only，所有 handler 顺序执行。 | 为热路径建立 compiled route table、priority index、bounded fan-out 和 per-route metrics。 |
| EUIH-P2-005 | Partial | `HostWorkbenchHitIndex` 有 geometry patch fast path。 | hit index 仍绑定 host presentation 而非稳定 scene/frame token；完善 spatial index、pointer-device query 和 stale diagnostics。 |
| EUIH-P2-006 | Open | pane data sync 逐属性 mutation，缺行时用 fallback labels/values。 | 用 row schema/virtualization/typed model patch；fallback 必须显式 `Unavailable` 状态而不是伪造业务值。 |
| EUIH-P2-007 | Open | redraw wake deadlines、maintenance、runtime、input timer、lifecycle 分散在 state 字段。 | 统一 scheduler wheel/priority queue，记录 wake reason、deadline、owner、coalescing 和 missed budget。 |
| EUIH-P2-008 | Open | `FocusObserver`/callback table 没有 owner teardown contract。 | 所有 observer 返回 RAII subscription，关闭/插件 revoke 自动取消并记录未清理订阅。 |
| EUIH-P2-009 | Open | event metadata 只有 timestamp/sequence/optional window id。 | 加入 trace/span、device, pointer, session, document, frame, security principal 和 source capability。 |
| EUIH-P2-010 | Open | editor UI tests 多为源码属性、局部 model/patch tests，缺真实 winit/多窗口/IME/a11y。 | 建立 deterministic host harness、virtual clock、fake presenter、multi-window/input replay、property/fuzz/stress suites。 |
| EUIH-P2-011 | Partial | native presenter 有 create/upgrade/fallback 和 close prompt 路径。 | 把 fallback/upgrade 结果纳入 user-visible health/status/telemetry，区分 startup fallback 与 runtime device loss。 |
| EUIH-P2-012 | Open | 没有可证明的 retained host frame-time、allocation、input latency、present drop 基线。 | 建立 Unreal/Bevy/Godot/Fyrox 对齐的 benchmark protocol，报告 p50/p95/p99、allocation/frame、drop/retry、multi-window scaling。 |

# 5. 参考引擎对比

| 能力 | 参考实现 | Zircon 当前状态 | 需要达到的目标 |
|---|---|---|---|
| 应用级输入与多用户焦点 | Unreal `SlateApplication`/`FSlateUser`，Bevy `InputFocus`/`FocusedInput` | event loop + host callbacks；没有 per-user/focus path authority | window/user/pointer scoped focus/capture，统一 route/reply/receipt |
| 输入 Reply/传播 | Unreal `FReply` capture/focus/detect drag；Godot Viewport/Control GUI route | native route 与 workbench callback 分离，无 terminal disposition | 单一 typed Reply，capture/bubble/consume/cancel/reject |
| retained tree 与消息队列 | Fyrox `UserInterface`、`MessageDirection`、RoutingStrategy | Vec projection、直接 property mutation、exact route | immutable snapshot + diff/patch + queued message transaction |
| 文本编辑/IME | Unreal `SlateEditableTextLayout`；Godot Window/Control | `set_ime_allowed` bool + text focus state | selection/composition/document revision/ack/cancel/session |
| Accessibility | Bevy AccessKit adapter per window；Godot AccessibilityServer | state 中有语义数据但未见 per-window adapter/action queue | frame-pinned semantic tree、action routing、teardown/resync |
| editor command/UI contribution | Unreal ToolMenus/command lists；Unity DebugManager/contexts | EditorUiControlService 与临时 route service 并存，插件 payload 多为 string | capability/schema/versioned contribution 与唯一 route authority |
| frame/presenter | Unreal centralized tick/paint/window routing；Godot viewport/window lifecycle | 多 generation + dirty bool + child presenter patch | cross-domain frame token、present receipt、degraded recovery |
| 多窗口 viewport product | Unreal per-window Slate/viewport clients；Godot per-window Viewport/Window | 独立 UiHostWindow 存在，但无 per-window viewport image publication | window/view instance/product/frame 绑定，禁止 kind-global fallback |

这些参考源码证明的是 owner/协议/生命周期形态，不是简单照搬 C++ 或 ECS。Zircon 应保留 Rust ownership、typed errors 和 runtime/editor 分层，但必须提供同等强度的 authority 和验证边界。

# 6. 重构路线

## Phase A：身份与关闭状态（前置阻塞）

1. 定义 `EditorUiSessionId`、`UiWindowSessionId`、`SurfaceSessionId`、`InputId`、`UiFrameToken`、`PresentReceiptId`，禁止固定 native window id 和 callback source 推断。
2. 建立 `UiWindowRegistry` 与 `UiRuntimeDriver` adapter，统一主窗口、浮动窗口、Runtime surface lease、wake 和 shutdown barrier。
3. 把 close/shutdown/revoke 统一为状态机；所有 late callback、pending input、present retry 必须获得 terminal receipt。
4. 保留 Editor253/Runtime200 的 viewport/product owner，不在 host 再造 kind-global image fallback。

## Phase B：输入与事件协议

1. 以 `InputRouter` 统一 native platform event、pointer capture、focus、IME、workbench tool lease 和 Runtime UI dispatch。
2. 引入 `UiEventEnvelope`、`UiEventReply`、`UiEventEffects`、`UiCommitReceipt`；native route、workbench callback、Editor event controller 只处理同一 correlation id。
3. 将 callback table 改成 typed subscription bus：token、owner、priority、generation、cancel、bounded queue、reentrancy guard。
4. 接入 Editor255 scheduler/modal authority；补齐 touch/pen/multi-pointer、focus path、IME composition 和 a11y action。

## Phase C：retained scene 与 template runtime

1. 以 immutable retained scene snapshot 为唯一结构事实源，建立 stable node handle、control index、parent adjacency、layout/hit-test index。
2. 将 template projection 从全量 clone 改为 structural/style/state diff；layout-only 和 property-only patch 不得重建 surface。
3. 将 `EditorUiControlService` 提升为 window/session scope 的唯一 route authority；删除 projection support 中的临时 service。
4. 生成组件/row/binding schema，支持 plugin capability/version/revoke epoch；缺失节点、未知 symbol、schema mismatch 进入 typed diagnostic。

## Phase D：frame、present 与产品观测

1. 用 `UiFrameAuthority` 原子提交 structure/layout/interaction/viewport/hit/a11y/product snapshot；输入和 presenter pin token。
2. 建立 presenter state machine、last-good frame、retry budget、device-loss/degraded state 和 per-window damage queue。
3. 将 Runtime event receipt、Editor operation receipt、frame commit、GPU submit、present、a11y update 串成 bounded receipt store/cursor。
4. 以 deterministic virtual clock/fake presenter 构建测试，再进行真实 winit、多窗口、IME、screen reader、plugin reload 和压力验证。

## Phase E：性能和迁移收尾

1. 为 projection、route dispatch、hit test、frame commit、allocation、input latency、present drop 建立基线和预算。
2. 迁移旧 `UiHostCallbacks`、字符串 control id、primitive payload 和 direct property mutation；迁移期间保留显式 compatibility adapter，但禁止新调用点增加。
3. 删除 `Rc<RefCell<RetainedEditorHost>>` 跨域重入和 fixed window metadata；通过 source tests、API lint 和 architecture gate 防回归。

# 7. 工程闸门

| Gate | 当前 | 通过条件 |
|---|---|---|
| EUIH-G01 | Fail | 每个窗口都有 registry-issued session token，不依赖固定 `NATIVE_HOST_WINDOW_ID` |
| EUIH-G02 | Fail | main/floating window 的 Runtime surface lease 与 `MainPageId` 可双向追踪 |
| EUIH-G03 | Fail | close/shutdown/revoke 有可枚举 terminal state 和 barrier |
| EUIH-G04 | Fail | `RetainedEditorHost` 不再直接拥有所有 runtime/UI/presenter authority |
| EUIH-G05 | Fail | callback 注册返回可撤销 token，重复注册不静默覆盖 |
| EUIH-G06 | Fail | callback dispatch 有 reentrancy guard 和 bounded queue |
| EUIH-G07 | Fail | pointer/native/workbench/runtime 使用一个 typed event/reply |
| EUIH-G08 | Fail | capture/focus/IME/tool/modal/a11y 都记录 owner/generation |
| EUIH-G09 | Fail | multi-pointer/device state 和 cancel/replay 可测试 |
| EUIH-G10 | Partial | metadata 同时保留 monotonic/platform clock、window/session/device/source |
| EUIH-G11 | Fail | retained node 有 stable handle/index，热路径无线性 `node_by_control_id` |
| EUIH-G12 | Fail | template surface 支持 structural/style/state diff，不做无变化全量重建 |
| EUIH-G13 | Fail | 一个 `EditorUiControlService` 覆盖 session 内全部 routes |
| EUIH-G14 | Fail | binding/component schema 有 version/capability/unknown-field policy |
| EUIH-G15 | Fail | property mutation 缺失/过期/非法返回 typed outcome，不能 silent `Ok(())` |
| EUIH-G16 | Fail | 单一 `UiFrameToken` 同时约束 structure/layout/input/hit/a11y/render/product |
| EUIH-G17 | Partial | dirty/recompute 有依赖图、预算和可验证 fast path |
| EUIH-G18 | Fail | presenter 有 Ready/Degraded/Lost/Closing 状态、last-good frame 和 retry receipt |
| EUIH-G19 | Fail | per-window Scene/Simulate/Game 图像 publication 带 product/frame provenance |
| EUIH-G20 | Fail | input/event/frame/present/a11y 有 correlation id 和 bounded receipt cursor |
| EUIH-G21 | Fail | Runtime200 `UiRuntimeDriver` 作为唯一 Runtime UI owner 被 Editor 使用 |
| EUIH-G22 | Fail | Editor255 scheduler/modal/input capture 具有真实 retained host caller |
| EUIH-G23 | Fail | 每窗口 AccessKit adapter/action queue 与 frame token、close teardown 对齐 |
| EUIH-G24 | Partial | plugin UI contribution 有 revoke epoch、route teardown、schema migration |
| EUIH-G25 | Fail | deterministic host harness 覆盖多窗口、IME、focus、capture、stale frame |
| EUIH-G26 | Fail | property/fuzz/replay/soak 验证 input ordering、route cancellation、shutdown |
| EUIH-G27 | Fail | benchmark 报告 p50/p95/p99 latency、alloc/frame、present drop 和窗口扩展曲线 |
| EUIH-G28 | Fail | source/API checks 阻止旧 callback/direct mutation/fixed-id 新调用点回归 |

# 8. 验证与实施前置

本报告没有修改生产 Rust、ZUI、Cargo manifest、ABI 或测试。实施任何 Phase A-D 前必须重新计算选择集 fingerprint，并至少完成：

- `cargo check -p zircon_editor`、focused unit/property tests 和 workspace compile closure。
- deterministic virtual-clock host harness：主窗口 + 两个浮动窗口、重排/缩放、pointer cancel、IME composition、focus restore、present retry。
- stale frame/input replay：旧 `UiFrameToken`、重复 `InputId`、关闭后 late callback、plugin revoke 中的 route dispatch。
- fake/real presenter 对比：startup fallback、device loss、resize failure、last-good frame、per-window viewport product provenance。
- Windows native winit、screen reader/AccessKit、multi-monitor scale、touch/pen/mouse、长时间 soak 与 allocation/latency benchmark。

在这些证据出现前，不得把当前 retained host 的 callback 数量、generation 字段、独立浮动窗口或局部 fast path 解释为“已经完成工程级 UI framework”，也不得宣称性能或表现优于 Unreal。
