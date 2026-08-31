---
title: Runtime52 Moved Merge Preview Report
category: zircon_runtime
report_id: Runtime52-moved-merge-preview-report-2026-08-28
date: 2026-08-28
session_id: root-runtime52-two-task-performance-batch-closeout-20260830
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime52 Moved Merge Preview Report

## Scope

This slice removes the final report clone from merge preview planning. It advances the allocation
and preview-cost concerns behind DSA-P2-008 and contributes a focused 8K-slot data point toward
DSA-G36. It does not close the archive durability, product-service integration, or full Runtime52
parent-plan gates.

## Implementation

`RuntimeSessionArchiveMergePlan` now exposes `into_report(self)`, which moves its owned report out
of the consumed plan. `preview_merge_archive` consumes the prepared plan through that accessor
instead of borrowing the report and cloning all three slot-ID vectors and every contained string.
The existing borrowed `report()` accessor and commit path remain unchanged.

Three Rust regressions cover buffer identity across the consuming accessor, continued borrowed
report access, and an empty report. The buffer-identity assertions directly distinguish a move
from a deep clone.

## Performance Evidence

The release model prepares one report containing 8,192 formatted slot IDs: 4,096 inserted, 2,048
replaced, and 2,048 skipped. It uses 31 alternating legacy/optimized sample pairs after five
warmups, excludes preparation cloning from the timed region, and verifies identical checksums.
The acceptance result uses the final conservative rerun.

| Metric | Before | After | Change |
| --- | ---: | ---: | ---: |
| Report clone allocations | 8,195 | 0 | -100% |
| Report clone requested bytes | 303,104 | 0 | -100% |
| P50 per prepared-plan preview | 1,247,800 ns | 1,500 ns | -99.880% |
| P95 per prepared-plan preview | 2,338,200 ns | 2,900 ns | -99.876% |

Both implementations produced checksum `11003984662318678016`. A preceding independent run
measured P50 `1,179,400 -> 1,800 ns` (-99.847%) and P95 `3,140,700 -> 2,600 ns`
(-99.917%). Requested bytes count the three cloned Vec buffers and cloned string payloads; normal
deallocation behavior is outside that allocation count.

## Validation

- Source contract: 3/3 passed after a confirmed 0/3 initial state.
- Exact Rust formatting and Python contract compilation: passed.
- Scoped `git diff --check`: passed for the exact seven-path batch scope.
- This task is queued together with incremental selected retention in one Runtime52 two-task
  asynchronous validation batch. The batch runs six source contracts, six `runtime52_batch_` Rust
  regressions, and two release models for two exact performance rows; no local Cargo lane was
  launched.
- Commit and WeCom publication remain pending independent review and managed validation.

## Remaining Parent-plan Work

Runtime52 still requires product-owned retention classes, bytes/age/pressure/lease/tombstone
policy, durable revision/CAS, async operation integration, query budgets, and 1/1K/100K-slot
p99/RSS/I/O evidence. This local allocation removal does not substitute for those gates.
