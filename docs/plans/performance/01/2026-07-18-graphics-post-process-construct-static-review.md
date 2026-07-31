---
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/construct
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/construct/construct/construct.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/construct/create_pipeline_bundle/build.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/construct/create_pipeline_bundle/post_process_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/construct/bind_group_layouts/post_process.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/construct/buffer_bundle/buffer_bundle.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/render/07-postprocess-color-pipeline.md
  - docs/plans/zircon_runtime/render/08-material-shader-permutation.md
  - docs/plans/zircon_runtime/render/17-performance-and-profiling.md
reference_sources:
  - dev/bevy/crates/bevy_render/src/render_resource/pipeline_cache.rs
tests:
  - post-process resources construct subtree seventy-five of seventy-five Rust files reviewed, 3822 current lines
  - shared post shader-module source guard RED nine child creates then GREEN zero
  - shared post pipeline-layout source guard RED nine child creates then GREEN zero
  - production depth-mode source transforms, shader modules and shared layouts each reduced from nine to one
  - rustfmt and scoped git diff check passed
  - current-source focused Cargo, F0/F2 startup trace and driver pipeline timing pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Graphics post-process资源构造逐文件性能静态审查（2026-07-18）

## 范围与结论

已完整阅读`post_process/resources/construct/**`当前75/75个Rust文件、3,822行，覆盖17个bind-group layouts、15类持久buffers、fallback textures、约27条render/compute pipelines及最终`ScenePostProcessResources`装配。参数buffers、samplers、fallback views和pipelines均由scene renderer core长期持有，不是每帧构造；这为PERF-MVP-369/370的dynamic-offset ring与generation binding bundle提供了现成owner。构造期仍同步创建几乎全部可选pipeline，编号PERF-MVP-371（P0）。

## 已直接止损：九条post entry共享shader module与pipeline layout

blur、DoF、motion blur、scene composite、SSR reflection/coarse/resolve/specular-occlusion和最终post共9条pipeline使用同一`post_process.wgsl`、同一depth-sampling变体与同一29-binding layout。旧实现却在每个文件分别执行`post_process_shader_source`、`device.create_shader_module`和`device.create_pipeline_layout`，仅entry point/target format不同。

本轮在`create_pipeline_bundle`中只生成一次depth-mode WGSL、一个`zircon-post-process-shared-shader`和一个shared pipeline layout，再以借用传给9个pipeline builders。源码TDD先确认child module/layout creates=9/9、shared signatures=0/0；GREEN为0/0、signatures=9/9、build shared module/layout=1/1、source transform=1。pipeline labels、entry points、formats、blend/write mask和shader内容不变。

## PERF-MVP-371：可选post pipelines仍同步阻塞renderer构造

`ScenePostProcessResources::new`仍同步创建几乎完整post能力集合：bloom、cluster、HZB single/MSAA、exposure、LUT、DoF、TAA、velocity、motion-vector/blur、SSR、upscale/output/FXAA和SMAA三阶段等约27条pipelines，即使当前项目/相机没有启用对应效果。SSAO已用`OnceLock`首用创建，证明该owner能表达lazy生命周期，但单独例外不足以保护F0/F2冷启动。

Render07/08应把内建post pipeline描述符放入与PERF-MVP-356一致的typed pipeline cache/queue：F2最小必需集合在设备/项目加载窗口预热，optional effects按compiled post artifact需求排队并single-flight；ready前采用明确定义的bypass/neutral fallback或受控等待，不能在frame thread首次遇到时同步编译全部。shader module/layout按source/depth/ABI key复用，pipeline cache必须去重相同descriptor并记录queued/creating/ready/error。

Bevy `PipelineCache`把render/compute descriptors排队，暴露queued/creating/ready/error状态并支持异步编译；其文档也明确不自动去重相同pipeline，调用方必须避免重复插入。Zircon应结合已有generation key与PERF-MVP-355..358做显式dedup，而不是照搬一个可能重复queue的容器。

所有bind-group uniform entries当前`has_dynamic_offset=false`，PERF-MVP-369/370实施时必须同步迁移ABI和set-bind-group offset；不能只加ring却继续为每slot创建独立buffer/bind group。fallback LUT/black/white/HZB textures只在构造期创建和上传一次，本轮不列为帧热点。

## 验收

按post feature集合minimal/all、backend depth mode raw/fallback、cold/warm cache、pipelines 1/9/27记录WGSL transforms、shader-module/layout/pipeline creates、driver compile wall time、frame-thread stall、queue depth/latency、first-use bypass和RSS。当前共享族要求transforms/modules/layouts=1/1/1；最终F0构造不得同步创建未请求optional pipeline，duplicate descriptor compile≤1，F2必需集合在首帧前ready且frame-thread compile stall=0。

受管focused Cargo、constructor source guards、F0/F2冷暖启动trace、driver/pipeline-cache counters及首次启用各effect像素完成前，只保留在`pending.md`。
