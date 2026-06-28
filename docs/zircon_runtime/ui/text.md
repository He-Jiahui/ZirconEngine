---
related_code:
  - zircon_runtime/src/ui/text/mod.rs
  - zircon_runtime/src/ui/text/font_registry.rs
  - zircon_runtime/src/ui/text/resolved_layout.rs
  - zircon_runtime/src/ui/text/measure_cache.rs
  - zircon_runtime/src/ui/text/raster/mod.rs
  - zircon_runtime/src/ui/text/shaper.rs
  - zircon_runtime/src/ui/text/layout_engine.rs
  - zircon_runtime/src/ui/text/layout_engine/visual_order.rs
  - zircon_runtime/src/graphics/text/mod.rs
  - zircon_runtime/src/graphics/text/font/mod.rs
  - zircon_runtime/src/graphics/text/font/default_families.rs
  - zircon_runtime/src/graphics/text/font/database.rs
  - zircon_runtime/src/graphics/text/layout/mod.rs
  - zircon_runtime/src/graphics/text/layout/line_break.rs
  - zircon_runtime/src/graphics/text/layout/kinsoku.rs
  - zircon_runtime/src/graphics/text/layout/measure.rs
  - zircon_runtime/src/graphics/text/shaping/mod.rs
  - zircon_runtime/src/graphics/text/shaping/cosmic.rs
  - zircon_runtime/src/graphics/text/shaping/line_break.rs
  - zircon_runtime/src/core/framework/render/text/shaped_run.rs
  - zircon_runtime/src/core/framework/render/text/shaping_service.rs
  - zircon_runtime/src/core/framework/render/text/font/mod.rs
  - zircon_runtime/src/ui/text/grapheme.rs
  - zircon_runtime/src/ui/text/rich_text.rs
  - zircon_runtime/src/ui/text/hit_test.rs
  - zircon_runtime/src/ui/text/edit_state.rs
  - zircon_runtime/src/ui/surface/render/extract.rs
  - zircon_runtime/src/ui/surface/render/resolve.rs
  - zircon_runtime/src/ui/surface/render/text_fields.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_atlas.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_atlas/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_font_bake.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_render/tests.rs
  - zircon_runtime/src/ui/tests/text_shaper.rs
  - zircon_runtime/src/ui/tests/text_pipeline.rs
  - zircon_runtime/src/ui/tests/text_layout.rs
  - zircon_runtime/src/ui/tests/text_hit_testing.rs
  - zircon_runtime/src/asset/assets/font.rs
  - zircon_runtime_interface/src/ui/surface/render/command.rs
  - zircon_runtime_interface/src/ui/surface/render/typography.rs
  - zircon_runtime_interface/src/ui/surface/render/text_layout.rs
  - zircon_runtime_interface/src/ui/surface/render/text_shape.rs
implementation_files:
  - zircon_runtime/src/ui/text/mod.rs
  - zircon_runtime/src/ui/text/font_registry.rs
  - zircon_runtime/src/ui/text/resolved_layout.rs
  - zircon_runtime/src/ui/text/measure_cache.rs
  - zircon_runtime/src/ui/text/raster/mod.rs
  - zircon_runtime/src/ui/text/shaper.rs
  - zircon_runtime/src/ui/text/layout_engine.rs
  - zircon_runtime/src/ui/text/layout_engine/visual_order.rs
  - zircon_runtime/src/graphics/text/mod.rs
  - zircon_runtime/src/graphics/text/font/mod.rs
  - zircon_runtime/src/graphics/text/font/default_families.rs
  - zircon_runtime/src/graphics/text/font/database.rs
  - zircon_runtime/src/graphics/text/layout/mod.rs
  - zircon_runtime/src/graphics/text/layout/line_break.rs
  - zircon_runtime/src/graphics/text/layout/kinsoku.rs
  - zircon_runtime/src/graphics/text/layout/measure.rs
  - zircon_runtime/src/graphics/text/shaping/mod.rs
  - zircon_runtime/src/graphics/text/shaping/cosmic.rs
  - zircon_runtime/src/graphics/text/shaping/line_break.rs
  - zircon_runtime/src/core/framework/render/text/shaped_run.rs
  - zircon_runtime/src/core/framework/render/text/shaping_service.rs
  - zircon_runtime/src/core/framework/render/text/font/mod.rs
  - zircon_runtime/src/ui/surface/render/extract.rs
  - zircon_runtime/src/ui/surface/render/resolve.rs
  - zircon_runtime/src/ui/surface/render/text_fields.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_atlas.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_atlas/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_render/tests.rs
  - zircon_runtime/src/ui/tests/text_shaper.rs
  - zircon_runtime/src/ui/tests/text_pipeline.rs
  - zircon_runtime/src/ui/tests/text_layout.rs
  - zircon_runtime_interface/src/ui/surface/render/command.rs
  - zircon_runtime_interface/src/ui/surface/render/typography.rs
  - zircon_runtime_interface/src/ui/surface/render/text_layout.rs
  - zircon_runtime_interface/src/ui/surface/render/text_shape.rs
plan_sources:
  - .codex/plans/ZirconEngine UITextInputA11y 缺口收束计划.md
  - docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md
  - docs/plans/zircon_runtime/runtime/01-tech-stack-and-dependency-governance.md
  - docs/engine-architecture/runtime-tech-stack.md
  - docs/plans/zircon_editor/editor_ui/03-text-and-font-stack.md
  - user: 2026-06-12 implement editor UI architecture from docs/plans/zircon_editor/editor_ui
  - dev/bevy/crates/bevy_text/src/lib.rs
  - dev/bevy/crates/bevy_ui_render/src/lib.rs
tests:
  - rustfmt --edition 2021 --config skip_children=true --check zircon_runtime/src/ui/text/mod.rs zircon_runtime/src/ui/text/layout_engine.rs zircon_runtime/src/ui/text/shaper.rs zircon_runtime/src/ui/tests/text_shaper.rs zircon_runtime/src/ui/tests/mod.rs (2026-05-23: passed after UiTextShaper boundary addition)
  - cargo test -p zircon_runtime --lib text_shaper --offline --jobs 1 --target-dir D:\cargo-targets\zircon-text-shaper-20260523 --message-format short --color never (2026-05-23: deferred while unrelated Cargo/rustc processes were active)
  - rustfmt --edition 2021 zircon_runtime/src/ui/text/font_registry.rs zircon_runtime/src/ui/text/resolved_layout.rs zircon_runtime/src/ui/text/measure_cache.rs zircon_runtime/src/ui/text/raster/mod.rs zircon_runtime/src/ui/text/mod.rs zircon_runtime/src/ui/tests/text_pipeline.rs zircon_runtime/src/ui/tests/mod.rs (2026-06-12: passed)
  - cargo test -p zircon_runtime --lib style_mapping --no-default-features --features core-min --locked --jobs 1 --target-dir target\codex-editor-ui-runtime-coremin --message-format short --color never -- --nocapture --test-threads=1 (2026-06-12: timed out after 604 seconds while compiling runtime test binary; matching target processes stopped, no Rust diagnostics emitted)
  - target/codex-editor-ui-runtime/debug/deps/zircon_runtime-de6f737e1b69a0f9.exe text_pipeline --nocapture --test-threads=1 (2026-06-12: passed, 5 passed)
  - rustfmt --edition 2021 --check zircon_runtime\src\ui\text\mod.rs zircon_runtime\src\ui\surface\render\extract.rs zircon_runtime\src\ui\surface\render\text_fields.rs zircon_runtime\src\ui\tests\text_layout.rs (2026-06-13: passed after TextField render extract preedit span injection)
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-ui-preedit-layout-0613-coremin --message-format short --color never (2026-06-13: passed with existing warnings only)
  - cargo test -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-ui-preedit-layout-0613-coremin ui::tests::text_layout::render_extract_injects_preedit_span_without_document_value_mutation --message-format short --color never -- --exact --nocapture (2026-06-13: timed out after 1204s during Windows lib-test compile/link; no Rust diagnostics, no zircon_runtime-*.exe test binary, matching cargo/rustc processes stopped)
  - cargo test -p zircon_runtime --lib runtime_input_manager --locked --jobs 1 --target-dir target/codex-editor-ui-runtime --message-format short --color never (2026-06-12: rebuild blocked by unrelated unresolved import crate::core::frame_clock in zircon_runtime/src/core/runtime/state/runtime_inner.rs)
  - rustfmt --edition 2021 --check zircon_runtime/src/ui/text/mod.rs zircon_runtime/src/tests/runtime_absorption/structure_convention/runtime_dead_code.rs zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15.rs zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/foundation.rs zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/foundation.rs (2026-06-27: passed after Runtime 15 F12 UI text edit-state dead-code suppression cleanup)
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_atlas/tests.rs::sdf_atlas_plan_deduplicates_glyph_slots_across_batches
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_render/tests.rs::sdf_draw_plan_creates_one_textured_quad_per_glyph
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/render_ui_sdf_atlas_tests.rs::runtime_15_screen_space_ui_sdf_atlas_tests_are_child_owner_split
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/render_ui_sdf_render.rs::runtime_15_screen_space_ui_sdf_render_tests_are_child_owner_split
  - zircon_runtime/src/tests/extensions/tech_stack_dependency_guard.rs::runtime_text_doc_records_three_layer_stack_and_cross_reference
  - rustfmt --edition 2021 zircon_runtime_interface/src/ui/surface/render/text_layout.rs zircon_runtime_interface/src/ui/surface/render/text_shape.rs zircon_runtime_interface/src/ui/surface/render/command.rs zircon_runtime_interface/src/tests/render_contracts.rs zircon_runtime/src/ui/text/layout_engine.rs zircon_runtime/src/ui/text/layout_engine/tests.rs (2026-06-28: passed)
  - cargo check -q -p zircon_runtime_interface --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-text-0628-interface-contract (2026-06-28: passed)
  - cargo check -q -p zircon_runtime_interface --tests --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-text-0628-interface-contract (2026-06-28: passed)
  - cargo check -q -p zircon_runtime --lib --no-default-features --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-text-0628-check (2026-06-28 SH/LB shaped glyph advance DTO slice: timed out after 244s with no Rust diagnostics; matching validation processes stopped)
  - rustfmt --check zircon_runtime/src/graphics/text/layout/mod.rs zircon_runtime/src/graphics/text/layout/line_break.rs zircon_runtime/src/ui/text/layout_engine.rs zircon_runtime/src/ui/text/layout_engine/tests.rs (2026-06-28 LB-M2 UAX#14 Word-wrap consumption: passed)
  - cargo test -p zircon_runtime --lib --no-default-features --locked --target-dir E:\cargo-targets\zircon-runtime-text-0628-check --message-format short --color never word_wrap_uses_uax14_cjk_break_opportunities (2026-06-28 LB-M2 UAX#14 Word-wrap consumption: passed, 1 passed)
  - cargo check -p zircon_runtime --lib --no-default-features --locked --target-dir E:\cargo-targets\zircon-runtime-text-0628-check --message-format short --color never (2026-06-28 LB-M2 UAX#14 Word-wrap consumption: passed with existing warnings only)
  - rustfmt --check zircon_runtime/src/graphics/text/layout/mod.rs zircon_runtime/src/graphics/text/layout/line_break.rs zircon_runtime/src/graphics/text/layout/kinsoku.rs zircon_runtime/src/ui/text/layout_engine.rs zircon_runtime/src/ui/text/layout_engine/tests.rs zircon_runtime/src/tests/runtime_absorption/code_review_findings/plugin_importer_dx.rs (2026-06-28 LB-M2 CJK kinsoku line-start slice: passed)
  - cargo test -p zircon_runtime --lib --no-default-features --locked --target-dir E:\cargo-targets\zircon-runtime-text-0628-check --message-format short --color never text_wrap_cjk_kinsoku_no_leading_punctuation (2026-06-28 LB-M2 CJK kinsoku line-start slice: passed, 1 passed)
  - cargo test -p zircon_runtime --lib --no-default-features --locked --target-dir E:\cargo-targets\zircon-runtime-text-0628-check --message-format short --color never word_wrap_uses_uax14_cjk_break_opportunities (2026-06-28 LB-M2 CJK kinsoku follow-up regression: passed, 1 passed)
  - cargo test -p zircon_runtime --lib --no-default-features --locked --target-dir E:\cargo-targets\zircon-runtime-text-0628-check --message-format short --color never text_shape_ (2026-06-28 LB-M2 CJK kinsoku follow-up regression: passed, 6 passed)
  - cargo check -p zircon_runtime --lib --no-default-features --locked --target-dir E:\cargo-targets\zircon-runtime-text-0628-check --message-format short --color never (2026-06-28 LB-M2 CJK kinsoku line-start slice: passed with existing warnings only)
  - rustfmt --check zircon_runtime/src/graphics/text/layout/mod.rs zircon_runtime/src/graphics/text/layout/line_break.rs zircon_runtime/src/graphics/text/layout/kinsoku.rs zircon_runtime/src/ui/text/layout_engine.rs zircon_runtime/src/ui/text/layout_engine/visual_order.rs zircon_runtime/src/ui/text/layout_engine/tests.rs zircon_runtime/src/ui/text/hit_test.rs zircon_runtime/src/ui/tests/text_hit_testing.rs (2026-06-28 LB-M2 soft hyphen break suffix slice: passed)
  - cargo test -p zircon_runtime --lib --no-default-features --locked --target-dir E:\cargo-targets\zircon-runtime-text-0628-check --message-format short --color never text_wrap_soft_hyphen_inserts_hyphen (2026-06-28 LB-M2 soft hyphen break suffix slice: passed, 1 passed)
  - cargo test -p zircon_runtime --lib --no-default-features --locked --target-dir E:\cargo-targets\zircon-runtime-text-0628-check --message-format short --color never text_hit_test_soft_hyphen_break_suffix_maps_to_source_hyphen (2026-06-28 LB-M2 soft hyphen break suffix slice: passed, 1 passed)
  - cargo check -p zircon_runtime --lib --no-default-features --locked --target-dir E:\cargo-targets\zircon-runtime-text-0628-check --message-format short --color never (2026-06-28 LB-M2 soft hyphen break suffix slice: passed with existing warnings only)
  - cargo test -p zircon_runtime --lib --no-default-features --locked --target-dir E:\cargo-targets\zircon-runtime-text-0628-check --message-format short --color never text_wrap_cjk_kinsoku_no_leading_punctuation (2026-06-28 LB-M2 soft hyphen regression sweep: passed, 1 passed)
  - cargo test -p zircon_runtime --lib --no-default-features --locked --target-dir E:\cargo-targets\zircon-runtime-text-0628-check --message-format short --color never word_wrap_uses_uax14_cjk_break_opportunities (2026-06-28 LB-M2 soft hyphen regression sweep: passed, 1 passed)
  - rustfmt --check zircon_runtime/src/graphics/text/layout/mod.rs zircon_runtime/src/graphics/text/layout/line_break.rs zircon_runtime/src/graphics/text/layout/kinsoku.rs zircon_runtime/src/ui/text/layout_engine.rs zircon_runtime/src/ui/text/layout_engine/tests.rs (2026-06-28 LB-M2 long-word/NBSP slice: passed)
  - cargo test -p zircon_runtime --lib --no-default-features --locked --target-dir E:\cargo-targets\zircon-runtime-text-0628-check --message-format short --color never text_wrap_long_word_falls_back_to_glyph (2026-06-28 LB-M2 long-word/NBSP slice: passed, 1 passed)
  - cargo test -p zircon_runtime --lib --no-default-features --locked --target-dir E:\cargo-targets\zircon-runtime-text-0628-check --message-format short --color never word_wrap_keeps_non_breaking_space_group_together (2026-06-28 LB-M2 long-word/NBSP slice: passed, 1 passed)
  - cargo check -p zircon_runtime --lib --no-default-features --locked --target-dir E:\cargo-targets\zircon-runtime-text-0628-check --message-format short --color never (2026-06-28 LB-M2 long-word/NBSP slice: passed with existing warnings only)
  - rustfmt --check zircon_runtime/src/graphics/text/layout/kinsoku.rs zircon_runtime/src/graphics/text/layout/line_break.rs zircon_runtime/src/ui/text/layout_engine/tests.rs (2026-06-28 LB-M2 CJK open punctuation line-end slice: passed)
  - cargo test -p zircon_runtime --lib --no-default-features --locked --target-dir E:\cargo-targets\zircon-runtime-text-0628-check --message-format short --color never line_break_chunks_keep_cjk_open_punctuation_with_following_text (2026-06-28 LB-M2 CJK open punctuation line-end slice: passed, 1 passed)
  - cargo test -p zircon_runtime --lib --no-default-features --locked --target-dir E:\cargo-targets\zircon-runtime-text-0628-check --message-format short --color never text_wrap_cjk_kinsoku_no_trailing_open_punctuation (2026-06-28 LB-M2 CJK open punctuation line-end slice: passed, 1 passed)
  - cargo test -p zircon_runtime --lib --no-default-features --locked --target-dir E:\cargo-targets\zircon-runtime-text-0628-check --message-format short --color never text_wrap_cjk_kinsoku_no_leading_punctuation (2026-06-28 LB-M2 CJK open punctuation regression sweep: passed, 1 passed)
  - cargo test -p zircon_runtime --lib --no-default-features --locked --target-dir E:\cargo-targets\zircon-runtime-text-0628-check --message-format short --color never word_wrap_uses_uax14_cjk_break_opportunities (2026-06-28 LB-M2 CJK open punctuation regression sweep: passed, 1 passed)
  - cargo check -p zircon_runtime --lib --no-default-features --locked --target-dir E:\cargo-targets\zircon-runtime-text-0628-check --message-format short --color never (2026-06-28 LB-M2 CJK open punctuation line-end slice: passed with existing warnings only)
  - rustfmt --check zircon_runtime_interface/src/ui/surface/render/typography.rs zircon_runtime/src/ui/surface/render/resolve.rs zircon_runtime/src/ui/text/layout_engine.rs zircon_runtime/src/ui/text/layout_engine/tests.rs zircon_runtime/src/ui/tests/text_layout.rs zircon_runtime/src/graphics/scene/scene_renderer/ui/render.rs zircon_runtime/src/graphics/scene/scene_renderer/ui/text.rs zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_render.rs zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_render/tests.rs zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_atlas/tests.rs (2026-06-28 LB-M3 logical Start/End alignment slice: passed)
  - cargo check -p zircon_runtime_interface --lib --locked --target-dir E:\cargo-targets\zircon-runtime-text-0628-align-interface --message-format short --color never (2026-06-28 LB-M3 logical Start/End alignment slice: passed)
  - cargo check -p zircon_runtime --lib --no-default-features --locked --target-dir E:\cargo-targets\zircon-runtime-text-0628-align --message-format short --color never (2026-06-28 LB-M3 logical Start/End alignment slice: passed with existing warnings only)
  - cargo test -p zircon_runtime start_end --lib --no-default-features --locked --target-dir E:\cargo-targets\zircon-runtime-text-0628-align --message-format short --color never -- --nocapture (2026-06-28 LB-M3 logical Start/End alignment slice: passed, 3 passed)
  - cargo test -p zircon_runtime render_extract_preserves_logical_start_text_align --lib --no-default-features --locked --target-dir E:\cargo-targets\zircon-runtime-text-0628-align --message-format short --color never -- --nocapture (2026-06-28 LB-M3 render-extract logical alignment slice: passed, 1 passed)
  - rustfmt --check zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/mesh_pipeline_variant_cache_owner.rs; cargo test -p zircon_runtime runtime_15_non_base_mesh_variant_cache_owner_is_wired --lib --no-default-features --locked --target-dir E:\cargo-targets\zircon-runtime-text-0628-align --message-format short --color never -- --nocapture (2026-06-28 LB-M3 validation support repair: passed, 1 passed)
  - rustfmt --check zircon_runtime/src/ui/text/layout_engine/visual_order.rs zircon_runtime/src/ui/text/layout_engine/tests.rs (2026-06-28 SH-M2 RTL mirrored punctuation slice: passed)
  - cargo test -p zircon_runtime text_bidi_mirrors --lib --no-default-features --locked --target-dir E:\cargo-targets\zircon-runtime-text-0628-bidi-mirror --message-format short --color never -- --nocapture (2026-06-28 SH-M2 RTL mirrored punctuation slice: passed, 2 passed)
doc_type: module-detail
---

# Runtime UI Text

## Purpose

Runtime UI text owns the shared layout, edit, hit-test, and shaping boundary used by render extraction, TextInput behavior, accessibility text geometry, and future SDF/native text backends.

The active measurement path now uses the shared runtime text service in `graphics/text/layout`, which consumes `graphics/text/shaping` shaped runs. UI text still owns source-run wrapping, rich-run ranges, low-fidelity bidirectional visual order, ellipsis construction, caret/selection geometry, and hit testing, but width, line-height, baseline, cache buckets, hit-test grapheme widths, and resolved-line glyph advances come from the shared shaping/metric owner instead of a fixed equal-width estimate.

## Shaper Boundary

`UiTextShapeRequest` carries the source text, resolved style, layout frame, and optional clip frame. `UiTextShaper` provides two operations:

- `shape_text(...)` returns `UiResolvedTextLayout`, the shared geometry DTO consumed by render extract and hit testing.
- `measure_text(...)` returns `UiSize`, the same measurement used by layout callbacks.

`UiSharedTextShaper` is the current default implementation. It delegates UI line organization to `layout_engine.rs`, while `layout_engine.rs` delegates measurement to `graphics/text/layout/measure.rs`; that measurement owner now derives width and per-grapheme advances from `graphics/text/shaping::shape_horizontal_line(...)` and the neutral `ShapedGlyphRun` contract. The shaping owner also projects UAX#14 break opportunities into cluster flags, and Word wrapping now consumes the first LB-M2 layout owner function, `graphics/text/layout::line_break_chunks(...)`, so CJK text without spaces can wrap from shared cluster break data. `graphics/text/layout/kinsoku.rs` owns the first CJK line-start and line-end prohibition slices and marks protected chunks with `allow_glyph_fallback = false`; `layout_engine.rs` only follows that chunk metadata. Soft hyphen is also chunk metadata now: `layout_engine.rs` keeps a pending break suffix and only appends the visible `-` when an actual width wrap pushes the next chunk to a new line. The same `allow_glyph_fallback` metadata lets ordinary overwide words fall back to grapheme wrapping while NBSP glue groups overhang instead of being split. LB-M3 now keeps logical `start` / `end` in `UiTextAlign`; `layout_engine.rs` resolves `Auto` and external `Mixed` requests to a concrete first-strong base direction before placing Start/End, and render extraction/native/SDF batches carry `text_direction` so draw backends preserve the same semantics. The current SH-M2 interim BiDi scaffold in `layout_engine/visual_order.rs` mirrors a small table of single-codepoint RTL punctuation while preserving source ranges. Full greedy line breaking, complete CJK kinsoku tables, justification, full UAX#9 level/isolate resolution, backend-level mirroring, and vertical layout remain later LB-M2/LB-M3/LB-M4 work. Future richer shaping services must preserve the `UiResolvedTextLayout` contract before diverging into native or SDF raster/cache backends.

## Backend Responsibility Matrix

This matrix mirrors the runtime dependency decision in [Runtime Tech Stack](../../engine-architecture/runtime-tech-stack.md#text-stack-boundary). It is the runtime text ownership boundary until a later milestone replaces the active layout backend through `UiTextShaper`.

| Layer | Input | Output | Owner modules | Current status |
|---|---|---|---|---|
| Shaping, segmentation, layout, and measurement | Source text, resolved style, layout frame, optional clip frame, grapheme segmentation | `UiResolvedTextLayout`, line boxes, caret/selection geometry, `UiSize` measurement | `zircon_runtime/src/ui/text/{shaper.rs,grapheme.rs,layout_engine.rs}` plus `zircon_runtime/src/ui/text/layout_engine/visual_order.rs` and `zircon_runtime/src/graphics/text/layout/{measure.rs,line_break.rs,kinsoku.rs}`; public access through `layout_text(...)`, `measure_text_size(...)`, and `UiTextShaper` | Active backend is `UiSharedTextShaper` plus `graphics/text/layout` glyph metrics, first UAX#14 Word-wrap chunks, first CJK line-start kinsoku chunk metadata, soft-hyphen break suffix metadata, first-strong paragraph base-direction resolution for Auto/Mixed requests, logical Start/End alignment, and an interim RTL mirror table in the visual-order scaffold. `shared_text_shaper_matches_public_layout_entrypoint` locks public-layout parity, `text_measurement_uses_backend_glyph_metrics` rejects equal-width measurement, `word_wrap_uses_uax14_cjk_break_opportunities` locks no-space CJK Word wrapping, `text_wrap_cjk_kinsoku_no_leading_punctuation` locks that `。` does not lead a wrapped line, `text_wrap_soft_hyphen_inserts_hyphen` locks `pre-` / `fix` output, `text_align_start_end_auto_uses_first_strong_rtl_direction` / `text_align_start_end_auto_uses_first_strong_ltr_direction` lock Auto base direction before Start/End placement, and `text_bidi_mirrors_paren_in_rtl` locks RTL punctuation mirroring without source-range drift. |
| Font registry, raster, and SDF policy | Runtime `FontAsset` records, inferred or explicit families, render-mode preference, fallback chain, resolved text layout | Neutral font records, raster policy decisions, bitmap/SDF cache inputs | `zircon_runtime/src/ui/text/{font_registry.rs,raster/mod.rs,measure_cache.rs,resolved_layout.rs}` plus `graphics/text/font` and `graphics/scene/scene_renderer/ui/{sdf_font_bake.rs,sdf_render.rs}` | Runtime owns the policy boundary. `fontsdf 0.5.3` stays a runtime text/raster dependency, and SDF font bake now resolves font assets through `graphics/text/font::FontDatabase` instead of reading files independently. `fontdue 0.9.3` remains editor-only retained-host fallback debt. |
| GPU/native text submission | Extracted UI text primitives and resolved layout output from the text subsystem | Render-side text draw submission and native glyph resources | Runtime graphics/UI render paths including `rhi_wgpu/ui_surface/text.rs`, `rhi_wgpu/ui_surface/geometry.rs`, `graphics/scene/scene_renderer/ui/{render.rs,text.rs,sdf_render.rs}`, `ui/surface/render/resolve.rs`, plus `graphics/text/{font,layout}` and `ui/text/shaper.rs` for intent coverage | `glyphon 0.11.0` remains the native text/render dependency. Native and SDF render modes now use `SharedTextService` for layout metrics, native glyphon font asset loading goes through the shared runtime `FontDatabase`, and `ScreenSpaceUiTextBatch` carries `text_direction` so Start/End are mapped consistently by `native_text_align(...)` and SDF `aligned_text_start_x(...)`. Later atlas/raster milestones still decide the final draw/cache policy. |

## Current Guarantees

`layout_text(...)` now routes through `UiSharedTextShaper`, while `measure_text_size(...)` routes through the same backend. Owner text render extraction and focused TextField text commands construct `UiTextLayoutRequest` values and call `resolve_text_layout(...)`, so render extraction consumes the same request-level source hash and optional preedit span model used by the text pipeline tests.

The existing `layout_engine` still handles grapheme cluster wrapping, rich text run splitting, ellipsis range preservation, and the low-fidelity BiDi scaffold. It asks `graphics/text/layout` for line width, line metrics, ellipsis fit, public measurement, one advance per visual grapheme, and UAX#14/kinsoku/soft-hyphen-aware line-break chunks for Word wrapping; those values now originate from `ShapedGlyphRun` rather than direct UI-side backend calls. `layout_engine.rs` does not own CJK punctuation or U+00AD tables: it only skips glyph fallback when a shared `LineBreakChunk` says fallback is not allowed, and it only appends the shared `break_suffix` when it actually wraps at that chunk. Its visual-order child owner now owns a temporary RTL single-codepoint mirror table for parentheses, arrows, and common paired symbols; the mirror changes only fragment visual text and preserves source byte ranges for hit-test and render DTO projection. Logical `Start` / `End` alignment stays logical through style parsing and is resolved only after line direction is known; explicit RTL and Auto/Mixed text whose first strong character is RTL map Start to the right edge and End to the left edge. `UiResolvedTextLine.glyph_advances` carries those advances to the shared render DTO projection, while `UiShapedGlyph` now has contract slots for actual `font_id`, cluster flags, and glyph rotation. `UiRenderCommand::text_paint(...)` also uses the same advances for caret, selection, composition, and rich-run geometry, so paint decorations do not fall back to equal-width text math when resolved layout data is present.

`UiShapedText::from_resolved_layout(...)` remains a neutral projection layer. In this slice it uses `glyph_advances` for synthetic grapheme glyph frames and rich text paint-run frames. Deserializing older layouts remains possible because `glyph_advances`, `font_id`, `cluster_flags`, and `rotation` all default, but the active runtime layout path now fills the advances explicitly.

`cosmic-text`, Parley, Swash, HarfBuzz, Unicode line-break implementation details, CJK line-start punctuation tables, soft-hyphen detection, and backend Start/End mapping remain governed by the runtime tech-stack decision: third-party shaping types stay inside `graphics/text/shaping/cosmic.rs`, UAX#14 classification stays inside `graphics/text/shaping/line_break.rs`, kinsoku policy stays inside `graphics/text/layout/kinsoku.rs`, soft-hyphen chunk metadata stays inside `graphics/text/layout/line_break.rs`, and UI modules only see neutral text DTOs, measured advances, chunk metadata, logical alignment enums, or the current temporary visual-order mirror output. The mirror table in `visual_order.rs` is an interim compatibility point until full UAX#9 shaping/BiDi ownership moves to the planned backend path.

## Edit State Ownership

Runtime 15 F12 UI text edit-state dead-code suppression cleanup (`runtime_15_ui_text_edit_state_dead_code_suppression_cleanup_static_passed_cargo_deferred`) removed the lingering `cfg_attr(not(test), allow(dead_code))` from `ui/text/mod.rs`. The edit-state owner remains production code because `ui/component/state_reducer/text_input.rs`, `ui/surface/input/editable_text.rs`, `ui/surface/input/keyboard_clipboard.rs`, and `ui/surface/input/text_pointer.rs` all consume `apply_text_edit_action(...)`.

`runtime_15_ui_text_edit_state_dead_code_suppression_cleanup` locks that `ui/text/mod.rs` exposes `mod edit_state;` without suppression and that `ui/text/edit_state.rs` continues to own insert/delete/selection/composition transitions. The broader production dead-code gate now scans attribute lines for `allow(dead_code)`, so `cfg_attr(not(test), allow(dead_code))` cannot bypass the Runtime 15 M5 suppression sweep.

## Validation

`text_shaper.rs` focused tests assert that the default shaper returns the same layout as the public `layout_text(...)` entry point and the same measurement as `measure_text_size(...)`, including a combining-mark ellipsis case. `shared_text_shaper_matches_public_layout_entrypoint` locks public layout parity, while `text_shaper_stack_uses_shared_text_service_for_font_backends` locks NativeGlyphon/SdfAtlas layout intent onto the shared text service. `ui_shaped_text_contract_uses_measured_glyph_advances` locks the interface contract so shaped glyph frames and paint runs use non-uniform measured advances, `text_layout_exports_backend_grapheme_advances` locks the runtime line DTO export, and the `text_shape_` focused lib-test set now locks the shaping owner contract: source ranges, RTL/space/tab flags, backend advance variation, ligature cluster coverage, and UAX#14 word/CJK soft-break flags. `word_wrap_uses_uax14_cjk_break_opportunities` locks the first LB-M2 UI consumption path by proving narrow CJK Word-mode text wraps at shared UAX#14 opportunities without spaces. `text_wrap_cjk_kinsoku_no_leading_punctuation` locks the next LB-M2 slice by proving `"中文。"` resolves to `"中"` then `"文。"` and no line starts with `。`. `text_wrap_soft_hyphen_inserts_hyphen` locks the soft-hyphen slice by proving `"pre\u{00ad}fix"` resolves to `"pre-"` then `"fix"` with no U+00AD visual glyph, and `text_hit_test_soft_hyphen_break_suffix_maps_to_source_hyphen` locks caret source mapping for the visible break suffix. `text_align_start_end_follow_rtl_base_direction`, `native_text_align_maps_start_end_through_text_direction`, `sdf_draw_plan_maps_start_end_through_rtl_direction`, and `render_extract_preserves_logical_start_text_align` lock the first LB-M3 logical alignment slice across UI layout, render extraction, native glyphon, and SDF draw planning. `text_align_start_end_auto_uses_first_strong_rtl_direction`, `text_align_start_end_auto_uses_first_strong_ltr_direction`, `text_align_start_end_mixed_request_uses_first_strong_base_direction`, and `render_extract_auto_direction_uses_first_strong_for_logical_start_align` lock the follow-up first-strong base-direction slice. `text_bidi_mirrors_paren_in_rtl` and `text_bidi_mirrors_arrow_in_rtl` lock the SH-M2 interim mirror table by proving RTL visual lines mirror parentheses and arrows while keeping the original source byte ranges attached to the mirrored visual fragments. The 2026-06-28 `zircon_runtime --lib --no-default-features` check passes with existing warnings only for the first-strong target-dir, and the produced lib-test binary passes `first_strong` 4/4, `start_end` 6/6, `mixed_direction` 1/1, `neutral_separator` 1/1, and `rich_directional_ellipsis` 1/1; the SH-M2 mirror focused Cargo test also passes 2/2 in `E:\cargo-targets\zircon-runtime-text-0628-bidi-mirror`. Visual proof for the current runtime text slices includes `docs/tests/runtime/text/runtime_text_cjk_kinsoku_preview_20260628.png`, `docs/tests/runtime/text/runtime_text_soft_hyphen_preview_20260628.png`, `docs/tests/runtime/text/runtime_text_long_word_nbsp_preview_20260628.png`, `docs/tests/runtime/text/runtime_text_cjk_open_punctuation_preview_20260628.png`, `docs/tests/runtime/text/runtime_text_rtl_start_end_alignment_preview_20260628.png`, `docs/tests/runtime/text/runtime_text_first_strong_direction_preview_20260628.png`, and `docs/tests/runtime/text/runtime_text_rtl_mirrored_punctuation_preview_20260628.png`; matching target-path checks confirm no text verification image was written under repo `target`.

## Font Registry

`UiFontRegistry` is the UI-facing owner for font family registration before the real shaping backend lands. It records `FontAsset` sources, inferred or explicit family names, effective render-mode preference, and a fallback chain whose default values come from `graphics/text/font` instead of a UI-local hardcoded list. Asset registration now uses `FontAsset::effective_render_mode()` so legacy `render_mode`, `render_strategy.default_mode`, and `allow_native` / `allow_sdf` constraints are interpreted once at the asset boundary. Registered families and manifest `fallback_families` are merged into the UI fallback chain with empty entries filtered and existing families deduped case-insensitively. The registry deliberately stores neutral records instead of exposing a shaping library type. A later cosmic-text integration can populate its `FontSystem` from the shared runtime `FontDatabase` without changing the call sites that register editor/runtime font assets.

Screen-space UI native glyphon and SDF render paths now share that runtime database for project font assets. Native loading registers the font file once and injects the registered face into glyphon `FontSystem`; SDF bake resolves the same font asset to `FontFaceId` and constructs fontsdf faces from shared `Arc<[u8]>` bytes. The text system also loads system font faces into the same Zircon `FontFaceId`/family index during initialization, while keeping the third-party `fontdb::Source` hidden inside `graphics/text/font`.

## Layout Request And Cache

`UiTextLayoutRequest` wraps the existing `layout_text(...)` path with the extra information the editor UI plan needs: style key extraction, optional clip frame, source hashing, and an optional preedit span. `UiPreeditSpan` replaces a byte range in the temporary layout input only; it does not mutate the source text. This preserves the IME invariant that preedit text is visual state until commit. TextField render extraction now converts retained composition metadata into this span before shaping, so a command can keep its retained `text`/`editable.text` as the document value while the `UiResolvedTextLayout` line text shows the active preedit replacement.

`UiTextMeasureCache` is the Stage A cache boundary. Its key contains the source/preedit hash, a style key, and a width bucket. The width bucket now comes from `graphics/text/layout::width_bucket_for(...)`, using the shared measured advance for the current style instead of an equal-width character capacity. The cache records per-frame shape count so later frame reports can assert that the same text/style/wrap bucket is shaped once and reused by measure, arrange, and render extraction.

## Raster Path Policy

`UiGlyphRasterPolicy` captures the SDF-versus-bitmap routing decision without introducing a new rasterizer dependency. Static small UI text defaults to bitmap, large text defaults to SDF, and explicitly scalable text prefers SDF. This keeps the future fontsdf/bitmap atlas choice local to the text subsystem while existing `UiResolvedTextLayout` and render extraction remain unchanged.

The 2026-06-24 Screen-space UI SDF atlas test owner split keeps render-side SDF glyph slot planning, atlas sizing, cache retention/eviction, and glyph-run slot mapping in `graphics/scene/scene_renderer/ui/sdf_atlas.rs` and moves the atlas plan/cache tests into `graphics/scene/scene_renderer/ui/sdf_atlas/tests.rs`. Guard `runtime_15_screen_space_ui_sdf_atlas_tests_are_child_owner_split` and status anchor `render_plan14_sdf_atlas_test_owner_split_static_passed_cargo_deferred_active_compile_lane` lock this boundary; the slice has scoped static evidence only while Cargo/WGPU/RenderDoc remain deferred behind active compile lanes.

The 2026-06-24 Screen-space UI SDF render test owner split keeps the render-side SDF atlas/pipeline/vertex production path in `graphics/scene/scene_renderer/ui/sdf_render.rs` and moves the draw-plan/prepare-report tests into `graphics/scene/scene_renderer/ui/sdf_render/tests.rs`. Guard `runtime_15_screen_space_ui_sdf_render_tests_are_child_owner_split` and status anchor `render_plan14_sdf_render_test_owner_split_static_passed_cargo_deferred_active_compile_lane` lock this boundary; the slice has scoped static evidence only while Cargo/WGPU/RenderDoc remain deferred behind active compile lanes.
