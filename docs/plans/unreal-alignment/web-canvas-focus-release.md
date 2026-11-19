# Web 输入：画布失去同页焦点后，松开的键仍被原实例视为按住

在 Chrome 157 中，画布收到 `a` 的按下事件。宿主读取按下帧后调用兄弟按钮的 `focus()`，浏览器把 `a` 的松开事件送到了按钮。旧实例在松开帧和下一空帧仍报告游戏键按住。观测通过 WebAssembly 示例读取生产监听器的原始输入和 `DefaultInputManager` 状态。[浏览器观察程序](https://github.com/He-Jiahui/ZirconEngine/blob/8c0a258c2ca43b0da7ee0c864a6f679cbe7d58e1/zircon_app/examples/web_listener_fixture.rs)

## 1. 用户路径

`WebEngine.load` 接受宿主页面提供的 canvas。带有表单、工具栏或第二个游戏画布的页面可以在同一文档内转移焦点。原画布得到键盘焦点时会接收 `keydown` 和 `keyup`。本例在读完按下帧后直接聚焦页面按钮，确保目标元素确实改变。[Web 嵌入入口](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_app/src/web/mod.rs#L18-L40) · [画布键盘监听](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_app/src/web/listeners.rs#L99-L120)

## 2. 实测影响

物理键码 `65` 与逻辑键 `a` 同时受影响。下表记录浏览器输入驱动实际发送的单个按键。两次键盘事件及中间的画布 `blur` 均有 `isTrusted=true`，页面可见，窗口始终聚焦，输入队列丢弃数为零。

| 采样帧 | DOM 目标 | 原画布的输入队列 | 键码 65 / 逻辑 a 按住 | 两者的释放边沿 |
|---|---|---|---|---|
| 按下 | canvas | `KeyboardInput(65, a, pressed=true)` | 是 / 是 | 无 / 无 |
| 转移焦点 | canvas 失焦，按钮获焦 | 空 | 是 / 是 | 无 / 无 |
| 松开 | 按钮 | 空 | 是 / 是 | 无 / 无 |
| 下一空帧 | 焦点仍在按钮 | 空 | 是 / 是 | 无 / 无 |

同一程序中，若画布持续持有焦点，两个键身份都会在松开当帧产生释放边沿，下一帧回到空闲。[浏览器观察程序](https://github.com/He-Jiahui/ZirconEngine/blob/8c0a258c2ca43b0da7ee0c864a6f679cbe7d58e1/zircon_app/examples/web_listener_fixture.rs)

## 3. 根因与 Unreal 的生产处理

ZE 将键盘监听挂在画布上，将失焦清理挂在 window 上。元素失焦时触发的 `blur` 留在画布，window 本身保持聚焦，文档可见性也保持原值。旧实例的下一帧清除上帧边沿，按住集合继续保留。[窗口与文档监听](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_app/src/web/listeners.rs#L138-L167) · [逐帧输入顺序](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/browser/frame.rs#L45-L66) · [按键状态更新](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/input/runtime/default_input_manager/apply_event.rs#L48-L69)

UE 的 Slate 将焦点变更通知旧控件。`SViewport::OnFocusLost` 转交 `FSceneViewport`，键盘用户失焦会清除视口键状态并通知 `UGameViewportClient::LostFocus`。默认设置启用 `FlushPressedKeys`，PlayerInput 会释放已按下的键。文件位置和版本指纹见第 6 节。

## 4. ZE 的修复方向

在原画布的元素失焦事件进入时，按事件顺序结束该实例已接纳的键盘输入。游戏输入管理器在下一次成功提交帧后，应报告对应的物理键和逻辑键各产生一次释放边沿。重新聚焦并再次按下时，应能重新产生按下边沿。

## 5. 输入状态与模块边界

ZE 的窗口级 `FocusLost` 同时清理活动触点、鼠标增量、手柄输入及 IME 状态，Web UI 还会向该实例的各 surface 发送窗口失焦。画布的键盘焦点与窗口、触点和其他画布各有所有者，元素切焦应只终止原实例的键盘状态。另一项输入调查处理的是始终聚焦时 `A` 按下、`a` 松开的逻辑名配对。[全窗口失焦的状态清理](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/input/runtime/input_state.rs#L180-L218) · [Web UI 的窗口失焦语义](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/browser/ui.rs#L149-L174) · [逻辑键配对 #151](https://github.com/He-Jiahui/ZirconEngine/pull/151)

## 6. 现有测试与 ZE 回归

UE 的 `Slate.FocusTest` 在同一窗口内切换两个按钮，检查旧按钮的一次失焦回调及新按钮的一次获焦回调。三个 EnhancedInput 原测分别覆盖排队的释放、仍按住的键，以及松开后清除忽略状态。它们用模拟的 `EKeys::A` 直接调用 PlayerInput 的清理。此次 UE 对照采用原测试源码。

ZE 的现有管理器用例直接提交 `FocusLost`。同页回归还需逐帧覆盖画布到兄弟按钮的真实焦点转移、原页普通按放、返回原画布后的再次按下，以及页面隐藏时的既有清理。双画布对照应检查只有失去焦点的实例产生释放边沿。[ZE 管理器已有失焦测试](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/input/tests/input_manager/frame_state.rs#L441-L527)

UE 参考树报告版本 6.0.0，BranchName 为 `UE5`。以下行区间和 SHA-256 标识本次对照的原文件。

```text
Engine/Build/Build.version:2–9  3dcd8b6654872d013f2b00aa5f0ef1e0db89d4e8627ce820554675a27f228ea6
Engine/Source/Runtime/Slate/Private/Framework/Application/SlateApplication.cpp:3190–3214,5165–5214  06f96167fd20bde540b7f60629b53c8cfbddd3ae4b7d93368b9bc4f9f45cfe99
Engine/Source/Runtime/Slate/Private/Widgets/SViewport.cpp:362–368  bc45e9b12cfcd064e0983d22cf057431e0e9d92cacbe9a387de74a66d34db1a0
Engine/Source/Runtime/Engine/Private/Slate/SceneViewport.cpp:1443–1462  d0ba08074bd6a111ab1bb918102ccdff358a6749ed4f2614b164ce8eb0f9d3b4
Engine/Source/Runtime/Engine/Private/GameViewportClient.cpp:2601–2618  5465e8e384fdac3439c8154033b10aaf4710a1bf73e694ab4879d1d787a755dc
Engine/Source/Runtime/Engine/Private/UserInterface/InputSettings.cpp:38–44  b88803966acba458dee998400cb168758d7586d95fecb4dae8667cd3785ff95f
Engine/Source/Runtime/Engine/Private/PlayerController.cpp:2383–2389  acb605a0588591b9ed60d88daa84a2bac966d9d1ad4eadf5f2faadb759df36f7
Engine/Source/Runtime/Engine/Private/UserInterface/PlayerInput.cpp:143–193  e58db5316cab04dd2b4ad5ae3fdf974bb31b5f0a8670cb7cd59682d163d0a547
Engine/Plugins/Tests/WidgetAutomationTests/Source/WidgetAutomationTests/Private/FocusTest.cpp:16–68,81–89  8c9a53f3a8539cd110feb6933ae87cc45ec57bde18cda01cd1d5db97abde79b7
Engine/Plugins/EnhancedInput/Source/EnhancedInputTestSuite/Private/InputSystemTest.cpp:602–647,649–681,683–728  ccb611ef6f25639e7d96a8e313887f8022394fd8f42d78125e997bd2d658a9e4
Engine/Plugins/EnhancedInput/Source/EnhancedInputTestSuite/Private/InputTestFramework.h:185–186  a6e171a7dcebc52a6ec9e149660b67bce63747e980e648251e65d6ac994ca080
Engine/Plugins/EnhancedInput/Source/EnhancedInputTestSuite/Private/InputTestFramework.cpp:291–312  f0eb3e6e6fc60003c23b31bf6e52b39d2a155b4b809c734e18d4614a88419cbd
```

## 7. 原草稿的浏览器回归

提交 `ba32996f1fb67131c2632afb72ba3de19b68a4c5` 在本草稿上独立编译生产 `Listeners`、输入队列和 `DefaultInputManager`，用 Chrome 157 检查 21 个输入帧。普通按放与窗口失焦的对照达到预期。转移到兄弟按钮、两个画布之间来回切换以及同帧重新聚焦等场景共有 **8 帧失败、13 帧通过**。[固定按键与焦点场景](https://github.com/He-Jiahui/ZirconEngine/blob/ba32996f1fb67131c2632afb72ba3de19b68a4c5/tools/validation/browser/web-focus-fixture.html#L64) · [游戏状态观察](https://github.com/He-Jiahui/ZirconEngine/blob/ba32996f1fb67131c2632afb72ba3de19b68a4c5/zircon_app/examples/web_focus_fixture.rs#L61)

浏览器输入驱动还向画布发送 `a` 按下，实际焦点移动到同页按钮后向按钮发送 `a` 松开。两个键盘事件和画布 `blur` 的 `isTrusted` 均为 `true`。游戏读回键码 65 和逻辑键 `a` 的 `held` 在失焦帧、松开帧和空帧均为 `true`，对应的 `just_released` 均为 `false`。同一实例回焦再按 `a` 时也缺少新的按下边沿。

## 8. 画布切焦后的状态

提交 `23e2c2a5def64da0f62153127f05ca391ae2ed94` 让原画布的键盘焦点变更进入该画布自己的输入队列，帧提交保留按下、失焦、再次按下的先后顺序。原窗口级失焦仍使键盘和鼠标一起释放。[画布监听](https://github.com/He-Jiahui/ZirconEngine/blob/23e2c2a5def64da0f62153127f05ca391ae2ed94/zircon_app/src/web/listeners.rs#L138) · [逐帧输入提交](https://github.com/He-Jiahui/ZirconEngine/blob/23e2c2a5def64da0f62153127f05ca391ae2ed94/zircon_runtime/src/browser/input.rs#L34)

同一份 21 帧 Chrome 157 用例由 13 帧达到预期提升为 **21/21**。原画布失焦当帧报告 `KeyCode(65)` 和 `Key("a")` 各有一次释放边沿，鼠标左键继续按住，光标与 IME 请求计数均为零。按钮随后收到了 `keyup`，回到原画布的下一次 `keydown` 产生新的按下边沿。双画布互换焦点、同帧重新按下、普通按放和原窗口全量失焦均达到预期。[固定浏览器用例](https://github.com/He-Jiahui/ZirconEngine/blob/23e2c2a5def64da0f62153127f05ca391ae2ed94/tools/validation/browser/web-focus-fixture.html#L64) · [游戏状态观察](https://github.com/He-Jiahui/ZirconEngine/blob/23e2c2a5def64da0f62153127f05ca391ae2ed94/zircon_app/examples/web_focus_fixture.rs#L61)
