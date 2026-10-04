---
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/execute_compiled_scene_graph_stages.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/submit_compiled_scene_frame.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/hzb/hzb_occlusion_culler.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/indirect_draw_execution.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/render/01-render-graph-rdg-alignment.md
  - docs/plans/zircon_runtime/render/03-gpu-scene-gpu-driven.md
  - docs/plans/zircon_runtime/render/04-visibility-culling.md
  - docs/plans/zircon_runtime/render/05-lighting-shadows.md
  - docs/plans/zircon_runtime/render/07-postprocess-color-pipeline.md
  - docs/plans/zircon_runtime/render/11-environment-lighting.md
  - docs/plans/zircon_runtime/render/17-performance-and-profiling.md
reference_sources:
  - dev/bevy/crates/bevy_render/src/gpu_readback.rs
  - dev/UnrealEngine/Engine/Source/Runtime/RHI/Public/RHIGPUReadback.h
tests:
  - compiled-scene render subtree seventeen of seventeen Rust files reviewed, 4327 current lines
  - compiled-scene aggregate twenty-six of twenty-six Rust files reviewed, 5502 current lines
  - stable history frame borrow and late-stage iterator source guard RED then GREEN
  - sprite stage iterator source guard RED then GREEN
  - persistent screen-space-reflection view source guard RED then GREEN
  - first HZB execution selection without Vec source guard RED then GREEN
  - rustfmt and scoped git diff checks passed
  - current-source focused Cargo, F2 counters, GPU timestamps and RenderDoc pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Graphics compiled-scene逐文件性能静态审查（2026-07-18）

## 范围与结论

已完整阅读`scene_renderer_core_render_compiled_scene/render/**`当前17/17个Rust文件、4,327行，随后读完root outputs/wiring、history copy、runtime-prepare forwarding与scene-pass routing，整个`scene_renderer_core_render_compiled_scene/**`当前26/26个Rust文件、5,502行已完成静态审查。compiled graph的resource-write与lifetime查询已有编译期索引，executor registry验证也以pipeline/registry generation快路复用；这些不是新瓶颈。

本切片确认五组主线根因：HZB telemetry在提交后同步等待GPU（PERF-MVP-373）；history/stage路由的frame深clone与临时stage Vec（PERF-MVP-374）；execution fallback GPU对象和资源别名String逐帧创建（PERF-MVP-375）；indirect execution/compaction/readback buffers逐帧重建（PERF-MVP-376）；irradiance volume选择每帧收集全部可见mesh positions并做volume×mesh containment（PERF-MVP-377）。补读scene-pass routing后又确认stage/pass字符串二次扫描（PERF-MVP-378）。

## 已直接止损

- `execute_compiled_scene_graph_stages`原无论history是否可用都深clone完整`ViewportRenderFrame`；现在history稳定时使用`Cow::Borrowed(frame)`，只有确需删除history resources的冷/失效路径才clone。late graph stages由临时`Vec`改为borrowed iterator。
- `active_sprite_graph_stages`原每camera/frame为最多3个stage分配`Vec`；现在直接返回iterator并交给已支持`IntoIterator`的sprite stats消费者。重复构建sprite geometry本身仍归PERF-MVP-337。
- history binder原已持久保存`screen_space_reflection_view`却每帧从texture再次`create_view`；现在clone持久view handle，与HZB/GI history一致。
- execution-owned binder只消费第一条HZB execution，却先把固定4-slot数组flatten并collect到`Vec`；现在直接`.next()`，删除该帧分配。

四组均先以源码门禁观察RED，再实现GREEN并运行rustfmt/scoped diff check。Cargo预约在本切片期间三次未到FIFO队首，最终被协调器标为consumed且`jobId=null`，没有把它记为动态测试通过。

## PERF-MVP-373：HZB统计把GPU异步队列同步回主线程

`submit_compiled_scene_frame`在`queue.submit`后，只要HZB report有dispatch就无条件调用`collect_last_readback_stats`和每phase indirect-args readback。前者执行`map_async`后`device.poll(PollType::wait_indefinitely)`；后者对args与draw-count分别走同样等待，单frame可多次poll/recv并复制完整args Vec。该数据只进入RenderStats诊断，却会让正常F2帧承担GPU完成等待，形成明确CPU↔GPU bubble。

Render04/17必须改成有界2–4槽readback ring：frame N只encode copy并发起map，后续frame用non-blocking completion消费最近已完成generation；ring满时丢旧telemetry或降采样，绝不能阻塞render submission。diagnostics关闭时copy/map/readback allocation应为0。Bevy的`gpu_readback`在提交后发起`map_async`，以后续`try_recv`触发`ReadbackComplete`；Unreal `FRHIGPUBufferReadback`以fence `IsReady()`轮询后才lock，均不在提交帧强制等待。

## PERF-MVP-374..377剩余根因

PERF-MVP-374由Render07/01/17把historyless variant收进PERF-MVP-362的compiled post artifact，camera cut/resize也只切预编译variant，不再clone frame、重建historyless stack/graph。当前stable-history frame clone与late/sprite stage Vec已为0。

PERF-MVP-375由Render04/05/01让SceneRendererCore或compiled binding plan持久拥有light-grid 3类与HZB 6类neutral fallback buffers；当前binder在声明资源但无producer时逐帧`create_buffer(_init)`并格式化backing name。plugin 7类neutral buffers继续由既有PERF-MVP-348负责。resource generation稳定时fallback create/name alloc必须为0。

PERF-MVP-376由Render03/04/17收敛indirect execution workspace：`assign_execution_owned_indirect_args`当前每帧建两份draw-index Vec、新建args buffer、逐draw copy并Arc clone；随后每phase `MeshIndirectDrawExecution::build`又创建args/metadata/visible/draw-count/compacted buffers，telemetry再创建readback buffers。应按phase/generation复用capacity，dirty ranges批量上传/拷贝，并与PERF-MVP-373共享异步readback owner。

PERF-MVP-377由Render11/18/17在scene/lighting generation发布可见bounds或spatial candidate summary。当前有irradiance volume时，每camera/frame先收集所有layer-visible mesh translation到Vec，然后每个volume遍历positions并执行transform containment，选中后还clone volume。稳定scene/volume generation不得重复position collect，查询应只访问候选并返回borrowed/handle identity。

PERF-MVP-378由Render01/17执行hard cut：`execute_graph_stage`当前每调用一个stage都全扫`pipeline.pass_stages`，每个命中entry又按`pass_name`线性扫描`graph.passes()`；而`CompiledRenderGraph`已经有PassId→index，`CompiledRenderPipelinePassStage`却只保存String。编译期应把stage entry收敛为PassId/dense pass index，并生成按stage连续range或固定offset表；frame执行直接遍历range并O(1)取pass，String只留错误/导出。相关`render_graph/graph.rs`由活动Render01租约保护，本会话没有越权修改。

## 验收

用cameras 1/8、meshes 0/1/1k/100k、graph passes 1/32/256/1k、stages 1/8/32、indirect phases 0/1/4、args 1/1k/100k、history stable/cut/resize、HZB diagnostics off/on记录frame clone bytes、stage Vec alloc、stage/pass visits与String comparisons、view creates、fallback/indirect/readback buffer creates、copy bytes/calls、map requests、blocking polls/wait wall、position collect/containment visits与CPU/GPU p50/p95/p99。最终正常产品帧`wait_indefinitely`=0；diagnostics off readback工作=0；stage dispatch visits近O(executed passes)且name comparisons=0；stable generation frame/stage/fallback/indirect workspace/irradiance rebuild=0；语义、F2像素、Cargo、timestamp与DX12 RenderDoc通过后才可进入`review.md`。
