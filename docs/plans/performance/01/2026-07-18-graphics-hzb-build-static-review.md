---
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/resource_lookup.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/post_process/computed_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/post_process/screen_space_reflection.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/post_process/screen_space_reflection/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_hzb_build/execute_hzb_build.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_hzb_build/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_hzb_build/tests.rs
  - zircon_runtime/src/graphics/visibility/occlusion/hzb_builder.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/shaders/hzb_build.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/shaders/hzb_build_msaa.wgsl
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/render/04-visibility-culling.md
  - docs/plans/zircon_runtime/render/17-performance-and-profiling.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/SceneTextureReductions.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/DeferredShadingRenderer.cpp
tests:
  - HZB build and mip-view execution slice eight of eight Rust files reviewed, 2307 current lines
  - single-sample and four-sample WGPU HZB chain regression reviewed
  - HZB parameter temporary Vec source guard changed from one to zero
  - fixed stack parameter packing source guard added
  - rustfmt and scoped git diff check passed
  - current-source Cargo GPU timing and RenderDoc pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Graphics HZB构建与mip-view执行逐文件性能静态审查（2026-07-18）

## 范围与结论

已完整阅读HZB build、GPU resource lookup、shared computed resources、SSR mip consumer与`HzbBuilder`当前8/8个Rust文件、2,307行，并读取single/MSAA WGSL和产品调用链。`HzbBuildPlan`明确`MAX_MIPS_PER_REDUCE_PASS=4`，1923×1081输出1024²、11 mip、`reduce_pass_count=3`；但产品执行完全忽略该批宽，逐mip创建view/bind group并开启compute pass。默认MVP渲染因此存在确定的CPU提交、driver object与GPU pass放大，编号PERF-MVP-367（P0）。

## 已直接止损：HZB参数打包不再分配Vec

`create_hzb_params_upload_buffer`旧实现每帧`map(...).collect::<Vec<HzbParams>>()`，随后立刻把字节复制给新WGPU upload buffer。mip domain受u32 texture extent限制最多32，本轮改为512-byte固定栈数组，只上传实际`mip_count`切片；参数顺序与字节布局不变。源码RED→GREEN为params Vec collect 1→0，并新增fixed stack guard。该修复只删除CPU heap allocation，不掩盖每帧upload buffer创建和逐mipcopy。

## PERF-MVP-367：11-mip HZB被录制为11次独立compute提交

1080p/常见2K视口的HZB计划是11 mip。`record_hzb_build_to_resource`循环每个mip：首级外每级创建source view，每级创建target view；`execute_hzb_build_mip_with_resources`又逐级解析shader resource binding、copy一段参数到持久uniform buffer、创建bind group、begin compute pass并dispatch。稳定帧约产生21个mip views、11个bind groups、11次buffer copy、11个compute passes以及1个新upload buffer。物理HZB texture虽然可被transient pool复用，这些前端对象与命令仍每帧重建。

SSR reflection pyramid的相邻mip循环也通过同一resource lookup逐级`create_view`，说明PERF-MVP-366的pooled view bundle应同时服务HZB/SSR；但HZB的pass数量放大是独立算法/着色器问题，不能只靠view cache解决。

当前WGSL每dispatch只写一个storage mip。Render04 VC-M2原设计和`HzbBuildPlan::reduce_pass_count`已经要求批处理，但实现未闭环。正确方向是增加1–4 mip permutation/统一compute kernel：一个workgroup级reduce连续写最多4个storage mip，首批读取depth，后续批读取上批末mip；11 mip只录制3个reduce passes。每个physical HZB backing按generation缓存all-mip views与batch bind groups；参数放入持久dynamic-uniform/ring，稳定尺寸buffer create=0，单帧upload/copy有明确常数上限。MSAA首批保留逐sample closest/furthest语义，后续批走普通HZB source。

Unreal `SceneTextureReductions.cpp::BuildHZB`的compute路径以`kMaxMipBatchSize`设置mip-count permutation，为每批绑定UAV数组并按批而非按mip添加pass；这与Zircon现有`MAX_MIPS_PER_REDUCE_PASS=4`完全同构。Zircon应完成自己已经声明的三批合同，而不是继续用11个单mip pass冒充VC-M2完成。

## 验收

按1×1、1920×1080、1923×1081、3840×2160，sample count 1/4，HZB off/on记录mip count、reduce batches、texture-view/bind-group/buffer creates、buffer copies、compute passes/dispatches、CPU record p95、GPU timestamp与barriers。1923×1081要求mip=11、reduce passes/dispatches≤3；warm stable尺寸params heap alloc/upload-buffer create/mip-view create/bind-group create=0或由持久bundle给出固定零增长，且无逐mip11次copy/pass。single/MSAA每mip texel链、furthest/closest、occlusion/SSR/SSAO消费者、async-compute fallback与像素必须等价。受管focused WGPU/Cargo、Render04 visibility/render_hzb/render_graph、F2产品trace及DX12 RenderDoc显示HZB reduce markers≤3前，保留在`pending.md`。
