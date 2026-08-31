# Rich table-cell projection profile and structural redesign

Date: 2026-08-30

Status: `baseline_profile_complete / interval_projection_implemented_static /
isolated_post_profile_complete / managed_product_validation_pending`

## Scope

This review covers compiled rich-table cell projection construction. It does not claim parser,
table layout, shaping, renderer, frame-time, package-power, or product acceptance.

`CompiledRichText::table_cell_projection_indices` currently visits every table cell and rescans all
runs, paragraphs, and tables three times to produce the indices intersecting that cell. The result is
cached once, but one-time ownership does not make its `O(C * (R + P + T))` build cost acceptable.
`UiParsedText::from_projection` then copies, sorts, and deduplicates those already ordered indices,
adding a second avoidable projection step.

Local Unreal source keeps rich parsing limited to stripped output plus line/run ranges and metadata:
`FDefaultRichTextMarkupParser::Process` emits `FTextLineParseResults` and
`FTextRunParseResults`, while `URichTextBlock` supplies the parser and decorators to
`FRichTextLayoutMarshaller`. Zircon-specific table projection must therefore remain a bounded
marshaller/layout index derived from canonical ranges; it must not become another parser, cluster
authority, or process-global cache.

## Baseline method

An isolated optimized Rust benchmark was compiled and run entirely under
`E:/Git/ZirconEngine/target/codex_text_profile`, with compiler temporary files redirected to that
same E-drive directory. It reproduces the current three full scans for `N` canonical, non-overlapping
runs, paragraphs, top-level tables, and one cell per table. Each cell has one matching run and one
matching paragraph, so emitted output is linear while the scan work is exactly `3 * N * N` interval
comparisons. Each lane uses 31 samples.

## Baseline results

| runs / paragraphs / tables / cells | comparisons | emitted indices | first working-set delta | p50 | p95 | p99 |
|---:|---:|---:|---:|---:|---:|---:|
| 256 each | 196,608 | 512 | 24,576 bytes | 232 us | 275 us | 330 us |
| 1,024 each | 3,145,728 | 2,048 | 65,536 bytes | 3,407 us | 4,266 us | 6,488 us |
| 4,096 each | 50,331,648 | 8,192 | 208,896 bytes | 60,544 us | 85,779 us | 123,556 us |

Raw 256-object microseconds:

```text
262,237,275,330,235,215,242,233,231,230,240,219,243,195,205,232,238,237,225,241,221,223,225,215,195,246,235,220,243,229,228
```

Raw 1,024-object microseconds:

```text
3381,3540,3132,3004,3159,3450,3471,3380,3823,3599,3547,3541,3629,3392,3063,2994,2980,2881,3407,3051,3209,3115,3024,3293,6488,3855,4266,3983,4166,3554,3973
```

Raw 4,096-object microseconds:

```text
55846,60393,123556,57574,60545,60966,58377,57542,85779,62049,66769,65678,61392,54245,56076,55142,70082,55876,57900,58509,57849,53662,52214,60544,65196,73075,66637,65347,63247,64178,58788
```

The object count increases by 16x from 256 to 4,096, while comparisons increase by 256x and p50
increases by about 261x. The emitted result remains only two indices per cell. This isolates the
algorithmic rescan as the bottleneck; vector allocation is not the dominant cause.

## Structural decision

The implementation plan is:

1. Build request-local interval projection indices for the already source-ordered runs, paragraphs,
   and tables. Each tree node stores the maximum range end below it.
2. For a cell query, binary-limit candidates by `start < cell.end`, prune any tree branch whose
   maximum end is `<= cell.start`, and emit surviving original indices in ascending order.
3. Filter table candidates by parent depth and full range containment after the interval query.
4. Keep these search trees temporary to `CompiledRichText` construction. Retain only the existing
   compact cell projection output; do not add a global cache or a second source-range authority.
5. Stop copying/sorting/deduplicating projection indices at the private UI boundary once tests prove
   every producer supplies checked, strictly increasing indices.
6. Add explicit run/paragraph/table/cell/projection admission before large output allocation. The
   faster query is not authorization for unbounded hostile input.

Expected canonical-input construction complexity is `O(R + P + T + sum(log N + K))`, where `K` is
the number of indices actually emitted for each cell. A linear order check preserves this path for
parser-owned runs/paragraphs/tables; defensive direct-constructor input falls back to
`O(N log N)` sorting. Temporary search memory is `O(R + P + T)` and retained output remains
`O(sum K + C)`. Correctness gates must cover disjoint, nested, empty, boundary-touching, and malformed
ranges; performance gates must repeat the same 31-sample lanes and report raw data.

Managed Cargo, real parsed-table layout, allocation/RSS, package power, WGPU/PNG, and a matched Unreal
workload remain required before milestone acceptance.

## Implementation evidence

`CompiledRichText` now builds three request-local balanced interval trees over checked run,
paragraph, and table ranges. Each node stores subtree `max_end`; a cell query prunes branches that
cannot intersect and emits only candidate source indices. Canonical source order is admitted by a
linear check, while out-of-order direct-constructor fixtures retain a defensive sort. Table
candidates are then filtered by depth/containment, preserving the previous nested-table semantics.
The trees are dropped after construction and are not a second retained cache or parser authority.

`UiParsedText::from_projection` now consumes the checked producer's source-order, duplicate-free
indices directly; its private sort/dedup passes were removed. A focused interval test covers
out-of-order construction input, boundary-touching, and non-overlapping ranges, while the existing
nested table projection test continues to verify run/paragraph/nested-table results. Run, paragraph,
table, cell, projection-index, block-depth, and table-depth budgets now fail typed before owner growth.
The static Runtime Text suite passes 38/38, targeted rustfmt and scoped diff checks pass, and no
`cluster_ranges` reference remains. Grapheme normalization moved to the 100-line
`parser/run_alignment.rs` owner, keeping the semantic parser root at 715 lines.

## Isolated post-change method and results

The final interval path was compiled with the same release settings, E-drive compiler temporary
directory, fixtures, timing boundary, and 31-sample lanes. Timing includes construction of all three
request-local trees, all cell queries, candidate ordering, and nested-table filtering. `visited nodes`
counts entered interval nodes, including nodes rejected by `max_end`; it replaces the old exact full-
scan comparison count as the scale indicator. `projection_checksum` equals emitted-index count in
every lane, so the measured output cardinality matches the baseline.

| runs / paragraphs / tables / cells | visited nodes | emitted indices | first working-set delta | p50 | p95 | p99 | old/new p50 |
|---:|---:|---:|---:|---:|---:|---:|---:|
| 256 each | 8,838 | 512 | 69,632 bytes | 147 us | 283 us | 524 us | 1.58x |
| 1,024 each | 44,550 | 2,048 | 53,248 bytes | 789 us | 907 us | 1,124 us | 4.32x |
| 4,096 each | 215,046 | 8,192 | 360,448 bytes | 3,337 us | 4,467 us | 5,611 us | 18.14x |

Raw 256-object microseconds:

```text
197,190,524,148,140,145,283,136,203,147,143,142,151,144,159,147,146,151,148,133,145,152,149,153,136,169,153,142,125,124,137
```

Raw 1,024-object microseconds:

```text
907,815,779,815,868,741,1124,880,695,879,842,790,862,812,813,796,693,789,695,783,774,772,781,693,653,744,767,798,745,794,767
```

Raw 4,096-object microseconds:

```text
3275,3348,3344,3337,3238,3292,3460,2993,2644,5611,4467,3239,3383,3278,3318,3261,3305,3708,3251,3460,3290,3802,3426,3540,3477,3330,3400,3546,3330,3096,3355
```

From 256 to 4,096 objects, old p50 grew 260.97x while the final path grew 22.70x. At 4,096 objects,
entered interval nodes are 234.05x below the old comparison count, and p50/p95 improve 18.14x/19.20x.
This confirms that the isolated quadratic rescan is no longer the dominant construction cost. The
small 256-object p95/p99 lane is noise-sensitive and does not show a tail-latency win; the first-sample
working-set delta is also higher at 4,096 because the request-local trees trade temporary memory for
time. Therefore this result does not close allocation, RSS, power, or product acceptance.

A managed post-change profile must still run the real parsed-table/layout workload and report
allocation count/bytes, peak RSS, package power, and matched Unreal experience. Full Cargo, real WGPU
framebuffer, and a new rendered PNG under `docs/tests/runtime/text` also remain pending.
