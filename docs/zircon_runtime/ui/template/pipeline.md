---
related_code:
  - zircon_runtime/src/ui/template/mod.rs
  - zircon_runtime/src/ui/template/pipeline.rs
  - zircon_runtime/src/ui/template/loader.rs
  - zircon_runtime/src/ui/template/validate.rs
  - zircon_runtime/src/ui/template/instance.rs
  - zircon_runtime/src/ui/template/build/surface_builder.rs
  - zircon_runtime/src/ui/template/asset/compiler/package/artifact.rs
  - zircon_runtime/src/ui/tests/template_pipeline.rs
implementation_files:
  - zircon_runtime/src/ui/template/pipeline.rs
  - zircon_runtime/src/ui/template/instance.rs
  - zircon_runtime/src/ui/template/asset/compiler/package/artifact.rs
  - zircon_runtime/src/ui/tests/template_pipeline.rs
plan_sources:
  - docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md
  - docs/ui-and-layout/shared-ui-template-runtime.md
  - docs/engine-architecture/generated-code-boundary.md
tests:
  - zircon_runtime::tests::runtime_absorption::ui_architecture::runtime_09_template_pipeline_boundary_records_compile_instance_validate_authority
  - zircon_runtime::ui::tests::template_pipeline::template_validate_rejects_unknown_component_contract
  - zircon_runtime::ui::tests::template_pipeline::template_instance_failure_surfaces_loader_error
  - zircon_runtime::ui::tests::template_pipeline::compiled_template_artifact_stays_binary_leaf_dto_not_generated_source
doc_type: module-detail
---

# UI Template Pipeline Boundary

runtime_09_m3_1_template_compile_instance_validate_boundary_static_passed_cargo_pending

`zircon_runtime::ui::template::UiTemplateRuntimePipeline` is the runtime entry point for the old recursive template path while it remains available for migration and tests. It fixes the observable phase order through `UI_TEMPLATE_RUNTIME_PIPELINE_STAGES`:

```text
load -> validate -> instance -> build
```

The phase split is intentionally thin:

- `UiTemplateLoader` owns TOML string/file parsing and IO errors.
- `UiTemplateValidator` owns structural template validity before any expansion.
- `UiTemplateInstance::from_validated_document(...)` owns already-validated template and slot expansion.
- `UiTemplateSurfaceBuilder` owns converting an instance into a lazy `UiSurface`.

`UiTemplateRuntimePipelineError` keeps these phases visible as `Load`, `Validate`, `Instance`, and `Build` variants. That makes failure ownership explicit: malformed TOML stays a loader error, unknown component/template references stay a validate error, expansion defects stay an instance error, and tree/surface construction defects stay a build error.

The acceptance anchors are present but broader behavior execution is still deferred by the current implementation-first request:

- `template_validate_rejects_unknown_component_contract`
- `template_instance_failure_surfaces_loader_error`
- `compiled_template_artifact_stays_binary_leaf_dto_not_generated_source`

Generated output policy is also explicit. `UiRuntimeCompiledAssetArtifact` records `runtime_09_m3_1_binary_leaf_dto_artifact_not_generated_source` and `requires_generated_source_marker() == false` because the current compiled template artifact is a binary/TOML payload DTO, not a generated Rust/source file. If a future template compiler writes source files, the first line must follow Runtime 02 M4 generated-code policy:

```rust
// @generated <generator> - do not edit by hand
```

Generated source remains limited to leaf DTO/table/adaptor material. Runtime behavior, validation rules, loader policy, instantiation, and surface mutation stay handwritten in `zircon_runtime::ui::template`.
