---
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/depth_sampling_mode.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/render_region.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/shader_sources.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/render/07-postprocess-color-pipeline.md
  - docs/plans/zircon_runtime/render/17-performance-and-profiling.md
tests:
  - post-process resources root four of four Rust files reviewed
  - post-process resources aggregate one hundred forty of one hundred forty current Rust files statically reviewed
  - rustfmt and scoped git diff check passed for direct fixes in the module
  - current-source focused Cargo, F2 counters, GPU timestamps and RenderDoc pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Graphics post-process resources root逐文件性能静态审查（2026-07-18）

已完整阅读`post_process/resources`剩余root当前4/4个Rust文件；结合HZB 3个、`execute_post_process/**` 27个、其余`execute_*` 31个与`construct/**` 75个，整个目录当前140/140个Rust文件已完成静态审查。

`depth_sampling_mode.rs`的backend lowercase与GL/ANGLE WGSL替换只在`ScenePostProcessResources::new`发生；本轮共享post shader修复又把大型source替换从9次降为1次。`shader_sources.rs`均为static include/concat；`mod.rs`仅module wiring。`render_region.rs`仍由FXAA/output-transfer/SMAA每stage创建uniform buffer，已归PERF-MVP-370的persistent terminal-region ring，不另建重复任务。

本目录的主要未闭环根因统一由PERF-MVP-367、369、370、371负责：HZB批处理、post/GI binding与upload、effect/SMAA资源与disabled cull、optional pipeline queue。受管Cargo、F2产品计数/像素、GPU timestamp与RenderDoc完成前，目录保持在`pending.md`，不进入`review.md`。
