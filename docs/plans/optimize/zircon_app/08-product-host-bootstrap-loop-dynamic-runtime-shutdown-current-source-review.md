---
title: Zircon App Product Host / Bootstrap / Loop / Dynamic Runtime / Shutdown 当前源码复审
category: zircon_app
report_id: App08
review_date: 2026-08-24
baseline_head: f811b3bf474d70347199772a175422333dfb36f6
baseline_epoch: 420
verification_head: 79f64878f3b9526517644c055ad3bf5cadfccd0f
verification_epoch: 421
supersedes_currentness_of:
  - docs/plans/optimize/zircon_app/01-product-host-bootstrap-loop-dynamic-runtime-shutdown-review.md
related_code:
  - zircon_app/Cargo.toml
  - zircon_app/build.rs
  - zircon_app/src/bin/editor.rs
  - zircon_app/src/bin/runtime_preview.rs
  - zircon_app/src/entry
  - zircon_app/src/plugins
  - zircon_app/src/runtime_presenter.rs
related_consumers:
  - zircon_editor/src/core/play
  - zircon_editor/src/core/gateway/session
  - zircon_runtime_host/src/foreign_output
  - zircon_runtime_interface/src/runtime_api/abi/api_table.rs
  - zircon_runtime/src/dynamic_api/session/ffi.rs
plan_sources:
  - docs/plans/optimize/zircon_app/01-product-host-bootstrap-loop-dynamic-runtime-shutdown-review.md
  - docs/plans/optimize/zircon_runtime/01-core-runtime-lifecycle-registry-review.md
  - docs/plans/optimize/zircon_runtime/99zk-runtime-builtin-module-catalog-profile-target-feature-selection-extension-registration-capability-load-report-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime_interface/01-runtime-dll-abi-ffi-version-handle-foreign-ownership-review.md
  - docs/plans/optimize/zircon_runtime_interface/05-runtime-host-foreign-output-safe-api-ownership-admission-budget-fuse-observability-review.md
  - docs/plans/optimize/zircon_editor/07-play-session-process-pie-game-view-live-edit-recovery-review.md
  - docs/plans/optimize/zircon_plugins/01-plugin-sdk-package-catalog-distribution-native-abi-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Launch/Private/Launch.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Launch/Private/LaunchEngineLoop.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Launch/Private/Windows/LaunchWindows.cpp
  - dev/bevy/crates/bevy_app/src/app.rs
  - dev/bevy/crates/bevy_app/src/plugin.rs
  - dev/bevy/crates/bevy_app/src/schedule_runner.rs
  - dev/bevy/crates/bevy_winit/src/state.rs
  - dev/Fyrox/fyrox-impl/src/engine/executor.rs
  - dev/Fyrox/fyrox-impl/src/plugin/mod.rs
  - dev/Fyrox/fyrox-impl/src/plugin/dylib.rs
  - dev/godot/main/main.cpp
  - dev/godot/main/main.h
  - dev/Graphics/Packages/com.unity.render-pipelines.core/package.json
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/package.json
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/RenderPipeline/HDRenderPipeline.cs
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# App08 · Product Host、Bootstrap、Loop、Dynamic Runtime 与 Shutdown 当前源码复审

## 1. 结论

App01 的核心判断仍成立：`zircon_app` 当前是“Editor 宿主 + 桌面单窗口 runtime preview + 一组 composition helper”，还不是可以承载 Editor、Desktop Client、Server、Web、Android、Play Child 与 Commandlet 的统一工程级产品宿主。`target-server` 仍没有 binary，`run_headless()` 仍在激活 Core 后立即返回；client 的 Minimal/Server profile 仍借用 Winit，且 `target-client` 强制引入 desktop window/input/X11/Wayland bundle。Cargo feature、Runtime profile、module graph、DLL session 与实际 artifact 没有收敛成一份可验证的 Product BuildSet。

当前源码确有进展，不能沿用旧报告的全部负面结论。foreign output 已由 `zircon_runtime_host::foreign_output` 对 host request、profile、operation、plugin event、world query 与 world invalidation 建立统一 bytes/items/decode-time/nesting policy；失焦 cadence 已从约 60 Hz 降为 10 Hz；runtime 顶层能同时保留 event-loop、app callback 与 session teardown 三类错误；两个 binary 的 log shutdown 失败现在会覆盖成功退出码；`PluginGroupBuilder::finish()` 已限制为测试使用。这些分别使旧 P1-12、P1-19、P1-22、P1-24 与 P2-3 发生 Closed 或 Partial 变化。

但产品生命周期的主断口没有关闭。`RuntimeSession::drop` 仍从运行态直接 unbind viewport 1 并 destroy；destroy 失败只能 `abort`，没有 quiesce、worker/callback drain、deadline、module deactivation 与 evidence flush 的统一顺序。Winit host 丢弃 `WindowId`/`DeviceId`，没有 suspended/exiting，`Destroyed` 不清理 host owner，resize 直接重 bind。更严重的是 native surface target 目前只生成 Win32 handle：即使 Cargo 宣称 X11/Wayland/Android/Web，非 Win32 路径仍退回完整 CPU readback/pixel conversion。

Play 边界也仍未形成协议。`--play-report-pipe` 只是 stdout 行文本中的 outlet 标签；Editor 的 bounded output pump只把它当 diagnostic line，不解析或驱动状态。Editor 在 backend spawn 返回后立即进入 Playing，而 runtime 在 session create 后、world/scene 与首帧 ready 前就发送 Ready。当前 report 写失败还会通过 `?` 覆盖原本的 startup/terminal 错误，降低故障归因质量。

本轮按 App01 原编号复判：**4 项 P0 全 Open**；原 27 项 P1 为 **23 Open、3 Partial、1 Closed**，新增 P1-28..30 后合计 **26 Open、3 Partial、1 Closed**；8 项 P2 为 **7 Open、0 Partial、1 Closed**。17 项产品资格门为 **15 Fail、2 Partial、0 Pass**。这是一份 current-source refresh，不与 App01 重复累计分类总数。

## 2. 审查边界、统计与 currentness

### 2.1 Zircon 冻结集合

统计口径为物理行、非空行、文件 bytes、Rust `#[test]` declaration、`#[ignore]` 与 `include_str!` 次数。fingerprint 将 normalized lowercase relative path 排序，为每个文件拼接 `path + NUL + lowercase(file SHA-256) + LF` 后再取 SHA-256。

| 范围 | 文件 / 行 / 非空行 / bytes / tests / ignored / `include_str!` | fingerprint |
|---|---:|---|
| App production focused set | **120 / 16,628 / 15,105 / 608,328 / 229 / 0 / 17** | `8b27173c442719b7586ba0b386ddbcca12b3a988173b0fa3433167c3fdd715a4` |
| App focused tests | **60 / 9,006 / 8,315 / 341,842 / 197 / 0 / 163** | `5ce241e36ea0aab60bf5fb0ebb0dc7b3db93eef3735f9c71ba6c4130a8fec217` |
| `Cargo.toml` + `build.rs` | **2 / 186 / 179 / 6,491 / 0 / 0 / 0** | `4ce05502bfa291484fd2dda312a55bd3d78dc0179bcab6b23cf057dffe8cdb37` |
| App selected union | **182 / 25,820 / 23,599 / 956,661 / 426 / 0 / 180** | `a576b778adf2d7a54c64a77ba7a1f2eaf530032aa6ff6ef38c4dfaa3ff203ae3` |
| 跨 crate selected consumers | **23 / 4,926 / 4,535 / 170,487 / 16 / 0 / 7** | `a88564a12fabfa85306b59cd842f993820196ccf7aea0f36048d7f015b8e827d` |
| 五引擎 selected references | **19 / 22,420 / 19,345 / 870,891 / 23 / 0 / 0** | `a0021cf60ee87654bdf1cc23a58f59bb3856252c6b4de51e52fc4196de8105eb` |

App production focused set包括 `zircon_app/src/entry`、`src/plugins`、`src/lib.rs`、`src/prelude.rs`、`src/runtime_presenter.rs`、`src/bin/editor.rs` 与 `src/bin/runtime_preview.rs`，排除由 App02 独立拥有的 PBR viewer。focused tests按路径段 `tests`、叶文件 `test.rs`/`tests.rs`/`*_tests.rs` 分类。crate integration仍只有3个文件、4项测试，其中只有 Editor authoring restart 是真实跨 composition 行为。

跨 crate集合冻结 Editor Play/process、Editor gateway session、shared foreign output、Runtime API table 与 dynamic session FFI 的直接消费边界。参考 revision 为 Bevy `fb89a8649d9b359e53ffb6e5492ebb7c059ac8af`、Fyrox `8d815db36494f1badb347547dfc7094bf4fbbdf8`、Godot `8c7e6c5877a78e8e61ea4fd42673219a9091dca7`、Unity Graphics `a7e4c051d256a781ab362c64316b125a1e104694`。`dev/UnrealEngine` 没有独立 Git 元数据，以3个物理文件和参考集合 fingerprint 冻结。

### 2.2 读取方法

1. 从 `Cargo.toml` 与 `EntryConfig` 沿 profile/target/manifest/render/window 解析到 builtin module、first-party plugin group、Core register/activate与bootstrap返回 owner。
2. 分别追踪 Editor GUI、authoring automation、export/headless与runtime binary，比较 linked/dynamic deployment、project composition与退出路径。
3. 从 V7 API load/validate 到 session create、foreign output、wake、surface bind、frame capture、host request、session Drop与DLL unload逐项核对owner。
4. 从 Winit `ApplicationHandler` 沿 window/device/input/IME/gamepad/host request/cadence/present/resize/destroy追踪平台生命周期。
5. 从 Editor `request_play`、process spawn/output pump到runtime CLI reporter和terminal result追踪跨进程状态真值。
6. 对 App01 的全部 P0/P1/P2 与14项 gate逐项复判；新增问题只使用 P1-28..30，不重编号旧项。

### 2.3 工作树与动态证据边界

- Session登记基线为 `f811b3bf474d70347199772a175422333dfb36f6` / epoch 420；成文前仓库已推进到 `79f64878f3b9526517644c055ad3bf5cadfccd0f` / epoch 421。
- focused范围存在其他 Session 或用户的未提交修改，包含 `Cargo.toml`、builtin/engine/bootstrap/first-party plugin、gamepad/host request及其tests；Editor Play与Runtime dynamic FFI也有修改。本文读取当前bytes，不把它们视为集成或资格证据，也不修改这些Rust文件。
- MVP03拥有 frame capture/redraw相关在途工作；本文只判断当前调用与owner关系，不关闭其动态资格。
- review-only没有运行Cargo、binary、DLL、Winit、signal、Android/Web、X11/Wayland、surface/device loss、fault injection、soak或benchmark。静态证据能证明的artifact缺失、调用断路、固定handle和owner丢失不依赖这些动态测试；运行期资格仍由gate决定。

## 3. 当前产品与所有权链

```text
Product CLI / Editor request / export helper
        |
        v
mutable EntryConfig ------------------------+
        |                                    |
        +--> effective plugin manifest       |
        +--> builtin runtime modules         |  repeated selection/materialization
        +--> registration/feature modules ---+  App append + group sort
        +--> Default/Dev/Headless group      |
        v                                    v
BuiltinEngineEntry -> Core register/activate -> CoreHandle
                                                |
                    +---------------------------+-----------------------+
                    |                                                   |
Editor GUI: dynamic LoadedRuntime -> RuntimeSession -> gateway         |
Automation: linked runtime -> linked session -> separate composition   |
Runtime binary: dynamic LoadedRuntime -> Winit -> RuntimeEntryApp ------+
                    |
                    +--> one Window / viewport 1 / one surface bool
                    +--> Win32 native surface OR CPU capture fallback
                    +--> Drop: unbind viewport 1 -> destroy -> maybe abort

Editor Play: spawn child -> stdout/stderr diagnostics -> Playing
Runtime Play: stdout text Starting/Ready/Terminal; no framed consumer/ack
```

| 声明角色 | artifact / runner | 当前真实终点 | 当前判定 |
|---|---|---|---|
| EditorHost | `zircon_editor` + retained Editor host | Core、dynamic session与gateway由局部owner持有，返回后显式drop | 有真实基础，未统一composition/shutdown |
| DesktopClient | `zircon_runtime` + Winit | 单window、viewport 1、Win32 native present或CPU fallback | 仅desktop preview，不是多平台player |
| Headless/Minimal | 无独立artifact；复用client Winit | 固定16 ms pump，缺server退出条件 | profile不是产品 |
| Server | `target-server` feature，无binary | `run_headless()` bootstrap后立即返回 | 产品不存在 |
| Web/Android | additive features，无独立entry/artifact | 仍被`target-client/default-platform`桌面bundle约束 | 声明不构成交付物 |
| EditorPlayChild | runtime binary +逻辑report outlet | spawn即Playing，stdout无协议consumer | 进程存在不等于world ready |
| Export/Embedded | helper返回Core或composition片段 | caller自带loop与shutdown | 不是packaged product host |

## 4. 必须保留的工程基础

1. `LoadedRuntime`继续持有dynamic library，required slot在构造期校验；`RuntimeSession`保持library到destroy之后，`RuntimeFrame<'session>`与foreign releaser表达了正确的跨DLLowner方向。
2. `RuntimeSession` destroy失败时`abort`是必要emergency fence：不能在无法证明foreign worker/callback停止时卸载DLL。目标是补正常quiesce路径与durable emergency evidence，不是删除最后防线。
3. shared `ForeignOutputState`与各`ForeignOutputBudget`已统一host request、profile、operation、plugin event、world query/invalidation的bytes/items/decode time/empty/nesting限制；release失败与protocol fuse测试应保留。
4. wake token registry不暴露Rust object pointer，trampoline含panic containment，并在session成功destroy后注销；这是reactive runner可复用的owner模型。
5. cadence已区分interactive、10 Hz unfocused、1 Hz occluded与headless，并记录accepted/coalesced/suppressed demand；后续应把它提升为resolved policy而非删除。
6. `finish_runtime_process`能合并event-loop、runtime app与runtime session三类terminal failure；两个binary也已让log shutdown失败覆盖成功exit code。
7. IME/cursor/gamepad host request已有typed decode、viewport检查和部分数量/rumble限制；应补completion与generation，不回退为裸平台调用。
8. first-frame文件写入已有staging/flush/sync/replace方向；最终资格仍需绑定同一presented frame token和在途MVP03验证。

## 5. P0 当前源码重判

| ID | 状态 | 当前源码证据 | 必须重构为 |
|---|---|---|---|
| P0-1 | **Open** | `Cargo.toml`仍只有Editor、Runtime、PBR viewer三个binary；`run_headless()` bootstrap后立即返回 | 独立`zircon_server` artifact、无Winit scheduler、signal/admin/health/readiness、bounded drain与确定exit |
| P0-2 | **Open** | Web/Android仍只是附加feature；required `target-client`继续带`default-platform`、desktop Winit/X11/Wayland/input/dynamic DLL | role/platform互斥BuildSet、每平台entry/linkage/artifact/CI receipt |
| P0-3 | **Open** | 无process-wide shutdown state machine；Core/session/window/plugin/log依赖局部Drop与调用点顺序 | 持有完整owner graph的幂等`ShutdownCoordinator`与startup rollback stack |
| P0-4 | **Open** | report outlet仍写stdout行文本；Editor只泵diagnostic，spawn成功即Playing；Ready仍在session create后发出 | versioned framed双向IPC、token/build/snapshot identity、world/first-frame ready、heartbeat、stop ack、deadline/EOF语义 |

## 6. P1 当前源码重判

### 6.1 Composition、配置与诊断

| ID | 状态 | 当前源码证据 | 必须重构为 |
|---|---|---|---|
| P1-1 | Open | `EntryConfig`字段公开可改；setter只重算部分profile/target/manifest/window，仍无`validate/finalize` | intent与immutable `ResolvedProductHostConfig`分离，记录每项provenance与冲突 |
| P1-2 | Open | `EntryRunner`/`BuiltinEngineEntry`/builtin assembly仍有多组`bootstrap_with_*`排列组合 | 单一`ProductCompositionRequest -> ProductComposition` transaction；helper只构造request |
| P1-3 | Open | GUI用`LoadedRuntime::load_default`，automation用linked runtime/session并另建composition | 显式`RuntimeDeploymentMode`，dynamic product与linked fixture共享同一合同和矩阵 |
| P1-4 | Open | Editor preparation、App Core composition、DLL session与gateway继续多次投影manifest/capability/module truth | 带generation/hash/build identity的`ProjectRuntimeCompositionReceipt`，所有consumer验证同一代 |
| P1-5 | Open | builtin selection warning与profiling failure仍直接`eprintln!`，启动日志初始化又偏晚 | typed bootstrap event + early ring buffer + single sink；stdout保留机器协议 |

### 6.2 Dynamic library、ABI 与 owner

| ID | 状态 | 当前源码证据 | 必须重构为 |
|---|---|---|---|
| P1-6 | Open | V7先实现size/offset-aware读取，随后仍要求`size_bytes == size_of::<ZrRuntimeApiV7>()` | 明确选择exact-frozen或prefix-compatible policy，并以older/newer table fixture验证 |
| P1-7 | Open | runtime library仍从env、sibling与`deps`猜路径，只校验symbol/version/slots | shipping manifest、hash/signature、build ID、target triple、channel与允许目录 |
| P1-8 | Open | linked与dynamic load都传`ZrHostApiV1::empty(...)` | 最小稳定host services table、capability negotiation、thread/reentrancy/shutdown lease |
| P1-9 | Open | Editor gateway仍逐字段重建一张部分V7 table | 带library lease、size、version与capability的validated `RuntimeApiView` |
| P1-10 | Open | surface lifecycle仍是一枚共享`AtomicBool`，Drop硬编码unbind viewport 1 | session-owned viewport/surface generation registry，反向枚举解绑 |
| P1-11 | Open | 运行态直接destroy，无request-stop/quiesce/cancel/drain/poll-shutdown阶段 | versioned session shutdown protocol与deadline；失败后durable evidence再emergency abort |
| P1-12 | **Closed** | shared foreign-output层已为六类输出统一bytes/items/decode-time/nesting/empty policy，并由App与Editor gateway复用 | 保持单一policy registry；后续真实fault DLL资格由P1-26/gate 7承载 |
| P1-13 | Open | 控制面仍以whole-buffer JSON为主；缺schema hash、unknown-field policy、cursor/page、large blob handle | 高频POD、低频versioned bounded envelope、大对象stream/shared blob；计量bytes与CPU |

### 6.3 Window、surface、input 与 cadence

| ID | 状态 | 当前源码证据 | 必须重构为 |
|---|---|---|---|
| P1-14 | Open | 一个window/presenter/viewport，viewport固定1；incoming `WindowId`/`DeviceId`被丢弃 | `WindowRegistry + ViewportRegistry`，明确primary/secondary/embedded/offscreen与输入owner |
| P1-15 | Open | `Destroyed`只转发runtime event，未与CloseRequested共享host清理transition | 按WindowId幂等撤销input/request/surface/window owner，再按policy退出或重建 |
| P1-16 | Open | `ApplicationHandler`无suspended/exiting/memory lifecycle | 平台lifecycle state machine，Android/Web/desktop fixture验证surface释放重建 |
| P1-17 | Open | resize直接`bind_current_window_surface()`；bind/present失败多为fatal，无generation、fence与Lost/Outdated恢复 | surface replace transaction、旧代停止提交、recoverable/error taxonomy与device policy |
| P1-18 | Open | CPU fallback仍capture完整RGBA、逐像素转XRGB并丢alpha；无row pitch/color/HDR合同 | 明确degraded mode，format/alpha/color/stride/generation ABI与共享texture/blit优先 |
| P1-19 | **Partial** | unfocused已改为100 ms，occluded为1 s；headless仍固定16 ms且复用Winit | 分离simulation/render/network/background scheduler，resolved cadence policy与overrun/backpressure |
| P1-20 | Open | host request虽有typed parse和部分drain limit，仍无request ID/ack/retry；无window时可静默返回，批处理缺总execution-time continuation | generation-qualified request/completion、typed unsupported/transient/permanent outcome及count/bytes/time预算 |
| P1-21 | Open | native present后为first-frame evidence再次`capture_frame`，没有present token/fence绑定 | renderer发布presented frame token，capture异步绑定同一token与build/session/surface identity |
| P1-22 | **Partial** | 顶层现可合并event-loop/app/session三类错误；每个app/session owner内部仍只保留首错 | bounded ordered failure ledger、phase/component/severity/time/suppressed count |

### 6.4 Process、证据与性能

| ID | 状态 | 当前源码证据 | 必须重构为 |
|---|---|---|---|
| P1-23 | Open | CLI/project/env/Play starting report等失败仍发生在正式process log前 | 进程入口即启用bootstrap ring，正式sink初始化后replay并脱敏typed fields |
| P1-24 | **Partial** | log shutdown失败已覆盖成功exit code；但“teardown complete”在shutdown前写入，durability失败仍留下完成字样 | durable terminal receipt只在flush确认后发布；失败走OS emergency sink/failure ledger |
| P1-25 | Open | 426个focused test attributes仍有180次`include_str!`，source-order/substring guard占比高 | architecture lint只保留必要边界，主体改为state-machine、binary、DLL与platform行为测试 |
| P1-26 | Open | integration仍3文件/4测试；没有真实runtime binary、staged DLL、server、signal、destroyed、surface loss或fault DLL矩阵 | packaged artifact smoke、dynamic/linked双fixture、逐ABI slot fault injection与shutdown soak |
| P1-27 | Open | 有局部cadence/foreign-output计数，但无同workload host startup/frame/idle/shutdown/serialization基线 | 固定硬件/OS/build/workload的P50/P95/P99、RSS/power/bytes/crossing预算和回归门 |
| P1-28 | **Open（新增）** | `runtime_native_surface_target()`只匹配Win32；X11/Wayland/macOS/Android/Web均返回None并退回CPU capture | 每个声明平台的native surface adapter、capability/build closure、present/suspend/loss资格；无adapter即BuildSet拒绝 |
| P1-29 | **Open（新增）** | `NativePluginRuntimeBootstrap`注释要求host比runtime graph长寿，但public `into_core(self)`只返回Core并drop host | 删除owner-losing API或让`ProductComposition`/Core lease持有host；用drop/late-call fixture证明卸载顺序 |
| P1-30 | **Open（新增）** | startup/terminal路径先`report_play_startup(...)?`再返回primary error；stdout写失败可覆盖原始失败，outlet又未framing/校验 | report failure作为secondary ledger项，永不替换primary；framed writer校验token/长度并定义backpressure |

## 7. P2 当前源码重判

| ID | 状态 | 当前源码证据 | 必须重构为 |
|---|---|---|---|
| P2-1 | Open | capture/exit/input probe/runtime DLL/profiling等env仍散落 | typed startup option schema、source/provenance receipt、test与shipping option分层 |
| P2-2 | Open | 60 s frame demand cap、16 ms headless等policy default仍藏在host常量 | named policy default进入resolved config、diagnostics和telemetry |
| P2-3 | **Closed** | `PluginGroupBuilder::finish()`现为`#[cfg(test)] pub(crate)`；产品路径使用typed `try_finish` | 保持产品面无panic shortcut |
| P2-4 | Open | binary已叫`zircon_runtime`，源码/诊断仍混用runtime preview产品语义 | 明确PackagedPlayer、EditorPlayChild、DeveloperPreview名称与policy |
| P2-5 | Open | 用户请求profiling时export failure仍只写stderr，不进入terminal result | capture stop/export纳入shutdown deadline与typed terminal outcome |
| P2-6 | Open | library默认只猜sibling与`deps` | 发行manifest列明artifact/hash/ABI/build/platform与候选诊断 |
| P2-7 | Open | CLI parser、help、diagnostic与option contract仍由手写长字符串同步 | typed option schema生成parser/help/diagnostic字段 |
| P2-8 | Open | 163次test-side `include_str!`继续锁死文件名、文本和源码顺序 | 行为测试/专用AST validator替代大部分substring guard |

## 8. 五套参考源码给出的工程差异

| 能力 | Unreal | Bevy | Fyrox | Godot | Unity Graphics | Zircon 当前差异 |
|---|---|---|---|---|---|---|
| 产品主循环 | `GuardedMain`与`FEngineLoop`区分pre-init/init/tick/exit | `App`可替换runner，以`AppExit`结束 | normal与headless executor独立 | setup/start/iteration/cleanup | 不含Player loop源码，不作此项推断 | server无runner，headless复用或立即退出 |
| cleanup owner | cleanup guard保证`EngineExit`必达 | plugin finish/cleanup + runner exiting | plugin init/deinit与graphics context created/destroyed | worker/resource/script/render/server/display/input有序cleanup | HDRP `Dispose`显式释放render graph、XR、sky/water/postprocess/RTHandle等 | 主要靠局部Drop；无全进程依赖逆序coordinator |
| 多窗口/事件 | platform/Slate/RHI owner | `WindowId -> Entity`路由，unknown window可诊断 | event与graphics context生命周期明确 | DisplayServer/window ID | render pipeline按camera/XR/resource owner清理 | WindowId丢弃，viewport固定1 |
| suspend/resume | platform/RHI生命周期 | resumed/suspended/exiting显式处理，exiting时event loop仍可清窗 | resumed创建context、suspended销毁context | OS/display server分阶段 | `Dispose`/recreate区分Editor验证与禁用 | suspended/exiting缺失，surface失败多终止 |
| 动态代码/插件 | module manager、loading phase与逆序unload | 主要静态composition | plugin/dylib lifecycle可观察 | GDExtension初始化级别与shutdown | package/assembly manifest表达构建闭包 | DLL owner基础存在，build/trust/quiesce与native plugin owner合同未闭合 |
| 证据与性能 | trace/benchmark和广泛phase点 | schedule/runner可测 | executor分型便于fixture | cleanup benchmark点 | 包级测试与graphics lifecycle证据 | source guard多，缺真实artifact/fault/paired workload |

Zircon不应复制Unreal的宏、全局单例或具体线程模型；应复制其“产品阶段有唯一owner、退出按依赖逆序、失败仍保证必要清理”的工程约束。Unity Graphics本地corpus只证明graphics package/resource lifecycle，不能用于声称Unity Player/Editor lifecycle已经比较完成。

## 9. 目标架构

### 9.1 ProductRole 与 Resolved BuildSet

定义`EditorHost`、`DesktopClient`、`Server`、`WebClient`、`AndroidClient`、`EditorPlayChild`、`Commandlet`、`Embedded`。`ProductRoleRequest`经过platform、artifact、linkage、capability、module/plugin、render/input/window与shutdown policy resolver，生成immutable `ResolvedProductBuildSet`。Cargo feature只表达“可编译能力”，不得充当运行期角色真值；任何缺native surface、runner、artifact或required provider的组合在Core activation前拒绝。

### 9.2 ProductComposition

唯一transaction输出`ProductComposition`：resolved config、Core owner、runtime deployment/API view/session、native plugin host、module/plugin/capability graph、artifact/build identity、startup diagnostics与rollback stack。所有便捷API只构造request；不能再有返回裸Core并丢失required owner的路径。

### 9.3 ProductHost

`ProductHost`拥有runner、window/viewport/surface generation registry、runtime session、host request executor、cadence scheduler与failure ledger。desktop/mobile/web使用不同platform adapter但共享host transition；server拥有独立无窗口scheduler。simulation、render、network和maintenance cadence分离，WindowId/DeviceId/viewport generation贯穿输入与host request。

### 9.4 ShutdownCoordinator

统一阶段至少为：`RequestStop -> Quiesce -> Drain -> ReleasePresentation -> DestroyRuntime -> DeactivateCore -> FlushEvidence -> PublishTerminal -> Exit`。每阶段幂等、可超时、保留primary+secondary failure，并被正常退出、startup rollback、window close、server signal、Editor stop与emergency branch共同复用。DLL unload只能发生在callbacks/workers、surface、operation/watch/subscription和native plugin generation均已证明retired之后。

### 9.5 Typed Play Child IPC

stdout/stderr只承担human diagnostics。独立framed control channel必须携带随机session token、protocol/build/project/snapshot identity、sequence、payload length与checksum；状态至少区分spawned、session-ready、world-ready、first-frame-ready、stopping、terminal。Editor在所需ready级别前不得进入Playing；stop先请求cancel并等待ack，deadline后才终止process tree。

## 10. 分阶段重构计划

### M0：冻结合同，停止扩散

- 冻结ProductRole/artifact/platform/linkage/capability矩阵和exit code。
- 定义composition receipt、host/shutdown state machine、failure ledger与V7兼容政策。
- 禁止新增`bootstrap_with_*`、裸Core owner丢失API和未归档env product switch。

### M1：真实Server与统一shutdown

- 增加`zircon_server` artifact、无Winit scheduler、fixed/update/network tick policy。
- 接入SIGINT/SIGTERM/Windows service或console stop、health/readiness、bounded drain。
- Core反向module cleanup、日志/profiling durability与startup rollback接入同一coordinator。

### M2：收敛composition与dynamic runtime

- `ProductComposition`持有Core、API view、session与native plugin host，删除owner-losing helper。
- 明确V7 exact/prefix policy，加入host services/capability/build/trust handshake。
- 增加session quiesce、operation/watch/subscription/callback drain和fault DLL。

### M3：平台host与presentation工程化

- 建立multiwindow/viewport/surface generation registry，处理suspend/resume/exiting/memory pressure。
- 为Win32、X11、Wayland、macOS、Android、Web分别建立受BuildSet约束的native surface adapter。
- surface Lost/Outdated/device loss分类恢复；CPU readback明确为diagnostic degraded mode。

### M4：Play child协议闭环

- 建立双向framed IPC、token/build/snapshot identity、ready phase、heartbeat与stop ack。
- Editor controller由协议驱动状态，不再以spawn成功等价Playing。
- report/write/parse failure进入secondary ledger；primary startup/terminal cause始终保留。

### M5：产品、故障与性能资格

- CI启动真实Editor、Desktop Client、Server与Play child artifact；平台成熟后加入Web/Android。
- fault DLL覆盖短/长表、missing slot、oversized/malformed output、late callback、free/destroy失败。
- platform fixture覆盖4 window、external destroyed、suspend/resume、surface/device loss。
- 建立同硬件/OS/build/workload的startup、idle、frame、Play、shutdown P50/P95/P99、RSS/power/bytes基线与reference paired run。

## 11. 产品资格 Gate

| Gate | 状态 | 关闭要求 |
|---|---|---|
| G01 packaged server | **Fail** | 无图形环境运行多tick，通过管理命令、signal与内部fatal三类原因优雅退出 |
| G02 role/platform artifact matrix | **Fail** | 每个声明角色有独立BuildSet与CI artifact，无互斥平台additive拼接 |
| G03 startup rollback | **Fail** | 任一阶段注入失败后owner census归零并保留rollback receipt |
| G04 idempotent shutdown | **Fail** | 同一stop重复调用返回同一terminal truth，各阶段有deadline与次级错误 |
| G05 Core reverse cleanup | **Fail** | 全部active module按依赖逆序cleanup，失败不伪装回Running |
| G06 DLL quiesce/unload | **Fail** | 无late callback/worker；destroy失败在durable emergency record后abort |
| G07 foreign output fault handling | **Partial** | shared预算已有；仍需真实fault DLL、release证明与session protocol-failed状态 |
| G08 multiwindow routing | **Fail** | 4个并发window/viewport按ID路由，单窗销毁不污染其他owner |
| G09 lifecycle/surface recovery | **Fail** | suspend/resume、Lost/Outdated/device loss与generation替换有真实platform fixture |
| G10 Play handshake | **Fail** | 校验token/build/snapshot并收到world/first-frame ready后才进入Playing |
| G11 durable log/profiling terminal | **Partial** | log失败已影响exit；仍需flush后发布receipt及profiling进入terminal ledger |
| G12 behavioral test evidence | **Fail** | source guard显著下降，binary/DLL/platform/state-machine成为主要证据 |
| G13 host performance budget | **Fail** | startup/frame/idle/shutdown/serialization/RSS/power有P50/P95/P99回归阈值 |
| G14 paired reference benchmark | **Fail** | 同硬件、OS、build、场景、画质、窗口/帧率与原始trace可复现 |
| G15 native surface platform closure | **Fail** | 所有声明desktop/mobile/web平台有native present或BuildSet明确拒绝 |
| G16 native plugin owner lifetime | **Fail** | public API不能丢host，retirement/unload顺序有drop/late-call fixture |
| G17 primary failure preservation | **Fail** | report/IPC写失败只能成为secondary，不能覆盖startup/terminal primary |

## 12. 相邻责任边界

- `zircon_runtime/01`与Runtime46拥有Core module activation、rollback、反向cleanup与service revoke；App负责让产品host真实调用并把结果纳入terminal receipt。
- `zircon_runtime_interface/01`拥有V7布局、handle、FFI与version policy；App拥有validated view、library/session lease与产品deployment/trust使用方式。
- `zircon_runtime_interface/05`与`zircon_runtime_host`拥有foreign output safe owner/budget/fuse；App只负责所有调用点统一消费并通过真实fault DLL资格。
- `zircon_editor/07`拥有Editor Play UI/state/controller；App与Editor共同拥有typed child protocol，任何一侧单独写stdout或单独改UI都不能关闭P0-4。
- `zircon_plugins/01`拥有native discovery/signature/hot reload与generation retirement；App composition必须持有native host并参加shutdown。
- App02拥有PBR viewer与RenderDoc/evidence tool host；其offscreen或viewer成功不能替Desktop Client、Server或Play child背书。
- Runtime136拥有最终module/catalog/profile/feature composition compiler；App必须成为frozen plan consumer，不再另行append和选择。

## 13. 本轮完成定义

本轮完成当前源码静态review、App01全量编号复判、五引擎selected reference对照和分阶段重构计划；没有修改production/Test/Cargo代码，也没有运行动态资格或宣称性能优于Unreal。App08替代App01的currentness，但App01保留历史证据和原始问题定义；统计时两者不得重复累计。

实施前必须重新计算本文selected fingerprints并核对工作树owner。任何单点修复只有在对应产品gate通过后才能从Open/Partial变为Closed；新增类型名、source guard或linked fixture不能单独构成工程级完成证据。
