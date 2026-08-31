# Runtime 15 IBL GPU readback owner split

## Scope

- Target: `zircon_runtime/src/graphics/backend/render_backend/read_ibl_bake_artifact_sections.rs`.
- Baseline: current clean owner before this slice, 970 lines total: about 577 production lines and 393 inline test lines.
- Priority sources: `docs/plans/engine-code-structure-convention.md`, `docs/plans/engine-code-review-findings-2026-06.md`, and Runtime 15 M3/M4.
- This is an ownership and code-layout correction. It does not change the readback algorithm, claim lower CPU/GPU time or power, or close Plan 11 IBL product acceptance.

## Architecture review

The old file combined four independent responsibilities:

1. Descriptor-driven WGPU resource requirements and builder inputs.
2. One-submission readback batching plus asynchronous map/completion state.
3. Buffer and cubemap staging allocation, row alignment, and padding removal.
4. WGPU integration fixtures, texture upload helpers, and seam-quality assertions.

Call-site review found that renderer environment owners consume only the four existing backend exports. The adjacent `render_backend/mod.rs` is already a stable facade and is currently modified by another session, so this slice leaves it unchanged.

The primary Unreal reference was local `dev/UnrealEngine`:

- `Engine/Source/Runtime/RHI/Public/RHIGPUReadback.h` separates buffer and texture readback lifecycle objects.
- `Engine/Source/Runtime/RHI/Private/RHIGPUReadback.cpp` keeps enqueue, staging allocation, lock, and unlock inside those readback owners.
- `Engine/Source/Runtime/RenderCore/Private/RenderGraphUtils.cpp` adds explicit readback copy passes while renderer features retain their own pending state.

The resulting Zircon boundary follows that ownership model without copying Unreal APIs: the backend facade chooses artifact sections, the batch owner controls submission/map lifetime, and the staging owner controls WGPU layout conversion.

## Implemented layout

| Owner | Responsibility | Current lines |
|---|---|---:|
| `read_ibl_bake_artifact_sections.rs` | Declarative module wiring and descriptor-driven orchestration | 67 |
| `read_ibl_bake_artifact_sections/resources.rs` | Readback resource contract and required-slot error | 81 |
| `read_ibl_bake_artifact_sections/batch.rs` | Command batch, pending map lifecycle, completion, and section assembly | 274 |
| `read_ibl_bake_artifact_sections/staging.rs` | Buffer/cubemap staging and padded-row removal | 189 |
| `read_ibl_bake_artifact_sections/tests.rs` | Resource, layout, WGPU payload, and seam behavior tests | 401 |

The hard cut preserves the existing four exports and all current renderer call sites. There is no compatibility module, duplicate implementation, facade expansion, or generic `utils`/`helpers` owner.

## Behavior invariants

- A cache-miss artifact still uses one command encoder submission and one device wait on the synchronous path.
- The pending path still exposes the same command-buffer take, begin-map, poll-ready, and finish lifecycle.
- Map failure and callback disconnect still unmap every allocated staging resource.
- Cubemap bytes remain face-major, then mip-major, with WGPU row padding removed before artifact assembly.
- PMREM, SH9, and IEM resource requirements still derive only from `IblBakeArtifactDescriptor.contents()`.

## Current evidence and status

- Scoped `rustfmt --edition 2021 --check` passed across this slice and the earlier owned runtime files.
- Final scoped `git diff --check` passed, apart from the repository checkout's LF/CRLF notice.
- The root owner dropped from 970 to 67 lines; every child owner remains below 500 lines.
- An in-module source contract locks the declarative root, child mounts, and concrete owner placement.
- Static migration comparison found all 50 old function/type definitions and all 5 old tests in the new owners; one structure regression was added.
- Managed Cargo and WGPU execution were intentionally not requested while the shared validation path is blocked. Status is `implemented_static_passed_managed_validation_deferred`.

No performance or power result is reported because the algorithm did not change. Any later IBL readback optimization must start from GPU timestamp, CPU submission/map wait, transfer-byte, and cache-miss frequency baselines and must be reviewed separately from this structural slice.
