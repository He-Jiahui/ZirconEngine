Plan: docs/plans/zircon_runtime/text/03-line-breaking-measure-and-layout.md
Milestone: M3
Status: completed
Files: ["docs/plans/zircon_runtime/text/03/2026-07-09-line-breaking-measure-and-layout-output-records.md","docs/plans/zircon_runtime/text/03/2026-07-15-source-visual-geometry-m3-milestone-manifest.md","docs/zircon_runtime/ui/text/geometry.md","docs/zircon_runtime/ui/text/hit_test.md","docs/zircon_runtime_interface/ui/surface/render.md","zircon_runtime/src/ui/tests/text_hit_testing.rs","zircon_runtime/src/ui/text/geometry.rs","zircon_runtime/src/ui/text/geometry/source_metrics.rs","zircon_runtime/src/ui/text/geometry/tests/mixed_bidi.rs","zircon_runtime/src/ui/text/hit_test.rs","zircon_runtime/src/ui/text/hit_test/visual_source.rs","zircon_runtime/tests/runtime_text_multilingual_product_framebuffer.rs","zircon_runtime/tests/runtime_text_multilingual_product_framebuffer/proof_assertions.rs","zircon_runtime/tests/runtime_text_multilingual_product_framebuffer/proof_commands.rs","zircon_runtime_interface/src/ui/surface/mod.rs","zircon_runtime_interface/src/ui/surface/render/mod.rs","zircon_runtime_interface/src/ui/surface/render/text_geometry.rs","zircon_runtime_interface/src/ui/surface/render/text_geometry/mod.rs","zircon_runtime_interface/src/ui/surface/render/text_geometry/source_map.rs","zircon_runtime_interface/src/ui/surface/render/text_geometry/source_map/tests.rs"]

# Text03 M3 source/visual geometry implementation manifest

## Scope Delivered

`UiTextLineSourceMap` is now the single neutral owner for resolved-line source byte ranges, visual grapheme ranges, direction-aware caret affinity, discontiguous selection spans and resolved main-axis advance. Runtime caret/range/IME geometry, hit testing and renderer editable decorations consume that owner. The former monolithic interface geometry file and runtime hit-test mapper were deleted without aliases or compatibility shims; both owners are folder-backed and remain below the production-file budget.

The implementation preserves whole-run conservatism for non-isomorphic multi-grapheme replacements, projects the same advance onto y for `VerticalRl`, and authors a real mixed Hebrew/LTR editable product row for the following M4 framebuffer acceptance stage.

## Fresh Testing Evidence

- Exact owned `rustfmt --edition 2021 --check` and scoped `git diff --check` pass.
- Windows coordinator-managed `zircon_runtime_interface` source-map filter: 4 passed / 0 failed, including mixed-BiDi affinity, discontiguous spans, RTL inverse edges and non-isomorphic whole-run snapping.
- Independent review after remediation: 0 Critical / 0 Important / 0 Minor, Accept.
- Production owners remain within structure budgets: interface geometry 165 lines, source map 274 lines, runtime geometry 536 lines, source metrics 92 lines, hit test 223 lines and product exporter root 769 lines.

## Review

The independent reviewer first rejected ambiguous non-isomorphic run caret projection and weak per-decoration framebuffer evidence. Both were remediated with whole-run edge snapping, a fourth source-map regression, area/color-specific product assertions and a fresh exact format check. Final review is 0 Critical / 0 Important / 0 Minor, Accept.

## Acceptance Boundary

This M3 commit closes the architecture and implementation slice only. The output record intentionally remains `testing_pending` for default-feature runtime geometry/hit/IME filters and the exact ignored WGPU exporter. M4 must generate and visually inspect the new framebuffer only under `docs/tests/runtime/text`; no target-directory image is accepted.
