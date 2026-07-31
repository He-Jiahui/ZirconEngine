---
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/transient_resource_pool.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/materialization.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/materialization/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/transient_materialization.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/bind_compiled_scene_graph_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/submit_compiled_scene_frame.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/render/01-render-graph-rdg-alignment.md
  - docs/plans/zircon_runtime/render/17-performance-and-profiling.md
reference_sources:
  - dev/bevy/crates/bevy_render/src/texture/texture_cache.rs
  - dev/UnrealEngine/Engine/Source/Runtime/RenderCore/Private/RenderGraphBuilder.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/RenderCore/Private/RenderGraphResourcePool.cpp
tests:
  - transient materialization production slice six of six Rust files reviewed, 2588 lines
  - materialization alias sparse persistent mip and storage-view tests reviewed
  - production create bind release end-frame lifecycle traced
  - active Render01 RG-M2 source lease and frozen manifest respected; no source edit made by this review
  - current-source Cargo scale trace and RenderDoc pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Graphics瞬态资源池与物化链路逐文件性能静态审查（2026-07-18）

## 范围与结论

已完整阅读`transient_resource_pool.rs`、`materialization.rs`、`materialization/tests.rs`、`transient_materialization.rs`、`render_graph_execution_resources.rs`与产品binder当前6/6个Rust文件、2,588行，并追踪产品路径每帧`RenderGraphExecutionResources::new`、`begin_frame`、bind/materialize、release与`end_frame`。当前实现已经缓存compiled transient allocation plan、按生命周期槽复用物理WGPU资源，并把预算超限从重复全池最老项查找收敛为一次候选排序；但稳定compiled graph仍每帧重建槽位分组、名字映射和逻辑TextureView，物理资源池命中并未消除CPU分配与driver front-end工作。该剩余问题编号PERF-MVP-366。

## 已有止损与边界

当前RG-M2实现以完整纹理/缓冲描述符键复用物理backing，非重叠逻辑资源共享slot；persistent/imported/sparse资源绕过错误池化，SSR coarse资源保留父纹理mip alias。`TransientResourcePool::end_frame`先清除过期条目，再计算一次保留count/bytes；只有超预算才构造并排序候选，最终count/bytes直接进入report。相比旧的多轮统计/逐次最老项全扫，该路径已有明确止损，不重复创建新的“预算驱逐算法”根因。

本切片由活动会话`render01-rg-m2-transient-pool-hardcut-20260717`持有源码租约，且其RG-M2静态manifest已冻结等待受管验证。性能会话未越权修改源码；下面的follow-up应在RG-M2完成focused/broad/product/RenderDoc验收并提交后进入RG-M3，避免污染现有验证快照。

## PERF-MVP-366：稳定graph仍逐帧物化slot map、String映射与逻辑TextureView

`materialize_transient_texture_slots`与buffer对应函数每帧各创建一个`BTreeMap<slot, Vec<&lifetime>>`，重新遍历compiled allocation plan并为每个slot分配Vec。之后每个物理slot都`format!`生成`rg-transient-*-bucket-...-slot-...`名字；每个逻辑resource又clone name，并把backing name复制进多个`BTreeMap<String, ...>`。`RenderGraphExecutionResources`本身在产品帧中重新`default()`，因此这些map的节点与String容量无法跨帧复用。

纹理slot命中pool后仍为每个逻辑lifetime调用`Texture::create_view`。当多个非重叠逻辑纹理共享同一backing且使用默认mip-0 view时，物理texture count下降，但逻辑view创建、name map写入和WGPU对象前端调用仍按逻辑资源数发生。Bevy的`TextureCache`把`default_view`与texture一起驻留在cache entry；Unreal RDG则在compile/setup收集allocation/deallocation op、预留数组，并按资源生命周期把create/deallocate交给transient allocator，而不是在执行时重新按名字分组。

正确收敛方向是让`CompiledRenderGraph`持有immutable dense materialization plan：按physical slot预分组逻辑resource handles、预合并usage/descriptor、记录特殊mip alias，并把调试名字与热路径identity分离。执行期直接线性遍历physical slots，`RenderGraphExecutionResources`以resource handle索引的dense storage绑定资源；String map只保留在dump/diagnostic边界。pool entry同时保存可复用default view，只有新建backing或非默认mip/array view才调用`create_view`。execution workspace应按graph generation复用capacity或由frame arena提供，release只清长度与所有权，不重建树节点。

资源池维护的剩余两轮线性工作（stale retain后再做预算count/bytes）先以counter验证规模；若成为热点，可由stale traversal携带保留总量或维护增量count/bytes，确保常见未超预算帧每类pool最多一次完整遍历、候选分配/排序为0。该项不得牺牲`u128`精确累计、确定性oldest-first驱逐与现有饱和尺寸回归。

## 参考与验收

Bevy证明默认view可以与descriptor-keyed texture一同缓存，但其桶内线性找free entry和每帧全cache aging不应直接照搬；Zircon已有compile-time lifetime slots，应继续前移物化计划。Unreal的lifetime op、setup reserve、RHI transient allocator与trace stats说明编译计划、物理分配和诊断命名可以分层，Zircon不需要在每帧热路径保留String authority。

按logical resources 16/64/256/1024、physical slots 1/16/64/256、stable/1% changed、pool under/over budget记录slot-map builds、BTree/String/Vec allocations、logical-name clone bytes、`create_view` calls、pool entry visits、candidate allocations/sorts、created/reused/evicted backing与CPU p95。warm stable graph要求slot grouping build=0、backing-name format/String clone=0、workspace heap growth=0、默认view create=0；新建/特殊view调用只随物理backing或显式view descriptor变化。现有alias report、persistent/imported/sparse/SSR mip、预算与饱和尺寸语义必须等价。受管`materialization` focused tests、`render_graph` broad gate、F2产品trace与DX12 RenderDoc中physical backing小于logical resources且像素一致全部完成前，本切片保留在`pending.md`。
