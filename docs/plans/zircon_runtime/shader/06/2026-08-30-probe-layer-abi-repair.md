---
plan: shader06
scope: reflection-probe-layer-filter
status: implementation_complete_validation_pending
---

# Shader06 probe layer ABI repair

The fixed 16-byte `GpuReflectionProbeHeader` now carries the selected camera's
32-bit scene-schema mask in its former padding word. `GpuReflectionProbe.misc.w`
continues to carry the probe mask, and the shared `zr_environment_select_probes`
WGSL helper rejects a candidate whose mask does not intersect the camera mask
before evaluating influence weight.

This closes the existing camera/probe mask consumer gap without adding a
binding, allocation, shader permutation, sample, or PSO key. It does not claim
the M9 spatial assignment: object reflection-mask input is still absent from
the GPU scene/material ABI, so the fixed global scan remains the explicit
fallback until 09B/09C publish the typed local-list contract.

Focused source contracts:

- Rust header layout remains 16 bytes and serializes the camera mask.
- The legacy `with_probe_count` constructor defaults to `u32::MAX`, preserving
  historical all-layer visibility for callers that have not migrated to the
  explicit camera-mask constructor.
- WGSL consumes probe and camera masks before spatial weighting.
- Existing probe record layer-mask packing is unchanged.

Managed Cargo/WGPU, current-source screenshots, RenderDoc replay, GPU timing,
RSS/VRAM, and WPR/WPA power evidence remain open. The implementation must not
be marked as a fragment-scan performance improvement until the M9 assignment
ABI is wired and measured on the same scene at 1/8/32/64 probe counts.

The reflection-probe runtime now also exposes an explicit runtime-cache consumer:
it requires the request's source hash, reads only a current GPU-runtime artifact,
and reuses the existing PMREM-to-TextureAsset registration owner. Cache misses,
stale/rejected blobs, and missing source identity fail closed; this path never
creates or overwrites a source `.zcube`. Editor scene/catalog/undo publication
is still a separate owner boundary.

The runtime plugin now exposes `register_captured_reflection_probe_from_runtime_cache`.
It resolves only the current `RendererGpuRuntime` cache for the request's explicit
source hash, then reuses the PMREM texture registration owner in memory. This
removes the previous consumer gap without treating runtime output as an
`AssetImporterCpu` source bundle; editor scene/catalog/undo publication remains
pending.
