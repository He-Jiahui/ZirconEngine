# 输入系统：浏览器修饰键变化后释放错逻辑身份（独立 Draft 问题草案）

> 取证说明：UE 的本机路径与文件 SHA256 是用户提供参考树的依据；本轮 D: 审计快照/工作记录仅本地保存，不随本 PR 发布，也不冒充 GitHub 可读证据。UE/ZE 测试均未在本轮执行。

- 状态：**源码确定的问题；建议仅据本文开一个问题定义 Draft。** 本轮不改实现、不补测试、不运行 Cargo/UE/浏览器测试、不操作远端。
- 审查源：共享根 `HEAD=cb495bb1c952a24c86f4513b8099bc4942c3a87d`；父任务确认可公开基线为 `e89349ca51b4e9d024c194cd3bdf645bd25b6c37`。只读 `git diff e893… cb495… -- zircon_runtime zircon_plugins zircon_runtime_interface zircon_app zircon_editor --stat` 为空，因此下文 ZE 链接固定到可访问的 e893，不夹带本地退役流程提交。
- UE：本地 [Build.version:1–9](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Build/Build.version#L1-L9>) 为 6.0.0 / BranchName UE5；无独立 UE revision 承诺，以文末原文件 SHA-256 冻结。
- 已读规范：规范根与报告工作树的 AGENTS、zircon-engineering / zircon-dev、checkout policy、milestone validation policy。保留共享根他人脏技能文件；检查相关 owner 祖先路径未发现附加 AGENTS。未恢复旧协调器/Jenkins。

## 1. 差异、精确 owner / 符号与生产可达性

**主缺陷在 Web producer：每次把当前 DOM 字符写成 held 身份，未满足公开事件要求的按放名称配对契约；不是 manager 集合操作错误。**

- 生产宿主 [Web监听器:99–118](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_app/src/web/listeners.rs#L99-L118>)，`Listeners::new` 的 keydown/keyup 闭包每次取 `event.key()`，直接写入 `KeyboardInput.logical_key`；`event.key_code()` 则仍为同一键码。没有保存按下身份。
- 状态 owner [apply_input_event:48–69](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/input/runtime/default_input_manager/apply_event.rs#L48-L69>)，按下 `Key(logical_key)`，释放只 `release(Key(本次logical_key))`；物理 `KeyCode` 与逻辑 `Key` 不关联。
- 这不是“也许下层会统一大小写”：[InputButton:15–29](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/core/framework/input/input_button.rs#L15-L29>) 明确使用完整 String、大小写不折叠；[ButtonInputState::release:47–63](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/core/framework/input/button_input_state.rs#L47-L63>) 只移除精确匹配项；[BrowserInput::push:34–79](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/browser/input.rs#L34-L79>) 只做长度约束/队列准入，不改字符身份。

真实路径（不是仅测试构造的 evaluator）：

```text
zircon_app/src/lib.rs:78–83  target-web + wasm32 + !native-host
 -> web/mod.rs:18–36  WebEngine::load -> Listeners::new(canvas, engine.input())
 -> web/listeners.rs:99–118  DOM keydown / keyup -> BrowserInput::push
 -> zircon_runtime/src/browser/frame.rs:49–66
      begin_frame -> BrowserInput::drain -> InputManager::submit_event -> Level::tick
 -> input/runtime/default_input_manager.rs:124–131 -> apply_input_event
 -> ButtonInputState<Key/KeyCode> -> InputManager::button_pressed / frame_snapshot
```

对应调用点：[WebEngine::load:18–36](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_app/src/web/mod.rs#L18-L36>)、[BrowserEngine::frame:49–66](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/browser/frame.rs#L49-L66>)、[准入调用:124–131](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/input/runtime/default_input_manager.rs#L124-L131>)、[直接查询:166–178](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/input/runtime/default_input_manager.rs#L166-L178>)。

在同一 canvas 持续聚焦、无队列溢出、无 visibility change 的普通 Shift+A 顺序下，DOM 的 `key` 随修饰键状态变化：

| 事件输入 | 源码写入/移除 | 当前源码推导结果 |
|---|---|---|
| Shift 按下，key=`Shift`，keyCode=16 | 按下 KeyCode(16)、Key("Shift") | 两者 held |
| A 按下，key=`A`，keyCode=65 | 按下 KeyCode(65)、Key("A") | 两者 held |
| 先松 Shift | 移除 KeyCode(16)、Key("Shift") | A 的两项仍 held |
| 再松 A，key=`a`，keyCode=65 | 移除 KeyCode(65)，尝试移除未按下的 Key("a") | **Key("A") 仍 held，且没有其 release edge** |

此表是可逐分支复核的预测，**不是已运行的浏览器复现**。相同输入也可经现有公开 `KeyboardInput` 入口到达。四条事件远低于 1024 队列容量；[失焦监听:138–145](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_app/src/web/listeners.rs#L138-L145>) 不会在“仍聚焦、仅释放 Shift”时自动触发。[begin_frame:91–108](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/input/runtime/default_input_manager.rs#L91-L108>) 只清边沿，不清 held，不能自愈。

## 2. 有界影响

影响 `target-web` 中通过逻辑 `InputButton::Key` 查询/绑定该按下身份的消费者：所有物理键已松开，仍可读到旧逻辑键 held，直到 FocusLost 等显式清理；连续帧亦保留错误。物理 `KeyCode(65)` 正常释放，**不宣称所有键盘控制或桌面宿主都卡住**。

当前命名动作 evaluator 的产品调用缺口已由旧计划登记，不据此虚构 action 产品后果。本问题已在可达的物理/逻辑状态服务成立，无需先把 evaluator 接入产品。没有运行脚本/产品验证，也不把大小写不同直接等价为 UI 文本丢失。

## 3. 根因与 UE 的解决方式

根因是把“本次可打印/布局逻辑值”直接当成跨事件 held 键，而没有建立 release 对应的稳定控制身份。缺少配对，不是 `BTreeSet` 的集合操作错误。

已读 UE 对应模块依赖：

| 模块 | Build.cs 所确认边界 |
|---|---|
| [ApplicationCore:10–33](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/ApplicationCore/ApplicationCore.Build.cs#L10-L33>) | Core；Windows XInput/平台输入 owner |
| [Slate:18–35](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Slate/Slate.Build.cs#L18-L35>) | InputCore、SlateCore；可选 ApplicationCore |
| [InputCore:5–20](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/InputCore/InputCore.Build.cs#L5-L20>) | Core/CoreUObject，平台键映射 |
| [Engine:73–115](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Engine/Engine.Build.cs#L73-L115>) | InputCore、Slate；可选 ApplicationCore |
| [EnhancedInput:12–25](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Plugins/EnhancedInput/Source/EnhancedInput/EnhancedInput.Build.cs#L12-L25>) | Engine/InputCore/Slate/ApplicationCore |
| [EnhancedInputTestSuite:11–29](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Plugins/EnhancedInput/Source/EnhancedInputTestSuite/EnhancedInputTestSuite.Build.cs#L11-L29>) | 测试模块，Engine/EnhancedInput/InputCore；editor 时才依赖 UnrealEd |

UE Windows 生产链：

```text
WindowsMessageInputDevice.cpp ProcessKeyDown / ProcessKeyUp
 -> MessageHandler OnKeyDown / OnKeyUp(ActualKey, CharCode, repeat)
 -> SlateApplication.cpp OnKeyDown / OnKeyUp
 -> InputCoreTypes.cpp FInputKeyManager::GetKeyFromCodes
 -> FKeyEvent -> viewport/player input -> UEnhancedPlayerInput::InputKey
 -> UPlayerInput::InputKey -> KeyStateMap[Params.Key]
```

关键短引文与精确处：

- [Windows键事件:662–736](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/ApplicationCore/Private/Windows/WindowsMessageInputDevice.cpp#L662-L736>)：up/down 都从 `Win32Key` 得到 `ActualKey`，并用 `MapVirtualKey(..., MAPVK_VK_TO_CHAR)`，不是把当前 Shift 产生的字符代替物理身份。
- [字符事件:588–597](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/ApplicationCore/Private/Windows/WindowsMessageInputDevice.cpp#L588-L597>) 独立调用 `OnKeyChar(Character, bIsRepeat)`。
- [Slate按下:5032–5037](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Application/SlateApplication.cpp#L5032-L5037>) 与 [Slate释放:5157–5162](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Application/SlateApplication.cpp#L5157-L5162>) 采用同一 `GetKeyFromCodes`。
- [键码优先映射:1595–1600](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/InputCore/Private/InputCoreTypes.cpp#L1595-L1600>) 短引文：`KeyMapVirtualToEnum.Find(KeyCode)`；仅未命中才用 CharCode。
- [EnhancedPlayerInput:535–550](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Plugins/EnhancedInput/Source/EnhancedInput/Private/EnhancedPlayerInput.cpp#L535-L550>) 先 `Super::InputKey(Params)`；[PlayerInput:401–459](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/UserInterface/PlayerInput.cpp#L401-L459>) 同一 `Params.Key` 定位状态，`IE_Released` 将 RawValueAccumulator 归零。

这里只借鉴 **稳定控制身份与字符输入分离、release 回到同一键状态**。不声称 UE 拥有当前 ZE 的 WebEngine，也不把其 Windows virtual-key 数值直接写进 ZE Web ABI。

## 4. Rust 如何照 UE 思路实现（尚未实施）

[KeyboardInput 的既有公开契约:163–185](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/core/framework/input/input_event.rs#L163-L185>) 已明确要求提交方保持按放名称匹配，`logical_key=None` 只更新物理码。因此最小修复优先放在 Web 的控制事件提交边界：在 [BrowserEngine::frame:52–59](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/browser/frame.rs#L52-L59>) 已给 UI 消费原 DOM 事件之后、向 manager 提交之前，用既有物理码保存首次 Down 对应的 logical alias，repeat 不替换该 alias；Up 向 manager 提交同一个 alias，再清除记录。实际文本/IME 与 raw UI 当前字符沿原通道保留，不统一大小写，不把 `event.code()` 新命名空间迁移夹带进来。manager/replay 的录制事件是完成配对后的控制事件，不再声称其 raw keyup 同时保留 DOM 的 `a`。

Web 私有配对必须消费 drain 中的所有 FocusLost（包括 [队列溢出自行注入:73–86](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/browser/input.rs#L73-L86>)）并在销毁时清空，不能只让 DOM blur 闭包清理一张看不到队列重置的表；保留当前 focus reset 合同。无匹配 Down 的 Up、缺失或不稳定物理码如何处理，要先在现有 Web 输入约定中明示，不能猜配。跨物理码同 alias 聚合、左右 Shift 同码、旧 ButtonPressed writer 混用属于相邻既有债务/本地未合 R12 提交 `c7c2797ea3c95186215a7c216f9512979595a980` 的 `r12_side_code_with_aggregate_logical_name_keeps_aggregate` 等控制；该提交未改 Web listener/BrowserInput，也未覆盖单码 65 的 A↓/a↑。不新建平行 modifier 任务，不在本问题扩大为全局 manager 重设计。把所有来源的配对改放进 DefaultInputManager 即使不改 wire，仍会改变公开事件行为，必须另审，不作为此次默认修复。

## 5. ABI / ECS / 后端差异与可行性边界

- ZE Web 直接传 Rust InputEvent，经 wasm-bindgen/web-sys，不走桌面 `ZrRuntimeEventV1`；这是独立于 native key hash/scan-code/repeat 丢失的问题。
- `InputButton::Key` 与 `KeyCode` 是既有公开、可序列化身份。只在 Web producer 保持其已承诺的配对，不新增 ABI 字段、不改枚举 wire/action map，也不改变 manager 对 `logical_key=None` 的物理码专用语义。无 wire 变化不能冒充无公共行为变化。
- UE 在 UObject/PlayerInput 每 player 保存 KeyState；ZE 是 Core manager + Level/ECS tick。修复应在输入 owner 完成，不让 ECS 每帧扫描/猜测 release，也不引入 FKey/UObject 仿制层。
- Web 当前 keyCode 是旧浏览器字段；若要改成 `event.code()` 的新稳定命名空间，必须另审 ABI/持久绑定兼容性。本问题不能夹带该迁移。
- 可行性：可在 BrowserEngine 既有 Web 输入生命周期内保存有界私有配对，由 drain→UI→manager 的提交边界统一消费；仍需确认 repeat、FocusLost、销毁和未知物理码政策。native TEXT 当前还会构造 `pressed=false, logical_key=None`（[keyboard_ime.rs:37–56](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/dynamic_api/session/events/keyboard_ime.rs#L37-L56>)），不能为修 Web 而让通用 manager 自动解释所有 None 为别名释放。**只有静态方案，没有实现可行性验证、编译或测试执行结果。**

## 6. 具体修改草案与严格后续顺序（不实施）

1. 先仅以本文开独立 Draft；opening commit 不能包含以下回归或修复。
2. Draft 后先从原 DOM listener 记录真实事件 payload 的 RED，再把其 payload 经现有 BrowserInput→manager harness 检验 held/release；不能只写脱离 producer 的大小写模型，也不能把违反公开配对约定的手造 payload 当作 manager 自身缺陷。保留 `logical_key=None`、native producer、FocusLost 与原 R12 相邻控制。
3. 最小实现 owner 是 [BrowserEngine 现有帧提交阶段:52–59](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/browser/frame.rs#L52-L59>) 的 Web 私有配对/清理；[DOM Listeners:99–118](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_app/src/web/listeners.rs#L99-L118>) 保留原字符载荷，不改 DefaultInputManager 事件语义。现有 [WebEngine/CDP harness:31–68](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/tools/validation/browser/validate-web-memory.mjs#L31-L68>) 可参考 DOM 入口，但其现有 receipt 没有 held 查询：需测试侧观测接线，或将 DOM payload probe 与 BrowserInput→manager 状态断言分层，不能把 receipt 冒称端到端 held 验收。
4. 等价回归在原始源证明 RED 后，才在同一 Draft 试最小修复，并验证原始输入/期望未变；native、gamepad generation、FocusLost 等控制不得弱化。

### 测试对照 / 迁移表

| UE 原有测试名与行号 | 原输入和预期 | ZE 已有落点 / 拟覆盖 | 性质 |
|---|---|---|---|
| [FInputBindingDigitalTest / Input.Binding.DigitalTrigger:38–68](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Plugins/EnhancedInput/Source/EnhancedInputTestSuite/Private/InputBindingTest.cpp#L38-L68>) | 同一 FKey 按下、保持100tick、释放；Completed 后下一tick不再触发 | [input_snapshot_just_pressed_is_true_for_exactly_one_frame:167–195](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/input/tests/input_manager/frame_state.rs#L167-L195>) | 已有单键状态对应；不是 Web Shift 用例迁移 |
| [FInputBindingMultipleKeyValuesTest:102–165](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Plugins/EnhancedInput/Source/EnhancedInputTestSuite/Private/InputBindingTest.cpp#L102-L165>) | 两键驱动同一 action；切换、双按、全松后不再触发 | [action_map_resolves_chords_and_reports_just_activated:15–59](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/input/tests/action_mapping.rs#L15-L59>) 是 chord 而非同 alias；可在现有 action 测试 owner 补贡献控制 | 不等价，不能宣称已迁移 |
| **未找到原 UE“DOM key 随 Shift 改变、up须释放down alias”同例测试** | 同聚焦：Shift↓、A↓(A)、Shift↑、A↑(a)；所有 KeyCode 与按下alias均不held，恰一次真实release；repeat不造幽灵alias | [frame_state 测试owner](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/input/tests/input_manager/frame_state.rs>) + [BrowserInput现有harness:126–165](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/browser/input.rs#L126-L165>)，再做原DOM监听器集成 | **由上述UE生产身份分层推导的新回归，不称迁移；全部未写未运行** |

搜索范围：本地 EnhancedInputTestSuite 的 InputBinding / Integration / System / SystemExtended / Trigger / Modifier 测试、ApplicationCore Windows 输入实现与 Slate key event 路由；ZE input 测试、browser input/UI、Web监听器与工具测试中 keyboard / logical key / repeat / release / FocusLost。未找到不代表整个 UE 所有测试不存在该例。

### 模块覆盖 / 已有测试 / 去重 / 未清事项

- ZE 实际 owner 已查 runtime input + framework contract、browser input/frame/UI、app Web/native producer、dynamic session normalizer，以及 sound 等插件目录。未找到独立 input 插件：当前输入不是 zircon_plugins provider；不能仅因目录不叫 Input 忽略 Web owner。
- 已有键盘事件日志 [input_manager_event_log_harness:138–155](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/input/tests/input_manager/touch_gamepad.rs#L138-L155>) 只提交一次固定 KeyA 按下；[focus_loss_releases_active_input_without_disconnect:441起](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/input/tests/input_manager/frame_state.rs#L441>) 是失焦控制；均未证明本问题的同focus release。
- [PR125 快照](<D:/CodexData/worktrees/ze-ue-audit-20261117/reports/ue-module-audit-20261117/pr125-snapshot.json>) 是 graphics report-label 测试比较与 editor import 修复，closed / 未merged，不是输入等价PR。只读 `git show d44ef2708` 确认已合 #123 是 input-stack owner / 500-line budget 同步，虽抽出 apply_event 但没有 Web listener 配对修复/同例回归。
- 已读 [Runtime56旧输入总账](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/docs/plans/optimize/zircon_runtime/56-input-device-event-frame-state-action-map-focus-gamepad-recording-replay-host-product-integration-review.md#L172-L250>)、[Runtime220当前重判:172–199](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/docs/plans/optimize/zircon_runtime/220-runtime-input-device-event-frame-action-context-trigger-modifier-user-focus-gamepad-rumble-record-replay-host-editor.md#L172-L199>)：不重复开 native 左右modifier、逻辑键缺失、context priority、gamepad generation、无action产品consumer等大项；本条是已有合法 Web 事件错误配对的窄缺陷。
- [当前open135–145快照](<D:/CodexData/worktrees/ze-ue-audit-20261117/reports/ue-module-audit-20261117/github-pr-snapshot.json>) 无输入同题。去重仅覆盖这份快照、#123本地提交和已列文档，不声称穷尽历史PR。
- 未清：真实浏览器按键序列、每个支持浏览器的 key/code 事实、配对与多来源冲突策略、Rust/wasm可编译性、ECS/script产品消费、压力/性能/ABI回归。**均 not_run / 未验证；没有测试PASS、修复或验收结论。**

### UE 来源 SHA-256（本地原文件；短引用之外不复制实现）

| 来源 | SHA-256 |
|---|---|
| [ApplicationCore Build](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/ApplicationCore/ApplicationCore.Build.cs>) | ec667eda1b75d9031972495d89463bd983cf38d7f2f4d5cf4b641f94bfa4f6a4 |
| [Slate Build](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Slate/Slate.Build.cs>) | bf21de4192671f27ed81df3cedfdb1a7ecd5b990b1478c16b02ae7dc9d5fdc4f |
| [InputCore Build](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/InputCore/InputCore.Build.cs>) | 6074156ee4e754211fa74b1ffb3cfc24b10003b95c5ebc9fc19067e9de5f671e |
| [Engine Build](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Engine/Engine.Build.cs>) | 1c6a698c731ca203fc9648a53c13bcc3768fae1fb91cd9ec658a3abfa962e212 |
| [EnhancedInput Build](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Plugins/EnhancedInput/Source/EnhancedInput/EnhancedInput.Build.cs>) | 16ae8e26c2f07d7f6e2bacc9ee1ea286e445bcdefb9d61615d1692fc3f818b1c |
| [EnhancedInputTestSuite Build](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Plugins/EnhancedInput/Source/EnhancedInputTestSuite/EnhancedInputTestSuite.Build.cs>) | ae1b326434b5b467d7527dcdaa20347fba92b9826612e87f1e5ffa4cf6a6e26e |
| [WindowsMessageInputDevice](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/ApplicationCore/Private/Windows/WindowsMessageInputDevice.cpp>) | 9905a900beaf79b1d4285dbd1246b9e13fe0a00754a919c3a46d37c320534184 |
| [SlateApplication](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Application/SlateApplication.cpp>) | 06f96167fd20bde540b7f60629b53c8cfbddd3ae4b7d93368b9bc4f9f45cfe99 |
| [InputCoreTypes](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/InputCore/Private/InputCoreTypes.cpp>) | 4e6d1c3da0f67c0491433b5aedfe99e6ea21ab906f15cfc0c2e9926ef1675091 |
| [PlayerInput](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/UserInterface/PlayerInput.cpp>) | e58db5316cab04dd2b4ad5ae3fdf974bb31b5f0a8670cb7cd59682d163d0a547 |
| [EnhancedPlayerInput](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Plugins/EnhancedInput/Source/EnhancedInput/Private/EnhancedPlayerInput.cpp>) | 8d06cc7e5229a0536e4131406dbf1939e3abfbd0ce6408a9340f0a4fb2d2a0f4 |
| [InputBindingTest](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Plugins/EnhancedInput/Source/EnhancedInputTestSuite/Private/InputBindingTest.cpp>) | 1577d3865608a38ce719c2f84813cebc4697696c8771100ed6f4b5279387e4b2 |

## 7. 原实现的 Windows 和浏览器实测

Windows x64/MSVC 在提交 `0358324d28cd1bdbedd6d0bfcfe8168e06c9fd0f` 运行六个用例。持续聚焦时提前松开 Shift、变更字形的重复键、两个数字键码贡献同一逻辑名三项失败。普通键与 Ctrl 对照、失焦与溢出后重新按下、物理码专用消息与 UTF-8 边界三项通过。[六项状态回归](https://github.com/He-Jiahui/ZirconEngine/blob/0358324d28cd1bdbedd6d0bfcfe8168e06c9fd0f/zircon_runtime/tests/browser_key_pairing.rs#L119)

Chrome 157 加载提交 `8c0a258c2ca43b0da7ee0c864a6f679cbe7d58e1` 编译的 Wasm，用 `KeyboardEvent` 向测试画布回放按放顺序。五个场景都运行了实际 canvas 监听器，23 个采样帧均通过原消息与释放监听器的核对。24 项游戏状态断言通过 17 项。[浏览器用例](https://github.com/He-Jiahui/ZirconEngine/blob/8c0a258c2ca43b0da7ee0c864a6f679cbe7d58e1/tools/validation/browser/web-listener-fixture.html#L68) · [Wasm 观察入口](https://github.com/He-Jiahui/ZirconEngine/blob/8c0a258c2ca43b0da7ee0c864a6f679cbe7d58e1/zircon_app/examples/web_listener_fixture.rs#L69)

| 操作 | 监听器实际接收 | 管理器实际状态 |
|---|---|---|
| 先松 Shift，再松 A | 键码 65，逻辑 `a`，无文本 | 键码已经释放，`A` 仍按住，空帧后仍按住 |
| 松开 Shift 后出现 A 的重复事件 | 键码 65，逻辑和文本均为 `a`，`repeat=true` | 保留 `A` 的同时多出一次 `a` 按下边沿 |
| 主键区 `1` 与小键盘 `1` 都按下，仅松开前者 | 松开键码 49 | 小键盘 97 仍按住，逻辑 `1` 已提前释放 |

UE 的 `Input.Triggers.Keyboard.BooleanReleased` 用相同的 SpaceBar 身份覆盖按下、持续和释放。源文件为 `Engine/Plugins/EnhancedInput/Source/EnhancedInputTestSuite/Private/InputTriggerTest.cpp:1175–1214`，SHA-256 `207e9b6eafb1bffe5efd0bc6aa04199550e3b5d07ae8673dce578d4aa561bf3a`。上述浏览器采样使用 ZE 的真实输入代码和受控 DOM 事件。

## 8. 相同输入在修复后的结果

在 `99fb3a50138f2d806c6f2d6f99b4aaa4eb1eefc6`，Web 帧先把当前逻辑名和文本发送给 UI，再按稳定的非零数字键码向游戏状态提交首次按下时的逻辑名。松开键码 65 时，游戏中的 `A` 产生一个释放边沿，在下一帧回到空闲。重复按键的原始 `text="a"`、`repeat=true` 均可观察。主键区与小键盘两个 `1` 的数字码为 49 和 97，首次松开时 `1` 继续按住，最后一次松开才释放。[Web 输入提交实现](https://github.com/He-Jiahui/ZirconEngine/blob/99fb3a50138f2d806c6f2d6f99b4aaa4eb1eefc6/zircon_runtime/src/browser/input.rs#L17)

| 执行范围 | 原实现 | 修复后 |
|---|---:|---:|
| Windows x64/MSVC，`zircon_runtime` 开 `native-host`，`browser_key_` 六项 | 3/6 | **6/6** |
| Chrome 157，原 canvas 回调、UI 前置观察和游戏管理器逐帧读回 | 17/24 | **24/24** |

另在独立 Chrome 配置中通过浏览器输入驱动发送 Shift、A、释放 Shift、重复 `a`、释放 `a`。浏览器对这五个事件都标记 `isTrusted=true`。在释放帧读取 `Key("A")` 得到 `held=false, just_released=true`，下一空帧两个值均为 false。数字码为 0 的兼容用例保留 `a` 的按下和释放。原始 Windows 用例的 Git blob 为 `e0b5627aa66349d9578cefb6f152ccc69e8287d0`。浏览器使用同一份 HTML 与 Wasm 示例源码。[浏览器检查](https://github.com/He-Jiahui/ZirconEngine/blob/99fb3a50138f2d806c6f2d6f99b4aaa4eb1eefc6/tools/validation/browser/web-listener-fixture.html#L68) · [Windows 回归](https://github.com/He-Jiahui/ZirconEngine/blob/99fb3a50138f2d806c6f2d6f99b4aaa4eb1eefc6/zircon_runtime/tests/browser_key_pairing.rs#L119)

本次的稳定身份是宿主已经提供的数字 `keyCode`。两个 DOM `code` 共用同一数字码时，左右 Shift 的个别占用超出这项修复的覆盖范围。
