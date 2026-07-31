---
related_code:
  - zircon_runtime/src/render_graph
  - zircon_runtime/src/graphics/pipeline/compiled_graph_cache.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/realtime_ibl_runtime.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/render/01-render-graph-rdg-alignment.md
reference_sources:
  - dev/godot/servers/rendering/rendering_device_graph.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/RenderCore/Private/RenderGraphBuilder.cpp
tests:
  - zircon_runtime/src/render_graph/tests
  - current-source Windows zircon_runtime render_graph tests pending
  - current-source RenderDoc graph dump/marker/pixel comparison pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# RenderGraph逐文件性能静态审查（2026-07-18）

## 范围与覆盖

`zircon_runtime/src/render_graph`当前源 **14/14** 个Rust文件、**3,767** 行已逐文件重读：builder/compile、compiled graph与transient allocation、dump/types/error/root，以及全部测试。`builder/compile.rs`与`graph.rs`当前仍有其他会话写租约及未提交改动，本切片读取当前源但不覆盖其改动；文件数保持14，行数随readback lifetime、typed lookup indices和allocation-plan缓存实现增长。

## 热点与直接修复

- 每次`read_*`/`write_*`都调用`ensure_resource`，原实现线性扫描全部resource declarations，authoring总成本可达O(accesses × resources)。PERF-MVP-224已按typed handle单调上界改为O(1)，并加越界行为测试与源码复杂度守卫。
- 当前`CompiledRenderGraph`已为pass、resource declaration/lifetime和pass-resource access建立HashMap索引，并把transient allocation plan从每次查询重建改为构造期缓存；这些改动消除了执行/诊断消费者的线性handle lookup及重复alias-plan build。索引仍通过resource name String反查handle并复制名称，后续dense handle应直接贯通compiled access，避免String成为内部join key。
- `manual_reachability`在intermediate×source循环中clone/extend HashSet；多writer验证再做writer pair比较，`validate_reads_have_ordered_producers`对每个read全扫pass/access。插件/feature扩展图会放大为超线性CPU与分配。
- `cull_passes`为每个pass临时collect writes；transient bucket/slot计划使用多处Vec线性find/position。compile与alias owner正被Render01修改，本切片只交接指标和验收，不与活跃实现竞争。
- `CompiledRenderGraph`不可变且主pipeline已有cache，但`update_base_stats`每帧调用`graph.stats()`，后者多遍扫描passes/resources；该统计应在compile时预计算。realtime IBL每个有工作batch仍重新build/compile小图，稳定拓扑应按request/operation signature复用。
- `RenderGraphDump`的clone/format和allocation name查找只在显式capture/diagnostics路径，未发现默认每帧调用，不把诊断成本误列为F2 blocker。

## 参考引擎对照

Godot `RenderingDeviceGraph::end`使用复用的thread-local vectors、显式adjacency与degree数组完成拓扑遍历，之后按level/priority排序并批量处理barrier；Unreal RDG为compile建立专用cycle/CSV计时，预留pass容量，以producer DFS做root culling，并在资源规模超过阈值时并行setup/compile。Zircon MVP不需要立即复制UE的全部并行RDG，但必须先消除HashSet clone和重复全图扫描、缓存immutable stats，并用规模基准决定何时值得并行。

## 动态验收

待受管Cargo运行全部render_graph测试与新增builder守卫；构造16/64/256/1024 pass×resource的chain、fan-out、multi-writer与plugin-heavy图，记录compile p50/p95、visited edge/access、HashSet/Vec allocation和复杂度斜率。另记录compiled lookup probes、allocation-plan builds和内部name-clone bytes：同compiled graph allocation-plan build=1，查询重建=0。稳定main scene记录cache miss=0与stats pass/resource visited=0；稳定realtime IBL topology记录compile=0。最后用current-source RenderDoc对拍graph dump、non-culled markers、pass/resource数量和像素结果；完成前保持`pending.md`，不进入`review.md`。
