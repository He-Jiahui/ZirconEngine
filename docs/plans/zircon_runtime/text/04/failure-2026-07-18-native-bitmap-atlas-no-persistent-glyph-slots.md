---
handoff_kind: failure
status: open
created_at: 2026-07-18
summary_slug: native-bitmap-atlas-no-persistent-glyph-slots
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_runtime/text/04-glyph-atlas-and-rasterization.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_runtime/text/04
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/text/native_bitmap_atlas.rs
  - zircon_runtime/src/text/native_bitmap_atlas/source_cache.rs
  - zircon_runtime/src/text/native_bitmap_atlas/retry_frame.rs
  - zircon_runtime/src/text/native_bitmap_atlas/storage.rs
  - zircon_runtime/src/text/atlas/bitmap_run.rs
  - zircon_runtime/src/text/atlas/page.rs
  - zircon_runtime/src/text/render_state.rs
---

# Native bitmap atlas缺少persistent glyph slot

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：`zircon_runtime/src/text`根7/7 Rust文件及atlas slot回查
- 修复责任计划：`docs/plans/zircon_runtime/text/04-glyph-atlas-and-rasterization.md`
- 责任切片：AT-M3。
- 交接原因：glyph identity、slot/page residency、dirty/upload与storage format归Text04；performance audit不在Text01活跃atlas改动期间建立第二套slot owner。

## 失败现象与复现证据

source cache hit按visible glyph occurrence clone `NativeBitmapAtlasCachedGlyphImage`，包括完整`Vec<u8>`。`GlyphAtlasBitmapSource`没有CacheKey/GlyphRasterKey；bitmap run只继承page specs，每帧重建空shelf allocators，对每个source重新allocate、mark dirty并生成upload。稳定frame因此仍复制bitmap、重新分配slot并上传全部可见glyph。

retry selection对source×queued做nested find，再用Vec contains过滤并多轮clone image bytes；mixed storage按连续format分组，clone每个source image与atlas。`prepare_report()`先构建一次storage submissions，真实prepare分支随后再次构建。静态证据见`docs/plans/performance/01/2026-07-18-text-root-static-review.md`。

## 最低共享层根因

当前`GlyphAtlasSet`只持resident pages，不持`GlyphRasterKey -> {page, rect, generation}`。source cache拥有raster bytes但不拥有GPU slot，bitmap run拥有临时slot却没有glyph identity，导致两个缓存层无法表达“同一glyph已resident”。屏幕位置/color被放进source而非draw instance，进一步阻止重复occurrence共享slot。

## 架构修复验收

- Text04建立唯一persistent slot table，key至少覆盖face/instance、glyph id、px/variation/subpixel/content format与face epoch；value携page key/rect/page generation。
- raster bytes改为shared ownership；同一key的多个draw occurrence只引用slot，screen rect/foreground/background属于draw instance，不参与raster slot identity。
- shelf allocator与slot map跨帧持久；只有new/invalidated/evicted glyph产生dirty rect与upload，page rebuild按generation使旧slot fail closed。
- retry按glyph key索引，选择/回填近O(S+Q)；storage partition借用shared sources并维护单一atlas owner，不深clonebytes或为report重建submission。
- stable 300 frames相同draw list：raster bytes clone=0、slot alloc=0、dirty/upload bytes=0；100/10k重复glyph occurrence unique slot=1。
- changed glyph只上传新增/失效slot；记录unique sources、occurrences、slot hit/miss/evict、clone bytes、dirty/upload bytes、page rebuild与CPU p50/p95。
- WGPU/Softbuffer像素、alpha/subpixel/color mixed order、retry/placeholder、face invalidation与RenderDoc texture update/resource lifetime对拍通过。

## 禁止临时方案

- 不得只把glyph bitmap Vec改为Arc而继续每帧重分配slot和全量上传。
- 不得把screen rect或text color放进raster slot key，导致同字形不同位置/颜色重复resident。
- 不得用更大的atlas page掩盖无slot reuse；容量增加只会放大稳定frame上传。
- 不得在prepare report中重新执行真实submission plan；report必须消费已有统计。

## 修复结果与回传

Open state: `PERF-MVP-231已完成根模块/bitmap run静态验证；等待Text04回传persistent key-slot表、shared bytes、stable零upload、mixed/retry近线性、current-source Cargo与RenderDoc证据`。
