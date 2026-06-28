---
related_code:
  - zircon_runtime/src/graphics/mod.rs
  - zircon_runtime/src/graphics/text/mod.rs
  - zircon_runtime/src/graphics/text/font/mod.rs
  - zircon_runtime/src/graphics/text/font/default_families.rs
  - zircon_runtime/src/graphics/text/font/database.rs
  - zircon_runtime/src/graphics/text/font/asset_registration.rs
  - zircon_runtime/src/graphics/text/font/coverage.rs
  - zircon_runtime/src/graphics/text/font/test_font_fixtures.rs
  - zircon_runtime/src/asset/assets/font.rs
  - zircon_runtime/src/asset/importer/ingest/import_font_asset/mod.rs
  - zircon_runtime/src/asset/importer/ingest/import_font_asset/parse_sfnt.rs
  - zircon_runtime/src/graphics/text/layout/mod.rs
  - zircon_runtime/src/graphics/text/layout/line_break.rs
  - zircon_runtime/src/graphics/text/layout/kinsoku.rs
  - zircon_runtime/src/graphics/text/layout/measure.rs
  - zircon_runtime/src/graphics/text/shaping/mod.rs
  - zircon_runtime/src/graphics/text/shaping/cosmic.rs
  - zircon_runtime/src/graphics/text/shaping/line_break.rs
  - zircon_runtime/src/graphics/text/shaping/tests.rs
  - zircon_runtime/src/core/framework/render/mod.rs
  - zircon_runtime/src/core/framework/render/text/mod.rs
  - zircon_runtime/src/core/framework/render/text/shaped_run.rs
  - zircon_runtime/src/core/framework/render/text/shaping_service.rs
  - zircon_runtime/src/core/framework/render/text/font/mod.rs
  - zircon_runtime/src/core/framework/render/text/font/face.rs
  - zircon_runtime/src/core/framework/render/text/font/family.rs
  - zircon_runtime/src/core/framework/render/text/font/database.rs
  - zircon_runtime/src/core/framework/render/text/font/composite.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_font_bake.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_render/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_atlas/tests.rs
  - zircon_runtime/src/ui/text/layout_engine.rs
  - zircon_runtime/src/ui/surface/render/resolve.rs
  - zircon_runtime/src/ui/text/font_registry.rs
  - zircon_runtime/src/ui/text/measure_cache.rs
  - zircon_runtime/src/ui/text/hit_test.rs
  - zircon_runtime/src/ui/text/shaper.rs
  - zircon_runtime/src/ui/text/layout_engine/tests.rs
  - zircon_runtime/src/ui/tests/text_shaper.rs
  - zircon_runtime/src/ui/tests/text_hit_testing.rs
  - zircon_runtime_interface/src/ui/surface/render/command.rs
  - zircon_runtime_interface/src/ui/surface/render/typography.rs
  - zircon_runtime_interface/src/ui/surface/render/text_layout.rs
  - zircon_runtime_interface/src/ui/surface/render/text_shape.rs
  - zircon_runtime_interface/src/tests/render_contracts.rs
implementation_files:
  - zircon_runtime/src/graphics/text/mod.rs
  - zircon_runtime/src/graphics/text/font/mod.rs
  - zircon_runtime/src/graphics/text/font/default_families.rs
  - zircon_runtime/src/graphics/text/font/database.rs
  - zircon_runtime/src/graphics/text/font/asset_registration.rs
  - zircon_runtime/src/graphics/text/font/coverage.rs
  - zircon_runtime/src/graphics/text/font/test_font_fixtures.rs
  - zircon_runtime/src/asset/assets/font.rs
  - zircon_runtime/src/asset/importer/ingest/import_font_asset/mod.rs
  - zircon_runtime/src/asset/importer/ingest/import_font_asset/parse_sfnt.rs
  - zircon_runtime/Cargo.toml
  - zircon_runtime/src/graphics/text/layout/mod.rs
  - zircon_runtime/src/graphics/text/layout/line_break.rs
  - zircon_runtime/src/graphics/text/layout/kinsoku.rs
  - zircon_runtime/src/graphics/text/layout/measure.rs
  - zircon_runtime/src/graphics/text/shaping/mod.rs
  - zircon_runtime/src/graphics/text/shaping/cosmic.rs
  - zircon_runtime/src/graphics/text/shaping/line_break.rs
  - zircon_runtime/src/graphics/text/shaping/tests.rs
  - zircon_runtime/src/core/framework/render/mod.rs
  - zircon_runtime/src/core/framework/render/text/mod.rs
  - zircon_runtime/src/core/framework/render/text/shaped_run.rs
  - zircon_runtime/src/core/framework/render/text/shaping_service.rs
  - zircon_runtime/src/core/framework/render/text/font/mod.rs
  - zircon_runtime/src/core/framework/render/text/font/face.rs
  - zircon_runtime/src/core/framework/render/text/font/family.rs
  - zircon_runtime/src/core/framework/render/text/font/database.rs
  - zircon_runtime/src/core/framework/render/text/font/composite.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_font_bake.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_render/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_atlas/tests.rs
  - zircon_runtime/src/ui/text/layout_engine.rs
  - zircon_runtime/src/ui/surface/render/resolve.rs
  - zircon_runtime/src/ui/text/font_registry.rs
  - zircon_runtime/src/ui/text/measure_cache.rs
  - zircon_runtime/src/ui/text/hit_test.rs
  - zircon_runtime/src/ui/text/shaper.rs
  - zircon_runtime_interface/src/ui/surface/render/command.rs
  - zircon_runtime_interface/src/ui/surface/render/typography.rs
  - zircon_runtime_interface/src/ui/surface/render/text_layout.rs
  - zircon_runtime_interface/src/ui/surface/render/text_shape.rs
plan_sources:
  - docs/plans/zircon_runtime/text/index.md
  - docs/plans/zircon_runtime/text/01-font-resource-faces-and-database.md
  - docs/plans/zircon_runtime/text/02-shaping-unicode-and-bidi.md
  - docs/plans/zircon_runtime/text/03-line-breaking-measure-and-layout.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - rustfmt --edition 2021 zircon_runtime/src/graphics/text/mod.rs zircon_runtime/src/graphics/text/layout/mod.rs zircon_runtime/src/graphics/text/layout/measure.rs zircon_runtime/src/ui/text/layout_engine.rs zircon_runtime/src/ui/text/measure_cache.rs zircon_runtime/src/ui/text/hit_test.rs zircon_runtime/src/ui/text/shaper.rs zircon_runtime/src/ui/text/layout_engine/tests.rs zircon_runtime/src/ui/tests/text_shaper.rs zircon_runtime/src/ui/tests/text_pipeline.rs zircon_runtime/src/ui/tests/text_hit_testing.rs (2026-06-28: passed)
  - cargo check -p zircon_runtime --lib --no-default-features --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-text-0628-check --message-format short --color never (2026-06-28: passed with existing warnings only)
  - rustfmt --edition 2021 zircon_runtime/src/graphics/text/font/database.rs zircon_runtime/src/graphics/scene/scene_renderer/ui/text.rs zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_font_bake.rs zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_render.rs zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_render/tests.rs (2026-06-28: passed)
  - rustfmt --edition 2021 --check zircon_runtime/src/graphics/text/font/database.rs zircon_runtime/src/graphics/scene/scene_renderer/ui/text.rs (2026-06-28: passed after system-font/composite candidate wiring)
  - cargo check -q -p zircon_runtime --lib --no-default-features --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-text-0628-check (2026-06-28: passed with existing warnings only; rerun after system-font/composite candidate wiring)
  - rustfmt --edition 2021 --check zircon_runtime/src/asset/assets/font.rs zircon_runtime/src/asset/importer/ingest/import_font_asset/mod.rs zircon_runtime/src/asset/importer/ingest/import_font_asset/parse_sfnt.rs zircon_runtime/src/graphics/text/font/database.rs zircon_runtime/src/graphics/scene/scene_renderer/ui/font_asset.rs zircon_runtime/src/graphics/scene/scene_renderer/ui/text.rs zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_font_bake.rs (2026-06-28: passed)
  - cargo metadata --locked --format-version 1 --no-default-features (2026-06-28: passed after local cache downloads)
  - rustfmt --edition 2021 --check zircon_runtime/src/graphics/scene/scene_renderer/ui/font_asset.rs zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_font_bake.rs zircon_runtime/src/graphics/text/font/database.rs zircon_runtime/src/asset/importer/ingest/import_font_asset/mod.rs zircon_runtime/src/asset/importer/ingest/import_font_asset/parse_sfnt.rs zircon_runtime/src/asset/assets/font.rs zircon_runtime/src/asset/tests/assets/font.rs (2026-06-28 FR-M2 render-strategy follow-up: passed)
  - cargo test -q -p zircon_runtime --lib render_strategy_default_mode_feeds_ui_font_default --no-default-features --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-text-0628-fr-m2-render-strategy (2026-06-28: timed out during compile with no Rust diagnostics; matching validation processes stopped)
  - rustfmt --edition 2021 --check zircon_runtime/src/graphics/text/font/mod.rs zircon_runtime/src/graphics/text/font/database.rs zircon_runtime/src/graphics/text/font/asset_registration.rs zircon_runtime/src/graphics/text/font/test_font_fixtures.rs zircon_runtime/src/graphics/scene/scene_renderer/ui/font_asset.rs zircon_runtime/src/graphics/scene/scene_renderer/ui/text.rs zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_font_bake.rs zircon_runtime/src/asset/assets/font.rs (2026-06-28 FR-M2 asset-registration follow-up: passed)
  - cargo test -q -p zircon_runtime --lib text_font_database_registers_font_asset_family_members_and_fallbacks --no-default-features --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-text-0628-fr-m2-register-asset-logical (2026-06-28: timed out during compile with no Rust diagnostics; matching validation processes stopped)
  - rustfmt --edition 2021 --check zircon_runtime/src/graphics/text/font/mod.rs zircon_runtime/src/graphics/text/font/database.rs zircon_runtime/src/graphics/text/font/coverage.rs zircon_runtime/src/graphics/text/font/asset_registration.rs zircon_runtime/src/graphics/text/font/test_font_fixtures.rs (2026-06-28 FR-M2/FB-M1 cmap-filter follow-up: passed)
  - cargo test -q -p zircon_runtime --lib text_font_fallback_candidates_filter_known_cmap_coverage --no-default-features --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-text-0628-fb-m1-cmap-filter (2026-06-28: timed out during compile with no Rust diagnostics; matching validation processes stopped)
  - cargo check -q -p zircon_runtime --lib --no-default-features --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-text-0628-check (2026-06-28 FR-M2 rerun: timed out during compile with no Rust diagnostics)
  - cargo test -p zircon_runtime --lib text_measurement_uses_backend_glyph_metrics --no-default-features --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-text-0628-check --message-format short --color never (2026-06-28: timed out during compile with no Rust diagnostics; no matching validation process left running)
  - cargo test -p zircon_runtime --lib text_font --no-default-features --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-text-0628-check --message-format short --color never (2026-06-28: timed out during compile with no Rust diagnostics; matching validation processes stopped)
  - rustfmt --edition 2021 --check zircon_runtime_interface/src/ui/surface/render/text_layout.rs zircon_runtime_interface/src/ui/surface/render/text_shape.rs zircon_runtime_interface/src/ui/surface/render/mod.rs zircon_runtime_interface/src/ui/surface/mod.rs zircon_runtime_interface/src/ui/surface/render/command.rs zircon_runtime_interface/src/tests/render_contracts.rs zircon_runtime_interface/src/tests/contracts.rs zircon_runtime/src/ui/text/layout_engine.rs zircon_runtime/src/ui/text/layout_engine/tests.rs zircon_runtime/src/graphics/scene/scene_renderer/ui/render/tests.rs zircon_editor/src/tests/host/retained_window/native_runtime_text_painter.rs (2026-06-28 SH/LB shaped glyph advance DTO follow-up: passed)
  - cargo check -q -p zircon_runtime_interface --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-text-0628-interface-contract (2026-06-28 SH/LB shaped glyph advance DTO follow-up: passed)
  - cargo check -q -p zircon_runtime_interface --tests --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-text-0628-interface-contract (2026-06-28 SH/LB shaped glyph advance DTO follow-up: passed)
  - cargo check -q -p zircon_runtime --lib --no-default-features --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-text-0628-check (2026-06-28 SH/LB shaped glyph advance DTO follow-up: timed out after 244s with no Rust diagnostics; matching validation processes stopped)
  - rustfmt --check zircon_runtime/src/graphics/text/shaping/mod.rs zircon_runtime/src/graphics/text/shaping/cosmic.rs zircon_runtime/src/graphics/text/shaping/line_break.rs zircon_runtime/src/graphics/text/shaping/tests.rs (2026-06-28 SH-M1 UAX#14 break flag projection: passed)
  - cargo check -p zircon_runtime --lib --no-default-features --locked --target-dir E:\cargo-targets\zircon-runtime-text-0628-check --message-format short --color never (2026-06-28 SH-M1 UAX#14 break flag projection: passed with existing warnings only)
  - cargo test -p zircon_runtime --lib --no-default-features --locked --target-dir E:\cargo-targets\zircon-runtime-text-0628-check --message-format short --color never text_shape_ (2026-06-28 SH-M1 UAX#14 break flag projection: passed, 6 passed)
  - rustfmt --check zircon_runtime/src/graphics/text/layout/mod.rs zircon_runtime/src/graphics/text/layout/line_break.rs zircon_runtime/src/ui/text/layout_engine.rs zircon_runtime/src/ui/text/layout_engine/tests.rs (2026-06-28 LB-M2 UAX#14 Word-wrap consumption: passed)
  - cargo test -p zircon_runtime --lib --no-default-features --locked --target-dir E:\cargo-targets\zircon-runtime-text-0628-check --message-format short --color never word_wrap_uses_uax14_cjk_break_opportunities (2026-06-28 LB-M2 UAX#14 Word-wrap consumption: passed, 1 passed)
  - cargo check -p zircon_runtime --lib --no-default-features --locked --target-dir E:\cargo-targets\zircon-runtime-text-0628-check --message-format short --color never (2026-06-28 LB-M2 UAX#14 Word-wrap consumption: passed with existing warnings only)
  - rustfmt --edition 2021 --check zircon_runtime/src/core/framework/render/mod.rs zircon_runtime/src/graphics/text/shaping/cosmic.rs zircon_runtime/src/graphics/text/shaping/mod.rs zircon_runtime/src/graphics/text/layout/measure.rs (2026-06-28 render text facade import repair: passed)
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
doc_type: module-detail
---

# Runtime Graphics Text

## Purpose

`zircon_runtime::graphics::text` is the runtime implementation owner for shared font, shaping, line breaking, text measurement, and future text layout services. It keeps third-party text stack types inside graphics implementation files while UI, render extraction, editor, and app code consume neutral Zircon DTOs.

`graphics/text/font` is the FR-M1/FR-M2 font owner. It provides a crate-private `FontDatabase`, default runtime fallback family data, best-match queries by neutral `FontQuery`, shared `Arc<[u8]>` face bytes, stable variation-instance ids, source-path+face-index deduplicated font-file registration, selected-face metadata ingestion, system font discovery through glyphon/fontdb, CompositeFont sub-font candidate enumeration, cmap-aware candidate filtering, and injection of registered faces into glyphon `FontSystem`. `asset_registration.rs` owns the `.font.toml` family-member projection and logical asset-face key; `coverage.rs` owns sfnt cmap coverage extraction and permissive Unknown coverage semantics so the database owner stays focused on storage, indexes, and matching. The neutral DTOs live under `core/framework/render/text/font`; implementation details remain crate-private under `graphics/text/font`.

FR-M2 now adds the first font-asset metadata slice. `.font.toml` import parses sfnt/TTC metadata into `FontAssetMetadata`: face count, per-face name data, OS/2 weight and width class, style, `fvar` variation axes and named instances, and compact cmap coverage ranges. Runtime UI font manifests carry `face_index` into the shared `FontDatabase`, so native glyphon registration keys can distinguish faces in one source file. `FontDatabase::register_font_file` also reads the selected face's family, weight, style, and stretch from the source bytes, so project font best-match is no longer forced to Regular when the manifest omits style metadata. `FontDatabase::register_font_asset` consumes imported family-member descriptors and fallback families; its asset source key includes family, style, weight, stretch, and variation coordinates, so multiple logical instances can share one physical face without being collapsed. Native/SDF manifest paths now use that asset-registration entry for `.font.toml` manifests. `font_asset.rs` consumes `FontAsset.render_strategy.default_mode` as the UI font default after the legacy `render_mode` field, and clamps that default through `allow_native` / `allow_sdf` before the renderer sees it.

`graphics/text/shaping` is now the SH-M1 shaping owner. `cosmic.rs` is the only runtime text implementation file that directly touches glyphon/cosmic-text `FontSystem`, `Buffer`, `LayoutRun`, and `LayoutGlyph`; it projects backend glyph id, source range, visual range, advance, baseline, direction, cluster flags, and rotation into neutral `core/framework/render/text::{ShapedGlyphRun, ShapedTextLine, ShapedGlyph}` contracts. `shaping/line_break.rs` is the UAX#14 line-break opportunity leaf; it uses `unicode-linebreak` and feeds soft/mandatory break flags into cluster-start glyphs without moving full wrapping, kinsoku, or justification logic into the shaping root. `graphics/text/layout/measure.rs` consumes that shaped run for line width, line metrics, and per-grapheme advances instead of importing third-party text backend types itself. `graphics/text/layout/line_break.rs` is the first LB-M2 layout consumer: it shapes an unconstrained segment, reads neutral cluster soft-break flags, and exposes `line_break_chunks(...)` so UI Word wrap can break CJK text without ASCII spaces. `graphics/text/layout/kinsoku.rs` applies first CJK line-start and line-end prohibition slices by merging or marking forbidden punctuation chunks as non-glyph-fallback chunks; UI code only respects the `LineBreakChunk::allow_glyph_fallback` metadata and does not own punctuation tables. Current `font_id` values remain `None` until the runtime `FontDatabase` and shaping `FontSystem` share a stable face-id bridge.

LB-M2 now also has soft-hyphen and NBSP metadata slices. `LineBreakChunk` carries an explicit `source_range` plus optional break suffix metadata, so U+00AD can stay out of visual chunk text while still mapping the visible wrap-time `-` back to the source soft-hyphen range. Chunks containing U+00A0 are marked as non-glyph-fallback chunks, which preserves NBSP as glue when a narrow Word-wrap frame would otherwise split the group by grapheme. The line-break owner provides the neutral metadata; UI decides only whether a real width wrap occurred before displaying a suffix or invoking glyph fallback.

LB-M3 now has its first logical alignment slices. `UiTextAlign` keeps `Start` and `End` as interface-level values, the surface resolver preserves those logical values, and `layout_engine.rs` resolves explicit, Auto, and current `Mixed` requests to a concrete paragraph base direction before placing text. `ScreenSpaceUiTextBatch` carries `text_direction` into native glyphon and SDF draw planning; `native_text_align(...)` and `aligned_text_start_x(...)` map Start/End through the same direction rule, so render backends do not reinterpret RTL text as physical left/right alignment. The current SH-M2 mirror-table slice lives in UI `layout_engine/visual_order.rs` as an interim scaffold and proves source-range-stable mirrored RTL punctuation; full UAX#9 level runs, isolates, script segmentation, and backend-owned mirroring still belong to the later shaping/BiDi owner.

## Boundaries

`graphics/text/mod.rs`, `graphics/text/font/mod.rs`, `graphics/text/shaping/mod.rs`, and `graphics/text/layout/mod.rs` are crate-private owner modules. They are intentionally not public facade exports from `graphics/mod.rs`.

Allowed consumers in this slice:

- `ui/text/layout_engine.rs` asks for text size, line metrics, line width, and ellipsis fit.
- `ui/text/font_registry.rs` asks the runtime font database for the default fallback family chain.
- `ui/text/measure_cache.rs` asks for a wrapped width bucket.
- `ui/text/hit_test.rs` asks for measured grapheme widths.
- `ui/text/shaper.rs` records `SharedTextService` as the active layout backend for Native and SDF render-mode intents.
- `graphics/scene/scene_renderer/ui/render.rs` propagates resolved text direction into every screen-space UI text batch.
- `graphics/scene/scene_renderer/ui/text.rs` owns the screen-space UI text-system instance, loads native glyphon font assets through `FontDatabase`, and maps logical Start/End through text direction before calling glyphon.
- `graphics/scene/scene_renderer/ui/sdf_font_bake.rs` and `sdf_render.rs` resolve SDF font assets through `FontDatabase`, build fontsdf faces from shared face bytes, and map logical Start/End through text direction in the SDF draw plan.
- `asset/importer/ingest/import_font_asset/{mod.rs,parse_sfnt.rs}` parses font source metadata during asset import without exposing parser types to UI or renderer surfaces.

Third-party shaping types such as `FontSystem`, `Buffer`, `LayoutRun`, `LayoutGlyph`, `Metrics`, `Attrs`, and `Family` stay in `graphics/text/shaping/cosmic.rs`. Unicode line-break classification stays in `graphics/text/shaping/line_break.rs` and leaves only neutral boolean flags on `ShapedGlyphClusterFlags`; `graphics/text/layout/line_break.rs` consumes those flags rather than re-importing Unicode break libraries into UI. CJK line-start punctuation policy stays in `graphics/text/layout/kinsoku.rs` and leaves only neutral chunk metadata for UI Word wrap. Soft-hyphen detection and break-suffix metadata also stay in `graphics/text/layout/line_break.rs`; UI must not re-scan U+00AD to invent its own hyphenation state. Font database implementation state and face bytes stay in `graphics/text/font`; layout measurement consumes neutral shaped-run contracts from `graphics/text/shaping`. Logical Start/End alignment is part of the neutral UI typography contract, but backend mapping belongs in the native/SDF renderer leaves after `ScreenSpaceUiTextBatch` has carried `text_direction`. These types must not leak into editor/app surfaces or public graphics facade exports.

`core/framework/render/mod.rs` is allowed to re-export the neutral text DTOs from the child `render/text` owner so runtime implementation leaves can import `core::framework::render::{ShapedGlyphRun, TextShapingService, ...}` without reaching into the private `render::text` module path. That facade remains DTO-only; glyphon/cosmic-text implementation state stays in `graphics/text/shaping`.

## Current Guarantees

The font database owner removes the UI-local hardcoded default fallback family list; UI now consumes the runtime text owner for default fallback families. Native glyphon font loading and SDF bake/render font loading now share the same source-path registration table and `Arc<[u8]>` face bytes, so project font assets are no longer independently read by each backend. System font faces are indexed under Zircon `FontFaceId` values for query and fallback ordering, while their `fontdb::Source` remains internal and is only used when injecting into glyphon.

CompositeFont sub-font script/range data now has a candidate enumeration path in `FontDatabase`. It is a data-plane slice: it orders candidates by sub-font match, default family, request family, then runtime fallback chain. For project fonts whose sfnt cmap can be parsed, `coverage.rs` records compact codepoint ranges and `fallback_candidates` excludes known non-covering faces for the requested codepoint; Unknown coverage remains permissive for system fonts and synthetic test faces. Tofu behavior, cluster-level consistency, diagnostics, and deep fallback policy remain owned by text plan 06.

Imported `FontAsset` records now preserve source metadata and parsed family members. This is still a metadata/data-plane step: `FontDatabase` consumes selected-face file metadata for direct font registration, imported `family_members` and `fallback_families` through `register_font_asset`, and the UI manifest loader consumes `render_strategy.default_mode`. WOFF2 decode is explicitly not wired. The SDF `fontsdf` backend still cannot rasterize non-zero collection faces, so `sdf_font_bake.rs` now skips unsupported face indices and falls back to the default font instead of silently rendering face 0. True multi-face TTC SDF rendering remains open until the raster path accepts an explicit face index.

The shared measurement owner removes the old UI-wide fixed half-em equal-width path from measurement, wrapping decisions, cache buckets, hit-test grapheme midpoint calculation, shaped glyph frame projection, rich text paint-run frames, and editable caret/selection/composition decoration placement. `UiShapedGlyph` now carries render-facing `font_id`, cluster flags, and rotation placeholders so the current neutral projection can grow into the full SH-M1/SH-M3 contract without adding compatibility shims later.

This is still a Stage A slice, not the complete text architecture. The first `ShapedGlyphRun` contract and cosmic-backed owner are in place, UAX#14 break opportunities are projected as cluster flags, Word wrap now consumes the first shared line-break chunks for CJK no-space text, LB-M2 has first CJK line-start/line-end prohibition slices, soft hyphen now inserts a visible `-` only when the selected wrap break uses U+00AD, NBSP glue groups resist glyph fallback splitting, LB-M3 preserves logical Start/End through layout and native/SDF draw planning for explicit RTL plus Auto/Mixed first-strong base direction, and the current UI visual-order scaffold mirrors single-codepoint RTL punctuation without source-range drift. Actual fallback-selected `FontFaceId`, script-aware fallback runs, full UAX#9 level/isolate behavior, backend-owned mirroring, full greedy line breaking, complete CJK line-head/line-tail kinsoku tables, complete non-breaking-space policy, justification, shrink/clamp, ellipsis variants, vertical layout beyond a rotation contract, atlas ownership, SDF/MSDF policy, rich text, IME candidate geometry, and multi-threaded caches remain governed by `docs/plans/zircon_runtime/text/01` through `09`.

## Validation

Focused tests were updated to reject equal-width measurement (`WWW` must measure wider than `iii`), keep combining-mark ellipsis on grapheme boundaries, use measured cache buckets, hit-test by measured grapheme widths, export backend grapheme advances into resolved lines, project non-uniform advances into shaped glyph frames, query best-match font weight, prove shared `Arc<[u8]>` face bytes, keep variation-instance hashes stable, verify a registered font file can be reused while feeding glyphon fontdb, ensure CompositeFont CJK candidates precede the default Latin face, filter known non-covering cmap faces from fallback candidates, parse imported TTF metadata/cmap coverage, enumerate test-built TTC faces, verify direct `FontDatabase` registration uses file weight metadata and TTC face indices for best-match, verify `FontAsset` family members and fallback families are registered by the database, verify UI font manifests consume `render_strategy.default_mode` after legacy `render_mode`, and verify SDF fallback when a requested non-zero face index cannot be opened by `fontsdf`.

Cargo validation on 2026-06-28 passed for `zircon_runtime --lib --no-default-features` with existing warnings only after the shaping owner slice and again for the LB-M3 logical Start/End plus first-strong target-dirs, and `zircon_runtime_interface --lib` plus `zircon_runtime_interface --tests` passed after the SH/LB shaped glyph advance DTO follow-up. After retargeting stale typed-error include guards, updating the camera-loop tests to the current frame-submission callback signature, and fixing `plugin_importer_dx.rs` to point at its current child module file, focused `cargo test -p zircon_runtime --lib --no-default-features --locked --target-dir E:\cargo-targets\zircon-runtime-text-0628-check text_shape_` passes 6/6. The six tests cover source-range monotonicity, RTL/space/tab flags, backend glyph advance variation, ligature cluster coverage, word-space UAX#14 soft breaks, and CJK UAX#14 soft breaks. `word_wrap_uses_uax14_cjk_break_opportunities` proves the first LB-M2 UI consumer wraps narrow CJK Word-mode text using shared UAX#14 break chunks instead of ASCII-space splitting. `text_wrap_cjk_kinsoku_no_leading_punctuation` now proves `"中文。"` lays out as `"中"` then `"文。"` in a narrow Word-wrap frame, with no resolved line starting with `。`. `text_wrap_soft_hyphen_inserts_hyphen` proves `"pre\u{00ad}fix"` wraps as `"pre-"` then `"fix"` without retaining U+00AD in visual text, and `text_hit_test_soft_hyphen_break_suffix_maps_to_source_hyphen` proves the visible break suffix maps back to the source soft-hyphen range. The LB-M3 focused set proves layout Start/End follows explicit RTL and Auto/Mixed first-strong base direction, native glyphon maps Start/End through `text_direction`, SDF draw planning maps the same semantics even when glyph quads are clipped to the frame, and render extraction preserves logical `start` instead of collapsing it to physical left. The SH-M2 mirror focused tests `text_bidi_mirrors_paren_in_rtl` and `text_bidi_mirrors_arrow_in_rtl` pass 2/2 and prove mirrored RTL punctuation keeps the original source byte ranges. The validation path also exposed a moved-value support bug in `mesh_pipeline_variant_cache_owner.rs`; changing the string aggregation to borrow inputs restored the structure guard, and `runtime_15_non_base_mesh_variant_cache_owner_is_wired` passes 1/1. Screenshot evidence includes `docs/tests/runtime/text/runtime_text_shared_metrics_preview_20260628.png`, `docs/tests/runtime/text/runtime_text_cjk_kinsoku_preview_20260628.png`, `docs/tests/runtime/text/runtime_text_soft_hyphen_preview_20260628.png`, `docs/tests/runtime/text/runtime_text_long_word_nbsp_preview_20260628.png`, `docs/tests/runtime/text/runtime_text_cjk_open_punctuation_preview_20260628.png`, `docs/tests/runtime/text/runtime_text_rtl_start_end_alignment_preview_20260628.png`, `docs/tests/runtime/text/runtime_text_first_strong_direction_preview_20260628.png`, and `docs/tests/runtime/text/runtime_text_rtl_mirrored_punctuation_preview_20260628.png`; matching target-path checks confirm no text verification image was written under repo `target`.

