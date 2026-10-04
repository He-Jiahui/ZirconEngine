---
related_code:
  - zircon_runtime/src/dynamic_api/session/events.rs
  - zircon_runtime/src/dynamic_api/session/input_events.rs
  - zircon_runtime/src/dynamic_api/session/runtime_ui.rs
  - zircon_runtime/src/dynamic_api/session/state.rs
  - zircon_runtime/src/dynamic_api/session/registry/session_store.rs
  - zircon_runtime/src/core/manager/service.rs
  - zircon_runtime/src/core/runtime/handle/resolution.rs
  - zircon_runtime/src/input/runtime/default_input_manager.rs
  - zircon_runtime/src/input/runtime/event_buffer/frame.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/01/2026-07-18-runtime-input-framework-static-review.md
  - docs/plans/zircon_runtime/runtime/12-input-stack-and-action-mapping.md
  - docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md
  - docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/ApplicationCore/Public/GenericPlatform/AsyncInputConsumer.h
  - dev/UnrealEngine/Engine/Source/Runtime/ApplicationCore/Private/GenericPlatform/AsyncInputConsumer.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/ApplicationCore/Private/GenericPlatform/MultiThreadedInputMessageHandler.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Application/SlateApplication.cpp
  - dev/godot/core/input/input.cpp
tests:
  - current dynamic session events/input conversion 2 of 2 Rust files and 4 inline tests reviewed
  - supporting session registry, runtime UI, manager resolution and default input retention owners reviewed
  - M0 static performance contract 3 of 3 passed after RED
  - focused rustfmt plus 1.94.1 and scoped diff check passed
  - current-source Cargo, WPR, allocator and product input traces pending
doc_type: implementation-evidence
status: m0_static_complete_dynamic_blocked
---

# Runtime dynamic session输入准入当前架构复审（2026-08-23）

## 范围与当前性

已逐行复读`dynamic_api/session/{events,input_events}.rs`当前 **2/2** 个Rust文件。实施前为**1,071行、45,039 B、4 tests**；M0后为**1,074行、45,057 B、4 tests**，manifest SHA256为`4395e607c8542f710d483287691f0387247171c113676fe96c859e38e27fbb27`。同时沿调用链复核`ffi -> session_store -> events -> runtime_ui/input manager -> frame buffer`七个直接owner；其中`runtime_ui.rs`、`state.rs`、`default_input_manager.rs`已有其他Session改动，本轮只读并保留，未混入修改。

## current source数据流与瓶颈

`handle_event`在`session_store.rs:181-203`取得整个`RuntimeDynamicSession` mutex后执行完整动作。以pointer move为例，同一锁内依次执行runtime UI dispatch、input manager提交和`LevelSystem::with_world_mut` camera更新；touch move还提交`CursorMoved`与`Touch`两条事件。OS/host采样频率因此直接成为session串行、UI route、manager解析/锁和World mutation频率。

每次`submit_input_event`都clone带服务名的manager handle，进入core services mutex，按name/index/generation校验并clone type-erased service Arc，再downcast和clone typed Arc；`DefaultInputManager::submit_event`随后取得统一state mutex。构造期虽然已经成功解析manager，却只保存generation handle。缓存Arc不能直接作为小修：必须先冻结active session期间input module是否允许deactivate/reactivate，以及旧Arc在generation失效后的可见性。

末端`FrameEventBuffer`只合并**相邻**`CursorMoved` latest和`MouseMotion` accumulated。它发生在UI route、manager resolution、state mutex、状态更新及event recorder之后，因此只减少后续保留量，不减少前述主线程工作。gamepad absolute axis不在该buffer合并；`DefaultInputManager`虽把同axis transition更新为最后值，但用线性`iter_mut().find`，并仍保留每个raw frame event。frame events按`begin_frame`清空且recording另有边界是正确止损，但一帧内entry/bytes/age仍无硬上限。

runtime UI对每个非pointer event逆序遍历surface，并为除最后一个以外的surface clone完整`UiInputEvent`；keyboard/IME包含owned Strings。pointer未capture时也对surface逐个clone/route，每次调用`rebuild_dirty`。稳定dirty generation可能使rebuild快速返回，但surface visit、event clone和route仍随`events * surfaces`增长。keyboard还把typed key code投影成`physical_key/logical_key String`，属于UI边界成本，不能扩散到核心输入身份。

## 参考引擎依据与结构判词

Unreal `AsyncInputConsumer.h:13-19,81-115`把producer与单consumer线程分开，使用MPSC队列，明确game-thread consumer对absolute analog按`(axis, device)`保留每次drain的最后样本，并复用成员scratch避免稳态帧分配；`AsyncInputConsumer.cpp:50-126`实现一次drain、一次last-index构建和有序跳过被覆盖样本。`MultiThreadedInputMessageHandler.cpp:13-24,105-125`只把平台事件fan-out到consumer，不在producer callback同步跑完整Slate/gameplay逻辑。`SlateApplication.cpp:6478-6517`则在consumer侧完成mouse route。

Godot `input.cpp:1581-1643`在`use_accumulated_input`下先尝试把新事件累积到buffer尾，不能合并的事件才入队；`flush_buffered_events`在明确边界逐项解析，并说明dispatch可能释放锁。两者共同支持的不是“把所有输入丢给后台线程”，而是：producer只做有界准入，按语义分类合并连续量，在唯一consumer/frame barrier保持边沿和UI顺序。

因此PERF-MVP-003/334的M1结构方向应为：

1. 在session mutex之外建立有生命周期保护的input admission owner；platform/input线程只标准化typed payload并写有界MPSC。队列分类为lossless edges/barriers、latest absolute levels和accumulated relative deltas。
2. button/key/touch begin/end/cancel、focus、resize/scale、IME commit与drop等barrier严格保序；cursor/touch move按pointer latest，absolute gamepad axis按device/axis latest，raw mouse/wheel按frame accumulated。合并不得跨越geometry/focus/capture barrier。
3. runtime frame consumer一次取得session action，按有界event/time budget drain；先处理geometry barrier，再由runtime UI单一owner决定consume，最后批量提交unhandled gameplay input和一次camera/world delta。`InputManager`增加batch入口并一次锁state，而不是每event重新解析服务和加锁。
4. active-session module generation契约冻结后，session持有typed input Arc；若运行期确需module replacement，则用generation publication在frame barrier原子换代，不能在每event走全局registry来模拟安全。
5. UI batch复用event storage和route scratch；typed/compact key/control identity留在热路径，String只在UI/ABI/diagnostic边界按需投影。多surface传播必须记录visits与clone bytes。

## 本轮M0

空`RuntimeUiSurfaceSet`以前仍先构造keyboard physical/logical Strings、clone keyboard/IME text、分配gamepad analog control String并推进UI sequence，随后零次surface遍历返回false。本轮把non-pointer UI event改为`FnOnce(UiInputEventMetadata) -> UiInputEvent`惰性构造器，并让pointer dispatch先执行empty guard。无runtime UI时，UI metadata/build/String clone路径从每相关事件一次降为零；核心`InputEvent`提交、camera行为、错误状态以及存在runtime UI时的route/consume顺序不变。

`tools/tests/test_runtime_session_empty_ui_input_m0_performance_contract.py`先得到2 failures + 1 error的RED，实施后3/3 GREEN；测试47行、1,744 B、SHA256 `1b84b87e3d782ee39048e8f882364f175a8baea9987441a18566e4cece5b6414`。focused `rustfmt +1.94.1 --edition 2021 --check`和scoped diff check通过。受管Cargo不可执行，现有Rust行为测试未运行；静态路径收敛不冒充wall time或功耗改善。

## 动态验收矩阵

按runtime UI surfaces **0/1/8/64**、events **125/500/1,000/10,000 Hz**、pointers/devices **1/8/64**、consumer stall **0/1/16/100 ms**、payload **0/16 B/1 KiB/1 MiB**运行。记录ABI/session lock wait/hold、admission entries/bytes/oldest age、drop/coalesce by class、manager resolutions、input state locks、UI surface visits/event clone bytes/layout rebuild、World mutations、drain p50/p95/p99、RSS和energy。

验收门为：空runtime UI的UI metadata/build/String clone严格为0；每frame manager resolution和state lock为常数；连续量work接近`unique(pointer/device/control)+barriers`而非raw sample数；edge/barrier零丢失零重排；队列entries/bytes/age硬有界且过载原因可观测；10分钟1 kHz输入RSS不随总事件数增长。随后用F2无UI和F4有UI产品副本执行WPR/ETW、allocator与像素/行为对拍。RenderDoc不负责输入CPU归因，仅在该变更影响present时做GPU/draw parity。current-source executable尚不可得，所以本切片继续留在`pending.md`。
