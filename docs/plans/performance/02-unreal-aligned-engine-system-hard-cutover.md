---
related_code:
  - Cargo.toml
  - zircon_app/src
  - zircon_runtime/src/core
  - zircon_runtime/src/scene
  - zircon_runtime/src/render_graph
  - zircon_runtime/src/graphics
  - zircon_editor/src/core
  - zircon_editor/src/scene
  - zircon_editor/src/ui
  - zircon_plugins
  - zircon_runtime_interface/src
related_tests:
  - docs/plans/performance/review.md
  - docs/plans/performance/pending.md
  - zircon_runtime/src/tests/runtime_absorption
  - zircon_editor/src/tests
  - tests/acceptance
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/minimum-viable-engine-foundation.md
  - docs/plans/mvp/index.md
  - docs/plans/zircon_runtime/runtime/02-core-spine-and-root-surface.md
  - docs/plans/zircon_runtime/runtime/03-schedule-and-frame-loop-alignment.md
  - docs/plans/zircon_runtime/runtime/06-plugin-surface-and-lifecycle.md
  - docs/plans/zircon_runtime/runtime/08-ecs-kernel-data-alignment.md
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
  - docs/plans/zircon_runtime/render/01-render-graph-rdg-alignment.md
  - docs/plans/zircon_runtime/render/03-gpu-scene-gpu-driven.md
  - docs/plans/zircon_runtime/render/04-visibility-culling.md
  - docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Private/Async/TaskGraph.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/LevelTick.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/RenderCore/Private/RenderGraphBuilder.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/GPUScene.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/InstanceCulling/InstanceCullingContext.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/FastUpdate/SlateInvalidationRoot.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Docking/TabManager.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Private/Modules/ModuleManager.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Projects/Private/PluginManager.cpp
  - dev/UnrealEngine/Engine/Source/Developer/HotReload/Private/HotReload.cpp
doc_type: implementation-plan
status: in_progress
gate: architecture-hard-cutover
last_refined: 2026-08-15
---

# Unreal 对齐引擎系统硬切换 Implementation Plan

> **For agentic workers:** 实施前必须使用 `zr-architecture-first-engineering`、`zr-reference-engine-routing`、`zr-hard-cutover-migrations` 与 `layered-milestone-development`；验证使用 `zircon-dev-validation`、`prefer-windows-validation` 和 `verification-before-completion`。

**Goal:** 不把当前临时功能结构当成终局，按 Unreal 已验证的系统所有权、生命周期、线程、世界更新、渲染抽取、编辑器失效和模块加载行为，重建由 `zircon_app`、`zircon_runtime`、`zircon_editor` 三根系统包组成的统一引擎；先恢复 F0/F2/F4 MVP 产品路径，再以同机动态数据证明结构瓶颈消失。

**Architecture:** `zircon_app` 只拥有产品入口、profile、窗口/进程主循环与 staging；`zircon_runtime` 通过内部 `core/{runtime,framework,manager,math,resource}` 吸收任务、模块、World/ECS、资源和渲染权威；`zircon_editor` 只拥有 authoring、workbench 和编辑器交互。`zircon_runtime_interface`、RHI 子 crate 与宏 crate只能是叶子传输/后端实现，不能成为第四套系统 owner。迁移采用同里程碑硬切换，禁止兼容 module、alias、旧 re-export、双写和双调度。

**Tech Stack:** Rust/Cargo、Windows managed validator、WPR/xperf/WPA、Tracy/Chrome profiling、wgpu timestamps/markers、RenderDoc 1.44、Unreal/Fyrox/Bevy/Graphics 本地源码。

---

## 1. 架构判定依据

详细源码锚点与结论见 [`02/2026-08-15-unreal-aligned-architecture-baseline.md`](02/2026-08-15-unreal-aligned-architecture-baseline.md)。当前硬切证据包括 [`02/2026-08-15-runtime-taskgraph-current-architecture-review.md`](02/2026-08-15-runtime-taskgraph-current-architecture-review.md)、[`02/2026-08-15-runtime-module-service-lifecycle-current-architecture-review.md`](02/2026-08-15-runtime-module-service-lifecycle-current-architecture-review.md)、[`02/2026-08-15-runtime-plugin-catalog-extension-bridge-current-architecture-review.md`](02/2026-08-15-runtime-plugin-catalog-extension-bridge-current-architecture-review.md)、[`02/2026-08-15-native-plugin-discovery-live-host-current-architecture-review.md`](02/2026-08-15-native-plugin-discovery-live-host-current-architecture-review.md)、[`02/2026-08-15-world-ecs-frame-extract-current-architecture-review.md`](02/2026-08-15-world-ecs-frame-extract-current-architecture-review.md) 与 [`02/2026-08-15-render-graph-current-architecture-review.md`](02/2026-08-15-render-graph-current-architecture-review.md)。本计划采用以下行为，不复制 Unreal 的 C++ 类型层级：

| 系统 | Unreal 源码行为 | Zircon 目标合同 |
|---|---|---|
| TaskGraph | 命名线程与 worker 队列分离，空队列阻塞，任务带线程归属和依赖 | 一个共享调度内核，显式 affinity/priority/dependency/budget；子系统不得私建无限线程或队列 |
| World tick | `StartFrame` 后按 tick group 推进，只有 DuringPhysics 显式允许异步跨组，`EndFrame` 封口 | World 是单一更新权威；stage barrier、deferred mutation 和 extract generation 明确 |
| RDG | 每帧图用稳定handle/registry和linear allocator表达；先依赖/裁剪/资源/barrier，再执行，资源setup/compile可并行，物理资源跨帧池化 | 稳定schema generation与轻量frame instance分离；稳态不重建schema/String DTO，逐帧finalize有线性预算，物理资源单一generation复用 |
| GPUScene/InstanceCulling | 稳态只消费 dirty/added change set，实例裁剪与 indirect command 在 GPU/RDG 中批处理 | immutable render extract + dirty range upload + GPU-driven visibility；禁止每帧全 World/全资源复制 |
| Slate/TabManager | stable proxy 的 unique dirty heap 走 fast path；根条件才 slow path；tab 先复用 live instance，缺失时才 spawn | stable widget/view handle、scoped invalidation、可见需求索引和实例复用；稳态 full recompute/spawn 为 0 |
| Module/PluginManager | 已加载 module 快路复用；加载/卸载有单一 manager；插件配置一次并按 phase 单向推进 | 单一 catalog/generation/lifecycle；发现、解析、能力验证与载入不在每帧或每 pane 重做 |

## 2. 当前结构性诊断

当前实现只作为瓶颈证据，不作为必须保留的 API：

| 域 | 已证据化问题 | 目标 owner |
|---|---|---|
| 调度 | runtime tasks、ECS schedule、asset pool、editor jobs 与插件 callback 各自形成队列/锁/指标，主线程可同步等待外部工作 | `zircon_runtime::core::runtime::tasks` 统一 scheduler；Editor 只提交 descriptor/handle |
| World/ECS | 更新、derived state、editor projection 和 render extract 存在多份 owner、全量 clone 与 adapter 分叉 | `zircon_runtime::scene` 单 World generation；命令缓冲提交后一次 extract |
| 渲染 | graph、compiled pipeline、GPUScene、mesh draw、visibility 与 history 有重复 materialize/copy owner | `zircon_runtime` 单 render-thread/RDG/GPUScene 管线 |
| 编辑器 | retained host 的 scoped path 仍扫描/clone 多个窗口；full recompute 同步调用全部插件 source | `zircon_editor` stable workbench/view registry + Slate-style invalidation；外部 payload 按 visible demand resolve |
| 插件 | discovery、manifest、registration、bridge、native callback 和 pane snapshot 缺单一 generation/affinity/budget | runtime module catalog + VM-oriented stable handle/capability/state migration |
| 产品测量 | 当前源码没有可运行的 current product binary，旧 target 不能代表当前实现 | `zircon_app` F0 build/stage/run 先恢复，再启用 CPU/GPU/功耗结论 |

## 3. 全局硬门

- [ ] 每个 subsystem 在实现前写清 descriptor、runtime handle/data、lifecycle、thread affinity、mutation boundary、extract boundary 和 diagnostics；缺任一项不得开工。
- [ ] 每类数据只有一个 owner 和一个 generation；consumer 使用 handle、borrowed view、immutable snapshot 或 delta，不复制第二套权威对象树。
- [ ] 共享任务系统提供 main/render/RHI/worker affinity、依赖、优先级、配额、取消、shutdown 和 queue-age 观测；私有线程池必须有不可共享的硬性理由与预算。
- [ ] 任何 foreign/plugin callback 不得在 World、EditorShell、message bus 或全局 registry 锁内执行；主线程 callback 必须由显式 affinity 声明和产品预算允许。
- [ ] steady state 的 full World extract、render schema/String rebuild、full GPU upload、full editor recompute、plugin rediscovery 和 tab respawn 均为 0；允许的逐帧frame-graph finalize必须近`O(P+A+E)`、使用复用arena且有counter，fallback必须带typed reason和计数。
- [ ] 旧 owner、旧路径及其测试在新 consumer 迁移的同一里程碑删除；禁止 compatibility shim、deprecated alias、转发 wrapper 和 legacy re-export。
- [ ] 源码复杂度结论必须给出输入规模和上界；性能结论必须给出 current-source 动态样本，不把单次 wall-clock 或旧 capture 写成验收。

## 4. M0 产品与测量基线恢复

### 实现切片

- [ ] 冻结 `Cargo.lock`、toolchain、GPU/driver、Windows 电源模式、分辨率、vsync、场景和 source fingerprint。
- [ ] 按 `mvp/00`、`mvp/01` 恢复 `zircon_app` runtime/editor current-source build、stage、首帧、退出与失败诊断；禁止以 library test 或旧 EXE 代替。
- [ ] 建立统一 frame/session trace schema：frame id、stage、thread/queue、World/extract/render generation、dirty counts、alloc/upload bytes、draw/pass、plugin callback 和 editor invalidation reason。
- [ ] 准备 F0 空启动、F2 最小场景、F4 打开/选择/修改/保存三套可重复脚本；所有输出位于 D/E 盘受管目录。
- [ ] 把现有 17,106 个 Rust 文件清单继续按模块保存在 `pending.md`；只有动态验收后的模块才进入 `review.md`。

### 测试阶段：M0 Baseline Gate

- [ ] 同一 validation copy 的 runtime/editor 均 build、stage、运行两次并干净退出，exit code 为 0。
- [ ] 三个产品脚本各运行至少 3 次，记录中位数、极差、CPU time、peak working set、I/O、线程峰值和失败率。
- [ ] WPR/xperf 能解析 CPU sampling、context switch/wait、disk I/O；profiling scope 能与 frame id 对齐。
- [ ] F2 捕获冷帧和第二稳定帧 RenderDoc，核对 marker、pass、draw/dispatch/copy、barrier、upload 与 GPU timestamp。

### 退出证据

- [ ] current-source 产品可运行，测量工具链不再被 stale binary 或 compile failure 阻断。
- [ ] 只冻结事实基线，不在 M0 宣称性能已接近其他引擎。

## 5. M1 Core/TaskGraph/Module 生命周期硬切

### 实现切片

- [ ] 由 Runtime02 定义唯一 `EngineRuntime`/service registry/module catalog/lifecycle phase，`zircon_app` 只驱动 phase，不持有 runtime service 副本。
- [ ] Runtime02/06 以 catalog generation 编译并原子发布唯一 module/service dependency graph；stable generation 的已 active module、重复 phase 与 registered handle 走 `O(1)` 快路，不 clone descriptor、不重编 topology。
- [ ] Runtime02/06 以 dense generational module/service slot 替换 String/HashMap/global-condvar steady path；name map 只服务 discovery，per-slot transition ticket/completion 定向唤醒 waiter，process-global service index 删除。
- [ ] Runtime06 把 module/service lifecycle 收敛为严格单向 phase 和唯一 transition owner；单 module、batch 与 lazy-service activation 共用 dependency closure，并把 `prepare -> quiesce -> approval -> cleanup -> publish -> retire` 作为唯一可卸载顺序。
- [ ] Runtime06/11 把 ready、factory、cleanup、destructor 与 observer 调度到声明 affinity 的 TaskGraph，使用 phase 绝对 deadline/cancel；main thread 禁止 1 ms sleep-poll，foreign destructor/callback 禁止在 registry lock 内执行。
- [ ] 由 Runtime11 把 runtime jobs、ECS tasks、asset work、editor background work 和允许调度的 plugin work落到一个 bounded scheduler；提供 named lane、worker lane、dependency、priority、cancel、panic 和 shutdown。
- [ ] Runtime11 以一个全局 worker budget 和共享 worker set 替换 Compute/AsyncCompute/Io 三个物理池；main/render/RHI 为 named affinity executor，navigation/renderer/plugin 不得创建第二套满规模 pool。
- [ ] Runtime11 把 keyed I/O 改为同 key 有序、异 key 有界并行的索引调度，移除 `VecDeque` middle-remove、全 suspended 扫描和 fence repeated scan 的平方级退化；timer control thread 不执行 foreign callback。
- [ ] Runtime11/Editor01 把退出收敛为有 deadline 的 quiesce/cancel/drain 状态机；main thread 和 guard Drop 禁止无界等待，超时必须返回 typed incomplete report。
- [ ] 由 Runtime03 编译 frame schedule；每帧只执行预编译 plan，registration/mutation 仅使 generation 失效一次。
- [ ] 由 Runtime06 建立 discover -> validate -> configure -> load phase -> active -> quiesce -> unload/rollback 单向状态机；stable generation 快路不扫描文件、不重绑 bridge。
- [ ] Runtime06/Plugins01 把 package/module/profile/capability/extension/bridge 收进同一 `EngineRuntime` catalog generation；app assembly、dynamic session、editor status 和 lifecycle observer 不得各建第二个 catalog、project-plan cache 或 bridge authority。
- [ ] Runtime06/11、Plugins01 与 Editor01/12/14 把 native discovery、DLL load、registration prepare、Play enter/exit、hot reload 和 retire 表达为同一 lifecycle ticket；共享 directory watcher 只合并 dirty key，不得每 plugin 私建 OS thread，main thread/Drop 不得同步等待 discovery、foreign callback 或无 deadline join。
- [ ] 硬删除重复 pool、detached spawn、平行 module registry、process-global service index、global resolution condvar、exact-one..five cardinality helper 和旧 lifecycle adapters；迁移所有 caller 后同步删除锁定旧源码形状的测试 fixture。

### 测试阶段：M1 Core Gate

- [ ] 1/2/N worker 与 1/1k/100k tasks 测 queue depth/age、steal、wake、wait、allocation、cancel、panic 和 deterministic shutdown。
- [ ] 实际 shared worker 总数不超过统一预算；16-thread 主机激活 navigation 不增加第二套 16-worker owner，1/2-thread host 的 report total 与实际 worker 数一致。
- [ ] 0/1/100k suspended、same-key storm、independent keys 和 fence chain 以 visits/moves/lock-hold counter 证明 coalesce/fence 无 `O(Q^2)`/`O(P^2)` 扫描。
- [ ] 两个无冲突任务并行，有冲突或 main/render affinity 的任务严格串行到正确线程；worker wait 不计为 main-thread wait。
- [ ] 0/1/100/1000 modules/plugins 的稳定 generation 重复访问不 discover/parse/load；phase 倒退、缺依赖和 unload-in-flight 返回 typed failure。
- [ ] stable generation 重复 activate/resolve 1,000 次时 topology build、descriptor clone、name hash 为 0；handle resolve 只做 `O(1)` slot lookup，catalog mutation 的 graph work 为 `O(M+S+E)`。
- [ ] 并发 activate 同 module 只执行一次 lifecycle；activate/deactivate overlap、observer reject、ready/factory/destructor slow/hung/panic、cancel/deadline 都得到确定 typed terminal，main-thread sleep 和 unrelated waiter wakeup 为 0。
- [ ] cleanup/destructor/observer/factory 的 callback-in-registry-lock 为 0；不可逆 cleanup 前完成 unload approval，deadline 到期后返回 retire/abandon report而不是伪装回 Active。
- [ ] 同一 registration batch 的 bootstrap/session catalog build=1；project-plan key含catalog/project-selection/target generation，同key single-flight、不同key不被全局mutex串行，catalog clone不清空已编译plan。
- [ ] 0/1/100/1000 native plugins 的 loaded/binding/bridge/replay/revision 只属于一个 public generation；不同 plugin 冷 build 不被全局 build mutex 串行，TOML parse/system prepare-under-loaded-lock=0，成功 batch publish=1、失败 publish=0。
- [ ] 1/100/1000 watched plugins 与 10k notification burst 下额外 worker thread 不随 plugin 数增长；watch callback 只 coalesce/enqueue，Play/hot-reload/unload/shutdown 的 main-thread filesystem/DLL/callback/join wait 为 0，并都有 typed deadline terminal。
- [ ] `git grep`/结构测试证明旧 pool/registry/compat path 为 0。

## 6. M2 World/ECS/Frame Extract 硬切

### 实现切片

- [ ] 由 Runtime08 先冻结 `WorldStorageGeneration`：entity/component/full-membership archetype/table schema/chunk/query/change-tick/command-lane共用dense slot；sparse-only变化不搬dense body，结构写按archetype range批处理。
- [ ] 由 Runtime03 建立`FrameScheduleGeneration`：UE-style tick group/stage barrier、dense system slot、access/affinity/dependency/command lane一次编译；stable frame不创建String key、不把system从registry移出/插回、不逐帧重建冲突batch。
- [ ] World mutation在stage边界以`prepare -> validate -> publish`至多发布一个immutable `WorldCommitGeneration`；其中包含added/changed/removed entity/component/resource/hierarchy ranges，observer/event/removed/derived state只消费该commit，禁止setter内递归发布或consumer各自重建影响集。
- [ ] 由Runtime07从同一commit更新camera-neutral `SceneRenderGeneration`，render freshness不再依赖宽`NodeCache`；Runtime10/Editor05共享单一`FrameExtractGeneration`协议，只投影per-view LOD/透明order/volume/visibility和animation/resource handle，不clone World或完整DTO。
- [ ] Editor authoring只通过typed command/facade修改Runtime World；Editor scene cache不是第二个运行时权威。Runtime11只执行上述compiled slots/ranges，不建立ECS私有调度器；旧move-out runner、五bool dirty、World/snapshot产品adapter在本里程碑硬删。

### 测试阶段：M2 World Gate

- [ ] 1/1k/100k entities、1/16/256/1k systems、1/8/31 components、1/8/256/4k archetypes/chunks、0/1/10/100% dirty与1/2/8 views下，分开记录plan build、key/hash、registry move、chunk/range、commit、query/derived/render visits、clone/alloc/lock和p50/p95/p99。
- [ ] 稳态frame的schedule rebuild/String key/registry remove-insert、World full clone、full derived rebuild、unchanged scene artifact和完整extract clone均为0；一个stage barrier的commit publish只能为0或1。
- [ ] 非冲突data-bearing Query/Res/event system在产品F2 trace中真实overlap；spawn/despawn、batch mutation、observer、removed window、undo/redo、scene load、multiple cameras和editor/runtime parity通过。
- [ ] 局部变更工作与affected chunks/ranges成比例；scene-global work不乘view count，非首相机LOD/透明order/volume/visibility正确；旧产品adapter和第二World/scene authority源码计数为0。

## 7. M3 Render Thread/RDG/GPUScene 硬切

### 实现切片

- [ ] 由Render01定义`RenderPipelineSchemaGeneration -> FrameRenderGraphInstance -> CompiledFrameGraphPacket -> TransientResourceGeneration`单一RDG；schema持interned dense pass/resource/executor slot，frame instance只承载generation handle、versioned write、root和动态extent。
- [ ] 以resource version生产者边自动导出RAW/WAR/WAW、culling、queue schedule、barrier和lifetime；同里程碑删除产品`previous -> pass`总序、executor-name拓扑特判、bare cross-graph index与旧String DTO/cache路径，不留兼容层。
- [ ] schema/compiled packet在framework state锁外构建并按generation原子publish；精确view/render extent不进入topology key，除非它真实改变pass集合。逐帧finalize使用可复用linear arena，Runtime11只在实测阈值以上执行parallel setup/compile task。
- [ ] Render01/RHI合并compile slot plan与physical pool authority：backend capability决定compatibility/alignment/fence/alias，记录requested/committed/aliased/create/reuse/evict bytes；不支持安全alias时保留可观测exact-descriptor fallback。
- [ ] Render17在compile时物化lint count与稳定stats，owned dump/lint rows仅显式capture/export构造；删除源码token测试，改为行为、复杂度counter、allocator和cross-generation rejection测试。
- [ ] 由 Render03 让 GPUScene 消费 `FrameExtractGeneration` 的 added/changed/removed range；稳定对象不重新编码、不重新上传。
- [ ] 由 Render02/04 统一 mesh draw command、visibility、HZB、instance culling 和 indirect draw；CPU 不为每 view/pass 重建完整 draw 列表。
- [ ] render thread/RHI submission 只接收 immutable packet；game/editor main thread 不等待 shader compile、readback、upload staging 或 foreign callback。
- [ ] 删除 graph 后第二套 history owner、逐 mip staging copy、重复 surface/tree materialization 和 legacy render adapters。

### 测试阶段：M3 Render Gate

- [ ] 1/32/256/1k passes、1/8/64 accesses、1/2/8 queue lanes/views、0/1/10/100% culled和1/1k/100k instances下，记录schema build、frame finalize、edge/version visit、ready width、String/hash/alloc、lock、setup task、resource bytes/fence、draw/dispatch/upload与CPU/GPU p50/p95/p99。
- [ ] 稳态schema build/String DTO rebuild/framework-lock compile/per-frame lint=0；frame finalize近`O(P+A+E)`且复用arena；independent async pass ready width>1并有CPU/GPU overlap；dead overwritten producer可cull，cross-generation handle被拒绝。
- [ ] warmup后stable physical create=0且unchanged GPUScene upload=0；resize只rematerialize受影响extent一次而不重建不变topology；pipeline/shader generation变化恰发布一次完整新代，失败继续旧代。
- [ ] RenderDoc 验证 cold/steady capture 的资源、barrier、load/store、copy 和 indirect draw；GPU timestamp 与 capture pass 对齐。
- [ ] 60 Hz 观察线下先消除结构性主线程/GPU stall；画质、遮挡正确性、velocity/history 和多相机行为不得退化。

## 8. M4 Slate-style Editor/Workbench 硬切

### 实现切片

- [ ] 由 EditorUI08 建立 stable widget/view/tab proxy、unique dirty set、pre/layout/post/paint reason 与 root slow-path reason；pointer/hover/paint 不提升为全局 layout。
- [ ] Workbench registry 持有 live tab/view instance；invoke 先复用，只有缺失且可见需求成立时才 spawn pane/template/source。
- [ ] layout、presentation、hit-test、paint、native-window sync 使用同一 generation-owned projection；按 dirty scope patch，不 clone 全窗口/全 floating rows。
- [ ] plugin pane 发布 visible demand index；只 resolve V 个 visible handles，支持 `NotModified`、data generation、estimated bytes、deadline、cancel、last-good 和明确 affinity。
- [ ] Editor14 把后台任务交共享 Runtime scheduler；main thread 每 tick 只消费 count+bytes+deadline 有界结果页。

### 测试阶段：M4 Editor Gate

- [ ] 0/1/100/10k widgets/tabs/windows 与 1M pointer events 下记录 slow-path、dirty visits、clone bytes、layout/paint、native OS calls 和 input p95。
- [ ] 空闲 30 s 记录 CPU、wakeups、recompute、redraw、plugin callback 和功耗；无输入/无数据变化时均为 0 或有逐项豁免。
- [ ] 打开/复用/浮动/关闭/恢复 tab，选择/修改/保存/undo/redo，DPI/resize 与多窗口交互通过。
- [ ] full recompute 只由根结构/布局 generation 触发；render-only 不进入 host recompute，visible V 不扫描/调用全部 S 个 plugin source。

## 9. M5 VM-oriented Plugin 系统硬切

### 实现切片

- [ ] 单一 package/module catalog 在 manifest/discovery generation 变化时构建一次；profile、capability、dependency、loading phase、registration、extension owner range 和 bridge slot 共享 interned dense index。
- [ ] project plan 以 `{catalog generation, project-selection generation, target}` 为键在publication lock外构建；同key single-flight，不同project/session不以target-only entry互相驱逐。
- [ ] extension generation 以owner->slot ranges/affected-edge index和RCU/COW发布替换全registry thaw/rebuild；handle携带catalog/entry generation，owner unload工作只与自身贡献和受影响依赖成比例。
- [ ] public plugin contract 使用 versioned descriptor、generational handle、capability、stable value/byte buffer、thread affinity 和 state migration；禁止跨动态库共享 Rust 对象、trait object 或锁。
- [ ] stateless callback 走 immutable callable/handle；stateful callback 每 instance 独占状态；reload 采用 prepare -> quiesce -> migrate -> publish 或 rollback。
- [ ] plugin 工作按 CPU time、entries、bytes、queue age 和 callback deadline 受控；只有明确 pure/non-main work 可进入 worker lane。
- [ ] native bridge 只作为受限 host adapter，不能形成第二套 catalog/module/lifecycle/scheduler owner；owner启停不得全表扫描，debug diagnostics不得让每次bridge call固定争用共享原子。
- [ ] native backend 只保存当前 plugin slot 的 immutable ABI/command/bridge/registration/state generation；loaded、binding、replay、revision 不得作为可独立发布的平行 registry，load batch 必须 stage/validate 后一次原子发布。

### 测试阶段：M5 Plugin Gate

- [ ] 0/1/100/1000 plugins、systems、pane sources 和 bridge calls，1/16 threads 下记录 discovery/parse/build counts、锁等待、callback time、queue bytes 和 RSS。
- [ ] stable generation 的 rediscovery/reparse/rebind/full snapshot 为 0；visible V 个 pane source 的 callback 次数等于 cache miss V，不等于 enabled S。
- [ ] app bootstrap、dynamic session、profile availability 和editor status共享同一catalog/plan identity；稳定查询projection/extension registry rebuild=0，两个同target不同project并发不cache thrash。
- [ ] owner贡献k/全registry N的卸载访问量近`O(k+affected edges)`，unrelated family thaw/scan/sort=0、bridge owner full scan=0；1/16 threads x 1M bridge calls无global lock/name hash和每call debug shared RMW。
- [ ] native batch在任一 candidate/ABI/registration/reload 失败时 public generation不变；并发读者只能看到完整旧代或完整新代。不同 plugin cold build有界并行、同slot single-flight，stable代重复查询parse/build/invalidation=0。
- [ ] capability denial、stale handle、panic/trap、timeout、reload-in-flight、state migration failure 和 rollback 全部通过且无 deadlock/UAF。
- [ ] `git grep`/ABI tests 证明 direct Rust object sharing 和旧 bridge registry 快路为 0。

## 10. M6 产品收敛与量化验收

### 实现切片

- [ ] 由 `zircon_app` 固定 runtime/editor/server profile 和 staged product manifest；移除临时入口、重复 bootstrap/catalog assembly 和产品路径分叉。
- [ ] F0、F2、F4、F5 使用同一 source fingerprint、硬件和脚本重复运行，关联 CPU/GPU/内存/I/O/功耗与功能结果。
- [ ] 在相同机器、分辨率、vsync、场景规模和画质条件下，若可构建参考引擎最小场景，再做归一化对照；否则只报告 Zircon 前后数据和源码复杂度，不伪造“接近 Unreal”百分比。
- [ ] 每个已通过里程碑形成 scoped commit；提交后发送企微量化消息，包含 commit、样本、前后数据、门禁结果与残余风险。

### 测试阶段：M6 Product Gate

- [ ] F0 启动/退出、F2 连续 300 帧、F4 空闲 30 s 与作者操作脚本各至少 3 次，报告 median/range 与 p50/p95/p99。
- [ ] CPU 主线程、worker 利用率/等待、peak RSS、I/O、GPU pass/draw/upload、能耗/平均功率数据绑定同一 run id。
- [ ] 所有 P0 模块必须完成逐 `.rs` 复审、静态扫描、focused/产品测试和动态 profile 后，才按模块移动到 `review.md`。
- [ ] 独立 code review 无 Critical/Important；全局文档/源码结构/格式门通过，无旧 owner 和兼容路径。

## 11. 跨计划责任路由

下列编号计划必须把本计划的硬门纳入自己的实现和验收，不得继续以临时局部修复关闭性能问题：

| 责任计划 | 必须承接的结构任务 |
|---|---|
| Runtime02/03/11 | 单一 runtime spine、compiled frame schedule、共享 TaskGraph 与 thread/queue diagnostics |
| Runtime06/10 | module/plugin 单向 lifecycle、stable ABI handle、generation 快路和 host/runtime 边界 |
| [Runtime03](../zircon_runtime/runtime/03-schedule-and-frame-loop-alignment.md) / [Runtime07](../zircon_runtime/runtime/07-runtime-performance-hotpath.md) / [Runtime08](../zircon_runtime/runtime/08-ecs-kernel-data-alignment.md) / [Runtime10](../zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md) / [Runtime11](../zircon_runtime/runtime/11-job-system-task-model.md) | PERF-MVP-604..620/632：compiled schedule slot、chunk storage、单一World commit、derived/observer ranges、scene/extract generation与TaskGraph执行 |
| Render01/02/03/[04](../zircon_runtime/render/04-visibility-culling.md)/[07](../zircon_runtime/render/07-postprocess-color-pipeline.md)/[12](../zircon_runtime/render/12-effects-particles.md)/[17](../zircon_runtime/render/17-performance-and-profiling.md) | PERF-MVP-620/632/633：generation/version RDG、锁外schema compile、linear frame instance、fence-aware transient authority、mesh draw/GPUScene、camera-neutral scene/per-view visibility/post/particle及RenderDoc/timestamp/预算 |
| Editor01/02/[05](../zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md)/12/14 | runtime facade、消息页、authoring transaction、plugin lifecycle ticket、共享watcher/scheduler、FrameExtractGeneration consumer和主线程预算 |
| EditorUI02/08 | stable proxy、scoped invalidation、workbench live instance、presentation/hit-test/paint generation |
| [Plugins01](../zircon_plugins/01-plugin-architecture-core.md)/09/12 | catalog、VM ABI、capability/state migration、visible demand、bounded callback；observer/plugin event只能消费WorldCommitGeneration或其稳定ABI投影 |
| MVP00/F0/F2/F4/F5 | current-source product build/run 与同机功能、性能、功耗终验 |

跨 owner 问题以 failure handoff 链接本计划及对应源码报告；本计划保持总架构和横向数据权威，各编号计划拥有具体实现。

## 12. 完成定义

- [ ] M0-M6 均完成各自测试阶段，且状态表有 source-bound evidence。
- [ ] 三根系统包 owner 清晰，支持 crate 不拥有平行生命周期、World、scheduler、render graph 或 editor model。
- [ ] F0/F2/F4 MVP 的结构性 P0 瓶颈消失，复杂度门和动态数据同时成立。
- [ ] RenderDoc/WPR/Tracy/功耗证据来自 current-source 产品，不来自旧 capture、fixture 或 source model。
- [ ] 所有旧路径在 hard cut 同里程碑删除，无兼容层延长双系统寿命。
- [ ] 每个已验收模块已从 `pending.md` 按模块移入 `review.md`；未动态验收的静态审查保持 pending。
- [ ] 每个 accepted milestone 已提交独立 commit，并在提交后发送企微量化摘要。

## 状态与产出记录

每个里程碑测试通过后只记录一次 accepted outcome；静态审查、失败尝试和局部实现不单独写入本表。

| 里程碑 | 范围 | 状态 | 完成日期 | 验证批次 / 量化结果 / 残余风险 |
|---|---|---|---|---|
