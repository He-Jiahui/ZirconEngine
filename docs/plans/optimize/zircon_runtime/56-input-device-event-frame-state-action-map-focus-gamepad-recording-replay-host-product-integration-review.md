---
title: Input Device、Event、Frame State、Action Map、Focus、Gamepad、Recording、Replay、Host 与 Product Integration 工程化差距
category: zircon_runtime
report_id: Runtime56
review_date: 2026-08-20
baseline_head: bea1acf91b909525ab1759e2c800858b0eda6528
baseline_epoch: 335
related_code:
  - zircon_runtime/src/input
  - zircon_runtime/src/core/framework/input
  - zircon_runtime/src/dynamic_api/session/events.rs
  - zircon_runtime/src/dynamic_api/session/input_events.rs
  - zircon_runtime/src/dynamic_api/session/host_requests.rs
  - zircon_runtime/src/dynamic_api/session/runtime_ui.rs
  - zircon_runtime/src/dynamic_api/session/state.rs
  - zircon_runtime/src/dynamic_api/session/construction.rs
  - zircon_runtime/src/script/vm/gameplay_host/input.rs
  - zircon_runtime/src/script/vm/gameplay_host/values.rs
  - zircon_runtime_interface/src/runtime_api/session/events.rs
  - zircon_runtime_interface/src/runtime_api/host/host_requests.rs
  - zircon_app/src/entry/runtime_entry_app/keyboard_input
  - zircon_app/src/entry/runtime_entry_app/pointer_input
  - zircon_app/src/entry/runtime_entry_app/ime_input
  - zircon_app/src/entry/runtime_entry_app/gamepad
  - zircon_app/src/entry/runtime_entry_app/file_drag_drop
  - zircon_app/src/entry/runtime_entry_app/event_dispatch.rs
  - zircon_app/src/entry/runtime_entry_app/event_loop_policy
  - examples/vampire
tests:
  - zircon_runtime/src/input/tests
  - zircon_runtime/src/dynamic_api/tests
  - zircon_runtime/src/script/vm/gameplay_host/tests.rs
  - zircon_app/src/entry/tests/runtime_entry_input_guards
  - zircon_app/src/entry/tests/runtime_entry_input_guards/protocol.rs
  - zircon_app/src/entry/tests/runtime_entry_source_guards/host_requests.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_runtime/06-platform-input-process-review.md
  - docs/plans/optimize/zircon_runtime/11a-runtime-ui-architecture-tree-layout-input-accessibility-review.md
  - docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md
  - docs/plans/optimize/zircon_runtime/38-gameplay-framework-game-instance-world-context-level-game-mode-game-state-local-player-controller-pawn-possession-spawn-travel-network-save-scalability-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/24-stable-identity-handle-generation-owner-epoch-stale-reference-exhaustion-review.md
  - docs/plans/optimize/zircon_runtime/43-dynamic-runtime-session-registry-ffi-frame-event-extract-host-request-world-sync-ui-shader-prewarm-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/46-engine-module-service-contract-context-factory-descriptor-snapshot-composition-lifecycle-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/50-runtime-manager-resolver-named-service-handle-generation-lifecycle-product-integration-review.md
  - docs/plans/zircon_runtime/runtime/12-input-stack-and-action-mapping.md
  - docs/plans/zircon_runtime/runtime/12/failure-2026-07-19-app-entry-input-and-gamepad-storm-budget.md
  - docs/plans/performance/01/2026-08-14-runtime-input-ingress-current-review.md
  - docs/plans/performance/01/2026-08-15-input-action-evaluation-generation-and-workspace.md
reference_engines:
  - dev/UnrealEngine/Engine/Plugins/EnhancedInput/Source/EnhancedInput/Public/EnhancedInputSubsystemInterface.h
  - dev/UnrealEngine/Engine/Plugins/EnhancedInput/Source/EnhancedInput/Private/EnhancedInputSubsystemInterface.cpp
  - dev/UnrealEngine/Engine/Plugins/EnhancedInput/Source/EnhancedInput/Public/EnhancedPlayerInput.h
  - dev/UnrealEngine/Engine/Plugins/EnhancedInput/Source/EnhancedInput/Private/EnhancedPlayerInput.cpp
  - dev/UnrealEngine/Engine/Plugins/EnhancedInput/Source/EnhancedInput/Public/InputMappingContext.h
  - dev/UnrealEngine/Engine/Plugins/EnhancedInput/Source/EnhancedInput/Private/InputMappingContext.cpp
  - dev/UnrealEngine/Engine/Plugins/EnhancedInput/Source/EnhancedInput/Public/UserSettings/EnhancedInputUserSettings.h
  - dev/UnrealEngine/Engine/Plugins/EnhancedInput/Source/EnhancedInput/Private/UserSettings/EnhancedInputUserSettings.cpp
  - dev/godot/core/input/input.h
  - dev/godot/core/input/input.cpp
  - dev/godot/core/input/input_map.h
  - dev/godot/core/input/input_map.cpp
  - dev/godot/core/input/input_event.h
  - dev/godot/core/input/input_event.cpp
  - dev/bevy/crates/bevy_input/src/lib.rs
  - dev/bevy/crates/bevy_input/src/button_input.rs
  - dev/bevy/crates/bevy_input/src/gamepad.rs
  - dev/bevy/crates/bevy_input/src/keyboard.rs
  - dev/Fyrox/fyrox-impl/src/engine/input.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Debugging/DebugManager.cs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 56 · Input Device、Event、Frame State、Action Map、Focus、Gamepad、Recording、Replay、Host 与 Product Integration 工程化差距

## 1. 结论

当前输入子系统不是全空壳。`ButtonInputState`已有稳定的level/pressed/released语义；frame-local transition会在Level tick后清除；焦点丢失能局部合成release；事件队列可相邻合并cursor/motion；录制器有按条数上限；Action evaluator已有compiled generation、单次axis索引、可复用workspace和有序consumed-input索引；App gamepad poll有256条/2ms预算与continuation；动态host request分页还有commit/rollback。这些是可保留的底座。

但产品主链与这套底座是断开的。Builtin Input module在client/server/editor均被选择，注册空的零字段`InputDriver`和两个Immediate manager，`InputConfig::default()`却为disabled；全仓production没有一个`evaluate_actions`调用者，也没有Action Map或Recording/Replay产品consumer。真实脚本玩法每次调用`gameplay.key_pressed`都会解析`InputManager`、深拷贝`InputSnapshot`、再从字符串或裸码查询原始键。Vampire样例只用WASD原始按键，动作优先级、context、consumption与用户重映射均没有进入产品。

本轮确认三个独立P0。其一，模块readiness把空driver和无人消费的Action/Replay能力报成可用；其二，动态session先把事件交给Runtime UI，UI停止传播时直接跳过物理状态提交，按下与释放若被不同capture/focus状态吞掉，会永久卡住held状态；其三，Action Map把临时`gilrs::GamepadId`和版本不明的键盘Debug字符串哈希/平台raw code直接序列化，重连、设备顺序、winit命名或跨平台变化都可能静默改绑。它们不是“缺高级功能”，而是当前合同会向产品、存档和用户输入给出错误事实。

本轮登记 **3项P0、64项P1、16项P2和40项验收门禁**。目标不是继续给`InputManager`加helper，而是建立`InputIngressBroker + QualifiedPhysicalState + CompiledActionProgram + InputOwnershipArbiter + StableDeviceBinding + DeterministicInputJournal`，并让Local Player/Controller/Runtime UI/Editor/Replay真正消费同一代输入事实。Runtime06继续拥有Platform/Input/Process广度，Tooling32与PERF-MVP-012/426拥有通用热路径，Runtime11A拥有UI语义，Runtime38拥有Gameplay Framework，Runtime24拥有通用identity，Runtime43拥有动态session ABI，Runtime46/50拥有module/service/manager kernel；本篇拥有输入纵向组合、状态正确性、动作产品接线、稳定绑定和record/replay安全闭环。

本轮只做静态review与文档总账，没有修改production、tests、Cargo、ABI或参考源码；没有运行Cargo、窗口交互、gamepad设备矩阵、回放determinism、soak或benchmark。不能据此宣称性能或表现达到、超过当前Unreal。

## 2. 审查边界、规模与 currentness

### 2.1 物理冻结

| 范围 | 文件 / 行 / bytes / tests | fingerprint / 说明 |
|---|---:|---|
| Input contract与runtime实现 | 46 / 3,642 / 112,613 / 2 | SHA-256 `323352589249ae826ba1122053726980c22af5fc8cc7eeaaeedd873eff83c74e` |
| focused direct tests | 14 / 2,846 / 100,990 / 64 | SHA-256 `8c0666d95426f2d041ad026a920a2dba636e4585c02c3a95e61ab943f9b37b46` |
| dynamic session、App、script与sample产品链 | 45 / 6,774 / 245,875 / 49 | SHA-256 `8627041cf26bf2193cdd8a25b0b8d05267312a9f5460d41d7778097beed08b6b` |
| reference corpus | 20 / 15,964 / 621,744 / 44 | SHA-256 `53424dd5b8a5b5d1b9de2ad9bb58971e8a6a0a29a22faecc78599dc460a07531` |

fingerprint算法与Runtime55一致：相对路径转`/`、排序去重，以`path|lowercase per-file SHA-256`编码，LF连接且末尾不追加LF，再计算UTF-8 SHA-256。它只冻结本轮实际读取集合，不是input schema、device identity、action generation或replay artifact identity。

`zircon_runtime/src/input`与framework input本轮读取内容为clean；`zircon_runtime/src/dynamic_api/session/state.rs`、`input_events.rs`以及若干UI产品链文件已有其他会话/用户改动，本文按当前working tree读取且不覆盖。共享索引也持续变化，因此`source_recheck_required`保持true。基线HEAD为`bea1acf91b909525ab1759e2c800858b0eda6528`，coordinator baseline epoch为335。

### 2.2 已读与未重复宣称

- 逐文件读取`zircon_runtime/src/input`全部31个文件、framework input全部26个文件及focused tests。
- 追踪dynamic session的event/UI/input/host request/frame链，App keyboard/pointer/IME/file drag/device/gamepad/rumble链，Runtime Interface event/host request，以及脚本Gameplay Host和Vampire样例。
- 全仓反查production caller：`evaluate_actions`、Action Map/context配置、Recording/Replay没有产品consumer；脚本玩法仍消费raw snapshot。
- Runtime06已拥有空driver、状态/action/replay广度与逐事件ABI父问题；本文只登记新确认的输入纵向错误和产品闭环，不复制其P0/P1。
- 两份performance记录已证明单sample跨V1与至少6次mutex、并记录action evaluator generation/workspace优化；本文不重复登记PERF-MVP-012/426，也不把静态结构当性能结论。
- open gamepad storm failure仍由Input runtime执行计划12拥有；本文不改变其`open`状态。
- 只使用Unity Graphics DebugManager验证其调试输入的新旧路径，不外推Unity完整玩家输入系统。

## 3. 当前真实产品链

### 3.1 物理输入与UI抢占

```text
winit/gilrs event
  -> zircon_app converts to RuntimeEventV1
  -> RuntimeDynamicSession::dispatch_event
       -> RuntimeUiSurfaceSet::dispatch_event(event)
       -> if reply.stops_propagation(): return              [InputManager完全看不到]
       -> submit_input_event(event)
            -> RuntimeEventV1 -> InputEvent                 [丢viewport/device/time/sequence]
            -> InputManager::submit_event
  -> level/script tick reads InputSnapshot
  -> begin_frame clears transient edges
```

按下与释放分别走这条链。UI在按下后获得capture、modal或focus，再吞掉释放时，`InputManager`保留永久held；反过来只吞按下则脚本收到无来源release。Action evaluator虽提供`consumed_buttons/axes`，产品主链从不调用，因此UI consumption不是动作层的显式仲裁，而是对物理事实的提前删除。

### 3.2 动作系统死支路

```text
InputModule descriptor
  -> Immediate InputDriver (ZST, no behavior)
  -> Immediate InputManager
  -> Immediate InputActionManager

tests/manual caller:
  InputActionManager::set_action_map
  -> evaluate_actions(snapshot, active_contexts, consumed*)

production gameplay:
  gameplay.key_pressed("W")
  -> resolve InputManager
  -> clone InputSnapshot
  -> parse string/raw code
  -> query button level state
```

context priority只被存储/排序，evaluation不使用；未知context默认enabled；空active-context集合表示全部启用；contextless action永远启用。因而即使未来把单个caller接上，也还没有可承担UI优先级、Local Player或可重映射产品合同的语义。

### 3.3 录制、回放与host输出

```text
submit_event
  -> state mutation + event queue
  -> optional recorder(SystemTime millis, saturating sequence)

replay(recording)
  -> iterate frames/events immediately
  -> submit_event(event)             [忽略时间、sequence、frame pacing]
  -> host-output-shaped events may become IME/cursor/rumble OS requests
```

录制artifact没有schema/build/map/device/clock/project/session/RNG/completeness/checksum；frames无界，条数容量也不约束payload bytes。Replay没有隔离target、reset、live-input arbitration或副作用策略，不能承担确定性复现、自动化或玩家输入回放。

### 3.4 持久绑定身份

`InputActionMap`直接序列化`InputButton::Gamepad { gamepad: GamepadId(u64), ... }`和`InputAxisBinding.gamepad`。这个u64来自进程本次运行的`gilrs::GamepadId`，不是硬件GUID、用户slot或稳定controller identity。键盘侧又把左右Shift/Ctrl/Alt合并为16/17/18，WASD特殊编码，其余`winit::KeyCode`按Debug字符串做FNV哈希，unidentified key保留平台raw code；事件的scan code固定为0并忽略repeat。当前持久map因此既不稳定，也不能无损表达实际键盘事件。

## 4. 可保留基础

- `ButtonInputState`把level与frame edge分开，同帧press/release测试覆盖基本状态机。
- frame transition清理发生在Level tick之后，脚本能读取本帧edge。
- 焦点丢失会为本地已按下按钮生成release，而不是只清空集合。
- Action evaluator使用compiled generation、按button/axis建立索引，并复用workspace；warmup后存储可保持稳定。
- consumed button/axis采用已排序索引，查询不需要每次重建HashSet。
- cursor/motion相邻事件可合并，App gamepad poll有256条/2ms预算和continuation。
- recorder已有count admission；动态host request page有事务式commit/rollback。
- App rumble host已有effect数量上限和cleanup；ABI层对部分wheel/IME几何做finite/length验证。

这些基础只证明局部数据结构有价值，不证明Action Map、record/replay、device identity、UI仲裁或产品输入已经工程化。

## 5. P0 阻断项

| ID | 当前证据 | 工程后果 | 硬切目标 / owner |
|---|---|---|---|
| INP-P0-001 | Input module在三类profile均选择；`InputDriver`是空ZST，manager均Immediate且无service dependency；默认config disabled；production中Action evaluator、Action Map与Recording/Replay均0 consumer，真实脚本走raw snapshot | capability/load/readiness可把“类型注册”和测试helper误报为产品输入；后续玩法继续绑定临时键码，任何action优化都不影响真实产品 | 编译`InputCapabilityContract`，只有真实ingress owner、action tick consumer、health与terminal teardown齐备才可Ready；删空driver或把真实platform ingress迁入driver；脚本只消费typed action。Runtime56 + Runtime42/46/50/App01 |
| INP-P0-002 | dynamic session先dispatch Runtime UI；`stops_propagation`直接return且不提交物理状态；press/release独立路由；Action consumed API零产品调用 | capture/focus/modal切换可吞掉单侧edge，造成永久held、幽灵release、镜头或玩法持续移动；UI与gameplay看到不同物理事实 | ingress先写qualified physical state，再由`InputOwnershipArbiter`对同一sequence生成UI/gameplay ownership与action consumption；focus/capture转移合成有generation的cancel/release。Runtime56 + Runtime11A/43/App01 |
| INP-P0-003 | persisted Action Map序列化临时`gilrs::GamepadId`；键盘左右modifier折叠、WASD特判、其余Debug字符串FNV、unidentified平台raw code；无schema/version/layout/device GUID | 重连、设备枚举顺序、winit命名、平台或版本变化可静默把用户绑定指向其他设备/键，且无法可靠迁移或诊断 | 引入versioned `StableInputControlId`：device class + stable hardware/profile identity + user slot + standardized physical/logical control；加载必须校验/迁移/隔离，绝不静默接受未知映射。Runtime56 + Runtime24/25/45 |

## 6. P1 工程化差距

### 6.1 Module、Action、Context 与 Product Composition

| ID | 差距 | 目标 / owner |
|---|---|---|
| INP-P1-001 | `InputDriver`没有字段、方法、ingress、health或teardown | 删除占位；或让真实platform event source实现driver contract、backpressure与lifecycle |
| INP-P1-002 | manager descriptor没有driver/service dependency，注册顺序不能证明能力可用 | compiled dependency绑定ingress、clock、device registry与action owner；Runtime46 |
| INP-P1-003 | `InputConfig.enabled=false`与模块/manager Ready并存 | readiness区分Selected/Disabled/Starting/Ready/Degraded/Unavailable并说明原因 |
| INP-P1-004 | 没有runtime-owned action evaluation phase或每Local Player tick owner | 把compiled action evaluation接入明确schedule phase与player scope；Runtime38 |
| INP-P1-005 | script每次`key_pressed`都resolve manager、clone snapshot、解析字符串 | 脚本绑定typed action handle/instance，frame context一次解析并批量读取 |
| INP-P1-006 | evaluator的`consumed_buttons/axes`没有产品producer | UI/capture/gameplay通过同一ownership plan提供consumption，不靠提前删事件 |
| INP-P1-007 | `InputActionContext.priority`只存储/排序，evaluation不使用 | 高优先级context先决策，并可阻断低优先级同control action |
| INP-P1-008 | 未知context ID编译后默认为enabled | unknown context在compile/load时报typed error，不可fail-open |
| INP-P1-009 | 空active-context集合被解释为“全部启用” | 用显式All/None/Set模式，空Set必须表示None |
| INP-P1-010 | contextless action始终全局active | action必须声明scope/default context或显式global policy |
| INP-P1-011 | duplicate action/context ID没有校验，结果依赖map构造顺序 | compiler拒绝duplicate并报告两个source locations |
| INP-P1-012 | binding引用未知action时被静默忽略 | compile返回structured diagnostics和invalid artifact，不生成部分program |
| INP-P1-013 | action/context/control ID允许空值、任意字符串且无schema | 使用validated interned ID、namespace与schema revision |
| INP-P1-014 | 同一control可在多context/action重复触发，无冲突查询 | 编译conflict graph，声明Consume/Share/Chord/Block策略并提供query |
| INP-P1-015 | map替换没有公开generation、held-key suppression或cancel transition | rebuild发布generation；旧held control可选择等释放、cancel或重触发 |
| INP-P1-016 | action state全局，缺Local Player/Controller/World/Component scope | 每player/world owner独立状态、context stack和device routing |
| INP-P1-017 | 只有标量/按钮组合，没有trigger、modifier、hold/tap/repeat/chord | typed trigger/modifier pipeline，显式started/ongoing/triggered/completed/canceled |
| INP-P1-018 | 轴值只有scalar相加，缺Vector2/3、radial deadzone和composite | typed action value与validated composite processor |
| INP-P1-019 | 没有受控action/control injection，测试只能直接构造内部状态 | 提供qualified synthetic source、权限、sequence与隔离target |
| INP-P1-020 | action state复制拥有型String集合且无source/device/time/map generation | interned handle + source contribution + device/player + monotonic time + map generation |

### 6.2 Event、Device、Window 与 Wire Contract

| ID | 差距 | 目标 / owner |
|---|---|---|
| INP-P1-021 | `RuntimeEventV1`的viewport在转换成内部`InputEvent`时丢失 | 内部事件保留qualified window/viewport，直到消费终点 |
| INP-P1-022 | 通用输入事件没有device/user/seat/source sequence/monotonic timestamp | `QualifiedInputEvent`统一携带这些identity与clock字段 |
| INP-P1-023 | App把keyboard scan code固定为0，并忽略winit repeat | wire同时保留physical、logical、text、location、repeat与native code |
| INP-P1-024 | 左右Shift/Ctrl/Alt折叠为相同码 | standardized physical control区分left/right/location |
| INP-P1-025 | 其余KeyCode按Debug spelling做FNV且测试锁死数字 | 由版本化枚举/表驱动映射，Debug文字不得成为wire ABI |
| INP-P1-026 | unidentified native code没有平台namespace/layout/generation | 包含platform code set、layout/driver provenance并只作可诊断fallback |
| INP-P1-027 | Runtime只重建少量logical key，Space/Enter/arrow/F-key等不可从真实ABI得到 | 完整映射或直接传输validated logical key；测试只用可达事件 |
| INP-P1-028 | UI event metadata可用默认时间且sequence饱和后重复 | 单调clock和非重复sequence，溢出触发epoch rollover/fail-close |
| INP-P1-029 | 单窗口focus loss全局释放keyboard/mouse/gamepad/touch | 按window/device/seat执行focus policy；gamepad不默认从属窗口focus |
| INP-P1-030 | cursor/motion没有坐标空间、DPI、scale、source window/device | 显式logical/physical/relative space与transform generation |
| INP-P1-031 | line wheel和pixel wheel数值直接相加，unit取最后事件 | 分单位累计或先按配置归一化；禁止量纲混合 |
| INP-P1-032 | `WheelScrolled`与`MouseWheel`两套事件并存且语义重叠 | 硬切单一versioned wheel event并迁移consumer |
| INP-P1-033 | 多个内部float路径未统一拒绝NaN/Inf或非法范围 | ingress validator按事件类型执行finite/range contract |
| INP-P1-034 | touch只有id/phase/position，缺pressure/tool/tilt/force/device/window | versioned touch/pen contact record与capability bits |
| INP-P1-035 | gamepad button/axis event可在未连接设备上被接受 | connection generation先于sample；stale/disconnected sample拒绝或隔离 |
| INP-P1-036 | gamepad settings全部硬编码默认值 | per-device/profile settings由validated config和hardware mapping产生 |
| INP-P1-037 | axis/button setting构造可接受退化区间，归一化存在边界风险 | typed constructor返回错误并覆盖deadzone/inversion/curve边界测试 |
| INP-P1-038 | device信息只有name/vendor/product，没有GUID/capability/battery | `InputDeviceDescriptor`含stable GUID、capability、mapping、power与generation |
| INP-P1-039 | disconnect/reconnect没有旧slot到新device generation的显式迁移 | DeviceRegistry发布remove/add/rebind receipt并清理旧贡献 |
| INP-P1-040 | file drag把PathBuf lossy转String，缺source window/type/security provenance | 保留platform path/URI bytes、drop session、window与admission result |
| INP-P1-041 | IME host request缺target revision、request ID、deadline和ack | request/response绑定text target generation并返回typed terminal result |
| INP-P1-042 | cursor host request缺window/capture generation、request ID和ack | qualified cursor/capture operation，stale target拒绝 |
| INP-P1-043 | rumble request缺统一request ID、deadline、device generation与completion | effect operation返回accepted/applied/canceled/failed receipt |
| INP-P1-044 | host输出命令与物理输入事件共用`InputEvent`语义域 | 分离InputIngress、InputDecision与InputHostEffect三类合同 |

### 6.3 Frame State、Retention、Recording、Replay 与 Lifecycle

| ID | 差距 | 目标 / owner |
|---|---|---|
| INP-P1-045 | 一个`Mutex`保护设备、state、queue、recording和host request全部状态 | 按owner/snapshot publication分离写入与只读frame view；Tooling32/Performance01 |
| INP-P1-046 | snapshot/frame snapshot在锁内深拷贝sets/vectors/strings | immutable generation snapshot或double buffer，读者持lease |
| INP-P1-047 | 每个ABI sample跨多层转换和多次锁，已由PERF记录证明 | 按batch/page ingress一次validation与一次publication；沿用PERF-MVP-012 |
| INP-P1-048 | `begin_frame`会静默丢弃未drain transient events/host commands | frame boundary返回consumption status；未处理数据按policy保留/overflow |
| INP-P1-049 | 队列只合并相邻cursor/motion，其他edge可无界增长 | per-kind budget、critical edge reservation、coalescing与gap marker |
| INP-P1-050 | recording/queue主要按条数限流，payload bytes无全局上界 | count+bytes+time三维admission并纳入session/global budget |
| INP-P1-051 | backpressure只有局部drop计数，没有producer throttle/consumer lag contract | typed overflow、lag、retry/resync与pressure telemetry |
| INP-P1-052 | public `InputManager` trait为begin/drain/record/status等提供no-op默认 | 必需能力改为required method；可选能力显式Unsupported |
| INP-P1-053 | concrete manager未覆盖`subscribe_events`，公共订阅始终`None` | 实现有界订阅或删除虚假能力，并由真实consumer验证 |
| INP-P1-054 | poison mutex直接取回inner继续运行，无Degraded状态 | 记录fault、隔离损坏generation并触发重建/terminal failure |
| INP-P1-055 | recording timestamp使用可倒退的`SystemTime`毫秒 | 单调clock + timebase metadata；wall clock只作旁路诊断 |
| INP-P1-056 | sequence饱和后重复，不产生epoch或错误 | checked increment、epoch rollover或停止录制并标incomplete |
| INP-P1-057 | `InputRecording.frames`无界 | chunked journal、总bytes/duration上限、streaming writer与retention |
| INP-P1-058 | recording缺schema/build/map/device/clock/project/session/RNG/completeness/checksum | versioned manifest + chunk hash + terminal completeness receipt |
| INP-P1-059 | replay忽略record timestamp/sequence，事件立即全部提交 | scheduler按timebase/frame/sequence恢复，检测乱序与gap |
| INP-P1-060 | host-effect-shaped事件可被录制并在replay重发OS副作用 | 默认只录ingress/decision；host effect单独journal并需sandbox policy |
| INP-P1-061 | replay接受缺帧、不完整和sequence gap artifact | preflight验证manifest、hash、continuity和required device/map schema |
| INP-P1-062 | replay没有reset、隔离world/player、live-input arbitration | `ReplaySession`拥有target、reset snapshot、live overlay policy和cancel |
| INP-P1-063 | recording/replay只有简单status，没有typed错误、progress与terminal receipt | 接入Operation contract，暴露accepted/progress/completed/failed/canceled |
| INP-P1-064 | Input module没有显式quiesce、device drain、host effect cancel或产品health | module lifecycle执行stop ingress -> cancel effects -> drain -> release devices -> terminal report |

## 7. P2 完整产品能力

| ID | 能力 | 前置条件 |
|---|---|---|
| INP-P2-001 | 本地化键名、平台glyph与layout-aware显示 | StableInputControlId与device profile完成 |
| INP-P2-002 | 键盘布局提示和冲突解释 | physical/logical双身份与validator完成 |
| INP-P2-003 | Action Map编辑器的冲突图与context优先级可视化 | compiled conflict graph完成 |
| INP-P2-004 | per-device无障碍预设、sticky/chord/hold辅助 | player/device scope与trigger pipeline完成 |
| INP-P2-005 | 安全rebind capture、保留键和超时取消UX | ownership arbiter与stable binding完成 |
| INP-P2-006 | 键鼠/手柄prompt自动切换与防抖 | device activity provenance完成 |
| INP-P2-007 | haptic curve、channel、mix与priority | qualified rumble operation完成 |
| INP-P2-008 | pinch/swipe/rotate等gesture recognizer | touch/pen contact contract完成 |
| INP-P2-009 | sensor、pen、MIDI或专业控制器扩展 | extensible device/control schema完成 |
| INP-P2-010 | frame input history与action contribution debugger | bounded journal与source contribution完成 |
| INP-P2-011 | remote/network input可视化与authority标记 | player/source identity和network contract完成 |
| INP-P2-012 | deterministic replay导入/导出与差异CLI | replay manifest和oracle完成 |
| INP-P2-013 | 官方layout/profile模板与hardware database更新 | signed/versioned mapping catalog完成 |
| INP-P2-014 | per-action latency/drop/contention telemetry dashboard | bounded diagnostics和clock完成 |
| INP-P2-015 | 输入provenance、ownership与capture调试UI | arbitration journal完成 |
| INP-P2-016 | 跨设备/平台/帧率benchmark与soak harness | correctness、fault与artifact gates先通过 |

P2不能替代P0/P1；prompt、gesture或编辑器再漂亮，也不能掩盖卡键、错误持久绑定或零产品Action consumer。

## 8. 参考引擎对照

| 参考 | 本轮源码事实 | 对Zircon的约束 | 不照搬的部分 |
|---|---|---|---|
| Unreal Enhanced Input | Mapping Context是asset；LocalPlayer subsystem拥有user settings与applied contexts，World subsystem另有player input；context按priority降序；高优先级consumption阻断低优先级；rebuild可忽略held key直到release；Action有trigger/modifier/injection/instance；User Settings按local player/profile/hardware mapping持久化并报告失败 | context priority必须执行；map rebuild必须处理held状态；action状态按player/world scope；用户重映射需要稳定hardware/profile identity与versioned save | 不复制UObject/Blueprint宏或UE类层级；先冻结Rust owner、artifact和schedule合同 |
| Godot | InputEvent保留device，窗口事件保留window_id；keyboard保留physical/logical/unicode/location/echo；InputMap支持ALL_DEVICES、per-action deadzone、duplicate校验和suggestions；Input按device保存action state并在断开时清理；有buffered/accumulated input与joy GUID mapping | 事件不能在ABI后丢window/device；键盘不能只剩哈希码；action state与disconnect必须per-device；加载映射要校验并给诊断 | 不复制Singleton式全局authority；Zircon应按Runtime/Session/Player scope |
| Bevy | keyboard事件保留physical/logical/repeat/window；raw与filtered gamepad event分开；每个gamepad是Entity并带settings，重连保留settings；axis/button settings有typed validation error；schedule保证connection先于sample processing | 连接generation、原始/过滤事件、validated settings和schedule顺序必须显式 | 不把ECS Entity直接作为跨存档硬件identity |
| Fyrox | `InputState`源码明确定位为简化shortcut，建议复杂系统使用event-based input，并说明它不保留device origin | Zircon脚本raw snapshot只能作为兼容shortcut，不能作为工程级玩法主合同 | 不以Fyrox的简化层作为目标上限 |
| Unity Graphics | DebugManager新路径启停InputActionMap并用action/composite/performed callback；legacy路径显式处理repeat policy | 即使只是调试UI，也应有明确action map lifecycle和repeat policy，不应散落裸键查询 | Graphics仓库不是完整Unity玩家输入参考，不据此推断设备、存档或网络能力 |

参考结论不是“代码量要和Unreal相等”，而是Zircon必须把identity、owner、priority、generation、lifecycle、persisted artifact和产品consumer变成可验证合同。只有这些正确性门通过后，才能比较输入延迟、吞吐、内存和表现。

## 9. Owner、依赖与硬切边界

| Owner | 本篇负责 | 继续由父报告负责 |
|---|---|---|
| Runtime56 | 物理输入纵向组合、UI/游戏状态一致性、Action product tick、stable binding、record/replay安全与input lifecycle | 不拥有通用module/service/manager kernel、所有UI或所有platform host |
| Runtime06 | Platform/Input/Process首轮广度与host owner | 不再单独细化本篇已编号的action/replay/device纵向finding |
| Runtime11A / 11B | Runtime UI input effect、focus/IME/text语义 | 物理状态先写与ownership arbitration由Runtime56连接 |
| Tooling32 + Performance01 | lock/clone/batch/queue热路径与benchmark | 本篇只登记合同影响，不重复PERF-MVP-012/426 |
| Runtime38 | Local Player/Controller/Gameplay Framework输入消费 | Action program和player-scoped state由Runtime56提供 |
| Runtime24/25/45 | 通用identity、artifact/path、持久设置backend | 输入control/device/profile schema由Runtime56定义 |
| Runtime43 + Interface01/03/07 | dynamic session event/host request ABI、foreign page与UI gateway | 输入字段、sequence和ack语义由Runtime56提出 |
| Runtime42/46/50 + App01 | module selection、service/manager lifecycle与product host | Input readiness只在真实consumer和health存在时可发布 |

硬切删除清单：空`InputDriver`；脚本产品raw key字符串入口；无schema的持久`GamepadId`/FNV键码；UI早退前删除物理事实；`InputManager`能力no-op默认；把host effect伪装成input event；忽略时间和完整性的replay。迁移期可以有显式`LegacyRawInputShortcut`，但必须标Deprecated/Unavailable for shipping bindings，并且不能继续作为Vampire或默认模板主链。

## 10. 重构里程碑

### M0 · Truth Freeze 与可达性

- 为module readiness、script raw path、action/record/replay consumer建立source guard和product reachability测试；
- 冻结现有event/control/map/recording schema并标记不稳定字段；
- 建立卡键、设备重连错绑和回放副作用三个最小失败repro。

### M1 · Qualified Ingress 与 Stable Device Registry

- 引入window/device/user/seat/clock/sequence/generation完整事件；
- connection先于sample，disconnect清理贡献并发布terminal receipt；
- 键盘、wheel、touch、file drag与gamepad统一validation和versioned control identity。

### M2 · Physical State 与 Ownership Arbitration

- ingress先发布物理事实，再产生UI/gameplay/editor ownership decision；
- capture/focus/modal/context变化以generation处理held control；
- 按window/player/device隔离状态，删除全局focus release。

### M3 · Compiled Action Program 与 Product Cutover

- compiler校验ID、context、priority、conflict、trigger、modifier、typed value与generation；
- Local Player/Controller schedule消费action state；
- Vampire、模板和脚本API硬切typed action，raw shortcut不再作为shipping路径。

### M4 · User Binding、Profile 与 Authoring

- stable control/device profile、user slot、hardware mapping、versioned migration与conflict query；
- 保存、加载、损坏隔离、设备缺失和跨平台fallback返回typed diagnostics；
- Editor建立真实Action Map编译/重绑/冲突产品闭环。

### M5 · Deterministic Journal、Replay 与 Host Effects

- chunked manifest、单调时间、sequence、checksum、completeness与bounded writer；
- replay preflight、isolated target、reset/live policy、paced schedule与gap rejection；
- host effect分离并以request/response operation执行，默认回放无OS副作用。

### M6 · Lifecycle、Fault、Performance 与 Competitive Evidence

- input module完成quiesce/drain/device/effect teardown与Degraded/Failed health；
- correctness/fault/soak后再做batch ingress、immutable snapshot、contention/latency优化；
- 在同设备、同事件序列、同帧率、同平台与同采样协议下比较Unreal/Godot/Bevy，禁止用微基准替代产品证据。

## 11. 验收矩阵

| Gate | 验收内容 |
|---|---|
| INP-G01 | Input module只有在真实ingress、action tick、health和teardown齐备时报告Ready；空driver不存在 |
| INP-G02 | production至少有一个Local Player/Controller action consumer，Vampire和默认模板不再调用raw key字符串 |
| INP-G03 | UI吞掉release、按下后获得capture、modal切换、focus切换均不会留下永久held或幽灵edge |
| INP-G04 | 同一event sequence的physical state、UI decision、action decision可关联并解释ownership |
| INP-G05 | Action context priority会真实阻断低优先级冲突，测试覆盖Share/Consume/Block |
| INP-G06 | unknown/duplicate/empty action或context、unknown binding target都使compile失败并有source diagnostic |
| INP-G07 | active context的All/None/Set语义无歧义，空Set不启用全部action |
| INP-G08 | map rebuild对held control按policy生成ignore/cancel/retrigger，旧generation结果不会混入新map |
| INP-G09 | trigger/modifier/hold/tap/repeat/chord与Vector2 composite有确定性测试 |
| INP-G10 | 每Local Player/Controller/World的action state、context和device routing完全隔离 |
| INP-G11 | persisted binding不含临时gilrs ID、Debug字符串哈希或无namespace平台raw code |
| INP-G12 | 同一controller断开重连保留用户slot/settings；不同controller不会继承错误binding |
| INP-G13 | mapping artifact有schema/version/profile/hardware identity，未知版本fail-close或迁移 |
| INP-G14 | keyboard wire覆盖physical/logical/text/location/repeat/native namespace，左右modifier不折叠 |
| INP-G15 | Space/Enter/arrows/F-keys及布局变化通过真实App ABI到达action，不靠测试直接构造不可达事件 |
| INP-G16 | internal event保留window/viewport/device/user/seat/source/monotonic time/sequence/generation |
| INP-G17 | sequence溢出不会重复；clock回拨不改变事件顺序 |
| INP-G18 | line与pixel wheel不做量纲混加；legacy duplicate wheel variant已删除 |
| INP-G19 | cursor/motion在多window、多DPI、relative/captured模式下坐标与generation正确 |
| INP-G20 | touch/pen contact保留device/window/pressure/tool等可用字段，非法值在ingress拒绝 |
| INP-G21 | connection事件总在sample前处理；disconnected/stale generation sample不会改变state |
| INP-G22 | gamepad axis/button settings构造拒绝退化区间，per-device校准通过边界与property test |
| INP-G23 | window focus loss只释放该window policy拥有的输入，不无条件释放全部gamepad/touch |
| INP-G24 | file drag保留无损path/URI、source window与admission provenance |
| INP-G25 | IME/cursor/rumble请求含target generation、request ID、deadline并返回terminal ack |
| INP-G26 | host effect与physical input类型、queue、recording policy完全分离 |
| INP-G27 | frame snapshot通过immutable generation/lease发布，读者不在全局mutex下深clone所有集合 |
| INP-G28 | begin-frame遇到未drain事件或host command有显式policy与指标，不静默丢弃 |
| INP-G29 | event/recording queue同时受count、bytes、duration/global budget约束，并发producer不会突破 |
| INP-G30 | overflow产生typed gap/backpressure并保留关键release/cancel，不造成stuck state |
| INP-G31 | `InputManager`必需方法无no-op默认；不支持的订阅/录制能力返回Unsupported |
| INP-G32 | mutex poison/fault进入Degraded/Failed并隔离损坏generation，不静默继续 |
| INP-G33 | recording manifest包含schema/build/map/device/clock/project/session/RNG/completeness/checksum |
| INP-G34 | 长录制以bounded chunk streaming写出，frames和payload bytes均无无界增长 |
| INP-G35 | replay按monotonic time/frame/sequence调度，乱序、gap、hash错误和incomplete artifact在preflight拒绝 |
| INP-G36 | replay有isolated target、reset、live-input policy和cancel；默认不触发真实IME/cursor/rumble副作用 |
| INP-G37 | record/replay operation提供accepted/progress/completed/failed/canceled terminal receipt |
| INP-G38 | stop流程先停ingress，再cancel host effects、drain queue、断开device并发布terminal health；无泄漏线程/handle |
| INP-G39 | correctness、fault、device reconnect、multi-window、layout、soak全部通过后，才运行batch/latency/contention benchmark |
| INP-G40 | 同硬件、同OS、同事件序列、同窗口/帧率/采样协议与统计方法完成Unreal对照前，不允许“性能或表现优于Unreal”结论 |

## 12. 状态与产出记录

| 项目 | 状态 | 日期 | 证据 |
|---|---|---|---|
| Input contract/runtime逐文件审查 | review_complete | 2026-08-20 | 46文件、3,642行、112,613 bytes、2 inline tests |
| focused tests审查 | review_complete | 2026-08-20 | 14文件、2,846行、100,990 bytes、64 tests |
| dynamic/App/script/sample产品链审查 | review_complete | 2026-08-20 | 45文件、6,774行、245,875 bytes、49 embedded tests |
| Unreal/Godot/Bevy/Fyrox/Unity Graphics对照 | review_complete | 2026-08-20 | 20文件、15,964行、621,744 bytes、44 tests |
| P0/P1/P2与验收门禁 | review_complete | 2026-08-20 | 3 P0 / 64 P1 / 16 P2 / 40 gates |
| Production重构 | pending | - | 本篇不修改production、tests、Cargo或ABI |
| 动态/性能/竞争性验证 | pending | - | 未运行Cargo、窗口、设备矩阵、replay、soak或benchmark |

Runtime56的review完成不等于输入系统完成。实施前必须先重读current source和Input runtime计划12的open failure；任何source fingerprint、ABI、module profile、script host、dynamic UI routing或gamepad bridge变化都应使本报告进入recheck。下一批review应转向尚未深审的独立Runtime垂直面，不继续扩写本篇或回到tooling优化。
