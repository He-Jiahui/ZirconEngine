---
related_code:
  - zircon_runtime/src/text/mod.rs
  - zircon_runtime/src/text/cache/mod.rs
  - zircon_runtime/src/text/cache/frame_dedup.rs
  - zircon_runtime/src/text/cache/layout_cache.rs
  - zircon_runtime/src/text/cache/measure_cache.rs
  - zircon_runtime/src/text/cache/shaped_cache.rs
  - zircon_runtime/src/text/cache/tests.rs
  - zircon_runtime/src/core/framework/render/text/shaped_run.rs
  - zircon_runtime/src/text/shaping/mod.rs
  - zircon_runtime/src/text/parallel/shape_pool.rs
  - zircon_runtime/src/text/layout/measure.rs
  - zircon_runtime/src/text/layout/line_break/mod.rs
  - zircon_runtime/src/text/layout/line_break/greedy.rs
  - zircon_runtime/src/text/layout/line_break/glyph_fallback.rs
  - zircon_runtime/src/ui/text/layout_engine.rs
  - zircon_runtime/src/ui/text/layout_engine/ellipsis.rs
  - zircon_runtime/src/ui/text/layout_engine/line_box.rs
  - zircon_runtime/src/ui/text/layout_engine/overflow_style.rs
  - zircon_runtime/src/ui/text/layout_engine/vertical.rs
  - zircon_runtime/src/ui/text/layout_engine/wrapping.rs
  - zircon_runtime/src/ui/text/resolved_layout.rs
  - zircon_runtime/src/ui/text/shaper.rs
  - zircon_runtime/src/ui/text/measure_cache.rs
  - zircon_runtime/src/ui/surface/render/extract.rs
  - zircon_runtime/src/ui/surface/render/text_prewarm.rs
  - zircon_runtime/src/ui/tests/text_pipeline
implementation_files:
  - zircon_runtime/src/text/cache/mod.rs
  - zircon_runtime/src/text/cache/frame_dedup.rs
  - zircon_runtime/src/text/cache/layout_cache.rs
  - zircon_runtime/src/text/cache/measure_cache.rs
  - zircon_runtime/src/text/cache/shaped_cache.rs
  - zircon_runtime/src/text/cache/tests.rs
  - zircon_runtime/src/text/shaping/mod.rs
  - zircon_runtime/src/text/parallel/shape_pool.rs
  - zircon_runtime/src/text/layout/measure.rs
  - zircon_runtime/src/text/layout/line_break/mod.rs
  - zircon_runtime/src/text/layout/line_break/greedy.rs
  - zircon_runtime/src/text/layout/line_break/glyph_fallback.rs
  - zircon_runtime/src/ui/text/layout_engine.rs
  - zircon_runtime/src/ui/text/layout_engine/ellipsis.rs
  - zircon_runtime/src/ui/text/layout_engine/line_box.rs
  - zircon_runtime/src/ui/text/layout_engine/overflow_style.rs
  - zircon_runtime/src/ui/text/layout_engine/vertical.rs
  - zircon_runtime/src/ui/text/layout_engine/wrapping.rs
  - zircon_runtime/src/ui/text/resolved_layout.rs
  - zircon_runtime/src/ui/text/shaper.rs
  - zircon_runtime/src/ui/text/measure_cache.rs
  - zircon_runtime/src/ui/surface/render/extract.rs
  - zircon_runtime/src/ui/surface/render/text_prewarm.rs
plan_sources:
  - user: 2026-07-06 runtime text/layout architecture implementation and editor spacing follow-up
  - docs/plans/zircon_runtime/text/index.md
  - docs/plans/zircon_runtime/text/09-threading-caching-and-performance.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - zircon_runtime/src/graphics/text/cache/tests.rs
  - zircon_runtime/src/ui/tests/text_pipeline
  - rustfmt --edition 2021 --check zircon_runtime/src/graphics/text/mod.rs zircon_runtime/src/graphics/text/cache/mod.rs zircon_runtime/src/graphics/text/cache/frame_dedup.rs zircon_runtime/src/graphics/text/cache/layout_cache.rs zircon_runtime/src/graphics/text/cache/measure_cache.rs zircon_runtime/src/graphics/text/cache/shaped_cache.rs zircon_runtime/src/graphics/text/cache/tests.rs zircon_runtime/src/ui/text/measure_cache.rs zircon_runtime/src/ui/surface/render/extract.rs zircon_runtime/src/ui/tests/text_pipeline
  - git diff --check -- zircon_runtime/src/graphics/text/mod.rs zircon_runtime/src/graphics/text/cache/mod.rs zircon_runtime/src/graphics/text/cache/frame_dedup.rs zircon_runtime/src/graphics/text/cache/layout_cache.rs zircon_runtime/src/graphics/text/cache/measure_cache.rs zircon_runtime/src/graphics/text/cache/shaped_cache.rs zircon_runtime/src/graphics/text/cache/tests.rs zircon_runtime/src/ui/text/measure_cache.rs zircon_runtime/src/ui/surface/render/extract.rs zircon_runtime/src/ui/tests/text_pipeline
  - cargo check -p zircon_runtime --lib --no-default-features --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-text-frame-dedup-0706 --message-format short --color never (2026-07-06; log docs/tests/runtime/text/runtime_text_frame_dedup_production_routing_cargo_check_20260706.log SHA256 15D4173CE0CEB10E6535D97BE42111ACB8242D1DFCDD372729C7E0C8D0219006; exit docs/tests/runtime/text/runtime_text_frame_dedup_production_routing_cargo_check_20260706.exit.txt SHA256 F7ABB1ED6FA4EC935C9687BEE5E430DD148C7C644FC9E069B0B7605F0CD71832)
  - cargo test -p zircon_runtime text_measure_cache --lib --no-default-features --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-text-frame-dedup-0706 --message-format short --color never -- --nocapture --test-threads=1 timed out after 604s while compiling the lib-test binary; all cargo/rustc processes for that target-dir were stopped
  - rustfmt --edition 2021 zircon_runtime/src/graphics/text/shaping/mod.rs zircon_runtime/src/graphics/text/layout/measure.rs zircon_runtime/src/graphics/text/layout/mod.rs zircon_runtime/src/graphics/text/layout/line_break/greedy.rs zircon_runtime/src/graphics/text/layout/line_break/glyph_fallback.rs zircon_runtime/src/graphics/text/layout/line_break/mod.rs zircon_runtime/src/ui/text/layout_engine.rs zircon_runtime/src/ui/text/layout_engine/ellipsis.rs zircon_runtime/src/ui/text/layout_engine/line_box.rs zircon_runtime/src/ui/text/layout_engine/overflow_style.rs zircon_runtime/src/ui/text/layout_engine/vertical.rs zircon_runtime/src/ui/text/layout_engine/wrapping.rs zircon_runtime/src/ui/text/resolved_layout.rs zircon_runtime/src/ui/text/shaper.rs zircon_runtime/src/ui/text/measure_cache.rs zircon_runtime/src/graphics/text/cache/shaped_cache.rs zircon_runtime/src/ui/tests/text_pipeline (2026-07-06 PF-M1 shared shaped-run provider routing: passed; log docs/tests/runtime/text/runtime_text_shaped_run_provider_rustfmt_check_20260706_r2.log SHA256 E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855; exit SHA256 13BF7B3039C63BF5A50491FA3CFD8EB4E699D1BA1436315AEF9CBE5711530354)
  - cargo check -p zircon_runtime --lib --no-default-features --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-text-shaped-cache-0706 --message-format short --color never (2026-07-06 PF-M1 shared shaped-run provider routing: passed; log docs/tests/runtime/text/runtime_text_shaped_run_provider_cargo_check_20260706_r3.log SHA256 4AB158F5A68376AB92AB0FFA3BEC5F62CA9171796E5C909DFEE969055AF52A95; exit SHA256 13BF7B3039C63BF5A50491FA3CFD8EB4E699D1BA1436315AEF9CBE5711530354)
  - rustfmt --edition 2021 --check zircon_runtime/src/ui/tests/render_segmented_controls.rs zircon_runtime/src/ui/tests/render_selection_controls.rs zircon_runtime/src/ui/tests/render_sliders.rs zircon_runtime/src/ui/text/layout_engine/line_box.rs zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/mod.rs (2026-07-06 spacing/cache layout focused unblock: passed; log docs/tests/runtime/text/runtime_text_spacing_cache_layout_rustfmt_check_20260706.log SHA256 E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855; exit SHA256 13BF7B3039C63BF5A50491FA3CFD8EB4E699D1BA1436315AEF9CBE5711530354)
  - git diff --check -- zircon_runtime/src/ui/tests/render_segmented_controls.rs zircon_runtime/src/ui/tests/render_selection_controls.rs zircon_runtime/src/ui/tests/render_sliders.rs zircon_runtime/src/ui/text/layout_engine/line_box.rs zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/mod.rs docs/tests/runtime/text/runtime_text_spacing_cache_layout_rustfmt_check_20260706.log docs/tests/runtime/text/runtime_text_spacing_cache_layout_rustfmt_check_20260706.exit.txt (2026-07-06 spacing/cache layout focused unblock: passed; log SHA256 100DEA9A34630C4F68552DA9566E62A8A1B1423B7FA4B9716719B81D98CC8F93; exit SHA256 13BF7B3039C63BF5A50491FA3CFD8EB4E699D1BA1436315AEF9CBE5711530354)
  - cargo test -p zircon_runtime text_measure_cache_reuses_shaped_runs_between_measure_and_layout --lib --no-default-features --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-text-shaped-cache-0706 --message-format short --color never -- --nocapture --test-threads=1 (2026-07-06 spacing/cache layout focused unblock: passed 1/1, 6905 filtered; log docs/tests/runtime/text/runtime_text_spacing_cache_layout_focused_test_20260706.log SHA256 1DCD85CCAA299AF05AEA2CDCEEA331023D408473CF297356B8AA6114F16DE40A; exit SHA256 13BF7B3039C63BF5A50491FA3CFD8EB4E699D1BA1436315AEF9CBE5711530354)
  - target/cargo-target scan for runtime_text_spacing_cache_layout*.png and runtime_text_shaped_run_provider*.png (2026-07-06: match_count=0; log docs/tests/runtime/text/runtime_text_spacing_cache_layout_target_scan_20260706.log SHA256 C7A9A3B0A23F3901DC4F059839FF0ABCA2908F239A0464D1E647D22EA17DF449; exit SHA256 13BF7B3039C63BF5A50491FA3CFD8EB4E699D1BA1436315AEF9CBE5711530354)
  - rustfmt --edition 2021 --check zircon_runtime/src/graphics/text/layout/measure.rs zircon_runtime/src/graphics/text/layout/mod.rs zircon_runtime/src/ui/text/layout_engine.rs zircon_runtime/src/ui/tests/text_pipeline (2026-07-06 PF-M1 shape-count perf guard: passed; log docs/tests/runtime/text/runtime_text_render_perf_shape_once_rustfmt_check_20260706.log SHA256 E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855)
  - /tmp/zircon-runtime-text-shaped-cache-0706-wsl/debug/deps/zircon_runtime-45cc3b24adef629c render_perf_text_measure_then_layout_shapes_once --nocapture --test-threads=1 (2026-07-06 WSL direct lib-test binary: passed 1/1; log docs/tests/runtime/text/runtime_text_render_perf_shape_once_focused_wsl_binary_20260706.log SHA256 15A8674D402A487BF3A833FC2C87FFB0799E7B316BC652E8E80D459F58319B42)
  - /tmp/zircon-runtime-text-shaped-cache-0706-wsl/debug/deps/zircon_runtime-45cc3b24adef629c text_measure_cache --nocapture --test-threads=1 (2026-07-06 WSL direct lib-test binary: passed 9/9; log docs/tests/runtime/text/runtime_text_render_perf_shape_once_text_measure_cache_wsl_binary_20260706.log SHA256 071AD21B834B93D1A314B4E6B06763ED0E5DD4EFB72258E164C1AD6169517C80)
  - rustfmt --edition 2021 --check zircon_runtime/src/ui/tests/text_pipeline (2026-07-07 PF-M4 scroll-list cache reuse shape/layout guard: passed; log docs/tests/runtime/text/runtime_text_scroll_cache_reuse_perf_rustfmt_check_20260707.log SHA256 E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855)
  - git diff --check -- zircon_runtime/src/ui/tests/text_pipeline (2026-07-07 PF-M4 scroll-list cache reuse shape/layout guard: passed with LF/CRLF warning only; log docs/tests/runtime/text/runtime_text_scroll_cache_reuse_perf_diff_check_20260707.log SHA256 22FD7413CC13074CC1E6687BCD6B088A4C9EA15A00B765AC4A5F1739E0495A05)
  - target/cargo-target scan for runtime_text_scroll_cache_reuse*.png (2026-07-07: match_count=0; log docs/tests/runtime/text/runtime_text_scroll_cache_reuse_perf_target_png_scan_20260707.log SHA256 E99A2829ECABA7855E71B61F879991A86D2DF3070B1CD2A23AD2D4242C975B7D)
  - rustfmt --edition 2021 --check zircon_runtime/src/graphics/text/parallel/mod.rs zircon_runtime/src/graphics/text/parallel/shape_pool.rs (2026-07-08 PF-M2 parallel shape pool: passed; log docs/tests/runtime/text/runtime_text_parallel_shape_pool_rustfmt_check_20260708.log SHA256 E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855)
  - static scan for production allow/panic/unwrap/expect/TODO and direct rayon in zircon_runtime/src/graphics/text/parallel/shape_pool.rs (2026-07-08 PF-M2 parallel shape pool: match_count=0; log docs/tests/runtime/text/runtime_text_parallel_shape_pool_static_scan_20260708.log SHA256 4726D31A4C8D09B577716B35527BCE04CD30AAE02F550A65CEE3B0F59EE5B56E)
  - target/cargo-target scan for runtime_text_parallel_shape_pool*.png (2026-07-08: match_count=0; log docs/tests/runtime/text/runtime_text_parallel_shape_pool_target_png_scan_20260708.log SHA256 CBA2D2A96483F9C4925BEB67AE0DEEFB035A3DB1D53A1B75C66242522C9CF147)
  - cargo test -p zircon_runtime --lib render_perf_text_parallel_shape_count --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-text-phase-recheck-0707 --quiet --color never -- --test-threads=1 --nocapture (2026-07-08 PF-M2 parallel shape pool follow-up: passed, 1 passed; after explicit `Vec<PendingShapeJob>` pending-queue type)
- rustfmt --edition 2021 --check zircon_runtime/src/ui/text/measure_cache.rs zircon_runtime/src/ui/text/mod.rs zircon_runtime/src/ui/tests/text_pipeline and focused cargo test `render_perf_text_parallel_shape_pool_prewarms_ui_measure_cache` (2026-07-08 PF-M2 UI paragraph shape-pool prewarm: rustfmt passed SHA256 E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855; focused Cargo passed 1/1 with log docs/tests/runtime/text/runtime_text_parallel_shape_pool_ui_prewarm_cargo_final_20260708.log SHA256 EE1284083D8D0A0EEA123EAC87A479D1E8EC9E91F0DE3F2F8820D38FEDDE7275; target/cargo-target same-stem PNG scan count 0 SHA256 C7A9A3B0A23F3901DC4F059839FF0ABCA2908F239A0464D1E647D22EA17DF449)
- rustfmt --edition 2021 --check zircon_runtime/src/ui/surface/render/text_prewarm.rs zircon_runtime/src/ui/surface/render/mod.rs zircon_runtime/src/ui/surface/render/extract.rs zircon_runtime/src/ui/text/measure_cache.rs zircon_runtime/src/ui/tests/text_pipeline and focused cargo test `render_extract_automatically_prewarms_visible_owner_text_before_layout` (2026-07-08 PF-M2 surface owner-text automatic shape prewarm: rustfmt passed SHA256 E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855; focused Cargo passed 1/1 with log docs/tests/runtime/text/runtime_text_surface_auto_shape_prewarm_cargo_final_20260708.log SHA256 7AF294DB8A34FE868E3754C7E297FCE0884396CB4D6B3A95FDF507793453CCF2; no PNG because this is a nonvisual cache route)
- rustfmt --check zircon_runtime/src/ui/surface/render/extract.rs zircon_runtime/src/ui/surface/render/text_prewarm.rs zircon_runtime/src/ui/text/measure_cache.rs zircon_runtime/src/ui/tests/text_pipeline/*.rs and focused cargo test `prewarms` (2026-07-08 PF-M2 surface component Text command prewarm/layout: rustfmt passed with log docs/tests/runtime/text/runtime_text_component_command_prewarm_rustfmt_check_actual_path_20260708.log SHA256 E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855; focused Cargo passed 3/3 with log docs/tests/runtime/text/runtime_text_component_command_prewarm_cargo_retry_20260708.log SHA256 7020C78EAD179467B661D383928016CB2D90B2386E43930D31271861BF2E72E3; no PNG because this is a nonvisual cache/layout route)
  - git diff --check -- zircon_runtime/src/ui/tests/text_pipeline zircon_runtime/src/graphics/text/layout/measure.rs zircon_runtime/src/graphics/text/layout/mod.rs zircon_runtime/src/ui/text/layout_engine.rs docs/plans/zircon_runtime/text docs/zircon_runtime/graphics/text-cache.md docs/zircon_runtime/graphics/text.md docs/zircon_runtime/ui/text.md docs/plans/engine-code-structure-convention.md docs/plans/engine-code-review-findings-2026-06.md (2026-07-06 PF-M1 shape-count perf guard: passed; log docs/tests/runtime/text/runtime_text_render_perf_shape_once_diff_check_20260706.log SHA256 8770EE88BDFDD3B2A2AD17104DDAD0D156CE1E12D8A8920E45D65D06A74BCD6A)
  - target/cargo-target scan for runtime_text_render_perf_shape_once*.png and *phase_no_rollover*.png (2026-07-06: match_count=0; log docs/tests/runtime/text/runtime_text_render_perf_shape_once_target_scan_20260706.log SHA256 E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855)
doc_type: module-detail
---

# Runtime Text Cache

## Purpose

`zircon_runtime/src/graphics/text/cache` owns the first PF-M1 cache contracts for runtime text: shaped-run reuse before measure, layout, raster, or render-specific policies are applied; a shared measure-cache data plane that UI-facing adapters can consume without owning eviction and hit/miss behavior themselves; a layout-cache data plane for exact wrap-width or valid-width-range layout reuse; and a frame-dedup data plane for same-frame measure↔layout reuse. The module exists so the text pipeline can share one unconstrained shaped result between measure closures and full layout passes without letting UI wrap width or alignment leak into the shaping cache.

The cache subtree remains deliberately data-plane oriented. `UiTextMeasureCache` now consumes shaped-run, measure-cache, layout-cache, and frame-dedup storage. The UI layer still owns the UI-specific key projection and resolved layout value type, while `graphics/text/shaping::TextShapeRunProvider` is the small seam that lets layout helpers request shaped runs from either the direct backend or the shared UI cache.

The 2026-07-08 writing-mode key follow-up keeps that UI-specific projection honest for vertical text. `UiTextStyleKey` now includes `UiTextWritingMode`, so `UiTextMeasureCache` cannot reuse a HorizontalTb full layout or same-frame dedup entry for a VerticalRl request with the same text, frame, and other style fields. `style_key_encodes_text_writing_mode` covers the style key, and `text_measure_cache_separates_layouts_by_writing_mode` covers the persistent layout cache plus frame-dedup reports. This is a cache-key correction only; it does not add a root text facade, renderer shortcut, font override, or component letter-spacing rule.

## Related Files

`graphics/text/mod.rs` mounts the cache subtree and remains structural. `graphics/text/cache/mod.rs` mounts the cache leaves and re-exports crate-local contracts. `graphics/text/cache/shaped_cache.rs` owns all shaped-run cache behavior. `graphics/text/cache/measure_cache.rs` owns generic measure-cache storage, LRU retention, exact-text collision protection, and frame reporting. `graphics/text/cache/layout_cache.rs` owns generic layout-cache storage, exact text collision protection, exact wrap-width or `[min,max)` valid-width hit checks, width-miss reporting, and frame/touch LRU. `graphics/text/cache/frame_dedup.rs` owns same-frame key/text/value reuse and per-frame hit/miss/collision reporting. `graphics/text/shaping/mod.rs` owns the provider trait that selects direct shaping or cached shaping. `graphics/text/parallel/shape_pool.rs` consumes the same shaped-run cache contract for PF-M2 paragraph batches: it checks cache hits before worker dispatch, de-duplicates same-batch misses by key plus exact text, and inserts shaped results back into `ShapedRunCache` after worker completion. `graphics/text/layout/measure.rs` and `graphics/text/layout/line_break/*` consume that provider for line metrics, widths, grapheme advances, source-range widths, and wrap decisions. `ui/text/layout_engine*.rs`, `ui/text/resolved_layout.rs`, and `ui/text/shaper.rs` pass the provider through UI layout, overflow, ellipsis, and vertical layout. `ui/text/measure_cache.rs` adapts shaped-run, measure, layout, current-frame dedup, and explicit paragraph prewarm to UI text requests without moving those cache contracts into UI. Its `UiTextShapePrewarmRequest`/`prewarm_horizontal_paragraphs(...)` path feeds visible UI labels into the parallel shape pool using the same `UiTextDirection::Auto` full-range key shape that later measure/layout paths use, and its frame prewarm report now accumulates multiple batches in one frame. `ui/surface/render/text_prewarm.rs` is the surface render collector that calls this adapter before owner-text layout and after component command generation for missing-layout horizontal text commands, while `ui/surface/render/extract.rs` consumes resolved UI layout values after the cache adapter has applied same-frame dedup and prewarm. `graphics/text/cache/tests.rs`, `graphics/text/parallel/shape_pool.rs`, and `ui/tests/text_pipeline/` own private and pipeline contract coverage for key contents, collision handling, LRU semantics, frame-end retention, shape-on-miss, measure-on-miss, layout-on-miss, same-frame reuse, parallel miss de-duplication, UI paragraph prewarm reuse, surface owner-text automatic prewarm, component Text command prewarm/layout, and measure↔layout shaped-run reuse behavior.

The cache stores `core/framework/render/text::ShapedGlyphRun` values. Keys are derived from `TextShapeRequest`, which is the same neutral request shape consumed by `graphics/text/shaping`.

The measure cache is generic over its key and value. `ui/text/measure_cache.rs` uses it with `UiTextMeasureSizeKey`/`UiSize` for natural-size measurement and `UiTextMeasureKey`/`UiTextLayoutResolution` for full layout so the UI adapter can keep exact frame/clip-frame/style/source-hash semantics while sharing the lower-level cache retention and reporting rules.

The layout cache is also generic over key and value. Its width validity accepts either a single exact wrap width or a result-valid width interval. That matches the PF-M1 requirement that wrapped layout results cannot be bucketed around line-break thresholds, but may later be reused inside a proven validity interval.

The frame-dedup cache is generic over key and value. It clears entries on `begin_frame(frame_index)` by design, then reuses an exact key + exact text result inside that frame. `UiTextMeasureCache` now owns one current-frame dedup table for natural-size measurement and one for full layout resolution, so duplicate same-frame UI requests hit before the persistent LRU caches are consulted.

The shared shaped-run provider path is deliberately narrower than the layout cache. `UiCachedTextShapeProvider` projects each `TextShapeRequest` into `ShapedRunCacheKey` and stores the returned `Arc<ShapedGlyphRun>` in the cache. Natural-size measurement and full-layout resolution still have separate final value caches, but their line metrics, width checks, ellipsis decisions, grapheme advances, and source-range measurements now request shaped data through the same provider inside one `UiTextMeasureCache` frame. `ui/text/layout_engine/line_box.rs` only requests the provider-measured space width when the source line actually contains a tab; ordinary compact labels such as `editor base.zui` use the measured grapheme advances directly, so the layout pass does not add a second incidental shaped request for `" "`.

## Behavior Model

`ShapedRunCacheKey::from_request(...)` records text hash, source range, resolved font family, normalized font weight, font size bits, line height bits, tab size bits, base direction, orientation, vertical mode, and a features hash for kerning state. It intentionally does not record wrap width, text alignment, or overflow policy. Those belong to later layout and measure caches because they affect line breaking or clipping, not the unconstrained shaped glyph run.

`ShapedRunCache` stores each run behind `Arc<ShapedGlyphRun>` and stores the requested text as `Arc<str>`. Lookup first matches the key, then requires exact text equality before returning the run. If a key matches but the stored text differs, the lookup reports a collision miss and returns `None`. This enforces the plan requirement that text hash collisions cannot be accepted as valid cache hits.

The default limits are 1024 runs and an 8 MiB estimated byte budget. Entries track `last_used_frame` and a monotonic touch order. Insert and hit paths update the LRU stamp; capacity or byte-budget overflow removes the oldest entry. `finish_frame()` only trims if limits are exceeded, so a quiet offscreen panel does not lose cached shaped runs merely because it was not referenced that frame.

`TextMeasureCache<K,V>` uses the same frame/touch LRU model for resolved measurement or layout values. Its default capacity is 4096 entries. Lookup compares the key and the exact resolved text; a same-key/different-text match is counted as a collision miss and cannot return a stale value. `get_or_insert_with(...)` returns whether the caller actually measured, giving UI and future perf tests a stable counter for "measure/layout work executed" rather than "lookup attempted".

`TextLayoutCache<K,V>` defaults to 2048 entries. Lookup requires key equality, exact text equality, and a `TextLayoutWidthValidity` match. `Exact(width)` requires identical normalized `f32` bits. `Range { min, max }` hits when `min <= requested_width < max`, which allows a future line-breaking pass to cache a result only for the interval where that result is known to remain valid.

`TextFrameDedup<K,V>` has no cross-frame LRU. It is a current-frame table with an exact-text guard, collision miss reporting, update reporting for exact duplicate inserts, and hit/miss counters. `render_perf_text_measure_then_layout_shapes_once` now uses the shaped-run report after a stable `"Hg"` metrics warmup to prove that one source label's measure+layout path inserts and misses only one post-warmup shaped run.

## Design And Rationale

The shaped cache is separate from the UI measure adapter. The UI measure key contains absolute frame geometry and wrap buckets because it stores fully resolved layout data. Reusing that key shape for shaped runs would violate the PF-M1 contract: measure at infinite width and full layout at real width must share the same shaped run and only redo line-breaking/layout work.

The measure cache is generic rather than UI-specific to keep `graphics/text/cache` as the eviction/reporting owner without making `graphics/text` depend on private `ui/text` layout structs. The adapter boundary remains in `ui/text/measure_cache.rs`; the data-plane behavior lives in graphics text.

The layout cache uses a dedicated width-validity type instead of reusing the measure cache key. Full layout depends on wrap width, alignment, overflow, and max-line policy, and only some results are safely reusable across a width interval. Keeping that rule explicit prevents the earlier width-bucket shortcut from reappearing in a lower-level cache.

The cache also remains under `graphics/text/cache` rather than the scene renderer. Shaping is a runtime text service used by UI, native atlas, SDF, and future editor/runtime paths. Scene-renderer code can consume the cache later, but it should not own the cache contract.

## Test Coverage

The private test module covers:

- key equality across wrap, alignment, and overflow differences;
- key inequality for font-size changes;
- rejection of same-key/different-text lookups;
- coexistence of colliding text entries under the same key;
- LRU eviction only after capacity overflow;
- frame-end retention when the cache is under limit;
- `get_or_insert_with(...)` shaping only on a miss;
- measure-cache same-key/different-text collision rejection;
- measure-cache LRU eviction only on capacity overflow;
- measure-cache frame-end retention;
- measure-cache `get_or_insert_with(...)` measuring only on a miss;
- layout-cache exact wrap-width hits and width misses;
- layout-cache valid-width interval hits;
- layout-cache same-key/different-text collision rejection;
- layout-cache LRU/frame-end retention;
- layout-cache `get_or_insert_with(...)` laying out only on a miss;
- frame-dedup same-frame value reuse;
- frame-dedup reset between frames;
- frame-dedup collision rejection;
- frame-dedup exact-entry update.
- UI measure then full-layout shaped-run reuse for `editor base.zui`, proving the layout path hits the shaped cache populated by natural-size measurement instead of shaping a second copy.
- post-metrics-warmup shape-count evidence for `editor base.zui`, proving source text measure+layout inserts and misses one shaped run while preserving the stable metrics sample for line height;
- scroll-list shape/layout reuse for editor-style labels, proving a 3-row scroll only shapes the newly visible rows while overlapping rows hit the shaped cache and absolute layout geometry still misses for changed row positions;
- PF-M2 parallel paragraph shape batches, proving the first batch shapes only unique uncached paragraph texts, a later batch reuses cached runs, and same-batch duplicate misses share one shaped `Arc`;
- platform-independent word-wrap bucket coverage, where `text_measure_cache_reshapes_when_wrap_bucket_changes` derives narrow and wide frames from `measure_line_width(...)` instead of assuming a fixed 25px word width.

On 2026-07-06, `rustfmt --edition 2021` passed for the updated UI cache adapter, render extract, and text-pipeline test files. `cargo check -p zircon_runtime --lib --no-default-features --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-text-frame-dedup-0706 --message-format short --color never` passed; the log is `docs/tests/runtime/text/runtime_text_frame_dedup_production_routing_cargo_check_20260706.log` (SHA256 `15D4173CE0CEB10E6535D97BE42111ACB8242D1DFCDD372729C7E0C8D0219006`) and the exit stamp is `docs/tests/runtime/text/runtime_text_frame_dedup_production_routing_cargo_check_20260706.exit.txt` (SHA256 `F7ABB1ED6FA4EC935C9687BEE5E430DD148C7C644FC9E069B0B7605F0CD71832`). Focused `cargo test -p zircon_runtime text_measure_cache --lib --no-default-features --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-text-frame-dedup-0706 --message-format short --color never -- --nocapture --test-threads=1` timed out after 604s while compiling the lib-test binary; all cargo/rustc processes for that target directory were stopped, so the focused tests are updated but not counted as green yet.

On 2026-07-06, the PF-M1 shared shaped-run provider routing slice passed `rustfmt --edition 2021` for the shaping, layout, UI layout-engine, cache adapter, and text-pipeline files. The rustfmt log is `docs/tests/runtime/text/runtime_text_shaped_run_provider_rustfmt_check_20260706_r2.log` (SHA256 `E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855`) and the exit stamp SHA256 is `13BF7B3039C63BF5A50491FA3CFD8EB4E699D1BA1436315AEF9CBE5711530354`. `cargo check -p zircon_runtime --lib --no-default-features --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-text-shaped-cache-0706 --message-format short --color never` passed; the log is `docs/tests/runtime/text/runtime_text_shaped_run_provider_cargo_check_20260706_r3.log` (SHA256 `4AB158F5A68376AB92AB0FFA3BEC5F62CA9171796E5C909DFEE969055AF52A95`) and the exit stamp SHA256 is `13BF7B3039C63BF5A50491FA3CFD8EB4E699D1BA1436315AEF9CBE5711530354`.

The same date follow-up repaired the focused-test compile drift by moving segmented, selection-control, and slider fixture focus/hover setup to `UiSurface.component_states`, then narrowed `line_box.rs` so non-tab text does not shape a space for tab alignment. `rustfmt --edition 2021 --check` passed for the touched UI fixture, line-box, and graph-execution re-export files; the log is `docs/tests/runtime/text/runtime_text_spacing_cache_layout_rustfmt_check_20260706.log` (SHA256 `E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855`). Scoped `git diff --check` passed with log SHA256 `100DEA9A34630C4F68552DA9566E62A8A1B1423B7FA4B9716719B81D98CC8F93`. Focused `cargo test -p zircon_runtime text_measure_cache_reuses_shaped_runs_between_measure_and_layout --lib --no-default-features --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-text-shaped-cache-0706 --message-format short --color never -- --nocapture --test-threads=1` passed 1/1 with 6905 filtered; the log is `docs/tests/runtime/text/runtime_text_spacing_cache_layout_focused_test_20260706.log` (SHA256 `1DCD85CCAA299AF05AEA2CDCEEA331023D408473CF297356B8AA6114F16DE40A`) and the exit stamp SHA256 is `13BF7B3039C63BF5A50491FA3CFD8EB4E699D1BA1436315AEF9CBE5711530354`. This follow-up produced no new screenshot; target/cargo-target scans found no matching `runtime_text_spacing_cache_layout*.png` or `runtime_text_shaped_run_provider*.png` (scan log SHA256 `C7A9A3B0A23F3901DC4F059839FF0ABCA2908F239A0464D1E647D22EA17DF449`). The existing no-rollover editor crop proof PNGs remain under `docs/tests/runtime/text`.

The PF-M1 shape-count follow-up keeps `DEFAULT_METRICS_SAMPLE` as the line-height source and intentionally rejected the experimental approach that used each label as its own metrics sample, because that changed the existing word-wrap line count on narrow frames. The accepted guard prewarms `"Hg"` metrics, then checks `editor base.zui` source-run shaped-cache deltas. Direct WSL execution of the already compiled lib-test binary passed `render_perf_text_measure_then_layout_shapes_once` 1/1 and `text_measure_cache` 9/9; logs are `docs/tests/runtime/text/runtime_text_render_perf_shape_once_focused_wsl_binary_20260706.log` (SHA256 `15A8674D402A487BF3A833FC2C87FFB0799E7B316BC652E8E80D459F58319B42`) and `docs/tests/runtime/text/runtime_text_render_perf_shape_once_text_measure_cache_wsl_binary_20260706.log` (SHA256 `071AD21B834B93D1A314B4E6B06763ED0E5DD4EFB72258E164C1AD6169517C80`). The fixed word-wrap test now derives frame widths from `measure_line_width(...)`, so Linux and Windows font metric differences cannot turn the single-word line into an unrelated glyph-wrap failure.

The PF-M4 scroll-list cache reuse follow-up adds `render_perf_text_scroll_list_reuses_cache`. It prewarms stable `"Hg"` metrics, lays out five editor-style labels, then scrolls by three rows. The second frame may miss full layout because the row `y` positions changed, but it may only miss/insert three shaped runs for the newly visible rows; the two overlapping rows must hit the shaped cache. This guard is intentionally nonvisual and does not exercise raster/upload bytes yet. Static validation passed through rustfmt and scoped diff-check; focused Cargo was deferred because unrelated cargo/rustc lanes were active. No matching target/cargo-target PNG was found.

The PF-M2 parallel shape-pool follow-up adds `graphics/text/parallel/shape_pool.rs`. It keeps paragraph batch scheduling outside UI cache adapters but consumes the same `ShapedRunCache`: cache hits return immediately, same-batch misses are de-duplicated before worker dispatch, and shaped results are inserted serially before ordered return. `render_perf_text_parallel_shape_count` and the duplicate-miss regression are present in the module. Static validation passed rustfmt, scoped diff-check, and production-risk scan; a 2026-07-08 follow-up added the explicit `Vec<PendingShapeJob>` pending queue type after the editor screenshot closeout exposed Rust type inference failure, then focused Cargo `render_perf_text_parallel_shape_count` passed 1/1. The same date's UI prewarm follow-up adds `UiTextShapePrewarmRequest` and proves `prewarm_horizontal_paragraphs(...)` can batch editor-style visible rows into the same shaped-run cache before layout, with focused Cargo `render_perf_text_parallel_shape_pool_prewarms_ui_measure_cache` passing 1/1. The surface follow-up adds `ui/surface/render/text_prewarm.rs` and proves render extract automatically prewarms visible owner text before layout with focused Cargo `render_extract_automatically_prewarms_visible_owner_text_before_layout` passing 1/1. The component-command follow-up reuses that owner after painter command generation, prewarms missing-layout horizontal `Text` commands, resolves their layouts, and passed focused Cargo `prewarms` 3/3. No matching target/cargo-target PNG was found because these slices are nonvisual.

## Open Issues

PF-M1 production shaped-run routing, measure-to-full-layout shared shaped-run reuse, the first source-run shape-count guard, the PF-M4 scroll-list shape/layout cache reuse guard, the explicit PF-M2 UI paragraph prewarm entry, the surface render owner-text automatic prewarm collector, and component-generated horizontal text-command prewarm/layout are now wired through `UiTextMeasureCache`. PF-M2 paragraph shape-pool coverage has focused Cargo green for `render_perf_text_parallel_shape_count`, `render_perf_text_parallel_shape_pool_prewarms_ui_measure_cache`, `render_extract_automatically_prewarms_visible_owner_text_before_layout`, and `render_extract_prewarms_and_layouts_component_text_commands` through the `prewarms` filter. Rich/vertical prewarm, PF-M4 scroll raster/upload-byte counters and cache hit-rate counters, PF-M3 async raster/upload completion, live editor-window typography QA, and full glyphon `TextAtlas` cutover remain open.
