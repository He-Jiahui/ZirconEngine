---
related_code:
  - zircon_app/src/entry/runtime_entry_app/pointer_input
  - zircon_app/src/entry/runtime_entry_app/keyboard_input
  - zircon_app/src/entry/runtime_entry_app/ime_input
  - zircon_app/src/entry/runtime_entry_app/gamepad
  - zircon_app/src/entry/runtime_entry_app/device_events
  - zircon_app/src/entry/runtime_library/runtime_session.rs
  - zircon_runtime/src/input
  - zircon_runtime/src/dynamic_api/session/events.rs
  - zircon_runtime/src/dynamic_api/session/input_events.rs
  - zircon_runtime/src/dynamic_api/session/state.rs
  - zircon_runtime/src/dynamic_api/session/registry
  - zircon_runtime/src/dynamic_api/session/runtime_ui.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/runtime/12-input-stack-and-action-mapping.md
  - docs/plans/zircon_runtime/runtime/12/failure-2026-07-19-app-entry-input-and-gamepad-storm-budget.md
  - docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/ApplicationCore/Private/GenericPlatform/GenericPlatformInputPump.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/ApplicationCore/Private/GenericPlatform/AsyncInputConsumer.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/ApplicationCore/Private/HAL/InputThread.cpp
  - dev/bevy/crates/bevy_input/src/mouse.rs
  - dev/bevy/crates/bevy_gilrs/src/gilrs_system.rs
tests:
  - zircon_runtime/src/input/tests
  - zircon_app/src/entry/runtime_entry_app/gamepad/polling/drain_budget.rs unit tests
  - current-source managed Windows Cargo pending
  - WPR/Tracy input storm matrix pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime input ingress 当前源码性能复审（2026-08-14）

## 范围与证据边界

本轮完整复读 App 输入 producer 23 个文件、1,129 行，以及 `zircon_runtime/src/input/**`
29 个文件、3,074 行，共 **52/52 个文件、4,203 行**。同时沿实际调用图核对 App
`RuntimeSession::handle_event`、Runtime dynamic FFI/session registry/event adapter、Core manager
service resolution 与 Runtime UI dispatch；这些共享 owner 不计入 52 文件覆盖数。

Runtime input 目录当前干净，App 与 dynamic API 相关文件已有其他 Session 的未提交修改，本轮不接管
实现。现有 pointer bounded test 和 10/100/1,000 action scaling test 只证明局部数据结构合同；没有
current-source managed Cargo、真实 125/500/1,000 Hz producer trace 或 1k/10k/100k burst 数据，故
本记录不声明性能验收。

## 当前调用链与结构性瓶颈

1. App 对 cursor、raw motion、wheel 和 gamepad axis 的每个原始样本都同步调用一次 V1
   `handle_event`。Runtime 的帧缓冲虽然能合并相邻 cursor/motion，但这发生在 ABI、session lookup、
   dispatch 和 manager locking 之后，无法收回入口成本。
2. 普通未被 Runtime UI 消费的事件，静态调用图至少执行全局 session registry lock、session
   lifecycle begin/finish 两次 lock、session lock、Core service registry lock、InputManager state lock，
   即每事件至少 **6 次 mutex acquisition**。这还没有计入 UI hit-test、字符串/事件 clone 与业务逻辑。
3. 存在 Runtime UI 时，每个输入事件按 surface 反向路由，并在路由前调用 `rebuild_dirty`；多 surface
   路径会 clone event。指针 move 还执行 hit-test、hover diff、route/bubble 与 transient state 更新。
   这使高频样本在进入 gameplay input 前就可能按 `events x surfaces x route work` 放大。
4. `DefaultInputManager` 在每个 `GamepadAxis` 上对本帧 transition `Vec` 做 `iter_mut().find`，最坏为
   `O(events x active_axes)`；`FrameEventBuffer` 不合并 gamepad axis，因而保留全部样本。生产输入目录
   静态可见 30 个 clone call、6 个 collect call、30 处 BTree 容器提及和一个线性 `iter_mut` 热点。
5. `frame_snapshot` 在 input-state lock 内 clone 多个列表、集合、字符串与 transition；action evaluator
   每次 evaluate 重新构建两个 frame-axis `BTreeMap`、consumed/active/output `BTreeSet`，并多次 clone
   action id。10/100/1,000 bindings 测试只计 source visits，不计 allocation bytes、p95 或 10k 规模。
6. gamepad 已有 256 events/2ms drain 与每 gamepad 32 rumble effect 上限，但每个 drained event 仍同步
   跨 ABI；gilrs 也没有向 reactive event loop 提供 producer wake。预算限制单次卡顿，却可能把 backlog
   延迟到以后帧，当前没有 queue age/peak/coalesce counters。

Event recorder 只有显式启用时才执行 `SystemTime::now` 和 event clone，这不是默认稳态主因。按钮、
touch、IME、lifecycle 与连接状态属于有序边沿，不能为了缩短队列静默丢弃。

## 历史产物审计

E 盘已有 2026-08-08/09 pointer-storm JSON，但均不满足当前验收：`ui-debug` 的 1,000 次 move 用时
22,095.508 ms，100 次响应采样中 29 次 unresponsive，peak working set 1,377,976,320 bytes，随后
进程退出；另一份 `default` 记录 1,000 次 move 用时 42,754.907 ms、0/100 unresponsive、working
set 增长 4,558,848 bytes，但没有 executable/source fingerprint。另有 debug 仅完成 102/1,000、
private bytes 增长 297,320,448 后退出，Softbuffer/Vulkan 记录也分别以 2173/101 退出。

这些脚本通过 `SetCursorPos` 加 sleep 注入 OS cursor move，没有 ABI calls、lock wait、UI route、queue
或 input-to-frame counters；部分 JSON 还引用 C 盘旧二进制。它们只能证明旧产品链曾有响应/稳定性
问题，本轮不复用其 C 盘产物，也不把不同旧二进制的数字拼成 before/after。

## 参考引擎结论

- Unreal `GenericPlatformInputPump.cpp:90-159` 按 device thread affinity 轮询，并把输入线程事件 fan-out
  到 consumer queue；`AsyncInputConsumer.cpp:50-121` 在单 consumer thread drain，复用 scratch
  capacity，并仅对绝对 controller analog axis 按 `(axis, device)` 保留最新样本，其他事件保持原序。
  `InputThread.cpp:81-104` 把采样线程等待与实际 poll trace 分开。它证明“高频采样、线程安全入队、
  有界批消费、按语义合并”必须是同一套合同。
- Bevy `mouse.rs:250-356` 把 mouse motion/scroll 累加为每帧资源；`gilrs_system.rs:38-112` 在一个系统
  drain gilrs 后再发布 ECS messages。Bevy 本身仍保留 raw messages，因此只可借鉴帧聚合层，不能把其
  所有事件量假设为 Zircon ABI 的合理预算。

Unreal 对绝对 analog 使用 latest，Bevy 对 relative motion 使用 sum；两者共同否定“一种 coalesce
规则覆盖所有输入”。cursor position/absolute axis 取 latest，relative motion/同单位 scroll 累加，
边沿事件必须形成顺序 barrier。

## 统一优化计划

继续使用 **PERF-MVP-012** 与 **PERF-MVP-426**，修复 owner 保持 Runtime12/Runtime10，不新增重复
failure。Runtime10 新增版本化 batch ingress，不原地扩形 V1：App 在 pump 周期内用可复用 scratch
按 viewport/device/pointer/control 聚合 relative delta、latest cursor 与 absolute axis；button、touch、
IME、lifecycle、connection 等边沿到来前先 flush 当前聚合段，使边沿观察到正确的先前位置/状态。

每个 batch/page 只进入一次 session registry 和 session lock。Runtime 解析后一次解析/缓存有效的
InputManager service，并在一次 manager lock 下应用批数据；generation 失效仍要 fail closed，不能用
裸 `Arc` 绕过卸载合同。Runtime UI 提供 batch dispatch，复用布局与 route scratch；只有前一事件确实
产生 layout dirty 时才重建，不能无条件把整个 batch 只 layout 一次而破坏后续 hit-test 正确性。

batch 同时设 entry、bytes、age 和 page 上限，绝对状态按 key 覆盖，relative delta 饱和累加，边沿
事件不得 drop。轴 transition 用按 `(gamepad, axis)` 的帧内索引消除线性 find。action map 在修改时
编译 immutable action/context/axis lookup，evaluate 使用复用 scratch 或稳定 dense id；是否替换
BTree 只有在确定性顺序合同和 allocation/profile 数据齐全后决定。

gilrs producer 通过既有 capacity-one wake owner 唤醒 reactive loop；不得以持续 `Poll` 掩盖无 wake。
现有 V1 单事件路径保留兼容但不作为新产品热路径，source-shape tests 升级为 batch 行为、顺序、
容量和 counter tests。

## 动态验收矩阵

- 125/500/1,000 Hz cursor/raw motion/gamepad axis，1k/10k/100k burst；0/1/4 Runtime UI surfaces；
  action bindings 10/100/1k/10k，30/60/120 Hz consumer。
- 记录 producer events、batch/pages、ABI calls、各层 lock acquisitions/wait、UI rebuild/hit-test/clone、
  transition probes、snapshot/action allocations 与 clone bytes、queue entries/bytes/age、coalesce/drop、
  input-to-frame p50/p95/p99、main-thread CPU 和功耗。
- coalescible 样本的 ABI/registry/session/manager 次数按 batch/page 而非 event 增长；相同最终输入的
  1k 与 100k burst 产生等价 frame state。按钮/touch/IME/lifecycle 边沿 lossless、有序，指针按下时
  观察到 barrier 前最后位置。
- entry/bytes/age 达硬上限后行为可观测且内存稳定；停止消费 60 秒后 queue/RSS 不继续增长；
  reactive idle 收到首个 gamepad 输入后在预算内 wake。WPR/Tracy 用当前产品入口核对锁等待、CPU 栈
  和功耗，不以微基准或旧二进制替代。

在 managed Cargo、真实 producer storm、WPR/Tracy、功耗基线和独立审查齐全前，App 输入 producer
与 `zircon_runtime/src/input/**` 继续保留在 `pending.md`，不进入 `review.md`。
