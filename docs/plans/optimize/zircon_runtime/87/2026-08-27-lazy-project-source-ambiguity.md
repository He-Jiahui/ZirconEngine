---
title: Runtime87 Lazy Project Source Ambiguity
category: zircon_runtime
report_id: Runtime87-lazy-project-source-ambiguity-2026-08-27
date: 2026-08-27
session_id: root-runtime87-lazy-project-source-ambiguity-20260827
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime87 Lazy Project Source Ambiguity

## Scope

`source_operation_path_for_project_uri` previously joined every project root, collected every
existing candidate into a temporary `Vec<PathBuf>`, and then cloned the sole path on the common
unique-match branch. Missing, unique, and ambiguous resolutions all paid for the collection.

The implementation now consumes the filtered root iterator lazily. A missing URI returns after the
first failed `next`, a unique URI moves the first `PathBuf` directly into the result after probing
for a second match, and only an actual ambiguity allocates the complete ordered candidate vector.
Project-root order, path joining and existence checks, missing error mapping, and the full ordered
ambiguity list are unchanged.

This is a focused allocation improvement adjacent to AREF87-P1-014 and G15. It does not claim to
add the still-required bounded structured root/source identities to ambiguous candidates.

## Behavior Evidence

- `multiple_project_roots_scan_distinct_res_uris_and_reject_collisions` covers unique resolution
  from the second registered root, a missing URI, and a two-root collision.
- `test_runtime87_lazy_project_source_ambiguity_performance_contract.py` rejects eager candidate
  collection, requires the first/second-match fast path, and freezes missing and ambiguous error
  construction plus ordered first/second/remaining candidate assembly.
- The deterministic model asserts exact parity for missing, unique, and ambiguous outcomes before
  collecting timing or allocation evidence.

## Deterministic Performance Model

The optimized release model uses two registered project roots and one existing asset in the second
root. Each sample resolves the same URI 1,024 times after five warmups; 31 legacy/streaming sample
pairs alternate execution order. This represents the common multi-root unique-source path while
retaining separate parity cases for missing and ambiguous results.

| Metric | Eager collection | Lazy ambiguity | Reduction |
|---|---:|---:|---:|
| allocations per resolution | 6 | 4 | 33.333% |
| allocated bytes per resolution | 498 | 282 | 43.373% |

| Run | Legacy P50 ns | Lazy P50 ns | Reduction | Legacy P95 ns | Lazy P95 ns | Reduction |
|---:|---:|---:|---:|---:|---:|---:|
| 1 | 725,000 | 580,000 | 20.000% | 936,000 | 715,300 | 23.580% |
| 2 | 756,500 | 590,200 | 21.980% | 5,527,600 | 916,400 | 83.420% |
| 3 | 761,000 | 597,400 | 21.500% | 965,000 | 22,444,800 | -2,225.890% |
| 4 | 746,900 | 594,000 | 20.470% | 14,465,700 | 6,220,500 | 57.000% |

The four-run worst-case P50 reduction is 20.000%. P95 remains diagnostic because one Windows
scheduling outlier affected the lazy sample. The managed gate requires exact allocation counts
`6 -> 4`, exact allocated bytes `498 -> 282`, at least 15% lower P50, exact result checksum `1496`,
and nonzero timing checksum `94978048`.

This isolated model measures root candidate construction and selection only. It is not an
end-to-end claim about filesystem metadata latency, asset import, registry publication, frame time,
power, or external-engine performance.

## Validation

Passed locally without Cargo:

- 3/3 Python source/performance contracts after an observed RED on the eager implementation;
- Python bytecode compilation, Rust formatting, and scoped diff checks;
- four independent optimized release-model runs with exact result parity and all managed gates met.

Managed validation must run the focused project-root Rust tests, all three Python contracts,
formatting, scoped diff, and a fresh optimized release model in one coordinator ticket. Cargo
validation is not claimed until that asynchronous ticket reaches a passing terminal state.

## Remaining Parent-Plan Work

Runtime87 still owns stable GUID resolution, structured ambiguity identities, snapshot/budget
receipts, repair authorization and persistence, redirect/tombstone behavior, migration, Editor
integration, and product-scale evidence. This slice only removes unnecessary materialization and a
clone from the existing filesystem source lookup.
