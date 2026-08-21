---
related_code:
  - examples/woc/native/Cargo.toml
  - examples/woc/native/apps/woc_client/Cargo.toml
  - examples/woc/native/apps/woc_client/src/main.rs
  - examples/woc/native/apps/woc_client/src/lib.rs
  - examples/woc/native/apps/woc_client/src/application.rs
  - examples/woc/native/apps/woc_client/src/input
  - examples/woc/native/apps/woc_client/src/preferences
  - examples/woc/native/apps/woc_client/src/presentation
  - examples/woc/native/apps/woc_client/src/shell
  - examples/woc/native/apps/woc_client/src/windows
  - examples/woc/native/apps/woc_client/tests
  - examples/woc/native/plugins/woc_runtime/src/presentation.rs
  - examples/woc/README.md
tests:
  - examples/woc/native/apps/woc_client/tests/application.rs
  - examples/woc/native/apps/woc_client/tests/input.rs
  - examples/woc/native/apps/woc_client/tests/preference_storage_support.rs
  - examples/woc/native/apps/woc_client/tests/preferences.rs
  - examples/woc/native/apps/woc_client/tests/presentation.rs
  - examples/woc/native/apps/woc_client/tests/shell.rs
  - examples/woc/native/apps/woc_client/tests/windows.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_app/01-product-host-bootstrap-loop-dynamic-runtime-shutdown-review.md
  - docs/plans/optimize/zircon_app/03-woc-product-role-host-zrvm-transaction-state-client-server-integration-review.md
  - docs/plans/optimize/zircon_runtime/11a-runtime-ui-architecture-tree-layout-input-accessibility-review.md
  - docs/plans/optimize/zircon_runtime/11b-runtime-text-font-shaping-layout-editing-ime-review.md
  - docs/plans/optimize/zircon_runtime/11c-gpu-ui-renderer-atlas-sdf-batch-clip-submit-review.md
  - docs/plans/optimize/zircon_runtime/19-woc-command-protocol-payload-codec-admission-movement-outcome-runtime-review.md
  - docs/plans/optimize/zircon_tooling/10-test-architecture-partition-selection-isolation-fixture-flake-results-review.md
  - docs/plans/optimize/zircon_tooling/11-woc-parity-oracle-trace-golden-differential-replay-evidence-review.md
  - docs/plans/woc/01-woc-zrvm-one-to-one-replication.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Launch/Private/LaunchEngineLoop.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/GameEngine.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/GameUserSettings.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Application/SlateApplication.cpp
  - dev/bevy/crates/bevy_winit/src/state.rs
  - dev/bevy/crates/bevy_winit/src/lib.rs
  - dev/bevy/crates/bevy_window/src/event.rs
  - dev/bevy/crates/bevy_input/src/gamepad.rs
  - dev/bevy/crates/bevy_render/src/pipelined_rendering.rs
  - dev/fyrox/fyrox-impl/src/engine/executor.rs
  - dev/fyrox/fyrox-impl/src/engine/mod.rs
  - dev/fyrox/fyrox-impl/src/renderer/mod.rs
  - dev/godot/main/main.cpp
  - dev/godot/scene/main/scene_tree.cpp
  - dev/godot/servers/display/display_server.cpp
  - dev/godot/core/input/input.cpp
  - dev/godot/servers/audio/audio_server.cpp
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Common/DynamicResolutionHandler.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/RenderGraph/RenderGraph.cs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 04 · WOC Native Client Window、Input、Shell/UI、Presentation Frame 与产品集成工程化差距

## 1. 结论

`woc_client` 已经积累了相当多可保留的纯模型代码：61 个 production Rust 文件、11,635 个物理行、361,156 bytes，另有 47 个测试文件、11,398 行和 355 个 `#[test]`。Typed command mapper 覆盖 151 类 gameplay intent；shell、角色选择、背包、任务、设置、键位、手柄与 20/60 Hz presentation 都有局部不变量。这不是“什么都没做”，重构不能把已有协议校验和状态测试推倒重来。

但它当前仍不是 native client。`main.rs` 只有 8 行，只生成并打印 Client identity JSON 后退出；binary target 没有引用自己的 `woc_client` library。`Cargo.toml` 只有 `serde_json`、`woc_protocol`、`woc_runtime` 和关闭默认 feature 的 `zircon_runtime`，没有 `zircon_app`、window/event loop、graphics/swapchain、retained UI/text、audio、transport、async executor 或 platform input adapter。全量 source 扫描也没有 `winit`、`wgpu`、`std::net`、socket、audio backend、renderer、async runtime 或线程化 host 实现。

`WocClientSession` 看似是 composition root，实际只组合 shell、HUD route、command mapper 和 frame driver；`ClientWindowController`、`StoredClientSettings`、`StoredKeybinds`、`StoredGamepadBindings`、welcome view、inventory view、graphics budget 等关键模型都没有进入它，更没有外部产品调用者。`SettingApplication::Audio/Renderer/Fullscreen/RootCssVariable`、`HudHostEffect`、`ShellHostEffect` 与 `OnlineShellEffect` 在 client source/tests 之外的 native production 调用数全部为 0。当前架构把最困难的 UI painting、VM construction、effects、window、renderer、audio 与 network 全部留给一个不存在的“host”。

本轮还确认了一个独立于“尚未接线”的提交一致性 bug。`ClientFrameDriver::advance_frame()` 先让 authority 成功提交，再调用 `timeline.push()`；即使 push 因 generation/tick/receipt 冲突失败，代码仍扣除 accumulator、清空 pending commands、推进 movement sequence 并增加 committed count，最后才返回 `Timeline` error。调用者收到失败，但 authoritative state、input consumption 和 client timeline 已处于不同阶段，违反 `ClientAuthority` 注释声明的事务语义，也没有测试覆盖该分支。

在线路径不能以“以后 host 会处理”作为安全边界。认证、realm、角色操作只是可重复产生的 effect，没有 request id、session generation、cancel token、timeout、TLS endpoint identity 或 stale-response guard；`AuthFlowEffect` 和 `AuthSecondFactor` 还派生 `Debug`，会把 password、reset token、2FA/recovery code 暴露给普通日志格式化。离线路径同样会在 `PrepareOfflineSession` 真正成功前把 shell 切到 Welcome，失败后没有回滚/重试状态。

因此当前正确的能力声明是：WOC 有一组 DOM-free、host-neutral 的客户端状态与协议模型；没有 native executable product、engine render/UI/audio/input/network connection，也没有可执行客户端验收证据。本轮登记 **5 项 P0、88 项 P1 和 16 项 P2**。App03 继续拥有四角色 ProductHost、ZrVM transaction 与 client/server 总闭环；本报告只拥有 native client 入口、设备输入、窗口/HUD、设置、安全在线 shell、presentation/present loop 的细化重构。

## 2. 审查边界与可复核证据

### 2.1 物理范围

| 子域 | Production 文件 / 行 / bytes | Test 文件 / 行 / `#[test]` | 当前产品事实 |
|---|---:|---:|---|
| root composition | 3 / 479 / 7,361 | 1 / 295 / 10 | binary 不引用 library；session 只组合 4 个纯模型 |
| input | 20 / 4,010 / 127,569 | 17 / 4,147 / 155 | 151 intents、61 key actions、固定 gamepad/touch 数学；无 OS device adapter |
| preferences | 10 / 2,007 / 58,457 | 9 / 1,385 / 50 | 43 numeric + 41 bool；29/84 application route 为空；无真实 subsystem consumer |
| presentation | 10 / 412 / 12,534 | 2 / 564 / 11 | accumulator/timeline 模型；无 swapchain、render submit、network snapshot 或 pacing host |
| shell | 13 / 3,765 / 118,360 | 13 / 2,743 / 90 | auth/realm/character/offline 纯状态；无 transport、secure credential 或 async request owner |
| windows | 5 / 1,199 / 36,875 | 5 / 1,090 / 22 | inventory/quest/settings view model；无 retained widget tree、focus、layout、paint |
| package 合计 | 109 个物理文件（含 manifest）/ 737,561 Rust bytes | 355 tests | 纯模型测试量不能替代 executable/product acceptance |

测试中 `assert_cmd`、`cargo_bin`、`std::process::Command`、`winit`、`wgpu`、`std::net`、TCP/UDP、Tokio、`async fn`、property/fuzz、Loom 和 Criterion 命中均为 0。也就是说没有一条测试启动 client binary、创建真实窗口/设备/surface、连接服务器、验证 present、覆盖异步竞态或测量帧预算。

### 2.2 静态与动态检查

| 检查 | 结果 | 结论边界 |
|---|---|---|
| `cargo metadata --manifest-path examples/woc/native/Cargo.toml --no-deps --format-version 1` | PASS；确认独立 lib + bin 与 7 个 integration-test target | 只证明 manifest 可解析，不证明代码可编译或产品可运行 |
| binary reachability | `main.rs` 只调用 `woc_runtime::identity_report_json` | 61 个 client source 文件均不在 executable 调用闭包 |
| product caller 反查 | window/settings/keybind/gamepad/welcome/graphics-budget 关键 owner 在 client tests/source 外均为 0 | 不是“host 在别处”；当前 native workspace 没有 host |
| platform/backend source scan | OS window/GPU/audio/network/async backend 命中为 0 | 当前仅有 DTO、effect 与数学 helper |
| settings application | 84 个注册设置中 29 个 route 为 `&[]`；其余也只返回 symbolic effect | 设置被保存不等于运行 subsystem 已应用 |
| asset literals | class catalog 的 33 个 `assets/m8/...` 路径当前均存在 | 可保留 authored asset 选择；仍没有 loader/renderer/materialization |
| WOC native Cargo gate | 复用既有证据：132.6 秒后 `woc_protocol` 6 个 compile error，0 tests 执行 | 未重复运行未变化失败 lane；355 是 authored tests，不是本轮绿色结果 |

### 2.3 参考引擎责任对照

- Unreal 的 `FEngineLoop`、`UGameEngine`、Slate application 与 `UGameUserSettings` 分别拥有 process phase、game viewport/tick/present、window/input/UI dispatch 和 validate/apply/confirm/save。WOC 不必复制类层次，但不能把这些责任全写成无人消费的 enum effect。
- Bevy 的 Winit runner 把 resume/suspend、window/device event、redraw、update mode、exit 与 app schedule连接；`bevy_window` 有 typed lifecycle/window/input event，pipelined rendering 又明确 main/render world handoff。Rust 架构完全可以建立真实 runner，而不是让 8 行 binary 绕过 library。
- Fyrox `Executor` 在 event loop 中初始化/销毁 graphics context，处理 resize/redraw，稳定 update rate，调用 engine update/render，并把 UI、sound、resource manager 纳入 Engine。它证明“host-neutral model”只应是内层，不是产品终点。
- Godot `Main::iteration`、`SceneTree::physics_process/process`、DisplayServer、Input 与 AudioServer 分别拥有固定/变步、消息 flush、窗口、设备与音频线程/总线。客户端产品需要同类责任闭环和 lifecycle order。
- Unity Graphics 本地镜像只用于 graphics 对照：Dynamic Resolution 由 camera/settings/scaler 和 resolution-change callback 驱动，RenderGraph 有 Begin/End/Execute 与 validity check。WOC 的 `GraphicsRuntimeBudget` 当前既没有 scaler controller，也没有 render graph/present consumer，不能据常量表宣称动态画质。

## 3. 可保留的正确基础

### 3.1 Typed protocol mapping 与 queue bound

151 类 gameplay intent 最终经过 payload encoder、`validate_command_payload` 和 `ClientSend` descriptor 检查；command sequence 只在成功映射后前进，pending queue 也复用了每 tick 4,096 command 上限。这些边界应迁入统一 Input Action/Command Submission service，而不是退回 route string 到任意 JSON。

### 3.2 Pure state model 有较高局部测试密度

355 个测试覆盖键位冲突、存储损坏回退、游戏手柄标签、touch 数学、shell transition、角色操作、inventory 交互和 authority retry。虽然当前 workspace gate 是红色，这些 authored assertions 仍是重构回归资产，应在真实 host 测试之下继续保留。

### 3.3 Presentation 对 authority 与 display time 做了初步分离

20 Hz fixed authority、60 Hz sampling、bounded catch-up、command retry、movement sequence 和 projection validation 是合理骨架。修复必须把 authority commit、timeline publish、input consumption 和 present receipt 收敛成明确 transaction，而不是删掉 fixed/presentation 分层。

### 3.4 Preference 已开始使用引擎 platform service

客户端没有自行写 OS 文件，而是通过 `zircon_runtime` 的 `PreferenceStorage`、key namespace 与 mutation submission。这一 ownership 方向正确；缺口是错误、pending、durability、coalescing 和 apply acknowledgment 被 client adapter 吞掉，真实 host 又不存在。

### 3.5 Class preview 资产当前物理闭合

`class_catalog.rs` 的 33 个 model/skin literal 当前都能在 WOC 项目根解析。后续应将其生成到 content/asset catalog 并做 load/render evidence，不应把路径存在性误写成角色预览已渲染。

## 4. P0：Native Client 产品准入前必须硬阻断

### WOC-CLIENT-P0-001 · Client binary 是 identity reporter，完全绕过 client library

`main.rs` 不引用 `woc_client` crate，也不构造 `WocClientSession`。进程打印一行 JSON 后以成功状态退出；window、runtime、scene、VM、input、render、UI、audio、network 和 shutdown 都没有执行机会。这是 App03 四角色 P0 在 client 侧的精确 reachability 证据。

必须新增 engine-owned `ClientProductHost`/runner，由 binary 只负责解析启动参数、选择 product descriptor 并进入 host lifecycle。资格门必须从 packaged binary 观察到 startup -> ready -> frame/present -> requested shutdown -> terminal receipt；identity report 只能是启动前 admission 的一项，不得作为产品主体。

### WOC-CLIENT-P0-002 · 不存在可实例化的 native client composition

当前 manifest 没有 app/window/graphics/UI/audio/network 依赖；`WocClientSession` 又明确把 painting、VM construction 和 effects 留给 host，同时没有组合 windows、preferences、device input 或 product services。`SettingApplication`、HUD/shell/online effects 全部无 production consumer，因此“host-neutral”实际成为“所有集成责任未实现”。

需要由 `zircon_app` 持有的 client composition contract：Platform Window/Input、Graphics device/surface/render graph、Runtime scene/ZrVM、Runtime UI/Text/GPU UI、Audio、Network、Preference、Jobs/Telemetry 都通过 capability-checked service handle 注入；缺 provider 必须 fail-close。WOC 只能提供 project policy、view data 和 action mapping，不能私造引擎 backend。

### WOC-CLIENT-P0-003 · Frame driver 在 timeline 失败时形成部分提交并丢失输入

authority 成功后，`timeline.push(snapshot)` 的错误被暂存；随后代码无条件减 accumulator、clear commands、commit movement 并递增 tick count，最后才传播 timeline error。若 authority 返回回退 generation/tick、冲突 digest 或 receipt regression，VM 已提交，客户端却报告失败并消费输入，timeline 保留旧快照。

必须把 candidate authority commit 与 presentation publish 放入一个可验证 transaction：authority 先产生未提交 candidate/receipt，timeline validate 后原子 publish，再 ack input/movement；任一步失败都 rollback candidate 并保留相同输入 identity。至少注入四类 timeline fault，逐项证明 VM state、timeline、accumulator、commands 和 movement sequence 全部未变。

### WOC-CLIENT-P0-004 · Online/auth 路径没有安全 transport 与请求身份，secret 可被 Debug 输出

在线 flow 接收任意 `base_url` 并产出 login/register/reset/takeover/delete 等 effect，但没有 endpoint trust、TLS、protocol handshake、request id、session generation、deadline、cancel 或 replay/idempotency contract。认证 effect 派生 `Debug` 且包含 password、reset token、2FA code 与 recovery code；普通诊断即可泄露 secret，String clone/clear 也不构成安全擦除。

Online capability 在 secure transport、credential vault/secret type、redacted diagnostics、request correlation、stale completion rejection、rate/backoff 与 server authorization 全部完成前必须 Unavailable。不得用“不记录 Debug”约定、明文 effect、URL 字符串白名单或 UI disable 代替类型与 transport 边界。

### WOC-CLIENT-P0-005 · 355 个纯模型测试被当成进度证据，但 client 既不能编译也没有 executable acceptance

当前 workspace 在 protocol 编译阶段失败，client tests 未执行；测试源码又完全没有 binary、window、GPU、network、async、race、fuzz 或 benchmark lane。即使未来 355 tests 全绿，也只能证明 pure model，不证明 native client 能启动、渲染、连接、持久化或退出。

建立分层准入：clean-clone compile/all-targets -> pure model -> service adapter -> real Windows/Linux/macOS event loop -> GPU present/screenshot -> audio device/fallback -> loopback/chaos network -> suspend/resume/DPI/input hotplug -> packaged smoke/performance/leak。发布状态必须绑定 source/build/backend/driver/evidence fingerprint。

## 5. P1：按责任域重构

### 5.1 Product host、lifecycle 与 service composition

| ID | 差距与重构要求 |
|---|---|
| WOC-CLIENT-P1-001 | `WocClientSession` 只有 shell/HUD/mapper/frame 四字段。新增显式 ClientComposition，纳入 windows、settings、keybind/gamepad state、scene/session、service handles 与 teardown order。 |
| WOC-CLIENT-P1-002 | `HudHostEffect`、`ShellHostEffect`、`OnlineShellEffect` 只是返回值，无统一 dispatcher/ack。建立 typed effect bus，要求 request receipt、completion/failure/cancel 和 owner generation。 |
| WOC-CLIENT-P1-003 | 没有 Constructing/Ready/Suspended/Recovering/Stopping/Stopped lifecycle。产品状态必须阻止事件在错误阶段进入，并记录阶段转换原因。 |
| WOC-CLIENT-P1-004 | `offline_available` 是构造时 bool，不能反映 VM/plugin/asset capability 变化。改为 capability snapshot + generation，provider 丢失时显式降级/退出。 |
| WOC-CLIENT-P1-005 | 没有 project/package/scene materialization、asset preload 或 first-frame ready 定义。与 App03 的真实 ZrVM adapter、Runtime scene transition 和 asset build receipt连接。 |
| WOC-CLIENT-P1-006 | 没有 window create/resize/focus/minimize/close/DPI/monitor/fullscreen owner。复用 `zircon_app` 平台窗口服务，不在 WOC 中增加 `cfg(target_os)`。 |
| WOC-CLIENT-P1-007 | 没有 graphics device/surface generation、lost/outdated recovery、swapchain format/HDR/present mode。Client session 必须绑定 surface generation，旧 GPU result 不得提交。 |
| WOC-CLIENT-P1-008 | 没有 retained widget tree、layout、paint、text/IME、accessibility。接入 Runtime11a/11b/11c，WOC 只生成 typed view/action data。 |
| WOC-CLIENT-P1-009 | 设置声明 Audio route，但没有 mixer/device/bus/voice/footstep owner。接入 engine audio service并处理 device loss、mute/background policy。 |
| WOC-CLIENT-P1-010 | 在线 route 没有 transport/session client，离线 authority 也没有 production `WocProjectVm`。Client host 必须显式选择 LocalAuthority 或 NetworkAuthority，不得由 UI bool 决定。 |
| WOC-CLIENT-P1-011 | 没有 async job/executor，news、realm probe、auth、asset load、preference durability 都无法非阻塞执行。使用 engine jobs/cancellation；主线程只提交和消费 bounded completion。 |
| WOC-CLIENT-P1-012 | 没有 fatal error owner、crash context、shutdown drain 或非零 exit policy。每个 service 的 stop/quiesce/release 顺序要进入 terminal receipt。 |

### 5.2 Fixed tick、presentation、render 与 frame pacing

| ID | 差距与重构要求 |
|---|---|
| WOC-CLIENT-P1-013 | `elapsed_ns` 完全信任 caller，并用 saturating add；巨大/饱和值可制造永久 backlog。输入时钟需 monotonic generation、max delta、suspend discontinuity 与 telemetry。 |
| WOC-CLIENT-P1-014 | catch-up 只限制单次 tick 数，永不丢弃 backlog，也无 slow-client policy。定义 max simulation debt、degradation/disconnect/recovery 和长期 overload gate。 |
| WOC-CLIENT-P1-015 | pending commands 全部进入第一条 catch-up tick，后续 tick 为空；命令没有 intended tick/timestamp。输入采样与 command admission 必须绑定 fixed boundary 和重放身份。 |
| WOC-CLIENT-P1-016 | timeline 只有 previous/current 两帧，不是 network jitter buffer。在线客户端需要 sequence/ack、reorder/loss、adaptive interpolation delay 和 bounded history。 |
| WOC-CLIENT-P1-017 | 没有 local prediction、server reconciliation、correction smoothing 或 rollback history。不能把本地 TransactionalAuthority 的顺滑度外推到在线模式。 |
| WOC-CLIENT-P1-018 | movement stream 只有单 actor 单调 sequence，没有 network ack/resend/window、ownership transfer 或 controlled actor generation change。与 Runtime19 movement contract统一。 |
| WOC-CLIENT-P1-019 | focus loss、pause、modal/text input、suspend 不会清空 held movement/latched jump。Platform lifecycle 必须产生 deterministic input cancellation frame。 |
| WOC-CLIENT-P1-020 | presentation snapshot 未绑定 scene/world/surface/resource generation。hot reload、scene change 或 device reset 后必须拒绝旧 frame/resource handle。 |
| WOC-CLIENT-P1-021 | `visit_presented_actors` 只回调 transform，没有 spawn/despawn、mesh/material/skeleton/LOD/resource readiness。建立 bounded render extraction schema 与 missing-resource policy。 |
| WOC-CLIENT-P1-022 | HUD 使用当前离散 projection，但没有 UI diff/dirty generation；每帧重建风险由 host 隐式承担。发布 immutable view snapshot + changed sets。 |
| WOC-CLIENT-P1-023 | 没有 render acquire/submit/present receipt、GPU fence、frames-in-flight 或 backpressure。simulation commit 不能被误当作 displayed frame。 |
| WOC-CLIENT-P1-024 | 没有 VSync/VRR/frame limiter/background throttle/occlusion policy。采用 host presentation scheduler，并分别测 simulation、CPU frame、GPU frame 与 present latency。 |
| WOC-CLIENT-P1-025 | `GraphicsRuntimeBudget` 是无人调用的常量表。动态分辨率必须消费 GPU timing、hysteresis、camera/output约束并返回实际 scale generation。 |
| WOC-CLIENT-P1-026 | projection 仍是每 tick bounded JSON decode 和 owned model；App03 的 full-state/JSON P0 是前置依赖。Client 侧改用 versioned bounded binary reader/immutable handles。 |
| WOC-CLIENT-P1-027 | frame errors只有 enum variant，没有 `Display/Error`、fault context、tick/generation/backend identity。接入 structured diagnostics，且 secret 字段必须 redacted。 |

### 5.3 Keyboard、mouse、gamepad、touch 与 action mapping

| ID | 差距与重构要求 |
|---|---|
| WOC-CLIENT-P1-028 | `ClientInputDevice` 在 `map()` 中被直接丢弃。设备 identity、source trust、last-active-device、glyph与anti-cheat provenance 应进入 input event metadata。 |
| WOC-CLIENT-P1-029 | 没有 OS event -> engine input event adapter。接入 platform input queue，处理 focus、repeat、IME、raw mouse、wheel、pointer lock和时间戳。 |
| WOC-CLIENT-P1-030 | 没有 gameplay/UI/text/modal/action context stack。统一 action resolver必须支持优先级、consume、capture和enable predicate。 |
| WOC-CLIENT-P1-031 | 键位使用 DOM `KeyW`/`Digit1` 字符串，native scan code/layout语义未定义。引擎层提供 physical/logical key typed code与跨平台序列化。 |
| WOC-CLIENT-P1-032 | `bind()` 接受空字符串和任意非规范 combo，只拒绝 Escape。建立 parser/schema、modifier规则、platform reserve和未知码诊断。 |
| WOC-CLIENT-P1-033 | 61 action registry、标签与默认键位手写在 app source，无法与内容、localization、console cert policy合并。改为 versioned action-map asset/generated projection。 |
| WOC-CLIENT-P1-034 | key dispatch 每次线性扫描 61x2 String，held movement也扫描 caller slice。构建 normalized reverse index和per-frame transition state，热路径不分配。 |
| WOC-CLIENT-P1-035 | gamepad kind 依赖设备名称 substring/vendor文本，按钮固定为 0..16。使用标准 mapping DB、GUID/vendor/product、platform remap与 unknown capability。 |
| WOC-CLIENT-P1-036 | gamepad binding接受任意 action String，storage同样不校验 action catalog。加载/绑定时拒绝未知、退役或不允许的 action，并提供 migration。 |
| WOC-CLIENT-P1-037 | 没有多手柄、local player assignment、hotplug、battery、motion sensor、trigger calibration。设备生命周期属于 engine input service。 |
| WOC-CLIENT-P1-038 | deadzone/look/touch math公开接受 NaN、Infinity、deadzone>=1、负 elapsed/speed等值，可能产生 NaN/除零。所有公共数学入口必须 finite/range validate。 |
| WOC-CLIENT-P1-039 | `rising_edges` 只报告 press，数组缩短/断连不生成 release。维护 per-device button state并在断连/focus loss合成 release/cancel。 |
| WOC-CLIENT-P1-040 | touch 只有无状态数学函数，没有 pointer id、capture/cancel、safe area、orientation、gesture arbitration或multi-touch state machine。接入 typed gesture recognizer。 |
| WOC-CLIENT-P1-041 | long-press/double-tap依赖 caller传入毫秒值，没有 monotonic clock generation；clock reset可误判。由 input service在同一时间域计算。 |
| WOC-CLIENT-P1-042 | 设置声明 vibration 但没有 haptics command、duration/amplitude bound、device support或stop-on-suspend。增加 capability-checked haptics service。 |
| WOC-CLIENT-P1-043 | `intent.rs` 单文件 1,418 行、151 intent并混合 combat/social/guild/mail/market/party/instance等域。按生成 schema/domain adapter拆分，避免业务合同继续集中膨胀。 |
| WOC-CLIENT-P1-044 | command sequence只在进程内从 caller给定值开始，没有 reconnect/session negotiation、ack checkpoint或wrap replacement。sequence authority必须由 transport/session拥有。 |

### 5.4 Settings、preference durability 与 runtime application

| ID | 差距与重构要求 |
|---|---|
| WOC-CLIENT-P1-045 | `woc_settings` JSON没有 schema version、source build或migration chain。新增 versioned document、typed migration、backup/rollback和unknown-field policy。 |
| WOC-CLIENT-P1-046 | enum、integer、toggle等 43 项全部存为 `f64`，set只 clamp不量化，`graphicsPreset=2.7`可合法驻留。每项使用 typed value和step/domain validator。 |
| WOC-CLIENT-P1-047 | 84 个 setting中29项 application为空，保存后永远不生效。未实现项必须标记 Unavailable，不能展示成可用设置。 |
| WOC-CLIENT-P1-048 | 其余 application也只是 symbolic enum，全仓无 renderer/audio/fullscreen consumer。引入 apply transaction、per-subsystem ack/failure和UI status。 |
| WOC-CLIENT-P1-049 | native crate中保留 RootCssVariable、ElementCssVariable、BodyClass、BrowserEffects等 Web DOM概念。抽成跨host语义设置，再由Web/retained UI分别适配。 |
| WOC-CLIENT-P1-050 | `read_preference_text` 把 backend error、invalid key与invalid UTF-8都压成 absent。保留 unavailable/denied/corrupt/transient分类并进入诊断/UI。 |
| WOC-CLIENT-P1-051 | `submit_preference_text` 返回 `Option` 并吞掉 submit error；bind/set仍报告成功。mutation API必须区分 memory changed、queued、durable和failed。 |
| WOC-CLIENT-P1-052 | pending read在构造时直接变成 defaults，只有 host主动 `refresh_from_storage()` 才能纠正；当前没有 host。建立 async load state和ready notification。 |
| WOC-CLIENT-P1-053 | 每个 stored owner只保留最后一个 submission，连续写会丢失前序 receipt。使用coalesced generation、flush barrier和latest-durable watermark。 |
| WOC-CLIENT-P1-054 | 每次 slider/keybind mutation立即编码完整 JSON并提交，没有 debounce/coalesce/backpressure。UI edit session与 durable commit分离。 |
| WOC-CLIENT-P1-055 | 所有写入使用 `PreferenceWorkDeadline::none()`，退出时也没有 flush。定义interactive/background/shutdown deadline与失败策略。 |
| WOC-CLIENT-P1-056 | settings只有全局 key，keybind scope是自由字符串；没有 account/device/character/profile identity schema。建立限定 scope与隔离/迁移规则。 |
| WOC-CLIENT-P1-057 | reset/application跨多个 subsystem但没有原子 preflight/rollback。先 validate全部目标，再按依赖应用，失败恢复 previous applied snapshot。 |
| WOC-CLIENT-P1-058 | GPU默认分档依赖 renderer name substring、memory/core/touch启发式。改为 adapter capability、benchmark/telemetry、thermal/battery和用户确认。 |
| WOC-CLIENT-P1-059 | `application_plan()` 每次构造全量 Vec，没有 changed set、order dependency或generation。发布稳定 diff plan并记录每个consumer ack。 |
| WOC-CLIENT-P1-060 | fullscreen/resolution没有 preview/confirm/revert watchdog；Unreal的 ConfirmVideoMode 对照说明此项是产品安全合同。窗口模式变更必须可自动恢复。 |

### 5.5 Shell、auth、realm、character 与 offline launch

| ID | 差距与重构要求 |
|---|---|
| WOC-CLIENT-P1-061 | offline picker提交时先切到 Welcome，再要求 host准备 authority；prepare失败没有状态回滚。增加 Preparing/Failed/Retry/Cancel和correlated completion。 |
| WOC-CLIENT-P1-062 | Continue先切Loading，`finish_loading()`又只靠host调用，没有load token、progress、failure/cancel或scene ready receipt。接入原子 scene/session transition。 |
| WOC-CLIENT-P1-063 | online/auth/realm/character effect都没有 operation id或session generation。所有异步completion必须匹配当前请求和screen generation。 |
| WOC-CLIENT-P1-064 | login/register/reset可重复submit且没有 Busy/Disabled/Idempotency状态。模型需要 in-flight、retry/backoff、timeout与duplicate suppression。 |
| WOC-CLIENT-P1-065 | username/forgot username先检查原字符串非空再 trim，纯空白可产生空请求。共享 server/client canonical validator，UI只负责即时提示。 |
| WOC-CLIENT-P1-066 | reset token只检查 `is_empty()`，无长度/字符/expiry/origin bound。token必须由secure deep-link/session owner验证并redact。 |
| WOC-CLIENT-P1-067 | secret存于普通 String、被clone到effect，`clear()`不保证擦除。使用non-Debug secret wrapper/secure store，缩短明文生命周期。 |
| WOC-CLIENT-P1-068 | stale `AuthCompletion` 只按当前screen处理，没有请求类型/nonce；旧响应可改变新登录流程。completion携带operation/session identity。 |
| WOC-CLIENT-P1-069 | realm `base_url` 是任意 String，无scheme/host/port/trust配置验证。只接受signed realm descriptor和transport解析后的endpoint identity。 |
| WOC-CLIENT-P1-070 | realm status更新没有refresh generation；旧probe可覆盖新directory同名realm。status batch绑定directory generation并原子完成推荐计算。 |
| WOC-CLIENT-P1-071 | `select()` 不检查 online/full/checking，remembered realm可在probe前直接进入角色页。连接准入由resolved status + transport handshake决定。 |
| WOC-CLIENT-P1-072 | realm/roster/release Vec与String没有网络输入上限；局部字段校验不限制总bytes/rows。decoder阶段施加schema预算。 |
| WOC-CLIENT-P1-073 | roster Name sort在每次比较中重复 `to_ascii_lowercase()` 分配。预计算locale-aware sort key，并限制 roster规模。 |
| WOC-CLIENT-P1-074 | create/rename/delete/takeover没有完整request状态；delete/takeover在host确认前清除本地pending。保留intent直到server receipt并支持retry/cancel。 |
| WOC-CLIENT-P1-075 | 角色名规则由client ASCII helper私有拥有，server/content authority可能漂移。规则进入生成合同，client/server共享version/fingerprint。 |
| WOC-CLIENT-P1-076 | class role、能力、颜色、model/skin path与skin count手写在app source，和content/runtime catalog双真相。由qualified content catalog生成view projection。 |
| WOC-CLIENT-P1-077 | WelcomeSessionStorage没有Result，set/remove失败不可见；news/Discord/chest也只有caller注入值，无load generation。复用preference/session service与async state。 |
| WOC-CLIENT-P1-078 | `native_app` 会无条件选择 Touch continue hint，即使是桌面native。提示应由last-active input device与accessibility policy决定。 |

### 5.6 Windows、HUD、inventory 与 quest projection

| ID | 差距与重构要求 |
|---|---|
| WOC-CLIENT-P1-079 | `ClientWindowController` 不在 `WocClientSession`，HUD的OpenQuest/OpenOptions也不调用它。建立唯一 WindowWorkspace owner和typed open/close action。 |
| WOC-CLIENT-P1-080 | shell/HUD/window都解析手写route String，编译器不能证明view action与handler一致。route/action ID从UI schema生成并在build时闭合。 |
| WOC-CLIENT-P1-081 | window只有三个bool/Option，没有focus、z-order、modal、input capture、saved geometry、DPI/safe area或accessibility focus。责任归Runtime UI workspace。 |
| WOC-CLIENT-P1-082 | inventory search/text_value无长度限制并直接持久化。输入、document与preference层都要有UTF-8/UTF-16/bytes预算。 |
| WOC-CLIENT-P1-083 | `build_inventory_window_view` 每次重建HashMap、clone全部item，再clone visible/cells并逐项lowercase。使用catalog handle、stable item key、incremental/virtualized view。 |
| WOC-CLIENT-P1-084 | catalog缺失的item没有fallback presentation，会从visible过滤掉却仍占capacity。显示unknown placeholder并记录content mismatch，不能静默消失。 |
| WOC-CLIENT-P1-085 | stale click只比较当前index与值；无instance的同值stack可ABA通过。projection提供stable item generation/precondition，server仍做最终authority验证。 |
| WOC-CLIENT-P1-086 | deposit submit只比 `item_id`，没有index/instance/count version，且max_count由host传入。命令携带stable inventory precondition并由authority裁决。 |
| WOC-CLIENT-P1-087 | quest view每次clone所有ID/objective，只有选择ID，无tracker/window diff、localization/content handle或大列表virtualization。输出immutable bounded view snapshot。 |
| WOC-CLIENT-P1-088 | inventory可售/可弃/可用、bug-report availability等由client projection决定。UI可以提示，但server必须重复authorize，拒绝结果需回流并刷新projection。 |

## 6. P2：质量、可维护性与诊断补强

| ID | 差距与建议 |
|---|---|
| WOC-CLIENT-P2-001 | `main.rs` 对identity error使用 `expect`，没有typed fatal diagnostic/exit code。即使保留identity CLI也应稳定输出failure receipt。 |
| WOC-CLIENT-P2-002 | `lib.rs` glob re-export全部模块，当前约331个public declaration没有稳定API层次。按host/project/view/test-support分层导出。 |
| WOC-CLIENT-P2-003 | 多数public error只派生Debug/PartialEq，不实现Display/Error/source。统一错误分类和diagnostic code。 |
| WOC-CLIENT-P2-004 | `ClientSettings::all()` 只是语义不清的全量clone。改为snapshot命名或删除，避免caller误认增量视图。 |
| WOC-CLIENT-P2-005 | key/gamepad/action/class fallback label硬编码英文，与translation key混用。统一localization key +可审计fallback catalog。 |
| WOC-CLIENT-P2-006 | class color是未注明color space的raw `u32`。使用typed linear/sRGB color并由content importer校验。 |
| WOC-CLIENT-P2-007 | `intent.rs`、settings options、routes等大文件缺少generated/source ownership注释与schema fingerprint。拆分时补producer和审计信息。 |
| WOC-CLIENT-P2-008 | route parser每次按prefix/string parse，错误只回显原字符串。生成perfect/static lookup并记录view/action owner。 |
| WOC-CLIENT-P2-009 | settings使用BTreeMap<&'static str,...>承载固定schema，增加查找和状态不完整可能。生成typed struct/enum index。 |
| WOC-CLIENT-P2-010 | graphics budget常量没有单位型别，秒/毫秒/比例均是f64。使用Duration、ScaleFactor与validated thresholds。 |
| WOC-CLIENT-P2-011 | welcome `published_at` 是任意String，无法排序/时区显示/验证。decoder使用typed timestamp，UI按locale格式化。 |
| WOC-CLIENT-P2-012 | character `last_played_epoch_ms` 允许负值，level/playtime/string长度缺完整范围。补schema validation与negative fixtures。 |
| WOC-CLIENT-P2-013 | inventory filter JSON、keybind JSON、gamepad JSON各自手写serde Value遍历。生成typed documents并共享corruption/migration harness。 |
| WOC-CLIENT-P2-014 | 纯测试fixture重复构造较大projection/authority，缺统一builder和golden摘要。提取test-support crate但不得进入production dependency。 |
| WOC-CLIENT-P2-015 | 没有rustdoc example说明host effect completion/ownership；当前注释容易让caller误以为返回effect即完成。为public orchestration contract补状态图和示例。 |
| WOC-CLIENT-P2-016 | 没有feature/capability文档列出desktop/mobile/web/headless差异。由product descriptor生成machine-readable capability matrix。 |

## 7. 逐文件扫描矩阵

下表覆盖本轮 61 个 production Rust 文件；“保留”只表示局部模型有价值，不代表产品已接线。

| 文件 | 逐文件结论 |
|---|---|
| `src/main.rs` | P0：identity打印后退出，binary不引用library。 |
| `src/lib.rs` | P2：无分层glob export，扩大未稳定public surface。 |
| `src/application.rs` | P0/P1：composition缺windows/settings/device/backend；host effect无ack。 |
| `src/input/mod.rs` | 仅re-export，无统一InputService owner。 |
| `src/input/hud_routes.rs` | route string + 本地UI bool；host effect无consumer/authority receipt。 |
| `src/input/intent.rs` | 1,418行/151 intent单体mapper；typed codec可保留，需生成与域拆分。 |
| `src/input/touch.rs` | 无状态数学，无pointer lifecycle；finite/range未校验。 |
| `src/input/gamepad/mod.rs` | 仅re-export，无device lifecycle。 |
| `src/input/gamepad/bindings.rs` | arbitrary action String，缺catalog/migration/multi-device。 |
| `src/input/gamepad/layout.rs` | 固定17按钮与名称启发式，缺标准mapping DB/GUID。 |
| `src/input/gamepad/math.rs` | deadzone/look无finite/range防护，release/disconnect语义缺失。 |
| `src/input/gamepad/options.rs` | 纯row model，无真实device apply/haptics。 |
| `src/input/gamepad/storage.rs` | pending/error/durability被压平，unknown action可载入。 |
| `src/input/keybind/mod.rs` | 仅re-export，无action-map asset/schema owner。 |
| `src/input/keybind/bindings.rs` | conflict骨架可保留；combo parser与reverse index缺失。 |
| `src/input/keybind/combo.rs` | DOM code/string组合，native physical/logical key未建模。 |
| `src/input/keybind/options.rs` | 纯view row与bind转发，无capture session/OS reserved policy。 |
| `src/input/keybind/profile.rs` | 两条magic repair signature是局部migration，不是versioned document migration。 |
| `src/input/keybind/registry.rs` | 61 action/default/英文label手写双真相。 |
| `src/input/keybind/storage.rs` | save立即整文档提交，错误吞掉且只留最后receipt。 |
| `src/input/movement/mod.rs` | 仅re-export。 |
| `src/input/movement/keyboard.rs` | caller-held字符串扫描，缺focus/release/context。 |
| `src/input/movement/resolve.rs` | source merge顺序可保留；缺device generation与cancel frame。 |
| `src/preferences/mod.rs` | storage helper被crate-private导出，未形成产品settings service。 |
| `src/preferences/storage.rs` | engine preference方向正确；错误/UTF-8/pending/durability语义丢失。 |
| `src/preferences/settings/mod.rs` | 仅re-export，无schema/version owner。 |
| `src/preferences/settings/application.rs` | 84 symbolic route、29空route、Web DOM语义混入native。 |
| `src/preferences/settings/graphics_budget.rs` | 预算表无人消费，无GPU timing/governor。 |
| `src/preferences/settings/graphics_default.rs` | renderer字符串启发式，无adapter capability/benchmark。 |
| `src/preferences/settings/options.rs` | 纯control Vec，频繁分配；未实现项仍可展示。 |
| `src/preferences/settings/registry.rs` | 43 numeric/41 bool手写，enum/toggle退化为f64。 |
| `src/preferences/settings/state.rs` | clamp基础可保留；无step/type/generation。 |
| `src/preferences/settings/storage.rs` | schema/migration/async ready/apply transaction缺失。 |
| `src/presentation/mod.rs` | 仅re-export，无render/present owner。 |
| `src/presentation/authority.rs` | Local transactional adapter骨架可保留；真实VM/network authority缺失。 |
| `src/presentation/error.rs` | fault上下文不足，未实现standard error/diagnostic。 |
| `src/presentation/frame.rs` | 只暴露tick/backlog/HUD alpha，无present/GPU receipt。 |
| `src/presentation/frame_driver.rs` | P0部分提交；时钟/backlog/pacing/network history不完整。 |
| `src/presentation/limits.rs` | 复用protocol command count正确；未形成bytes/memory/backpressure预算。 |
| `src/presentation/movement/mod.rs` | 仅re-export。 |
| `src/presentation/movement/error.rs` | 只包装validation/sequence exhaustion，无transport recovery。 |
| `src/presentation/movement/stream.rs` | retry identity可保留；无ack/reconnect/ownership transfer。 |
| `src/presentation/movement/tick_input.rs` | borrowed batch避免额外owner；仍只服务local authority call。 |
| `src/shell/mod.rs` | 仅re-export；shell没有async operation registry。 |
| `src/shell/auth_flow.rs` | P0 secret Debug；请求相关性、secure storage、validator不完整。 |
| `src/shell/character_roster.rs` | 基础验证/排序可保留；输入预算、locale key和generation缺失。 |
| `src/shell/class_catalog.rs` | 33资产存在；content双真相且没有loader/render。 |
| `src/shell/mode_selection.rs` | static offline bool，不消费live capability。 |
| `src/shell/offline_flow.rs` | host成功前推进状态；无failure/retry/cancel。 |
| `src/shell/offline_session.rs` | typed bootstrap方向正确；仍依赖App03真实VM/scene闭环。 |
| `src/shell/online_character_flow.rs` | operation状态/receipt/stale guard缺失，pending过早清除。 |
| `src/shell/online_shell.rs` | 纯screen aggregator；没有transport session generation。 |
| `src/shell/realm_directory.rs` | base URL/trust/status generation/capacity校验缺失。 |
| `src/shell/routes.rs` | 609行string route dispatcher；view/action compile closure缺失。 |
| `src/shell/welcome_screen.rs` | 纯view函数可保留；storage error、async generation、device hint有缺口。 |
| `src/shell/woc_shell.rs` | 顶层flow骨架可保留；effect completion与rollback未建模。 |
| `src/windows/mod.rs` | 仅re-export；没有Runtime UI workspace连接。 |
| `src/windows/inventory.rs` | 每次全量HashMap/clone/lowercase；unknown item静默不可见。 |
| `src/windows/inventory_actions.rs` | UI提示逻辑可保留；stable precondition/server authorization不足。 |
| `src/windows/quest_log.rs` | 全量clone view，无diff/virtualization/localization handle。 |
| `src/windows/routes.rs` | controller不在session；focus/layout/modal与typed action缺失。 |

## 8. 目标架构与 owner 边界

| Owner | 唯一责任 | 本报告禁止其承担的责任 |
|---|---|---|
| `zircon_app::ClientProductHost` | process/window/event loop、service composition、lifecycle、shutdown、terminal receipt | WOC业务规则、command payload真相 |
| Platform Window/Input services | OS event、device identity、focus/DPI/suspend、clipboard/IME、gamepad/touch/haptics | WOC route String、角色/战斗语义 |
| Graphics/Render host | device/surface、render extraction、frame graph、GPU timing、present、dynamic resolution | authoritative gameplay tick |
| Runtime UI/Text/GPU UI | retained tree、layout、focus、accessibility、text/IME、paint/batch | auth/network请求与world authority |
| Audio service | device/mixer/bus/voice/haptics-adjacent capability、background policy | settings document存储 |
| Network/Online client | endpoint trust、handshake、request/session identity、timeout/retry/cancel、secret redaction | UI screen状态 |
| `woc_runtime` authority adapter | Local/Network authority candidate、projection decode、commit/rollback receipt | OS window与renderer提交 |
| `woc_client` project layer | WOC view policy、typed actions、shell/window state、content projection adapter | 平台backend、GPU/audio/network实现 |
| Runtime19 protocol owner | command/movement wire、sequence/ack/budget/precondition | UI gesture和窗口焦点 |
| App03 ProductHost总报告 | 四角色、真实ZrVM、client/server总闭环 | 本报告已细化的client UI/input/settings实现项 |

需要新增且由单一owner生成/版本化的最小schema：`ClientProductDescriptor`、`ClientLifecycleReceipt`、`ClientFrameTransaction`、`InputActionMap`、`InputDeviceEvent`、`OnlineOperationEnvelope`、`ClientSettingsDocument`、`SettingsApplyReceipt`、`WindowWorkspaceSnapshot`、`RenderExtractionSnapshot` 与 `PresentReceipt`。所有schema必须带generation/version/budget；跨线程/跨进程handle必须遵循全局ABI/ownership报告，不能传裸trait object或无版本String action。

## 9. 分层实施里程碑

### M0 · 收缩能力声明并建立红色产品门

保持binary当前行为但把它标记为 `IdentityOnly/Unavailable`，禁止playable/release声明；加入binary reachability、backend dependency、effect consumer和timeline-fault红色测试。先修当前protocol compile blocker，确保后续门能执行。

### M1 · Engine-owned ClientProductHost 与真实生命周期

实现window/event loop、service composition、startup-ready-shutdown receipt；接入App03真实ZrVM/project/scene materialization。Windows首条lane必须能创建窗口、处理resize/focus/close并稳定退出，随后扩展Linux/macOS/mobile。

### M2 · 原子 authority/presentation/render transaction

修复candidate commit，建立fixed input boundary、timeline/jitter buffer、render extraction、GPU submit/present receipt、surface generation和fault rollback。Local和loopback network authority共享同一外层frame contract。

### M3 · Engine input action system与设备适配

完成typed physical/logical keys、context/capture、mouse/raw input、gamepad mapping/hotplug、多设备、touch gestures和haptics；迁移61 action和151 intent到generated action/protocol projection。

### M4 · Secure online shell与异步operation registry

建立signed realm descriptor、TLS/session handshake、secret wrapper/redacted logging、operation identity、timeout/retry/cancel、stale response guard；再接auth/realm/character/news。没有secure provider时在线入口保持Unavailable。

### M5 · Retained UI/window/settings闭环

把shell/HUD/inventory/quest/settings接入Runtime UI/Text/GPU UI；建立typed action、focus/accessibility、virtualized list、versioned preferences与transactional settings apply/confirm/revert。

### M6 · 平台、性能与发布验收

覆盖Windows/Linux/macOS、DPI/多显示器、suspend/resume、device/surface/audio loss、network chaos、fresh-process persistence、GPU截图、frame pacing、内存/分配/leak和packaged artifact。再以Tooling11真实parity actual runner证明WOC行为，而不是只检查纯模型。

## 10. 验收门

1. clean clone通过WOC generator check、`cargo check --workspace --all-targets`与`cargo test --workspace`，receipt绑定source/generator/toolchain。
2. packaged `woc_client` 不再打印identity即退出；能进入Ready并持续present，关闭窗口后有typed terminal receipt和正确exit code。
3. executable reachability证明ClientProductHost、window、input、graphics、UI、audio、preference、Local/Network authority均由resolved descriptor实例化。
4. 缺任一required provider时fail-close且UI capability不可见；不得fallback到dummy后仍报告Ready。
5. timeline generation/tick/digest/receipt四类fault注入后，VM、timeline、accumulator、commands、movement sequence逐字节/逐值不变。
6. 5秒stall、suspend 30秒、clock discontinuity和长期overload不会形成永久backlog；policy与telemetry可复核。
7. resize/minimize/restore/DPI/monitor/fullscreen/HDR/surface loss循环无旧generation提交、黑帧或资源泄漏。
8. Windows/Linux/macOS真实keyboard/mouse/gamepad/touch-capable lane验证focus loss release、hotplug、mapping、multi-device与haptics capability。
9. auth日志、panic、trace、crash context与Debug formatting中password/token/2FA/recovery code命中为0。
10. stale auth/realm/character/news completion、重复submit、timeout、cancel和reconnect都不能改变新session state。
11. endpoint只来自validated signed realm descriptor；TLS/handshake/protocol/schema identity不匹配全部拒绝。
12. settings migration、corrupt read、denied/quota/transient write、pending startup、rapid slider coalesce与shutdown flush都有fresh-process证据。
13. fullscreen/resolution错误或未确认能在watchdog期限内恢复previous mode。
14. inventory 10,000项/quest大列表通过virtualization与allocation预算；unknown content显示placeholder并产生诊断。
15. GPU frame evidence包含非空正确场景/HUD、present receipt、render graph pass、driver/device/surface identity和可复核截图。
16. CPU update、render extraction、GPU frame、present latency、allocations/frame和memory high-water均有P50/P95/P99门，且与同画质参考场景成对比较。
17. audio device loss/background/volume apply与voice unavailable路径不会阻塞主线程或静默丢设置。
18. normal close、fatal VM、GPU loss、network disconnect和OS signal均按依赖逆序drain；线程、窗口、surface、audio、socket、preference work无泄漏。
19. pure model tests继续全绿，并新增binary/platform/GPU/network/async/race/property/fuzz/benchmark分层；任一上层门不得用lower-layer test数替代。
20. Tooling11的54场景actual runner至少有client-visible场景经真实ProductHost产生trace；golden自比较不能计入本里程碑。

## 11. 禁止的临时修复

- 不得只在 `main.rs` 加一个无限sleep/假event loop来让进程“持续运行”。
- 不得在WOC中直接依赖Winit/WGPU/CPAL/HTTP库形成绕过Zircon service的第二套引擎host。
- 不得让host用巨大match消费effect却不返回operation receipt、failure、cancel和generation。
- 不得在timeline错误时“忽略错误继续画旧帧”，也不得只移动clear语句而不解决authority rollback。
- 不得用日志约定保护secret；类型必须默认不可Debug、不可普通clone，并由redaction gate验证。
- 不得把online endpoint硬编码为一个URL、关闭证书校验或用mock server通过产品验收。
- 不得以29个空application route的设置“暂不生效”为理由继续展示可编辑控件。
- 不得为每个平台在WOC内新增`cfg(target_os)`输入、窗口、目录或音频特例。
- 不得用固定截图、offscreen clear color、CPU生成图或identity JSON冒充真实native present evidence。
- 不得以355个测试数量、Cargo metadata PASS或33个资产路径存在替代compile/runtime/render/network验收。

## 12. 本轮边界

本轮只新增审查文档与索引，不修改 `woc_client`、`woc_runtime`、`woc_protocol`、Zr source、manifest、tests、asset或generated artifact。既有WOC workspace compile失败与npm aggregate短路没有因source未变化而重复执行；本报告的动态结论明确复用App03/Tooling11已记录证据。实现开始前必须重新读取最新source并重新跑M0门，不能把本报告行号或数量当成永久真相。
