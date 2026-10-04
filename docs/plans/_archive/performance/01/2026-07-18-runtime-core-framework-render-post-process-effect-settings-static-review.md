---
related_code:
  - zircon_runtime/src/core/framework/render/post_process/effect_stack_settings
  - zircon_runtime/src/core/framework/render/post_process/effect_stack_settings.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats/post_process_diagnostics.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/render/07-postprocess-color-pipeline.md
  - docs/plans/zircon_runtime/render/17-performance-and-profiling.md
tests:
  - post-process effect-stack settings eight of eight Rust files reviewed
  - frame stats report and graph execution caller chain traced
  - graph-resource and executor repeated-scan source guard RED to GREEN
  - existing effect enable sanitize and missing-resource tests inspected
  - rustfmt and scoped git diff check passed
  - focused Cargo scale counters F2 trace and RenderDoc pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime post-process effect settings逐文件性能静态审查（2026-07-18）

## 范围与结论

已完整阅读`core/framework/render/post_process/effect_stack_settings/**`当前8/8个Rust文件、1,456行，包括production与tests；另读取root settings及frame stats调用链。各效果settings均为Copy小合同且clamp为O(1)。frame stats原为构建resource status分别扫描graph inputs三次、executor ids五次；本轮已RED→GREEN合并为每集合单遍且保持现有资源/velocity判定。剩余根因是每帧为诊断报告物化动态String列表。

## PERF-MVP-361：effect-stack统计每帧分配标签String并重复拥有图状态

`update_base_stats`每帧调用`RenderPostProcessEffectStackReport::from_settings_with_resources`。报告固定检查11个effect families，却为active、approximated及missing resource labels逐项分配`String`和Vec；motion blur/SSR缺链时还用`format!`产生多条前缀标签。它与sealed post-process graph/execution ids重复表达相同状态，并在后续RenderStats/diagnostics快照中继续clone。当前resource-status扫描已从8轮降为graph/executor各一轮，但stable effect configuration的label allocation仍不归零。

Render07/17应让sealed post-process execution report发布dense effect/resource bitsets和固定计数，`RenderStats`只共享generation report或复制bitset；String label仅在editor UI、capture或日志导出边界按需格式化，并联动PERF-MVP-324/343的stats generation/delta owner。settings/resource generation稳定时effect report不重建，graph executor/resource扫描由sealed report唯一承担。

## 验收要求

按graph nodes/executors 1/100/10k、active effects 0/1/11、missing resources 0/1/all、stable/changed generation记录graph visits、executor visits、label String/Vec alloc、report builds与stats clone bytes：当前resource detection graph/executor visits各≤1轮；最终stable generation report build/label alloc/clone=0，changed build≤1，UI closed时String materialization=0。现有enable/clamp、SSR/motion-vector missing-resource ordering与Cargo通过，并以F2/RenderDoc验证pass事实与bitset一致前，本切片留在`pending.md`。
