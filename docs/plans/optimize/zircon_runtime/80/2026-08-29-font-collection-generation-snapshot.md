---
record_kind: structural_performance_plan
status: static_implemented_managed_validation_pending
created_at: 2026-08-29
owner_plan: docs/plans/optimize/zircon_runtime/80-runtime-font-asset-source-cook-database-face-fallback-variation-color-resolved-glyph-cache-product-integration-current-source-review.md
related_text_plans:
  - docs/plans/zircon_runtime/text/01-font-resource-faces-and-database.md
  - docs/plans/zircon_runtime/text/02-shaping-unicode-and-bidi.md
  - docs/plans/zircon_runtime/text/09-threading-caching-and-performance.md
---

# Font Collection Generation Snapshot

## Current-source finding

The canonical shaping path probes a process-global `u64` font generation, then the cosmic
thread-local cache independently snapshots `FontDatabase`. A publication between those two
operations can make one shaping attempt observe an identity from one generation and backend
state from another. The bounded two-attempt retry prevents an infinite loop, but a number alone
is not a resource lifetime: it cannot keep the database bytes and backend catalog used by an
in-flight attempt alive.

`SharedFontDatabase::snapshot` also clones the complete `FontDatabase` after acquiring the
published state.
`FontDatabase` shares font bytes through `Arc<[u8]>`, but still clones indexes, the backend
catalog, and container state. This slice separates that work into
`text.font_database/shared_owned_snapshot_clone`; `shared_snapshot` now measures only the
publication lock and Arc acquisition. No p50, p95, RSS, or power conclusion is accepted before
managed measurement.

## Reference decision

Unreal Slate is the primary reference. `FSlateRenderer` retains `FSlateFontServices`;
`FSlateFontServices` retains the game/render-thread font caches and measure services; and
`FSlateFontCache` owns the composite cache, renderer, shaper, SDF generator, and FreeType state.
The relevant local sources are:

- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Rendering/SlateRenderer.h`
- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Rendering/SlateRenderer.cpp`
- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Fonts/FontCache.h`
- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Fonts/FontCache.cpp`

Zircon therefore needs one retained font service/collection owner. The immediate prerequisite is
an immutable `FontCollectionSnapshot` that owns its `FontCollectionService`,
`Arc<FontDatabase>`, stable collection identity, and exact generation.
This is not a second database or cache: it is the publication object that a future
session-owned `FontCollectionService` and its work leases will retain.

## Implementation slice

1. Publish the shared database as `Arc<FontDatabase>` and acquire generation plus the exact Arc
   while holding the same read lock.
2. Keep the existing owned-clone API temporarily only for mutable renderer-local consumers;
   canonical shaping must use the immutable snapshot directly.
3. Bind the cosmic thread-local font system refresh to the supplied snapshot rather than probing
   global state again. Collection identity plus generation is the invalidation key because equal
   generations are comparable only inside one collection; Arc identity is a lifetime mechanism,
   not another cache key.
4. Keep the current bounded publication retry around neutral handle projection. Removing that
   retry requires the next hard cut: collection-qualified handles plus a retained old-generation
   handle-registry lease.
5. Own the handle registry, immutable registry publication, and counters inside each
   `FontCollectionService`. `TextFontFaceHandle` carries collection, slot, and generation; a
   foreign collection rejects the complete pair even when generation and backend IDs match.
6. Bind canonical neutral projection, artifact projection, renderer SDF bake, and SDF font-asset
   caches to their supplied collection service. Process-default entry points remain adapters, not
   hidden state selected inside those consumers.
7. Acquire one immutable handle-resolver snapshot together with the database snapshot when a UI
   raster artifact view is signed. An in-flight view may finish after publication, while new view
   acquisition still requires the current collection generation.
8. Keep source registration and old-generation resolution batched. Stable raster resolution is
   one registry-snapshot Arc plus one database Arc per artifact view, not a database clone per
   line or a lock/probe per glyph.

## Complexity and measurement contract

- Stable snapshot acquisition: `O(1)` Arc clone under the existing read lock.
- Font publication: one `FontDatabase` clone and one Arc allocation on mutation; generation only
  advances when render inputs change.
- Stable shaping attempt: no extra database clone and no second database selection inside the
  cosmic cache.
- Handle registration/resolution: `O(U + N)` for `U` unique face/instance pairs and `N` projected
  glyph pairs; collection checks are O(1) per unique handle and old-generation resolution is
  lock-free after the resolver snapshot is acquired.
- Artifact lease: two O(1) Arc clones per view; no font bytes, backend catalog, or registry vector
  is copied.
- Generation refresh: still rebuilds at most four locale-specific cosmic font systems; the
  existing `generation_refresh` scope and face/entry counters remain authoritative.

Managed validation must compare the same 1/100/1k/10k text matrix before and after this cut and
report snapshot lock time, clone count, shaping restart count, cosmic refresh count, CPU p50/p95,
RSS, and package power. Until then the expected clone removal is an algorithmic claim only.

## Exit state

Current state: `collection_registry_and_inflight_lease_static_implemented /
managed_validation_pending`. Source regressions cover Arc reuse without publication, exact cosmic
snapshot binding, foreign-collection rejection, SDF collection isolation, and old database plus
registry resolution after publication. Rustfmt, scoped diff-check, and static global-reprobe guards
pass. Cargo and dynamic tests were deliberately not run outside the managed validation stage.
`RFF-P1-015` is statically implemented but remains unaccepted until managed tests run.
`RFF-P1-013` remains partial because the process adapter is still the default owner;
`RFF-P1-017` still needs document/window/PIE session injection; `RFF-P1-022` still needs
generational backend-face slot reclamation. No CPU, RSS, power, WGPU, or product screenshot claim is
made.
