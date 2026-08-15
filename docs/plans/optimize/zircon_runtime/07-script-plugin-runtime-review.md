---
related_code:
  - zircon_runtime/src/script
  - zircon_runtime/src/plugin
  - zircon_runtime/src/dynamic_api
  - zircon_plugins/plugin_sdk
  - zircon_plugins/zr_vm_language
  - zircon_editor/src/core/plugin
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/zircon_runtime/runtime/06-plugin-surface-and-lifecycle.md
  - docs/plans/zircon_runtime/runtime/13-script-binding-and-reflection.md
  - docs/plans/performance/01/2026-07-22-runtime-script-static-review.md
  - docs/plans/performance/01/2026-07-22-native-plugin-loader-live-host-static-review.md
  - docs/plans/performance/02/2026-08-15-native-plugin-discovery-live-host-current-architecture-review.md
  - docs/plans/performance/02/2026-08-15-runtime-plugin-catalog-extension-bridge-current-architecture-review.md
  - docs/plans/zircon_plugins/01/failure-2026-07-17-native-host-api-global-context-lock.md
  - docs/plans/zircon_plugins/01/failure-2026-07-17-native-plugin-callback-global-lock.md
  - docs/plans/zircon_plugins/01/failure-2026-07-22-native-callback-per-call-lease-and-abi-copy.md
  - docs/plans/zircon_plugins/01/failure-2026-07-27-native-live-key-hot-reload-contract-drift.md
  - docs/plans/zircon_runtime/runtime/06/failure-2026-07-31-native-plugin-v4-surface-inventory-drift.md
  - docs/plans/zircon_runtime/runtime/13/failure-2026-07-22-runtime-script-binding-hotpath.md
reference_engines:
  - dev/bevy/crates/bevy_app/src/plugin.rs
  - dev/bevy/crates/bevy_app/src/app.rs
  - dev/bevy/crates/bevy_app/src/plugin_group.rs
  - dev/Fyrox/fyrox-impl/src/plugin/mod.rs
  - dev/Fyrox/fyrox-impl/src/plugin/dylib.rs
  - dev/Fyrox/fyrox-impl/src/script/mod.rs
  - dev/godot/core/object/script_language.h
  - dev/godot/core/object/script_instance.h
  - dev/godot/core/extension/gdextension_manager.h
  - dev/godot/core/extension/gdextension_manager.cpp
  - dev/godot/core/extension/gdextension.h
  - dev/godot/core/extension/gdextension.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Modules/ModuleManager.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Private/Modules/ModuleManager.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Projects/Public/Interfaces/IPluginManager.h
  - dev/UnrealEngine/Engine/Source/Runtime/Projects/Private/PluginManager.cpp
---

# 07 · Script / Plugin Runtime 工程化差距

## 1. 结论

Zircon 的脚本与插件基础并非只有临时 stub。当前 `HostExportRegistry` 和 `ScriptCallTable` 会在调用前检查 capability；借用式 `ScriptHostValueRef` 与 byte view 已避免通用输入的强制 clone。脚本包发现具备深度、条目、manifest、路径、字节、wall-time 与 cancellation 预算。原生插件发现也具备 deadline、root/candidate/diagnostic/read/scratch/observer budgets、不可变 last-good generation 和增量 manifest index。Native V4 command 已在加载时构建 dense slot table，并通过 host-owned bounded output sink 限制单命令输出；callback admission 使用 atomic transition bit/callback count 和每 generation `Arc` library pin，外部回调不在 registry 锁内执行。VM hot reload 已实现 save-state、反射 schema migration、staged reflection commit、rollback/reactivation；cooperative GC 也有 host deadline、FIFO dedup schedule 和 telemetry。这些能力应保留，不应因为体系需要重构而退回全局字符串查找、每调用复制或无预算扫描。

真正问题在于这些局部正确性没有汇聚成一个工程级、可证明的执行与发布控制面。native discovery/live host、runtime catalog、extension registry、bridge、runtime profile、bootstrap 和 dynamic session 仍维护平行 authority/generation；batch load 仍按插件逐个发布。原生动态库直接加载到主进程，capability 只能限制 Zircon host API，无法限制 DLL 自行访问 OS、内存和进程，也无法隔离 access violation、abort、死循环或内存破坏。ABI 同时公开 V2/V3/V4 名称和不同 layout epoch，兼容性所有权不清晰。脚本侧声明了 memory soft/hard limits，却没有把它们接入分配、调用或 GC admission；普通 export 没有 instruction/fuel、wall deadline、host-call count、cancel 或 per-package execution budget。真实 ZrVM backend 仍由一个进程级 `OnceLock<Mutex<()>>` 串行所有 package、world 和 thread，并以该锁为 raw pointer owner 的 `unsafe Send/Sync` 前提。

scene script 目前只形成 `onStart/onUpdate/onFixedUpdate` 三个 callback，首个 binding 错误会终止剩余 binding；fixed update 可以在 start 前触发，binding generation 重建又会广泛重置 `started` 与 callback cache。脚本值合同只有少量 scalar/string/bytes/host handle，复杂 gameplay/component 数据仍经 JSON 和 owned projection 传递。运行时没有等价于 Godot ScriptLanguage 的 debugger stack/local/global/expression、breakpoint/step、source map、per-export profile 与 thread enter/exit 控制面。项目里已有 Vampire/WoC 启动脚本接线和 ZrVM real-backend tests，但产品级测试仍没有覆盖真实 editor/app/export 会话、并发包、活跃调用中的 reload、crash containment、长时内存和确定性回放。

本轮登记 14 项 P1 和 2 项 P2，没有新增 P0。当前差距会阻止多项目并发、可信插件分发、无停顿 hot reload 和可调试脚本达到 Unreal 级工程门槛，但现有静态证据尚不能证明已发布数据发生不可恢复损坏，因此暂不定 P0。若未签名 native package 已进入受信任分发链，或真实 DLL crash/memory corruption 可破坏 editor 项目数据，应把 P1-4 上调为 P0 并冻结第三方 native plugin 自动加载。

## 2. 审查边界与物理覆盖

### 2.1 已读范围

- `zircon_runtime/src/script` 当前物理快照 102 个 Rust 文件、约 17,104 行、153 个 `#[test]`。本轮逐层读取 VM public surface、host exports/call table、capability、package/manifest/discovery、instance/manager/hot-reload/GC、reflection、gameplay host、scene system 与 tests；没有把文件数量误当成全部行为已验收。
- `zircon_runtime/src/plugin` 613 个 Rust 文件、约 51,837 行、381 个 `#[test]`。重点读取 native ABI/loader/live host、package manifest、runtime plugin catalog、extension registry、bridge、runtime profile、export build plan 和产品 bootstrap/dynamic-session 消费链。
- `zircon_runtime/src/dynamic_api` 74 个 Rust 文件、约 14,094 行、162 个 `#[test]`；沿 session startup script、linked plugin、runtime profile、Vampire gameplay/HUD/menu/frame tests 检查真实产品入口。
- `zircon_plugins` 当前快照约 2,835 个 Rust 文件、222,013 行、1,986 个 `#[test]`。本篇没有宣称逐算法完成所有 feature plugin 审查；详细读取 plugin SDK、first-party catalog、`zr_vm_language` real backend 与 product manifest，其他 physics/render/navigation 等插件内部算法分别进入 08/09/10。
- `zircon_editor/src/core/plugin` 35 个 Rust 文件、约 5,779 行、51 个 `#[test]`，用于核对 editor admission/isolation/manager/publication/watcher 和 Play transition 边界，不替代后续 editor 插件工作流专篇。
- 复用并重新核对两个 2026-08-15 current-source 全读记录：native discovery/live host 88/88 文件，以及 runtime catalog/extension/bridge/profile 135/135 文件。旧 2026-07-22 static review 只作为历史差距索引；凡与当前源码冲突，以当前源码和 current-source 记录为准。
- 对照 Bevy `Plugin` 的 build/ready/finish/cleanup 与 uniqueness；Fyrox DynamicPlugin prepare/reload 和 ScriptTrait lifecycle；Godot ScriptLanguage/ScriptInstance debugger/profile/reload/thread contract 与 GDExtension initialization-level reload；Unreal ModuleManager/IPluginManager 的 version selection、dependency chain、module phase、content/localization mount/unmount 和 compatibility report。

### 2.2 明确未覆盖

- 本篇不评价 animation/physics/audio/navigation/network/rendering 插件内部算法正确性和画面上限；它们进入 08/09/10。这里只审查共同的 package、dependency、activation、ABI、script execution、hot reload、trust 和 observability 控制面。
- Unity Graphics 参考树主要拥有 SRP、render graph、shader/resource lifetime 证据，不提供可与脚本 VM、native module manager 直接对齐的主控制面。本篇不为凑引擎列表制造错误类比，Unity 对照延后到 09/10。
- 没有运行 Cargo、真实第三方 DLL、恶意插件、crash process、脚本 storm、1k package、长时 GC、WPR/ETW/Tracy 或 editor/app/export 产品验收。相关源码在本轮有大量其他 Session 活跃修改，本文所有 finding 均为 `recheck_required`。
- `zircon_plugins` 的 2,835 文件只完成目录与公共注册面的物理盘点；除 SDK/ZrVM/catalog 以及当前 owner reports 涵盖的范围外，不能由 07 宣称整个插件库已经深审完成。

## 3. 当前闭环与应保留能力

### 3.1 Script host boundary 与 discovery budgets

`HostExportRegistry` 将 export descriptor、capability 和 callable 绑定在一起，`ScriptCallTable` 在调用前验证 package capability；borrowed `ScriptHostValueRef`/byte view 比把每个参数转换成 owned value 更适合热路径。后续 typed ABI 应建立在这条借用路径上，而不是回退成通用 JSON/string dispatch。

`VmPluginPackageDiscovery` 已定义 root/depth/entry/manifest/path/byte/time/cancel 限额且不跟随 symlink；错误可以保留 last-good package set。这个“有预算扫描 + immutable publication”模式应与 native discovery 共用更高层 generation transaction，而不是删掉现有局部保护。

### 3.2 Native live host 的局部并发与输出控制

Native callback admission 使用 transition bit 和 active-callback count；每个 generation 的 library 由 `Arc` pin 住，foreign callback 在 registry lock 外运行，diagnostic storage 也已分成 64 shards。V4 command 在 load-time 建 dense slot，输出进入 host-owned bounded sink，并有全局 256 MiB hard cap。旧 failure 中“每次命令都持全局 callback mutex”“command output 完全不受限”已经不是 current-source 事实，不能重复登记。

现有 discovery refresh 也拥有 cancellation/deadline、candidate/read/diagnostic/scratch/observer budget、增量 manifest index 和 immutable last-good。问题位于跨 plugin batch publication、authority 收敛、callback quiescence、trust/isolation 和其他 payload/state budget，不应破坏已经完成的局部热路径。

### 3.3 VM hot reload、reflection 与 cooperative GC

`HotReloadCoordinator` 已把 state save、new instance activation、schema migration、reflection staging/commit 和失败 rollback 串成明确步骤；callback generation 也会刷新。GC bridge 能按 package 排队、dedup、设置 host wall deadline，并分别记录 host/backend telemetry。真实 `zr_vm_language` tests 覆盖 lifecycle、state、GC 和最小文档示例。后续目标是把这些机制提升为 per-slot lease/generation 和全产品 acceptance，而不是另造第二套 reload/GC coordinator。

## 4. 差距清单

### P1-1：插件控制面存在多个平行 authority，没有单一 compiled generation

2026-08-15 current-source review 已确认 `NativePluginLiveHost` 同时维护 loaded generation、registration replay、runtime behavior/system/interface/capability 等多组 registry，并有独立 generation-build locks。其外又存在 `RuntimePluginCatalog`、extension registry、bridge、runtime profile、project plan cache、bootstrap 和 dynamic session publication。相同 plugin identity、capability、dependency 与 availability 因而可在多个 snapshot 中处于不同 generation；bootstrap 和 dynamic session 还能重复构建 catalog。

目标建立唯一的 `PluginCatalogGeneration`：它由 discovery/package resolution 编译，包含 `PluginSlot -> BackendGeneration`、dependency closure、capabilities/interfaces、extension/system/module contributions、content roots、runtime profile projection 和 diagnostics。所有 consumer 持同一 immutable generation handle；派生索引在 generation build 中一次完成。native/script/static backend 只是 slot 的实现类型，不再各自拥有产品可见真相。旧 catalog/bridge/profile registry 要么成为 generation 内索引，要么硬切删除，不能靠双写长期同步。

### P1-2：batch load、activation、reload 与 publication 不是一个失败原子事务

当前 native batch load 按插件逐个调用和发布；某个后续插件失败时，早期插件的 library、registrations、extensions 或 profile 已可能对消费者可见。catalog/extension/bridge 也各自 rebuild/publish，不能证明 consumer 永远只见旧完整 generation 或新完整 generation。live host 在 active callbacks 存在时主要拒绝 transition，没有统一 drain ticket、deadline 和跨所有 contribution 的 rollback ledger。

目标 transaction 分成 resolve/prepare/quiesce/activate/validate/publish/retire。prepare 阶段加载库、解析 ABI、构建 dependency graph、注册到 staging generation，但不暴露给运行时；quiesce 对受影响 slot 发 admission close，等待 callback/task/state ticket 到 deadline；一次 compare-exchange 发布整批 generation；失败只销毁 staging。旧 generation 在所有 callback/service/scene/extension lease 释放后 retire/unmount。load/reload/unload 必须生成 durable transition report，能区分 prepare failure、quiesce timeout、activation failure、publication conflict 和 retirement leak。

### P1-3：native package dependency 不是版本/来源/信任约束求解器

`PluginDependencyManifest` 当前只有 `id`、`required`、可选 `capability` 和 `interfaces`；没有 version requirement、conflicts/provides/replaces、source repository、content digest、signer/trust、target/platform/profile 或 feature expression。package 自身虽有 semver shape/SDK compatibility 检查，editor 也能发现部分 cycle，但没有一个运行时统一 solver 生成可锁定、可重放的 dependency plan。bridge-specific closure 不能替代跨 native/static/script/content 的全局图。

Unreal PluginManager 会在发现的多个版本中选择 requested version，报告 dependency/reference chain、missing/sealed/disallowed/incompatible plugin，并按 module phase 和 content/localization mount policy配置 enabled graph。Zircon 目标不是复制 Unreal 类层次，而是交付 deterministic `PluginResolutionPlan`：精确 package/version/source/digest/signer/target/build/profile，完整 required/optional/conflict/provide graph，稳定拓扑和拒绝原因。plan 写入 lockfile/artifact；editor、runtime、export/cook 使用同一 plan，不得在各进程重新猜测。

### P1-4：native DLL 在主进程直接执行，当前 isolation 不能提供信任或故障边界

native loader 最终调用 `libloading::Library::new` 将候选 DLL 映射进 editor/runtime 主进程。editor `isolation.rs` 的 `catch_unwind` 和 SDK panic guard只能捕获 Rust unwind；它们无法隔离 access violation、stack corruption、abort、死循环、线程泄漏、allocator mismatch 或任意 OS/file/network/process 访问。capability table 只限制插件通过 Zircon host API 能做什么，不可能沙箱 native code 自行发起的系统调用。当前 package/library搜索也没有 signer certificate、trust chain、quarantine 或签名验证合同；content/source hash 不是发布者身份。

目标把插件分成内建受信任、签名 in-process、受限 out-of-process 和拒绝四类。发现阶段验证 package manifest/content/library/debug artifacts 的 digest 与签名链，记录来源、publisher、policy decision 和 revocation state；未知/开发包默认不自动进入产品 in-process。可隔离插件通过受监管 worker process + versioned IPC + shared-memory pages 执行，具有 CPU/memory/time/output/handle预算和crash restart/quarantine。必须 in-process 的渲染/低延迟扩展只接受显式 trust policy，并用 crash artifact、watchdog、thread registration 和 leak audit降低风险；文档不能把 `catch_unwind` 称为沙箱。

### P1-5：ABI 同时公开 V2/V3/V4 名称与多个 layout epoch，兼容责任和可执行范围不清晰

`plugin::native` 当前公开 `NativePluginByteSliceV2`、`OwnedByteBufferV2`、`CallbackStatusV2` 以及 V3 aliases；registration scope 同时有 V3/V4，descriptor/host function table 仍带 V3，runtime behavior 使用 V4，entry report又有独立 layout epoch。SDK复制对应声明并把部分 V3 alias 指向 V2 layout。旧 V2 behavior metadata 可以读取但不能执行，这使“类型可解析”“host可注册”“behavior可调用”“state可迁移”四种兼容性被一个版本名混在一起。

目标冻结一张显式 ABI epoch matrix：package manifest epoch、descriptor epoch、host API table、callback table、registration batch、owned buffer/allocator、behavior/state schema和debug symbols分别列出 host read/execute/upgrade政策。只保留一个 current public SDK surface；旧 epoch 进入隔离 adapter crate/process，或做硬切迁移，不再从 root facade 暴露误导性 aliases。C header/Rust SDK/host decoder由同一 schema生成，并用 size/alignment/offset/calling-convention/allocator/panic/string/nullability conformance tests覆盖 MSVC/GNU/Linux/macOS。未知字段与版本 fail closed并产出 compatibility report。

### P1-6：native callback 除 command output 外缺少统一执行、输入、状态与驻留预算

V4 command output 已有上限，但 command input、save-state returned buffer、restore-state input、diagnostic C string、callback wall time 和插件自行保留的 host handles没有统一 per-plugin policy。`take_owned_bytes` 按 foreign pointer/length复制插件返回状态；若长度很大，host在验证内容前就支付分配和copy。`invoke_command/save_state/restore_state/unload` 是同步调用，没有 deadline、cancel、watchdog 或 cooperative yield。in-process callback卡死会阻塞负责它的host线程；capability admission并不解决资源占用。

目标为每个 `PluginSlotGeneration` 编译 `NativeExecutionPolicy`：input/output/state/diagnostic bytes，resident/in-flight buffers，active callbacks/threads/handles，wall/CPU time和shutdown deadline。所有跨ABI variable buffer先验证 declared length/cap和allocator provenance；大型state用host-owned paged sink/artifact，而不是foreign unbounded buffer。out-of-process backend可强制超时和内存上限；in-process backend至少通过watchdog、admission close、cooperative cancel token和hung-state quarantine把风险显式化。所有超限进入structured terminal status，不能截断后当成功。

### P1-7：真实 ZrVM backend 由一个进程级 mutex 串行，无法支持多包/多世界并发

`zr_vm_language/runtime/src/real_backend/lock.rs` 用静态 `OnceLock<Mutex<()>>` 返回 `'static` guard；activate/deactivate/save/restore/schema/GC/export和owner drop都依赖该锁。`ZrVmRuntimeOwner` 对 raw-pointer binding types 的 `unsafe Send/Sync` 正确性说明也建立在“所有访问由同一 process-wide lock 串行”上。一个 package 的长脚本、GC或drop会阻塞所有其他 package/world/thread；poison recovery只能恢复mutex可用，不能证明底层VM状态未损坏。

目标先明确 ZrVM C/runtime 的真实线程模型。如果 runtime instance 可隔离，则每个 VM domain拥有独立 runtime/session/allocator和owner thread或per-domain lock，package slot在domain内调度；不同domain可并发。若底层库确实是 process-global non-reentrant，则把它放入专用 worker process/actor，使用bounded request queue、deadline和batch host calls，并明确该backend无法满足同进程低延迟并行；不能用 `unsafe Send/Sync + global mutex`伪装并发能力。验收必须覆盖1/10/100 package、1/4/16 worlds和call/GC/reload竞争。

### P1-8：VM slot 通过“从 map 取出实例”互斥调用，重入、并发调用与 lifecycle quiescence 没有统一合同

`HotReloadCoordinator::call_slot_export` 会从全局 slots map暂时取出 active instance，调用结束再放回。相同 slot 的 nested/reentrant callback或并发调用在期间只能得到“active instance unavailable”。load/reload/unload使用一个全局 `lifecycle_guard` 串行不相关package，普通 export又不持同一 lifecycle lease；reload撞上active call时主要表现为instance unavailable/reject，而不是可观测的close-admission、drain、deadline、generation handoff。GC使用另一全局guard，最终仍靠instance take间接互斥。

目标为每个 slot建立状态机和generation lease：Active(generation)、Closing、Reloading、Retiring、Failed。call取得 `VmCallLease` 后固定backend generation；reload先关闭新 admission，再等待已发lease/host-call/task到deadline，save/migrate/activate staging generation并原子切换。是否允许同slot重入由backend capability明确声明；允许时需要stack/actor mailbox或reentrant-safe context，不允许时返回专用ReentrancyDenied并带call chain，不能伪装成instance丢失。不相关slot不得被一个global lifecycle mutex串行。

### P1-9：脚本 memory policy 只做声明校验，普通调用没有 fuel、deadline、host-call 或取消预算

`VmPluginMemoryPolicy { soft_limit_bytes, hard_limit_bytes }` 当前能解析并验证非零与soft<=hard，但搜索运行路径没有发现它被allocator、bytecode load、host value、state、reflection或GC admission消费。`VmPluginInstance::call_export` 只接收module/export/arguments，没有 execution context、fuel/instruction count、deadline、cancel token、host-call count或allocation budget。discovery cancellation和GC deadline不能约束普通脚本执行；无限循环或极端host-call fan-out仍可能占满runtime线程。

目标编译 `ScriptExecutionPolicy`：bytecode/code pages、heap soft/hard、stack depth、alloc count/bytes、instructions/fuel、wall/CPU slice、host calls、host bytes/handles、async tasks和per-frame/package totals。每次调用携带 `ScriptExecutionContext`，backend必须定期poll cancellation/deadline/fuel；soft limit触发incremental GC/telemetry，hard limit以可分类trap终止当前call或domain。预算继承到host export、nested call、async continuation和reload state，不能在host边界重新置零。deterministic profile还要固定clock/random/locale和浮点政策。

### P1-10：scene script lifecycle、调度、失败隔离与binding identity不完整

`scene_system.rs` 只缓存 `onStart/onUpdate/onFixedUpdate`。`onStart` 只在Update分支首次调用，因此若schedule先运行FixedUpdate，同一binding可以先收到`onFixedUpdate`。没有onInit/onEnable/onDisable/onDestroy、scene enter/exit、pause/resume、OS event、message/event、editor/tool和state migration callback。`tick_script_bindings` 顺序遍历并用 `?` 返回，首个失败会中止剩余binding及当前system。projection按dynamic component generation重建 `Rc<ActiveScriptBinding>`，重建会重置`started`和callback cache；binding identity基于格式化字符串和数组index，reorder/局部编辑不具备稳定generation语义。

Fyrox ScriptTrait至少区分init/start/deinit/OS event/update/message；Godot ScriptInstance/Node lifecycle拥有enter/exit tree、ready/process/notification等更完整边界。Zircon目标建立stable `ScriptComponentId(index,generation)`和明确lifecycle state machine：Constructed、Initialized、Enabled、Started、Disabled、Destroying、Destroyed，scene/world/package reload转换各有顺序和exactly-once规则。schedule先批量准备call contexts，再按声明的access/capability并行调度；错误按project policy隔离为component/package/world fail-fast或continue-with-quarantine，并聚合report，不能由第一个错误偶然决定剩余脚本是否运行。

### P1-11：脚本值与反射/玩法边界过窄，复杂数据仍依赖 JSON 和 owned projection

`ScriptHostValue` 当前主要是Null/Bool/Int/Float/String/Bytes/HostHandle，没有typed array/map/struct/enum、borrowed slice/view、nullable/Result、out/ref、async future、entity/component/resource lease或schema generation。`scene_system` 从dynamic component的 `serde_json::Value` clone并反序列化bindings，gameplay component/state API也用JSON表示复杂对象。这样虽然易于接线，却把schema错误推迟到运行时，形成字符串查找、parse/format、allocation/copy和stale handle风险；reflection catalog与实际host value ABI没有共同布局计划。

目标由同一 reflection/type registry编译 `ScriptTypeId + SchemaGeneration + Layout/MarshallingPlan`。标量、small POD、slice、string view、typed struct/enum和host lease有明确borrow/own/copy/retain规则；复杂world mutation使用command buffer或typed transaction，不把整个对象编码成JSON。hot reload通过schema-aware field mapping/migration plan处理旧state；未知/stale generation fail closed。JSON只保留authoring/diagnostic/document边界，不能成为frame gameplay ABI。每个export记录marshal bytes/allocs/copies和host-call latency，才能证明新合同优于旧路径。

### P1-12：没有工程级脚本 debugger、source mapping、profiler 与执行追踪面

current-source 搜索没有找到VM breakpoint/step/pause、stack frames、locals/globals/watch/evaluate、source map、exception/trap mapping、per-export inclusive/exclusive time、allocation profile、thread enter/exit或remote debugger protocol。editor目前能从log中的 `ScriptLocation`跳转，build diagnostics也可定位源码，但这不是运行中调试器。GC telemetry和generic host hotpath counter无法回答“哪个package/export/entity/frame分配最多、调用最慢、发生何种trap”。

Godot `ScriptLanguage` 明确提供debugger stack/local/global/member、expression、reload、profiling start/stop/frame/accumulated和thread enter/exit接口。Zircon目标定义backend-neutral `ScriptDebugAdapter` 与 `ScriptProfiler`: stable source/document/module/function ids，debug symbols/source maps，breakpoint bind/rebind，pause/continue/step，stack/locals/watch受预算快照，trap/exception链，per-package/export/entity call/time/alloc/host-call/GC统计。debug pause必须与game loop、network/audio/render和hot reload协调，不能持world/plugin registry锁；shipping build可裁剪符号，但仍保留crash stack/artifact和低开销counters。

### P1-13：VM package manifest不足以支持依赖、兼容、可重现构建和安全分发

`VmPluginManifest` 只有name/version/entry/capabilities/management policy；`VmPluginPackage` 是manifest、可选ZrVM project path/mode和raw bytecode。没有script dependencies/version ranges、engine/runtime/language/host API compatibility、target/profile、content digest/signature、build id、compiler/options、debug/source-map artifact、native host interface requirements或lockfile identity。project path更像开发输入，不是可验证的发布artifact；raw bytecode也没有chunk/checksum/streaming/size policy。

目标将source project、compiled package和installed package分层。build artifact manifest记录compiler/language/host ABI、target/profile/options/source/content/dependency hashes、bytecode pages、reflection/debug symbols/source maps和signature；runtime只加载resolved/verified installed package。script dependencies进入P1-3同一resolution plan，并可与native/static providers声明interface/capability要求。discovery预算只保护扫描过程，package内部page/section/signature/decompression也必须有长度、ratio、time和memory预算。

### P1-14：现有测试没有证明真实产品会话、并发规模、崩溃与性能目标

`zr_vm_language` owner已有4个real-backend tests覆盖基础lifecycle/state/GC/minimal example，是重要起点。Vampire/WoC项目manifest也选择ZrVM language并配置startup packages，dynamic session具备`load_startup_scripts`。但 dynamic API 中Vampire gameplay/HUD/menu/frame相关real-ZrVM tests目前标记ignore并说明coverage移到plugin owner；plugin owner tests使用临时fixture和mock manager，不能证明editor->Play/app->dynamic session->scene->export/cook的完整产品链。也没有同slot重入、10/100 packages、并发world、active-call reload、hung script、malformed/unsigned DLL、worker crash/restart、长时heap/handle稳定和deterministic replay证据。

目标建立分层acceptance而不是只增加unit数量：ABI/schema conformance；fake backend lifecycle/property tests；real ZrVM/native backend integration；editor/app/export/cook产品session；stress/fault/security；性能与参考引擎同机对比。每次结果绑定source manifest、binary/package hashes、backend/build/target、scene/workload、trace和budget policy。只有同硬件同场景的CPU/GPU/RSS/latency/frame-time数据才能支撑“优于Unreal”；静态代码阅读不能证明性能领先。

### P2-1：catalog/profile/report/control-plane仍有重复owned DTO、字符串索引和全量派生重建

当前 review 已确认 project plan cache只按target键控且使用global lock，extension unload会重建registry，runtime profile重建availability index，bootstrap/dynamic session重复catalog build；native SDK/host之间也仍有owned DTO和字符串projection。脚本real backend每次调用会把arguments clone/转换成 `Vec<zrvm::Value>`，格式化export label并转换String。它们未必在小项目立刻成为主瓶颈，但在1k plugins、10k exports/interfaces、频繁profile切换或高频script calls下会扩大锁、分配和copy成本。

目标让P1-1 generation build一次intern plugin/module/export/interface/type ids，生成dense spans和immutable shared tables；lookup/call hot path只使用typed handles与borrowed views。增量变化按affected slots更新persistent index，profile是generation上的轻量projection，不重新扫描所有descriptor。需要记录generation build time/bytes、cache hit、strings interned、DTO/copy bytes、lookup/call p50/p95/p99，按数据决定小对象是否值得specialize。

### P2-2：测试数量与source-shape guard不能替代跨ABI、并发、真实故障和产品证据

script/plugin/dynamic API/plugin workspace合计已有大量tests，capability、discovery budget、hot reload rollback、GC、native callback lease和catalog行为都有覆盖，应保留。但多个测试依赖mock backend、temporary manifest、`include_str!`/`.contains(...)` source guards，或只验证结构/错误字符串。它们无法证明MSVC/GNU ABI layout、foreign allocator、DLL access violation、hung callback、global VM contention、active-call reload、signed package chain、editor close/drop、1k plugin scale和shipping product session。

目标测试标签区分unit/property/integration/product/stress/fault/security/performance；source-shape guard只能记structure evidence。关键矩阵由managed harness设置deadline和resource caps，保存crash dump/trace/artifact/digest；失败必须指出哪个generation/slot/call/budget断裂。没有真实backend和产品artifact时，计划状态只能是implementation_pending或validation_pending，不能因test count关闭。

## 5. 参考引擎对齐结论

### 5.1 Bevy 与 Fyrox

Bevy `Plugin` 把build、ready、finish和cleanup作为App生命周期阶段，并默认要求plugin type uniqueness；PluginGroup负责有序组合和enable/disable。它适合参考静态/进程内Rust plugin的阶段语义与重复保护，但没有直接解决不受信任native DLL、ABI兼容和多版本package solver，不能作为Zircon最终上限。

Fyrox DynamicPlugin拥有prepare/reload与动态库生命周期，ScriptTrait覆盖init/start/deinit/OS event/update/message；它证明Zircon三回调scene lifecycle过窄。Fyrox现有dynamic/script模型仍不足以提供本文要求的签名信任、out-of-process isolation、fuel、完整debugger和原子跨plugin generation，因此只借鉴生命周期分层，不照搬owner结构。

### 5.2 Godot

Godot GDExtensionManager按initialization level加载/初始化/反初始化，并为reload维护prepare/finish与实例跟踪；它对可reload边界、兼容限制和对象存活有显式管理。ScriptLanguage/ScriptInstance公开validate/reload/debug stack/local/global/expression/profile/thread hooks，给editor与runtime一个统一语言适配面。Zircon应借鉴“backend-neutral language/debug contract”和“extension level + instance lifetime”，同时避免Godot全局singleton、裸ObjectID和缺少强generation/budget的部分。

### 5.3 Unreal

Unreal ModuleManager提供load/unload/abandon、compatibility query、module change event和library unmount；PluginManager管理多版本发现/选择、dependency/reference chain、sealed/disallowed/incompatible检查、module phases以及content/localization/module mount/unmount。对Zircon最关键的是“一个enabled plugin graph控制所有贡献和挂载”以及可解释的失败链，而不是复制宏和类名。

Zircon目标还应比传统in-process module manager更严格：package digest/signature、generation lease、失败原子batch、host-owned budgets、可选worker isolation和deterministic artifact。达到Unreal规模不等于复刻其历史约束；性能领先必须通过同机工作负载、trace与产品稳定性共同证明。

## 6. 目标架构与所有权

### 6.1 Package 与 catalog control plane

1. `PluginPackageStore` 负责发现、下载/安装、digest/signature、quarantine和artifact lifetime，不执行插件。
2. `PluginResolver` 输入project request/lockfile/target/profile/trust policy，输出deterministic dependency plan。
3. `PluginGenerationBuilder` 在staging中加载/验证backend，编译capability/interface/system/extension/content/runtime-profile索引。
4. `PluginCatalogGeneration` 是产品唯一可见authority；editor/runtime/export/cook都持同一plan和generation identity。
5. `PluginTransitionCoordinator` 负责prepare/quiesce/activate/publish/retire/rollback和durable report。

### 6.2 Backend 与执行控制

1. static Rust、trusted native、isolated native、ZrVM与未来语言backend实现同一slot lifecycle，但各自声明threading/reentrancy/isolation/debug capability。
2. 每个slot generation拥有admission gate、call/task/state/host-handle leases和compiled execution policy。
3. call context携带deadline/cancel/fuel/bytes/host-call/trace/determinism metadata，预算跨nested/async/host boundary继承。
4. reload只切换slot backend generation；旧generation在所有lease释放后retire，content/extensions/systems与backend同事务。
5. crash/hang/budget trap产生structured status并按policy隔离component/package/domain/process，不把全局mutex poison当恢复完成。

### 6.3 Script scene、types 与 tooling

1. stable script component identity与完整lifecycle state machine独立于dynamic JSON projection generation。
2. reflection registry编译typed marshalling plans；frame hot path使用borrowed views/handles/command buffers。
3. scene schedule根据declared world/resource access与backend thread model编译batch，错误按明确policy聚合。
4. debugger/profiler/source maps作为language backend contract，不依赖日志字符串猜位置。
5. product trace把package/export/component/entity/generation、CPU/wall/fuel/alloc/host-call/GC/reload关联到同一frame/session。

## 7. 重构顺序

### M0：current-source冻结、ABI/authority盘点与风险闸门

- 重新取script/plugin/dynamic API/SDK/ZrVM/editor plugin指纹，确认其他Session改动已落定并取得owner授权。
- 列出所有平行catalog/generation/registry/cache、所有load/reload/unload入口、所有V2/V3/V4/epoch类型和真实consumer。
- 暂停未知来源native DLL的自动in-process加载；定义开发override和审计日志，不把现状包装成安全隔离。

### M1：package resolver 与唯一 catalog generation

- 扩展native/script package manifest和lock artifact，加入version/source/digest/signer/target/profile/dependency/conflict/provide合同。
- 交付deterministic resolver、完整reference chain和staging generation builder。
- bootstrap、dynamic session、editor、export/cook改读同一resolution plan；catalog/extension/bridge/profile平行authority硬切为generation索引。

### M2：transactional lifecycle 与 generation lease

- 交付per-slot admission/call/task/state/handle lease和prepare/quiesce/publish/retire transaction。
- native/script/static contribution整批原子发布，失败完整rollback；active-call reload具有deadline与durable report。
- 删除global lifecycle serialization和“take instance from map”作为主要互斥协议，明确reentrancy capability。

### M3：执行预算、ZrVM domain 与 native isolation

- memory policy真正接入allocator/GC/load/call；增加fuel/deadline/cancel/host-call/nested/async预算继承。
- 根据ZrVM真实线程能力实现per-domain owner或worker actor，移除以process-global mutex支撑的伪并发 `unsafe Send/Sync`。
- 建立trusted/isolated native backend、worker supervision、shared-memory/IPC budgets、crash restart/quarantine和signing policy。

### M4：scene lifecycle、typed ABI 与调试性能工具

- 稳定script component identity，补齐init/enable/start/update/fixed/disable/destroy/message/OS/pause/reload状态机和failure policy。
- reflection schema生成typed marshalling plans，玩法热路径硬切JSON/string lookup；保留document/diagnostic JSON边界。
- 交付debug adapter、source maps、breakpoint/step/stack/locals/watch、per-export profile和shipping crash trace。

### M5：产品、故障、安全与性能验收

- 用Vampire/WoC真实editor/app/export/cook会话替代被ignore的上层real-backend证据空洞。
- 运行dependency/version/signature、ABI matrix、reload/crash/hang/OOM/long-run、1k plugin/100 package和multi-world矩阵。
- 与参考引擎在同硬件同项目规模下比较startup/reload/frame script CPU/RSS/p99/GC/hitch和tool latency；证据达标后才关闭06/13与plugin failures。

## 8. 验收矩阵

### 8.1 Package、dependency 与 lifecycle

- 1/10/100/1,000 packages，1/10 versions，required/optional/conflict/provide/cycle/missing/sealed/disallowed；resolver结果稳定且lockfile可重放。
- 单/批load、后项失败、activation failure、publication conflict、active-call reload、quiesce timeout、retirement leak；consumer只能观察完整old或new generation。
- content/extensions/systems/interfaces/profile与backend同事务mount/unmount；失败后没有半注册项、stale callable或未卸载library。

### 8.2 Script execution 与 scene

- 1/10/100 packages，1/4/16 worlds，1/1k/100k components，30/60/120 Hz update，fixed/update不同排序；exactly-once lifecycle和stable identity成立。
- infinite loop、deep recursion、allocation storm、host-call fan-out、大state/argument/result、nested/reentrant/async call；fuel/time/memory/bytes/handles按policy终止且不拖死其他domain。
- hot reload在0/1/1k active calls、GC、scene transition和debug pause期间执行；state/schema/callback generation正确，旧lease最终归零。
- typed ABI记录marshal alloc/copy/bytes和p50/p95/p99；与JSON旧路径在同场景对比，不以微型fixture代替产品trace。

### 8.3 Native ABI、信任与故障

- MSVC/GNU/Linux/macOS的size/alignment/offset/calling convention/allocator/string/null/unknown epoch conformance；旧epoch只按matrix允许的read/adapter范围工作。
- valid/invalid/revoked/unknown signer、digest mismatch、source change、package zip/decompression bomb；policy decision可审计且fail closed。
- access violation、abort、hang、memory/CPU/output/state超限、worker crash/restart、thread/handle leak；isolated backend不破坏editor/runtime主进程，in-process风险有明确trust gate。

### 8.4 Tooling 与产品证据

- breakpoint bind/rebind、pause/continue/step、stack/locals/watch/evaluate、trap/source map、reload while paused；不持world/catalog锁且有snapshot预算。
- per-package/export/entity call/time/alloc/host-call/GC/reload counters与frame trace一致，shipping crash artifact可解析。
- Vampire/WoC从project open、Play、hot reload、stop、standalone app、export/cook、restart完整通过；运行1/8/24 h后RSS/handle/thread/generation稳定。
- Unreal/Fyrox/Godot/Bevy对比只使用等价场景、同硬件、同build/profile和公开trace；没有数据时结论只能是“目标”，不能写“已优于”。

## 9. 与既有计划的关系

- Runtime06继续拥有native plugin surface/lifecycle和ABI hard cut；本文要求把V2/V3/V4/epoch兼容矩阵、batch atomic generation、trust/isolation与budget纳入重新验收。旧native facade closeout不等于第三方插件工程化完成。
- Runtime13继续拥有script binding/reflection与ZrVM接线；本文重开memory policy执行、per-domain concurrency、slot lease、scene lifecycle、typed ABI、debug/profiler和产品证据。现有host binding/GC局部优化应保留。
- `zircon_plugins/01` failures继续拥有native discovery/callback/replay/catalog/profile局部性能修复；P1-1/P1-2负责把这些局部owner收敛成单一generation transaction，不能让每个failure各造一套缓存。
- Runtime04/05分别拥有asset/content artifact与scene/world identity；plugin package mount依赖04，script component lifecycle依赖05。07不能复制asset store或ECS scheduler，只定义跨边界的generation/lease/transaction。
- Editor后续专篇拥有plugin browser、permission/trust UI、watcher/Play操作与diagnostic UX；07拥有其必须消费的resolver、transition ticket、isolation和debug backend合同。
- Graphics插件内部算法、GPU feature registration和Unity Graphics对照转交09/10；它们仍必须消费07的package/generation/ABI/trust公共控制面。

## 10. 当前状态

`review_complete；implementation_pending；recheck_required`

本篇只完成静态current-source审查、参考源码对照和重构路由，没有修改生产代码，也没有声明Cargo、真实ZrVM/native DLL、签名链、crash isolation、脚本/插件规模或性能对比通过。07关联的script/plugin/dynamic API/SDK/ZrVM/editor plugin文件存在其他Session的大量修改；进入M0前必须重新读取diff、取得coordinator授权并复核所有源码锚点与failure状态。
