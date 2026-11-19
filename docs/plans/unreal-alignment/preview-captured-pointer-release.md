# Preview / PIE：Game 内按下、同窗外松开时释放未回到原 Play

> 取证说明：UE 的本机路径与文件 SHA256 是用户提供参考树的依据；本轮 D: 审计快照/工作记录仅本地保存，不随本 PR 发布，也不冒充 GitHub 可读证据。UE/ZE 测试均未在本轮执行。

状态：**源码确认的输入所有权缺陷；未运行产品或测试。** 这是既有 Editor07 `E-PLAY-P1-20` 输入/focus 桥债务中 live-release 的窄源码复核，非首次发现；不把旧大项称为已修。单独 Draft 调查草案，不是退出/卸库、场景复制或 frame 回收问题。本轮无 Cargo、源码修改、Git 写操作及远端操作。ZE 实读 `cb495bb1c952a24c86f4513b8099bc4942c3a87d`，四个引擎 crate 对公开 `e89349ca51b4e9d024c194cd3bdf645bd25b6c37` 无差异，ZE 链接固定到后者。UE 只读 6.0.0 / BranchName `UE5`；只列短符号、定位与 SHA256。

## 1．两边不一致、路径行号和符号调用链

ZE 原生 `PointerButton` 经 `UiHostWindowEventLoop::handle_pointer_button`（[pointer:76–114](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/retained/contract/window/event_loop/events/pointer.rs#L76-L114>)）先尝试 retained pane input；该续报捕获只支持 `UiAssetPreview` / `GeneratedBottom`，不含 Game（[retained_pane_input:48–112](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/retained/contract/window/retained_pane_input.rs#L48-L112>)）。
通用 release 优先分支只处理 resize/tab drag（[release:7–16](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/retained/contract/pointer/buttons/release.rs#L7-L16>)），其余 Down/Up 均按各自当前位置 `route_pointer_to_pane`（[pane_route:13–36](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/retained/contract/pointer/buttons/pane_route.rs#L13-L36>)）。
真实 authored Game viewport 在 clip 外直接不再返回 Game target（[document:18–32](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/contract/pointer/routing/panes/entry/local/document.rs#L18-L32>)）；只有 Game target 才调用 Game 输入回调（[viewport:29–37](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/retained/contract/pointer/buttons/viewport/viewport.rs#L29-L37>)）→ [callback wiring:20–28](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/retained/app/callback_wiring/pane_surface/viewport.rs#L20-L28>) → `game_viewport_pointer_event` / `runtime_pointer_event` 编码（[game_input:42–73,119–146](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/retained/app/viewport/game_input.rs#L42-L146>)）→ `PlaySessionController::route_preview_input` → 原 Play gateway（[preview_routing:92–122](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/core/play/controller/preview_routing.rs#L92-L122>)）。
UE 嵌入式 PIE 则由 `PlayLevel` 建 GameViewportClient 并交给 Level viewport（[PlayLevel:3060–3064,3150–3154](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/PlayLevel.cpp#L3150-L3154>)）→ `SLevelViewport::StartPlayInEditorSession` 安装 PIE `FSceneViewport`（[SLevelViewport:4988–5013](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Editor/LevelEditor/Private/SLevelViewport.cpp#L4988-L5013>)）→ 允许捕获的 Down 请求 `AcquireFocusAndCapture`（[SceneViewport:694–748](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Slate/SceneViewport.cpp#L694-L748>)）；`FSlateApplication::RoutePointerUpEvent` 优先沿 captor path 而非当前位置命中路由（[SlateApplication:5630–5657](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Application/SlateApplication.cpp#L5630-L5657>)），经 `SViewport::OnMouseButtonUp` 回原 viewport（[SViewport:297–299](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Widgets/SViewport.cpp#L297-L299>)），在该 viewport 世界发送 `IE_Released`（[SceneViewport:801–825](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Slate/SceneViewport.cpp#L801-L825>)）。
UE 证据只支持**已经依捕获策略获得 capture 的支路**；`NoCapture` 的 Up 也可能按新 hit-test 路由。ZE 的 Play/SIE 枚举没有已证实的 `EMouseCaptureMode` 等价，不能仅排除 NoCapture 就宣称所有模式相同；UE 的 [GameViewportClient 捕获策略初始化:564–589](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/GameViewportClient.cpp#L564-L589>) 还受项目/命令行和 headless 条件控制。本问题的强 witness 应以 ZE 自身的已接纳物理 Press/Release 及真实 Runtime Slider capture 合同成立，而非无条件要求所有游戏 Down 都全局捕获。

## 2．有界影响，不扩大为 authoring World 污染

收窄条件为：嵌入式 Play 已接纳 Game 内真实 Runtime Slider 的 Down，并已实际建立其 pointer capture / DragBegin；随后在**同一原生窗口、Game clip 外**空白区域 Up，原 Game pane 始终仍 focused，不引发窗口失焦、pane focus 切换或另一次 Down。Down 已到 `RuntimeDynamicSession::handle_mouse_button`，其 Press/Release 分别提交物理按钮事件（[events:327–375](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/dynamic_api/session/events.rs#L327-L375>)）；缺失 Release 后持久 buttons 集合不会移除按住值（[apply_event:26–30](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/input/runtime/default_input_manager/apply_event.rs#L26-L30>)）。普通 begin_frame 只清边沿、不清按住（[default_input_manager:73–109](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/input/runtime/default_input_manager.rs#L73-L109>)），所以同一 Play 会话可能持续观察 `MouseLeft` 等按住，Runtime UI 拖动/capture 也缺少结尾，直到之后有效释放、输入重置或会话停止。SIE 被 `route_preview_input` 明确排除，本条不据此声称它受影响；没有证据表明 authoring World 被改坏。窗口失焦已有 lifecycle reset 路径，但不覆盖本条件，不能拿失焦重置当同窗 Up 已处理。未测性能、GPU 或实际产品截图。

补救路径并非不存在：[sync_play_preview_input_focus:316–333](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/retained/app/host_lifecycle/tick.rs#L316-L333>) 在 Game view 的 focused 从 true 变 false 时发送 Background；[shell focused 判定:110–114](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/workbench/shell_state.rs#L110-L114>) 查询的是 manager focused view 而非 hover。原 pane 仍 focused 的被动 Move/Up 不触发该补救；native blur 与 pane-focus reset 都应做成功清理对照，不能隐瞒。

真实 Runtime 合同来源为 [range PrimaryPress:40–68](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/ui/surface/surface/default_interactions/range.rs#L40-L68>) 的 `capture_pointer` / DragBegin，以及 [PrimaryRelease:103–136](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/ui/surface/surface/default_interactions/range.rs#L103-L136>) 的 DragEnd / released_capture；不能只凭测试名称认定 capture 已发生。

## 3．根因和 UE 方法

根因在 Editor retained native routing：Game 的第一下 Down 没有建立“后续终态归同一 Play 会话”的手势 owner，而下一次 Up 又按新坐标做 fresh hit。现存 `workbench_pointer_input` 仅观察 hierarchy/tooltip 等（[observer:73–98](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/retained/app/tooltip.rs#L73-L98>)），primary release 后置回调也只结束 hierarchy（[wiring:43–64](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/retained/app/callback_wiring/host_shell/runtime.rs#L43-L64>)），不是 Game 兜底。Runtime 本身能正确接纳 clip 外 Up，问题在事件抵达它之前。UE 的方法是指针捕获 ownership：按捕获策略接纳 Down 后将 Up 路由到 captor，`FSceneViewport` 再处理对的游戏 client/world；不是把世界位置硬裁回视口，也不是由整个 Editor 全局广播释放。

## 4．Rust 应沿用的思想

目标首先是让 Runtime 已 capture 的手势收到自己的终态。是否为所有已接纳 Game Down 记录终态 owner 是待审的具体政策，不由 UE NoCapture 排除法推出。候选可在 Editor 保存有界、短命、按 pointer/button 区分的 Game 手势记录，内容至少是源窗口、surface、坐标映射、`PlayInstanceId` 与 `GatewaySessionIdentity`。fresh Down 才命中当前 Game；配对 Up/必要 Cancel 先查 owner，并在原 surface 坐标系中继续传递外部坐标，不能裁回内部造成假点击。终态必须验证仍是原会话，不能把 Stop A / Start B 之后的旧 Up 送进新 Play。尤其不能仅保存一个布尔捕获标志后继续调用现有“选择当前 attached gateway”的 `route_preview_input`：它在调用时取当前 instance/identity，需额外提供带期望身份的路径或在现有 transition gate 内原子核验。

## 5．ABI、ECS、retained UI 边界

修复 owner 是 `zircon_editor` 的 retained 原生输入契约及 Play routing glue；`zircon_runtime` 已按所属 session 的 `ZrRuntimeEventV1` 处理 `ButtonReleased`，不应把丢事件补偿塞进每帧 ECS 模拟或给所有 session 广播 Release。`zircon_runtime_interface` 的鼠标事件已有状态、按钮、坐标；当前问题无需改 ABI。`zircon_app` 的 session factory/lease 负责创建和销毁运行时，不应转为鼠标捕获 authority。UE 的 UObject/world switch、SlateUser capture 以及可选 OS mouse lock 不能机械搬入 Rust；借鉴的是手势所有权与隔离代际，而非 C++ 生命周期或全局世界切换。本文只对当前鼠标路径给方案，不把 touch/multiplayer 输入支持一并宣称解决。

## 6．具体草案、先测试后修复

独立 Draft 先只放本调查。之后复用现有 normalized native pointer 测试入口（[pointer test entry:165–188](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/retained/contract/window/event_loop/events/pointer.rs#L165-L188>)）及 recording Play gateway，在实际发布的 Game clip 内对真实 Slider Down，先断言 Runtime captured owner / DragBegin 确已建立，保持原 pane focused 再移到同窗非 Game 空白区域并 Up；断言**原同一** Play identity 恰好接收一对 Press/Release、Runtime 物理按钮归 false，且 authoring/editor 手势没有承接该 Up。保留内→内、未配对 Up、不同按钮、SIE、窗口失焦及 Game pane focus 切换的 Background 清理对照；再做 Stop A / Start B 后旧 Up 不进入 B。最小候选修复是在原生分发 fresh-hit 之前增加 Game owner 的终态续报，接纳 Down 成功后才登记，配对终态/会话退休时幂等清除；身份检查和派发放在现有 transition gate 下。surface 被移除、菜单/模态阻断、跨窗与焦点丢失时采用哪种终态，必须写成明确合同；不能让本窄修复复活 retired session。以上均为拟议回归和改法，**尚未执行、尚未验证可编译**。

## 测试与去重表

| 来源 | 名称 / 路径行 | 已有输入与预期 | 对本问题的覆盖 / 既有落点 | 本轮状态 |
|---|---|---|---|---|
| UE 生产合同 | `FSlateApplication::RoutePointerUpEvent` / `FSceneViewport::OnMouseButtonUp`，见上列定位 | 已捕获鼠标 Down 后，Up 归 captor，再送原 viewport client 的 Released | 已查 Slate/SlateCore 测试目录、UnrealEd Play/Automation 相关源码及 AutomationDriver tests，未找到现存同形 PIE 跨 viewport Up 测试；**源码推导，非测试迁移** | UE 未运行 |
| UE 近邻测试，非等价 | `FAutomationDriverSpec / TestFramework.Driver / Element / Release`，[AutomationDriver.spec:981–988](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Plugins/Tests/AutomationDriverTests/Source/AutomationDriverTests/Private/AutomationDriver.spec.cpp#L981-L988>) | 同一个 KeyD 元素 Left Press/Release，期望键序列 D | 未跨 Game viewport，也未检查 PIE session；不能以它冒充迁移 oracle | 只读，未运行 |
| ZE 已有 | `play_preview_input_routes_only_to_the_active_play_gateway`，[cases:471–519](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/core/play/tests/cases.rs#L471-L519>) | Move(12,24) 与 Background，Play 接收，SIE 拒绝 | 只从 controller 入口送事件，没有 native Down→外部 Up；扩展 identity 防串会话对照，不替代 native 回归 | 只读，未运行 |
| ZE 已有 | `ui_capture_release_commits_physical_state_before_propagation_stop`，[physical_input_ownership:35–77](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/dynamic_api/session/tests/physical_input_ownership.rs#L35-L77>) | 直接给 Runtime Slider Down(24,24)、Up(960,640)，要求 MouseLeft=false 且 journal 有 Released | 证明正确的下层合同，但绕过 retained 丢失点；与 native Game gateway fixture 串联复用输入预期 | 只读，未运行 |
| ZE 既有非等价 | `native_escape_cancels_surface_capture_and_restores_gizmo_world_and_history`、`native_focus_loss_cancels_surface_capture_and_restores_gizmo_world_and_history`，[native_viewport_cancel:17–97](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/retained/app/tests/native_viewport_cancel.rs#L17-L97>) | Scene gizmo 外移后 Escape/失焦，恢复 transform/history，并验证后续导航 | Scene authoring，不是 Game Play 鼠标配对释放；不能据其通过/存在关闭本问题 | 只读，未运行 |

去重：已核对 #128（merge `9b37fc5df2c5b0794453c6af53755f66e190688f`，当前 HEAD 祖先）的 DLL pin / teardown 修复及快照，它解决 PIE 后退出时 runtime 映像/清理，不解决 live Game 手势续报。已读 R14 `ca6d11f52` 的 snapshot/gizmo cancel、terminal retirement、runtime consumer 清理测试范围；本机该 exact commit 的 ancestor 查询返回 1，不能把未合分支内容当基线修复或当前运行证据。当前既有 Scene cancel 与 Runtime physical release 对照已逐项区分。本问题也不是 #144 的 viewport affine；近期 #135–145 快照未发现同根因 Draft。

## UE 文件指纹（SHA256）

| 文件 | SHA256 |
|---|---|
| [GameViewportClient.cpp](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/GameViewportClient.cpp>) | `5465E8E384FDAC3439C8154033B10AAF4710A1BF73E694AB4879D1D787A755DC` |
| [PlayLevel.cpp](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/PlayLevel.cpp>) | `D8D9C5E71F93C288C58E117FEBEC701A3FCE7275ED90026ACB4EE56AC091F247` |
| [SLevelViewport.cpp](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Editor/LevelEditor/Private/SLevelViewport.cpp>) | `200EAFBB74BB7D2F001B029AA00D87E7858E94B02D00E0D30D894168318139E5` |
| [SceneViewport.cpp](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Slate/SceneViewport.cpp>) | `D0BA08074BD6A111AB1BB918102CCDFF358A6749ED4F2614B164CE8EB0F9D3B4` |
| [SlateApplication.cpp](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Application/SlateApplication.cpp>) | `06F96167FD20BDE540B7F60629B53C8CFBDDD3AE4B7D93368B9BC4F9F45CFE99` |
| [SViewport.cpp](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Widgets/SViewport.cpp>) | `BC45E9B12CFCD064E0983D22CF057431E0E9D92CACBE9A387DE74A66D34DB1A0` |
| [AutomationDriver.spec.cpp](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Plugins/Tests/AutomationDriverTests/Source/AutomationDriverTests/Private/AutomationDriver.spec.cpp>) | `62A0E0A384BE81FAE4E10EC29925FD6058332FE850842DAED0AA853163422DA6` |

## 7. 真实 Play 相机、同窗外部释放及独立会话观察

测试在一份真实冷项目中启动已链接的 V8 minimal Play，经可见 Game pane 的裁剪几何，向同一 retained 宿主送构造的 Winit 指针事件；观察原游戏会话的 InputManager 原生计数、真实 Camera 姿态以及独立 Edit World。stdout 也列出了 DX12 和 RTX 3060 Laptop 适配器；这不证明已为本次 Game Surface 呈现正确画面。不是实际 OS 输入、打包 DLL、GPU 首帧或 Slider 的验收。

在主案前，状态带普通交互及无主 Up 不得广播；窗内接纳的 RMB drag/Up 必须恰好释放一次并停止转动；失焦可清除 drag 而不凭空计鼠标 release。回到 Game 的 Move 也必须确实送到原 Play，不能以没发输入来制造“相机未动”。 实测原会话鼠标释放 `1→1`；诊断中回入指针移动 `6→7`，随后原 Play 相机姿态仍变化，Edit 相机保持原样。同一主窗 Game focus 未离开；原 Play 也按正常退出路径退休。

真实 V8 Play 相机已进入右键旋转；同窗 Game 外释放时原会话的 release 计数不增加，回入后 Move 实际送达且仍转动。inside、无主输入和失焦对照先通过。没有把相机观测当作 authored Slider 或 OS 验收。 尚无本 PR 生产修复。

Windows x64/MSVC 1.94.1，`zircon_editor --no-default-features --lib --profile test`。实测输入 [同一测试版本](https://github.com/He-Jiahui/ZirconEngine/commit/8036cb85a46298834544b31c31b5978c948022ed)，编译回执 `ddd1b4829a83871a98558d9c3918145300cf5aaff88d9578f50c46bbd9a78414`，实际执行 `4be1290f0057735226bb773dd8f9650c2569a775f682cefb011c8e9ac0fe46dc`，stdout `708501ccdfed0cd6bc09e40bc1b2216463d812775302dbcc2fb28da0ebd64e37`。

## 8. 在 B 的可见区域放开 A 的右键，以及在非 Game 区按下的对照

真实 Play A 已退休后，在新 B 的 Game 内放开旧右键，B 的释放计数 0→1；状态栏按下、Game 内放开也多出一次释放。不同按键与失焦对照通过。原来的同窗外释放丢失仍可稳定复现。

第一组仍是原会话的已接纳右键拖动，释放时未收到 Up，回入后 Move 确实进入且相机仍旋转。第二组使用相同 retained 窗口和真正的 V8 会话：先验证 A 的右键旋转确实已开始；通过正式 Stop 释放 A，创建同一工程的 B，核实新 Play instance、ABI session 及 gateway generation 均不同。旧 Up 在 B 的真实 Game clip 内，先记录 B 的实际输入边计数，再确认 B 新的一组按下、移动、松开均正常并完成停止；Edit 相机未被写。另一案证明非 Game 的右键 Down 没有送入原会话，随后同键尾却在 Game 内变成了孤立的 release；普通中键和真正 blur 的作用另作控制。

这些是正常 retained 路由、构造的 Winit 输入与真实已链接相机/V8 会话结果。它们不是硬件鼠标捕获、staged DLL、GPU 呈现、左键 Slider 拖动或实际窗口管理器输入的验收。

Windows x64/MSVC 1.94.1，`zircon_editor --no-default-features --lib --profile test`，同一[受测提交](https://github.com/He-Jiahui/ZirconEngine/commit/119f1c37ac06834a2d47944ae008677bb97851d8)和编译回执 `97d1985cb46b80ecaab780ad90a6e0910ba159bab2eb8bc4b33ee5e2d053cfe4`。各次均显式 `--ignored --test-threads 1 --show-output`，独立项目和用户配置。

| 过滤器 | 实际运行 | 执行回执 SHA-256 |
|---|---|---|
| `game_drag_release_outside_clip_reaches_original_play` | 0 过 / 1 败 | `f9ac395b149f89f1196b9b36048f2ce871c37623981dbc14456eaa2bc6486964` |
| `game_pointer_owner_` | 1 过 / 2 败 | `abbfb094a72b48d0b753bc6e823867a049df70fc528436063a3989d01b14235a` |
