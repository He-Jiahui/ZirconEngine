---
related_code:
  - zircon_runtime/src/graphics/runtime/render_framework/budget
  - zircon_runtime/src/graphics/runtime/render_framework/capability_summary
  - zircon_runtime/src/graphics/runtime/render_framework/frame_profiler.rs
  - zircon_runtime/src/graphics/runtime/render_framework/frame_profiler
  - zircon_runtime/src/graphics/runtime/render_framework/query_stats
  - zircon_runtime/src/asset/assets
related_plans:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/02-unreal-aligned-engine-system-hard-cutover.md
write_scope: []
status: pending
---

# Framework budget and asset closure

This is a current-source static closure record. Both scopes remain pending because
the current-source Cargo/product gate is not green; neither scope enters
`review.md` on static evidence alone.

## Scope

- Framework budget/capability/frame-profiler/query-stats: 12 Rust files, 1,481
  physical lines, 1,355 nonempty lines, 59,128 bytes, 20 inline tests; raw
  content SHA256 `101999a880f0531cab7fb2fd2d5c08185b92bb8f3a2da083d306822d4df0c9f7`.
  Isolated rustfmt passes 8/12; the four failures are foreign formatting work.
- `zircon_runtime/src/asset/assets/**`: 114 Rust files, 24,135 physical lines,
  21,963 nonempty lines, 834,255 bytes, 146 tests, 21 ignored tests and 12
  include sites; raw content SHA256
  `f9665b59df0d5314230783e8a350fac671e2834e21bd59065cb61f29e1104eca`.
  Isolated rustfmt passes 66/114; 44 tracked and 11 untracked paths contain
  foreign work.

## Retained findings

### Framework

- Per-pass upload traffic is compared with a resident staging budget; persistent
  residency reports only `ResourceStreamer`; graph transient peaks are mixed with
  a partial resident pool.
- The global degrade ladder advances per accepted viewport submission, so pressure
  and recovery depend on viewport count and submission order.
- A terminal-camera profile mixes whole-stack CPU time with one camera's graph/GPU
  identity. Four retained profiles can expire late GPU results. Disabled
  diagnostics still build pass/subsystem profiles and labels, and full
  `RenderStats` queries deep-clone broad state.
- Advanced feature compile eligibility and runtime capability admission use split
  recipes.

### Assets

- Mesh/model preparation can materialize complete AoS geometry and deep-clone SDF
  and virtual-geometry payloads merely to inspect a primitive; mirrored subassets
  remain separate authorities.
- Project document conversion creates multiple full TOML value/intermediary
  generations during decode and encode without node/depth/expanded-byte admission.
- Texture manifest and array/cubemap assembly retain source, decoded, converted,
  layer and successor allocations at once without an aggregate peak proposal.
- External DDS/KTX and zcube paths can bypass the canonical cubemap face cap before
  allocating quadratic f32 face-major payloads.
- Public asset payload types expose deep `Clone`; shader index packing, recursive
  scene/UI traversal and WAV decode retain pending schema/byte-budget audits.
- Lightmap assembly is O(P) but retains bake pages and a complete ordered payload;
  PMREM extraction creates a second durable texture payload.

## Architecture handoff

1. Make `DeviceProfile` the sole immutable capability/budget source and publish one
   device-scoped `RendererResidencyGeneration`. Keep upload traffic separate from
   current/candidate/pending/retired residency.
2. Evaluate pressure once per device/presentation epoch and publish child-camera
   and viewport-submission profile generations. Tie GPU profile lifetime to the
   terminal query owner; use dense compiled labels and Arc-backed/narrow queries.
3. Compile immutable `AssetPayloadGeneration` slabs and typed views. Decode and
   assembly begin only after a bounded proposal admits source bytes, dimensions,
   nodes, decoded/current/candidate/peak bytes and deadline.
4. Use one schema-directed bounded project conversion, transfer or stream texture
   slabs, and enforce device/profile face caps before external or zcube f32
   allocation. Publish cooked artifacts only after complete generation validation.
5. Diagnostics Disabled performs no management projection; explicit exports are
   bounded and Arc-backed.

## Gates and ownership

M0 adds scale, byte, clone and residency RED counters. M1-M2 establish the device,
frame-profile and asset decode/assembly generations. M3-M5 implement transactional
publication, feature-specific residency, bounded document/texture conversion and
generation-qualified artifacts. M6 runs current-source Cargo, F0/F2/F4 product
traces, WPR/Tracy/RenderDoc and power measurements before any module is accepted.

Route framework work to Render01/17, Runtime07/11 and the framework hard-cut plan;
route asset payload/decode/material/texture work to Runtime04/08/10/11 and the
asset plans under `docs/plans/optimize`. Existing Unreal evidence is retained for
current/used/peak memory separation, per-mip ownership transfer, exact subresource
identity and terminal candidate publication; it is not a Zircon runtime benchmark.

## Validation status

Known blockers remain: the UI/layout and text production compiler errors, the SDF
cfg(test) mismatch, stale compiled-scene/OIT source guards and the graphics feature
reexport issue. No current-source executable, WPR, RenderDoc or benchmark result is
claimed. No production code was changed.
