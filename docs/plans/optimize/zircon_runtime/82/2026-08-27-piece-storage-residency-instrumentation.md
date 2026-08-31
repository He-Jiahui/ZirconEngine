# Runtime82 retained document storage residency instrumentation

Date: 2026-08-27

Status: `document_storage_residency_report_implemented /
linear_tail_insert_chunk_growth_measured / append_only_addition_source_implemented /
separator_neutral_local_hard_line_edit_implemented / baseline_and_post_matrix_complete /
admission_thresholds_unfrozen / wall_allocation_rss_matrix_complete /
wpr_cpu_power_blocked_by_windows_policy / matched_unreal_profile_pending`

## Finding

The crate-private retained document is a piece table over one immutable original `Arc<str>` and one
immutable addition `Arc<str>` per non-empty replacement. Adjacent pieces coalesce only when they
reference the same chunk and adjacent source ranges. Consequently, repeated one-character tail
inserts currently retain one addition chunk and one piece per edit. Storage metadata and allocator
overhead can therefore grow with edit count even when the visible document remains small.

The 2026-08-28 managed profile now confirms this as a measured bottleneck: 10,000 one-character tail
inserts have a 1.711 second p50 and request 8.127 GB through the counting allocator while the final
content is 11 KB. The same profile independently measures full-hard-line preparation: one tail insert
into a one-million-character hard line has a 6.7 millisecond p50 and about 2 MB of counted allocation.
See `2026-08-28-retained-document-edit-baseline-and-structural-direction.md`. The selected append-only
addition source and separator-neutral local hard-line edit path are now implemented and rerun through
the same matrix. The 10k tail lane improved from 1.711 seconds to 4.508 milliseconds p50 and from
8.127 GB to 3.643 MB of counted allocation; one million-character middle/tail lanes no longer copy
the complete hard line. The result still does not authorize a guessed compaction interval, rope
migration, or gap-buffer rewrite.

## Implemented observation boundary

`TextDocumentStorageReport` is a content-free lower-bound report owned by `text/document/report.rs`.
It records:

- revision and current byte length;
- original bytes, addition source count, addition bytes, and source capacity;
- piece count and piece-vector capacity bytes;
- stable hard-line count/capacity bytes;
- grapheme-boundary count/capacity bytes;
- whether the current revision has a flattened snapshot and its distinct bytes;
- an estimated retained-byte lower bound.

The lower bound includes the document value, owned source capacity, vector capacities, retained index
capacities, and a distinct current flattened snapshot. It intentionally excludes allocator headers,
`Arc` control blocks, and old snapshots retained by external leases. It is therefore diagnostic
evidence only and must not be reused as a product admission limit.

Regression coverage encodes two facts without publishing source content: the initial source is not
double-counted when its snapshot reuses the original `Arc`, and sequential one-character tail
inserts now retain one addition source and coalesce to one addition piece. Store admission separately
tests addition-source and piece-fragmentation limits. The pre-change eight-chunk/eight-piece fact is
preserved only by the baseline profile rather than as a current complexity target.

## Reference-engine review

Unreal Slate keeps editable text behind `FSlateEditableTextLayout` and applies an edit transaction
before updating the layout (`SlateEditableTextLayout.cpp:2138-2153`). `FTextLayout::InsertAt`
mutates the retained line text, marks the line model dirty, removes its generated line views, and
repairs run ranges (`TextLayout.cpp:2336-2399`). `FTextLayout::RemoveAt` follows the same retained
line/run/view boundary (`TextLayout.cpp:2683-2774`).

This supports Zircon's separation of document mutation, dirty structural models, and regenerated
visual layout. It does not prove that Unreal's string container is the right Zircon storage
algorithm, nor does it establish a compaction threshold.

## Completed managed profile and remaining qualification

The same executable and toolchain ran from an E-drive build directory with capture disabled for
timing samples. Use Latin, CJK, combining-mark, and emoji documents at 1, 100, 1k, and 10k graphemes,
plus a one-million-character base case. For each corpus capture sequential tail inserts, middle
inserts, replacements, deletes, and mixed undo-shaped edits at 1, 100, 1k, and 10k operations.

For cold and warm cases, and separately before/after requesting a flattened snapshot, the matrix
collected 31 samples of edit p50/p95/p99, allocation count/bytes, process RSS, addition source count/bytes,
piece count/capacity bytes, total report lower bound, snapshot flatten time, and source-index rebuild
time. Baseline and post JSONL/CSV evidence is under `docs/tests/runtime/text`. A future power capture
must use the same workload without profiler capture. Compare matched document,
operation stream, compiler mode, platform, font, and capture state with the Unreal reference; do not
compare unmatched editor startup or frame workloads.

The measured dominant costs selected the completed append-only source and local hard-line changes.
Any next structural change still requires the following evidence gate:

- contiguous tail edits dominated by source/piece metadata may justify an append-source change;
- fragmented piece traversal dominated by piece count may justify evidence-based coalescing or
  compaction;
- large middle edits dominated by suffix movement may justify a gap buffer, tree-backed piece table,
  or rope evaluation;
- snapshot/index rebuild dominance must be addressed at the document-range lease and incremental
  index boundary rather than hidden by storage compaction.

Any selected policy must freeze byte/work/memory admission, bound old-revision leases, preserve
document UUID/revision and receipt ranges, report compaction work separately, and rerun the same
matrix to prove that the original bottleneck disappeared without moving it into allocation, RSS,
layout, or power.
