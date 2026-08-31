---
title: Runtime52 Incremental Selected Retention Report
category: zircon_runtime
report_id: Runtime52-incremental-selected-retention-2026-08-28
date: 2026-08-28
session_id: root-runtime52-two-task-performance-batch-closeout-20260830
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime52 Incremental Selected Retention Report

## Scope

This slice improves selected-slot protection after a tag-scoped retention preview. It advances the
allocation and stable-ordering concerns behind DSA-P2-008 and contributes a focused 8K-slot data
point toward DSA-G36. It does not close the retention policy gaps in DSA-P1-060, archive durability,
product service integration, or the Runtime52 parent plan.

## Implementation

Retention planning already partitions canonical BTree-ordered slot IDs into sorted, disjoint
`retained_slot_ids` and `removed_slot_ids`. The selected-protection owner now updates that report in
place: it binary-searches and removes the selected ID from the removed partition, then
binary-searches and inserts one owned ID into the retained partition.

The previous implementation rescanned every archive slot, linearly scanned all remaining removed
IDs for every slot, cloned every retained result string, and sorted the rebuilt vector. The new
helper never enumerates the archive. A selected ID that is not in the removed partition remains a
no-op. Three Rust regressions cover canonical middle insertion, the no-op branch, and defensive
partition uniqueness.

## Performance Evidence

The release model uses 8,192 canonical slot IDs with 4,096 initially retained and 4,096 removed,
then protects one removed middle ID. It uses 31 alternating legacy/optimized sample pairs after
five warmups and verifies byte-for-byte equivalent ordered partitions. The acceptance result uses
the final conservative rerun with the no-op branch present.

| Metric | Before | After | Change |
| --- | ---: | ---: | ---: |
| Membership comparison upper bound | 33,546,240 | 28 | effectively -100% |
| Result string clones | 4,097 | 1 | -99.976% |
| P50 per protected report | 254,330,300 ns | 75,200 ns | -99.970% |
| P95 per protected report | 343,541,600 ns | 177,100 ns | -99.948% |

Both implementations retained checksum `1296207490175078401`. A preceding independent run
measured P50 `280,710,800 -> 64,400 ns` (-99.977%) and P95
`399,631,800 -> 81,600 ns` (-99.980%). The comparison figure is a conservative algorithmic upper
bound; vector element moves remain linear in the insertion/removal distance but do not clone slot
strings.

## Validation

- Source contract: 3/3 passed after a confirmed 0/3 initial state.
- Exact Rust formatting and Python contract compilation: passed.
- Existing Runtime52 integration coverage already fixes selected middle-slot ordering, tag-scope
  exclusion, and preview/commit equality.
- Scoped `git diff --check`: passed for the exact seven-path batch scope.
- This task is queued together with moved merge-preview report ownership in one Runtime52 two-task
  asynchronous validation batch. The batch runs six source contracts, six `runtime52_batch_` Rust
  regressions, and two release models for two exact performance rows; no local Cargo lane was
  launched.
- Commit and WeCom publication remain pending independent review and managed validation.

## Remaining Parent-plan Work

Runtime52 still requires product-owned retention classes, bytes/age/pressure/lease/tombstone policy,
durable revision/CAS, async operation integration, query budgets, and 1/1K/100K slot p99/RSS/I/O
evidence. This local report update does not substitute for those gates.
