---
related_code:
  - zircon_runtime/src/plugin/native_plugin_loader
  - zircon_editor/src/core/play/controller.rs
  - zircon_editor/src/core/play/plugin_activation/native.rs
  - zircon_editor/src/ui/retained_host/app/module_plugin_actions/live_host/native_backend.rs
  - zircon_editor/src/ui/retained_host/app/module_plugin_actions/live_host/development_watch.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/02-unreal-aligned-engine-system-hard-cutover.md
  - docs/plans/zircon_runtime/runtime/06-plugin-surface-and-lifecycle.md
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
  - docs/plans/zircon_plugins/01-plugin-architecture-core.md
  - docs/plans/zircon_plugins/11-plugin-call-bridge.md
  - docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md
  - docs/plans/zircon_editor/editor/12-plugin-management.md
  - docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Private/Modules/ModuleManager.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Projects/Private/PluginManager.cpp
  - dev/UnrealEngine/Engine/Source/Developer/HotReload/Private/HotReload.cpp
tests:
  - native plugin loader scope 88 of 88 Rust files reviewed
  - 27846 lines and 284 test attributes inventoried
  - deterministic 88-file source manifest SHA256 e0b29812f0ca992ee05a022f3aa7d4a2cde24b47321e78dc364fa79367cfdce8
  - rustfmt edition 2021 check passed for 88 of 88 files
  - current-source Cargo product WPR allocator and power acceptance blocked
doc_type: implementation-evidence
status: static_complete_dynamic_blocked
---

# Native plugin discovery 与 live host 当前架构审查（2026-08-15）

## 范围与结论

本轮按当前工作树逐文件复读 `zircon_runtime/src/plugin/native_plugin_loader/**` 88/88 个 Rust 文件，共 27,846 行、284 个 test attribute；按路径分为 65 个生产/混合文件和 23 个 test-only 文件。将规范化相对路径与逐文件内容哈希按稳定顺序组合，得到 SHA256 `e0b29812f0ca992ee05a022f3aa7d4a2cde24b47321e78dc364fa79367cfdce8`；88/88 通过 `rustfmt --edition 2021 --check`。

当前源码已经修掉多项历史热路径问题：foreign callback 在 loaded registry 锁外执行；callback admission 使用原子 generation lease；host context 使用代际 page directory 与 `ArcSwap` 无锁读取；command name 在载入代际解析为 slot，V4 command 输出使用 host-owned bounded sink；bridge method 使用 dense slot directory。这些合同必须保留，不能因整体硬切退回全局 mutex、每调用分配或跨动态库 Rust 对象。

剩余主瓶颈不是上述单次调用，而是控制面所有权。`NativePluginLiveHost` 同时维护 loaded、bridge binding、bridge generation、registration replay generation、revision 五个平行 registry，加两把跨所有插件的 generation build lock；Editor 又在该后端之外维护 watcher 线程和同步 Play activation。目标不是给这些表继续加 cache，而是让 Plan02 的 `EngineRuntime` 唯一 plugin catalog/lifecycle generation 原子发布一个 native backend snapshot，Editor 只提交有 deadline 的 lifecycle ticket。

当前源码仍没有可运行的 current product binary；managed app build 与 focused runtime test 被其他 current-source 编译错误阻断。因此本文只有静态所有权、锁范围和复杂度证据，不是 WPR、allocator、wall-clock、功耗或吞吐实测，目录继续留在 `pending.md`。

## 应保留的当前合同

- discovery 已有进程级单一 authority、canonical root identity、每 root 一个 active 加一个 pending generation、latest-wins 合并、deadline/budget 和 last-good immutable snapshot。
- 单 manifest refresh/remove 已进入 `ManifestBatch` 与 immutable manifest index，不再把每个有效通知无条件退化成全 root scan；旧 PERF-MVP-035 的静态根因已消失。
- loaded registry 锁内只取得 callback snapshot/lease owner，command、save、restore、unload 等 foreign ABI 在锁外执行；动态库由 generation owner 延寿。
- callback admission 以 transition bit 加 active count 原子状态实现，诊断可关闭并分 64 个 cache-line aligned shard；旧 PERF-MVP-021/541 不再描述当前源码。
- host context 以 256-slot page、generation handle、两次 generation check 和 `Arc` pin 解析；稳定 lookup 不拿 writer mutex。bridge method 已是四级 sparse-dense slot directory，不走 String/BTreeMap method lookup。
- `NativePluginLoadReport` 使用 `OnceLock<NativePluginLoadProjection>`；同 report 的 manifest/diagnostic projection 只构建一次，mutation 会使其失效。

以上只说明历史局部问题已修复，不证明整体 native plugin architecture 已验收。

## P0：live host 仍是第二套插件生命周期权威

`native_plugin_live_host.rs:168-182` 包含五个 `NativePluginLiveRegistry<T>` owner：loaded entries、runtime bridge bindings、bridge generations、registration replay generations 和 revisions。每个 registry 内部又固定包含 runtime/editor/native/vm 四个 `BTreeMap<String, T>`，因此当前类型形状最多物化 20 个平行 map；此外 `runtime_bridge_generation_build_lock` 与 `runtime_registration_replay_generation_build_lock` 是跨全部 plugin id 的全局构建锁。

`registration_replay.rs:439-539` 的两个 cache miss 路径先取得全局 build lock，再按 revision/build/cache 循环。两个互不相关的插件不能并行构建 bridge 或 registration generation。更严重的是 `registration_replay.rs:600-664` 从 loaded lookup 开始一直持有全局 loaded mutex，期间复制 manifest source/capability/component rows、解析 TOML、进入另一把全局 bridge build lock并构造全部 prepared systems；任何插件的 descriptor、callback snapshot、load/unload 或状态查询都会被一个插件的冷构建阻塞。

正确边界是 `PluginCatalogGeneration -> PluginSlot -> NativeBackendGeneration`。catalog mutation 在 TaskGraph 上执行 `stage -> validate -> compile bridge/replay plan -> initialize -> atomic publish -> retire`；每插件 slot 持自己的 transition ticket/immutable backend generation。不同 slot 的冷构建可有界并行，同 slot single-flight；loaded、bindings、replay 与 revision 不再作为五套可独立观察的 truth。

## P0：load batch 逐插件发布，失败与读者可观察半完成代际

`loading.rs:137-258` 先构造一次 load-report projection，但随后逐插件获取 loaded mutex、发布 bridge binding、插入 loaded row并使 replay generation 失效；替换路径还在每个插件之间释放锁执行旧 generation unload，再重新入锁发布。N-plugin batch 会有 O(N) 次 loaded publication 与 O(N) 次 replay invalidation，任意并发读者都可能在第 k 个插件后观察到部分新 batch。后续插件失败也没有一个 catalog-level transaction 撤销前面已发布的插件。

目标 batch 先在 publication lock 外完成所有 candidate、ABI、capability、bridge、registration 和 rollback plan；成功只交换一次 `Arc<PluginRuntimeGeneration>`，失败 publish count 为 0。foreign callback 继续在锁外，但其结果进入 staged transaction，而不是逐项改变公共 truth。

## P0：Editor 热重载和 Play 转换绕过统一 TaskGraph

`development_watch.rs:65-124` 为每个 watched plugin 创建一个 `sync_channel(1)` 和命名 OS thread，使用 350 ms debounce；worker 在私有线程直接调用同步 `hot_reload_editor_plugin`。`Drop:128-135` 无 deadline 地 `join`。`native_backend.rs:33-58` 又在 watch registry mutex 内创建 watcher/thread，并在 `retain` 删除旧 watch 时触发 Drop/join；因此 watched plugin 数 P 产生 O(P) 私有线程，UI unload/replace 可在 registry mutex 内等待 350 ms、文件系统或完整 hot reload。

`core/play/controller.rs:133-193,196-236,239-275` 在 controller transition gate 内同步调用 activation/deactivation；`plugin_activation/native.rs:38-130` 再持自己的 transition mutex，同步 discovery/load、保存全部 runtime plugin state并调用 enter/exit command。它没有 TaskGraph ticket、caller deadline 或 progress generation，产品调用者会在整段插件工作完成前被占住。discovery 的公开 refresh/remove API 同样在 `discover/authority.rs:188-231` 调用 `wait_terminal()`；目前非测试产品调用为 0，所以这是接线前 API 缺陷，不冒充已测 UI 热点。

Unreal 的对照不是“所有工作都上 worker”。`HotReload.cpp:1175-1224` 由一个 DirectoryWatcher 服务登记目录 callback；`1086-1145` 只把检测到的 module 放进共享 queue；`1226-1290` 在 ticker 上非阻塞检查并在满足 phase/PIE 条件后统一消费。`ModuleManager.cpp:980-1061,1316-1405` 仍由单一 manager 管理 ready/load/unload 和变更广播。Zircon 应继承共享 watcher、队列、phase 和唯一 lifecycle owner，同时用自身 TaskGraph affinity/deadline 保证主线程只执行必须的 publish/activation 段。

## P1：discovery/report 仍按 owned DTO 边界复制

`discover/authority.rs:348-355` 每次把 immutable discovery snapshot 的全部 candidates/diagnostics `to_vec` 成 owned public report；warm discover 不再做文件系统 I/O，但仍按 package/report 大小复制。`manifest_index.rs:83-101` 每次发布又遍历全部 candidates，为 duplicate 选择 clone candidate/plugin id/path。`NativePluginLoadProjection` 虽由 OnceLock 限制为每 report 一次，内部仍深 clone candidate/package/diagnostic rows。

这些是 control-plane payload cost，尚未证明压过生命周期全局锁。Plan02 不应再造 report cache；`PluginCatalogGeneration` 直接保留 interned/dense immutable rows，public diagnostics/export 边界按需物化 owned DTO。动态规模数据决定是否需要 paged/borrowed report API。

## Unreal 对齐后的唯一 owner

1. `EngineRuntime::PluginCatalogGeneration` 统一 package/module/profile/capability/extension/native discovery identity。
2. `CompiledPluginPlan` 统一 target/project selection、loading phase、bridge slot、registration access 与 owner ranges。
3. `PluginLifecycleCoordinator` 以 per-slot ticket 在共享 TaskGraph 上执行 load/quiesce/migrate/publish/retire；Editor 只提交 request 并观察 terminal/progress。
4. `NativeBackendGeneration` 仅保存动态库 owner、validated ABI tables、dense command/method slots、prepared registration plan和state migration handle，不保留第二 catalog/revision registry。
5. `PluginRuntimeGeneration` 以一个 `Arc` 原子发布完整 batch；callback 先 pin generation/lease，再锁外调用。失败 batch 保留 last-good。

这与 Unreal `PluginManager.cpp:2034-2080,2884-2978` 的一次配置、严格 loading phase和 `ModuleManager` 单 owner 一致；Zircon 不复制其 C++ singleton、raw module pointer 或全 `AllPlugins` phase scan。

## 历史任务校正与新增任务

- PERF-MVP-021/035/037/038/044/541/542/543/545 的原始静态根因已被 current source 全部或主要修复；本轮把它们改为回归与动态验收门，不继续指导错误的局部重写。
- PERF-MVP-630 承担 runtime native control-plane 单一代际、per-slot single-flight、锁外 compile 与 batch 原子发布。
- PERF-MVP-631 承担 Editor shared watcher、TaskGraph lifecycle ticket、主线程零 discovery/load/join wait 与有 deadline shutdown。
- PERF-MVP-629 继续负责更外层 catalog/extension/bridge 单一权威；630/631 不新建第四套 catalog 或 scheduler。

## 实施前 RED 门与动态验收

- 0/1/100/1000 plugins、bridge methods、registration systems：记录 loaded/binding/bridge/replay/revision registry count、generation builds、TOML parses、prepared systems、lock wait/hold、alloc/RSS 和 wall。目标 public generation owner=1、batch publish=1、失败 publish=0、parse/prepare-under-loaded-lock=0。
- 两个不同 plugin id 并发冷 build：不被全局 generation build mutex 串行；同 plugin/build key 只执行一次。stable generation 重复 1,000 次的 parse/build/invalidation 为 0。
- 1/100/1000 watched plugins 与 1/10k notification burst：额外 OS worker thread 不随 P 增长，callback 只合并 dirty key并提交 bounded task；同 key coalesced、不同 key 有界并行，Drop/main thread join wait 为 0。
- Play enter/exit、hot reload、unload、shutdown 全部返回 typed ticket/terminal；main-thread filesystem/discovery/DLL load/state callback/wait 为 0，必须 main-affinity 的 publish/activation 段有 p95/absolute deadline。
- 1/16 threads x 1M callback/bridge/context calls保留当前回归：loaded/context writer mutex acquire 为 0，command name/CString alloc 为 0，dense method lookup tree probe 为 0，diagnostics off无计时/shard RMW；reload/unload/stale/panic/rollback无 deadlock/UAF。
- M0 恢复 current F0/F4 后，runtime/editor各至少三次采集 WPR/xperf CPU sample、context switch/wait、线程峰值、allocator/RSS、I/O、wall和可归因功耗。没有同机同场景数据前不宣称接近 Unreal。

本轮不修改 Rust：`registration_replay` 的锁外解析、per-slot cache、load batch publication、watcher线程和Play同步转换必须同时迁移到同一 lifecycle generation/TaskGraph；只把某一段移出锁或替换一个 map 会扩大竞态窗口并继续保留双重权威。先由 Plan02 M1/M5 写 RED ownership/atomicity/thread gates，再做 hard cut。
