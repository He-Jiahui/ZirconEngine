---
title: Runtime93 Selection Mesh Lookup Optimization
category: zircon_runtime
report_id: Runtime93-selection-mesh-lookup-2026-08-24
date: 2026-08-24
session_id: root-runtime93-selection-lookup-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime93 Selection Mesh Lookup Optimization

## Scope

This slice removes the per-selected-entity full mesh scan from the SceneRenderer selection outline
path. It advances Runtime93's extract/prepare scaling work without changing mesh extraction,
visibility, LOD, bounds, resource streaming, GPU buffer lifetime, or the wider overlay architecture.

## Implementation

Selection lookup is now adaptive. Up to eight selected entities retain the former allocation-free
linear lookup, protecting the common single-selection path. Larger selections build one temporary
`EntityId -> &RenderMeshSnapshot` index and then resolve every selected entity through that index.

The index uses first insertion wins, preserving the former `Iterator::find` behavior if malformed
input contains duplicate mesh owners. Selected entities are still visited in caller order, missing
entities are skipped, and model/mesh bounds expansion is unchanged.

Regression coverage checks both lookup branches, caller order, missing owners, and duplicate-owner
first-match behavior. A source contract guards the adaptive threshold, one-index construction, and
first-insertion policy.

## Performance Evidence

| Evidence | Before | After / target | Structural change |
| --- | ---: | ---: | ---: |
| 2,000 tail selections in a 10,000-mesh frame | 18,001,000 entity comparisons | 12,000 total operations: 10,000 index inserts + 2,000 lookup probes | 99.9333% lookup-work reduction |
| Batch-selection complexity | O(selected x meshes) | O(meshes + selected) average | one frame-local borrowed index |
| Common selection size <= 8 | allocation-free linear lookup | allocation-free linear lookup | no common-path index allocation |
| 10,000 meshes / 2,000 selections release p95 | dynamic evidence pending | <= 100 ms and <= 50% of legacy p95 | coordinator release gate |

The ignored Windows-native release evidence alternates 11 legacy/optimized sample pairs and prints
`RUNTIME93_SELECTION_LOOKUP_BENCH_V1` with exact p95 nanoseconds, the target, mesh/selection counts,
and deterministic lookup-work counts. Dynamic elapsed time is accepted only from coordinator
terminal evidence.

## Validation

- Exact `rustfmt --check`, scoped `git diff --check`, and seven selection lookup/evidence source
  contracts: passed.
- Five non-ignored Runtime93 behavior regressions, two ignored release gates, and the locator model
  are submitted together in one coordinator-managed three-task batch using the `runtime93_` filter.
- No local Cargo lane is launched, and no coordinator compilation is monitored in real time.
- Terminal marker values, commit integration, optimization-record finalization, and automatic WeCom
  delivery remain pending that managed batch.

## Remaining Parent-plan Work

Selection outlines still expand model bounds into CPU line vertices and create/update overlay GPU
buffers through the broader SceneRenderer overlay path. Runtime93's authoritative geometry,
generation, stable-frame extraction, LOD, deformation, collision, and streaming milestones remain
separate work and are not claimed complete by this lookup optimization.
