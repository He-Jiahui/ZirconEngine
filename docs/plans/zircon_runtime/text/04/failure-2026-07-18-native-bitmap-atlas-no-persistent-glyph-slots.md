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

2026-08-01 implementation state: `open / resolving_failure / non_validation_implementation_complete / managed_validation_pending`。

- canonical owner 已硬切到 `zircon_runtime/src/text/atlas` 与 `zircon_runtime/src/text/native_bitmap_atlas`。`GlyphAtlasSlotCache` 持久保存 `GlyphRasterKey -> {page, page_generation, inserted_frame, rect, content_size}` 及每页 shelf allocator；key 覆盖 instanced face、glyph id、物理 px bucket、水平/垂直 subpixel、format、hinting/smoothing 与 synthetic style。
- source-cache pixels 使用共享 `Arc<[u8]>`；同 key 的 draw occurrence 只保留 screen placement，重复 occurrence 共用 slot。同帧重复 key 只上传一次，后续稳定帧命中不再产生 dirty rect 或 upload command；新 glyph 仅增量分配和上传。
- page eviction、page-size change、generation mismatch 与 upload failure 会失效对应 slot/allocator/shadow，并通过 raster-key reverse index 前向失效 CPU source；bounded CPU/page pressure 不跨 generation 复用旧 slot。
- focused source tests 已覆盖 stable-frame slot hit + zero upload、only-new-glyph upload、same-frame projection、duplicate-key single upload、page eviction/rebuild、retained shadow replay、mixed-storage 顺序与 submission counters。当前只声明实现和静态契约核验完成；300-frame/100/10k 规模执行、managed current-source Cargo、真实 WGPU/Softbuffer 像素与 RenderDoc texture lifetime 仍待 coordinator wakeup 后验收，未生成新截图。

### 2026-09-08 Current Frame Fixture Repair

Stable fixing Session `failure-roll-01a07160-text04`, baseline 601, retains
this lifecycle. Registration request `69a9354974c24e6c9d358ed480781815` first
returned an accepted timeout; its existing receipt subsequently completed.
No duplicate registration was submitted.

Managed Windows job `9e3f8ac2a6564bd48fbeab45874342e3` failed the text-only
library-test compile with 22 errors and zero tests. Its exact input
`runtime-graphics-text-sdf-test-support-3073-20260908`, under the approved
benchmark root, has digest
`ef03d95a2138382b1693cd35ab84a0cae4959fa3674fc5c689041e720fbbbc04`.
Two diagnostics came from constructing `NativeBitmapAtlasFrame` without its
private readiness receipt. The associated struct-update fixtures also crossed
that private field boundary.

Source `3091`, request `a1e848c73e1042c7b9264a60a433e073`, freezes six
Text04 support files. The shared `native_bitmap_atlas/tests.rs` fixture now
uses the production frame builder with empty work and a zero-capacity source
cache, then supplies its existing submission/source/count inputs. Readiness
remains initialized by its production owner. The frame-index, unsupported,
missing-raster and pending-placeholder fixtures reuse that helper and mutate
only their visible test fields; all assertions remain. No private field or
constructor is exposed. The same snapshot explicitly imports canonical
`TextDirection` in the context tests and repairs three byte comparisons in
the atlas/Swash tests, as recorded in the sibling staging failure.

Pre-edit snapshots `3086` and `3088` preserve exact provenance. The three
native-frame test files matched HEAD. `context.rs` and the Swash test retained
the exact archived `runtime-text04-20260831-r1` bytes; bitmap-run tests retained
`root-runtime-editor-optimize-20260901-r6` bytes. Transfers
`4d260401693b4376983c7fd585acbf72` and
`233bda4ba3a84fc88075b841a8c0ee30` record the handover. No unattributed
neighboring file was absorbed. Repository-edition formatting and scoped
whitespace checks pass. Source `3091` still needs managed compilation,
actual frame regressions and independent review. The original stable-frame,
occurrence-scale and WGPU/Softbuffer/RenderDoc acceptance remains required.

Managed Windows job `1f242fb1789e44ce87d7a5cacc569d5f` then compiled immutable
input `runtime-text-owner-test-support-3095-20260908`, digest
`4e5f800cd19e5748464ac759b4e1efbfae073ecf1c05d419327a6d6739cbe9e2`.
The text-only static locked library gate reported three errors, previously 22
under the same feature configuration. None remains in any of this input's
14 newly overlaid paths, including all six Text04 files. Cargo 101 / wrapper 1,
zero tests. Queue/sync/check times were 7.805/21.863/144.665 seconds; the full
10,955-file manifest and 354 dependencies were reverified. The exact receipt
and remaining diagnostics are in `results/text-library-recheck.json` and
`results/graphics-library-compiler-diagnostics.json` under that input. Remaining
errors are the Text03 vertical fixture, unproven dirty-upload bytes and the
separately reviewed Text02 artifact fixture. This accepts compiler progress
only; no frame or performance gate passed.

### 2026-09-08 Native Layout Feature Admission

The existing coordinator-efficiency review task returned C0 / I0 / M0 for all
six source files in snapshot 3091. It rechecked exact current/ObjectStore hashes,
archived provenance, the production empty-frame builder and unchanged assertions.
The report is `.codex/tmp/text-3100-review-20260908-result.txt`; this completes the
source review recorded as pending above, without accepting dynamic frame tests.

Original graphics-library job `94560adfdb1a45daa7e2d5785ae6677c` also diagnosed
`native_layout.rs` importing `crate::ui::surface::layout_text` when only the
graphics feature was enabled. The UI module is correctly feature-gated; the
product-test module admission was incomplete. Source snapshot 3113, request
`ceec97c44cca462c8c483513c2259147`, adds `all(test, feature = "ui")` to both
`native_layout` and its `cjk_layout_contract` consumer in
`zircon_runtime/src/graphics/scene/scene_renderer/ui/atlas_renderer/product_framebuffer.rs`.
Both modules still compile and retain their original tests under the UI feature.
The atlas-only product proof remains available without UI. No producer, assertion,
layout path or visibility changes.

Pre-edit snapshot 3111 preserves the HEAD-clean parent and this record;
transfer `a4e0728d8415412089659084cc72f2bc` establishes ownership. The repaired
parent hash is `7eb64a778c8138ff5092d4e476eaa6216c90f214e5de5baab34307771cc5b7d7`.
Scoped rustfmt and whitespace checks pass. Independent review in
`.codex/tmp/text-foundation-3114-review-20260908-result.txt` returned C0 / I0 / M0
for snapshot 3113, with exact hashes unchanged and no reviewer ownership conflict.
Managed graphics-only job `76c956f0eebe4ea0bf184b7e1b9f2cfe` reached 30 remaining
compiler errors, none from this parent or its native-layout fixture; it ran zero tests.
UI product job `83453a2550a34f61808210e167d29be8` stopped at four UI/Text production
compile errors before product tests. Both jobs used input
`runtime-graphics-text-ui-support-3113-20260908`, digest
`2a2096322809192c3da2dc6bab482a88efee8d4a1c54956c25c49c4de74062f3`.
These are compiler results, not native-layout dynamic acceptance.

Dirty-upload provenance and its borrowed-byte assertion are now preserved by source
snapshot 3118 and the sibling canonical `failure-2026-07-18-bitmap-atlas-full-page-staging-and-dirty-union.md`.
Its dynamic validation and review remain pending. The 300-frame/100/10k occurrence
execution and WGPU/Softbuffer/RenderDoc acceptance remain required; this failure stays open.
