---
title: Editor23 Source Outline Hash Membership
category: zircon_editor
report_id: Editor23-source-outline-hash-membership-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor23 Source Outline Hash Membership

## Scope

This slice removes redundant ordered-set construction from UI asset source-outline indexing. Node
IDs and first-seen tree IDs are membership-only sets. Final outline entries continue to sort by
source line and block label, while line-segment priority retains its existing ordered structures.

## Change

- Pass the document node iterator directly instead of materializing an ordered set at the caller.
- Build one borrowed `HashSet<&str>` for node membership and one borrowed hash set for first-seen
  tree nodes.
- Preserve direct-block precedence, source-line output order, line ranges, and ordered segment
  priority.

## Deterministic Performance Evidence

| Representative 65,536 admissions / 8,192 unique node IDs | Before | After |
|---|---:|---:|
| Membership structures built | 3 ordered trees | 2 hash sets |
| Membership class | O(log n) | average O(1) |
| Borrowed node key allocations | 0 | 0 |
| Published entry order | source line, then label | unchanged |

The ignored release gate runs 17 alternating samples and emits
`EDITOR23_SOURCE_OUTLINE_HASH_MEMBERSHIP_BENCH_V1`. Acceptance requires hash membership P95 to be
at most 60% of ordered membership P95. Exact Windows timings remain coordinator-owned.

## Acceptance

- A real outline build preserves source-line ordering despite duplicate, unordered input IDs.
- A bounded source contract rejects the redundant caller tree and requires both hash memberships.
- The release benchmark checks equivalent unique counts and enforces the 60% P95 threshold.

## Remaining Parent-plan Work

Editor23 still needs lossless V2 editing, atomic revision-checked saves, cross-asset transactions,
real creation/catalog wiring, schema-driven authoring, and 1k/10k/100k node qualification. This
slice only improves source-outline identity admission.
