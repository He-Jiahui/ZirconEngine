# Render Product M3A Assets

## Scope
- M3A product asset/material readiness contracts from `docs/superpowers/plans/2026-05-08-render-m4-plus-product-pipeline.md`.
- Affected layers: neutral render asset descriptors under `zircon_runtime::core::framework::render`, asset-side texture/model/shader/material projection, imported-asset dependency projection, and graphics resource-streamer material readiness reporting.

## Baseline
- M1 render profile vocabulary existed, but `RenderProductFeature::{Image, Mesh, Shader, Material}` did not yet map to product-ready asset contracts with readiness diagnostics.
- M3A focused test execution was previously blocked before render tests by unrelated `scene::tests::ecs_systems` lib-test compile errors requiring `Debug` for `SystemState<ResParam<MissingScore>>` and `SystemState<ResMutParam<MissingScore>>`.
- The current shared checkout remains heavily dirty with active ECS, reflection, UI, and AccessKit sessions; this record covers narrowed M3A runtime gates, not full workspace acceptance.

## Test Inventory
- Focused asset cases: `zircon_runtime/src/asset/tests/assets/render_product.rs` covers render image metadata, mesh bounds/VG presence, runtime WGSL source selection, material dependency extraction, invalid alpha cutoff, and structured fallback/readiness reports.
- Streamer cases: `zircon_runtime/src/graphics/scene/render_product_streamer_tests.rs` covers missing shader/texture readiness reports, blocking missing runtime WGSL, repeated blocking behavior, and dependency readiness revalidation after a prior successful prepare.
- Lower-layer blocker check: `scene::tests::ecs_systems::optional_resource_params_return_none_while_required_resources_error` verifies the prior required-resource `Debug` compile blocker no longer prevents lib-test execution.
- Boundary cases covered: non-finite/out-of-range alpha mask cutoffs, non-WGSL shader source without emitted WGSL, missing shader references, missing texture references, and container texture payloads that have metadata but are not upload-ready in M3A.
- Negative cases covered: unresolved shader/texture dependencies produce validation errors and fallback usage, and `RenderMaterialReadinessReport::is_ready()` rejects reports with fallback usage.

## Tooling Evidence
- Tool name: Windows Cargo through PowerShell with `CARGO_TARGET_DIR=E:\cargo-targets\zircon-render-m3`.
- Why these tools were used: the observed blocker and acceptance gates were Rust compile/test behavior, so focused Cargo checks directly exercise the affected layers.
- WSL-specific tools were not used for this narrowed gate because no platform-specific crash, memory, or thread-safety failure was observed; Linux CI-parity remains a later full-workspace gate.
- Exact commands:
- `cargo test -p zircon_runtime --lib scene::tests::ecs_systems::optional_resource_params_return_none_while_required_resources_error --locked --message-format short`
- `cargo test -p zircon_runtime --locked render_product_assets --message-format short`
- `cargo test -p zircon_runtime --locked material --message-format short`
- `cargo check -p zircon_runtime --lib --locked --message-format short`

## Results
- Lower-layer ECS blocker check passed on 2026-05-09: 1 passed, 0 failed.
- M3A render asset gate passed on 2026-05-09: 6 focused `render_product_assets` tests passed, 0 failed.
- M3A material gate passed on 2026-05-09: 47 lib tests plus 1 integration test passed, 0 failed.
- Scoped runtime library check passed on 2026-05-09: `cargo check -p zircon_runtime --lib --locked --message-format short` finished successfully.
- Warnings observed during test compilation were unrelated dead-code warnings for `new_with_virtual_geometry_for_test` and `new_for_test`.

## Acceptance Decision
- Accepted for narrowed M3A product asset/material readiness based on the focused asset tests, material/streamer tests, and scoped runtime library check listed above.
- Remaining risks: full workspace build/test, plugin workspace validation, WSL/Linux CI parity, M4A core pipeline phases, M4B postprocess, M5A PBR/light renderer integration, sprite, AA, VG/HGI, and Solari remain open later milestones.
