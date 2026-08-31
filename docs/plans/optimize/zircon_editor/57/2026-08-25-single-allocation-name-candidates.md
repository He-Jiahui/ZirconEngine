---
title: Editor57 Single-allocation Name Candidates
category: zircon_editor
report_id: Editor57-single-allocation-name-candidates-2026-08-25
date: 2026-08-25
session_id: root-editor57-name-line-split-20260825
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor57 Single-allocation Name Candidates

## Scope

This slice removes transient string buffers from Asset Browser file-name compaction candidates. It
preserves whitespace handling, extension matching, Unicode character boundaries, preferred tail
length, binary prefix search, measured-width acceptance, fallback output, and all existing compacted
labels. It does not change Editor57 asset identity, navigation, selection, mutation, or retained UI
contracts.

## Implementation

The retired candidate builders collected the prefix and tail into separate `String` values and then
formatted both into a third output `String`. The optimized builders calculate the exact UTF-8 byte
length of the borrowed character slices, allocate the final output once, and extend it directly with
the prefix, ellipsis, tail, dot, and suffix. ASCII and multibyte names follow the same character-count
selection as before.

Both compaction paths also stop eagerly constructing their minimum-prefix fallback before the binary
search. The fallback is now created only when no measured candidate fits, avoiding one otherwise
discarded candidate on successful searches.

The regression compares retired and optimized candidates for mixed multibyte and ASCII input. A
source contract requires direct writes into a pre-sized output, rejects the two temporary collectors,
and rejects eager fallback materialization.

## Performance Contract

| Evidence per candidate | Retired path | Optimized gate |
| --- | ---: | ---: |
| Owned string buffers | 3 | 1 |
| Eager fallback candidates on a successful search | 1 | 0 |
| Alternating release benchmark | 11 samples x 32 x 48 names x 4 candidates | optimized P95 <= 85% of retired P95 |

The benchmark emits `EDITOR57_SINGLE_ALLOCATION_NAME_CANDIDATES_BENCH_V1` with both P95 timings,
reduction basis points, sample/iteration/name/candidate counts, owned buffers per candidate, and eager
fallback counts.

## Validation

The TDD source probe first observed no pre-sized output, both temporary collectors, and eager fallback
materialization, then observed all six optimized source contracts. Rust 1.94.1 formatting and scoped
diff checks passed before submission. One managed Editor batch covers this slice together with the
pending source-tree capacity slice, including behavior equivalence, both source contracts, and both
ignored release benchmarks. Dynamic P95 evidence, integration SHA, automatic commit, and automatic
WeCom performance delivery remain coordinator-owned and pending.
