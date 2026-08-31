---
related_code:
  - zircon_runtime/src/core/framework/render/image/metadata_validation.rs
  - zircon_runtime/src/core/framework/render/image/metadata_validation/format.rs
  - zircon_runtime/src/core/framework/render/image/metadata_validation/tests.rs
implementation_files:
  - zircon_runtime/src/core/framework/render/image/metadata_validation.rs
  - zircon_runtime/src/core/framework/render/image/metadata_validation/format.rs
  - zircon_runtime/src/core/framework/render/image/metadata_validation/tests.rs
plan_sources:
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
  - docs/plans/zircon_runtime/render/13-texture-pipeline.md
  - docs/plans/optimize/zircon_runtime/92-runtime-texture-image-cubemap-array-volume-format-sampler-mip-compression-upload-streaming-residency-budget-eviction-virtual-texture-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/109-editor-texture-image-cubemap-render-target-sampler-compression-streaming-preview-current-source-review.md
tests:
  - zircon_runtime/src/core/framework/render/image/metadata_validation/tests.rs
  - rustfmt --edition 2021
  - static function, production-string, and test-name migration comparison
  - git diff --check
doc_type: milestone-detail
---

# Render 13 texture metadata validation owner split

## Status and completed items

| Milestone | Slice | Status | Date | Evidence |
|---|---|---|---|---|
| TX-M1.2 | Texture metadata validation folder-backed owner split | `render_plan13_texture_metadata_validation_owner_split_implemented_static_passed_managed_validation_deferred_algorithm_unchanged` | 2026-08-26 | Root 723 -> 173 lines; format owner 162 lines; test owner 394 lines; functions 13/13, production strings 61/61, tests 18/18 with zero delta. |

Completed:

- Kept the public diagnostic types and `validate_texture_metadata` rule route in the facade.
- Moved DDS/KTX/KTX2/ASTC and scalar format classification into a private format owner.
- Moved all 18 existing behavior tests into a dedicated test owner.
- Preserved rule order, severities, diagnostics, format tables, compression labels, and public paths.
- Recorded the typed schema/build/artifact/install architecture gap and the measurement gate required before algorithm optimization.

## Review basis

The local Unreal texture references separate authored settings, build/compression settings and services, build metadata, and runtime RHI resource ownership. Runtime review 92 and Editor review 109 show that Zircon's full target is likewise a typed schema/build/artifact/upload/residency system rather than a larger string validator. This slice establishes the narrow metadata-rule owner boundary without claiming that target is complete.

There is no compatibility module, duplicate implementation, public API expansion, algorithm replacement, rule change, new format support, or performance claim.

## Verification

- Scoped `rustfmt --edition 2021` completed for all three Rust files.
- Static migration comparison retained all 13 production functions, all 61 production string literals, and all 18 behavior tests.
- The facade has zero DDS/KTX/KTX2/ASTC/DXGI table tokens and mounts private `format` and test owners.
- All owners are below 400 lines; whitespace and conflict-marker scans passed.
- Tracked-root `git diff --check` passed with only the repository LF/CRLF checkout notice.
- Managed Cargo, importer/runtime product behavior, GPU, RenderDoc, profiler, and power validation were not run while bypassing the current shared validation blocker.

## Open scope

Render 13 and the complete runtime architecture remain `in_progress`. TX-M1.2 behavior qualification is deferred, and the full typed texture authority remains open: source/recipe identity, platform-qualified immutable artifacts, bulk mip/page artifacts, requested-versus-actual format truth, generation-qualified install receipts, upload/residency/budget ownership, Editor consumption, managed validation, profiling, milestone commit, coordinator integration receipt, and WeCom publication are not completed by this slice.
