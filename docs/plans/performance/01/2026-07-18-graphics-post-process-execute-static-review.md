---
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/execute/run/execute.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/execute/pass_params_buffer.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/execute/create_bind_group/bind_group_entries.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/encode_hybrid_gi_probes/encode.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/encode_hybrid_gi_trace_regions/encode.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/execute/write_hybrid_gi_buffers/write.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/execute/write_reflection_probes/write.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/render/07-postprocess-color-pipeline.md
  - docs/plans/zircon_runtime/render/17-performance-and-profiling.md
  - docs/plans/zircon_runtime/render/18-advanced-lighting-features.md
reference_sources:
  - dev/bevy/crates/bevy_core_pipeline/src/fullscreen_material.rs
  - dev/bevy/crates/bevy_core_pipeline/src/oit/mod.rs
tests:
  - execute_post_process subtree twenty-seven of twenty-seven Rust files reviewed, 1892 current lines
  - reflection and hybrid GI active-prefix upload guards RED then GREEN
  - disabled zero-count uploads changed from three full writes to zero
  - rustfmt and scoped git diff check passed
  - current-source focused Cargo, F2 counters, GPU timestamps and RenderDoc pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Graphics post-process真实执行内核逐文件性能静态审查（2026-07-18）

## 范围与结论

已完整阅读`post_process/resources/execute_post_process/**`当前27/27个Rust文件、1,892行，覆盖camera matrices、hybrid-GI/reflection encode、参数构建、29-binding bind group、buffer write和fullscreen pass。默认post pass每帧创建参数uniform buffer与bind group；可选probe数据还存在重复索引与无效上传，编号PERF-MVP-369（P0）。

## 已直接止损：只上传active probe前缀

旧实现无论feature是否关闭、count是否为0，都会把固定容量reflection、hybrid-GI probe和trace-region数组完整`queue.write_buffer`；count小于上限时未使用的zero tail也一并上传。shader分别由`feature_flags.w`与`hybrid_gi_counts.xy`界定访问范围，因此count=0时旧buffer内容不可见，count=N时只需覆盖前N项。

本轮让三条路径仅在count>0时写入`[..count]`。源码守卫先验证六个guard/slice条件全为false，再验证实现中各精确出现一次且测试名存在；disabled稳定帧三类上传由3次降为0，N项上传字节从capacity收敛为N×element size。buffer容量、binding和shader循环均未改变。

## PERF-MVP-369：默认post pass每帧创建uniform与bind group

`execute_post_process`每帧构造`PostProcessParams`后调用`device.create_buffer`、`queue.write_buffer`，随后为29个bindings调用`device.create_bind_group`。大多数texture/buffer binding来自同一compiled graph/resource generation；参数虽随camera/frame变化，也不要求新建buffer。Render07应把binding 4改为dynamic uniform offset或等价persistent ring，并以明确的texture/buffer/resource generation key缓存post binding bundle。相同资源generation只更新参数slot，不重复创建driver对象；history/LUT/size/feature资源变化才精确失效。

Bevy fullscreen material以`ComponentUniforms`/`DynamicUniformIndex`提供持久参数buffer，并用`PostProcessBindGroupCache::should_update(view_target)`只在view target texture identity变化时重建两套ping-pong bind groups；其OIT同样长期持有dynamic uniform和容量型buffers。Zircon可借鉴生命周期和失效原则，但key必须覆盖render-graph physical resource generation、history/LUT及depth sampling mode，不得只看逻辑资源名。

## Hybrid-GI prepared sideband仍重复建索引

每camera执行中，probe encoder对最多16个resident probes分别线性扫描`probe_scene_data`和`probe_rt_lighting_rgb`；trace encoder则把完整scene data重建`BTreeMap`，再建`BTreeSet`去重最多16个scheduled IDs。正确owner是Render18/plugin prepare generation：resident probe应与scene/RT lighting resolved row对齐，scheduled trace region应直接携resolved row或dense handle；camera阶段只做投影并写active prefix，不重建tree或做P×S lookup。camera-dependent projection不能错误提升到scene generation，但camera-independent join必须只做一次。

`build_post_process_params`本身是固定字段投影，没有heap collection；其view basis/projection可在PERF-MVP-346的prepared camera matrices闭环后复用，不另建重复问题。reflection encoder当前返回count=0，active-prefix修复已使占位路径完全不上传。

## 验收

按post off/on、history/LUT/depth mode组合、cameras 1/8、probes/regions 0/1/16、scene sideband rows 0/16/1k/100k、stable/1% changed记录uniform-buffer/bind-group create+destroy、binding-bundle rebuild、params upload calls/bytes、probe upload calls/bytes、tree nodes/alloc、scene-data comparisons和CPU record p95。当前要求count=0三类probe writes=0且count=N只写active bytes；最终warm稳定资源generation参数buffer/bind-group create=0、bundle build≤1/generation、params≤1 packed upload/camera frame，probe join每prepare generation≤1、camera projection近O(active)。

受管focused Cargo、F2默认post与hybrid-GI on/off像素、GPU timestamp、DX12 RenderDoc对象/pass及counter矩阵完成前，只保留在`pending.md`。
