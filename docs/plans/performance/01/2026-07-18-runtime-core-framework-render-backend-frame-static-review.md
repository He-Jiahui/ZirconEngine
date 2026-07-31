---
related_code:
  - zircon_runtime/src/core/framework/render/backend_types
  - zircon_runtime/src/core/framework/render/frame_extract.rs
  - zircon_runtime/src/graphics/types/viewport_render_frame_from_snapshot.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/render/10-renderer-family.md
  - docs/plans/zircon_runtime/render/17-performance-and-profiling.md
reference_sources:
  - dev/bevy/crates/bevy_render/src/extract_instances.rs
  - dev/bevy/crates/bevy_render/src/extract_component.rs
  - dev/bevy/crates/bevy_render/src/view/mod.rs
tests:
  - backend_types nine of nine Rust files reviewed
  - root frame_extract.rs one of one Rust file reviewed
  - source-guard RED to GREEN for owned snapshot moves, reference-only size queries and fixed stage storage
  - rustfmt and scoped git diff check passed
  - current-source cargo test -p zircon_runtime --lib frame_extract::tests passed six of six
  - scale counters, broader render Cargo, F2 traces and RenderDoc pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime render backend-types与root frame-extract逐文件性能静态审查（2026-07-18）

## 范围与结论

已完整阅读`core/framework/render/backend_types/**`当前9/9个Rust文件和root `frame_extract.rs` 1/1，共2,335行；并聚焦追踪viewport frame adapter、graph execution report和每帧stats写回。固定尺寸的camera target/history/capability/status DTO主要为Copy scalar，无独立热点；`RenderCapabilitySummary::capability_class_report`目前只在测试调用，`RenderCommand::SubmitFrameExtract`在tracked Rust生产代码无调用，因此不把它们冒充产品瓶颈。

## PERF-MVP-342：owned snapshot在frame adapter内深拷贝

`RenderFrameExtract::from_snapshot`取得整个`RenderSceneSnapshot`所有权后，仍逐项clone camera、mesh Vec、五类light Vec、environment和preview；编辑器`render_frame_submission`直接走这条MVP构帧路径。TDD已改为解构并移动owned payload，只保留visibility输入确实需要的render-layer副本；两个effective-size查询也不再复制固定相机快照。公开API、roundtrip和visibility语义不变，source guards、rustfmt和diff check已过。

剩余问题是compatibility adapters仍可能同时保留完整`scene`与`extract`，而`RenderViewExtract`又并存selected camera payload和camera descriptor。Render10/17应以generation-owned extract artifact/Arc投影收口唯一帧权威，旧snapshot adapter只用于测试、预览兼容或显式导出。Bevy把视图和实例抽入render world的专用组件，复用`ExtractedInstances`存储容量或以上帧长度reserve，而不是在一个owned packet内保留多份整场景DTO。

## PERF-MVP-343：graph diagnostics每帧重新物化并深复制

真实stats写回每帧clone完整alias report，profile report又clone全部pass records；alias report生产端clone每个logical/backing String后排序，stage report原来还为17个固定stage分配`BTreeSet`。本轮已把stage去重改为`[bool; RenderPassStage::ALL.len()]`固定表，删除这笔每帧heap allocation并保持已有计数/顺序测试语义。

alias/profile仍有两个owner：renderer execution record和`RenderStats`。Render17联动Render01应让sealed execution report成为同generation唯一权威，stats借用/Arc共享或只投影dense counters；alias String只在capture/UI导出时格式化，稳定帧不得重建、排序或深clone完整report。该根因与PERF-MVP-324的全量diagnostics snapshot相连，但graph report生产和双owner另由PERF-MVP-343验收。

## 验收要求

PERF-MVP-342按meshes/lights 0/1/1k/100k、cameras 1/8、editor/runtime/compat adapter记录payload clone bytes、Vec allocations、extract builds和CPU p95：当前owned adapter内部scene payload clone=0，最终stable generation extract rebuild=0且scene/extract完整双owner≤1。PERF-MVP-343按passes/resources/aliases 0/1/100/10k记录report builds、String/Vec/tree allocations、sort comparisons、clone bytes和CPU p95：stage-report heap alloc=0，alias/profile每generation build≤1、stats额外deep clone=0。受管Windows作业`cargo test -p zircon_runtime --lib frame_extract::tests`已通过6/6、0失败、8,464 filtered；但graph-report回归、规模counter、F2 trace和RenderDoc未完成，两范围继续留在`pending.md`。
