---
related_code:
  - zircon_app/src
  - zircon_runtime/src/core
  - zircon_runtime/src/scene
  - zircon_runtime/src/render_graph
  - zircon_runtime/src/graphics
  - zircon_editor/src/core
  - zircon_editor/src/scene
  - zircon_editor/src/ui
  - zircon_plugins
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/02-unreal-aligned-engine-system-hard-cutover.md
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
doc_type: milestone-detail
status: current
---

# Unreal 对齐系统架构源码基线（2026-08-15）

## 范围与方法

本记录在开始结构优化前冻结参考行为。逐段阅读本地 `dev/UnrealEngine` 中 TaskGraph、World tick、RDG、GPUScene/InstanceCulling、Slate invalidation、TabManager、ModuleManager 和 PluginManager；同时对照已完成的 Zircon scene/render/editor 当前源码报告。这里只提取 owner、lifecycle、thread、change propagation 和复杂度行为，不以 Unreal 的 C++ 类型名直接设计 Rust API。

当前工作树无 current-source 可运行产品 binary，故本记录是架构和复杂度基线，不是动态性能验收。旧 `target/profiling/zircon_editor.exe` 不进入证据。

## Unreal 源码锚点

| 子系统 | 源码锚点 | 直接行为证据 |
|---|---|---|
| TaskGraph | `TaskGraph.cpp:722-810,1114-1132,1374-1415` | named thread 队列空闲时等待事件；worker 从共享调度寻找任务；enqueue 区分本线程/跨线程并保留目标线程；worker 数由共享 scheduler 提供 |
| World | `LevelTick.cpp:1504-1524,1730-1888` | `UWorld::Tick` 是帧权威；空 level collection 早退；`StartFrame` 后按 PrePhysics/StartPhysics/DuringPhysics/EndPhysics/PostPhysics/PostUpdate/LastDemotable 推进，`EndFrame` 封口；DuringPhysics 明确不阻塞 |
| RDG | `RenderGraphBuilder.cpp:1341-1425,1780-2023` | execute 前 compile 依赖与 reference count，以输出/外部资源/NeverCull 为根做 pass culling；barrier/resource/view 准备通过 setup tasks 并行，最后再 execute passes |
| GPUScene | `GPUScene.cpp:840-978,1832-1842` | full upload 只在配置/资源布局 generation 改变或显式开关时发生；稳态 filter 只含 `GPUSceneDirty` 与 added primitives；upload 进入 RDG |
| Instance culling | `InstanceCullingContext.cpp:716-1074,1127-1489` | draw command build 支持 synchronous/deferred-or-async，合并 batch 后用 compute/indirect args 输出；culling/compaction 是 render graph 的明确阶段 |
| Slate invalidation | `SlateInvalidationRoot.cpp:288-340,356-440,1281-1405` | root layout 才设置 slow path；普通 widget reason 进入 unique heap/list；fast path 只处理 invalid widgets，发生根条件时清空 fast data 并可观测地退化 |
| Tab manager | `TabManager.cpp:1711-1823,2630-2647,2731-2755` | invoke 先找可复用 live tab；找不到才恢复/spawn；`OnSpawnTab` 仅在实例确实缺失且允许 spawn 时执行 |
| Module manager | `ModuleManager.cpp:980-1065,1316-1405,1968-1985` | load 先走 existing-module 快路；单一 manager 管 load/unload/abandon 和 change event；pending static initializers 消费后清空 |
| Plugin manager | `PluginManager.cpp:2034-2085,2884-2994,3336-3362` | configure 只在 pending plugins 存在时发现/标记/处理/挂载并清空；module load 按 phase 单向推进，禁止回退/跳 phase；显式插件按需 mount |

## 对 Zircon 的结构性判定

| 判定 | 当前证据 | 结论 |
|---|---|---|
| 三根系统 owner 必须固定 | workspace 虽有 interface/RHI/宏支持 crate，但产品入口、runtime authority、editor authoring 可清晰归属 app/runtime/editor | 支持 crate 只能实现叶子 ABI/后端，不得拥有平行生命周期或数据树 |
| 当前局部 cache 不能替代统一架构 | scene extract 存在多入口、World clone、DTO clone 与 sideband/多相机漂移 | 先建立单一 World/extract generation，再优化 consumer |
| 当前 render 子系统存在重复 materialization | 已审查 GPUScene、mesh draw、graph execution、history、HZB 显示 graph 后复制、逐 mip staging 和重复 owner | 目标是单一 RDG/GPUScene/visibility 管线，不保留双图/双 history owner |
| retained-host 局部修复不足 | scoped view 仍扫描窗口/浮动行；full recompute 调用所有 enabled plugin sources，只消费 visible 子集 | 需要 stable proxy、visible-demand index 和 live tab reuse，不是继续给全量 projection 加分支 |
| plugin callback 必须离开锁和无预算主线程 | targeted source 已锁外调用，但 full path 仍同步调用 S 个外部 source，接口无 generation/deadline/affinity | plugin data 必须由 handle/generation/NotModified 和 bounded scheduler 管理 |
| 动态结论当前不可成立 | managed app build 324.2 s 后因 6 个 foreign runtime errors 失败；focused runtime lib-test 843.4 s 后编译失败，均 0 tests | 先完成 M0 current-source product 恢复，再运行 WPR/Tracy/RenderDoc/功耗 |

## 算法与工程目标

- Scheduler：注册或依赖 generation 变化时构图，frame 执行近 `O(ready tasks + dependency edges)`；空闲阻塞，不 busy poll。
- World/ECS：query 与匹配 archetype/chunk 成比例；mutation 进入 dense deferred buffer；derived state 与 extract 和 changed set 成比例。
- RDG：compile 与 passes/resources/edges 成比例，steady generation 复用；pass culling 只执行可达输出子图；transient resource 按 lifetime alias。
- GPUScene/visibility：upload 与 added/changed/removed ranges 成比例；instance culling/compaction 批量化；稳定帧全量 upload 为 0。
- Editor：widget work 与 dirty proxies 及受影响 ancestors 成比例；slow path 只由 typed root reason 触发；tab/pane payload 与 visible cache misses V 成比例，不与 enabled sources S 强绑定。
- Plugin：discovery/parse/index 每 catalog generation 一次；stable call 不全局 mutex 串行；callback work 有 count/bytes/time/age 硬预算。

这些是规模合同而非凭空的纳秒目标。绝对预算在 M0 产品基线恢复后，以同一硬件、场景和 source fingerprint 冻结。

## 当前处理决定

- 已完成的 temporal velocity load/store 正确性修复与 render-only invalidation 分离属于目标架构内的叶子修复，保留。
- 未冻结目标 owner 前，不再对会被 hard cut 的全量 projection、平行 registry、adapter 或第二数据树做局部缓存堆叠。
- `review.md` 继续保持未接收；本记录、Plan02 和所有静态报告均留在 pending，直到相应 current-source 产品与动态门通过。
