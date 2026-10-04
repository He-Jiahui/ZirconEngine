---
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources
  - zircon_runtime/src/graphics/scene/scene_renderer/temporal/taa/taa_resolve_bind_group_cache.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/generic_compute_executor.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/render/01-render-graph-rdg-alignment.md
  - docs/plans/zircon_runtime/render/07-postprocess-color-pipeline.md
  - docs/plans/zircon_runtime/render/08-material-shader-permutation.md
  - docs/plans/zircon_runtime/render/17-performance-and-profiling.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/RenderCore/Public/RenderGraphUtils.h
  - dev/UnrealEngine/Engine/Source/Runtime/RenderCore/Public/ShaderParameterStruct.h
  - dev/UnrealEngine/Engine/Source/Runtime/RHI/Public/PipelineStateCache.h
tests:
  - current post_process resources slice 142 of 142 Rust files reviewed, 9450 lines, 100 inline tests
  - scoped rustfmt 142 of 142 clean
  - current-source Windows Cargo, F0 and F2 counters, WPR, GPU timestamps and RenderDoc pending
doc_type: implementation-evidence
status: static_complete_dynamic_blocked
---

# Graphics post-process resources current-source结构审查（2026-08-14）

## 当前范围与结论修正

`zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/**`当前物理清单142/142个Rust文件：9,450行、8,889个非空行、100条内联测试，fingerprint为`6F1E2B8BA479F52541A4B17EC9F76835FCB0441F3E909CAED2746ED1EB38617F`。本轮以2026-07-18已完成的construct/execute/root 140文件逐文件基线为起点，复读current inventory、全部current diff及新增/移动owner；22个modified和3个deleted状态项属于其他会话，本轮未改生产代码。142/142通过`rustfmt 1.8.0 --edition 2021 --check --config skip_children=true`。

current source有两项明确进展。第一，TAA resolve已有resource-identity keyed bind-group cache，瞬态texture复用会保持identity，稳定资源组合可复用bind group并记录实际create数。第二，专用SSAO layout、lazy pipeline、execute模块和registry入口已删除，SSAO改由graph metadata和`compute.generic`承接，减少一套重复GPU执行实现。后续不能再按旧“SSAO OnceLock”事实优化；其shader/schema/pipeline生命周期现在回链PERF-MVP-623。

## P0：TAA缓存仍是孤立例外

当前目录仍有26个`create_bind_group`源码调用点，分布于half-res transparency、bloom、cluster、exposure、LUT、DoF、HZB、motion vectors、FXAA/SMAA、SSR、upscale/output和通用29-binding post路径。实际每帧数量取决于compiled graph/effect开关，但除TAA外未见统一resource-generation bind bundle；多数路径每次execute直接create，即使texture/buffer identity未变。

参数更新同样分散：目录有13个`queue.write_buffer`调用点，默认post仍为每camera/pass创建参数buffer和29-entry bind group；terminal cache已持久化FXAA/output/SMAA部分region资源，active-prefix修复已避免零probe/cluster全容量上传，但没有形成所有effect共享的dynamic-uniform ring与binding cache。PERF-MVP-369/370应以TAA identity key为正确性模板，把所有effect收口到统一owner，禁止继续堆每效果私有Mutex/cache。

禁用效果仍可能通过clear pass写全尺寸目标；SMAA stage textures虽已有terminal cache，但仍在graph外拥有资源；auto-exposure与LUT generation、history和viewport resize必须进入精确失效key。目标不是让每个create快一点，而是让stable compiled post artifact的GPU对象创建为0、disabled node真正被graph cull。

## P0：构造期仍同步创建完整能力集合

`FullScenePostProcessResources::new`仍同步创建几乎全部内建bind-group layouts、buffers、fallback textures与pipeline bundle。共享`post_process.wgsl`的9个entry已保持一次source transform、一个shader module和一个pipeline layout，这是应保留的修复；但bloom、cluster、HZB single/MSAA、exposure、LUT、DoF、TAA、velocity、motion、SSR、upscale/output、FXAA与SMAA仍在renderer构造期创建，不以当前project/camera compiled artifact需求裁剪。

SSAO删除使专用pipeline数量减少，但其generic compute首次命中可能在submission路径Naga parse并create compute pipeline；这只是成本转移，不是异步prewarm。PERF-MVP-371与623必须共用Render08 typed pipeline/shader generation queue：F2必需集合加载期single-flight prewarm，可选效果按compiled post需求准备，ready前使用明确bypass/neutral/last-good，frame thread不得同步编译。

## Unreal Engine本地源码依据

- `RenderGraphUtils.h:451-563`的compute/RDG路径消费已解析shader ref、静态parameter metadata和parameter struct；执行只绑定参数并dispatch，不重新解释WGSL或资源schema。
- `ShaderParameterStruct.h:29-59,188-201`把bindings和metadata绑定到shader type/instance，支持generation级参数布局与资源绑定，而不是按pass String重建。
- `PipelineStateCache.h:145-185`区分PSO查找/创建并统计compute/graphics PSO hitch；Zircon的post constructor和generic compute必须把compile wall、queue latency与hitch变成一等数据。
- UE RDG使用pass parameter声明资源依赖并交由graph owner管理；Zircon的SMAA/terminal资源也应回到compiled graph/transient pool，而不是因有局部cache就保留第二套生命周期。

## 实施顺序与验收

1. Render07/17定义`PostBindingGeneration`：compiled physical resource identities、history/LUT/viewport/depth mode和effect feature mask共同形成精确key；统一dynamic-uniform ring与bind bundle，TAA迁入同一owner。
2. Render01把SMAA及所有中间资源纳入logical graph/pool，disabled effects在compile/cull阶段消失；Render05/18/plugin prepare发布已join的cluster/probe sideband。
3. Render07/08将pipeline bundle改为需求驱动的typed queue，Render01/08/Plugins01让generic SSAO使用同一shader/pipeline generation和bounded warmup。
4. Render17在current-source产品构建记录CPU/GPU对象、upload、pass、pipeline hitch与能耗，再用RenderDoc核对实际pass/draw/dispatch；不得以源码调用点数量冒充动态结果。

矩阵：effects none/minimal/all，AA none/FXAA/SMAA/TAA，history/LUT/depth raw/fallback，views 1/8，1080p/4K，lights/probes 0/1/max，stable/resize/camera/1% resource/shader change，cold/warm pipeline cache。记录buffer/texture/view/bind-group/layout/module/pipeline creates、params/probe uploads、binding rebuild/identity probes、clear/dispatch/pass数、pipeline queue/hitch、CPU p50/p95/p99、GPU timestamp、RSS与energy。

硬门：warm stable post buffer/texture/view/bind-group create=0，bundle build≤1/affected resource generation，params≤1 packed upload/camera frame；disabled clear/dispatch pass=0；SMAA backing由graph pool复用；optional pipeline不在F0构造，duplicate compile≤1，F2必需首帧ready且frame compile stall=0；SSAO generic stable include/hash/parse/pipeline create=0；pixels/history/AA/resize/device-loss语义等价。当前无current-source可运行产品二进制，managed build仍受共享foreign编译错误阻塞，WPR、GPU timestamp、energy与RenderDoc没有有效样本。本记录留在`pending.md`，不进入`review.md`。
