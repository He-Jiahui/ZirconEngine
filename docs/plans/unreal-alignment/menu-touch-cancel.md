# Runtime 输入：触摸取消不应激活可见的组件后备菜单

**问题定义 Draft；源码见证，不是实跑 RED。** ZirconEngine 固定到公开 `e89349ca51b4e9d024c194cd3bdf645bd25b6c37`。只讨论已准入的触摸 `Started → Cancelled`，且组件后备菜单确实正在显示；不依赖丢失 Down、鼠标 Up、不可见命中、切换 viewport 或 Editor。未运行 ZE、UE、设备测试或 Cargo，未改生产。

## 1. 同一手势的 Cancel 和 End 被错误合流

[ABI 定义][phase] 把 `Started=1`、`Ended=3`、`Cancelled=4` 分开。[`RuntimeDynamicSession::handle_touch`][touch] 校验坐标/阶段，原样提交 `CursorMoved` 与同 ID、同坐标、同阶段的 `InputEvent::Touch`；`Ended` 选 retained `Up`，`Cancelled` 选 retained `Cancel`。这两层都没有错误吞掉取消阶段。问题在接下来的后备路径：[`dispatch_runtime_ui_pointer`][empty] 遇到空 authored surface set 确定返回 `Ok(false)`（不是 Retained 控件误消费）；`handle_touch` 随后把 `ENDED | CANCELLED` 都送入 [`handle_released(LEFT)`][release]。后者根据**终态当前位置**命中可见按钮，同步在当前 World 写 `gameplay.control_state`。`Started` 在按钮内没有写命令，也没有启动摄像机左键行为，因而不能把结果归给 Down。

[后备显示入口][extract] 在没有 authored submission 时使用 [fallback extract cache][cache]；[菜单投影与命中][menu] 共用 `menu_layout`、同一 `gameplay.menu_state` 和同一按钮矩形。`"start"` 命中写 JSON 字符串 `"start_game"`，`"game_over"` 写 `"retry_game"`。即使让显示路径和命中路径严格使用同一所有者，**可见**按钮里的 `Cancelled` 仍会误写；那是与隐藏后备层仍可点击不同的合同。对于无 `ui_roots` 的项目，生产[明确返回空 surface set][project-ui]，不是损坏或未加载的 `.zui`。

## 2. 生产入口、可达条件和影响

- **真实 Wayland 入口。** [锁定 Winit 及 Wayland backend][lock] 为 `0.31.0-beta.2`；[Winit Touch 合同][winit-touch] 将未先发 `PointerButton(Released)` 的同指 `PointerLeft` 定义为取消。锁定的 [Wayland backend][winit-wayland] `src/seat/touch/mod.rs:18–72,184–214` 在 `wl_touch::down` 记录指头位置并发 `PointerButton(Pressed)`；真实 compositor `wl_touch::cancel` 从活动触点保存的表面和最后坐标发**带 `Some(position)`** 的 `PointerLeft(kind:Touch(id))`，不发 Released。一指、窗口仍存在、最后坐标仍在按钮内即可走该支路，不要求真的移出窗口。该 crate 包 SHA-256 `8ce5afb2ba07da603f84b722c95f9f9396d2cedae3944fb6c0cda4a6f88de545` 与锁文件相符。ZE [client/platform features][client] 给默认 client 接入 Wayland；[真正 event loop][loop] 运行 `RuntimeEntryApp`，主窗 [`PointerButton` / `PointerLeft` 路由][win-events] 分别经[触摸按钮适配][button]产生 `Started`（`Released` 才是 `Ended`），经[离开适配][cursor-left]先发成功的 `CursorLeft`、再发同指 `Cancelled` 和坐标。这里不是旧 Winit `WindowEvent::Touch(Cancelled)` 枚举写法。普通 Lift 会先发 Released 再发 Left（已核 Win32 backend 也如此），不能拿这组序列冒充真正取消；本文不声称已在 Windows 触屏上证实系统取消。
- **Session 与准入。** [App 转发][app-dispatch] 走当前 [Runtime ABI 会话][app-abi]；[FFI][ffi] 验版本、保留 ID/phase，并由 [session registry][registry] 在 live owner 上分派并取回结果。Runtime 只处理默认 viewport、非 detach/gap、未因 reload 中断的输入，[InputDriver][admit] 还要求当前附件、健康队列、预算及合法接收时限；[session 仅在完整物理 batch Accepted 时进入 UI/fallback][submit]。正常 `CursorLeft` 只提交窗口离开（[事件分派][event-route]），不是 `FocusLost`。因容量拒绝等而未到达 fallback 的运行不能算重现，即使顶层返回 `Ok`。输入管理器的[本来行为][active] 是清除当前指头，仍保留正确的取消事件，而非制造 MouseLeft release。
- **移动边界。** 锁定的 Winit [Android backend][winit-android] `src/event_loop.rs:406–446` 与 [UIKit backend][winit-uikit] `src/view.rs:574–615` 对普通手指取消也会给带坐标的 `PointerLeft`、不先给 Released；这是库后端行为，不是对 ZE 移动产品覆盖的承诺。ZE 生成的 [Android][mobile-android] `ACTION_CANCEL` 和 [iOS][mobile-ios] `touchesCancelled` 确实产出独立 phase 4，但相应 [`zircon_export_handle_touch`][mobile-bridge] 目前只返回 composition 是否 Running，不把 ID、phase 或坐标送入本会话；不能据这些模板声称移动导出应用能复现，也不并入这个桥的缺失。Web 的浏览器路径不是本问题的 dynamic session 见证。
- **影响限定到这次真实写入。** 若有已正确加载的消费者，Vampire [源脚本][vampire] 在 `onUpdate` 读这两个命令、重置位置/生命等、转入 playing，清菜单和命令。该脚本仅说明语义与可能后果；不声称 core-only、默认 `.exe` 或移动导出会装载真实 VM，更未实跑游戏。无需 VM 即可见证 `gameplay.control_state` 错写。

## 3. ZE 自身已明确规定：取消清理，而非激活

[Retained `activation_phase`][activation] 只把主键 `Up` 映为 `PrimaryRelease`；`Cancel` 没有激活相位，[释放捕获分支][ownership] 清除该指的 press/capture 并清理不再有捕获的 drag。现有 [ZE 真 retained 测试][retained-test] `unified_touch_cancel_routes_to_capture_and_releases_pointer_capture` 明确断言 `UiPointerCaptureReason::Cancel`、不发 `Commit("activated")`、pressed/capture 清除（测试直接预设 capture，非游戏命令测试）。这里应保留已发生的物理和 retained 清理，仅不把 `Cancel` 转为后备菜单命令。

## 4. Unreal 对照及证据边界

本机参考 `Engine/Build/Build.version:1–10` 自报 6.0.0、CL 0，不能推定 UE 上游提交。**Slate 的捕获取消与完成操作分开**：`Engine/Source/Runtime/Slate/Private/Framework/Application/SlateUser.cpp:299–317` 用 user/index 移除 captor，再通知 `OnMouseCaptureLost`。`Engine/Source/Runtime/Slate/Private/Widgets/Input/SButton.cpp:564–579,598–606` 的 capture-lost 仅 `Release()`、清 `bIsPressed`、可发 `OnReleased`，并不 `ExecuteOnClick()`；正常 `OnMouseButtonUp` 才按按压/触屏/位置/捕获及 click method 判定执行 Click（同文件 `403–443`）。这里的 **OnReleased 不等同 OnClicked**。这是行为分层参照，不是平台输入等价性：此参考树 iOS `Engine/Source/Runtime/ApplicationCore/Private/IOS/IOSView.cpp:662–670` 将 `touchesCancelled` 交为 `TouchEnded`；Android `Engine/Source/Runtime/Launch/Private/Android/LaunchAndroid.cpp:1236–1252` 也把 `ACTION_CANCEL` 与 `UP` 并为 `TouchEnded`。因此不能声称原生 UE 移动端对同一 OS 手势一定不激活。

真实原测：`Engine/Plugins/Tests/AutomationDriverTests/Source/AutomationDriverTests/Private/AutomationDriver.spec.cpp:932–939,981–988` 对 `KeyD` 左键 `Press → Release` 断言序列仅 `D`；`…/SAutomationDriverSpecSuite.cpp:430–441` 确认其真实目标是绑定 `OnClicked` 的 `SButton`。这只证明上游对**正常完成**有测试，限域未找到同用例的 `CaptureLost`/OS `TouchCancelled` 自动化断言，未移植或运行 UE 测试。UE 原文件 SHA-256：

| UE 根相对文件 | SHA-256 |
|---|---|
| `Engine/Build/Build.version` | `3dcd8b6654872d013f2b00aa5f0ef1e0db89d4e8627ce820554675a27f228ea6` |
| `Engine/Source/Runtime/Slate/Private/Framework/Application/SlateUser.cpp` | `7890af0636dc0562223baed4c1a26e74a275e8f1aa268ba9d060b0eff8a93ae6` |
| `Engine/Source/Runtime/Slate/Private/Widgets/Input/SButton.cpp` | `a1f82805aef162f42493668ed963ae0f2e15dffdcc72b4ffe471d7aa11949b1d` |
| `Engine/Source/Runtime/ApplicationCore/Private/IOS/IOSView.cpp` | `657cda82dd79b5946a5a571499efa8667186a3880311544e26e9ea3cf5ee5580` |
| `Engine/Source/Runtime/Launch/Private/Android/LaunchAndroid.cpp` | `a17b87daa533fc7324b4201ed6548cbc38c514d9ab7fb58a31f41379d22775ab` |
| `Engine/Plugins/Tests/AutomationDriverTests/Source/AutomationDriverTests/Private/AutomationDriver.spec.cpp` | `62a0e0a384be81fae4e10ec29925fd6058332fe850842daed0aa853163422da6` |
| `Engine/Plugins/Tests/AutomationDriverTests/Source/AutomationDriverTests/Private/SAutomationDriverSpecSuite.cpp` | `3ce1a412a68316cedd113bf579f0bf9e147b76c97a5ea45b46571a87eeee3d82` |

## 5. 最短可复核复现与对照（待写测试、待实跑）

1. 在现有 [session 物理触摸测试叶][session-test] 用 `RuntimeDynamicSession::new(Headless, None)`、默认 viewport 1、resize 到 `640×360`。该 boot [无项目时用空 authored surfaces][construction] 并[建立真实输入附件][attachment]。通过当前 Level 的 World 建一个实体，写唯一 `gameplay.menu_state={"state":"start"}` 或 `{"state":"game_over"}` 与 `gameplay.control_state="none"`，断言实际写入且允许同类型字符串（[动态组件声明和校验][world]），不启 VM。断言 [session 实际 UI submission][extract] 有 `runtime.gameplay.menu` 的 Start Game / Retry 及对应按钮；[几何][geometry] 在这个尺寸是 `(180,220)`、`280×48`，点 `(320,244)` 严格在内部。只有生产菜单确实投影、组件真实可写才进入下一步。
2. 在同一 live session 上用生产 [`ZrRuntimeEventV1::touch`][wire] 编码默认 viewport 1、同一指 `17`，先 `STARTED@(320,244)`，再 `CANCELLED@(320,244)`；可在第二事件前加真实宿主使用的 `CursorLeft`，不追加 `Ended`。核原始 batch 两次确已接纳：读 InputManager 的事件和活动触点（Started 后存在、Cancelled 后清除）；不能只判 status 成功。立即读取实体命令（不让脚本消费）。**合同**：Start 前后保持 `"none"`，Retry 同理。**未改源码预测**：Started 后仍为 `"none"`，取消后分别是 `"start_game"`、`"retry_game"`。
3. 对每个菜单状态用独立干净会话跑正控 `Started → Ended`（同一点，必须得到匹配命令）；负控 `Started@(320,244) → Cancelled@(8,8)` 保持 `"none"`，以及仍保留有效 `Ended` 的现有 [authored retained session 测试][authored-test]。另可检查 `Started → Cancelled → 新同指 Started → Ended` 的清理和之后正常激活。最小门在 [Runtime 自有 `dynamic-api` feature][feature] 的 `zircon_runtime --lib` 窄 filter（native-host，test profile，不启真实 VM 插件）；第一步只保证源级预期，先取匹配提交的行为 RED，再提交生产变化。原有 [menu helper 测试][menu-test] 借真实 World 分别验证提取与直接写入，却绕过触摸；[session 原 touch 测试][session-test] 验实际 phases 未配菜单；[batch 拒绝测试][budget-test] 验原子准入，不检验已准入取消的菜单命令。

## 6. 归属、去重与修复约束

本 PR 只提交独立调查。发布前检查全部开放 PR 文件及历史标题、正文：77 个 open PR、449 项文件关联未命中本题生产 `events.rs` / `menu.rs` 及触摸测试叶；211 项全态标题、已有 fallback 定向候选和固定原测试中未见同一见证。这是快照去重，不替代发布前重查。[#151](https://github.com/He-Jiahui/ZirconEngine/pull/151) 是浏览器键盘按/放身份；[#168](https://github.com/He-Jiahui/ZirconEngine/pull/168) 是 Editor 中真实 Up 没到原 Play 手势 owner，本题取消已经到 Runtime；[#209](https://github.com/He-Jiahui/ZirconEngine/pull/209) 是 Simulate gizmo Commit/undo reservation，虽然也谈终态，既非组件菜单，也非触摸阶段。与 authored UI 压住 fallback 后仍命中及裸 MouseUp 问题拆开，这个输入**必须有真实 Started、没有 authored UI、当前按钮确实可见**。

最小候选方向在 `RuntimeDynamicSession`：`Ended` 继续原左键完成策略；`Cancelled` 保持物理批、retained Cancel 和需要的清理，仅不走命令写入入口。不能为了关掉误激活而丢掉 Cancel 或挡住真正 End；不改 ABI、组件数据格式、camera input 策略、Winit、Editor 捕获、显示所有权或生成移动桥。这一路径与其他输入调查共享源码时，等到对应 owner 同意后再更新该独立 Draft，不能搭入其他 PR。

[phase]: https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime_interface/src/runtime_api/constants.rs#L48-L51
[touch]: https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/dynamic_api/session/events.rs#L447-L507
[release]: https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/dynamic_api/session/events.rs#L576-L609
[empty]: https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/dynamic_api/session/events.rs#L616-L636
[extract]: https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/dynamic_api/session/extract.rs#L25-L59
[cache]: https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/dynamic_api/session/ui_extract_cache.rs#L57-L83
[menu]: https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/dynamic_api/session/menu.rs#L47-L112
[project-ui]: https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/dynamic_api/session/project.rs#L376-L393
[lock]: https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/Cargo.lock#L9102-L9239
[winit-touch]: https://docs.rs/crate/winit-core/0.31.0-beta.2/source/src/event.rs#464
[winit-wayland]: https://docs.rs/crate/winit-wayland/0.31.0-beta.2/source/src/seat/touch/mod.rs#184
[winit-android]: https://docs.rs/crate/winit-android/0.31.0-beta.2/source/src/event_loop.rs#406
[winit-uikit]: https://docs.rs/crate/winit-uikit/0.31.0-beta.2/source/src/view.rs#574
[client]: https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_app/Cargo.toml#L110-L179
[loop]: https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_app/src/entry/entry_runner/runtime.rs#L641-L662
[win-events]: https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_app/src/entry/runtime_entry_app/window_events/dispatch.rs#L10-L77
[button]: https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_app/src/entry/runtime_entry_app/pointer_input/button.rs#L14-L31
[cursor-left]: https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_app/src/entry/runtime_entry_app/pointer_input/cursor.rs#L23-L47
[app-dispatch]: https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_app/src/entry/runtime_entry_app/event_dispatch.rs#L12-L31
[app-abi]: https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_app/src/entry/runtime_library/runtime_session.rs#L338-L347
[ffi]: https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/dynamic_api/session/ffi.rs#L192-L249
[registry]: https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/dynamic_api/session/registry/session_store.rs#L281-L319
[admit]: https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/input/runtime/input_driver.rs#L213-L271
[submit]: https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/dynamic_api/session/state.rs#L601-L655
[event-route]: https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/dynamic_api/session/events.rs#L147-L178
[active]: https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/input/runtime/default_input_manager/apply_event.rs#L108-L122
[mobile-android]: https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/plugin/export_build_plan/platform_host_files/mobile.rs#L347-L415
[mobile-ios]: https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/plugin/export_build_plan/platform_host_files/mobile.rs#L739-L805
[mobile-bridge]: https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/plugin/export_build_plan/platform_host_files.rs#L212-L225
[vampire]: https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/examples/vampire/scripts/vampire_game/main.zr#L42-L67
[activation]: https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/ui/surface/surface/event_routing.rs#L626-L652
[ownership]: https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/ui/surface/surface/event_routing/pointer_ownership.rs#L147-L168
[retained-test]: https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/ui/tests/input/touch_pointer_routes.rs#L156-L241
[session-test]: https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/dynamic_api/session/tests/pointer_frame_batch.rs#L181-L229
[construction]: https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/dynamic_api/session/construction.rs#L387-L426
[attachment]: https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/dynamic_api/session/construction.rs#L427-L442
[world]: https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/scene/world/dynamic_components.rs#L215-L259
[geometry]: https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/dynamic_api/session/menu.rs#L150-L172
[wire]: https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime_interface/src/runtime_api/session/events.rs#L432-L447
[feature]: https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/Cargo.toml#L138-L145
[authored-test]: https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/dynamic_api/session/tests/runtime_ui_surface.rs#L495-L539
[menu-test]: https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/dynamic_api/session/tests/menu.rs#L157-L215
[budget-test]: https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/dynamic_api/session/tests/physical_input_ownership.rs#L262-L328
