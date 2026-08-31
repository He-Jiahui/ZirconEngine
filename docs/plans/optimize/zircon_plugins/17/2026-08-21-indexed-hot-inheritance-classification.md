# Plugins17 Indexed Hot-Inheritance Classification Optimization Record

- Date: 2026-08-21
- Owner: `optimize-plugins17-hot-inheritance-r1-01a00797-20260821`
- Source plan: `docs/plans/optimize/zircon_plugins/17-first-party-virtual-geometry-source-runtime-editor-dist-catalog-asset-cook-cluster-page-streaming-culling-raster-product-integration-review.md`, supporting NVG-P1-023 and NVG-P1-046
- Status: implementation and focused static validation complete; managed release timing queued

## Problem

Applying a completed GPU page table classifies newly resident pages that inherit
the previous hot frontier. The prior implementation linearly scanned every
previous slot owner for every candidate and rebuilt a descendant search by
scanning the complete child-to-parent table at every hierarchy level. A large
completion could therefore multiply candidate count by resident-slot count and
hierarchy-edge count during one frame completion.

## Change

- Previous slot owners are indexed once as `slot -> page` before candidate
  classification. Duplicate-slot behavior remains first-owner-wins, matching
  the prior page-ordered linear scan.
- Ancestors of surviving hot pages are computed once. Shared ancestry terminates
  at the first already-indexed parent, so common chains are not traversed again.
- A candidate now checks the slot index, walks only its own parent chain for a
  hot ancestor, and tests one precomputed set for a hot descendant.
- A focused equivalence regression covers replacement of a hot slot, inheritance
  from a hot ancestor, inheritance from a hot descendant, an existing page, and
  an unrelated page against the retained legacy oracle.

## Deterministic Performance Evidence

The managed release gate classifies 2,048 new candidates alongside 64 existing
hot pages over a 4,096-edge hierarchy. Both branches include construction of
their respective slot-owner representation inside the timed section.

| Measure | Legacy | Optimized | Gate |
|---|---:|---:|---:|
| Repeated descendant parent-edge scans | 8,388,608 | 0 | eliminated |
| Slot-owner operations | 131,072 linear comparisons | 2,048 indexed lookups | exact |
| Shared hot-ancestor preparation | repeated inside candidate search | 4,160 parent lookups, 4,096 indexed ancestors | exact |
| Timing distribution | 21 samples | 21 samples | alternating first-run order |
| Nearest-rank P95 | pending | pending | optimized <= 25% of legacy |

Exact Windows P50/P95 values remain pending the combined coordinator batch and
must be written here before integration acceptance.

## Current Execution Evidence

- Integration Session: `root-runtime-interface03-activate-link-failure-20260831`.
- Exact-path ownership transfer applied by request
  `7788690352e049958d59223c90847d58`, from preview
  `6a34f771bb334e428bef3d596538264e` and fingerprint
  `4962ef7107a638dc7fce4400bad7ba8b228b04dcf1df1b9984003964621c9244`.
- Current production source SHA-256:
  `F6CDB6A327A67B2693A730D1FE743D8DBE1F3D29750ABDB94097DE2FAB709961`.
- Deterministic work-model manifest SHA-256:
  `3826DE9DB61E243A8BD83B74CBE4413B809D847369034D04DC34CA6A08C7C3B7`.
  The model records repeated descendant edge scans `8,388,608 -> 0` and
  slot-owner operations `131,072 -> 2,048`, with `4,160` shared ancestor
  parent lookups producing `4,096` indexed ancestors.
- Focused source/model contract passed locally `7/7`; the source-bound managed
  static ticket is `2acef3a01bd94382b3d948612c6c0971`.
- Exact ignored Windows release benchmark ticket
  `34b4ab3be75c4f16a8300cb66650e8b0` is queued. Its 21 alternating sample
  pairs are the only accepted source for the pending P50/P95 values.

The pinned Plugins17 child validator is
`zircon-validation-plugins17-hot-inheritance.ps1` at SHA-256
`3BC10A8E18BE58CAB8EB98BAEC862EAB215C173DA0245A830A1A81EB57839279`.
It is aggregated with the existing seven plugin batches by
`zircon-validation-plugin-super-batch-eight.ps1` at SHA-256
`3D634CD1FA7BD66FB49040AE763EE592C5D781FE25A4E6B2588CE72C726905BD`.

## Acceptance

- Indexed classification returns exactly pages 1, 3, and 100 for the mixed
  descendant, ancestor, replacement, existing, and unrelated regression corpus,
  matching the legacy oracle.
- The benchmark times representation construction and complete inheritance
  classification for both branches.
- `indexed_hot_inheritance_release_benchmark` emits 21 alternating raw sample
  pairs, recomputable nearest-rank P50/P95 values, and exact hierarchy/slot work
  counts.
- Exact-file Rustfmt and scoped diff checks are green. Cargo regression and
  release timing are queued in the managed Windows lane; no synchronous wait
  or duplicate Cargo invocation is used.

## Remaining Scope

This slice does not add GPU generation/fence ownership, byte-budgeted streaming,
or a typed duplicate page/slot conflict. The current normalization path still
uses last-wins filtering before this classifier, and formal NVG-P1-037 conflict
admission remains required. Production-scale GPU, VRAM, I/O, and frame evidence
also remains part of NVG-P1-046 rather than this CPU micro-gate.
