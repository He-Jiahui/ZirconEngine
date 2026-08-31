---
record_type: implementation-source-record
status: source_complete_validation_pending
created_at: 2026-08-30
owner_plan: docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
failure_key: asset-management-generation-projection
---

# Runtime04 Asset Management Generation

## Scope

This record covers the non-validation implementation slice for the missing immutable
asset-management projection. It is intentionally limited to the
`ProjectAssetManager` owner and does not move asset DTOs into `core::resource` or make
renderer-prepared material rows an asset-manager responsibility.

## Implemented boundary

- Added `ProjectAssetManagementGeneration` as an `Arc`-backed immutable snapshot containing
  asset-level model, mesh, scene, scene-entity, material, and shader record sets, stable per-kind
  `ResourceId` indexes, and both project-catalog and lower resource-generation identities.
- `ProjectAssetManager` stores the snapshot behind an owner lock. Construction starts with an empty
  closed-project projection; the first active project publication builds the rows from the already
  committed resource generation.
- `publish_project_generation` refreshes the snapshot before change broadcast and reactive wake
  delivery. The refresh is a no-op when both generation identities are unchanged and installs an
  empty snapshot when the project is closed. Open, close, import, watch, relocation, and deletion
  all therefore share the existing generation fence and publication owner.
- Stable management record reads now borrow the published rows and clone only the requested public
  return shape. Selected detail methods remain lazy payload loads. Renderer-prepared material rows
  are composed by the graphics consumer boundary and are not stored in the asset projection.

## Static evidence

- `rustfmt --edition 2021 --check` passes for the new generation owner and all touched manager
  owners.
- `git diff --check` passes for the scoped source, tests, and plan record (with repository-standard
  CRLF normalization warnings only).
- Source guards cover snapshot reader access, unchanged-generation refresh suppression, closed
  project clearing, and refresh-before-broadcast/wake ordering.
- Current owner file sizes after concurrent source convergence: `management_generation.rs` 243
  lines, `management.rs` 592 lines, `runtime.rs` 756 lines, and the graphics consumer accessor
  owner 850 lines; no touched owner approaches the structure convention hard limit.

## Deferred evidence

The managed Windows Cargo lane is still blocked at `cargo.acquire`, so no Cargo, focused test,
stable/one-percent/page scale, RSS, power, WGPU, commit, or WeCom acceptance evidence is claimed.
The existing broad Runtime04 Python audit also currently reports a missing guard owner for
`zircon_runtime/src/core/resource/tests.rs`; that finding is outside this source slice and was
not masked or changed. The parent failure remains `open` with
`implementation_complete / managed_validation_pending`.
