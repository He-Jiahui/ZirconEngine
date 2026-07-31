---
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/atlas_texture_upload.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/construct.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/screen_space_ui_renderer.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_advances.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text_pixel_snap.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/text/05-sdf-msdf-pipeline.md
  - docs/plans/zircon_runtime/text/09-threading-caching-and-performance.md
  - docs/plans/zircon_runtime/render/08-material-shader-permutation.md
  - docs/plans/zircon_runtime/render/17-performance-and-profiling.md
tests:
  - selected UI root small files 6 of 6 reviewed, 279 current lines
  - streaming grapheme and nonzero-detection source guards RED then GREEN
  - existing combining-mark and advance behavior tests preserved
  - rustfmt and scoped diff checks passed
  - current-source Cargo reservation 461d79d7bbe7445eb9645f3e8bfb7509 still not FIFO head
  - scale counters, editor text pixels and RenderDoc pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Graphics UI root小文件逐文件性能静态审查（2026-07-18）

## 范围与结论

已完整阅读`scene_renderer/ui`根部`atlas_texture_upload.rs`、`construct.rs`、`mod.rs`、`screen_space_ui_renderer.rs`、`sdf_advances.rs`、`text_pixel_snap.rs`当前6/6个Rust文件、279行。范围覆盖atlas upload模块面、UI renderer构造、renderer state、SDF layout advance映射与文字像素吸附，不包含其余7个大型root文件或子目录。

`atlas_texture_upload.rs`与`mod.rs`仅导出模块；renderer state仅持资源/report；pixel snap为每调用固定O(1)。`construct.rs`在renderer创建期同步创建基础UI pipeline且descriptor cache为None，属于PERF-MVP-356的driver/queued pipeline验收，不是每帧重复create。

## 已直接止损

`resolved_layout_advances_for_sdf_glyphs`在grapheme→character fallback中原先先`text.chars().count()`全扫，随后把全部graphemes collect到Vec，再逐grapheme重复数chars；最终sanitize先collect另一Vec再扫描`any`。现在使用grapheme/layout两个iterator同步推进，边展开边校验长度；sanitize边push边累计nonzero。组合字符的`[0, advance]`语义、count mismatch与all-zero行为不变，删除1个grapheme Vec、2次完整扫描和sanitize二次扫描。

源码门禁先RED后GREEN，既有combining-mark测试保留，`rustfmt --check`与scoped diff通过。更大的每帧SDF prepare/atlas rebuild仍归PERF-MVP-249，不能用本局部修复替代generation artifact。

## 验收

按text bytes/graphemes/chars 0/1/1k/100k、ASCII/CJK/combining/emoji ZWJ、layout count match/mismatch、stable/changed记录Unicode passes、grapheme/advance Vec alloc、sanitization visits与CPU p50/p95/p99。当前fallback mapping单次grapheme stream、grapheme temp Vec=0、sanitize visits=1；最终compiled SDF frame应直接消费shaping advances，同generation无需再次映射。current-source Cargo、编辑器text/SDF像素与RenderDoc通过前留在`pending.md`，不进入`review.md`。
