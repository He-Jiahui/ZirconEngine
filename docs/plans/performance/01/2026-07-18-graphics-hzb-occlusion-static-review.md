---
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/hzb
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/render/03-gpu-scene-gpu-driven.md
  - docs/plans/zircon_runtime/render/04-visibility-culling.md
  - docs/plans/zircon_runtime/render/17-performance-and-profiling.md
reference_sources:
  - dev/bevy/crates/bevy_pbr/src/render/mesh_preprocess.wgsl
  - dev/Fyrox/fyrox-impl/src/renderer/occlusion/mod.rs
tests:
  - hzb occlusion subtree 4 of 4 Rust files reviewed, 971 current lines
  - existing WGPU cull/compaction parity tests reviewed
  - current-source Cargo, scale counters, F2 occlusion pixels, timestamp and RenderDoc pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Graphics scene_renderer/hzb整个模块逐文件性能静态审查（2026-07-18）

## 范围与结论

已完整阅读`scene_renderer/hzb/**`当前4/4个Rust文件、971行，覆盖occlusion culler、phase dispatch、module wiring和WGPU cull/compaction tests。默认Forward+/Deferred产品路径启用HZB；根瓶颈已存在于PERF-MVP-024/367/373/376，本轮不重复编号。该目录的新增证据是把每phase transient GPU object与encoder ordering约束并入persistent indirect workspace owner。

## P0瓶颈与路由

- PERF-MVP-024/373：有dispatch时culler把stats复制到MAP_READ buffer，`collect_last_readback_stats`随后`map_async`、`PollType::wait_indefinitely`和channel receive。compiled-scene还对各phase args/draw-count做同类同步读取。产品提交线程因此等待本帧GPU，diagnostics off也没有从culler接口消除copy/readback的明确generation gate。应由Render04/17统一为有界异步ring，正常帧不逐args读取。
- PERF-MVP-376：每个eligible phase先清compaction outputs，再创建一份params upload buffer并copy到共享uniform buffer，创建包含previous HZB、args、metadata、visible remap、draw count、compacted args和stats的8-entry bind group，随后开启独立compute pass。参数copy必须在同一encoder内保持copy→dispatch顺序，不能用多次`queue.write_buffer`做表面优化；应放入phase+command+history-view generation的workspace，以upload ring/dynamic offsets和generation binding bundle复用。
- PERF-MVP-367：当前目录消费previous HZB；本帧HZB金字塔逐mip build的11-pass问题已由独立8文件证据覆盖，不在这里重复。

Bevy GPU preprocess用于对照“indirect args留在GPU并直接消费”，Fyrox用于对照“异步、延迟、可丢的occlusion telemetry”。保留保守可见性、previous-frame重投影、clear-before-dispatch、compact draw-count和command-local palette直绘合同。

## 验收

按phases 0/1/4、args 0/1/1k/100k、instances/arg 1/64、history unavailable/stable/cut、diagnostics off/on记录params upload buffer、bind-group、compute-pass、compaction clear、copy/map、blocking poll wall、readback age/drop和CPU/GPU p50/p95/p99。最终要求产品`wait_indefinitely`=0，diagnostics off readback/copy=0，stable execution+history-view generation的params upload/bind-group/workspace create=0；changed工作近affected phase。current-source Cargo、墙后实例/CPU fallback像素对拍、timestamp与DX12 RenderDoc通过前留在`pending.md`。
