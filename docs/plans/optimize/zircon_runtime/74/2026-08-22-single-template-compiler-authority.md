# Runtime74 Single Template Compiler Authority

Plan: docs/plans/optimize/zircon_runtime/74-runtime-ui-template-component-binding-expression-model-event-command-hot-reload-product-integration-review.md
Milestone: M1
Status: validation_pending
Files: ["docs/editor-and-tooling/crate-boundary-audit-round-2.md","docs/engine-architecture/runtime-architecture-review-m0.md","docs/engine-architecture/runtime-interface-convergence.md","docs/plans/optimize/zircon_runtime/74/2026-08-22-single-template-compiler-authority.md","docs/zircon_runtime/ui/architecture.md","docs/zircon_runtime/ui/template/pipeline.md","docs/ui-and-layout/shared-ui-template-runtime.md","zircon_editor/src/ui/template/catalog.rs","zircon_runtime/src/tests/runtime_absorption/structure_convention/facade_surface.rs","zircon_runtime/src/tests/runtime_absorption/ui_architecture/architecture_boundaries.rs","zircon_runtime/src/ui/prelude.rs","zircon_runtime/src/ui/template/instance.rs","zircon_runtime/src/ui/template/loader.rs","zircon_runtime/src/ui/template/mod.rs","zircon_runtime/src/ui/template/pipeline.rs","zircon_runtime/src/ui/template/validate.rs","zircon_runtime/src/ui/tests/block_box_layout.rs","zircon_runtime/src/ui/tests/canvas_slot_template.rs","zircon_runtime/src/ui/tests/event_routing.rs","zircon_runtime/src/ui/tests/template.rs","zircon_runtime/src/ui/tests/template/interaction_bindings.rs","zircon_runtime/src/ui/tests/template/layout_compute.rs","zircon_runtime/src/ui/tests/template/loader_instance_validation.rs","zircon_runtime/src/ui/tests/template/slot_contracts.rs","zircon_runtime/src/ui/tests/template/surface_containers.rs","zircon_runtime/src/ui/tests/template_grid_flow.rs","zircon_runtime/src/ui/tests/template_pipeline.rs","zircon_runtime/src/ui/tests/boundary/template_namespace.rs","zircon_runtime_interface/src/tests/contracts.rs","zircon_runtime_interface/src/tests/ui_contract_spine.rs","zircon_runtime_interface/src/ui/template/document.rs","zircon_runtime_interface/src/ui/template/mod.rs"]

- Date: 2026-08-22
- Owner: `optimize-runtime74-param-ref-compile-r3-bee4c707-20260822`
- Source item: `RTB-P1-001`
- Delivery state: implementation complete; grouped coordinator validation pending

## Problem

The asset `UiDocumentCompiler` and the legacy `UiTemplateRuntimePipeline` both accepted source
templates, validated them, expanded component/slot structure, and produced surface input. The
legacy path was used only by tests but remained publicly exported, so tests and product could
observe different validation and artifact contracts.

## Scope Delivered

- The only source path is `UiAssetLoader -> UiDocumentCompiler -> UiCompiledDocument`.
- `UiTemplateSurfaceBuilder::build_surface_from_compiled_document` consumes that canonical output.
- The legacy loader, validator, pipeline, document/error DTOs, exports, and prelude entries are
  deleted as a hard cut with no alias or compatibility facade.
- Layout/input tests deserialize an already-compiled `UiTemplateNode` fixture through a test-only
  DTO helper that rejects unresolved template, slot, and slot-fill fields; it does not expand or
  validate source templates.
- Boundary tests lock the single-authority export surface and reject all retired names.

## Deterministic Performance Gate

`template_compiler_authority_has_bounded_p95_latency` runs 21 samples with 100 canonical compiles
per sample, sorts the samples, selects nearest-rank P95, and requires the batch P95 to remain at or
below 250,000 microseconds. It emits:

`PERF-RUNTIME74-COMPILER-AUTHORITY sample_count=21 compiles_per_sample=100 samples_us=<21-raw-samples> compile_p95_us=<measured> p95_limit_us=250000 runtime_compiler_authorities=1 legacy_runtime_pipeline_exports=0`

The measured value is pending coordinator execution. No performance pass is claimed yet.

## Acceptance

- `asset_compiler_is_the_single_template_compile_authority` covers load, compile, and surface build.
- `legacy_recursive_template_document_is_not_a_runtime_compile_input` locks the source hard cut.
- `template_compiler_authority_has_bounded_p95_latency` supplies 21-sample nearest-rank P95 data.
- `bound_custom_template_component_dispatches_click_envelope_after_build` locks migrated compiled-node fixtures through the real event-routing path.
- `runtime_74_template_boundary_has_one_compiler_authority` statically locks owner files and exports.
- Grouped Cargo compile/tests and the performance gate are pending asynchronous coordinator
  validation.
