Plan: docs/plans/optimize/zircon_runtime/99-runtime-volumetric-fog-froxel-local-fog-volume-lighting-shadow-history-temporal-reprojection-product-integration-current-source-review.md
Milestone: M8
Status: completed
Files: ["zircon_runtime/src/graphics/scene/scene_renderer/advanced_lighting/froxel/media_inject.rs", "zircon_runtime/src/graphics/scene/scene_renderer/advanced_lighting/froxel/media_inject/performance_tests.rs", "zircon_runtime/src/graphics/scene/scene_renderer/advanced_lighting/froxel/executors/media_inject.rs", "tools/tests/test_runtime99_fog_volume_upload_performance_contract.py"]

# Runtime99 Fog Volume Upload Filter

## Scope delivered

This batch closes the CPU-side `P1-29` whole-vector clone in the volumetric media-inject path while
preserving render-layer selection, GPU buffer ABI, dispatch sizing, and the existing unfiltered
test/export entry point.

- The executor borrows the selected camera's render-layer set and the frame extract's fog-volume
  slice. It no longer clones either the camera culling mask or a filtered `Vec<FogVolumeData>`.
- `prepare_for_layers` validates settings and directly fuses layer filtering with conversion into
  the single contiguous `Vec<GpuFogVolume>` that is uploaded.
- Low quality returns an empty GPU-volume vector before visiting any local volume, instead of
  filtering and cloning all visible volumes before later discarding them.
- The prepared CPU request owns the GPU DTO vector before encoder mutation starts, keeping the
  immutable frame-extract borrow separate from the mutable command-encoder borrow.
- Focused equivalence coverage compares enabled, disabled, and unfiltered projections against the
  previous clone-first behavior.

The broader Runtime99 plan remains open: independent Local Fog scene truth and shape fidelity,
view culling and GPU binning, persistent upload/storage resources, lighting/environment
convergence, history validity, transparent composition, authoring, and product qualification are
not claimed by this slice.

## Fresh testing evidence

TDD first produced three failures against the old executor clone, missing fused collector, and
missing release benchmark. After implementation, the Python source contract passes 3/3, Python
bytecode compilation passes, Rust 1.94.1 formatting/parsing passes for all three Rust files, and
scoped whitespace validation passes.

Five process-level repetitions of a standalone Rust 1.94.1 optimized benchmark produced these
median-of-run nearest-rank values. Every process used 65,536 local volumes, 21 alternating
legacy/optimized pairs, a half-visible enabled view, and a fully visible disabled view. The managed
ignored test uses the actual module types and four operations per sample to reduce scheduler noise.

| workload | legacy | optimized | reduction |
| --- | ---: | ---: | ---: |
| 65,536 volumes, 32,768 visible, P50 | 7.4067 ms | 1.0521 ms | 85.795% |
| 65,536 volumes, 32,768 visible, P95 | 16.9597 ms | 2.1132 ms | 87.540% |
| enabled-path full-volume clones | 32,768 | 0 | 100% |
| enabled-path output vectors | 2 | 1 | 50% |
| 65,536 visible but local volumes disabled, P50 | 14.0198 ms | 0.0001 ms | 99.999% |
| 65,536 visible but local volumes disabled, P95 | 24.6416 ms | 0.0004 ms | 99.998% |
| disabled-path volume visits | 65,536 | 0 | 100% |
| disabled-path full-volume clones | 65,536 | 0 | 100% |

The Windows managed validation batch compiles the actual `zircon_runtime` graphics profile, runs
the behavior/equivalence test, and enforces at least 75% enabled-path and 90% disabled-path P95
reductions. No local Cargo command or Cargo dry-run was launched.

## Review

The old filtered DTO vector was private to this executor and was immediately converted into a
second vector, so borrowing the frame extract until preparation does not change ownership visible
to callers. `encode` remains as the unfiltered compatibility entry for existing module-local GPU
tests, while the executor's prepared path reports upload bytes from the exact converted vector.
Independent review remains an integration gate after managed validation returns.
