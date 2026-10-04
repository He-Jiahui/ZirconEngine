---
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/realtime_ibl_runtime.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/realtime_ibl_time_slice.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/realtime_ibl_graph_plan.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/realtime_ibl_gpu_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/realtime_ibl_capture_wgpu.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/realtime_ibl_gpu_timestamps.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/realtime_ibl_wgpu_recorder.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/render/01-render-graph-rdg-alignment.md
  - docs/plans/zircon_runtime/render/11-environment-lighting.md
  - docs/plans/zircon_runtime/render/17-performance-and-profiling.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/ReflectionEnvironmentRealTimeCapture.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/ReflectionEnvironment.cpp
tests:
  - realtime IBL current Rust source thirteen of thirteen files reviewed, 2838 lines
  - bake-key reuse, allocation-free label projection and stack capture-uniform source guards RED then GREEN
  - existing scheduler, graph, WGPU ABI and timestamp tests reviewed
  - scoped rustfmt and diff check passed
  - current-source Cargo, F2 pixels, timestamp and RenderDoc pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Graphics realtime IBL逐文件性能静态审查（2026-07-18）

## 范围与结论

已完整阅读realtime IBL runtime/time-slice/graph-plan/GPU-resources/capture/timestamps/recorder及tests当前13/13个Rust文件、2,838行。scheduler对published/pending generation、A/B slot、失败重试和stale completion已有明确合同；常规更新分16帧执行。但首个环境仍在一帧执行sky+cloud六面、7级source mip、8级PMREM与SH9，执行层每个活跃slice又重建整套graph/绑定/临时GPU对象，时间片没有覆盖CPU graph成本与首帧GPU尖峰。

## 本轮直接止损

`prepare_frame`原来为scheduler和`IblBakeArtifactRequest`连续派生两次相同`IblBakeKey`，现改为一次派生后Copy复用。`operation_label`原先collect中间`Vec<&str>`再join，现直接写入单一预分配String。capture uniform原先为固定112-byte ABI建立heap Vec，现改为`[u8; 112]`栈数组；既有字节/方向光测试增加静态数组类型约束。三项源码守卫均先RED后GREEN，scoped rustfmt与diff check已过；Cargo仍待协调器。

## PERF-MVP-401：预编译realtime IBL执行变体与有界GPU工作

每个活跃batch都新建`RenderGraphBuilder`，为A/B source/PMREM sampled+per-mip storage与SH9格式化约70个String资源名，clone ready/work slot资源树，动态添加/格式化passes并调用`compile()`；随后又从compiled lifetimes建立`HashSet<&str>`、逐name绑定新`RenderGraphExecutionResources`并全量成功验证。16帧时间片只限制dispatch内容，没有消除这些CPU构建、String/hash/map工作。

record阶段每次capture/downsample创建uniform buffer和bind group；每个PMREM/SH9 dispatch还创建params buffer、source sampler和bind group，SH9为了找一个command先构建完整command plan。当前`CaptureCloud`与`CaptureSky`走完全相同的capture kernel、参数和输出，full update与三帧cloud state形成重复GPU写；在真实cloud输入接入前不应提交第二次等价capture。

timestamp capability存在时，无论诊断consumer是否开启都为每个活跃batch打query、创建MAP_READ buffer并启动channel/map；pending/completed队列没有容量预算。公开诊断读取调用`wait_indefinitely`，会把缺少ready report的查询变成产品线程GPU同步点。runtime又在SceneRenderer构造期无条件创建双slot textures/views/buffers与capture pipelines，feature-off成本联动PERF-MVP-390。

Render11/01应按request geometry、scheduler state/substep和work slot编译有限variant表，复用immutable graph/pass/declaration handles与dense binding plan；sky key变化只更新dynamic params，不重编拓扑。初始环境也必须走有界bootstrap time slice，scene继续采样neutral/last-good直到publish；若保留blocking bootstrap，必须显式opt-in且不用于编辑器F2默认。Render11/17提供persistent params ring、per-view binding bundles和device sampler，cloud无真实输入时不生成重复pass。timestamp只在显式capture/profile打开时启用，readback使用固定in-flight ring与age/drop，产品查询永不无限等待。

Unreal `FRealTimeSlicedReflectionCapture`同样按sky/cloud faces、mip与convolution state分帧，并从render-target pool复用双buffer；resolution无变化不重新分配。Zircon已有正确state machine与persistent A/B纹理，应把compiled topology、bindings和readback生命周期也提升到同一generation owner。

## 验收预算

按first/stable/rebake/cancel/retry、states 0..11/substeps、face size 16/128/512、mips 1/4/8、diagnostics off/on及owners 1/8记录bake-key calls、graph builds/compiles、String/hash/Vec bytes、resource bindings/validation、uniform/bind-group/sampler/readback creates、dispatches、GPU bytes/time、queue depth/age/drop与CPU/GPU p95。当前key≤1/prepare、capture uniform heap=0、label temp Vec=0；最终每request geometry variant compile≤1、stable/active warm String/hash/GPU-object growth=0、first帧受预算、无cloud输入duplicate dispatch=0、sampler≤1/device、diagnostics off query/readback=0、产品`wait_indefinitely`=0。focused Cargo、F2像素、timestamp trace与DX12 RenderDoc完成前保留在`pending.md`。
