# Runtime220：Session 输入接纳与消费者终结源码记录

日期：2026-10-17。任务：optimize-20261013-01a13451。

最初 17 个源文件和 12 个行为回归已写入实际 checkout，并完成限范围独立源码审查 SOURCE_ONLY PASS；这是历史交付阶段的结论。后续消费者终结增量见下文。全部原生、GUI、公共全 union guard 与性能验收仍 OPEN，不能据源码审查关闭完整 M220.1。

## 已接入的源码行为

- Session 持有自己领取的精确 opaque attachment 凭据；构造、提交、帧推进及释放按该凭据核验。首次真实接纳后才进入 Receiving，Gap/Detached 不自动重新接入；旧凭据清理不能影响后代 Session。
- InputManager 新增 required Rust 接口 submit_batch_with_admission(Vec<InputEvent>, InputBatchLimits) -> Result<InputBatchSize, InputBatchRejection>。实际 DefaultInputManager 在同一物理状态锁内先完成整批有限值、容量、游戏手柄生命周期及累加边界预检，再应用事件。非法尾项不能留下已应用前缀或 recorder 部分写入。
- 实际容量以 Vec.capacity 和事件中所有拥有的 String.capacity 做 checked 计算。接纳 policy 默认每帧 8192 事件、4 MiB 拥有容量及 1 秒 runtime receive age；这些是工程策略值，尚非测得性能结果。
- 实际计数/字节 FrameCapacity 超限使用保留的真实 FocusLost 重置并进入 Gap；ReceiveExpired、ReceiveClockAhead 属普通 Rejected，不触发 Gap 或清理当前消费者。非法批次不消耗正常接纳容量。
- Touch 的 CursorMoved 与 Touch 组成一个物理子批次；物理批次成功后才继续原 UI/camera 分发。保留 InputConfig.enabled 对初始 action-map 的原含义，不以它禁用物理状态。
- 原单事件 Rust API 保留，C ABI 不变。新 Rust trait 方法没有 noop 或逐事件伪原子的 default；所有已查第一方实际实现中仅 DefaultInputManager/Facade alias 需要接入。外部实现兼容及原生编译仍待验证。

## 实际写入证据

唯一 source receipt：.codex/outbox/reports/optimize-20261013-01a13451/runtime220-session-ingress-admission-source-write-receipt.json，176113 bytes，SHA256 d538df75fea031c51d900d95909d644270bc53e9b9d0149fafabb9b3ce2ac5e8，phase COMPLETE_SOURCE_ONLY。保留其原始不可变交付事实。

源码 writer PowerShell 33476 / Python 49688，chunk 013d67，outer exit 0、optimize 0；13 个 before 与 4 个 absent 输入精确，17 份 delta 正反核验、写入、fsync、双次原始字节读回和末核完成，7 个 Git 子进程已 communicate/wait 并退出 0。任务没有 stage/commit/push 或创建 worktree。

root671 独立机械读回：PowerShell 15700 / Python 51908，chunk 7dc5a0，outer exit 0；固定 receipt 双读精确、17 个当前 after 原始字节逐一双读精确，root658 的 action_mapping.rs 夹具修复保持不变。该结果不是语义或公共全 union guard PASS。

## 回归与后续合批

已写五个真实 DefaultInputManager 原子批次用例、五个 InputDriver 接纳生命周期用例、两个正式动态 Session 路由用例，全部 UNRUN。Luna36 唯一 post-change reader（45b5e0，outer exit 0，rg17240 communicate/wait 0）已核实十二例均为可达的 #[test]。Session 真实链为 lib.rs → dynamic_api/mod.rs → dynamic_api/session.rs 的 cfg(test) tests → session/tests/mod.rs 的 physical_input_ownership，两个真实 FQN 为：

- dynamic_api::session::tests::physical_input_ownership::runtime220_session_owns_exact_ingress_and_resets_before_core_teardown
- dynamic_api::session::tests::physical_input_ownership::runtime220_session_touch_is_atomic_and_capacity_gap_releases_physical_state

同一 runtime row2 已登记十二个精确 FQN 的追加路由与独立 named AND，执行仍 UNRUN；它们不能补足旧 floor172。原 178 selectors 不匹配十二例的事实只对应实际读到的合同，未据此宣称历史 source pin 已通过。

沿既有 31 Cargo + 2 Python 后续批次，把真实新 FQN 追加到 runtime-scoped-regressions 的 substring OR 选择条件，并独立要求所有新增身份通过。原 floor172 不能由新增用例补足。保留全部旧 named/subset/typed/config、22 个性能组及 Runtime03 的绝对 P95、原相对和 RSS 门槛。Jenkins 合批交接位于 .jenkins/records/runtime-editor-optimize-01a13451-20261017-editor09-owned-close-source-successor.json，落盘不是队列 ACK 或通过。

最初十七源增量的独立核心审查为限范围 SOURCE_ONLY PASS；后续消费者增量需单独核对最终源码和审查结论。原生编译、回归及真实时间/内存/锁等待测量继续 OPEN。V1 只约束 runtime 接收后的时间，未实现完整 host timestamp、source epoch/sequence、seat/user 协议。原 M220.1/M220.2、P0/P1 和完整输入计划均未关闭。

本轮未创建构建产物、缓存、源码副本或临时 helper；登记 source receipt 与失败证据继续保留，实际释放空间 0 B。

## 阶段名称与后续增量边界

此前使用的 M220.1a 只是本地工程阶段标签。原计划定义 M220.1 和 M220.2，没有原 M220.1a/M220.1b 子里程碑；消费者终结属于原 M220.2 的所有权语义范围。本记录不把物理 FocusLost 重置视作全部消费者已终结，也不把工程 policy 当性能测量。


## 消费者终结：最终源码与限范围审查

10 个产品文件和 4 个测试文件已写入实际 checkout（14 unique，先前口头 11 个产品文件计数已纠正）。只有 Accepted FocusLost、精确当前 attachment 的真实 FrameCapacity 与精确 detach 成功，才同步结束 camera drag、所有 RuntimeUiSurface 的 capture/press/交互 timer 和 IME generation。ReceiveExpired、ReceiveClockAhead、其他 Rejected 与旧凭据不清理当前消费者；Gap/Detached 的 gate 位于 UI-first text/IME 之前。

独立核心审查发现的两处错误已修复：精确 detach 成功后立即记录 local detached，再执行可能失败的 UI 清理；window pump 返回错误时保留确切未交付的 high-precision/lock host 释放元数据，成功交付后才清除并去重。失败不会恢复 capture/press/drag，真实 text 回调可重试 pending 清理而不继续路由文字。Toast/typeahead 内容过期机制保留。

Session 直接拥有 camera、UI surface Vec、IME route 与 host request queue；attachment 只在构造时赋值，没有同 Session 重新接入。新 Session 创建新 Core/consumers；旧 pending 只触及旧 owner 字段，不解析当前 generation 作为替代凭据。这一源码范围不证明 V1 host wire 的跨 Session/seat 完整资格；原计划仍 OPEN。

作者最终机械核对：PowerShell 47796 / Python 48372 / chunk 7152c5，outer exit 0、optimize 0、children []；14 个最终 after 精确，原始 17 源 receipt 不变。早期 apply tool 终态 UNKNOWN，后续实际读回确认只写入三个文件，再由 guarded writer 收敛剩余增量；保留该历史，不把未知工具终态改写为成功。

独立 Sol28 对实际 A–D 产品 body、callee ab6063/0b60db、四个真实测试与最终 pins 的结论为 **限范围 SOURCE_ONLY PASS**，未见剩余具体静态 blocker；审查者没有新增 OS reader，采用作者实际执行证据。此前审查 reader 的 precreation 拒绝无 PID、无执行，历史继续保留。这不是公共全 union guard 或原生通过。

四个新增行为回归（全部 UNRUN，追加同一 runtime row2 OR，并分别要求 named AND）：

- `dynamic_api::session::tests::physical_input_ownership::runtime220_focus_loss_and_detach_end_real_camera_drag`
- `dynamic_api::session::ime_composition_route::tests::runtime220_cancel_active_retires_delayed_ime_callbacks_without_reusing_generation`
- `dynamic_api::session::runtime_ui::tests::runtime220_focus_loss_and_gap_retire_ui_capture_and_block_ui_first_text_ime`
- `dynamic_api::session::runtime_ui::tests::runtime220_failed_ui_terminalization_retains_host_release_until_detached_retry`

这些用例分别操作真实 Scene transform、实际 IME encode/consume、高交互 UiSurface 捕获、真实 MissingNode 失败和修复后的 text 回调，核对 FocusLost/Gap/detach、旧 IME callback 拒绝及两种 host release 各一次。原有 `dynamic_api::session::events::tests::checked_gamepad_session_admission_returns_the_manager_generation_rejection` 仅适配 mutable Session 与精确 typed rejection，原 frame/recorder 断言和 identity 保留；不属于 NEW exclusion，也不能通过追加 selector 扩宽原 floor 的统计谓词。

最终 raw after 映射（repo-relative；实际 before→after 与失败历史保留在同一 Jenkins 交接及 artifacts.json，未复制源码）：

| 文件 | bytes | SHA256 |
|---|---:|---|
| zircon_runtime/src/dynamic_api/camera_controller.rs | 7248 | 56f8548de9c02af04518052ce0603739524e2ff40854be82dd9b291ac0569549 |
| zircon_runtime/src/dynamic_api/session/construction.rs | 20027 | b8f731741a72344d661546b6f9648b99c4fb41e1022b2e7387cb35778aa11cf7 |
| zircon_runtime/src/dynamic_api/session/events.rs | 29659 | 17d3837a678bba485eaee09ee528cd743a4770c34539908139596d98e72e993d |
| zircon_runtime/src/dynamic_api/session/events/gamepad.rs | 7078 | ad6ff268bdeb00daefb1229f2cb786a28ac4694dce62d4c615a703871e135b06 |
| zircon_runtime/src/dynamic_api/session/ime_composition_generation.rs | 3983 | e466d241e671b1be83e97766f81e0a720c0cc4fe043e4a0499870854f986eaa9 |
| zircon_runtime/src/dynamic_api/session/ime_composition_route.rs | 9083 | 9a4fb1afb9d4b659718a6cceee13f35510563bc8265ceab7cd97772ac76b9020 |
| zircon_runtime/src/dynamic_api/session/runtime_ui/input_routing.rs | 25329 | f35199765a3b850f10ed1963cddebfea62050da6588a6e278e0bbb59320725ec |
| zircon_runtime/src/dynamic_api/session/state.rs | 37244 | bdd16b6525148be58bf9f86542b12c2dbb0c5f53cd7b343ca666e6c864a9461b |
| zircon_runtime/src/ui/dispatch/input_manager/manager.rs | 34250 | 2c6aeb1585a9fc2ef6978d16cc55af8a4cf47a51bdcda78d1fb15ee4d01a2993 |
| zircon_runtime/src/ui/dispatch/input_manager/timers.rs | 23258 | 60a4e2b1ba8f9f93fe369c7fde087cfb8667643db0d2d0355dbc9d0954fc97c6 |
| zircon_runtime/src/dynamic_api/session/tests/events.rs | 7866 | 301c297ba49990a8e7febf25bc86f1b6fa69dda2e101421140443f91af490f3b |
| zircon_runtime/src/dynamic_api/session/tests/physical_input_ownership.rs | 15165 | a1c81fcc0a269f7b8c2dcdc7ded6a32570d58cf106c10df58a2fa4b4c5c4fc93 |
| zircon_runtime/src/dynamic_api/session/tests/ime_composition_route.rs | 13414 | 676ef3bf3665da1bf3eff1f85d9b0e589c5e2bc7bb1a89cacf28dcace9ecc421 |
| zircon_runtime/src/dynamic_api/session/runtime_ui/tests/cases.rs | 42251 | 647315fc31d52b777ab704eb6d18f4d82299756d8e63432d8d0c5bf519c5e6a1 |

原生编译、16 个 Runtime220 新增用例及修复的已有用例、所有旧 floor/subset/config、GUI、公共 whole-union guard、RSS/P95 与 22 个性能组均待 Jenkins 单批验证。没有因 SOURCE_ONLY PASS 关闭这些条件。构建产物、临时 helper 和源码副本均未创建；实际释放空间仍为 0 B。

