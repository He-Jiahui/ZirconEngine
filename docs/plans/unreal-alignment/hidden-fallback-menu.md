# [Runtime/UI] 项目 HUD 已取代后备菜单，旧按钮矩形仍可发出游戏命令

**独立调查，未改产品、未运行本案 ZE/UE 回归。** 固定 ZE `e89349ca51b4e9d024c194cd3bdf645bd25b6c37`。目标是让输入与呈现对“当前使用哪一套 UI”有同一个答案；不是改变按钮的 MouseUp/DownAndUp 习惯，也不改游戏生命周期。

## 1. 产品合同与差异

[Session 合同](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/docs/crates/zircon_runtime/dynamic_api/session.md#L578-L580)区分两个来源：项目自己的 retained UI，以及没有项目 UI 时才选用的组件后备 overlay。[实际提交](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/dynamic_api/session/extract.rs#L25-L50)在成功取得项目 UI 时直接返回，**不合成** World 中的 Start/Retry 菜单。[surface 选择和绘制提交](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/dynamic_api/session/runtime_ui.rs#L196-L255)以已加载的 surfaces 判断，而非光标位置、像素透明度或控件数。

反之，如果当前项目 UI 不消费点击，[鼠标分发](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/dynamic_api/session/events.rs#L327-L375)会继续到[组件菜单命中及松键处理](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/dynamic_api/session/events.rs#L576-L604)；后者不看当前呈现来源。只要光标位于旧按钮矩形，[菜单命令写入](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/dynamic_api/session/menu.rs#L57-L93)就可向属于该 World 菜单实体的 `gameplay.control_state` 写入 `start_game` 或 `retry_game`。物理按钮仍正确松开；问题是**没有出现在当前 UI 提交中的按钮仍能激活**。

## 2. 合法见证与有界影响

在同一运行 Level 留有合法 `gameplay.menu_state.state="start"`（另一变体 `game_over`），同时在 manifest 声明一个可成功导入的 `.zui` View：小块可见 Text/HUD，`layout.input_policy="Ignore"`。[属性解析](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/ui/v2/surface_tree/layout.rs#L463-L474)和[命中登记](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/ui/tree/hit_test.rs#L359-L379)支持这种只画不命中的用法。先成功发布有 HUD 文字、无后备按钮的提交，在 640×360 视口的 (320,248) 同点左键按下并松开；该点落入[原后备按钮矩形](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/dynamic_api/session/menu.rs#L150-L171)。没有任何新窗口、丢失的 Up、触摸取消、异步换项目或模拟超时。

此时应只记录物理输入，命令保持 `none`。旧源码推导却会写入游戏‘开始’/‘重试’命令。[现有真实脚本消费者](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/examples/vampire/scripts/vampire_game/main.zr#L19-L67)在收到这些字符串时会重新初始化游戏状态，所以写入并非一个无人使用的调试标志。原 Vampire 项目并非预配此 HUD；这里是受支持能力的组合，不声称已在原样例或独立发布的主窗口运行。

## 3. 根因与本机 UE 原测

[项目指针分发](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/dynamic_api/session/runtime_ui/input_routing.rs#L333-L379)在没有捕获、且命中候选为空时返回 `false`。它表示**当前来源未处理**，而不是“另一个未呈现来源现在拥有这个按钮”。即便只有一个已加载的 surface，它也可有绘制内容而没有命中项。旧代码把‘未处理’与‘后备菜单可用’混为一件事。

UE（本机参考树 `6.0.0 / UE5 / CL 0`，上游 revision 未知）：`FHittestGrid::AddWidget`（`Engine/Source/Runtime/SlateCore/Private/Input/HittestGrid.cpp:859–865`）排除不可命中的控件；`LocateWidgetInWindow` 与无捕获的 `RoutePointerUpEvent`（`Engine/Source/Runtime/Slate/Private/Framework/Application/SlateApplication.cpp:1990–2014`；`Engine/Source/Runtime/Slate/Private/Framework/Application/SlateApplication.cpp:5676–5732`）走当前路径。`FAutomationDriverSpec` / `TestFramework.Driver` 的 `Element.IsVisible` 真实测试把 Piano 设为 `Hidden` 后断言不可见（`Engine/Plugins/Tests/AutomationDriverTests/Source/AutomationDriverTests/Private/AutomationDriver.spec.cpp:1053–1066`），同一套件 `Element.Click` 可见 KeyG 正控验证回调序列（`Engine/Plugins/Tests/AutomationDriverTests/Source/AutomationDriverTests/Private/AutomationDriver.spec.cpp:468–474`）。这是可见性及路由对照，**不是 UE 已有与 ZE 这套双来源模型等形的测试**；不声称所有不透明、透明节点或捕获都适用同一规则，不移植 Slate。

## 4. 可借用的 Rust 做法

只在 Session 判断本次是组件后备来源时才探测/提交其菜单动作。[已加载 surface 集合](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/dynamic_api/session/runtime_ui.rs#L107-L121)比 raw manifest 是否列出根更接近真实的呈现准入；项目声明根但驱动没接纳、或构建失败时，不得伪造 project-owner。不要每个输入都建一份渲染结果，不增 ‘最近绘制’ 的全局缓存，也不能简单地把所有 `Unhandled` 改成 `Handled`：摄像机、正常物理输入和项目自己未处理的非菜单行为仍须工作。

## 5. 接口、ECS 和可达范围

[实际项目 UI 准入](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/dynamic_api/session/project.rs#L376-L410)独立于 World 的组件状态。正式[.zui 插件导入器](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/ui_document_importer/runtime/src/lib.rs#L31-L60)能生产 View 文档；调用者须具备其注册。[公开 linked Runtime / Play 入口](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/dynamic_api/session/linked_session.rs#L13-L46)接纳真实注册；启用 `editor-scripting` 的 [App Play 工厂](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_app/src/entry/entry_runner/editor/play_session_factory.rs#L78-L122)会重新解析 ClientRuntime 所选插件并传给[同一会话 owner](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_app/src/entry/runtime_library/runtime_session/linked_play.rs#L8-L28)。问题范围是成功加载 HUD 和 Level 的 Runtime session。默认 core-only V8 DLL 启动并不因为另一个 crate 编入了插件就自动获得这些 Rust 注册，不把普通 standalone、实际 HWND 或设备列为已证。

不用变更 C ABI、Winit DTO、物理按放记录、World 或脚本信箱的所有权。保留没有 project surface、后备菜单**确实可见**时的 release 激活策略。‘按在外面、放在里面’本身不立项；它可能是上述文档允许的行为。

## 6. 同一 PR 的回归门和去重

先用无 authored UI 的同类工程确认：取得原后备按钮命令，真正按放后 Start/Retry 命令均能发布。再在有 passive HUD 的会话确认实际 UI 提交中有 HUD 而无后备按钮，发送同一对有限坐标、同视口的物理 ABI 事件。记录物理 held 的 true→false、按放 journal、同一实体的命令；遇到无 importer、空 HUD、初始化失败、错坐标或事件被拦不能算本问题 RED。保留界外点击和普通相机操作正控；修复只在见证到达该错误写入后做。[原菜单单测](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/dynamic_api/session/tests/menu.rs#L185-L215)只从正确几何取 action 并直接写 World；[原项目多根绘制测试](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/dynamic_api/session/tests/runtime_ui_surface.rs#L309-L346)只验证项目 UI，各自都没有让两套来源同时存在。

与 [#168](https://github.com/He-Jiahui/ZirconEngine/pull/168) 的宿主捕获后 Up 未回到原 Play 相反，本例接收到了正确 Up；[#151](https://github.com/He-Jiahui/ZirconEngine/pull/151) 是 Web 键值， [#209](https://github.com/He-Jiahui/ZirconEngine/pull/209) 是 Simulate Gizmo 释放提交。也不合并另一个 `Touch.Cancelled` 是否能激活**可见**后备按钮的议题。产品后续门是在具备真实插件注册的正常 Play 中比较可见帧与相同输入的游戏状态，本次没有运行或代签。

UE 参考指纹（SHA-256，仅列文件身份和上述行号，不发布 Epic 源码）：

- `Engine/Source/Runtime/SlateCore/Private/Input/HittestGrid.cpp`：`e1df5b8781a9584b1f625830a0f4d98359238cf49acf0508551ab519ae6c2c77`。
- `Engine/Source/Runtime/Slate/Private/Framework/Application/SlateApplication.cpp`：`06f96167fd20bde540b7f60629b53c8cfbddd3ae4b7d93368b9bc4f9f45cfe99`。
- `Engine/Plugins/Tests/AutomationDriverTests/Source/AutomationDriverTests/Private/AutomationDriver.spec.cpp`：`62a0e0a384be81fae4e10ec29925fd6058332fe850842daed0aa853163422da6`。

## 7. 四个真实 Session 回归已提交；当前编译前置不属于 UI

[tests-only `336f492231e3bed4e2f8b76773020a702413fb53`](https://github.com/He-Jiahui/ZirconEngine/blob/336f492231e3bed4e2f8b76773020a702413fb53/zircon_runtime/src/dynamic_api/session/runtime_ui/tests/menu_owner.rs)。四例用生产 `RuntimeDynamicSession` / Level。在同一实体写入真实 `gameplay.menu_state` 与 `gameplay.control_state` 后，从真正提交的 fallback quad 获取中心坐标，不用硬编码直接触发动作。两个 HUD 用例经生产 `.zui` 解析和 retained 编译器构建 Text/Ignore；测试只代替项目导入入口，并不证明 standalone 已加载工程。它们检查独占项目 UI 的实际提交、该点在已发布输入目录的零候选、焦点/捕获、物理鼠标按放与 World 里的命令。另两例保留可见菜单与界外操作/相机拖放。真实产品还须让当前 World 接纳这两个游戏组件并注册 `.zui` importer，单给原样例附上一个 UI 根不是完整准入证明。

已实际调用 Windows x64/MSVC Rust 1.94.1、`test` profile：`-p zircon_runtime --no-default-features --features dynamic-api,input-mouse --lib fallback_menu_owner_ --locked --offline --jobs 1 -- --test-threads 1 --show-output`。Cargo exit **101**，在整个 Runtime 库测试编译时止于未修改的[图形模型夹具:305–315](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/graphics/scene/resources/tests/model_geometry_resolution.rs#L305-L315)，`E0716: temporary value dropped while borrowed`。上面借用从临时 registry 得到的记录，稍后比较时 registry 已失效。该文件在共同 main、此次 tests-only commit 和实际工作文件是同一 blob `b8c738dae9ec9fa9a40abb41f3cbd7dae96c6723`；实际 stderr 只报这一项编译错误。原生回执 `d83af5f78e07ba2a79684939c7e312a46f5705619a01f030e64e6e3255b0c92f`，stdout 为 **0 字节**：**四个用例均未开始**。这不是隐藏按钮行为的 RED，也不是对候选修复的 GREEN。

这项构建前置已有独立 [#137](https://github.com/He-Jiahui/ZirconEngine/pull/137)，没有合入、复制、临时替换该修复或屏蔽相关测试；本 PR 只增加测试及测试挂载和该说明。下一个有效门是可编的原 owning 基线，或以 Runtime 作为普通库、通过正式 linked 会话在公开 integration target 验证真实 HUD 与命令。两种门的证据范围必须分别标明；不改 MouseUp 合同或触摸取消 [#213](https://github.com/He-Jiahui/ZirconEngine/pull/213)。

## 8. 公开 Runtime 入口实测

[同一版本的独立集成测试](https://github.com/He-Jiahui/ZirconEngine/blob/024df17925229c22e1dbfb9f7f5562381ab9c968/zircon_plugins/ui_document_importer/runtime/tests/hud_menu_public.rs)。通过实际已注册的 UI document importer 读取工程 `.zui`，正式 V8 a11y snapshot 证项目树已加载且保持原身份；鼠标按放都由真实 owner 录制，随后从原实体读取控制槽。无根同点 Start/Retry 与界外控制通过。两个有效项目 HUD 用例都跨过 admission、指针和 owner 正控后，才在旧按钮命令不应变更处失败。

原版独立 WGPU 首帧进程在 15 分钟上限内没有返回已核验的 framebuffer；这是受限中止，不计作通过或断言失败。没有通过旧按钮/新 HUD 像素前置，不据此认定可见画面。 GPU 原执行回执 `81cf649bd0b54a026a8de949d34718cac2af682ac8b6ba26351c09bb456fc229`。

完整 test target `hud_menu_public`：Windows x64/MSVC 1.94.1，`zircon_plugin_ui_document_importer_runtime`、defaults off、`zircon_runtime/dynamic-api`、`test` profile，先编译唯一集成目标，再运行 Cargo 指出的该二进制。真实结果 **1 过、2 败**；编译回执 `82e25476b4d1d4e5b5b488197796558ce2ffec057b8fd90ab4a9eb6ae11cf3e8`、exe SHA-256 `3db93298ebc74302e044a56ec982f3d90566e21b0c40465b664dbc172bb88058`、执行回执 `a8c2c5e0e53440b9f562633f10fed29dab1402fee7dd34ed74b4d0e30c0aac54`、stdout `eae0967ab5642911c8acd860ee19c3510b5e350a7a90968e2a0291677ce8f517`。夹具、配置、日志分别在物理测试根中，不使用用户工程；测试默认为显式运行。公开 linked 会话（无 OS 真机设备，不是屏幕或 Winit 事件泵验收）。

## 9. 同一回归修后配对

[同一版本的独立集成测试](https://github.com/He-Jiahui/ZirconEngine/blob/fe3563ae299455ebff6254c59a700a96386ada44/zircon_plugins/ui_document_importer/runtime/tests/hud_menu_public.rs)。通过实际已注册的 UI document importer 读取工程 `.zui`，正式 V8 a11y snapshot 证项目树已加载且保持原身份；鼠标按放都由真实 owner 录制，随后从原实体读取控制槽。无根同点 Start/Retry 与界外控制通过。项目 HUD 拥有该位置时同点不再写旧菜单命令。 原版两个有根工程在同点分别把 `none@1` 变为 `start_game@2` / `retry_game@2`；修后两者均为 `none@1`。无项目 UI 根的两个正控仍正常触发。仅改变开/关后备命中查询的条件，不改变输入日志、可见菜单松开触发或 #213 的取消分支。

原版独立 WGPU 首帧进程在 15 分钟上限内没有返回已核验的 framebuffer；这是受限中止，不计作通过或断言失败。没有通过旧按钮/新 HUD 像素前置，不据此认定可见画面。修后只复跑 CPU；没有复用旧 GPU 作为修后结果。 GPU 原执行回执 `81cf649bd0b54a026a8de949d34718cac2af682ac8b6ba26351c09bb456fc229`。

完整 test target `hud_menu_public`：Windows x64/MSVC 1.94.1，`zircon_plugin_ui_document_importer_runtime`、defaults off、`zircon_runtime/dynamic-api`、`test` profile，先编译唯一集成目标，再运行 Cargo 指出的该二进制。真实结果 **3 过、0 败**；编译回执 `bfbbfc2df5004ffc06c21fcfab98dae23ee3ab93245d7af307111612b8ee8ac9`、exe SHA-256 `71e4740d335c6c697c9844d511a01a871bef46cc5da1a4bf708497a2a7c9a5a6`、执行回执 `b0992571abfba4f845ae406e90bfc6508961a1d3276482265ca6e900d61e040b`、stdout `27a2f61885c3032733fc86bd1ac360626c293efd7a84fd7d51df0637c644b4c8`。夹具、配置、日志分别在物理测试根中，不使用用户工程；测试默认为显式运行。公开 linked 会话（无 OS 真机设备，不是屏幕或 Winit 事件泵验收）。
