---
related_code:
  - zircon_runtime/src/text/atlas/bitmap_run.rs
  - zircon_runtime/src/text/atlas/bitmap_run
  - zircon_runtime/src/text/atlas/dirty.rs
  - zircon_runtime/src/text/atlas/page.rs
  - zircon_runtime/src/text/atlas/page_residency.rs
  - zircon_runtime/src/text/atlas/shelf_allocator.rs
  - zircon_runtime/src/text/atlas/upload.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/atlas_texture_upload/binding.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/text/04-glyph-atlas-and-rasterization.md
  - docs/plans/zircon_runtime/render/17-ui-wgpu-surface-and-render-graph-integration.md
reference_sources:
  - dev/bevy/crates/bevy_text/src/font_atlas.rs
  - dev/bevy/crates/bevy_text/src/font_atlas_set.rs
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Fonts/FontCache.cpp
tests:
  - source-level RED to GREEN no per-glyph row-range Vec guard passed
  - rustfmt check and scoped diff check passed
  - current-source Windows zircon_runtime bitmap atlas tests pending
  - WGPU/Softbuffer/RenderDoc upload and pixel parity pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime text atlas bitmap/upload逐文件性能静态审查（2026-07-18）

## 范围与覆盖

`zircon_runtime/src/text/atlas/{bitmap_run.rs,bitmap_run/**}`当前源14/14个Rust文件，以及`atlas/{mod.rs,page.rs,page_residency/**,shelf_allocator.rs,dirty/**,upload/**}`当前源9/9个Rust文件已逐文件阅读，共23文件。覆盖bitmap allocation/retry/staging/prepared upload、page generation/residency、dirty merge、shelf allocation、upload command及全部对应测试；调用图追到WGPU atlas texture upload binding。

## 回链PERF-MVP-231：run allocator与page rebuild仍是逐帧owner

bitmap run每次从空`BTreeMap<PageKey, ShelfAllocator>`开始，为全部source重新分配slot、mark dirty并生成upload copy；传入resident atlas也只保留page spec/reference/generation，不保留glyph→slot或allocator state。`page_rebuild_residency_decision`会优先重建未在本帧引用的旧page，正是稳定帧再次dirty/upload的机制。该根因已由PERF-MVP-231 persistent glyph slot/allocator owner完整覆盖，不重复编号。

## PERF-MVP-243：小更新仍整页staging，稀疏dirty被外接矩形放大

生产page固定512×512。`glyph_atlas_bitmap_upload_staging_plan`第一次遇到任意copy就执行`vec![0; page_byte_len]`：R8每页256 KiB，RGBA每页1 MiB；即使dirty/upload command只有一个8×16 glyph仍分配并清零整页。后续binding把整页slice连同page stride/source offset交给`queue.write_texture`，GPU payload虽按dirty extent限制，CPU分配与清零没有限制。

`GlyphAtlasDirtyPage`只保存一个`merged_rect`，每次以几何union扩大。当前全量重分配时glyph多为连续区域；PERF231落地persistent slot后，两个相距很远的changed glyph会把中间未变区域一并写入。需要以多个dirty region、合并成本模型和full-page覆盖率阈值共同决策，不能把“每页一次write”当成唯一目标。

原`copy_upload_source_bytes`还为每个glyph分配`row_ranges`，大小等于glyph高度，然后第二遍复制。本轮先以源码门禁确认RED，再改为复制前检查末行source/destination上界并直接逐行copy，删除该临时Vec；`rustfmt --check`和scoped `git diff --check`通过。该局部止损不改变错误前不部分写入的fail-closed语义。

## page residency与参考引擎结论

page residency最多每format 8页、全atlas最多40页，其Vec/filter/min扫描是小常数控制面；真正数据面问题是slot/allocator未持久与page bytes反复重建。Bevy `FontAtlas`把CPU image与`GlyphCacheKey -> GlyphAtlasLocation`跨帧保留并只增量添加glyph；UE Slate把atlas data缓存到shaped glyph并由显式page阈值/flush管理。Zircon应在明确CPU/GPU byte预算下选择persistent page shadow或packed dirty-region staging，而不是每次构造零填充临时整页。

## 责任计划与验收

Text04收到`failure-2026-07-18-bitmap-atlas-full-page-staging-and-dirty-union.md`，WGPU row layout/write binding联动Render17，稳定slot回链PERF231。单8×16 R8/RGBA glyph更新必须记录dirty payload、staging allocated/touched bytes、upload bytes和write count；不得固定支付256 KiB/1 MiB临时清零。两个对角稀疏rect必须保持多region/成本阈值可解释；stable 300 frames staging/write/upload为0。current-source Cargo、1/100/1k changed glyph规模counter和Softbuffer/WGPU/RenderDoc资源/像素证据完成前，本范围保持pending。
