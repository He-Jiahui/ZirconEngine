---
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/font_asset.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/font_asset/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_upload.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_upload/tests.rs
focused_callers:
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text/font_assets.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_render/atlas_resources.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/text/01-font-resource-faces-and-database.md
  - docs/plans/zircon_runtime/text/05-sdf-msdf-pipeline.md
  - docs/plans/zircon_runtime/text/09-threading-caching-and-performance.md
  - docs/plans/zircon_runtime/render/13-texture-pipeline.md
  - docs/plans/zircon_runtime/render/17-performance-and-profiling.md
tests:
  - font_asset and sdf_upload slices 4 of 4 Rust files reviewed, 935 current lines
  - focused font cache and atlas resource callers inspected
  - source-page table and borrowed-report source guards RED then GREEN
  - existing multi-page mixed SDF/MSDF and partial-stride tests preserved
  - rustfmt and scoped diff checks passed
  - current-source Cargo reservation 461d79d7bbe7445eb9645f3e8bfb7509 still not FIFO head
  - scale counters, editor text pixels and RenderDoc pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Graphics UI font_asset与sdf_upload逐文件性能静态审查（2026-07-18）

## 范围与结论

已完整阅读`ui/font_asset.rs`及tests、`ui/sdf_upload.rs`及tests当前4/4个Rust文件、935行，并聚焦追踪`ui/text/font_assets.rs`缓存入口与`ui/sdf_render/atlas_resources.rs`真实GPU write caller。该范围不等于`scene_renderer/ui/**`整目录验收。

字体manifest只在`font_assets`成功cache miss加载，不是成功字体的每帧I/O；但加载失败不会写negative/generation record，同一missing/invalid asset可能随每次ensure重复走路径解析/manifest I/O。该根因回链PERF-MVP-250与Text01/05/09，不能用永久negative bool破坏hot reload。

## 已直接止损

多页SDF/MSDF上传命令原先对每个dirty page重新调用`distance_field_atlas_page_keys`，重建slot→BTreeSet→Vec，再从page 0逐页clone spec并累计source offset；all-dirty时最坏O(D×(S log P + P²))。真实write caller还先clone整个`SdfAtlasUploadReport`及其dirty-page Vec。

现在report全程借用；一次构造按page key有序的`(spec, source offset, byte length)`表，各dirty page用binary search定位，source offset投影收敛为O(S log P + P² + D log P)，且dirty report clone bytes=0。稳定无dirty路径仍直接返回零command。源码门禁先RED后GREEN；现有page>0、mixed SDF/MSDF、full resize与partial row-stride测试调用均迁移为借用，`rustfmt --check`与scoped diff通过。

剩余的atlas-set `page()`线性查找使一次page table仍可能O(P²)，且PERF-MVP-249的每帧prepare/page alloc/zero/copy/upload/buffer主根因尚未解决；最终应由generation-owned atlas page/offset metadata直接提供，不在本slice再建长期第二cache。

## 验收

按pages 1/2/16/256、slots 1/1k/100k、dirty pages 0/1/10/100%、SDF/MSDF mixed、stable/resize/1% glyph change、missing/reload fonts记录page-key/set builds、page probes/spec/report clones、command alloc、upload calls/bytes、manifest stat/read/parse、negative retry、CPU p50/p95/p99。当前stable command=0、report dirty-page clone=0、page-key table≤1/upload build；最终page metadata/offset≤1/atlas generation、changed近dirty pages、stable atlas rebuild/upload=0、missing同asset generation I/O≤1且reload精确失效。current-source Cargo、编辑器字体/SDF/MSDF像素与DX12 RenderDoc通过前留在`pending.md`，不进入`review.md`。
