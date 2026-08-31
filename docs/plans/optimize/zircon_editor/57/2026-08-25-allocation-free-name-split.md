---
title: Editor57 Allocation-free Asset Name Split
category: zircon_editor
report_id: Editor57-allocation-free-name-split-2026-08-25
date: 2026-08-25
session_id: root-editor57-name-line-split-20260825
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor57 Allocation-free Asset Name Split

## Scope

This slice removes redundant candidate materialization and scoring allocations from long asset-name
wrapping in the Asset Browser. It preserves measured-width precedence, separator and camel-case
preference, balance and target-distance tie breaks, Unicode byte boundaries, trimming, and the final
two-line strings. It does not change Editor57's asset identity, activation, navigation, mutation,
virtualization, or product-authority contracts.

## Implementation

The retired path collected separator breaks, camel-case breaks, every valid character position, and
the target position into a `Vec`, then sorted and deduplicated it. Every valid character position was
already present in that union, so the special candidates only created duplicates. Scoring then found
the byte index and boundary rank by rescanning the name and allocated two trimmed `String` values for
every candidate.

The optimized path walks `char_indices` once in ascending character order, carries the byte offset and
preferred-boundary bit into the unchanged lexicographic score, and retains only the best score and byte
offset. Candidate scoring trims borrowed slices; only the selected pair becomes owned output strings.
The ascending traversal preserves the retired `sort_unstable` plus `min_by_key` tie behavior.

The regression compares retired and optimized output for separator, camel-case, numeric, wide ASCII,
and multibyte names. A source contract rejects candidate buffering and sorting and requires the
borrowed scoring path.

## Performance Contract

| Evidence per long name with `C` valid split candidates | Retired path | Optimized gate |
| --- | ---: | ---: |
| Candidate buffers | 1 | 0 |
| Candidate sorts | 1 | 0 |
| Temporary scoring `String` allocations | `2C` | 0 |
| Per-candidate byte/boundary prefix scans | `2C` | 0 |
| Alternating release benchmark | 11 samples x 32 x 64 names | optimized P95 <= 90% of retired P95 |

The benchmark emits `EDITOR57_ALLOCATION_FREE_NAME_SPLIT_BENCH_V1` with both P95 timings, reduction
basis points, sample/iteration/name/candidate counts, candidate buffers, and scoring allocations.

## Validation

The TDD source probe first observed the legacy candidate buffer and sort with no borrowed scoring
path, then observed zero candidate buffers, zero candidate sorts, and borrowed scoring after the
implementation. Rust 1.94.1 formatting and scoped diff checks passed before submission. One managed
Editor batch covers retired/optimized behavior equivalence, the source contract, and the ignored
release benchmark. Dynamic P95 evidence, integration SHA, automatic commit, and automatic WeCom
performance delivery remain coordinator-owned and pending.
