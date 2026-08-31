---
title: Editor34 Atlas Manifest Discovery Linear Collection
category: zircon_editor
report_id: Editor34-atlas-manifest-discovery-linear-collection-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor34 Atlas Manifest Discovery Linear Collection

## Scope

This slice removes pairwise path deduplication while the retained Editor host discovers TOML atlas
manifests from one filesystem directory. Case-insensitive extension filtering, deterministic path
ordering, missing-directory behavior, cache location, and manifest resolution remain unchanged.
Directory iteration yields each directory entry once, so the removed duplicate scan could not
change the result. This does not promote the retained-host cache manifest into the formal Sprite
Atlas asset product described by Editor34.

## Change

- Project successful `read_dir` entries directly to their paths.
- Filter TOML extensions once without scanning previously accepted paths.
- Use unstable ordering because equal paths cannot occur in a single directory enumeration.
- Keep filesystem access and missing-directory fallback outside the pure collection helper.

## Deterministic Performance Evidence

| One directory with 4,096 unique TOML manifests | Before | After |
|---|---:|---:|
| Pairwise path comparisons | 8,386,560 | 0 |
| Manifest extension classifications | 4,096 | 4,096 |
| Final path sort | 1 | 1 |
| Result ordering changes | 0 | 0 |

The ignored release gate runs 17 alternating sample pairs and emits
`EDITOR34_ATLAS_MANIFEST_LINEAR_COLLECTION_BENCH_V1`. Acceptance requires linear collection P95 to
be at least 75% below the legacy repeated dedup scan. Exact Windows timings remain pending the
coordinator run.

## Acceptance

- `optimization_batch_20260826bf_atlas_manifest_collection_filters_and_sorts_paths` covers
  case-insensitive TOML filtering, non-manifest exclusion, and deterministic ordering.
- `optimization_batch_20260826bf_atlas_manifest_collection_eliminates_pairwise_dedup` locks the
  8,386,560-comparison model and rejects pairwise candidate scans.
- `optimization_batch_20260826bf_atlas_manifest_collection_p95` reports paired release P50/P95
  samples and enforces the 75% P95 reduction gate.

## Remaining Parent-plan Work

Editor34 still owns formal Sprite and Atlas source identities, build recipes, derived artifacts,
atomic publication, generation-qualified residency, stable entry IDs, background jobs, authoring
toolkits, Scene persistence, and renderer integration. This slice only converges retained-host
cache manifest discovery.
