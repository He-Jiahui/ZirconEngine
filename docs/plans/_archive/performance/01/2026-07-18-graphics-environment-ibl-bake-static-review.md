---
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/environment
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/render/01-render-graph-rdg-alignment.md
  - docs/plans/zircon_runtime/render/11-environment-lighting.md
  - docs/plans/zircon_runtime/render/17-performance-and-profiling.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/ReflectionEnvironmentCapture.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/ReflectionEnvironmentRealTimeCapture.cpp
  - dev/bevy/crates/bevy_pbr/src/light_probe/environment_map.rs
tests:
  - remaining environment IBL bake, lightmap and root Rust source twenty-two of twenty-two files reviewed, 6507 lines
  - complete environment directory forty-six of forty-six current Rust files reviewed, 10894 lines
  - pipeline-cache-owned source sampler source guard RED then GREEN
  - scoped rustfmt and diff check passed
  - current-source Cargo, F2 pixels, readback latency and RenderDoc pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Graphics environment IBL bake/lightmap逐文件性能静态审查（2026-07-18）

## 范围与结论

已完整阅读`scene_renderer/environment/**`余下IBL bake graph/executor/shader/command/binding/dispatch/pipeline/readback/writeback、lightmap与root当前22/22个Rust文件、6,507行。连同probe buffer 11/11和realtime IBL 13/13，environment全目录当前46/46个Rust文件、10,894行静态覆盖完成。GPU数值/ABI、PMREM seam、SH9/IEM parity、readback layout与runtime cache hit已有大量测试；本记录只声明静态审查与局部源码守卫，不声明动态验收。

## 本轮直接止损

普通graph executor和realtime recorder原先每个PMREM/SH9/IEM dispatch都调用`create_ibl_bake_wgpu_source_sampler`。现将相同descriptor的sampler提升到已有`IblBakeWgpuPipelineCache`，普通与realtime路径均借用cache owner；同cache/device sampler create从每dispatch降为1。源码守卫先RED后GREEN并覆盖两条产品路径，scoped rustfmt与diff check已过；Cargo待协调器。

此前probe候选registry读取、realtime bake key/label/capture uniform也已分别止损，详见同目录两份证据。lightmap grid已经按`light_set_generation`跳过稳定重建，atlas按asset ID命中；剩余buffer重建、contract validation、binding Arc clone及feature-off fallback owner继续回链PERF-MVP-353/390。

## PERF-MVP-402：单一compiled IBL command artifact与异步writeback

标准PMREM+SH9+IEM graph有10个pass。`record_ibl_bake_wgpu_pass_for_request`却在每个pass调用`ibl_bake_wgpu_command_plan_for_request`，后者再次为全部mips/kernels构造`ComputeDispatchBuilder`、entry-point/resource/parameter String和map、pipeline labels/keys、params Vec及每face readback-copy Vec，再线性匹配当前pass；因此一个10-pass bake近似构造10次完整10-command plan。realtime single-mip prefilter和SH9也先建完整plan再取一个command。

每pass随后clone pass/executor/resources、source/output WGPU handles和detail dispatch record，创建params buffer、bind group及storage mip view。pipeline/module/layout已有cache，但cache查找仍clone keys并多次contains/get；params序列化又建little-endian Vec。同步runtime writeback位于compiled-scene提交主链：GPU输出经MAP_READ等待、组装完整artifact后写cache文件，cache miss可把GPU等待、payload复制和文件I/O叠加到最小编辑器帧。

Render11/01应在request geometry+required contents+bake key generation边界只构造一次immutable `CompiledIblBakeArtifact`，包含dense command/pass mapping、params bytes/ranges、pipeline keys、output mip-view handles、readback-copy ranges和graph binding plan。executor由compiled pass handle直接取command，不解析pass name、不重建整套plan；standard/realtime共享同一kernel compiler，slice只覆盖face range/dispatch动态字段。params走persistent uniform ring，binding按source/output generation复用，mip views由output owner缓存。

writeback拆为提交后有界readback job：固定in-flight staging ring、非阻塞poll、generation single-flight、过期结果丢弃；payload assembly和atomic cache write在asset worker完成，render线程只发布ticket/ready artifact。缓存命中不创建graph/GPU/readback，miss时队列有age/drop/backpressure，禁止产品`wait_indefinitely`和提交线程文件I/O。

Unreal reflection capture把capture/convolution工作放在持久scene state、pooled render targets与RDG passes中，只有capture/size/state变化才更新；Bevy环境图直接从resident `GpuImage`句柄消费预过滤资产。Zircon现有pipeline cache、transient pool与artifact store足够承载该hard cut，无需保留per-pass重建兼容路径。

## 验收预算

按contents PMREM/SH9/IEM组合、passes 1/8/10、face size 16/128/512、mips 1/4/8、cache hit/miss/stale、jobs 1/8/64及standard/realtime记录command/shader/graph builds、String/map/Vec bytes、pass-name parses/find probes、params/bind/sampler/view creates、readback buffer/map/wait、payload copies、file bytes、queue age/drop和CPU/GPU p95。当前sampler≤1/cache；最终command artifact build≤1/request generation，10-pass plan build=1而非10、per-pass full-plan build/name parse=0、warm params/bind/view create=0、cache-hit GPU/readback/I/O=0、render线程wait/file I/O=0，队列有界且像素/artifact bytes一致。focused Cargo、F2、timestamp/readback trace与DX12 RenderDoc完成前保留在`pending.md`，不进入`review.md`。
