Plan: docs/plans/zircon_runtime/text/09-threading-caching-and-performance.md
Milestone: M2
Status: implementation_complete_second_review_complete_validation_pending
Files: ["zircon_runtime/src/core/runtime/diagnostics/profiling/scope.rs", "zircon_runtime/src/core/runtime/tasks/pool.rs", "zircon_runtime/src/core/runtime/tests/tasks.rs", "zircon_runtime/src/text/layout/mod.rs", "zircon_runtime/src/text/layout/rich_advance_index.rs", "zircon_runtime/src/text/layout/rich_advance_index/tests.rs", "zircon_runtime/src/text/parallel/shape_pool.rs", "zircon_runtime/src/ui/text/measure_cache.rs", "zircon_runtime/src/ui/text/layout_engine/candidate_line.rs", "zircon_runtime/src/ui/text/layout_engine/wrapping.rs", "zircon_runtime/src/ui/text/layout_engine/wrapping/tests.rs", "zircon_runtime/src/ui/surface/render/extract.rs", "zircon_runtime/src/ui/surface/render/popup_menu.rs", "zircon_runtime/src/ui/surface/render/popup_options.rs", "zircon_runtime/src/ui/surface/render/text_prewarm.rs", "zircon_runtime/src/ui/surface/render/text_prewarm/profile.rs", "zircon_runtime/src/ui/tests/text_pipeline/render_extract_prewarm.rs", "zircon_runtime/tests/runtime_text_multilingual_product_framebuffer/product_project_fixture.rs", "zircon_runtime/tests/runtime_text_multilingual_product_framebuffer/proof_path.rs", "zircon_runtime/tests/runtime_text_multilingual_product_framebuffer/proof_output.rs", "docs/plans/zircon_runtime/text/09/2026-08-09-text-pipeline-performance-architecture-and-profiling-plan.md"]

# Text09 M2 Owner Prewarm And Proof-Path Manifest

## Scope Delivered

The retained UI owner-text path collects owner requests before command construction. At or above
the calibrated parallel threshold, only owner shaping enters the process shared compute pool while
command construction remains on the caller; an explicit frame context follows the worker and the
caller joins once at the layout dependency boundary. Text fields and open popup rows disable this
overlap so eager component layout keeps the shared cache; remaining owner and popup requests merge
into one post-command-collection prewarm/shape batch.

Resolved owner layout preserves document revision, viewport, and editable state. Plain horizontal
source uses source-isomorphic paragraphs; normal rich and vertical source uses canonical visible
hard lines with the base style, while inline rich source uses the layout's coalesced resolved-span
projection. Product proof fixtures and PNG-writer tests use workspace-local work roots under
`docs/tests/runtime/text`.

## Fresh Testing Evidence

The cold mixed-component profile requires `owner_overlap_joins=0`, one non-empty prewarm
sample/span, and one non-empty shape-batch sample/span, all on the caller frame. A separate pure
owner profile requires one overlap join and one frame-bound extract/prewarm/shape publication.
Stable-frame assertions require cache hits and resolved layouts for eight owners, an InputField,
and an open popup row. Scoped Rustfmt and diff checks are clean apart from existing CRLF notices.
Managed Windows Cargo, profiling baseline, and WGPU framebuffer proof are pending coordinator
validation.

## Review

Independent second source review found no P0/P1/P2 after the owner sidecar, caller-preserving
overlap, component fail-closed routing, frame attribution, and proof-path repairs.
