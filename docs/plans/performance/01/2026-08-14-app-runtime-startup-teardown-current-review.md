---
related_code:
  - zircon_app/src/entry/entry_runner
  - zircon_app/src/entry/runtime_library
  - zircon_runtime/src/dynamic_api/session/construction.rs
  - zircon_runtime/src/dynamic_api/session/ffi.rs
  - zircon_runtime/src/dynamic_api/session/project.rs
  - zircon_runtime/src/dynamic_api/session/linked_plugins.rs
  - zircon_runtime/src/dynamic_api/session/linked_session.rs
  - zircon_runtime/src/dynamic_api/session/script_systems.rs
  - zircon_runtime/src/dynamic_api/session/runtime_ui.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
  - docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
  - docs/plans/zircon_plugins/01-plugin-architecture-core.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/Launch/Private/LaunchEngineLoop.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/AssetRegistry/Private/AssetRegistry.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/AssetRegistry/Private/AssetDataGatherer.cpp
  - dev/bevy/crates/bevy_app/src/app.rs
  - dev/bevy/crates/bevy_app/src/plugin.rs
  - dev/bevy/crates/bevy_asset/src/server/mod.rs
tests:
  - current-source scoped rustfmt check
  - managed Windows Cargo pending
  - cold/warm startup WPR matrix pending
  - teardown contention and failure-injection matrix pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# App runtime startup/teardown 当前源码性能复审（2026-08-14）

## 范围与证据边界

本轮完整复读 App F0 runtime runner 与 runtime-library owner **15/15 个文件、6,261 行
（5,630 非空行）**；并沿 `create_session` 调用图完整复读 Runtime 构造直接 owner **7/7 个文件、
2,481 行（2,317 非空行）**。App 文件集 SHA-256 为
`40AE3BAAD5CE59510A6CCF87F12181629238926E6C8CB639918FEBCD279650C1`，Runtime anchor
文件集为 `F464898C87CE574A670DCECF4E62F72EC611AAFDE52606EABA3ADDE87FB4BFDA`。

这些生产文件存在其他 Session 的未提交修改，本轮不覆盖其实现。App 15/15 owner 的
`rustfmt +1.94.1 --edition 2021 --check` 通过；Runtime 7 个 anchor 中 `ffi.rs`、
`construction.rs`、`project.rs`、`linked_plugins.rs`、`script_systems.rs` 因当前 import/断言排版
漂移未通过，本轮不修改这些 foreign-dirty 文件。managed Windows validator 在 Cargo 命令生成前
仍被外部 unmanaged artifacts 阻断；现有 `target/profiling` 二进制早于当前源码，
没有 source fingerprint，故不运行它来伪造 current-source WPR 基线。以下结论是当前哈希快照的
静态调用图，不是 F0 产品验收。

## 当前启动事务

`run_runtime_with_args` 依次解析 CLI、解析项目路径、启动 profile capture、加载 DLL、创建 winit
event loop，再同步调用 Runtime V3 `create_session`。Runtime 返回会话前又串行执行：

1. 打开项目 manifest/registry，构造 prepared project 与插件 manifest 快照；
2. clone/补全插件选择，分别构造 builtin module report、RuntimePluginCatalog 和 extension plan；
3. 创建 CoreRuntime，逐模块注册、批量 activate，并解析 input/render 服务；
4. `open_prepared_project` 启动 watchers、同步全项目 `scan_and_import`、准备并发布资源 generation；
5. 读取 navmesh；对每个脚本 root 提交 discovery 后立即 `.wait()`，再串行 materialize/load 包；
6. 同步读取默认或 Play scene、应用 world extension plan；
7. 遍历整个 asset registry，加载所有 UI kind artifact 构造 prototype store，再为每个 root 建树并
   执行 1280x720 初始 layout；
8. 扫描 world nodes 选择 orbit target，创建 camera/operation service，最后才返回句柄。

因此当前 `create_session` 是一个主线程可见的、工作量约为
`O(modules + project files/assets + scripts + scene entities + UI assets/nodes)` 的宽事务。它虽然把
脚本 discovery 的文件工作提交给 I/O pool，却马上同步等待每个 root；资源 scan/import、场景与 UI
装配也必须在 App 创建窗口之前全部完成。PERF-MVP-496/499 已记录 Runtime04 的多轮 inventory、
全量驻留和长 generation lock 根因，本报告不把它误写成 App 层局部锁优化。

插件侧还存在启动重复投影：`LinkedRuntimePluginPlan::prepare` 先 clone manifest，并对每个 registration
线性检查已有 selection，随后 builtin module assembly 和新建 catalog 各自再消费同一 registrations。
其成本需要并入 PERF-MVP-427 的 single generation-owned startup artifact；不能在 App 或 Runtime
再加一份 catalog cache。

## Ready 合同与测量盲区

App 在 `RuntimeSession::create_with_profile_and_project` 返回后立即发送
`phase=ready stage=runtime-session-create`，但此时 `RuntimeEntryApp` 尚未构造，winit 尚未进入
`resumed`，window/surface 尚未创建或绑定，更没有首帧 present。Editor 若把它当作可交互 Ready，
会系统性低估 start-to-first-frame，并在实际 surface/pipeline 初始化仍在主线程时放行用户操作。

profile capture 又在 DLL load 前启动，却只在 `event_loop.run_app` 正常返回后导出。DLL load、event-loop
创建或 session 构造任一早退都会丢失启动 capture；CLI 与 project-root resolve 发生在 capture 前，
也不在冷启动数据中。Runtime 内已有若干嵌套 `profile_scope!`，但缺少稳定的跨 ABI stage id、
wall/CPU/I/O/worker overlap、资源规模与 first-present 关联，失败路径也没有 RAII/finally export owner。

Ready 必须拆成至少三个有序事实：`session-created`（ABI/lifecycle owner 已建立）、
`surface-ready`（首个 MVP window/surface 可提交）、`first-presented`（首帧成功展示）。Editor Play
默认等待 `first-presented`；headless 只等待其明确 profile 所需的 service-ready 集合，不能伪造 surface。

## 销毁链路

App 当前已修复旧 PERF-MVP-574 的两项静态问题：owned ABI output 在 error 后也 exactly-once release；
`destroy_session` 失败不再 `mem::forget`，而是记录诊断并在 DLL unload 前 fail-stop abort。这个边界比旧
quarantine 正确，但尚无时延预算。

Runtime destroy 会先关闭新 wake entry，然后无 deadline 地等待 active actions 与 in-flight wake
callbacks；接着同步断开 plugin event mirrors、停止 project watchers、关闭 dynamic process log，
最后才移除 session 并允许 DLL drop。任一 foreign action/callback 或 watcher join 卡住，App 就永远
拿不到失败 status，现有 abort 分支也不会执行。当前没有每阶段 wait count/time、oldest action、
callback owner、watcher count 或 teardown deadline 证据。

正确方向不是超时后强行卸载 DLL，而是版本化 shutdown 协议：先 `quiesce` 禁止新工作并广播 cancel，
再按 owner drain（actions/wake -> mirrors/watchers/jobs -> process log -> session -> DLL），每阶段输出
progress 与 deadline；不能证明 callback/worker 已停时仍必须 fail-stop。该方案归 Runtime10/11 与
Plugins01，共享 owner 不得被 App 私有线程或 detach 绕开。

## 本地参考源码结论

- Unreal `LaunchEngineLoop.cpp:3469-3542` 把 PreInit 拆为 graphics/modules、mount、localization、
  asset registry/UObject、preload/render thread、remaining modules 和 finalize；每阶段有
  `SCOPED_BOOT_TIMING` 与失败边界。`4880-5108` 又把 engine create/init、PostEngineInit modules、
  `GEngine->Start`、loading-screen completion、heartbeat 和 init-complete 分开，最终日志才定义 engine
  initialized。它不支持用“session object 存在”替代“产品可运行”。
- Unreal AssetRegistry `AssetRegistry.cpp:1882-2035` 显式区分 async search 与同步
  `WaitForCompletion`；`AssetDataGatherer.cpp:1945-2048, 2361-2535, 4302-4470, 4954-5079` 用专用
  discovery/gather threads、background parallel directory scan 和并行 file-read scheduler，并将结果
  分批交给主线程。它支持“后台准备 + 有界主线程提交”，不是把全资产 import 放进启动临界区。
- Unreal `LaunchEngineLoop.cpp:5120-5320` 在 module unload 前先停新工作、两次 flush async loading、
  停 streaming/input/render/PSO work，再逆序 shutdown modules。它证明 unload 前必须有显式 quiesce/
  drain 顺序；Zircon 的 fail-stop 不能删除，但需要把无界黑盒等待变为可观测阶段。
- Bevy `plugin.rs:53-79` 与 `app.rs:232-294` 把异步插件 setup 的 `ready`、全插件 `finish` 和
  `cleanup` 分成状态机；`AssetServer` 在 `server/mod.rs:540-607` 通过 IoTaskPool 启动 load，调用者
  持 handle 并读取 `LoadState`/dependency state（1227-1345）。这支持按 MVP working set 发布 handle
  与 readiness，而非启动时驻留整个项目。

参考引擎提供的是阶段、owner、异步准备和有序 drain 原则，不给出 Zircon 的最终毫秒阈值；阈值必须
由同机、同配置、同工作集的 current-source profile 决定。

## 结构性优化计划与责任回传

| owner plan | required change | acceptance evidence |
| --- | --- | --- |
| Runtime04 | 延续 PERF-MVP-496/499：manifest/catalog metadata generation 可先发布；asset import、artifact decode、UI prototype 和非 MVP 资源按 working set/依赖异步 single-flight，短 CAS 发布 last-good | 1/1K/100K assets cold/warm/1% change：directory/stat/read/hash/import、project-lock wait/hold、resident bytes、first-present wall；warm unchanged I/O=0，首帧驻留近 MVP closure |
| Runtime10 | 新增 versioned startup milestone/status API 与 staged shutdown，不修改冻结 V1/V3 含义；失败路径 capture 必须 RAII 导出 | session/surface/first-present 顺序准确；每阶段 wall/CPU/error；startup 任一点失败仍有 E/F 盘 artifact；destroy 阶段、active owners 和 deadline 可见 |
| Runtime11 | 用现有 runtime-owned bounded pools 承接 inventory/import/script/UI prepare；定义 entries/bytes/age/concurrency/cancel/shutdown fence，不新建 App 私有 pool | workers 1/4/16、slow I/O 0/10/100 ms：main-thread blocked time、queue peak/age、worker overlap、cancel/drain；队列硬有界且关闭后无后台 callback |
| Plugins01 | 单个 generation-owned compiled plugin plan 同时供 module/extension/host 使用；插件 async setup 采用 ready/finish，unload 进入 quiesce/drain | plugins 0/1/100/1K：manifest/catalog builds、clone bytes、registration visits、setup overlap、callback drain；build <= 1/generation，卸载后 callback=0 |
| App/Editor F0 | Play reporter 等待 profile 对应 milestone；启动 capture owner 覆盖 CLI/project/DLL/session/window/surface/first-present/teardown，并保留原始失败 | runtime/editor cold+warm 各 5 次，报告 start-to-session、surface、first-present、ready-to-input p50/p95、CPU time、peak WS、I/O、thread peak；报告写入 E/F 盘 |

禁止把 `create_session` 简单搬到任意 worker 后直接触碰 winit/window/RHI owner；禁止并行运行有依赖的
module activation、project authority commit 与 world apply；禁止为加快 Ready 而跳过错误、依赖或
rollback。先把纯 I/O/parse/compile 形成 immutable candidate，再在 owner 线程短提交。

## 动态验收矩阵

1. Project：0/1K/100K files，0/1K/100K assets，cold/warm/1% change，脚本 roots 0/1/16，UI roots
   0/1/16、UI assets/nodes 1/1K/100K；记录各 stage、files/bytes、alloc/RSS、锁与 worker overlap。
2. Plugin：0/1/100/1K registrations，required/optional failure，setup 0/10/100 ms；记录 plan builds、
   selection visits、clone bytes、ready/finish wall 与 first-present 影响。
3. Failure：DLL missing/symbol/ABI/project/scan/script/scene/UI/surface/present 各阶段注入；每次都保留
   E/F 盘 capture、原始错误和已完成 stage，不遗留 session/watcher/thread。
4. Teardown：active action/wake/plugin mirror/watcher/job 各 0/1/100，callback 0/10/100 ms 与 hang；
   记录 quiesce/drain wall、deadline、abort/fail-stop，证明 DLL unload 前 callback/worker=0。
5. 工具：WPR/xperf 记录 CPU sampling、DiskIO、FileIO、CSwitch/ReadyThread、working set 与 power；
   Tracy/内建 counters 对齐 stage id。RenderDoc 仅在 surface-ready 后捕获首帧/稳定帧，不用于解释
   DLL/project/script 启动 CPU 根因。

当前结论为 `static_complete_dynamic_pending`，所以 App entry 总目录仍留在 `pending.md`，`review.md`
不增加任何文件。未取得 current-source managed build、WPR/Tracy 数值和 teardown failure matrix 前，
不得提交性能优化里程碑或发送“瓶颈已消失”的企微结论。
