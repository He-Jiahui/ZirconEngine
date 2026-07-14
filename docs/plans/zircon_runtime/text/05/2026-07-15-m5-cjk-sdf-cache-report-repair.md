# Text05 M5 CJK SDF cache report regression repair

Plan: docs/plans/zircon_runtime/text/05-sdf-msdf-pipeline.md
Milestone: M5
Status: accepted
Files: ["zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_font_bake.rs", "zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_font_bake/tests.rs", "zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_render/tests/mod.rs", "zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_render/tests/prepare_report.rs", "docs/zircon_runtime/graphics/scene/scene_renderer/ui/sdf_font_bake.md", "docs/plans/zircon_runtime/text/05/2026-07-09-sdf-msdf-pipeline-output-records.md"]

## Scope delivered

This closeout removes the last Text05-owned CJK SDF failure observed by the originating Editor02 scene gate. The lowest implementation owner remains the M2 SDF face/cache layer, while M5 owns the upward Native/distance-field paragraph and scene consistency acceptance that the regression blocked.

- `SdfAtlasBakeReport::resident_font_count` reports all materialized faces retained after a build.
- `loaded_font_count` reports only faces newly materialized by that build.
- The real Windows CJK regression carries the authoritative `FontFaceId` emitted by production shaping, so a preloaded Microsoft YaHei UI face cannot be confused with a distinct project CompositeFont face.
- CompositeFont ordering, runtime fallback selection, and renderer/Editor boundaries are unchanged; there is no test-only production branch or relaxed `2`/`>= 1` expectation.
- The canonical failure lifecycle was returned to `docs/plans/zircon_editor/editor/02/fixed-2026-07-15-sdf-font-bake-cjk-loaded-font-count-regression.md`.

## Fresh testing evidence

- Managed Windows package job `5fa88d54174d4467aa3b46f00105d972` produced the current-source `zircon_runtime` lib-test executable. The aggregate command was independently stopped by three compile errors in foreign `material_shader_redirect_dependency_contract` code.
- Current-source focused lower gate: `graphics::scene::scene_renderer::ui::sdf_font_bake::tests::` passed 13/13, including the real Windows CJK face and first-build/cached-build report semantics.
- Current-source renderer gate: `graphics::scene::scene_renderer::ui::sdf_render::tests::` passed 44/44; one explicit real-product exporter remained intentionally ignored.
- Originating upward gate: `scene::` ran 1,714 tests, with 1,705 passed, 6 ignored, and 3 failures. All three failures belong to concurrent renderer shadow binding/source-guard drift; Text, SDF, layout, dynamic-scene, and Editor02 scene assertions had zero failures.
- Scoped `rustfmt --check` and `git diff --check` passed. Production owner is 556 lines and folder-backed tests are 581 lines, both below the repository structure budgets.
- Handoff validator: 121 artifacts, 0 errors. Global plan-output audit retained five foreign violations only, all outside Text05.

## Review

Independent review final result: P0/P1/P2 = 0/0/0.

The reviewer confirmed the resident/new-load semantics, authoritative face-ID routing, strict test expectations, CompositeFont preservation, structure budgets, and canonical fixed-artifact provenance. No screenshot was created for this data/report repair; the M5 product renderer proof remains the existing real framebuffer evidence under `docs/tests/runtime/text`, never a strategy-text screenshot or a `target` artifact.
