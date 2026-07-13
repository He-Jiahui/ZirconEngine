---
related_code:
  - zircon_runtime/src/graphics/feature/render_feature_pass_descriptor/construct.rs
  - zircon_runtime/src/core/framework/render/shader/compute_dispatch.rs
  - zircon_runtime/src/core/framework/render/shader/fullscreen_pass.rs
implementation_files:
  - zircon_runtime/src/graphics/feature/render_feature_pass_descriptor/construct.rs
plan_sources:
  - docs/plans/zircon_runtime/shader/04-material-binding-and-renderer-contract.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - zircon_runtime/src/graphics/feature/render_feature_pass_descriptor/construct.rs
doc_type: module-detail
---

# Render feature shader authoring contract

## Purpose

`RenderFeaturePassDescriptor` consumes validated shader authoring plans and projects
their named resources into render-graph declarations. It is the shared boundary used
by compute features and fullscreen post-process passes; callers do not manually
duplicate resource access or binding-kind decisions after building a plan.

## Compute path

`ComputeDispatchBuilder` validates shader kind, compute entry point, dispatch extent,
and every declared resource by name, kind, and access. A valid plan supplies both the
graph workload metadata and graph resources through
`with_compute_dispatch_plan(...)`.

Storage buffers and storage textures become storage writes. Read-only buffers and
textures remain graph reads. Samplers participate in the shader ABI but are omitted
from the graph resource list because they do not own graph storage.

## Fullscreen path

`FullscreenPassBuilder` validates the fullscreen shader kind, fragment entry point,
and named pass inputs. `with_fullscreen_pass_plan(...)` projects its resources with
the same mapping. Frame bindings stay in group 0, pass inputs in group 1, and
generated parameter data in group 2 binding 0.

## Diagnostics

Named binding errors are returned as `ShaderDispatchBuildDiagnostic` before a graph
descriptor or WGPU pipeline is created. The diagnostic records the authored resource
name and expected/actual kind or access, so missing or mistyped resources remain
actionable authoring failures instead of backend validation failures.

The combined regression exercises a compute storage-buffer/texture mismatch and a
valid fullscreen texture projection in one feature-descriptor test. This locks both
the error and success paths at the shared graph boundary.
