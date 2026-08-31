---
related_code:
  - zircon_app/Cargo.toml
  - zircon_app/build.rs
  - zircon_app/src/bin/editor.rs
  - zircon_app/src/bin/runtime_preview.rs
  - zircon_app/src/entry
  - zircon_app/src/plugins
  - zircon_app/src/reference_cpu_presenter.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/zircon_runtime/01-core-runtime-lifecycle-registry-review.md
  - docs/plans/zircon_runtime/frameworks/02-module-kernel-and-lifecycle-unification.md
  - docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md
  - docs/plans/zircon_editor/editor/04-pie-and-simulation.md
  - docs/plans/zircon_editor/editor/16-cli-args-and-hub-integration.md
  - docs/plans/performance/01/2026-08-14-app-entry-root-cli-current-review.md
  - docs/plans/performance/01/2026-08-14-app-runtime-entry-support-current-review.md
  - docs/plans/performance/01/2026-08-14-app-editor-entry-current-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Launch/Private/Launch.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Launch/Private/LaunchEngineLoop.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Launch/Private/Windows/LaunchWindows.cpp
  - dev/UnrealEngine/Engine/Source/Programs/UnrealBuildTool/Configuration/Rules/TargetRules.cs
  - dev/UnrealEngine/Engine/Source/Programs/UnrealBuildTool/System/TargetReceipt.cs
  - dev/bevy/crates/bevy_app/src/app.rs
  - dev/bevy/crates/bevy_winit/src/state.rs
  - dev/Fyrox/fyrox-impl/src/engine/executor.rs
  - dev/godot/main/main.cpp
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: m0_shutdown_disposition_editor_exit_abi_source_implemented_static_review_passed_managed_validation_pending
source_recheck_required: true
---

# 01 · Product Host、Bootstrap、Loop、Dynamic Runtime 与 Shutdown 工程化差距

## 1. 结论

`zircon_app` 已经有一批不能抹掉的工程基础：模块选择会先做拓扑验证，Editor GUI 能把 Core、动态 runtime session 和 retained host 串成真实生命周期；runtime DLL 的函数表、owned buffer、frame buffer 和 wake callback 都有显式 owner；runtime event loop 能合并 event-loop、callback 与 session teardown 错误；窗口帧 cadence 也区分 continuous、reactive、low-power 和 headless，并暴露基本统计。这些内容明显超过一次性 demo bootstrap。

但从产品宿主角度看，当前实现仍是“桌面单窗口 runtime preview + Editor 专用入口 + 若干 bootstrap API”，不是 Unreal `Launch`/`FEngineLoop` 级别的统一产品外壳。`target-server` 没有 binary，唯一 `run_headless()` 激活模块后立即返回；可执行的无窗口 profile 只能借用 `target-client` 的 Winit/DLL 路径。`target-client` 又强制带入 desktop X11/Wayland、窗口、输入和动态 runtime，因此 manifest 中列出的 Web/Android/headless 能力没有形成各自可构建、可启动、可停机的产品角色。

当前停机安全主要集中在 `RuntimeSession::drop`：若 DLL session destroy 失败会直接 `abort`，避免带活 callback 卸载库。这是正确的最后防线，却不是完整 shutdown architecture。2026-08-27 的 composition 硬切已关闭公开 `EngineEntry::bootstrap -> CoreHandle` 旁路，Editor/runtime binary 也会把日志 drain 失败提升为进程失败；但 App 仍没有 process-wide shutdown coordinator、signal/OS termination owner、分阶段 quiesce/drain/flush/unload 顺序或幂等状态机。此前 Runtime01 已确认产品退出没有显式触发完整模块 cleanup，因此不能把“Rust drop 最终发生”当成与 Unreal/Fyrox/Godot 有序停机等价。

Play-in-Editor 的进程边界也尚未闭合。`--play-report-pipe` 目前只是写入 stdout 文本中的 outlet 标签，不创建或连接 pipe；Editor process backend 只把 stdout 当普通 diagnostics 收集，不解析 starting/ready/start-failed/terminal，更不据此推进 Play 状态。runtime 还在 session 创建后、首帧和场景可用性尚未验证时发出 `ready`。这不能作为可靠的启动握手、健康检查或超时/取消协议。

本轮登记 4 项 P0、27 项 P1、8 项 P2。首要重构不是继续增加 `bootstrap_with_*` 变体，而是定义 `ProductHost`/`ProductRole`/`ShutdownCoordinator` 三个权威边界：每个产品角色拥有可构建 artifact、runner、平台能力和退出条件；host 拥有 window/viewport/surface/session 资源图；shutdown coordinator 从停止接单到卸载 DLL 和日志落盘执行可观测、幂等、超时受控的反向阶段。没有 server/mobile/web 实机或 CI artifact、进程故障注入和同负载基线，不得宣称产品宿主能力或性能达到、超过 Unreal。

## 2. 审查边界与证据

### 2.1 当前源码范围

| 集合 | 文件 / 物理行 | 本轮证据 |
|---|---:|---|
| production focused set | 117 / 14,947 | E3：entry/bootstrap、runtime/editor runner、dynamic library/session、runtime app、plugin group、两个产品 binary |
| focused tests | 56 / 6,986 | E2：396 个 test attributes、0 ignored；其中 180 次 `include_str!` |
| combined focused set | 173 / 21,933 | E2-E3；production fingerprint `743cb2c27f4ca99e3f325cf1a711d886cb840b4aae7b853abf094ba3bd0f5ed1` |
| crate-level integration tests | 3 files / 4 tests | 一个真实 Editor authoring restart；其余两个为源码顺序和 plugin builder contract |

focused set 包含 `zircon_app/src/entry`、`src/plugins`、`src/reference_cpu_presenter.rs`、`src/lib.rs`、`src/prelude.rs`、`src/bin/editor.rs` 和 `src/bin/runtime_preview.rs`，排除 `src/bin/zircon_shader_pbr_viewer`。PBR viewer 有独立 scene/bootstrap、后台载入、native/CPU present、evidence 与 RenderDoc 生命周期，下一份 `zircon_app/02` 单独审查，不能用它的测试和工具能力替 runtime 产品入口背书。

production classifier 排除路径段 `tests`、叶文件 `test.rs`/`tests.rs` 和 `_tests.rs`，但保留 production 文件内嵌的 `#[cfg(test)]` 物理行。fingerprint 算法为路径排序、逐文件 SHA-256，再对 `path<TAB>hash<LF>` 清单取 SHA-256。成文前 `zircon_app` 未出现在工作区修改列表；实施前仍需重取指纹，因为相邻 Runtime、Editor 和计划文件存在其他 Session 修改。

本轮 owner chain 分四条追踪：

1. `EntryConfig` -> module/plugin selection -> `BuiltinEngineEntry` -> `CoreRuntime` activation -> `CoreHandle`。
2. `zircon_runtime` binary -> CLI/project/Play args -> DLL load -> Winit -> `RuntimeSession` -> `RuntimeEntryApp` -> surface/present -> teardown。
3. `zircon_editor` binary -> project preparation -> Core -> dynamic runtime gateway -> retained host -> session/core/log teardown。
4. export/automation -> linked runtime/native plugin host -> composition result -> explicit/RAII close。

只存在于 source guard、测试 helper、帮助文本或未被 Cargo binary 选择的函数，不计为产品能力。

### 2.2 参考源码边界

- Unreal `GuardedMain` 用 cleanup guard 保证 `EngineExit()` 必达，显式区分 pre-init、init、tick、Editor exit、engine exit 和 platform app exit；`FEngineLoop::Exit` 依赖有序停止 async loading、streaming、input、windows、modules、render thread、shader pipeline 和平台层。它是复杂宿主的工程上限，不要求 Zircon 复制全局变量或宏。
- Bevy `App` 将 runner、update、`AppExit`、plugin finish/cleanup 分离；Winit runner 按 `WindowId -> Entity` 路由事件，实现 resumed/suspended/exiting，并在 event loop 仍有效时先清窗口。这证明 Rust 中可以同时保持产品 runner 可替换、窗口多实例和显式 cleanup。
- Fyrox 为 headless 提供独立 loop，而不是创建桌面 Winit event loop 后隐藏窗口；normal runner 也显式处理 resumed/suspended 时 graphics context 的创建与销毁。
- Godot `Main::setup`、`Main::iteration`、`Main::cleanup` 形成可观测阶段，cleanup 明确清线程任务、脚本、render sync、module/server、display、input 和 restart-on-exit。
- 仓内 Unity Graphics 是渲染包而非 Unity Player/Editor 主循环源码，本轮不从中推断闭源 application lifecycle。它将在 PBR viewer 与图形宿主交界审查中只作为可证明的 graphics resource 参考。

### 2.3 明确未做

本轮没有运行 Cargo、真实 Editor/runtime binary、DLL ABI mismatch、signal/console-control、Android/Web、macOS fullscreen、窗口销毁、surface loss、device loss 或进程崩溃注入；也没有执行同负载 Unreal/Fyrox/Bevy/Godot benchmark。静态证据可以证明 Cargo artifact 缺失、feature dependency、固定 handle、被忽略的 callback 参数、输出 channel 和 teardown 调用顺序，但实际平台 API 行为必须由后续动态 gate 给出。

## 3. 当前产品角色不是一个闭合矩阵

| 声明角色 | Cargo artifact | 实际 runner | window/platform | terminal owner | 结论 |
|---|---|---|---|---|---|
| Editor host | `zircon_editor` | retained Editor host + dynamic runtime gateway | desktop Winit | host return后 drop session/core，binary drain log | 有真实基础，但部署与自动化路径分叉 |
| Client runtime | `zircon_runtime` | Winit `RuntimeEntryApp` | 单窗口、viewport 1 | Winit return + session Drop | 桌面 preview 可运行，不是多平台 product host |
| Headless/minimal profile | 无独立 artifact | client binary 中隐藏窗口 | 仍创建 Winit，固定 16 ms pump | 无 signal/管理面退出条件 | 仅 client preview 的一种配置 |
| Server target | **无 binary** | `run_headless()` bootstrap 后返回 | `platform-headless` | 不存在运行期 | P0：产品不存在 |
| Web/Android client | 无匹配 artifact/entry | client binary 要求 `target-client` | `target-client` 强制 desktop default-platform 与 DLL | 未实现 suspend/resume/termination | P0：feature 名称不等于可交付产品 |
| Export bootstrap | library API | 调用方自带 loop | 调用方决定 | 调用方决定 | 只有 composition helper，不是 packaged player |

这里最危险的不是“功能少”，而是同一个 profile 名称在 module selection、DLL session、Winit host 和 Cargo artifact 四层含义不同。`RuntimeSessionProfile::Headless` 可以传给 DLL；`EntryProfile::Headless` 可以选 module；`target-server` 可以编译 feature；但三者没有汇合成一个能持续运行并优雅退出的 server executable。

## 4. 可保留的真实基础

### 4.1 Module selection 与 Editor startup 已有可诊断路径

`BuiltinEngineEntry` 会生成 module selection report，拓扑排序失败能返回 typed `CoreError`；Editor startup 对 project、bootstrap、manager、runtime library、session 和 gateway 分阶段包装 recovery hint。Editor host 返回后会显式 drop runtime session，并把 host failure 与 teardown failure 合并。这套分阶段诊断应成为统一 host 的基础，而不是退回一个 `main().unwrap()`。

### 4.2 Dynamic runtime table 和 owner lifetime 基础正确

`LoadedRuntime` 持有 `libloading::Library`，函数表指针只在 library owner 存活时使用；required slot 在构造时验证，optional tail slot 按 offset/size 读取。`RuntimeSession` 又把 library 保留到 destroy 之后。`RuntimeFrame<'session>` 通过 lifetime 阻止 frame 越过 session，Drop 会调用 foreign free，并把 release failure写入 teardown state。这个 owner 关系值得保留。

### 4.3 Owned buffer 和 plugin event page 有部分防御

owned buffer 会检查 len/capacity、null/free callback、`isize::MAX` 和 frame 精确像素长度；失败路径尝试同时保留业务错误与 release 错误。plugin event page 还有 64 deliveries / 256 KiB 上限。缺口是这些预算没有推广到 host request、profile、operation、world query 等全部 foreign outputs。

### 4.4 Wake registry 与 reactive cadence 有明确 owner

wake token 不暴露 Rust object pointer，trampoline catch panic，registration 在 successful session destroy 后注销。frame cadence 能合并 reactive request、接受 runtime deadline、统计 accepted/coalesced/suppressed，并在 control-flow publish 前只做一次最终决策。这是后续统一 runner 的可用组件。

### 4.5 DLL destroy failure 的 abort 是合理最后防线

如果 foreign session destroy 失败，宿主无法证明 worker 和复制的 callback 已停止；此时继续 Drop 并卸载 DLL 可能执行已卸载代码。当前 `abort` 比继续运行安全。重构目标是让正常路径具备 quiesce、timeout、crash record 和可测试状态机，不是删除最后防线。

## 5. P0：先建立真实产品宿主

### P0-1：`target-server` 没有可执行产品，`run_headless()` 立即结束

`Cargo.toml` 只有 `zircon_editor`、`zircon_runtime` 和 PBR viewer 三个 binary；`target-server` 仅打开 `zircon_runtime/target-server`、diagnostic log 和 `platform-headless`。`entry_runner/headless.rs` 的完整实现只是 bootstrap `EntryProfile::Headless`，把 handle 绑定到 `_` 后返回。模块刚激活就随作用域结束，既不 tick world/system，也不监听 stop、signal、admin、health 或 watchdog。

client binary 中的 `minimal/headless` profile 不能替代 server：它依赖 `target-client`/Winit/DLL，创建 event loop 后以固定 16 ms cadence tick，`WindowExitCondition::DontExit` 又没有外部退出 owner。真正 server 至少需要独立 artifact、无窗口 runner、可配置 fixed/update schedule、SIGINT/SIGTERM/console service stop、health/readiness、backpressure、graceful drain timeout、非零 fatal exit 和 deterministic teardown。

验收必须启动 packaged server fixture，跑过多帧 world/system，分别通过正常管理命令、SIGINT/SIGTERM 和内部 fatal error退出；证明新请求停止、任务/网络/存档有界 drain、模块反向 cleanup、日志 flush、句柄归零，并返回稳定 exit code。

### P0-2：声明的 Web/Android/client feature 不能组成对应产品 artifact

`zircon_runtime` binary 要求 `target-client`，而 `target-client` 无条件包含 `default-platform`；后者又包含 desktop window/Winit、X11、Wayland、mouse/keyboard/touch/gamepad 和 gilrs。`platform-web`、`platform-android-game-activity` 等只是附加 feature，无法从 required `target-client` 中移除 desktop bundle。runtime runner 又无条件走 `libloading` sibling library；`platform_runtime_library_name()` 只覆盖 Windows/macOS/Unix，没有 Web 分支；ApplicationHandler 也未实现 suspend/exiting 等移动 lifecycle。

因此 Cargo 中出现 `platform-web`/Android 名称，不等于存在可编译、可启动的 Web/Android player。必须以 role-specific feature bundles 或 platform packages 硬切：desktop client、web client、Android game/native activity、headless server分别拥有 entry、runtime linkage策略、window/surface/input feature、asset location、panic/crash策略和 CI artifact。禁止继续用一个 additive `target-client` 把互斥平台能力全部相加。

### P0-3：没有 process-wide shutdown coordinator，正常退出仍依赖局部 Drop 偶然排序

首轮审查时，`EngineEntry::bootstrap` 和 `BuiltinEngineEntry::bootstrap` 完成 module activation 后直接向公开调用方返回 `CoreHandle`；2026-08-27 已将两者降为 crate-private，并由不可公开借出 Core 的 `ProductComposition` 统一持有 Core 与插件 owner。App 已定义 `Running -> Quiescing -> Draining -> ReleasingPlatform -> DestroyingRuntime -> DeactivatingModules -> FlushingDiagnostics -> Exited` 单调状态与ordered failure ledger，但状态机尚未持有实际资源，也没有 signal/OS termination owner。Runtime01 已确认产品入口不显式触发完整 module cleanup。Editor/runtime failure冷路径已共享ledger并检查 `shutdown_process_log(...)`，但log结果仍在ledger外；session destroy failure则跳过全部剩余收尾直接 abort。

必须新增进程级 coordinator，持有 Core、session、native plugin host、windows/surfaces、task/network/asset/render drains、profiling与diagnostic sinks的依赖图。每个阶段要幂等、可超时、可记录 primary+secondary failure；正常退出、startup rollback、window close、server signal、Editor stop、panic/crash前置 flush和DLL destroy failure都必须经过同一状态机或明确的 emergency branch。仅靠字段 Drop 顺序和 source-order test不达标。

### P0-4：Play process 的“report pipe”不是握手 channel，Editor 也不消费状态

runtime 的 reporter 只是 `write_all` 到 stdout，记录 `zircon_play_report outlet=<name> ...`；传入的 pipe name只作为文本标签。Editor `ProcessPlayBackend` 建立 piped stdout/stderr后把每一行包装为 diagnostics，未解析或校验 outlet/phase，也不会等待 ready、处理 start-failed、识别 terminal 或做 handshake timeout。backend `start()` 在 spawn 成功后立即返回；runtime 又在 session create 完成后、第一帧和场景 readiness之前发 `ready`。

这会让“进程存在”“session handle 创建”“目标场景可运行”“首帧呈现”混成同一状态，无法可靠驱动 PIE UI、自动测试、崩溃归因和 stop race。需要 versioned typed IPC：随机不可猜 session token、双向握手、协议版本、project/snapshot identity、runtime build/ABI identity、starting/session-ready/world-ready/first-frame-ready/terminal phase、heartbeat、cancel/stop ack、deadline与EOF语义。若继续复用 stdout，也必须有独立 framed protocol parser和严格 consumer；不能把普通日志行冒充 pipe。

## 6. P1：补齐配置、ABI、窗口和失败恢复

### 6.1 Product composition 与配置真值

#### P1-1：`EntryConfig` 可构造互相矛盾的 profile/target/render/window/plugin 组合（M0 源码已收敛，受管验证待执行）

首轮审查发现公开字段与链式 setter 会独立改写 `profile`、`runtime_profile`、`target_mode`、manifest、export profile、render profile和window descriptor。2026-08-27 的 M0 源码切片已删除该可变派生模型：`EntryConfig` 只保存用户请求，`resolve()` 统一校验 role/target/runtime profile/export、合并 profile manifest与显式插件选择，并在任何模块编译或native export插件加载前失败。

当前 `ProductRoleRequest` 已冻结desktop/editor/server以及web/android/editor-play/commandlet/embedded角色词表；只有已有owner的desktop/editor/server可解析，其余角色返回typed unsupported-role error，不伪造支持。`ResolvedProductHostConfig`字段私有且携带 `ProductHostConfigProvenance`，crate-private entry compiler、Editor预备流程、一方插件投影和runtime module composition只消费解析结果。Plugin manifest以runtime profile为base并按ID叠加entry/export选择，required/optional同ID返回typed conflict，optional不能降级已有required；manifest provenance用无堆分配的source set保留全部贡献源，两个Editor字段分别记录来源。Export profile的`runtime_profile_id`不再被setter清空，并进入现有module selection/composition receipt。`ExportRuntimeBootstrapConfig`也已删除重复的`EntryProfile`与target mode输入，统一从`ExportProfile.target_mode + target_platform`投影product role；Android、Web/Wasm、iOS embedded在专属host owner缺失时分别以`AndroidClient`、`WebClient`、`Embedded` typed fail-closed，不再静默冒充desktop client。platform capability resolver与统一`ProductComposition`源码均已在本轮后续切片落地；受管行为验证仍待执行。

#### P1-2：bootstrap public surface 过度排列组合，缺少单一 composition transaction（M0 源码已收敛，独立复核通过，受管验证待执行）

`EntryRunner` 同时公开 report/no-report、first-party/runtime/feature/native plugin/export-root等大量 `bootstrap_with_*`；`BuiltinEngineEntry` 又有对应 `for_config_with_*`，`builtin_modules.rs` 再复制多条 selection route。调用方是否保留 selection report、native host owner和feature registration取决于选中了哪个函数，而不是一个统一 transaction result。

应收敛为 `ProductCompositionRequest -> ProductComposition`：result始终持有 resolved config、module report、plugin/native owners、Core 和 diagnostics；调用方不能通过较短 helper意外丢失 owner或证据。便捷 API只构造 request，不再复制执行逻辑。

2026-08-27 的 M0 源码切片已完成该硬切：`ProductCompositionRequest`统一解析配置、选择first-party或显式linked provider、在配置准入后加载native export report，并通过单一private prepare stage生成report-only或完整composition；`ProductComposition`按安全drop顺序持有resolved config、module receipt/identity、Core、bridge lifecycle state、compiled plugin plan、diagnostics与可选native host。旧`EntryRuntimeBootstrap`、`NativePluginRuntimeBootstrap`、`bootstrap_with_*`及`into_core`/`into_parts`逃生口已删除，`EngineEntry`、`BuiltinEngineEntry`、`ProductComposition::core`均降为crate内部边界。Editor GUI、retained-host automation、headless和export facade均保留完整composition owner；生成的mobile/browser library以进程级`Vacant/Starting/Running/Stopping`状态机持有composition，bootstrap和析构均在mutex外执行，reentrant start/shutdown不死锁也不重叠generation，析构panic则保持`Stopping`并拒绝重新启动。ABI事件只在`Running`接受；Android `onDestroy`、iOS `applicationWillTerminate`和非BFCache browser `pagehide`均调用显式shutdown。native diagnostics保留在result中，不再由底层composition直接`eprintln!`。早期composition独立复审达到Critical/Important/Minor `0/0/0`；扩展到shutdown/Editor/FFI后的新一轮复核发现`0/2/1`，三项修复后的同一评审者重检为`0/0/0`。本段仍未宣称Cargo、产品行为或性能验收通过。

#### P1-3：Editor GUI 与 authoring automation 使用不同 runtime 部署路径

GUI 调用 `LoadedRuntime::load_default()` 加载外部 DLL，automation `EditorApplicationComposition` 固定 `LoadedRuntime::linked()` 并调用 linked session constructor。两条路径对 library discovery、ABI table、plugin registration、crash/unload和部署缺失的风险不同；一个 authoring integration test通过不能证明 packaged Editor DLL gateway可用。

应显式定义 `RuntimeDeploymentMode::{DynamicProduct,LinkedTest,Embedded}`，由同一 composition contract执行，测试矩阵同时覆盖 dynamic product和linked fast fixture。linked路径不能成为默认产品验收替身。

#### P1-4：Editor project preparation、Core composition与DLL session仍重复解释 plugin manifest

Editor先从 prepared project生成 runtime registration/capabilities，再用同一 registrations bootstrap Core；随后创建 projectless DLL session和gateway。这里已有避免“双开project”的正确注释，但 manifest、capability和module report仍在 app、runtime DLL、Editor gateway多次投影，没有一个带generation/hash的 composition receipt。

需要 `ProjectRuntimeCompositionReceipt`：包含 manifest identity、resolved plugin/features、module graph、capability set、render profile和runtime build identity。Core、DLL和Editor只接受同一 receipt或验证等价 hash，避免两侧选择逻辑漂移。

#### P1-5：module/plugin selection warning绕过结构化诊断

多条 selection路径直接 `eprintln!` plugin diagnostics，profiling export也直接写stderr；其他启动阶段则使用 typed diagnostic string和process log。严重级别、component、project/plugin identity和event correlation无法统一查询。

需要 typed startup event schema和单一 sink，在日志尚未初始化时使用 bootstrap ring buffer，初始化后 replay；stdout只保留明确机器协议，stderr只作最后降级。

### 6.2 Dynamic library、ABI 与 foreign owner

#### P1-6：V6 table 同时实现 size-aware tail读取，又要求 size精确等于本地 struct

loader先验证 required prefix并提供 field availability helper，却随后要求 `size_bytes == size_of::<ZrRuntimeApiV6>()`。这使同版本 append-only tail扩展完全不可用，也无法表达 compatible minor capability；所有变化只能发布新顶层 symbol/version。

若 V6 明确是 frozen exact layout，应删除假性的 tail compatibility并将 optional capability完全由slot presence表达；若目标是 prefix-compatible ABI，则接受 `size >= required_prefix`、忽略未知 tail，并在 ABI test中覆盖 older/newer table。两种策略必须择一并写进兼容政策。

#### P1-7：shipping runtime library discovery没有信任、签名或build identity政策

`ZIRCON_RUNTIME_LIBRARY` 可指向任意 absolute 或 product-relative library；loader验证symbol、ABI version和slot，不验证发行签名、engine build ID、target triple、feature/capability manifest、debug/release runtime或允许目录。开发 override合理，但 packaged Editor/player会暴露本地代码注入和错版本加载面。

应按 build channel区分 dev override与shipping policy，加载前验证 resolved path、manifest/hash/signature、engine build ID、ABI range和target；诊断保留请求与实际物理identity。不能用函数表版本代替binary provenance。

#### P1-8：host API 为空，跨DLL能力和服务协商不完整

`LoadedRuntime::{linked,load}` 都传 `ZrHostApiV1::empty(...)`；wake sink后来放进 session config。DLL无法通过 host table获取版本化 allocator、logging/crash context、clock/task、file/asset policy、telemetry或host capability，导致这些能力要么各自新增session字段，要么由DLL重复拥有进程全局服务。

应定义最小而稳定的 host services table和capability negotiation；高频服务使用函数表/handle而非JSON，所有callback带 owner token、threading/reentrancy和shutdown lease规则。没有需要的服务也必须显式声明 capability，而不是空表默认为兼容。

#### P1-9：Editor gateway重建一张部分 API table，存在双份ABI真值

`editor_gateway_api_table()` 从 `ZrRuntimeApiV6::empty()` 开始复制若干required/optional函数，而不是把validated table视图和size/capability直接交给gateway。新增slot时 loader、copy函数和Editor gateway三处必须同步，遗漏会把DLL已有能力静默变成None。

应让 gateway消费validated `RuntimeApiView`，其中table pointer、size、version、capability bitset和library lease为单一owner；禁止手工重建结构体快照。

#### P1-10：surface lifecycle用一个 `AtomicBool` 表示所有viewport，Drop又硬编码viewport 1

session只记录“有某个surface bound”，不记录handle、window/surface generation或多个binding。Editor gateway可以持有同一 lifecycle bool，session Drop却固定调用 `unbind_viewport_surface(ZrRuntimeViewportHandle::new(1))`。一旦多viewport、替换handle或Editor绑定非默认viewport，teardown会解绑错误资源，随后destroy可能失败并abort。

需要 session-owned `ViewportSurfaceRegistry`，以viewport handle映射surface generation/state，bind/rebind/unbind幂等且可枚举；destroy前反向解绑全部binding并报告每项结果。

#### P1-11：session缺少显式 quiesce/drain，destroy失败只能直接abort

现有API从运行态直接调用 `destroy_session`，没有 stop accepting operations、cancel/harvest operations、unsubscribe watches/events、stop worker、drain callbacks和deadline阶段。abort避免UAF，却不给产品保存crash envelope、停止子进程或完成前置flush的机会。

动态 API应增加 versioned `request_stop/quiesce/poll_shutdown/destroy` 或等价合同；host coordinator在仍持有library/callback storage时有界等待。deadline后写durable emergency record再abort。测试必须注入卡住worker、晚到wake、foreign free失败和destroy失败。

#### P1-12：foreign output预算不一致，host request/profile/operation/world可返回超大buffer

plugin event page有数量和字节上限，但 `drain_host_requests` 与 `profile_control` 在JSON parse前只检查到 `isize::MAX`；operation/world query沿用通用owned buffer，也没有host级总预算。可信DLL的逻辑bug或错版本可以让host分配/解析极大payload，阻塞UI线程或OOM。

所有ABI输出必须按operation定义 encoded-byte、item-count、nesting/decode-time预算；超限先释放foreign storage，再返回typed protocol violation并熔断session。预算进入telemetry和capability contract。

#### P1-13：控制面大量使用整包JSON，缺少schema hash、分页和零拷贝边界

host request、profile、plugin event、operation和world控制面反复 `serde_json` whole-buffer encode/decode。JSON适合低频可诊断命令，但当前没有统一schema ID/hash、unknown-field policy、page cursor、compression或large payload transfer handle；高频路径容易产生CPU、allocation和复制成本。

应按频率分层：固定高频event/frame demand/surface保持C ABI POD；低频命令使用versioned envelope与bounded page；大资源返回shared blob/stream handle。任何性能结论必须报告serialization CPU和bytes/frame。

### 6.3 Window、surface、input 与 frame owner

#### P1-14：runtime host是硬编码单window/单viewport模型

`RuntimeEntryApp` 只有一个 `Option<Window>`、presenter、viewport size、pointer position和 `ZrRuntimeViewportHandle::new(1)`；ApplicationHandler把传入的 `WindowId` 和 `DeviceId` 丢弃。所有event都投到viewport 1，无法区分第二窗口、popup、local multiplayer window、tool window、XR mirror或多display。

需要 `WindowRegistry<WindowId, HostWindow>` 与 `ViewportRegistry`，明确primary/secondary/embedded/offscreen关系、focus/input ownership、surface generation和close policy。未知window event要诊断而不是误投默认viewport。

#### P1-15：外部 `WindowEvent::Destroyed` 只发runtime event，不清理host window/surface

close-requested路径会unbind、drop presenter和window；Destroyed handler只发送 `window_destroyed`，仍保留 `self.window`。若OS或平台在非本地close路径销毁window，后续pump仍可能request_redraw或host request访问失效对象，Drop才尝试最终解绑。

Destroyed必须以WindowId查registry，先标记不可提交，再停止input/host requests、解绑对应surface、释放presenter/window，并按policy决定退出或重建。与close request要共享幂等transition。

#### P1-16：ApplicationHandler没有 suspended、exiting、memory/lifecycle处理

实现只有 resumed、can_create_surfaces、proxy wake、window/device event和about_to_wait。移动/桌面surface suspend、app background、memory warning、event-loop exiting均走默认no-op。相比Bevy/Fyrox显式lifecycle，Zircon无法在surface失效时释放swapchain/context，也无法在event loop仍有效时有序清window。

需要将platform lifecycle翻译成runtime event和host resource transition，区分 app inactive、suspended、surface unavailable、memory pressure与terminal exiting；Android/Web必须有真实平台fixture。

#### P1-17：resize直接重新bind当前surface，没有显式replace generation或恢复策略

已启用native present时，resize调用 `bind_current_window_surface()`，未先unbind，也没有old/new surface generation transaction。bind返回false或present一次失败都会fatal exit；没有 Lost/Outdated/Timeout/OOM分类、reconfigure、adapter/device recovery或降级到CPU fallback。

surface contract必须定义replace语义和generation：旧surface停止提交、等待in-flight fence、建立新surface、原子publish；Lost/Outdated可恢复，OOM/device-loss按policy升级。fallback是显式degraded mode，不应只在首次bind unavailable时生效。

#### P1-18：CPU fallback每帧完整readback、逐像素RGBA转XRGB，且没有颜色/alpha合同

fallback调用DLL capture整帧RGBA，在host逐像素组合u32并由softbuffer present；alpha被丢弃，transfer function、premultiplication、HDR和row pitch均未表达。1080p至少跨DLL复制约7.91 MiB/frame，4K约31.64 MiB/frame，另加CPU转换与surface copy，不能作为性能等价路径。

应把它标为诊断/兼容fallback，记录readback/copy bandwidth与原因；正式平台fallback优先使用共享texture或平台blit。frame ABI需携带format、alpha、color space、row pitch和generation。

#### P1-19：“低功耗”失焦游戏仍约60 Hz，headless cadence固定62.5 Hz

`UNFOCUSED_GAME_FRAME_INTERVAL` 等于16.67 ms interactive interval；只有occluded降到1秒。headless固定16 ms，不能由server tick rate、runtime demand、simulation fixed step或负载调节。常量也没有进入resolved product config。

应区分 render cadence、simulation fixed step、network tick、background maintenance和runtime deadline。失焦策略由用户/平台/音频/network需求决定；server用deadline-aware scheduler和overrun/backpressure统计，不能套窗口帧率。

#### P1-20：host request是fire-and-forget，失败只warning或静默丢弃

IME/cursor请求在没有window时直接return；错误只写warning；rumble也没有request ID/ack。runtime不知道请求是成功、暂不可用、viewport错误还是永久拒绝，可能继续保持错误cursor/IME/rumble状态。整批request也没有host侧count/time budget。

协议需要 request identity、target window/viewport generation、completion/result和retry policy；每pump有count/bytes/time预算与continuation。窗口缺失和platform unsupported必须返回typed outcome。

#### P1-21：first-frame capture在native present后再次capture整帧，证据与呈现不是同一提交

native present成功后，first-frame evidence另调 `capture_frame()`；若runtime已经推进resource/history或capture走不同readback，PNG不一定对应刚呈现的swapchain frame，同时产生一次完整GPU readback。该路径虽是opt-in diagnostics，仍不能作为严格视觉证据。

renderer应为presented frame发布frame token/fence；capture绑定同一token并异步readback，sidecar记录surface/color/build/session identity。超时不阻塞正常interactive loop。

#### P1-22：callback failure state只保留首个错误，跨阶段secondary failure丢失

首轮审查时，`RuntimeEntryAppFailureState` 和 session teardown state都只写第一个failure。当前runtime callback/event-loop/session/reporter以及Editor host/session已经共享bounded ordered ledger，保留primary、secondary和suppressed count；顶层log flush仍在binary外层处理，尚未进入该ledger。复杂停机需要primary cause，也需要完整secondary cleanup evidence。

统一 failure ledger 应保留bounded ordered chain、component/phase/severity/time和suppressed count；exit code由policy选择primary，不等于丢弃其余错误。

### 6.4 Process diagnostics 与验证策略

#### P1-23：大量启动错误发生在process log初始化之前

runtime在CLI、unknown args、project resolve、capture/exit env和Play starting report之后才初始化log；Editor也在完整launch parse后初始化。最容易出错的参数、路径和启动协议阶段只到返回值/stderr，无法进入统一log或crash envelope。

需要极早 bootstrap logger/ring buffer：进程开始即记录build/argv摘要和phase，正式sink可用后replay；敏感路径按typed field脱敏，不靠字符串替换。

#### P1-24：顶层忽略日志shutdown结果，仍返回业务成功exit code（历史发现，局部修复已落地）

首轮审查时两个binary都 `let _ = shutdown_process_log(...)`。当前 Editor/runtime 已检查该结果，并在业务成功但日志flush失败时通过统一`ProductProcessExitCode`返回failure；runtime与Editor其他失败也已共享typed ledger。log flush写入同一ledger与durable terminal receipt仍未完成。

shutdown结果必须进入failure ledger和exit policy；至少在成功业务但日志durability失败时返回可区分非零码或写OS emergency sink。teardown-complete记录只能在durable flush确认后发布。

#### P1-25：396个test attributes中有180次 `include_str!`，大量测试只锁源码文本

source guards能防止约定代码被误删，但不能证明runner、surface、DLL和shutdown行为。当前甚至有crate-level integration test读取两个binary源码，断言panic flush文本位于runner前、log shutdown文本位于后；它不会启动进程，也不会证明shutdown成功。

应把source guards降为少量architecture lint，核心置信度来自black-box binary、fake DLL、platform event-loop adapter、state-machine model和fault injection。测试数量不得作为产品完成证据。

#### P1-26：crate级只有4个测试，缺少产品生命周期矩阵

三个integration test文件中，只有Editor authoring restart是真实跨composition行为；没有启动 `zircon_runtime` binary、动态加载staged DLL、首帧/Play handshake、server loop、signal、window destruction、surface loss、foreign oversized buffer或shutdown timeout测试。

需要按product role建立packaged smoke/integration suite，并在CI运行实际artifact而不只是lib test。linked runtime fixture与dynamic DLL fixture都要有，且故障版本可控制每个ABI slot行为。

#### P1-27：没有可比较的host性能基线与预算门禁

现有cadence counters和fallback byte counter是起点，但没有startup phase time、frame pump CPU、event latency、serialization bytes、DLL crossings、context switch、window/surface rebuild、shutdown time、memory high-water和idle power基线。也没有与Unreal/Fyrox/Bevy同场景、同平台、同构建模式比较。

必须先定义 workload和measurement protocol，再谈“优于Unreal”：冷/热启动、空闲Editor、focused/unfocused client、headless 30/60/120 tick、1/4/16窗口、surface loss、Play spawn/stop、正常/故障shutdown分别给P50/P95/P99与硬预算。

## 7. P2：清理低风险但会持续制造漂移的实现

### P2-1：产品行为由多组未归档环境变量切换

first-frame exit/capture、force capture present、input probe、runtime DLL和profiling等env散落在runner/app中。应统一进入typed startup config，记录source和最终值；测试override与shipping option分开。

### P2-2：60秒frame demand cap、16ms headless等常量没有policy provenance

这些值应是named policy defaults并进入diagnostics/config receipt，而不是host内部不可见常量。超长runtime deadline被静默clamp也应计数。

### P2-3：`PluginGroupBuilder::finish()` 在公开API中用 `expect` 放大配置错误

已经有 `try_finish()` typed error，公开`finish()`不应在产品配置上panic；可限为测试helper或移除，所有composition走typed path。

### P2-4：binary和诊断仍使用 `runtime_preview` 命名，产品定位含混

Cargo binary名为 `zircon_runtime`，源码和许多错误却称runtime preview；应区分 packaged player、Editor Play child、developer preview，避免默认安全/性能/资产政策混用。

### P2-5：profiling export failure只打印stderr，不参与terminal report

若用户明确请求capture，导出失败应进入typed terminal outcome；未请求时才可作为warning。capture stop也应参加shutdown deadline。

### P2-6：library default search只尝试sibling和`deps`，没有发行manifest

开发布局够用，但发行包应由manifest列出runtime binary、hash、ABI和platform，而不是猜两个路径。找不到时诊断应列出候选与manifest identity。

### P2-7：help、diagnostic和CLI契约由手写长字符串维护

parser、help和docs容易漂移。应从typed option schema生成help/diagnostic field，但不能因此引入运行期反射开销。

### P2-8：测试中的大量源码顺序断言阻碍正常模块化

180次`include_str!`会把文件拆分、重命名和局部重构变成无行为变化的测试失败。应把必须保持的约束提升为API/state-machine test或专用static validator，只保留极少安全边界guard。

## 8. 参考实现差异矩阵

| 工程能力 | Unreal | Bevy | Fyrox | Godot | Zircon 当前 |
|---|---|---|---|---|---|
| 产品runner | GuardedMain/FEngineLoop，多角色编译 | 可替换runner + AppExit | normal/headless独立 | setup/start/iteration | Editor与desktop preview分支，server无runner |
| cleanup保证 | cleanup guard +分阶段Exit | finish/cleanup + exiting | loop owner/graphics lifecycle |显式cleanup顺序 | session局部Drop，进程无coordinator |
| headless | server/program路径 | ScheduleRunner等 | 独立headless loop | display/server配置 | client Winit隐藏窗口或立即返回 |
| 多窗口event | platform/slate window owner | WindowId映射Entity | event携带window | DisplayServer window id | WindowId被丢弃，viewport固定1 |
| suspend/surface | platform/RHI lifecycle | suspended/exiting | graphics context destroy/recreate | OS/display阶段 | handler默认no-op，失败多为进程退出 |
| 动态边界 | module/build/version政策 | Rust静态composition为主 | Rust静态composition为主 | GDExtension lifecycle | DLL owner基础好，协商/预算/信任不足 |
| Play/child握手 | Session/Editor多层管理 | 非主要参考面 | 非主要参考面 | remote/debug protocol | stdout标签，consumer不解析 |
| 停机证据 | 广泛trace/log/delegate | AppExit | loop返回 | benchmark/cleanup | 首错 + 若干字符串，log flush结果丢弃 |

Zircon 不需要照搬 Unreal 的全局单例与宏体系，但必须达到同级别的阶段所有权、反向依赖、failure containment和产品角色真实性。Rust RAII只能帮助表达owner，不能替代跨线程、跨DLL、跨进程的quiesce协议。

## 9. 目标架构

### 9.1 `ProductRole` 与artifact manifest

定义不可混用的角色：`EditorHost`、`DesktopClient`、`Server`、`WebClient`、`AndroidClient`、`EditorPlayChild`、`Commandlet`、`Embedded`。每个角色声明entry kind、linkage mode、platform/input/window/render能力、required modules、shutdown policy和artifact manifest。Cargo feature只负责构建能力，不能直接充当运行期profile真值。

### 9.2 `ProductComposition`

唯一composition transaction接收role/project/export/plugin requests，输出immutable receipt与owners：resolved config、Core、runtime deployment、native plugin hosts、module/plugin/capability report、build/ABI identity。startup任何阶段失败都通过同一rollback stack反向释放已创建owner。

### 9.3 `ProductHost`

host拥有event/schedule runner、window/viewport/surface registry、runtime session、host request executor和failure ledger。desktop/mobile/web可以有不同adapter，但都编译到同一host state transition；server使用无窗口scheduler，不依赖Winit。

### 9.4 `ShutdownCoordinator`

最小阶段：

1. `RequestStop`：冻结一次terminal reason/exit code，拒绝新窗口、operation和Play request。
2. `Quiesce`：停止simulation/network/plugin生产新任务，取消可取消工作。
3. `Drain`：有界等待任务、asset/save、GPU、telemetry和child process。
4. `ReleasePresentation`：逐generation解绑surface、清window/input/rumble。
5. `DestroyRuntime`：unsubscribe/watch/operation -> session quiesce -> destroy -> DLL unload。
6. `DeactivateCore`：按反向模块依赖cleanup，保留全部secondary failure。
7. `FlushEvidence`：profiling/log/crash envelope durable flush。
8. `Exit`：发布terminal receipt和稳定process code。

每阶段幂等并能在startup rollback复用；deadline升级到emergency abort时仍尽可能写最小durable record。

#### 2026-08-27 M0 shutdown owner-chain重审与实现顺序

当前源码已比首轮审查前进：`zircon_editor`与`runtime_preview` binary都会检查`shutdown_process_log`结果，并在业务成功但log flush失败时返回failure；runtime event callback、event-loop、session teardown、terminal reporter以及Editor host/session teardown已汇入同一bounded ordered ledger，旧的首错兼容槽和三`Option`字符串聚合已经删除。`ProductProcessExitCode`把普通host成功/失败稳定映射为portable `0/1`，同时保留command显式返回的非零`u8`。因此P1-24的“结果完全丢弃”和“没有统一ledger/exit policy”都只保留为历史发现。但这还不是process-wide shutdown：已定义的phase coordinator尚未持有window/surface/session/Core/module/log实际资源图，顶层log flush失败仍在binary边界直接覆盖exit code而未写入同一ledger。`RuntimeSession::Drop`的destroy失败abort继续作为防UAF的emergency底线，不能被普通退出路径复用为coordinator。

Unreal-first重审再次确认目标不是增加一个`on_exit`回调。`FEngineLoop::Exit`先停止ticker与异步生产，flush并禁止新async load，关闭窗口/input，等待streaming/PSO/GPU/task，然后反向`UnloadModulesAtShutdown`并退出RHI/task graph；`AppPreExit`只用于受保护的正常退出，而`AppExit`用`bCalledOnce`保证所有退出路径上的platform/config/log/trace teardown幂等。Bevy的`AppExit`提供typed portable process code，`finish/cleanup`主要是启动期plugin完成阶段；Fyrox的window close直接退出event loop并依赖owner析构。后两者可用于adapter和typed-exit对照，但不能降低Unreal式显式反向资源图的目标。

实现顺序据此冻结为：

1. 独立小owner定义typed terminal reason、portable exit policy、shutdown phase与bounded ordered failure ledger；ledger只在failure/shutdown冷路径加锁，固定容量和单条message预算，保留primary、secondary与suppressed count。
2. 建立幂等、单调且可审计的`ShutdownCoordinator` phase transition；先用fake phase owner验证正常、重复调用、startup rollback、secondary failure与deadline升级，不直接绑Winit或DLL。
3. 让runtime event loop、Editor host、surface/session、Core composition与diagnostic log向同一ledger/phase owner汇报，再删除现有首错state和`finish_*`字符串拼接器；迁移必须同批完成，不保留双ledger。
4. 动态runtime的`request_stop/quiesce/poll_shutdown/destroy`仍属于ABI/M2 owner；在ABI slot落地前，App coordinator只能把现有destroy标为legacy emergency branch，不能伪造quiesce已完成。

性能与功耗协议先于优化实现：每个phase记录monotonic elapsed、deadline/overrun、drained work与suppressed failure数量；成功frame/event路径不得触碰ledger lock。后续以正常/故障shutdown、空闲Editor和headless fixture采集P50/P95/P99、CPU time、context switch、memory high-water与idle power，并与固定硬件/构建/负载的Unreal经验值比较。本轮没有运行这些测量，不声明瓶颈已消失或功耗已接近参考引擎。

2026-08-27 的首个基础设施切片已实现独立 `product_shutdown` owner：`ProductHostPhase`覆盖composing/running以及quiesce、drain、platform release、runtime destroy、module deactivate、diagnostic flush、exited；`ProductTerminalReason`映射semantic exit class，公开`ProductProcessExitCode`按Bevy/通用OS进程约束把host成功/失败收敛为`0/1`，并保留command显式非零`u8`结果，不为startup/runtime/shutdown故障虚构不可移植的多个数值码；`ProductShutdownCoordinator`保留首个terminal reason，拒绝跳阶段/逆序并记录每阶段与总耗时。`ProductFailureLedger`固定保留16条、每条message最多512 UTF-8 bytes，后续输入只增加saturating suppressed count；容量与message预算是App账本模块内的具名策略常量，不提升为跨crate协议。

runtime与Editor failure path已接入同一ledger：原 `RuntimeEntryAppFailureState` 的每次 `is_recorded()` 都锁 `Mutex<Option<_>>`，现在成功路径只做 `AtomicBool::load(Acquire)`；只有callback/event-loop/session/host/reporter teardown失败才锁共享ledger。callback先完成ledger写入再以Release发布recorded flag，Acquire观察者不会看到“已发布但记录尚不存在”的状态；terminal reporter失败在最终snapshot前追加，已有runtime failure保持primary。Editor GUI、automation和close路径都在释放composition/Core owner后再释放runtime session，并在两者之后snapshot；只用于startup验证的`EditorManager`强引用已提前释放。旧session首错兼容槽、三`Option`聚合和临时`Vec`/join已删除。顶层diagnostic-log flush仍未进入该ledger，实际phase coordinator也尚未接管资源释放。此处只有源码复杂度与锁位置证据，没有运行profile、benchmark或功耗测量，不声明实际耗时下降。

#### 2026-08-27 actual-owner wiring gate

本轮继续核对真实调用图后冻结以下约束。Winit 0.31路径的`event_loop.run_app(app)`消费`RuntimeEntryApp`；app内部持有window、presenter与`RuntimeSession`，其`Drop`先尝试surface/window teardown，随后字段析构才执行session destroy；runner只有在整个app已释放后才恢复执行，而两个binary又在runner返回后执行process-log shutdown。因此在`run_app`返回点补写`ReleasingPlatform`或`DestroyingRuntime`会伪造发生时间，不能作为phase接入。

Unreal `GuardedMain`的cleanup guard保证`EngineExit`必达，`FEngineLoop::Exit`在模块卸载前停止ticker/异步生产、flush loading/streaming与任务并关闭输入/窗口，`AppExit`再以called-once门禁完成platform teardown。Zircon不复制其全局状态，但必须保留同样的owner跨度。下一实现切片必须先建立跨越event loop与log flush的process owner/terminal receipt：共享coordinator在app adapter的真实surface/window释放点记`ReleasingPlatform`，在session owner的真实destroy边界记`DestroyingRuntime`，runner恢复后才进入profiling/report/log flush并发布`Exited`。startup rollback也必须由同一owner guard记录已经构造和释放的资源，不能在错误返回后补齐阶段。

当前冻结V7只有合并式`destroy_session`，没有独立`request_stop/quiesce/poll_shutdown`，因此不能把runtime内部drain和module deactivation伪装成App可观测的独立成功阶段。本切片已给transition receipt增加明确的`Executed`、`NoOwner`与`LegacyCombined` disposition，并保持原有单调/idempotent gate；M0接线可据实记录无owner阶段与legacy combined destroy，M2新增ABI后再把它硬切为可超时的quiesce/drain/destroy。该改动是停机冷路径架构，不触碰frame/event成功热路径；本轮没有性能优化或功耗结论。

### 9.5 Typed child-process protocol

Editor Play child使用versioned framed protocol和随机session token。stdout/stderr继续是human diagnostics，不承担控制面。Editor在world-ready或first-frame-ready前不进入Playing；stop发送cancel并等ack，deadline后终止process tree；EOF、crash和protocol violation映射为不同terminal result。

## 10. 分阶段重构计划

### M0：冻结产品角色、生命周期和ABI政策

状态（2026-08-27）：`shutdown_disposition_editor_exit_abi_source_implemented_static_review_passed_managed_validation_pending`。

- 已完成：产品角色词表；`EntryConfig -> ResolvedProductHostConfig`单次解析；逐字段及manifest多来源provenance；profile-base + request/export plugin overlay；required/optional typed conflict与required precedence；role/target/runtime profile/export冲突的typed failure；Editor复用同一解析结果；native export先解析后加载；Export runtime profile进入module selection receipt；导出配置删除重复profile/target authority并按export target投影product role；8角色`ProductRoleDescriptor` target-rule目录；entry/runner/linkage/platform/window/input/render/shutdown/artifact manifest矩阵；role级window/render typed admission；resolved export platform与诊断投影；对应单元测试源码与跨crate source guard已更新。App不再声明一份未接入composition的静态required-module baseline，实际模块集合只由runtime profile、`RuntimeModuleCompositionCompiler`与composition receipt拥有。`ProductCompositionRequest -> ProductComposition`现为唯一公开执行transaction，report-only与full compose共享prepare stage，Core/module receipt/compiled plan/bridge lifecycle/native owner不再由调用方手工拼接生命周期；Editor、headless与export入口均已迁移，生成export模板保留完整composition。portable进程退出策略已冻结为host `0/1`加command显式`u8`；dynamic runtime ABI已确认只接受exact/frozen V7，短表、超长同版本表和旧symbol均不兼容。
- 静态证据：配置域现为13个具名实现文件加1个导航`mod.rs`，最大实现文件`resolution.rs` 386行；目标Rust叶文件与`skip_children`导航/root文件的Rust 1.94 `rustfmt --check`通过，作用域`git diff --check`通过；旧`entry_config.rs`生产/测试引用已清零；8/8角色均有descriptor与唯一artifact target；delivery分布为runnable 1、preview 1、configuration-only 1、unavailable 5；当前Cargo只声明`zircon_editor`与`zircon_runtime`两个对应可运行target，`zircon_server`不存在，未把`target-server` feature伪装为artifact；导出authority、profile overlay、required precedence、editor provenance、resolved platform与role capability的有界源码断言通过；Minimal无窗口契约、Windows/Linux Server OS保留与App required-module owner清零均有测试源码；App静态依赖审计为0个optional runtime plugin path dependency、0个对应feature mention、`risks=[]`。增量独立复核初次发现3个Important与1个Minor，修复及diagnostics期望校正后二次复核Critical/Important/Minor均为0；按同一审计器过滤本次两个文档为0个路径违规。全局文档路径基线仍有1363个其他文档违规，全量`validate-matrix.Tests.ps1`静态矩阵此前在127秒上限内超时且无判定结果，均不记作本切片通过。
- ProductComposition静态证据：implementation按职责拆成185行`composition.rs`、188行`request.rs`与纯导航`mod.rs`，没有新增大root；Rust 1.94叶文件及`skip_children` root/mod格式检查通过，作用域`git diff --check`通过；Rust源码中旧`EntryRuntimeBootstrap`、`NativePluginRuntimeBootstrap`、`bootstrap_export_runtime_with_report`、公开`bootstrap_with_*`和`clone_core`生产入口均为0，唯一命中是新source guard的禁止断言；两个native registration report向量均move-extend且无clone；配置解析文本顺序先于native root load；Editor显式和默认字段drop都先释放product composition再释放dynamic runtime session，startup验证用manager引用在runtime load前释放；mobile/browser生成库使用四态owner，所有13个C/JNI出口由精确函数清单逐体约束进入统一`catch_unwind`门禁。第一轮独立评审发现1个Critical与3个Important，修复后同一评审者复审Critical/Important/Minor为`0/0/0`；本轮扩大范围后的`0/2/1`修复后重检也为`0/0/0`。5份本轮owner文档的canonical path audit为0个违规，同一时点全库其余违规为1361且不归入本切片。Cargo/行为验收未执行。
- Shutdown ledger源码证据：实现按职责拆为230行`coordinator.rs`、134行`failure.rs`、112行`failure_ledger.rs`、103行`terminal.rs`、45行`phase.rs`与20行导航`mod.rs`；测试源码236行。生产Rust中的裸`16`/`512`只各出现于账本具名策略常量定义；runtime callback成功门禁为atomic read，账本mutex只出现在failure record/snapshot；旧`finish_runtime_process`的`Vec::with_capacity(3)`、三个terminal `Option`聚合、callback failure `Mutex<Option<_>>`和session首错兼容槽均已清零。ledger message在512-byte预算内执行UTF-8安全转义/截断；runtime reporter secondary failure在snapshot前记录；Editor host/session共享ledger；binary使用统一`ProductProcessExitCode`。transition receipt现记录`Executed`、`NoOwner`或`LegacyCombined`，startup rollback可把没有running owner的quiescing记为`NoOwner`，幂等重复调用不覆盖首次disposition；测试约束不把无owner和V7合并destroy标成独立执行成功。Rust 1.94格式与作用域diff检查通过；独立复核的ledger publish-order、reporter-order、message-bound与Editor manager寿命问题均已关闭，扩展范围及disposition追加重检均为Critical/Important/Minor `0/0/0`。Cargo/行为验收尚未执行。
- 结构裁决（Unreal-first）：重新核对`TargetRules.cs`的`TargetType`/`TargetLinkType`、`TargetDescriptor.cs`的requested target、`TargetReceipt.cs`的Launch/BuildProducts/RuntimeDependencies以及`LaunchEngineLoop.cpp`的PreInit/Init/Tick/Exit；并以Bevy runner/finish/cleanup和Fyrox独立headless loop作次级对照。Zircon据此把`ProductRoleDescriptor`限定为静态target rule，把实际文件/hash/ABI继续留给build receipt与现有Runtime BuildSet manifest，禁止用Cargo feature或静态descriptor冒充构建产物证据。Server保留Windows/Linux/macOS目标OS，由`ServerRuntime`和window/render policy表达headless topology；`Minimal`保留为DesktopClient下合法的无窗口utility configuration。二者都禁止把target type、target platform和runtime backend重新压成一个枚举。
- 性能评估：本切片不是热点算法优化，没有运行profile、功耗或同负载benchmark，也不作性能提升声明。角色解析新增内容是8分支静态descriptor查表和启动期typed检查；composition新增分配、native manifest I/O与diagnostic收集均只发生在启动prepare阶段，registration report通过move合并而非clone，frame/event/render loop零改动。后续若优化host loop，仍须先按本计划P1-27采集startup phase、frame pump CPU、event latency、idle power与shutdown时间基线。
- 待受管验证：`zircon_app` lib/all-target测试、export bootstrap行为、feature matrix和workspace回归；本轮按里程碑策略未执行Cargo。
- M0剩余：让已定义的shutdown phase真实接管runtime/Editor资源反向释放，把顶层log failure写入同一ledger，以及执行本轮composition/ledger/exit policy的受管Cargo与行为验收；因此M0未完成且不得提交里程碑commit/企微完成通知。

- role/artifact/capability矩阵已写入源码；受管Cargo与feature-matrix验收仍待测试阶段。
- composition receipt、host phase、terminal reason、portable exit code和bounded failure ledger源码已定义；继续把runtime/Editor实际资源owner接入coordinator。
- dynamic runtime已冻结为V7 exact-size、不降级且版本内不追加字段；后续新字段必须新table版本并hard cutover。dynamic library trust/build identity仍待定义。
- 禁止新增新的`bootstrap_with_*`和裸environment product switch。

### M1：交付server runner与统一shutdown coordinator

- 增加真实`zircon_server` artifact和无Winit runner。
- Core/module cleanup接入coordinator；startup rollback与正常退出共用阶段。
- 接入SIGINT/SIGTERM/Windows service/console stop、health/readiness和graceful deadline。
- binary处理log/profiling shutdown结果，发布durable terminal receipt。

### M2：收敛dynamic runtime owner与协议防御

- 引入validated API view、host services/capability handshake和build identity。
- 建立viewport surface registry、operation/watch/subscription registry和session quiesce。
- 为全部foreign output增加bytes/items/time预算与fault-injection DLL。
- 保留destroy failure emergency abort，但在abort前写最小crash evidence。

### M3：把desktop host升级为multiwindow/platform lifecycle owner

- WindowId/viewport/surface generation registry，删除固定handle和全局bool。
- 接入suspend/resume/exiting/memory pressure与surface Lost/Outdated/device-loss恢复。
- 拆分simulation/render/network/background cadence；把policy放入resolved config。
- CPU fallback明确标为degraded并补color/format/alpha合同。

### M4：完成Editor Play child握手

- 建立typed framed IPC、session token、protocol/build/snapshot identity和phase ack。
- Editor backend解析并驱动Starting/Ready/Playing/Stopping/Exited，而非只收日志。
- 加入heartbeat、start timeout、stop ack、EOF/crash/protocol violation和process tree cleanup。
- `ready`至少绑定world-ready；需要视觉验收时使用first-frame token。

### M5：产品artifact、故障和性能验收

- CI构建并启动desktop client、server、Editor、Play child；Web/Android在对应runner完成后进入gate。
- fault DLL覆盖短表/长表、missing slot、oversized buffer、late callback、free/destroy失败。
- platform fixture覆盖multiwindow、external destroyed、suspend/resume、surface/device loss。
- 建立startup/idle/frame/Play/shutdown与同负载reference benchmark，结果版本化保存。

## 11. 验收 Gate

1. `zircon_server` packaged artifact在无图形桌面环境运行多帧并通过三类terminal reason优雅退出。
2. 每个声明platform/role都有独立CI build artifact；不存在靠additive feature拼出互斥平台的配置。
3. 任意startup失败都产生composition receipt片段和完整rollback evidence，无活worker/window/session/plugin owner。
4. 正常退出按阶段执行且可重入；第二次stop只返回现有terminal result。
5. Core所有active module按反向依赖cleanup，cleanup failure不会伪装回Running。
6. DLL session quiesce后无晚到callback；destroy失败fixture在durable emergency record后abort。
7. foreign output超限不会JSON parse/OOM，foreign buffer始终释放，session进入明确protocol-failed状态。
8. 4个并发window/viewport事件按WindowId路由；关闭/销毁一个不影响其余surface/input。
9. suspend/resume重建surface generation且不提交旧surface；Lost/Outdated可恢复，device loss有typed terminal或重建结果。
10. Play child只有在校验token/build/snapshot并收到world-ready后进入Ready；错误、EOF、timeout和stop各有确定状态。
11. 顶层log/profiling flush失败进入exit code/failure ledger，不能仍声称teardown complete。
12. source guard占比显著下降；binary、server、dynamic DLL和fault-injection integration成为主要证据。
13. host CPU、serialization、idle power、startup/shutdown和fallback bandwidth有P50/P95/P99预算及回归阈值。
14. 与Unreal等比较时固定硬件、OS、build、窗口/帧率、场景和画质；原始trace与统计可复现。

## 12. 与相邻计划的责任边界

- Core module状态机、反向cleanup和服务撤销由 `zircon_runtime/01` 拥有；本计划负责让产品host实际调用并验证它。
- ABI struct、FFI安全、handle和versioning的全域审查进入 `zircon_runtime_interface`；本计划只拥有app loader/session使用方式。
- Editor retained host、PIE state和process backend UI由 `zircon_editor` 拥有；typed child protocol需要双方共同定稿，禁止app单边发字符串后宣布完成。
- native plugin discovery/hot reload/signature由 `zircon_plugins` 拥有；app composition必须持有其owner并参加shutdown。
- PBR viewer的工具生命周期、artifact evidence和RenderDoc bridge进入 `zircon_app/02`，不能替代产品host gate。

## 13. 本轮完成定义

首轮只完成静态review与重构计划；2026-08-27 已在重新读取owner chain、Unreal TargetRules/TargetReceipt/Launch、Bevy App runner和Fyrox Executor后落地P1-1角色/配置、P1-2统一composition、共享failure ledger、portable exit policy与V7 exact/frozen ABI裁决的M0源码切片。该批切片尚未经过受管Cargo与行为验收，phase coordinator也未接管真实资源图，因此只能记录为source implemented，不能把M0或整个计划标记为完成。

`zircon_app` 分类仍为进行中：产品配置权威已从可变`EntryConfig`派生状态收敛到一次性`ResolvedProductHostConfig`，公开bootstrap已收敛到统一composition owner，runtime/Editor冷路径也已共享failure ledger；但server/跨平台runner、真实资源shutdown coordinator、log-ledger接入、PBR viewer、完整export/package实物以及与 `zircon_runtime_interface`/plugins/editor/hub 的后续交界仍待完成。
