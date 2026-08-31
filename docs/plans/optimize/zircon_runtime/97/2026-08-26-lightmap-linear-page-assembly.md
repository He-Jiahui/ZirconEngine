# Runtime97 Lightmap Linear Page Assembly Optimization Record

- Date: 2026-08-26
- Owner: `root-runtime-events-20260824`
- Source plans: Runtime97 P1-11 and legacy 09F2 P1-8
- Status: implementation and release gate authored; batched managed validation pending

## Source Recheck

The current `lightmap_asset.rs` was clean before its exact-path lease was
acquired. `LightmapBakeOutput::validate()` already proves that the page list has
exactly `page_count` unique indices inside `0..page_count` and that every page
has the expected RGBA16F payload length. The conversion path can therefore use
the validated index as its ordering authority without comparison sorting.

## Problem

`texture_asset_from_lightmap_bake_output` collected every page reference into a
temporary vector, stable-sorted those references by `page_index`, and then used
a flattened collector to build the container payload. A large shuffled atlas
paid comparison-sort work after validation had already proven a complete page
permutation, and the flattened collector did not state the final byte capacity.

## Change

- Add one `ordered_lightmap_payload` owner after contract validation.
- Fill an exact-length borrowed-page table directly by validated `page_index`.
- Reserve the exact total payload byte count and append every page once in index
  order.
- Preserve validation errors, RGBA16F bytes, page ordering, descriptor fields,
  output type, and the original `LightmapBakeOutput` ownership contract.

## Deterministic Performance Evidence

| Workload | Before | After |
|---|---:|---:|
| 32,768 shuffled atlas pages | comparison sort, `O(P log P)` | 32,768 indexed writes, `O(P)` |
| 262,144-byte final payload | flattened collection | exact-capacity append |
| Page payload copies | one copy into final payload | one copy into final payload |
| Output ordering | ascending `page_index` | ascending `page_index` |

The ignored release gate runs 17 alternating legacy-sort/linear-index sample
pairs. Acceptance requires linear assembly nearest-rank P95 to be at most 70%
of legacy P95, a minimum 30% reduction. Exact Windows timings remain pending
the batched coordinator run.

## Acceptance

- `optimization_batch_20260826d_runtime97_lightmap_pages_use_linear_index_assembly`
  locks the validation-backed indexed production shape and exact byte reserve.
- `optimization_batch_20260826d_runtime97_lightmap_pages_preserve_index_order`
  compares all bytes against the legacy sorted result for 32,768 shuffled pages.
- `optimization_batch_20260826d_runtime97_lightmap_page_assembly_performance_evidence`
  emits `RUNTIME97_LIGHTMAP_LINEAR_PAGE_ASSEMBLY_BENCH_V1`, raw samples, page
  count, payload bytes, complexity labels, and the 30% P95 threshold.
- Exact-file Rust 1.94.1 rustfmt, source contracts, and scoped diff checks must
  pass before managed validation submission.

## Remaining Plan Work

This slice does not close Runtime97 or legacy 09F2. Artifact streaming,
move/zero-copy staging, byte-accounted residency, compression/mips, content
identity, multi-page device limits, build-data publication, and real product/GPU
qualification remain open.
