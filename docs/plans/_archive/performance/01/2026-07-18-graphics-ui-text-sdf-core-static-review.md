---
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_atlas.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_atlas
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_render
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/text/05-sdf-msdf-pipeline.md
  - docs/plans/zircon_runtime/text/09-threading-caching-and-performance.md
  - docs/plans/zircon_runtime/render/13-texture-pipeline.md
  - docs/plans/zircon_runtime/render/14-2d-stack.md
  - docs/plans/zircon_runtime/render/17-performance-and-profiling.md
tests:
  - sdf_atlas 7 of 7 Rust files reviewed, 1725 current lines
  - sdf_render 15 of 15 Rust files reviewed, 3796 current lines
  - text 8 of 8 Rust files reviewed, 2479 current lines
  - RED/GREEN source guards cover scalar-count allocation and stable material upload state
  - rustfmt check and git diff check passed for changed files
  - focused Cargo, F2 text pixels and RenderDoc pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Graphics UI text/SDF核心逐文件性能静态审查（2026-07-18）

## 范围与结论

已完整阅读`scene_renderer/ui`的`SdfAtlas` 7/7文件、1725行，`SdfRenderer` 15/15文件、3796行，以及text routing/native/fallback 8/8文件、2479行。范围含全部allocation/cache/owner/plan、material/vertex/product framebuffer与native/SDF fallback测试。

稳定SDF frame仍不是缓存命中。`collect_sdf_atlas_text_keys`按glyph拥有并clone font/family/language key，再clone进BTreeSet；`ScreenSpaceUiSdfAtlas::prepare`即使keys不变仍retain/insert、从头创建GlyphAtlasSet+shelf allocator、slot/run HashMap及四张transition set/map。renderer随后分别为generation failure、fallback advances和final render重复准备SDF CPU runs，重建atlas pixels、decoration/glyph/material Vec、6 vertices/glyph和GPU vertex buffer。upload report可以正确给出None，但重CPU工作已先发生。

native路径每frame重新创建shape buffer Vec、逐batch font-id glyph诊断扫描、text-area和bitmap-area Vec；mixed/placeholder storage又建submission Vec。显式native/SDF batches在routing入口先deep clone，failure overlay按span重复求full/prefix/span advance并clone整batch。字体/slot/raster的更底层任务由PERF-MVP-229/231/242负责，本层唯一prepared text plan和deep clone归PERF-MVP-398。

本轮直接止损三处：`resolved_glyph_advances`按text chars或shaped glyph len直接计数，不再先物化`render_scalars Vec<char>`；`SdfTextMaterialResources`保留已上传material identity与复用upload scratch，首次/扩容/变化仍写入，完全相同稳定帧不再分配bytes或`queue.write_buffer`；prepare report把allocation failure两扫和effect material三扫分别融合为一次。核心终态仍归PERF-MVP-249的single generation compiled SDF artifact。

参考沿用本地Bevy extract/prepare UI batch和UE Slate prepared batch边界；atlas owner另对照本仓库`dev/bevy`的glyph atlas prepare资源复用与UE Slate font atlas/texture resource lifetime。只采纳generation、resident page和single prepared authority原则，不复制接口。

## 验收

按glyph/text 1/100/10k、pages 1/16/256、plain/SDF/MSDF/MTSDF/native mixed、dirty 0/1/100%、stable 300 frames/1% text或effect change记录key/String clone、set/map/shelf/slot/run builds、CPU run prepare、bake pixels、material bytes/upload、vertex bytes/buffer create、native Vec/glyph diagnostic visits和CPU/GPU p50/p95/p99。当前要求identical material uniform upload=0、scalar-count temp Vec=0；最终stable atlas/run/material/vertex/native plan build、GPU create/upload均为0，changed近dirty glyph/page/range。Focused Cargo、F2 text parity/product framebuffer和DX12 RenderDoc通过前留在`pending.md`，不进入`review.md`。
