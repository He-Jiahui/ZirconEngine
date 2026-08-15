---
record_kind: milestone_validation_manifest
status: pending_validation
created_at: 2026-08-15
plan: docs/plans/zircon_runtime/render/01-render-graph-rdg-alignment.md
milestone: M3
---

Plan: docs/plans/zircon_runtime/render/01-render-graph-rdg-alignment.md
Milestone: M3
Status: pending_validation
Files: ["docs/plans/zircon_runtime/render/01-render-graph-rdg-alignment.md", "docs/plans/zircon_runtime/render/01/failure-2026-07-18-render-graph-compile-analysis-scaling.md", "zircon_runtime/src/graphics/pipeline/render_pipeline_asset/pass_authoring.rs", "zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile_tests.rs", "zircon_runtime/src/render_graph/builder/compile.rs", "zircon_runtime/src/render_graph/tests/resources.rs", "zircon_runtime/src/render_graph/tests/culling.rs"]

# M3 resource-hazard DAG validation manifest

## Scope delivered

- Resource hazards have one owner in `RenderGraphBuilder::compile`: manual edges seed one deduplicated adjacency, then per-resource access history supplies RAW, WAW, and WAR edges used by culling.
- Pipeline pass authoring only declares accesses and topologically orders a unique producer before its readers. It no longer creates a global authoring-order dependency chain or a second resource-history authority.

## Validation manifest

- Windows managed package compile: `zircon_runtime` default feature set.
- Managed focused regressions: RenderGraph resource/culling contracts, compiled-pipeline cache contracts, and render-pipeline pass-authoring contracts.
- Static gates already recorded for this exact scope: Rust 2021 formatting, scoped `git diff --check`, dependency-source contract, and structural review against the 800-line production/test budgets.

## Fresh testing evidence

- Coordinator actions validation `40062d1acf3e4e4087bcf6784e575a9e` was accepted with 44/44 workflow tests, but it did not execute the declared Rust package or focused regressions.
- A post-review `clear_discard -> load_store` regression was added afterwards, so that accepted workflow receipt does not cover the current source manifest. A coordinator-managed current-source Rust validation ticket is still pending. No Cargo, product, or visual result is claimed by this manifest.
- Managed focused-test attempt `4ec7f1e5e5d348bda48d3f23a9203e51` invoked `cargo test -p zircon_runtime --locked --lib render_graph` through `validate-matrix.ps1`, used the coordinator target pool, and exited `101` before running tests. Rust compiles the complete `zircon_runtime` `cfg(test)` library for that command; the durable Cargo fingerprint reports 368 pre-existing diagnostics in other test/module families. The Render01 source paths in this manifest have zero primary-span diagnostics. This is an integration-test-configuration blocker, not a focused-test pass or a source-complete validation result.

## Review

- Static independent review of the final resource-hazard source snapshot: critical 0, important 0, minor 0. It covers RAW/WAW/WAR direction, culling over the final adjacency, removal of authoring-order serialization, and the new discarded-store attachment `Load` diagnostic regression.
- The M3 source change still requires a fresh independent review after managed validation for milestone closeout.

## Deferred product evidence

- After the focused Rust batch is green, capture the default render pipeline on the same source fingerprint, retain PNG evidence under `docs/tests/runtime/render/`, and compare graph dump pass order with a RenderDoc capture from `D:\Tools\renderdoc`.
