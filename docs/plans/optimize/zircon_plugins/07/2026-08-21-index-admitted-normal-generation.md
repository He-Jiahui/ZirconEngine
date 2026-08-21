# Plugins07 Index-Admitted Normal Generation

- Date: 2026-08-21
- Owner: `optimize-plugins07-index-admission-r1-01a00797-20260821`
- Source plan: `docs/plans/optimize/zircon_plugins/07-first-party-asset-importer-source-dependency-subasset-artifact-determinism-sandbox-product-integration-review.md`, `IMP-P0-003`, `G05`
- Status: implementation complete; grouped managed regression and release measurements pending

## Problem

The first-party glTF and OBJ mesh helpers generated missing normals by converting file-provided
indices directly to position offsets. A malformed or damaged index could therefore panic inside
normal generation instead of returning the typed import failure required by the asset boundary.
The legacy calculation also loaded the first vertex position twice for every triangle.

## Change

- Both importers now validate triangle cardinality, checked `u32` to `usize` conversion, and every
  vertex bound before normal generation or acceptance of source-provided normals.
- Invalid meshes return `AssetImportError::Parse` with the rejected index and admitted vertex
  count. Focused glTF and OBJ regressions wrap the public conversion path with `catch_unwind` and
  require a typed error rather than a panic.
- Normal generation keeps the three admitted positions in local values, so the first position is
  not loaded again when calculating the second edge.

This slice covers triangle index admission and the normal-generation access pattern. It does not
claim the plan's broader accessor cardinality, finite-value, tangent, fuzz-corpus, or zero-publication
gates are complete.

## Deterministic Delta

For 65,536 triangles and eight normal-generation iterations per timing sample:

| Metric | Legacy | Index-admitted | Delta |
|---|---:|---:|---:|
| position component reads per sample | 6,291,456 | 4,718,592 | 25% fewer |
| explicit index admission checks per sample | 0 | 1,572,864 | fail-closed boundary added |
| out-of-range index behavior | panic possible | typed parse error | panic path eliminated |

The read counts are exact consequences of four versus three `Vec3` position loads per triangle.
Timing is a secondary guard because the new path deliberately adds validation work before the
calculation.

## Acceptance

- The glTF and OBJ behavior groups each require one non-ignored malformed-index regression to pass.
- The ignored glTF release benchmark runs 21 alternating legacy/admitted pairs with eight
  iterations per sample, emits both raw arrays, and uses nearest-rank P95.
- Index-admitted P95 may not exceed 110% of legacy while the 25% component-read reduction and
  admission-check counts must remain exact.
- Rust 1.94.1 formatting and scoped diff checks pass.
- Cargo regression counts and measured P95 remain pending the post-Main plugin aggregate batch; no
  timing result is claimed by this record yet.
