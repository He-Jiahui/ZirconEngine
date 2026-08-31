---
title: Tooling31 streamed asset tree staging
category: zircon_tooling
report_id: Tooling31
date: 2026-08-31
baseline_head: 0aeb32c037cf30028d7a8950ce373ae052c97c38
baseline_epoch: 576
status: release_validation_submitted
session: root-tooling31-streamed-asset-staging-release-r2-20260831
validation_request_id: 87bd1069905d46b885ca61146c20af99
implementation_files:
  - tools/zircon_build_asset_staging.py
tests:
  - tools/tests/test_tooling31_streamed_asset_tree_staging_performance_contract.py
  - tools/tests/test_zircon_build_asset_staging_owner_boundaries.py
---

# Tooling31 streamed asset tree staging

## Problem

Engine asset staging and compiled UI artifact staging both called `sorted(source_root.rglob("*"))`. Each pass materialized the complete asset tree before processing its first entry, then queried every path again to classify directories and files. Peak traversal memory therefore scaled with all entries in the staged tree.

## Change

Both staging paths now share an `os.scandir` depth-first iterator. It sorts only the current directory, reuses `DirEntry` file-type information, and yields entries immediately. Global path order, directory creation order, file validation, dry-run output, non-file exclusion, and the existing rule that directory symlinks are emitted but not recursively followed remain unchanged.

The performance contract rejects any use of `Path.rglob`, builds a nested asset fixture, and verifies that streamed traversal preserves the legacy globally sorted path order and directory/file classification. Its acceptance case invokes the production `_iter_tree_entries` implementation and a test-only reconstruction of the former `sorted(Path.rglob("*"))` traversal over the same physical tree.

## Historical preflight

Acceptance thresholds were at least 90% fewer peak buffered entries, 10% P50 latency reduction, and 5% P95 latency reduction.

| Measurement | Legacy | Optimized | Reduction |
|---|---:|---:|---:|
| Globally materialized entries / largest single-directory fanout | 4,224 | 128 | 96.97% |
| Five-round scan P50 | 707,810,200 ns | 334,711,400 ns | 52.71% |
| Five-round scan P95 | 779,467,500 ns | 374,246,600 ns | 51.99% |

The 2026-08-27 model used 128 top-level asset directories with 32 files each, for 4,224 visible entries including the directories. Before timing, it verified exact path-order and directory/file classification equality. Every timed round repeated that equality check; the stable result checksum was `112000`. The optimized value `128` described the largest list sorted by one call, not the simultaneously live recursive frontier, so it remains historical diagnostic evidence rather than the release memory gate.

## Repeatable acceptance preflight

The 2026-08-31 Windows preflight used the same 128 by 32 tree. The first assertion compared the complete legacy and production entry sequences. Four warm-up pairs preceded 21 alternating legacy/current sample pairs so either implementation led approximately half the measurements. The stable checksum was `75,008`.

The structural memory bound counts the entries retained by the legacy global list and the concurrently live root-plus-leaf directory frontier in the production recursive iterator. For this one-level fixture, the latter is `128 + 32 = 160`; it is deliberately more conservative than the largest-single-directory count.

| Measurement | Legacy | Optimized | Reduction | Gate |
|---|---:|---:|---:|---:|
| Buffered entries / directory frontier | 4,224 | 160 | 96.2121% | >= 90% |
| P50 | 468,981,100 ns | 274,886,400 ns | 41.3865% | >= 10% |
| P95 | 735,228,300 ns | 382,643,900 ns | 47.9558% | >= 5% |

Legacy raw samples in nanoseconds were `[735228300, 542626300, 456482200, 420523100, 448078400, 447833500, 444781300, 401617400, 393109400, 363894600, 377042500, 643068500, 501454400, 477981400, 573681900, 501758400, 842245100, 468981100, 443734500, 570466000, 609729300]`.

Optimized raw samples in nanoseconds were `[256859500, 382544700, 282596200, 232629900, 219824400, 197346400, 200401400, 199380900, 284870600, 180711000, 198385700, 346835600, 274886400, 359510200, 222993200, 311975000, 339432600, 302065100, 403353900, 382643900, 246468900]`.

This preflight is reproducible acceptance evidence, but it is not terminal until the coordinator executes the snapshotted request.

## Validation

- Red phase: the original contract failed because the legacy implementation invoked the guarded `Path.rglob` path.
- Green phase: `python -m unittest tools.tests.test_tooling31_streamed_asset_tree_staging_performance_contract tools.tests.test_zircon_build_asset_staging_owner_boundaries` passed 4/4 locally, including the production-path performance acceptance above and the existing ZUI/resource-copy behavior tests.
- `python -m py_compile` passed for the implementation and both contracts.
- Owned-path `git diff --check` passed.
- Coordinator request `87bd1069905d46b885ca61146c20af99` will run the same four-test batch from an exact four-path snapshot. Expected result: 4 tests passed with the `TOOLING31_STREAMED_ASSET_TREE_STAGING_PERF` marker and all three gates above satisfied.
