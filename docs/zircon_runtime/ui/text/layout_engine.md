---
related_code:
  - zircon_runtime/src/ui/text/layout_engine.rs
  - zircon_runtime/src/ui/text/layout_engine/candidate_line.rs
  - zircon_runtime/src/ui/text/layout_engine/direction.rs
  - zircon_runtime/src/ui/text/layout_engine/ellipsis.rs
  - zircon_runtime/src/ui/text/layout_engine/line_box.rs
  - zircon_runtime/src/ui/text/layout_engine/overflow_style.rs
  - zircon_runtime/src/ui/text/layout_engine/paragraph_layout.rs
  - zircon_runtime/src/ui/text/layout_engine/range_mapping.rs
  - zircon_runtime/src/ui/text/layout_engine/vertical.rs
  - zircon_runtime/src/ui/text/layout_engine/visual_order.rs
  - zircon_runtime/src/ui/text/layout_engine/wrapping.rs
  - zircon_runtime/src/graphics/text/layout/kinsoku.rs
  - zircon_runtime/src/graphics/text/layout/line_break/greedy.rs
  - zircon_runtime/src/graphics/text/layout/line_break/smart.rs
  - zircon_runtime/src/ui/text/hit_test.rs
  - zircon_runtime/src/ui/surface/input/text_pointer.rs
  - zircon_runtime/src/ui/text/layout_engine/tests.rs
  - zircon_runtime/src/ui/text/layout_engine/tests/alignment.rs
  - zircon_runtime/src/ui/text/layout_engine/tests/bidi.rs
  - zircon_runtime/src/ui/text/layout_engine/tests/glue.rs
  - zircon_runtime/src/ui/text/layout_engine/tests/grapheme.rs
  - zircon_runtime/src/ui/text/layout_engine/tests/justify.rs
  - zircon_runtime/src/ui/text/layout_engine/tests/kinsoku.rs
  - zircon_runtime/src/ui/text/layout_engine/tests/measure.rs
  - zircon_runtime/src/ui/text/layout_engine/tests/overflow.rs
  - zircon_runtime/src/ui/text/layout_engine/tests/sizing.rs
  - zircon_runtime/src/ui/text/layout_engine/tests/soft_hyphen.rs
  - zircon_runtime/src/ui/text/layout_engine/tests/tab.rs
  - zircon_runtime/src/ui/text/layout_engine/tests/vertical.rs
  - zircon_runtime/src/ui/text/layout_engine/tests/word_smart.rs
  - zircon_runtime/src/ui/text/layout_engine/tests/wrap_space.rs
  - zircon_runtime/src/ui/text/layout_engine/tests/wrapping.rs
  - zircon_runtime/src/ui/text/grapheme.rs
  - zircon_runtime/src/ui/text/rich_text.rs
  - zircon_runtime/src/ui/text/resolved_layout.rs
  - zircon_runtime/src/ui/tests/text_pipeline/measure_cache.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/render/tests/parity.rs
  - zircon_runtime/src/ui/tests/text_hit_testing.rs
  - zircon_runtime/src/ui/tests/text_layout
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_render/tests/mod.rs
  - zircon_runtime_interface/src/ui/surface/render/command.rs
  - zircon_runtime_interface/src/ui/surface/render/resolved_style.rs
  - zircon_runtime_interface/src/ui/surface/render/text_geometry.rs
  - zircon_runtime_interface/src/ui/surface/render/text_shape.rs
  - zircon_runtime_interface/src/ui/surface/render/text_layout.rs
  - zircon_runtime_interface/src/ui/surface/render/typography.rs
  - zircon_runtime_interface/src/ui/surface/mod.rs
  - zircon_runtime_interface/src/tests/render_contracts.rs
  - zircon_runtime/src/ui/tests/widget_text_input_pointer.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/render_commands.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text.rs
implementation_files:
  - zircon_runtime/src/ui/text/layout_engine.rs
  - zircon_runtime/src/ui/text/layout_engine/candidate_line.rs
  - zircon_runtime/src/ui/text/layout_engine/direction.rs
  - zircon_runtime/src/ui/text/layout_engine/ellipsis.rs
  - zircon_runtime/src/ui/text/layout_engine/line_box.rs
  - zircon_runtime/src/ui/text/layout_engine/overflow_style.rs
  - zircon_runtime/src/ui/text/layout_engine/paragraph_layout.rs
  - zircon_runtime/src/ui/text/layout_engine/range_mapping.rs
  - zircon_runtime/src/ui/text/layout_engine/vertical.rs
  - zircon_runtime/src/ui/text/layout_engine/visual_order.rs
  - zircon_runtime/src/ui/text/layout_engine/wrapping.rs
  - zircon_runtime/src/graphics/text/layout/kinsoku.rs
  - zircon_runtime/src/graphics/text/layout/line_break/greedy.rs
  - zircon_runtime/src/graphics/text/layout/line_break/smart.rs
  - zircon_runtime/src/ui/text/hit_test.rs
  - zircon_runtime/src/ui/surface/input/text_pointer.rs
  - zircon_runtime/src/ui/text/grapheme.rs
  - zircon_runtime/src/ui/text/rich_text.rs
  - zircon_runtime/src/ui/text/resolved_layout.rs
  - zircon_runtime/src/ui/tests/text_pipeline/measure_cache.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/render/tests/parity.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_render/tests/mod.rs
  - zircon_runtime_interface/src/ui/surface/render/command.rs
  - zircon_runtime_interface/src/ui/surface/render/resolved_style.rs
  - zircon_runtime_interface/src/ui/surface/render/text_geometry.rs
  - zircon_runtime_interface/src/ui/surface/render/text_shape.rs
  - zircon_runtime_interface/src/ui/surface/render/text_layout.rs
  - zircon_runtime_interface/src/ui/surface/render/typography.rs
  - zircon_runtime_interface/src/ui/surface/mod.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/render_commands.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text.rs
plan_sources:
  - .codex/plans/Zircon UI 与 Unreal Slate 差异审计及后续里程碑.md
  - docs/plans/zircon_runtime/text/03-line-breaking-measure-and-layout.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
  - user: 2026-05-06 continue M6 text-system convergence from Unreal Slate audit
tests:
  - rustfmt --edition 2021 --check zircon_runtime/src/ui/text/layout_engine/tests.rs zircon_runtime/src/ui/text/layout_engine/tests/*.rs (2026-07-03 layout_engine private tests owner split: passed)
  - docs/tests/runtime/text/runtime_text_layout_engine_tests_owner_split_preview_20260703.png (2026-07-03 layout_engine private tests owner split visual proof: inspected; SHA256 30B7801223DECD6797C6262138ABAEBDB5A9576EA97C866577B498992B1EE223; repo target, D:\cargo-targets, and E:\cargo-targets same-name match count 0)
  - docs/tests/runtime/text/runtime_text_layout_engine_tests_owner_split_validation_20260703.log (2026-07-03 layout_engine private tests owner split validation log: SHA256 F29A7E76D57A473EE1F9563A2D878FE2583709675CDA2F15912D57BB697C8D3D; focused Cargo deferred because external cargo/rustc lanes were active)
  - rustfmt --edition 2021 --check zircon_runtime/src/graphics/scene/scene_renderer/ui/render.rs zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_render.rs zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_render/tests/mod.rs zircon_runtime/src/graphics/scene/scene_renderer/ui/text.rs zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_atlas/tests (2026-07-02 LB-M4/SM-M1 vertical_rl SDF projection slice: passed)
  - cargo test -p zircon_runtime sdf_draw_plan_vertical_rl_advances_glyphs_on_y_axis --lib --no-default-features --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-text-0702-sdf-vertical --message-format short --color never -- --nocapture --test-threads=1 (2026-07-02 LB-M4/SM-M1 vertical_rl SDF projection focused test: passed 1/1)
  - docs/tests/runtime/text/runtime_text_vertical_rl_sdf_vertex_projection_preview_20260702.png (2026-07-02 LB-M4/SM-M1 vertical_rl SDF projection visual proof: inspected; SHA256 86C7CEA59DBAB17A63B6FB61C2FF72C591B4E7216439FE0AABF97C793E5BC1C1; repo target, D:\cargo-targets, and E:\cargo-targets same-name match count 0)
  - rustfmt --edition 2021 --check zircon_runtime_interface/src/ui/surface/render/text_geometry.rs zircon_runtime_interface/src/tests/render_contracts.rs (2026-07-01 LB-M4 vertical_rl edit decoration render-contract slice: passed)
  - cargo test -p zircon_runtime_interface ui_text_decorations_use_vertical_rl_geometry --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-text-0701-vertical-decorations-interface --message-format short --color never -- --nocapture --test-threads=1 (2026-07-01 LB-M4 vertical_rl edit decoration render-contract focused test: passed 1/1)
  - docs/tests/runtime/text/runtime_text_vertical_rl_edit_decoration_geometry_preview_20260701.png (2026-07-01 LB-M4 vertical_rl edit decoration visual proof: inspected; SHA256 A856E741CC5722DE9BD06A924E887657F2DBDB9301073F4B0172C020CE50A044; repo target, D:\cargo-targets, and E:\cargo-targets same-name match count 0)
  - cargo check -p zircon_runtime --lib --no-default-features --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-text-0701-vertical-geometry-check --message-format short --color never (2026-07-01 LB-M4 vertical_rl hit-test/caret/IME geometry slice: passed with existing warnings only)
  - E:\cargo-targets\zircon-plan08-three-shading-model-parity-0701\debug\deps\zircon_runtime-fe15dbfd02d9864e.exe text_hit_test_vertical_rl --nocapture --test-threads=1 (2026-07-01 LB-M4 vertical_rl hit-test geometry focused test: passed 1/1; separate cargo --no-run lane in E:\cargo-targets\zircon-runtime-text-0701-vertical-geometry-check timed out and was stopped, not counted)
  - E:\cargo-targets\zircon-plan08-three-shading-model-parity-0701\debug\deps\zircon_runtime-fe15dbfd02d9864e.exe source_geometry_uses_vertical --nocapture --test-threads=1 (2026-07-01 LB-M4 vertical_rl caret/range geometry focused test: passed 1/1)
  - E:\cargo-targets\zircon-plan08-three-shading-model-parity-0701\debug\deps\zircon_runtime-fe15dbfd02d9864e.exe cursor_rect_uses_vertical_rl --nocapture --test-threads=1 (2026-07-01 LB-M4 vertical_rl IME cursor rect focused test: passed 1/1)
  - docs/tests/runtime/text/runtime_text_vertical_rl_hit_test_geometry_preview_20260701.png (2026-07-01 LB-M4 vertical_rl hit-test/caret/IME visual proof: inspected; SHA256 6B137ECAA345CF7EEA898032AD93B5E2292B16563AC40B43C136684740BA1E6B; repo target, D:\cargo-targets, and E:\cargo-targets same-name match count 0)
  - cargo test -p zircon_runtime_interface vertical --locked --no-default-features --target-dir E:\cargo-targets\zircon-runtime-text-0701-vertical-writing-mode --message-format short --color never -- --nocapture --test-threads=1 (2026-07-01 LB-M4 vertical_rl interface/shaped contract: passed 2/2)
  - E:\cargo-targets\zircon-runtime-text-0701-vertical-writing-mode\debug\deps\zircon_runtime-fe15dbfd02d9864e.exe vertical_rl --nocapture --test-threads=1 (2026-07-01 LB-M4 vertical_rl runtime layout/render-extract tests: passed 2/2 after Cargo wrapper timeout produced the binary)
  - E:\cargo-targets\zircon-runtime-text-0701-vertical-writing-mode\debug\deps\zircon_runtime-fe15dbfd02d9864e.exe text_vertical_kinsoku --nocapture --test-threads=1 (2026-07-01 LB-M4 vertical kinsoku runtime test: passed 1/1)
  - docs/tests/runtime/text/runtime_text_vertical_rl_layout_preview_20260701.png (2026-07-01 LB-M4 vertical_rl visual proof: inspected; SHA256 7E2797C4D3D71DA40233B944521468167E995CEAD722E1CB3DA75140E2F1031F; repo target, D:\cargo-targets, and E:\cargo-targets same-name match count 0)
  - rustfmt --edition 2021 --check zircon_runtime/src/graphics/text/layout/kinsoku.rs zircon_runtime/src/ui/text/layout_engine/tests/kinsoku.rs (2026-07-01 LB-M2 JLREQ hyphen kinsoku table slice: passed)
  - cargo check -p zircon_runtime --lib --no-default-features --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-text-0701-jlreq-hyphen-kinsoku --message-format short --color never --quiet (2026-07-01 LB-M2 JLREQ hyphen kinsoku table slice: passed with existing warnings only after first 300s tool-window timeout completed in background and incremental rerun passed)
  - cargo test -p zircon_runtime jlreq_hyphen --lib --no-default-features --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-text-0701-jlreq-hyphen-kinsoku --message-format short --color never --no-run (2026-07-01 LB-M2 JLREQ hyphen kinsoku lib-test binary generation: passed with existing warnings after earlier no-diagnostic compile exits)
  - E:\cargo-targets\zircon-runtime-text-0701-jlreq-hyphen-kinsoku\debug\deps\zircon_runtime-fe15dbfd02d9864e.exe jlreq_hyphen --nocapture --test-threads=1 (2026-07-01 LB-M2 JLREQ hyphen kinsoku focused runtime tests: passed 2/2)
  - docs/tests/runtime/text/runtime_text_jlreq_hyphen_kinsoku_preview_20260701.png (2026-07-01 LB-M2 JLREQ hyphen kinsoku visual proof: inspected; SHA256 4D45E10D708A8A13DDA9FEC3C06F5681D122306DEE8521A31DAB874434788ABE; repo target and E:\cargo-targets same-name match count 0)
  - rustfmt --edition 2021 --check zircon_runtime/src/graphics/text/layout/kinsoku.rs zircon_runtime/src/ui/text/layout_engine/tests/kinsoku.rs (2026-07-01 LB-M2 spacing voicing mark kinsoku table slice: passed)
  - cargo check -p zircon_runtime --lib --no-default-features --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-text-0701-spacing-voicing-kinsoku --message-format short --color never --quiet (2026-07-01 LB-M2 spacing voicing mark kinsoku table slice: passed with existing warnings only after first 300s tool-window timeout completed in background and incremental rerun passed)
  - E:\cargo-targets\zircon-runtime-text-0701-spacing-voicing-kinsoku\debug\deps\zircon_runtime-fe15dbfd02d9864e.exe kinsoku --nocapture --test-threads=1 (2026-07-01 LB-M2 spacing voicing mark kinsoku focused runtime tests: passed 27/27 after cargo test compile/link timeout produced the binary)
  - docs/tests/runtime/text/runtime_text_spacing_voicing_mark_kinsoku_preview_20260701.png (2026-07-01 LB-M2 spacing voicing mark kinsoku visual proof: inspected; SHA256 82C63DFE03B83C03F87E9B1420D1404DE5E348051273561A439EA64EC3237599; repo target and E:\cargo-targets same-name match count 0)
  - rustfmt --edition 2021 --check zircon_runtime/src/graphics/text/layout/kinsoku.rs zircon_runtime/src/ui/text/layout_engine/tests/kinsoku.rs (2026-07-01 LB-M2 fullwidth white parenthesis kinsoku table slice: passed)
  - cargo check -p zircon_runtime --lib --no-default-features --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-text-0701-fullwidth-white-parenthesis-kinsoku --message-format short --color never --quiet (2026-07-01 LB-M2 fullwidth white parenthesis kinsoku table slice: passed in fresh 300s verification with existing warnings only)
  - E:\cargo-targets\zircon-runtime-text-0701-fullwidth-white-parenthesis-kinsoku\debug\deps\zircon_runtime-fe15dbfd02d9864e.exe kinsoku --nocapture --test-threads=1 (2026-07-01 LB-M2 fullwidth white parenthesis kinsoku focused runtime tests: passed 25/25)
  - docs/tests/runtime/text/runtime_text_fullwidth_white_parenthesis_kinsoku_preview_20260701.png (2026-07-01 LB-M2 fullwidth white parenthesis kinsoku visual proof: inspected; SHA256 4946765AE3C692066A4C04E38671F98D78B42C281AB6904C2A4A6E464E1AB52D; repo target and E:\cargo-targets same-name match count 0)
  - rustfmt --edition 2021 --check zircon_runtime/src/graphics/text/layout/kinsoku.rs zircon_runtime/src/ui/text/layout_engine/tests/kinsoku.rs (2026-07-01 LB-M2 CJK double-prime closing quote kinsoku table slice: passed)
  - cargo check -p zircon_runtime --lib --no-default-features --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-text-0701-cjk-double-prime-kinsoku --message-format short --color never --quiet (2026-07-01 LB-M2 CJK double-prime closing quote kinsoku table slice: passed with existing warnings only after a non-reproduced first-run asset::pack import state drift)
  - docs/tests/runtime/text/runtime_text_cjk_double_prime_closing_quote_kinsoku_preview_20260701.png (2026-07-01 LB-M2 CJK double-prime closing quote kinsoku visual proof: inspected; SHA256 EC77D5A02F5D0FE4E9A8D8E782ED90C29F081691B8BC6AD3EA9A06A3FC39C032; repo target and E:\cargo-targets same-name match count 0)
  - rustfmt --edition 2021 --check zircon_runtime/src/graphics/text/layout/kinsoku.rs zircon_runtime/src/ui/text/layout_engine/tests/kinsoku.rs (2026-07-01 LB-M2 Katakana phonetic extension small-kana kinsoku table slice: passed)
  - cargo check -p zircon_runtime --lib --no-default-features --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-text-0701-katakana-phonetic-kinsoku --message-format short --color never --quiet (2026-07-01 LB-M2 Katakana phonetic extension small-kana kinsoku table slice: passed with existing warnings only)
  - docs/tests/runtime/text/runtime_text_katakana_phonetic_extension_kinsoku_preview_20260701.png (2026-07-01 LB-M2 Katakana phonetic extension small-kana kinsoku visual proof: inspected; SHA256 44AD05A1739BBC75B7FA9D9B2FA356EB6F394DA90453B76ED1DDC11BCFB6A6DA; repo target and E:\cargo-targets same-name match count 0)
  - rustfmt --edition 2021 --check zircon_runtime/src/graphics/text/layout/kinsoku.rs zircon_runtime/src/ui/text/layout_engine/tests/kinsoku.rs (2026-07-01 LB-M2 CJK white bracket kinsoku table slice: passed)
  - cargo check -p zircon_runtime --lib --no-default-features --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-text-0701-cjk-white-bracket-kinsoku --message-format short --color never --quiet (2026-07-01 LB-M2 CJK white bracket kinsoku table slice: passed with existing warnings only)
  - docs/tests/runtime/text/runtime_text_cjk_white_bracket_kinsoku_preview_20260701.png (2026-07-01 LB-M2 CJK white bracket kinsoku visual proof: inspected; SHA256 9CB6BBB21F385F1D889B73B5EE900360C2AD5E9058D48D7F73AA43618983FFAF; repo target and E:\cargo-targets same-name match count 0)
  - rustfmt --edition 2021 --check zircon_runtime/src/graphics/text/layout/kinsoku.rs zircon_runtime/src/ui/text/layout_engine/tests/kinsoku.rs (2026-07-01 LB-M2 small ka/ke kinsoku + panic-free handoff slice: passed)
  - cargo check -p zircon_runtime --lib --no-default-features --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-text-0701-small-ka-ke-kinsoku --message-format short --color never --quiet (2026-07-01 LB-M2 small ka/ke kinsoku + panic-free handoff slice: passed with existing warnings only)
  - docs/tests/runtime/text/runtime_text_small_ka_ke_kinsoku_preview_20260701.png (2026-07-01 LB-M2 small ka/ke kinsoku visual proof: inspected; SHA256 C13AA26D914D75A370FED02C8FD87AD784080281F348C33D0BD5CB653D1FD381; repo target and E:\cargo-targets same-name match count 0)
  - rustfmt --edition 2021 --check zircon_runtime/src/graphics/text/layout/line_break/greedy.rs zircon_runtime/src/graphics/text/layout/line_break/mod.rs zircon_runtime/src/graphics/text/layout/mod.rs zircon_runtime/src/ui/text/layout_engine/wrapping.rs (2026-06-30 LB-M2 greedy wrap decision owner split: passed)
  - cargo check -p zircon_runtime --lib --no-default-features --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-text-0630-greedy-wrap-decision --message-format short --color never --quiet (2026-06-30 LB-M2 greedy wrap decision owner split: passed with existing warnings only)
  - docs/tests/runtime/text/runtime_text_greedy_wrap_decision_owner_preview_20260630.png (2026-06-30 LB-M2 greedy wrap decision owner visual proof: inspected; SHA256 C8CA68364812DC5C93D47ED2861131ABF0FAD4A258ED45E021D7644C2EF78278; repo target and E:\cargo-targets same-name match count 0)
  - rustfmt --edition 2021 --check zircon_runtime_interface/src/ui/surface/render/typography.rs zircon_runtime_interface/src/tests/contracts.rs zircon_runtime/src/ui/surface/render/resolve.rs zircon_runtime/src/ui/text/layout_engine/wrapping.rs zircon_runtime/src/graphics/scene/scene_renderer/ui/text.rs zircon_runtime/src/ui/tests/text_layout zircon_runtime/src/ui/tests/widget_text_input_ime_context.rs (2026-06-30 LB-M2 WordSmart wrap contract entry: passed)
  - cargo check -p zircon_runtime_interface --lib --tests --locked --target-dir E:\cargo-targets\zircon-runtime-text-0630-word-smart-interface --message-format short --color never --quiet (2026-06-30 LB-M2 WordSmart interface contract: passed)
  - cargo check -p zircon_runtime --lib --no-default-features --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-text-0630-word-smart-runtime --message-format short --color never --quiet (2026-06-30 LB-M2 WordSmart runtime contract: passed with existing warnings only)
  - cargo test -p zircon_runtime word_smart --lib --no-default-features --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-text-0630-word-smart-runtime --message-format short --color never -- --nocapture --test-threads=1 (2026-06-30 LB-M2 WordSmart focused runtime tests: timed out after 904s during Windows lib-test compile/link with no Rust diagnostics; matching cargo/rustc processes stopped; not counted as passing)
  - docs/tests/runtime/text/runtime_text_word_smart_wrap_preview_20260630.png (2026-06-30 LB-M2 WordSmart visual proof: inspected; SHA256 494880855721F5E0F6B48FA4DB8B8F34EEE1AB0DF0386C7F5B35646E9AB23AFF; repo target and E:\cargo-targets same-name match count 0)
  - rustfmt --edition 2021 --check zircon_runtime/src/graphics/text/layout/line_break/smart.rs zircon_runtime/src/graphics/text/layout/line_break/mod.rs zircon_runtime/src/graphics/text/layout/mod.rs zircon_runtime/src/ui/text/layout_engine/wrapping.rs zircon_runtime/src/ui/text/layout_engine/tests.rs zircon_runtime/src/ui/text/layout_engine/tests/word_smart.rs (2026-07-01 LB-M2 WordSmart ASCII trailing punctuation glue slice: passed)
  - cargo check -p zircon_runtime --lib --no-default-features --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-text-0701-jlreq-inseparable-kinsoku --message-format short --color never --quiet (2026-07-01 LB-M2 WordSmart ASCII trailing punctuation glue slice: passed with existing warnings only)
  - cargo test -p zircon_runtime word_smart --lib --no-default-features --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-text-0701-jlreq-inseparable-kinsoku --message-format short --color never -- --nocapture --test-threads=1 (2026-07-01 LB-M2 WordSmart ASCII trailing punctuation focused tests: first run exposed a 5/7 red case because shaping returned `go,next` as one chunk; after the shared smart owner split punctuation inside a chunk, rerun passed 7/7)
  - docs/tests/runtime/text/runtime_text_word_smart_punctuation_preview_20260701.png (2026-07-01 LB-M2 WordSmart ASCII trailing punctuation visual proof: inspected; SHA256 781220B1414FA2B62E4433A2540AC5DFB1E6810694C52C1CEFD165985E2822C6; repo target and E:\cargo-targets same-name match count 0)
  - rustfmt --edition 2021 --check zircon_runtime/src/graphics/text/layout/line_break/smart.rs zircon_runtime/src/graphics/text/layout/line_break/mod.rs zircon_runtime/src/ui/text/layout_engine/tests/word_smart.rs (2026-07-01 LB-M2 WordSmart ASCII quote-after-punctuation glue slice: passed)
  - cargo check -p zircon_runtime --lib --no-default-features --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-text-0701-jlreq-inseparable-kinsoku --message-format short --color never --quiet (2026-07-01 LB-M2 WordSmart ASCII quote-after-punctuation glue slice: passed with existing warnings only)
  - cargo test -p zircon_runtime word_smart --lib --no-default-features --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-text-0701-jlreq-inseparable-kinsoku --message-format short --color never -- --nocapture --test-threads=1 (2026-07-01 LB-M2 WordSmart focused tests after quote-after-punctuation slice: warm target rerun passed 13/13 with existing warnings only; cold target compile timeout produced no binary and is not counted)
  - docs/tests/runtime/text/runtime_text_word_smart_quote_punctuation_preview_20260701.png (2026-07-01 LB-M2 WordSmart ASCII quote-after-punctuation visual proof: inspected; SHA256 D2BFF7961FEE5E129A9C273E880B00E309F03410AA17FBCBCE1AB18268226BAD; repo target and E:\cargo-targets same-name match count 0)
- rustfmt --edition 2021 --check zircon_runtime/src/graphics/text/layout/line_break/smart.rs zircon_runtime/src/graphics/text/layout/line_break/mod.rs zircon_runtime/src/ui/text/layout_engine/tests/word_smart.rs (2026-07-01 LB-M2 WordSmart Unicode closing quote-after-punctuation glue slice: passed)
- cargo check -p zircon_runtime --lib --no-default-features --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-text-0701-jlreq-inseparable-kinsoku --message-format short --color never --quiet (2026-07-01 LB-M2 WordSmart Unicode closing quote-after-punctuation glue slice: passed with existing warnings only)
- cargo test -p zircon_runtime word_smart --lib --no-default-features --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-text-0701-jlreq-inseparable-kinsoku --message-format short --color never -- --nocapture --test-threads=1 (2026-07-01 LB-M2 WordSmart Unicode closing quote focused tests: red run failed 5 expected cases before U+2019/U+201D were protected; rerun passed 18/18 with existing warnings only)
- docs/tests/runtime/text/runtime_text_word_smart_unicode_quote_preview_20260701.png (2026-07-01 LB-M2 WordSmart Unicode closing quote visual proof: inspected; SHA256 B4DE63AFBA9AAD94001A43FA008479AF2351BDCD02AE23EAB3A9E1733F022B7F; repo target and E:\cargo-targets same-name match count 0)
- rustfmt --edition 2021 --check zircon_runtime/src/graphics/text/layout/line_break/smart.rs zircon_runtime/src/graphics/text/layout/line_break/mod.rs zircon_runtime/src/ui/text/layout_engine/tests/word_smart.rs (2026-07-01 LB-M2 WordSmart fullwidth/CJK trailing punctuation glue slice: passed)
- cargo test -p zircon_runtime fullwidth_trailing --lib --no-default-features --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-text-0701-word-smart-fullwidth --message-format short --color never -- --nocapture --test-threads=1 (2026-07-01 LB-M2 WordSmart fullwidth/CJK trailing punctuation focused tests: passed 4/4 with existing warnings only; full command completed in 14m26s and test execution in 0.66s)
- docs/tests/runtime/text/runtime_text_word_smart_fullwidth_punctuation_preview_20260701.png (2026-07-01 LB-M2 WordSmart fullwidth/CJK trailing punctuation visual proof: inspected; 1120x620; SHA256 F0B4F4AA2F80D3722F1F39B81F89A46DC4F15EBA8EFC03D157DAF834B4A7ADC0; repo target and E:\cargo-targets same-name match count 0)
- rustfmt --edition 2021 --check zircon_runtime/src/graphics/text/layout/line_break/smart.rs zircon_runtime/src/graphics/text/layout/line_break/mod.rs zircon_runtime/src/ui/text/layout_engine/tests/word_smart.rs (2026-07-01 LB-M2 WordSmart CJK/fullwidth closing delimiter glue slice: passed)
- touched Rust trailing-whitespace scan (2026-07-01 LB-M2 WordSmart CJK/fullwidth closing delimiter glue slice: no output)
- cargo test -p zircon_runtime cjk_closing_delimiter --lib --no-default-features --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-text-0701-word-smart-fullwidth --message-format short --color never -- --nocapture --test-threads=1 (2026-07-01 LB-M2 WordSmart CJK/fullwidth closing delimiter focused tests: passed 5/5 with existing warnings only; full command completed in about 7m45s and test execution in 0.78s)
- docs/tests/runtime/text/runtime_text_word_smart_cjk_closing_delimiter_preview_20260701.png (2026-07-01 LB-M2 WordSmart CJK/fullwidth closing delimiter visual proof: inspected; 1120x620; SHA256 73AAC7DF5328374E5F1CE9B0FECDC418E984407F4D5805381663C23C14C2E92E; repo target and E:\cargo-targets same-name match count 0)
- cargo test -p zircon_runtime punctuation_cluster --lib --no-default-features --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-text-0701-word-smart-fullwidth --message-format short --color never -- --nocapture --test-threads=1 (2026-07-01 LB-M2 WordSmart punctuation-cluster RED run: failed 3 expected owner cases before shared smart owner looped re-splits)
- rustfmt --edition 2021 --check zircon_runtime/src/graphics/text/layout/line_break/smart.rs zircon_runtime/src/graphics/text/layout/line_break/mod.rs zircon_runtime/src/ui/text/layout_engine/tests/word_smart.rs (2026-07-01 LB-M2 WordSmart punctuation-cluster glue slice: passed)
- touched Rust debt scan TODO/FIXME/unwrap/expect/panic/allow(dead_code)/Result<.*String (2026-07-01 LB-M2 WordSmart punctuation-cluster glue slice: no output)
- cargo test -p zircon_runtime punctuation_cluster --lib --no-default-features --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-text-0701-word-smart-fullwidth --message-format short --color never -- --nocapture --test-threads=1 (2026-07-01 LB-M2 WordSmart punctuation-cluster focused tests: passed 5/5 with existing warnings only)
- cargo test -p zircon_runtime word_smart --lib --no-default-features --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-text-0701-word-smart-fullwidth --message-format short --color never -- --nocapture --test-threads=1 (2026-07-01 LB-M2 WordSmart regression suite after punctuation-cluster slice: passed 34/34 with existing warnings only)
- docs/tests/runtime/text/runtime_text_word_smart_punctuation_cluster_preview_20260701.png (2026-07-01 LB-M2 WordSmart punctuation-cluster visual proof: inspected; 1120x620; SHA256 1FFA95EB87CD8A0A3156E67F85C86F126F37BCAE061481740BAFF7719BC48001; repo target and E:\cargo-targets same-name match count 0)
- rustfmt --edition 2021 --check zircon_runtime/src/graphics/tests/render_product_mesh_cache/project_plugin_registry_material_passes_staged_cache/fixture.rs zircon_runtime/src/graphics/text/layout/line_break/smart.rs zircon_runtime/src/graphics/text/layout/line_break/mod.rs zircon_runtime/src/ui/text/layout_engine/tests/word_smart.rs (2026-07-01 LB-M2 WordSmart ellipsis/leader trailing punctuation slice: passed)
- cargo test -p zircon_runtime ellipsis_trailing --lib --no-default-features --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-text-0701-word-smart-fullwidth --message-format short --color never -- --nocapture --test-threads=1 (2026-07-01 LB-M2 WordSmart ellipsis/leader focused tests: passed 4/4 with existing warnings only after repairing a validation-blocking material fixture field access)
- docs/tests/runtime/text/runtime_text_word_smart_ellipsis_punctuation_preview_20260701.png (2026-07-01 LB-M2 WordSmart ellipsis/leader visual proof: inspected; 1120x620; SHA256 C7639D78EA81567970BE95BD52F2F6F7D1DBBC37EB149E81EF838CC33C617693; repo target and E:\cargo-targets same-name match count 0)
- rustfmt --edition 2021 --check zircon_runtime/src/graphics/text/layout/line_break/smart.rs zircon_runtime/src/graphics/text/layout/line_break/mod.rs zircon_runtime/src/ui/text/layout_engine/tests/word_smart.rs (2026-07-01 LB-M2 WordSmart Arabic/RTL trailing punctuation slice: passed)
- cargo test -p zircon_runtime arabic_trailing --lib --no-default-features --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-text-0701-word-smart-arabic --message-format short --color never -- --nocapture --test-threads=1 (2026-07-01 LB-M2 WordSmart Arabic/RTL focused tests: first attempt timed out during Windows lib-test compile with no result; implementation rerun passed 3/3 with existing warnings only; direct lib-test binary arabic_ascii passed 1/1 after Cargo filter timeout)
- docs/tests/runtime/text/runtime_text_word_smart_arabic_punctuation_preview_20260701.png (2026-07-01 LB-M2 WordSmart Arabic/RTL visual proof: inspected; 1120x620; SHA256 540149B2C66D0110F5A705D99B3631803FD300341EDAF5233B30FCE3A06766E4; repo target and E:\cargo-targets same-name match count 0)
- rustfmt --edition 2021 --check zircon_runtime/src/graphics/text/layout/line_break/smart.rs zircon_runtime/src/graphics/text/layout/line_break/mod.rs zircon_runtime/src/ui/text/layout_engine/tests/word_smart.rs (2026-07-01 LB-M2 WordSmart Unicode double/interrobang trailing punctuation slice: passed)
- cargo test -p zircon_runtime unicode_double_punctuation --lib --no-default-features --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-text-0701-word-smart-arabic --message-format short --color never -- --nocapture --test-threads=1 (2026-07-01 LB-M2 WordSmart Unicode double/interrobang focused tests: warm-target compile-timeout attempt produced no RED evidence; cold target timed out without binary; final warm-target rerun passed 3/3 with existing warnings only)
- docs/tests/runtime/text/runtime_text_word_smart_unicode_double_punctuation_preview_20260701.png (2026-07-01 LB-M2 WordSmart Unicode double/interrobang visual proof: inspected; 1120x620; SHA256 B42D4B564BD3277DCFEB423560D32734BEE4C5E937ACE013BCBB2D7A2355E722; repo target and E:\cargo-targets same-name match count 0)
  - rustfmt --edition 2021 --check zircon_runtime/src/ui/text/layout_engine.rs zircon_runtime/src/ui/text/layout_engine/direction.rs zircon_runtime/src/ui/text/layout_engine/line_box.rs zircon_runtime/src/ui/text/layout_engine/wrapping.rs zircon_runtime/src/ui/text/layout_engine/ellipsis.rs zircon_runtime/src/ui/text/layout_engine/candidate_line.rs zircon_runtime/src/ui/text/layout_engine/range_mapping.rs zircon_runtime/src/ui/text/layout_engine/visual_order.rs zircon_runtime/src/ui/text/layout_engine/overflow_style.rs (2026-06-30 UI layout line-box + direction owner split: passed)
  - cargo check -p zircon_runtime --lib --no-default-features --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-text-0630-line-box-direction-owner --message-format short --color never --quiet (2026-06-30 UI layout line-box + direction owner split: passed with existing warnings only)
  - docs/tests/runtime/text/runtime_text_line_box_direction_owner_preview_20260630.png (2026-06-30 UI layout line-box + direction owner visual proof: inspected; SHA256 438A776894C0C1BF46E3CC16B90BC3F7168D27F126F4B8B9C5BD0F504BD0DC8D; repo target and E:\cargo-targets same-name match count 0)
  - rustfmt --edition 2021 --check zircon_runtime/src/graphics/text/layout/kinsoku.rs zircon_runtime/src/ui/text/layout_engine/tests/kinsoku.rs (2026-06-30 LB-M2 Japanese non-starter kinsoku table slice: passed)
  - cargo check -p zircon_runtime --lib --no-default-features --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-text-0630-japanese-nonstarter-kinsoku --message-format short --color never --quiet (2026-06-30 LB-M2 Japanese non-starter kinsoku table slice: passed with existing warnings only)
  - docs/tests/runtime/text/runtime_text_japanese_nonstarter_kinsoku_preview_20260630.png (2026-06-30 LB-M2 Japanese non-starter kinsoku visual proof: inspected; SHA256 C4D561C28049B2E9BE86CE6C94778BD6F83A39FA0A5BA1F6011C21DA05E3BE2D; repo target and E:\cargo-targets same-name match count 0)
  - zircon_runtime/src/ui/text/layout_engine/tests.rs
  - zircon_runtime/src/ui/tests/text_hit_testing.rs
  - zircon_runtime/src/ui/tests/widget_text_input_pointer.rs
  - zircon_runtime/src/ui/tests/text_layout
  - cargo test -p zircon_runtime --lib style_key_encodes_clamp_overflow_float_bits --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-text-0630-interface-key --message-format short --color never -- --nocapture (2026-06-30: blocked by existing runtime test harness compile/timeout issues; test remains in source)
  - zircon_runtime_interface/src/tests/render_contracts.rs
  - cargo test -p zircon_runtime --lib ui::text::layout_engine --locked --jobs 1 --target-dir "D:\cargo-targets\zircon-ui-text-grapheme-layout" --message-format short --color never -- --nocapture
  - cargo test -p zircon_runtime --lib ui::tests::text_layout --locked --jobs 1 --target-dir "D:\cargo-targets\zircon-ui-text-grapheme-layout" --message-format short --color never -- --nocapture
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir "D:\cargo-targets\zircon-ui-text-grapheme-layout" --message-format short --color never
  - cargo check -p zircon_runtime_interface --tests --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m6 --color never
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m6 --color never
  - cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m6 --color never
  - cargo test -p zircon_runtime_interface --lib render_contracts --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m6 --message-format short --color never -- --nocapture
  - cargo test -p zircon_runtime --lib screen_space_ui_plan --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m6 --message-format short --color never -- --nocapture
  - cargo test -p zircon_runtime --lib text_attrs --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m6 --message-format short --color never -- --nocapture
  - cargo test -p zircon_editor --lib native_runtime_text_painter --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m6 --message-format short --color never -- --nocapture
doc_type: module-detail
---

# Runtime UI Text Layout Engine

`layout_engine.rs` is the shared runtime entry owner for `UiResolvedTextLayout`. It turns template text plus `UiResolvedStyle` typography fields into neutral line records before graphics, editor, or debug consumers see the render command, while child owners under `ui/text/layout_engine/` own the mutable line state, wrapping, direction, line-box, overflow projection, style scaling, range mapping, and visual-order details.

As of the 2026-07-01 LB-M4 slice, `UiTextWritingMode::VerticalRl` is a writing-mode contract rather than a text-direction alias. Horizontal text remains the default. Vertical resolved layout is delegated to `layout_engine/vertical.rs`; each `UiResolvedTextLine` record is a semantic column, y is the main advance axis, and columns are placed from right to left. The shaped-text DTO carries the same writing mode so vertical glyph frames advance on y and ASCII glyphs can be marked `Cw90` while broader font-orientation parity remains a later slice.

The 2026-07-13 LB-M5 paragraph slice keeps BBCode paragraph policy writing-mode neutral. `paragraph_layout.rs` is the single owner of override merging, indent-level clamping, list-prefix measurement, and first/continuation constraints. Horizontal layout maps that logical scalar to x/width; VerticalRl maps it to y/height, while `vertical.rs` continues to own shaped columns and right-to-left x placement. Paragraph Left/Start, Center, and Right/End therefore map to physical top, center, and bottom without parser, interface, or renderer branches. Vertical rich-inline-object plus paragraph composition remains an explicit follow-up in `rich_inline_vertical.rs`, because its object metrics cannot be replaced by the generic text wrapper.

The follow-up vertical geometry slice keeps those same resolved column records as the source of editing geometry. `hit_test.rs` selects a `VerticalRl` column by x before using y midpoint and resolved `glyph_advances` to return the source byte offset. `geometry.rs` projects caret and selection ranges along y and emits horizontal 1px bars. The TextInput IME context consumes that caret frame for `SetCursorArea`, so candidate anchoring no longer inherits the old horizontal caret shape when a resolved vertical layout is available.

The render-contract edit decoration slice carries the same rule into `zircon_runtime_interface/src/ui/surface/render/text_geometry.rs`. `UiTextPaintDecoration` now branches on `UiResolvedTextLayout.writing_mode`: vertical selections cover the column width over the selected y span, composition underline becomes a right-edge side rule, and caret decorations become horizontal bars. That keeps neutral paint DTOs consistent before the platform painter or editor retained-host path consumes them.

The SDF projection slice carries `UiTextWritingMode` into `ScreenSpaceUiTextBatch` and lets `sdf_render.rs` choose between horizontal baseline/cursor-x placement and vertical column/cursor-y placement. In `VerticalRl`, SDF glyph quads are centered inside the text column and advance along y, so the render backend no longer turns a vertical resolved layout back into a horizontal SDF draw plan. This is still projection-only: shaping-time vertical substitutions and sideways Latin orientation remain outside the SDF renderer.

## Owner Split

The active owner split is:

- `layout_engine.rs` owns public crate-private entrypoints, top-level `layout_text(...)` orchestration, clip/horizontal overflow application, visual-order handoff, and resolved-line assembly.
- `direction.rs` owns explicit/Auto/Mixed paragraph base direction resolution and the strong LTR/RTL helpers used by visual ordering.
- `line_box.rs` owns measured/tab-aligned grapheme advances, Justify eligibility, line width clamping, logical Start/End x alignment, and the minimum fallback text advance.
- `paragraph_layout.rs` owns merged rich-block overrides, bounded indent/list-prefix extents, and the writing-mode-neutral first/continuation constraint policy projected to line width or column height.
- `wrapping.rs` owns source-run wrapping orchestration, newline preservation, Word chunk consumption, WordSmart chunk selection through the shared smart owner, Glyph fallback appending, and leading grapheme continuation while consuming line-fit checks from `graphics/text/layout/line_break/greedy.rs`.
- `candidate_line.rs` owns mutable candidate line text, source/visual ranges, resolved runs, pending break suffixes, and trailing wrap-space mutation.
- `ellipsis.rs` owns clipped-line merge and projection of shared overflow segments back into UI resolved runs.
- `range_mapping.rs` owns internal source/visual subrange mapping shared by ellipsis and visual ordering.
- `vertical.rs` owns the current `VerticalRl` column layout path, including height-main-axis wrapping, right-to-left column placement, height-based overflow extent, and CJK kinsoku chunk reuse.
- `visual_order.rs` owns the current low-fidelity visual-order scaffold and temporary RTL single-codepoint mirror table.
- `overflow_style.rs` owns shrink-to-fit and clamp-font-size effective style resolution before wrapping.

## Layout Flow

The root helper intentionally stays data-oriented:

- resolve paragraph direction through `direction.rs`
- parse plain/rich source runs through `rich_text.rs`
- dispatch `VerticalRl` requests to `vertical.rs`; horizontal text continues through the line-oriented owner flow below
- split newlines while preserving original byte ranges
- apply word, word-smart, glyph, or no-wrap policy through `wrapping.rs`
- ask `graphics/text/layout/line_break/greedy.rs` whether the current line plus next chunk still fits before mutating a candidate line
- consume `graphics/text/layout/line_break/smart.rs` chunk metadata for current WordSmart ASCII/fullwidth/CJK trailing punctuation, Unicode ellipsis/leader trailing punctuation, Unicode double/interrobang trailing punctuation, Arabic/RTL common trailing punctuation, ASCII/Unicode right quote-after-punctuation glue, CJK/fullwidth closing delimiters after punctuation, and consecutive trailing punctuation clusters such as `go,`, `go…`, `go‥`, `go‼`, `go⁇`, `go⁈`, `go⁉`, `go⁉!`, `go؟`, `go؟!`, `go,"`, `go,”`, `go，`, `go，」`, `go?!`, `go！？`, and `go，」！`
- consume `graphics/text/layout/kinsoku.rs` chunk metadata so protected non-starters, JLREQ pairs, and punctuation such as `ー`, `々`, `ヵ`, `ㇰ`, `〗`, `〞`, `｠`, `〖`, `｟`, and `……` stay on the correct side of a wrapped UI line
- trim word-wrap separator spaces at line boundaries
- apply height/horizontal overflow and ellipsis projection through `ellipsis.rs`
- convert mixed-direction lines into a low-fidelity visual-order string
- emit line frames, baselines, measured widths, source ranges, visual ranges, and resolved runs after `line_box.rs` resolves advances and alignment

The 2026-06-30 LB-M2 WordSmart contract entry put the `WordSmart` wrap mode inside the existing wrapping owner. The 2026-07-01 follow-ups now route `UiTextWrap::WordSmart` through shared `word_smart_line_break_chunks(...)`; `graphics/text/layout/line_break/smart.rs` owns the first ASCII trailing punctuation rule and can split a shaped single chunk like `go,a` into protected `go,` plus `a`. It also extends protected punctuation runs across ASCII closing quotes, Unicode right closing quotes U+2019/U+201D, the plan-approved fullwidth/CJK trailing punctuation set, U+2026/U+2025 ellipsis/leader punctuation, Unicode double/interrobang punctuation U+203C/U+2047/U+2048/U+2049, Arabic/RTL common trailing punctuation `،`/`؛`/`؟`, the first CJK/fullwidth closing delimiter chain after punctuation, and consecutive punctuation clusters, so `go,"a`, `go,”a`, `go，a`, `go…a`, `go‥a`, `go⁉a`, `go⁉!a`, `go؟a`, `go؟!a`, `go，」a`, `go?!next`, `go！？next`, and `go，」！next` become protected first chunks plus the following text. A later LB-M2 slice still needs full Unicode/Godot-style smart word classes, paired/full quote direction inference, broader punctuation classes beyond the listed set, CJK/multi-script policy, and native/SDF paragraph parity.

The 2026-06-30 LB-M2 greedy wrap decision owner split moves `line_text_fits(...)` and the current-line plus next-chunk append predicate into `graphics/text/layout/line_break/greedy.rs`. UI Word and Glyph wrapping now ask that shared owner before wrapping, instead of rebuilding candidate strings and width-fit policy inside `ui/text/layout_engine/wrapping.rs`.

The old fixed half-em advance scaffold has been replaced by shared measured advances where available; `line_box.rs` still owns a minimum fallback advance for empty or undersized frames. Glyphon/cosmic-text, SDF font bake, and later HarfBuzz/ICU integration remain responsible for final glyph metrics, font fallback, atlas/cache state, script shaping, cluster positioning, and GPU submission.

## Resolved Layout Cache Key

`resolved_layout.rs` owns the style cache key used by the text layout cache. Public typography can expose expressive values such as `UiTextOverflow::ClampFontSize { min_px, max_px }`, but the cache key must remain `Eq` so resolved layouts can be compared and reused deterministically. The 2026-06-30 guard introduces a private `UiTextOverflowKey` that maps overflow variants into cache-key-safe values and stores clamp bounds through their `f32::to_bits()` representation. This keeps the public interface ergonomic while preventing f32 equality from leaking into the cache key contract.

The 2026-07-13 SM-M5 identity cut removes `text_render_mode` from `UiTextStyleKey`. Native and SDF both consume `UiSharedTextShaper`; raster selection cannot change resolved frames, source ranges, advances, or line breaks and therefore must not split persistent or same-frame layout caches. The render mode remains on `UiResolvedStyle` for `UiTextShaperSelection` diagnostics and screen-space Native/SDF batch routing. Exact tests prove one key/one cache entry across both modes, compare horizontal plus VerticalRl batch frames/source ranges/advances, and cover Latin/CJK/mixed/RTL at the 23.5px bitmap and 24px SDF policy boundary. The current horizontal mixed-paragraph renderer acceptance gate also renders Native and SDF side by side into a real 1080×1690 WGPU framebuffer, requires pixels in both regions, and checks their bounding-box tolerance; the accepted PNG lives under `docs/tests/runtime/text`, never a Cargo target.

## Grapheme Boundary Scaffold

`grapheme.rs` centralizes Unicode segmentation for the layout foundation. Glyph wrapping, fixed-advance measurement, ellipsis truncation, and the low-fidelity RTL reversal consume grapheme clusters so combining marks such as `a\u{0301}` stay with their base character while the current text scaffold is still pre-shaping. When a rich marker splits a visible cluster across adjacent runs, the helper treats a leading continuation mark as part of the preceding visible cluster so wrap/truncation/reversal do not isolate the mark.

This follows the responsibility split seen in Unreal Slate, where too-long words fall back to grapheme-cluster wrapping and editable movement uses a character-boundary iterator. It also matches Godot's shaped-text navigation surface where next/previous grapheme positions are the unit for caret movement once shaped glyph data exists. Zircon still performs this before real shaping, so the helper only protects byte-range and visible-string scaffolds; it is not a replacement for HarfBuzz/ICU shaping or font fallback.

## BiDi Visual-Order Scaffold

The current visual-order helper follows the same responsibility boundary as Unreal Slate's `FSlateTextShaper::ShapeBidirectionalText`: split a paragraph into direction runs before shaping, then keep source-to-visual relationships available for later glyph enumeration. It also mirrors Slint's text layout boundary where positioned glyphs keep the original text byte offset so selection and caret geometry can be derived from rendered glyph order.

This helper is deliberately low fidelity. It tokenizes resolved runs into strong LTR, strong RTL, and neutral spans. Neutral tokens inherit a surrounding same-direction span, so punctuation inside an RTL phrase such as `שלום-עולם` travels with that RTL visual span. Separators at an LTR/RTL boundary stay on the LTR side so existing mixed-line spacing remains stable. RTL spans are reversed at token order and character order, while each visual run keeps the original source byte range and a current visual byte range.

This does not implement full Unicode BiDi, glyph mirroring, script-specific shaping, glyph positioning, or font fallback. Grapheme boundaries now prevent the scaffold from splitting clusters during wrap/truncation/reversal, but true grapheme cluster shaping remains deferred to the real text shaping backends. The shared layout engine only provides a stable intermediate contract for tests, render extract, and future shaped-text DTO derivation.

## Range Rules

`source_range` always points into the authored text bytes, even when a visual run has moved. `visual_range` points into the emitted visual line string. Ellipsis runs use a zero-length source range at the truncated line end because the ellipsis character is generated by overflow policy rather than authored text.

These byte ranges are the foundation for later caret, selection, composition underline, and shaped-glyph diagnostics. Tests assert both text order and ranges so later shaper upgrades can replace the low-fidelity algorithm without weakening the cross-layer contract.

## M6 Shared Shape And Editing Paint

The M6 continuation makes the shared render DTO consume this layout output more directly. `UiShapedText::from_resolved_layout(...)` now derives per-grapheme synthetic glyph records from each resolved line. Those records are not final backend glyph ids, but they give Widget Reflector, editor painter, and runtime debug payloads stable glyph count, visual frame, advance, and source range data for combining marks and emoji clusters.

`hit_test.rs` now consumes the same `UiResolvedTextLayout` and maps a surface-space pointer point back to a nearest source byte caret. The helper uses the resolved line frames, alignment, direction, fixed text advance, and grapheme runs that render extraction already emitted. `surface/input/text_pointer.rs` consumes that geometry for TextInput pointer press and captured drag, mirroring Bevy's editable text flow where `dev/bevy/crates/bevy_ui_widgets/src/editable_text.rs` converts a local pointer position into `TextEdit::MoveToPoint`, `TextEdit::ShiftClickExtension`, or `TextEdit::ExtendSelectionToPoint`, and `dev/bevy/crates/bevy_text/src/text_edit.rs` applies those edits through the text driver.

Editable selection, caret, and composition underline geometry now also snaps to grapheme cluster edges in `UiRenderCommand::text_paint(...)`. A selection whose byte range falls inside `a\u{0301}` expands to the whole visible cluster, so editor/runtime painters do not split an accent or emoji component. Runtime screen-space UI planning then consumes the same shared `UiTextPaintDecoration` frames: selection is emitted as a pre-text quad, while caret and composition underline are emitted as post-text quads after the glyphon/SDF text pass.

Rich style paint now follows the same route. `UiTextPaint.runs` is derived from shaped clusters and carries `UiTextPaintRun` records with Strong/Emphasis/Code style flags. Runtime planning prefers these shared runs over raw line text, so a resolved line containing plain, strong, and code fragments becomes separate text batches with stable run frames. The glyphon backend converts the flags to bold, italic, and monospace attrs, while the editor native painter uses the same DTO to apply software fallback bold/italic drawing. This closes the immediate renderer-local rich-run parsing gap without claiming final HarfBuzz-level metrics.

## Tests

`ui::text::layout_engine` module tests cover grapheme-safe fixed-advance measurement, glyph wrapping, ellipsis truncation, low-fidelity RTL reversal, rich-run boundary clusters, and shared kinsoku metadata consumption. The `ui::text::layout_engine::tests::kinsoku` child owner covers halfwidth, small ka/ke, Katakana phonetic extension small-kana, spacing voicing marks, JLREQ hyphens, JLREQ cl-08 inseparable pairs, Japanese non-starter, CJK white bracket/quote, CJK double-prime closing quote, and fullwidth white parenthesis cases including `text_wrap_cjk_kinsoku_no_leading_small_katakana_ka`, `text_wrap_cjk_kinsoku_no_leading_katakana_phonetic_extension_small_kana`, `text_wrap_cjk_kinsoku_no_leading_spacing_voicing_mark`, `text_wrap_cjk_kinsoku_no_leading_jlreq_hyphen`, `text_wrap_keeps_jlreq_inseparable_ellipsis_pair_together`, `text_wrap_cjk_kinsoku_no_leading_cjk_white_close_punctuation`, `text_wrap_cjk_kinsoku_no_leading_cjk_double_prime_closing_quote`, `text_wrap_cjk_kinsoku_no_leading_fullwidth_white_close_parenthesis`, `text_wrap_cjk_kinsoku_no_trailing_cjk_white_open_punctuation`, `text_wrap_cjk_kinsoku_no_trailing_fullwidth_white_open_parenthesis`, `text_wrap_cjk_kinsoku_no_leading_prolonged_sound_mark`, and `text_wrap_cjk_kinsoku_no_leading_iteration_mark`.

`ui::text::layout_engine::tests::word_smart` covers the first shared WordSmart punctuation rules with `word_smart_keeps_ascii_trailing_punctuation_with_previous_word`, `word_smart_keeps_ascii_closing_quote_after_trailing_punctuation_with_previous_word`, `word_smart_keeps_unicode_closing_quote_after_trailing_punctuation_with_previous_word`, `word_smart_keeps_fullwidth_trailing_punctuation_with_previous_word`, `word_smart_keeps_cjk_closing_delimiter_after_fullwidth_punctuation_with_previous_word`, `word_smart_keeps_trailing_punctuation_cluster_without_absorbing_next_word`, `word_smart_keeps_ellipsis_trailing_punctuation_with_previous_word`, `word_smart_keeps_unicode_interrobang_punctuation_with_previous_word`, `word_smart_keeps_unicode_double_punctuation_with_previous_word`, and `word_smart_keeps_arabic_trailing_punctuation_with_previous_word`, proving UI wrapping consumes shared smart chunks and keeps `go,` / `go,"` / `go,”` / `go，` / `go，」` / `go?!` / `go…` / `go‽` / `go⁉` / `go؟` together before wrapping the following text.

`ui::tests::text_layout` covers word wrapping, clip-frame line removal, rich ellipsis preservation, mixed LTR/RTL visual-order ranges, neutral separator assignment inside RTL spans, and editable text action interactions.

`ui::tests::text_hit_testing` covers layout-backed pointer-to-caret mapping for grapheme midpoint selection, multiline y routing and x clamping, aligned line frames, and the current `VerticalRl` column-x/y-advance mapping. `ui::tests::widget_text_input_pointer` covers that the same layout output drives TextInput press, Shift+press, drag selection, and empty-value editable layout fallback. `ui::tests::widget_text_input_ime_context` now also covers that a vertical TextInput reports the horizontal caret bar as its IME cursor rect. These tests are intentionally focused on the current resolved advance geometry and do not claim final backend cluster reverse lookup.

`render_contracts` covers the shared shaped artifact and decoration cluster snapping through `ui_shaped_text_contract_derives_grapheme_glyph_bounds` and `ui_text_decorations_snap_to_grapheme_cluster_edges`.

`screen_space_ui_plan` covers runtime renderer consumption of shared decoration geometry through `screen_space_ui_plan_uses_shared_text_decorations_as_pre_and_post_text_draws` and rich paint run splitting through `screen_space_ui_plan_splits_rich_text_runs_from_shared_paint`.

`text_attrs_maps_shared_rich_run_style_to_glyphon_attrs` covers the native glyphon mapping from shared rich run style flags to backend attrs. `native_runtime_text_painter` remains the editor native painter smoke gate for shared runtime text payload consumption.
