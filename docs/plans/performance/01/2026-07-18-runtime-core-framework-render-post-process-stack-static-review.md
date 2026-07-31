---
related_code:
  - zircon_runtime/src/core/framework/render/post_process/stack.rs
  - zircon_runtime/src/core/framework/render/post_process/stack
  - zircon_runtime/src/core/framework/render/frame_extract.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/build.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/render/01-render-graph-rdg-alignment.md
  - docs/plans/zircon_runtime/render/07-postprocess-color-pipeline.md
  - docs/plans/zircon_runtime/render/17-performance-and-profiling.md
tests:
  - post-process stack six of six Rust test files reviewed
  - stack root descriptor and frame submission caller chain traced
  - effect exposure SSR history terminal AA and upscale order tests inspected
  - scoped git diff check passed
  - focused Cargo scale counters F2 trace and RenderDoc pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime post-process stack逐文件性能静态审查（2026-07-18）

## 范围与结论

已完整阅读`core/framework/render/post_process/stack/**`当前6/6个Rust测试文件、1,080行，并完整读取642行root `stack.rs`及frame extract/submission产品调用链。测试锁定effect split、exposure、SSR temporal/history、FXAA/SMAA terminal及dynamic-resolution upscale顺序。确认MVP每帧重建/多owner根因；它涉及descriptor、validated graph、compile options、extract与submission context的架构边界，不做局部String替换冒充完成。

## PERF-MVP-362：每帧重建并多次深clone post-process descriptor/graph

`build_frame_submission_context`每帧调用`PostProcessStackDescriptor::from_extract_settings...`。构造器为initial resources、每个effect的required/produced/after关系创建大量`String`/Vec，并为阶段依赖连续clone多个Vec。随后stack深clone进入compile options，原stack再次`validated_graph`执行验证/排序，再把stack和graph深clone写回`Arc::make_mut(extract_source)`；graph原件另存`FrameSubmissionContext`。即便camera/effect/history/upscale/plugin feature generation稳定，以上build/validate/clone仍重复，而且`Arc::make_mut`可能连带复制更大的frame extract。

Render01/07/17应按`camera post settings + history capability + AA + render size/upscale + feature registry generation`编译唯一immutable post-process artifact，内部资源/节点使用dense/static IDs而非owned String；compile options、extract、submission context和stats共享`Arc` handle。history availability变化只切换预编译variant或精确失效，稳定generation build/validate/sort/clone=0。不得同时保留descriptor、validated graph和execution graph多份权威。

## 验收要求

按cameras 1/8/100、effects 0/1/all、SSR/TAA/history on/off、upscale on/off、stable/1% changed记录descriptor builds、String/Vec alloc、dependency Vec clone、graph validate/sort、stack/graph clone bytes、`Arc::make_mut` frame-extract clone bytes与CPU p95：stable generation全部=0；changed每唯一variant build/validate≤1；同generation compile/extract/context/stats共享同artifact identity。现有effect/exposure/SSR/history/terminal chain顺序、plugin feature、像素、Cargo、F2与RenderDoc通过前，本切片留在`pending.md`。
