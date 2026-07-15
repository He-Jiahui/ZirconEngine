---
related_code:
  - zircon_runtime/src/text/shaping/fallback_spans.rs
  - zircon_runtime/src/text/shaping/cosmic.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text/resolved_batches.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/render/text_advances.rs
  - zircon_runtime/tests/runtime_text_multilingual_product_framebuffer.rs
  - zircon_runtime/tests/runtime_text_multilingual_product_framebuffer/product_project_fixture.rs
plan_sources:
  - docs/plans/zircon_runtime/text/01-font-resource-faces-and-database.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - managed zircon_runtime lib-test binary text_font
  - managed zircon_runtime lib-test binary text_horizontal_
  - managed zircon_runtime lib-test binary graphics::text::sdf::tests::fdsm_gen::text_msdf_dynamic_generation_applies_real_variable_width_axis --exact
  - managed zircon_runtime lib-test binary graphics::scene::scene_renderer::ui::sdf_atlas::tests::plan::sdf_atlas_plan_separates_variable_font_instances_on_same_face --exact
  - managed zircon_runtime lib-test binary graphics::text::raster::swash::tests::text_raster_swash_request_preserves_arbitrary_variable_axes --exact
  - cargo test -p zircon_runtime --test runtime_text_multilingual_product_framebuffer --no-default-features --features target-client --locked --offline --jobs 1 --target-dir <managed> --no-run
  - cargo test -p zircon_runtime --test runtime_text_multilingual_product_framebuffer --no-default-features --features target-client --locked --offline --jobs 1 --target-dir <managed> export_runtime_multilingual_text_product_framebuffer_png -- --ignored --exact --nocapture --test-threads=1
doc_type: milestone-acceptance
status: accepted
---

# Text01 FR-M2 variable-font product acceptance

Plan: docs/plans/zircon_runtime/text/01-font-resource-faces-and-database.md

Milestone: M2

Status: accepted

Files: ["zircon_runtime/src/graphics/text/font/database.rs", "zircon_runtime/src/graphics/text/shaping/fallback_spans.rs", "zircon_runtime/src/graphics/text/shaping/cosmic.rs", "zircon_runtime/src/graphics/scene/scene_renderer/ui/render.rs", "zircon_runtime/src/graphics/scene/scene_renderer/ui/render/text_advances.rs", "zircon_runtime/src/graphics/scene/scene_renderer/ui/text.rs", "zircon_runtime/src/graphics/scene/scene_renderer/ui/text/font_assets.rs", "zircon_runtime/src/graphics/scene/scene_renderer/ui/text/prepare_report.rs", "zircon_runtime/src/graphics/scene/scene_renderer/ui/text/resolved_batches.rs", "zircon_runtime/src/graphics/scene/scene_renderer/ui/text/tests.rs", "zircon_runtime/tests/runtime_text_multilingual_product_framebuffer.rs", "zircon_runtime/tests/runtime_text_multilingual_product_framebuffer/product_project_fixture.rs", "zircon_runtime/tests/runtime_text_multilingual_product_framebuffer/proof_assertions.rs", "zircon_runtime/tests/runtime_text_multilingual_product_framebuffer/proof_commands.rs", "docs/zircon_runtime/graphics/text/font-variation-instances.md", "docs/plans/zircon_runtime/text/01/2026-07-10-font-resource-faces-and-database-output-records.md", "docs/plans/zircon_runtime/text/01/fixed-2026-07-14-dynamic-scene-format-version-root-export-drift.md", "docs/tests/runtime/text/runtime_text_variable_font_instances_product_framebuffer_20260714.png"]

## Scope delivered

- Formal project asset loading now precedes screen-space atlas preparation for auto/native/SDF batches, and the established shaping owner refreshes glyph data after new faces are registered.
- Fallback spans preserve the authoritative logical `FontFaceId` and `InstancedFaceId` across cosmic/glyphon projection, so logical instances sharing one physical family remain distinct through RustyBuzz, SDF generation, atlas keys, and framebuffer readback.
- The product fixture imports one real Windows Bahnschrift face, derives its actual `wdth` minimum/maximum, registers both logical members through one `.font.toml`, and renders the same string through the real SDF path.
- Folder-backed owners and canonical sibling imports satisfy the code-structure and June review requirements; no compatibility re-export, renderer family special case, test-pixel injection, second font database, or `target` screenshot is introduced.

## Fresh testing evidence

- Initial compile/GPU jobs `a33160e6543e4419a334c6422b8f7f37` and `37c142f4696747e0b63df8161787f7d2` established the physical product proof before review. Post-review compile job `d4fd827abfc3450090d20275d91b57ee` then exited 0 after all four review fixes.
- Final exact managed GPU job `d80d6dabac754907b50aa3ae2c1c1056`: exit 0, 1 passed / 0 failed, build 16m26s and test time 1409.03s.
- Managed focused job `61aaa263af684ab7b028956c772e0a20` reused the coordinator-retained default-feature lib-test binary: `text_font` 41 passed / 0 failed in 54.62s and `text_horizontal_` 6 passed / 0 failed in 24.53s.
- Managed exact focused job `deb789dcbdbe43c3b17fea6a234c9079`: real dynamic SDF width-axis 1/1, SDF atlas variable-instance separation 1/1, and arbitrary-axis Swash request 1/1.
- Final readback: narrow width 256px with 3187 changed pixels; wide width 346px with 3747 changed pixels; relative difference 4984 pixels.
- Artifact: `docs/tests/runtime/text/runtime_text_variable_font_instances_product_framebuffer_20260714.png`, 1080×1840, 353953 bytes, 2442 colors, SHA256 `754A7C1CC64D98B50D6FB798F702353C4BABB7EAAA5B722657529B4641BB9C40`.
- Final original-resolution visual inspection confirms multilingual/RTL/vertical/rich-text/table content renders and the bottom `wdth=min/max` samples visibly differ without tofu or overlap.
- Duplicate scan: repository `target`, `D:\cargo-targets`, `E:\cargo-targets`, and `F:\cargo-targets` all contain zero files with the artifact name.

## Review

- Earlier failures were retained as evidence: two runs disproved the partial-face-sync hypothesis; the prepare-time refresh changed the rendered geometry but still collapsed logical instances; the final fix addressed the lower shared identity loss instead of weakening the pixel assertion.
- Independent review first identified three Important defects and one complexity issue: failed font assets were negatively cached and conflated successful loads with face changes; raw VerticalRl batches could retain fallback-font advances after a project face loaded; the exporter could report a false-positive pass outside Windows; and glyph-to-span projection was linear in every span. The implementation now keeps failures retryable, derives invalidation from the authoritative database face-count delta, clears only internally derived advances, gates the whole exporter on Windows, and locates ordered spans with `partition_point` plus containment validation.
- The same reviewer rechecked all four remediations, ran scoped `rustfmt --check` and `git diff --check`, found no new Critical or Important issue, and returned `Accept`. Runtime acceptance remains grounded in the managed post-review Cargo/GPU evidence above rather than the static verdict alone.
- Product, FontDatabase/F2DOT14, horizontal shaping, dynamic SDF, atlas separation, and native Swash gates are all green, so FR-M2 is accepted. FR-M3 CompositeFont work and the remaining Text01 cross-platform fixture work stay open.
