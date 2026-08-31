---
title: Runtime Input Device、Event、Frame State、Action Map、Focus、Gamepad、Recording、Replay、Host 与 Product Integration Current Source Review
category: zircon_runtime
report_id: Runtime117
review_date: 2026-08-23
baseline_head: 9fee3ea0435961a81c85aa2502e64f1f357345d7
baseline_epoch: 365
supersedes:
  - docs/plans/optimize/zircon_runtime/56-input-device-event-frame-state-action-map-focus-gamepad-recording-replay-host-product-integration-review.md
related_code:
  - zircon_runtime/src/core/framework/input
  - zircon_runtime/src/input
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
  - zircon_runtime/src/dynamic_api/tests/input_events.rs
  - zircon_runtime/src/script/vm/gameplay_host/tests.rs
  - zircon_app/src/entry/tests/runtime_entry_input_guards
plan_sources:
  - docs/plans/optimize/zircon_runtime/56-input-device-event-frame-state-action-map-focus-gamepad-recording-replay-host-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/99q-runtime-platform-host-window-registry-monitor-display-event-loop-application-lifecycle-surface-command-product-integration-current-source-review.md
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
  - dev/UnrealEngine/Engine/Plugins/EnhancedInput/Source/EnhancedInputTestSuite/Private/InputBindingTest.cpp
  - dev/UnrealEngine/Engine/Plugins/EnhancedInput/Source/EnhancedInputTestSuite/Private/InputPlayerMappableKeysTests.cpp
  - dev/godot/core/input/input.h
  - dev/godot/core/input/input.cpp
  - dev/godot/core/input/input_map.h
  - dev/godot/core/input/input_map.cpp
  - dev/godot/core/input/input_event.h
  - dev/godot/core/input/input_event.cpp
  - dev/godot/tests/core/input/test_input_event_key.cpp
  - dev/godot/tests/core/input/test_input_event_mouse.cpp
  - dev/godot/tests/core/input/test_input_event.cpp
  - dev/bevy/crates/bevy_input/src/lib.rs
  - dev/bevy/crates/bevy_input/src/button_input.rs
  - dev/bevy/crates/bevy_input/src/gamepad.rs
  - dev/bevy/crates/bevy_input/src/keyboard.rs
  - dev/Fyrox/fyrox-impl/src/engine/input.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Debugging/DebugManager.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Debugging/DebugManager.Input.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Debugging/DebugManager.InputLegacy.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Tests/Runtime/DebugManagerTests.cs
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 99r · Runtime Input Current Source Review

## 1. 结论

当前输入子系统具有可保留的局部状态机和性能底座，但仍不是可承担工程级产品的输入架构。`ButtonInputState`、相邻pointer coalescing、gamepad deadzone/hysteresis、action generation/workspace、poll budget和host request分页都是真实实现；然而Builtin Input module仍注册空的零字段`InputDriver`，默认配置仍disabled，production仍没有任何`InputActionManager::evaluate_actions*`调用者，也没有Recording/Replay产品consumer。Vampire和脚本玩法仍以`gameplay.key_pressed("W")`等raw字符串为主合同。

Runtime56之后只确认两处局部进展。第一，`gameplay.key_pressed`改用`InputManager::button_pressed`，`DefaultInputManager`可直接查询held set，避免每次深拷贝整份`InputSnapshot`；但每次调用仍resolve manager、解析/分配raw key，且没有切到typed action。第二，IME/cursor/rumble host request不再被`begin_frame`清空，frame snapshot只展示本帧新增请求而drain仍可取回跨帧请求；但物理/瞬态事件队列仍在`begin_frame`无条件`clear()`，测试还明确固化这种丢弃语义。

三个独立P0全部仍Open：模块readiness发布虚假能力；Runtime UI在物理状态提交前停止传播会制造永久held/幽灵release；持久Action Map直接保存临时`gilrs::GamepadId`和不稳定键盘编码。当前账本为 **3 P0 Open、62 P1 Open、2 P1 Partial、16 P2 Open、40 Gate Fail**。目标架构必须收束为`InputIngressBroker + StableDeviceRegistry + QualifiedPhysicalState + InputOwnershipArbiter + CompiledActionProgram + DeterministicInputJournal`，并由Local Player、Runtime UI、Editor和Replay共同消费同一代输入事实。

本轮只做current-source静态review和文档记录，没有修改production、tests、Cargo或ABI；没有运行Cargo、真实窗口、设备矩阵、replay determinism、soak或benchmark，因此不能宣称性能或表现达到、超过当前Unreal。按用户范围，本篇不展开tooling优化。

## 2. 审查边界与物理冻结

| 范围 | 文件 / 行 / 非空行 / bytes / tests / dirty | fingerprint |
|---|---:|---|
| Input contract与runtime production | 46 / 3,797 / 3,376 / 119,164 / 4 / 3 | `ad21dad46615c49189749600523f3f7224eeb39e14024a68ef60e81f16b6451b` |
| focused input tests | 21 / 3,477 / 3,157 / 125,834 / 62 / 2 | `9d109494be02942c06c6fa9f3deccdd772e8d11e4a657a2e1f4d6b86cd2a7df9` |
| dynamic/App/script/sample产品链 | 56 / 16,815 / 15,659 / 574,673 / 50 / 4 | `5ab3ed22898b079662fa5729eeee8cabe97024851f30b1e6d80477713f87de1a` |
| reference corpus | 28 / 21,728 / 18,514 / 827,335 / 44 / 0 | `b00ac4b6ccb054db66725f8fabf75f9fbc1e30e23b91e175dca7391e9993149f` |

fingerprint算法：仓库相对路径转`/`并排序去重，以`path|lowercase per-file SHA-256`逐行编码，LF连接且末尾无LF，再计算UTF-8 SHA-256。它冻结本轮实际读取集合，不是schema、device identity、map generation或replay artifact identity。

本轮按当前working tree读取。`input_manager.rs`、`default_input_manager.rs`、`input_state.rs`以及部分dynamic session与tests存在其他会话/用户改动，本文只审查结果，不覆盖或归属这些源文件。基线HEAD为`9fee3ea0435961a81c85aa2502e64f1f357345d7`，coordinator epoch为365。Runtime12的gamepad storm failure继续保持open；本篇不伪造修复或关闭记录。

## 3. 当前产品事实链

```text
winit/gilrs event
  -> zircon_app RuntimeEventV1                 [丢WindowId/DeviceId，键盘scan=0]
  -> RuntimeDynamicSession::dispatch_event
       -> RuntimeUiSurfaceSet::dispatch_event
       -> if stops_propagation(): return       [物理状态事实被删除]
       -> RuntimeEventV1 -> InputEvent          [再丢viewport/time/sequence]
       -> DefaultInputManager::submit_event
  -> script gameplay.key_pressed(raw string)   [没有Action product tick]
  -> begin_frame clears transient event buffer [未drain事件丢失]
```

物理输入与UI仲裁必须拆开。UI可以先做命中、capture和路由决策，但press/release/device/focus事实必须先以同一sequence写入qualified physical state；随后ownership decision只决定哪些action或UI consumer可见，不能通过早退删除事实。Runtime12当前“gameplay consumes UI-unhandled”的描述只能用于decision层，不能继续作为physical ingress顺序。

Action evaluator的compiled generation只是内部lookup缓存：context priority不参与仲裁，unknown context被插入并默认enabled，空active context集合代表全部启用，contextless action始终active，binding指向未知action时静默不进入compiled action。测试覆盖这些helper可运行，却没有产品schedule、Local Player scope、rebind generation或真实UI consumed producer。

Recording只保存`SystemTime`毫秒、饱和sequence和克隆事件；`InputRecording.frames`无总上界，没有schema/build/map/device/clock/checksum。Replay按frame立即重新`submit_event`，忽略原timestamp/sequence，且测试证明IME/cursor等host-effect-shaped事件会被回放。它是内存测试helper，不是确定性回放制品。

## 4. 可保留基础与已确认局部进展

- `ButtonInputState`正确区分level、just pressed和just released；disconnect能清除该gamepad贡献。
- cursor/motion相邻事件可合并，focus event形成合并屏障；App gamepad poll具备256条/2ms预算和continuation。
- gamepad axis/button已有deadzone、livezone、change threshold和hysteresis，但设置来源与validation仍不完整。
- Action evaluator已有map-change generation、单次axis索引、复用workspace和排序后的consumed-input索引；这些优化应保留在未来compiled program内部。
- `button_pressed`直接查询消除了脚本raw查询的整快照clone，`INP-P1-005`因此为Partial，不代表产品Action完成。
- host request跨frame保留，`INP-P1-048`因此为Partial；物理事件仍被frame boundary静默丢弃。
- host request page支持borrowed encoding，Runtime UI的IME请求可以并入dynamic host output；request identity、ack和target generation仍缺失。

## 5. P0 阻断项

| ID | Status | 当前证据与后果 | 硬切目标 / owner |
|---|---|---|---|
| `INP-P0-001` | Open | 三类core profile均选择Input module；`InputDriver`是空ZST；manager Immediate且无真实ingress dependency；默认disabled；Action/Replay 0 production consumer。readiness会把类型注册误报为可用能力 | 编译`InputCapabilityContract`；只有真实ingress owner、player action tick、health和terminal teardown齐备才Ready；删除空driver。Runtime117 + Runtime42/46/50/App |
| `INP-P0-002` | Open | dynamic session先dispatch UI并在`Ok(true)`早退，press/release可能只提交一侧；Action consumed API无产品producer。capture/modal/focus变化可造成卡键和幽灵release | 物理事实先提交，`InputOwnershipArbiter`再对同一sequence生成UI/gameplay ownership；capture/focus转移按generation合成cancel/release。Runtime117 + Runtime11A/43/App |
| `INP-P0-003` | Open | persisted Action Map直接序列化临时`GamepadId(u64)`；modifier折叠、WASD特判、其他KeyCode按Debug字符串FNV，native raw code无namespace。重连/跨版本可静默错绑 | versioned `StableInputControlId`包含device class、hardware/profile identity、user slot和标准physical/logical control；未知映射fail-close或显式迁移。Runtime117 + Runtime24/25/45 |

## 6. P1 工程化差距

### 6.1 Module、Action、Context 与 Product Composition

| ID | Status | 差距 | 目标 |
|---|---|---|---|
| `INP-P1-001` | Open | `InputDriver`无字段、方法、ingress、health或teardown | 删除占位，或由真实platform event source实现driver contract |
| `INP-P1-002` | Open | descriptor未声明driver/clock/device registry/action schedule dependency | compiled dependency证明能力依赖与启动顺序 |
| `INP-P1-003` | Open | config disabled与module/manager Ready并存 | Selected/Disabled/Starting/Ready/Degraded/Unavailable分态 |
| `INP-P1-004` | Open | 无runtime-owned action evaluation phase和per-player tick owner | 固定schedule phase并按Local Player/World发布action state |
| `INP-P1-005` | Partial | direct `button_pressed`已消除snapshot clone；仍每次resolve manager、解析/分配raw字符串，Vampire仍用WASD字符串 | 脚本绑定typed action handle，frame context一次解析并批量读取 |
| `INP-P1-006` | Open | `consumed_buttons/axes`只有测试caller，无UI/capture产品producer | ownership plan生成可追踪consumption |
| `INP-P1-007` | Open | context priority只存储/排序，evaluation不使用 | 高优先级先决策并按策略阻断低优先级冲突 |
| `INP-P1-008` | Open | unknown context在generation中被创建且默认enabled | compile/load返回typed diagnostic并拒绝artifact |
| `INP-P1-009` | Open | 空active context集合表示All | 显式All/None/Set，空Set必须为None |
| `INP-P1-010` | Open | contextless action永久global active | action显式声明scope/default context/global policy |
| `INP-P1-011` | Open | duplicate action/context静默忽略或依赖构造顺序 | compiler拒绝duplicate并报告source locations |
| `INP-P1-012` | Open | binding引用未知action时静默不进入compiled action | structured diagnostics，禁止部分program |
| `INP-P1-013` | Open | action/context/control ID允许空值和任意String | validated interned ID、namespace、schema revision |
| `INP-P1-014` | Open | 同control跨context/action重复触发，无冲突图 | Consume/Share/Chord/Block策略和conflict query |
| `INP-P1-015` | Open | map替换无公开generation和held-key policy | rebuild发布generation并支持wait-release/cancel/retrigger |
| `INP-P1-016` | Open | action state全局，无Player/Controller/World scope | player/world独立context stack和device routing |
| `INP-P1-017` | Open | 无trigger/modifier/hold/tap/repeat/chord生命周期 | typed pipeline和started/ongoing/triggered/completed/canceled |
| `INP-P1-018` | Open | value只有f32相加，无Vector2/3/radial deadzone/composite | typed action value和validated composite processor |
| `INP-P1-019` | Open | 无受控action/control injection合同 | qualified synthetic source、权限、sequence和隔离target |
| `INP-P1-020` | Open | action output拥有String集合且无source/device/time/map generation | interned handle、source contribution和完整provenance |

### 6.2 Event、Device、Window 与 Wire Contract

| ID | Status | 差距 | 目标 |
|---|---|---|---|
| `INP-P1-021` | Open | RuntimeEvent viewport转内部InputEvent时丢失 | 保留window/viewport到消费终点 |
| `INP-P1-022` | Open | 通用事件无device/user/seat/source sequence/monotonic time | 统一`QualifiedInputEvent` |
| `INP-P1-023` | Open | App scan code固定0并忽略winit repeat | 同时保留physical/logical/text/location/repeat/native code |
| `INP-P1-024` | Open | 左右Shift/Ctrl/Alt折叠 | standardized physical control区分location |
| `INP-P1-025` | Open | KeyCode Debug spelling做FNV且测试锁死数字 | versioned enum/table，Debug文字不得作为ABI |
| `INP-P1-026` | Open | unidentified native code无platform/layout/generation | 带code-set和driver provenance的诊断fallback |
| `INP-P1-027` | Open | 真实ABI无法重建完整logical key集合 | 完整映射或直接传validated logical key |
| `INP-P1-028` | Open | UI metadata允许默认时间且sequence饱和重复 | monotonic clock和epoch rollover/fail-close |
| `INP-P1-029` | Open | 单窗口focus loss全局释放所有设备和touch/gamepad | window/device/seat policy；gamepad默认独立窗口焦点 |
| `INP-P1-030` | Open | cursor/motion无坐标空间、DPI、source window/device | logical/physical/relative space和transform generation |
| `INP-P1-031` | Open | line/pixel wheel直接混加，unit取最后事件 | 分单位累计或先按配置归一化 |
| `INP-P1-032` | Open | `WheelScrolled`与`MouseWheel`两套重叠事件 | 硬切单一versioned wheel event |
| `INP-P1-033` | Open | 多个float ingress未统一finite/range validation | per-kind validator和typed rejection |
| `INP-P1-034` | Open | touch仅id/phase/position | versioned touch/pen contact含pressure/tool/tilt/device/window |
| `INP-P1-035` | Open | 未连接gamepad sample仍可改变state | connection generation先于sample，stale sample隔离 |
| `INP-P1-036` | Open | gamepad settings硬编码default | per-device profile和hardware mapping配置 |
| `INP-P1-037` | Open | axis/button setting允许退化区间 | typed constructor返回error并做边界/property tests |
| `INP-P1-038` | Open | device只有name/vendor/product，无GUID/capability/power | stable descriptor、capability、mapping和generation |
| `INP-P1-039` | Open | disconnect/reconnect无slot迁移receipt | DeviceRegistry发布remove/add/rebind并清理旧贡献 |
| `INP-P1-040` | Open | file drag用`to_string_lossy`，无window/security provenance | 保留platform path/URI bytes、drop session和admission result |
| `INP-P1-041` | Open | IME仅有可选viewport，缺target revision/request ID/deadline/ack | 绑定text-target generation并返回terminal result |
| `INP-P1-042` | Open | cursor request无window/capture generation/request ID/ack | qualified cursor/capture operation |
| `INP-P1-043` | Open | rumble无统一request ID/deadline/device generation/completion | accepted/applied/canceled/failed receipt |
| `INP-P1-044` | Open | host输出命令与物理输入共用`InputEvent`语义域 | 分离Ingress、Decision、HostEffect合同 |

### 6.3 Frame State、Retention、Recording、Replay 与 Lifecycle

| ID | Status | 差距 | 目标 |
|---|---|---|---|
| `INP-P1-045` | Open | 单一Mutex覆盖state、queue、recorder和host queues | owner分离与immutable snapshot publication |
| `INP-P1-046` | Open | snapshot在锁内深拷贝sets/vectors/strings | double buffer或immutable generation lease |
| `INP-P1-047` | Open | 每sample跨ABI、多层转换和多次锁 | batch/page ingress一次validation/publication；沿用PERF owner |
| `INP-P1-048` | Partial | host request已跨frame保留；`FrameEventBuffer::begin_frame`仍无条件clear，测试明确断言未drain事件消失 | frame boundary返回consumption status；按policy保留/overflow |
| `INP-P1-049` | Open | 只合并相邻cursor/motion，其余edge可无界增长 | per-kind budget、critical edge reservation、gap marker |
| `INP-P1-050` | Open | queue/recording主要按count，payload bytes无全局上界 | count+bytes+time admission和global budget |
| `INP-P1-051` | Open | 只有局部drop/coalesce计数，无producer throttle/lag合同 | typed overflow、retry/resync和pressure telemetry |
| `INP-P1-052` | Open | public InputManager为begin/drain/record/status提供no-op默认 | required method；可选能力显式Unsupported |
| `INP-P1-053` | Open | concrete manager不实现subscribe，公共订阅永远None | 有界订阅或删除虚假能力 |
| `INP-P1-054` | Open | poison mutex取inner继续运行，无Degraded状态 | fault receipt、隔离generation、重建或terminal failure |
| `INP-P1-055` | Open | recording使用可回拨SystemTime毫秒 | monotonic clock和timebase metadata |
| `INP-P1-056` | Open | sequence饱和后重复 | checked increment、epoch rollover或incomplete terminal |
| `INP-P1-057` | Open | `InputRecording.frames`无界 | chunked journal、bytes/duration上限和streaming writer |
| `INP-P1-058` | Open | recording缺schema/build/map/device/clock/project/session/RNG/checksum | versioned manifest、chunk hash和completeness receipt |
| `INP-P1-059` | Open | replay忽略timestamp/sequence并立即提交 | timebase/frame/sequence scheduler和gap detection |
| `INP-P1-060` | Open | host-effect-shaped事件可录制并重放真实OS副作用 | 默认只录ingress/decision；effect另记且受sandbox policy |
| `INP-P1-061` | Open | replay接受缺帧、不完整和sequence gap | manifest/hash/continuity/schema preflight |
| `INP-P1-062` | Open | replay无reset、隔离target和live-input arbitration | `ReplaySession`拥有target、reset、overlay policy和cancel |
| `INP-P1-063` | Open | record/replay无typed error/progress/terminal receipt | Operation accepted/progress/completed/failed/canceled |
| `INP-P1-064` | Open | module无quiesce、device drain、effect cancel和产品health | stop ingress -> cancel effects -> drain -> release -> terminal report |

## 7. P2 完整产品能力

| ID | Status | 能力 | 前置条件 |
|---|---|---|---|
| `INP-P2-001` | Open | 本地化键名、平台glyph和layout-aware显示 | stable control/device profile |
| `INP-P2-002` | Open | 键盘布局提示与冲突解释 | physical/logical双身份和validator |
| `INP-P2-003` | Open | Action Map冲突图和context优先级编辑器 | compiled conflict graph |
| `INP-P2-004` | Open | per-device无障碍、sticky/chord/hold辅助 | player/device scope和trigger pipeline |
| `INP-P2-005` | Open | 安全rebind capture、保留键和超时取消UX | ownership arbiter和stable binding |
| `INP-P2-006` | Open | 键鼠/手柄prompt自动切换与防抖 | device activity provenance |
| `INP-P2-007` | Open | haptic curve、channel、mix和priority | qualified rumble operation |
| `INP-P2-008` | Open | pinch/swipe/rotate gesture recognizer | touch/pen contact contract |
| `INP-P2-009` | Open | sensor、pen、MIDI和专业控制器扩展 | extensible device/control schema |
| `INP-P2-010` | Open | frame history与action contribution debugger | bounded journal和contribution |
| `INP-P2-011` | Open | remote/network input authority可视化 | player/source identity和network contract |
| `INP-P2-012` | Open | deterministic replay导入/导出与diff CLI | replay manifest和oracle |
| `INP-P2-013` | Open | 官方layout/profile模板和mapping catalog更新 | signed/versioned catalog |
| `INP-P2-014` | Open | per-action latency/drop/contention telemetry | bounded diagnostics和clock |
| `INP-P2-015` | Open | provenance、ownership和capture调试UI | arbitration journal |
| `INP-P2-016` | Open | 跨设备/平台/帧率benchmark与soak harness | correctness/fault/artifact gates先通过 |

## 8. 参考引擎对照

| 参考 | 实现与测试事实 | 对Zircon的约束 | 不照搬 |
|---|---|---|---|
| Unreal Enhanced Input | LocalPlayer subsystem拥有contexts/user settings；context按priority处理并可消费低优先级冲突；rebuild可忽略held key直到release；Action含typed value、trigger、modifier、injection；TestSuite直接验证context consumption和player-mappable registration/profile | priority必须执行，rebuild必须有held policy，action/player/profile/device identity必须可持久化且有测试 | 不复制UObject/Blueprint层级，先冻结Rust owner、artifact和schedule |
| Godot | InputEvent保留device，keyboard保留physical/logical/unicode/location/echo；Input按device保存action state，支持joy GUID mapping和buffered/accumulated input；tests覆盖key/mouse/event变换 | ABI后不能丢window/device；键盘不能退化成哈希；state和disconnect必须per-device | 不采用全局Singleton作为Zircon最终authority |
| Bevy | keyboard event保留window、physical、logical、repeat；raw与filtered gamepad event分层；gamepad是Entity并带validated settings；测试覆盖disconnect/reconnect保留settings和sample filtering；gamepad独立window focus | connection/sample顺序、raw/filtered层、typed settings错误和focus policy必须显式 | ECS Entity不能直接作为跨存档stable hardware identity |
| Fyrox | `InputState`源码明确称其为simple shortcut，明确不保存device origin并建议复杂场景使用event-based input | Zircon raw snapshot只能作为受限compat shortcut，不能是shipping gameplay主合同 | 不把Fyrox shortcut当目标上限 |
| Unity Graphics | DebugManager新路径拥有InputActionMap enable/disable、composite和performed callback；legacy路径有明确repeat mode/delay，tests覆盖调试输入行为 | 即使调试UI也要有action-map lifecycle、composite和repeat policy | Graphics不是完整Unity玩家输入仓库，不外推设备/存档/网络能力 |

参考结论不是追求同等代码量，而是必须具备identity、owner、priority、generation、lifecycle、persisted artifact、product consumer和可运行的失败测试。只有正确性门禁通过后，才有资格做同硬件、同事件序列的竞争性性能比较。

## 9. 目标架构、Owner 与硬切边界

```text
PlatformInputSource
  -> InputIngressBroker(batch/page + validation + monotonic sequence)
  -> StableDeviceRegistry(window/device/user/seat/generation)
  -> QualifiedPhysicalState(immutable frame generation)
       -> Runtime UI routing decision
       -> InputOwnershipArbiter(capture/context/player policy)
       -> CompiledActionProgram(per-player typed action lifecycle)
       -> DeterministicInputJournal(ingress + decision, bounded chunks)
  -> InputHostEffect operations(request/ack, never disguised as ingress)
```

Runtime117拥有输入纵向组合、状态一致性、Action产品接线、stable binding和record/replay安全闭环。Runtime11A拥有UI focus/IME语义；Runtime38拥有Local Player/Controller消费；Runtime24/25/45拥有通用identity与持久artifact backend；Runtime43和Interface报告拥有dynamic ABI/page；Runtime42/46/50拥有module/service/manager kernel；Runtime99q/App拥有platform host。性能报告继续拥有通用lock/batch benchmark，本篇只规定正确性前置条件，不转向tooling优化。

硬切删除：空`InputDriver`；shipping脚本raw key字符串主入口；无schema的`GamepadId`/FNV持久绑定；UI早退删除物理事实；InputManager必需能力no-op默认；host effect伪装成input event；忽略时间、完整性和副作用的replay。迁移期兼容入口必须命名`LegacyRawInputShortcut`并明确不允许写入shipping binding。

## 10. 重构里程碑

### M117.0 · Truth Freeze 与 RED Repro

- 建立module readiness、action/replay product reachability source guard；
- 建立UI吞release卡键、capture转换幽灵edge、设备重连错绑、replay OS副作用四个RED测试；
- 修正Runtime12文字：physical fact first，UI/gameplay consumption为独立decision。

### M117.1 · Capability 与 Ingress Owner

- 删除空driver或迁入真实event source、health、backpressure和teardown；
- readiness只在ingress、clock、device registry和action tick齐备时Ready；
- per-event ABI迁移到bounded batch/page。

### M117.2 · Qualified Device、Control 与 Wire

- 引入window/device/user/seat/clock/sequence/generation；
- 稳定keyboard physical/logical/native identity和controller GUID/profile/user slot；
- connection先于sample，stale generation拒绝。

### M117.3 · Physical State 与 Ownership Arbitration

- 物理事实先发布，UI/gameplay/editor对同一sequence做decision；
- capture/focus/modal变化按generation合成cancel/release；
- 按window/player/device隔离state，删除全局focus release。

### M117.4 · Compiled Action Product Cutover

- compiler校验ID/context/priority/conflict/trigger/modifier/value/generation；
- Local Player/Controller schedule成为唯一shipping action owner；
- Vampire、模板和脚本硬切typed action。

### M117.5 · Profile、Rebind 与 Editor

- versioned user/device/profile mapping、迁移、冲突查询和损坏隔离；
- Editor使用同一compiler完成Action Map authoring/rebind产品闭环；
- missing device和跨平台fallback返回typed diagnostics。

### M117.6 · Journal、Replay 与 Host Effects

- chunked manifest、monotonic time、sequence、checksum、completeness和bounded writer；
- replay preflight、isolated target、reset/live policy、paced schedule和gap rejection；
- host effect分离为request/ack operation，默认回放不触发真实OS副作用。

### M117.7 · Fault、Soak 与 Competitive Evidence

- 完成quiesce/drain/device/effect terminal lifecycle和Degraded/Failed health；
- correctness/fault/multi-window/reconnect/layout/soak通过后再做contention/latency优化；
- 同硬件、OS、事件序列、窗口、帧率和统计协议对照Unreal/Godot/Bevy。

## 11. 验收矩阵

| Gate | Status | 验收内容 |
|---|---|---|
| `INP-G01` | Fail | Input Ready要求真实ingress、action tick、health、teardown，空driver不存在 |
| `INP-G02` | Fail | production有Local Player action consumer，Vampire/模板不再用raw key字符串 |
| `INP-G03` | Fail | UI吞release、capture/modal/focus切换不会留下held或幽灵edge |
| `INP-G04` | Fail | physical/UI/action decision可按同一sequence关联并解释ownership |
| `INP-G05` | Fail | context priority真实阻断低优先级冲突，覆盖Share/Consume/Block |
| `INP-G06` | Fail | unknown/duplicate/empty ID和unknown binding target使compile失败 |
| `INP-G07` | Fail | All/None/Set无歧义，空Set不启用全部action |
| `INP-G08` | Fail | rebuild对held control执行显式policy，generation不混用 |
| `INP-G09` | Fail | trigger/modifier/hold/tap/repeat/chord/Vector2有确定性测试 |
| `INP-G10` | Fail | per-player/controller/world state/context/device routing隔离 |
| `INP-G11` | Fail | persisted binding无临时gilrs ID、Debug hash、无namespace raw code |
| `INP-G12` | Fail | 同controller重连保留slot/settings，不同controller不继承错绑 |
| `INP-G13` | Fail | mapping artifact有schema/profile/hardware identity和迁移策略 |
| `INP-G14` | Fail | keyboard wire覆盖physical/logical/text/location/repeat/native namespace |
| `INP-G15` | Fail | Space/Enter/arrows/F-keys经真实App ABI到达action |
| `INP-G16` | Fail | internal event保留window/device/user/seat/time/sequence/generation |
| `INP-G17` | Fail | sequence溢出不重复，clock回拨不改变顺序 |
| `INP-G18` | Fail | line/pixel wheel不混量纲，legacy duplicate variant删除 |
| `INP-G19` | Fail | multi-window/DPI/relative/captured cursor坐标与generation正确 |
| `INP-G20` | Fail | touch/pen保留device/window/pressure/tool并拒绝非法值 |
| `INP-G21` | Fail | connection先于sample，stale/disconnected sample不改变state |
| `INP-G22` | Fail | settings拒绝退化区间并通过边界/property test |
| `INP-G23` | Fail | focus loss只释放policy拥有输入，不全局释放gamepad/touch |
| `INP-G24` | Fail | file drag保留无损path/URI、source window和admission provenance |
| `INP-G25` | Fail | IME/cursor/rumble含target generation、request ID、deadline和ack |
| `INP-G26` | Fail | host effect与physical ingress类型、queue、recording policy分离 |
| `INP-G27` | Fail | immutable snapshot/lease发布，读者不在全局Mutex深clone |
| `INP-G28` | Fail | frame boundary对未drain数据有显式policy和指标，不静默丢弃 |
| `INP-G29` | Fail | queue同时受count/bytes/time/global budget约束 |
| `INP-G30` | Fail | overflow有typed gap/backpressure并保留critical release/cancel |
| `INP-G31` | Fail | InputManager必需方法无no-op；可选能力返回Unsupported |
| `INP-G32` | Fail | mutex fault进入Degraded/Failed并隔离损坏generation |
| `INP-G33` | Fail | recording manifest含schema/build/map/device/clock/session/checksum |
| `INP-G34` | Fail | 长录制bounded chunk streaming，无无界frames/payload |
| `INP-G35` | Fail | replay按time/frame/sequence调度，乱序/gap/hash/incomplete拒绝 |
| `INP-G36` | Fail | replay有isolated target/reset/live policy/cancel且默认无OS副作用 |
| `INP-G37` | Fail | record/replay提供accepted/progress/terminal receipt |
| `INP-G38` | Fail | stop执行停ingress、cancel effect、drain、disconnect、terminal health |
| `INP-G39` | Fail | correctness/fault/reconnect/multi-window/layout/soak先于benchmark |
| `INP-G40` | Fail | 同协议竞争性对照完成前不宣称性能或表现优于Unreal |

## 12. 状态与产出记录

| 项目 | 状态 | 证据 |
|---|---|---|
| Input contract/runtime逐文件复核 | review_complete | 46文件、3,797行、119,164 bytes |
| focused tests复核 | review_complete | 21文件、3,477行、62 tests；确认frame事件丢弃合同被测试固化 |
| dynamic/App/script/sample产品链 | review_complete | 56文件、16,815行；Action/Replay 0 production consumer |
| 五套参考引擎实现与测试对照 | review_complete | 28文件、21,728行、44 tests |
| Finding与Gate账本 | review_complete | 3 P0 Open / 62 P1 Open / 2 P1 Partial / 16 P2 Open / 40 Gate Fail |
| Production重构 | pending | 本篇未改production/tests/Cargo/ABI |
| 动态、设备、回放、soak、性能验证 | pending | 本轮未运行，不能作竞争性结论 |

Runtime117 review完成不等于输入系统完成。任何Input ABI、Runtime UI propagation、module profile、script host、device identity、recording schema或product consumer变化都必须重读current source并刷新状态；Runtime12 open failure未关闭前，不得把局部poll budget或ignored microbenchmark当作输入栈验收。
