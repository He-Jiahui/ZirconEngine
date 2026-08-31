---
related_code:
  - zircon_runtime/src/graphics/backend/render_backend/read_ibl_bake_artifact_sections.rs
  - zircon_runtime/src/graphics/backend/render_backend/read_ibl_bake_artifact_sections/resources.rs
  - zircon_runtime/src/graphics/backend/render_backend/read_ibl_bake_artifact_sections/batch.rs
  - zircon_runtime/src/graphics/backend/render_backend/read_ibl_bake_artifact_sections/staging.rs
  - zircon_runtime/src/graphics/backend/render_backend/read_ibl_bake_artifact_sections/tests.rs
implementation_files:
  - zircon_runtime/src/graphics/backend/render_backend/read_ibl_bake_artifact_sections.rs
  - zircon_runtime/src/graphics/backend/render_backend/read_ibl_bake_artifact_sections/resources.rs
  - zircon_runtime/src/graphics/backend/render_backend/read_ibl_bake_artifact_sections/batch.rs
  - zircon_runtime/src/graphics/backend/render_backend/read_ibl_bake_artifact_sections/staging.rs
  - zircon_runtime/src/graphics/backend/render_backend/read_ibl_bake_artifact_sections/tests.rs
plan_sources:
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
  - docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md
tests:
  - zircon_runtime/src/graphics/backend/render_backend/read_ibl_bake_artifact_sections/tests.rs::ibl_readback_root_remains_a_declarative_orchestration_owner
  - zircon_runtime/src/graphics/backend/render_backend/read_ibl_bake_artifact_sections/tests.rs::batched_cube_readback_strips_padding_in_face_then_mip_order
  - zircon_runtime/src/graphics/backend/render_backend/read_ibl_bake_artifact_sections/tests.rs::batched_readback_preserves_pmrem_sh9_and_iem_payload_bytes
  - rustfmt --edition 2021 --check
  - git diff --check
doc_type: milestone-detail
---

# Runtime 15 IBL GPU readback owner split

## Status and completed items

| Milestone | Slice | Status | Date | Evidence |
|---|---|---|---|---|
| M3/M4 | IBL GPU readback folder-backed owner split | `runtime_15_ibl_gpu_readback_owner_split_implemented_static_passed_managed_validation_deferred` | 2026-08-26 | Root 970 -> 67 lines; resource/batch/staging/test owners 81/274/189/401 lines; stable backend exports and callers unchanged. |

Completed:

- Split descriptor/resource requirements into `resources.rs`.
- Split command submission, asynchronous map state, error cleanup, and section assembly into `batch.rs`.
- Split buffer/cubemap staging and padded-row decoding into `staging.rs`.
- Moved the inline WGPU and seam fixtures into folder-backed `tests.rs`.
- Kept `read_ibl_bake_artifact_sections.rs` as a declarative orchestration owner.
- Added a structure regression that prevents concrete readback owners from returning to the root.
- Left the concurrently modified `render_backend/mod.rs` facade untouched; its four existing exports remain source-compatible.

## Review basis

The split follows the local Unreal RHI/RenderCore separation between readback lifecycle objects and explicit render-graph copy routing. It also follows Zircon's hard-cut and module-boundary rules: each child is named by domain responsibility, the root is thin, and there is no compatibility shim or duplicated implementation.

This slice deliberately does not alter the readback algorithm. One submission/device wait, asynchronous map callback counting, full error unmapping, and face-major/mip-major artifact ordering remain unchanged.

## Verification

- Scoped `rustfmt --edition 2021 --check` passed across this slice and the earlier owned runtime files.
- Final scoped `git diff --check` passed, apart from LF/CRLF checkout notices.
- Source line budgets are recorded above and all owners are below the 500-line focused ceiling.
- Static migration comparison found all 50 old function/type definitions and all 5 old tests in the split owners; the root contract passed 8/8.
- Managed Cargo and WGPU tests were not run because this work is explicitly bypassing the current validation blocker. They remain required before accepted milestone closeout.
- No CPU, GPU, energy, or power improvement is claimed because no performance algorithm changed.

## Open scope

Runtime 15 and the full runtime architecture remain `in_progress`. This record closes only the source ownership implementation for IBL artifact readback. Managed compile/test, live WGPU behavior execution, wider Runtime 15 guards, milestone commit, coordinator integration receipt, and WeCom publication remain open.
