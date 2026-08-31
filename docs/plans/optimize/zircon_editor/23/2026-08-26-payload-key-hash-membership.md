---
title: Editor23 Payload Key Hash Membership
category: zircon_editor
report_id: Editor23-payload-key-hash-membership-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor23 Payload Key Hash Membership

## Scope

This slice reduces duplicate-membership cost while projecting explicit and schema-default binding
payload keys. It preserves explicit-before-default item order, recursive value formatting, preview
projection, and the ordered diagnostics map. It does not change binding schema or preview
authority.

## Change

- Track projected payload keys in `HashSet<String>` instead of `BTreeSet<String>`.
- Keep the existing owned key transfer and duplicate suppression points.
- Preserve explicit payload iteration followed by non-duplicate schema defaults.
- Keep diagnostics grouped and traversed through `BTreeMap` so visible diagnostic order is stable.

## Deterministic Performance Evidence

| Representative 65,536 admissions / 8,192 unique payload keys | Before | After |
|---|---:|---:|
| Membership class | ordered O(log n) | average O(1) hash |
| Owned key moves into the index | 8,192 | 8,192 |
| Published unique keys | 8,192 | 8,192 |
| Projection order | explicit, then schema defaults | unchanged |

The ignored release gate alternates 17 ordered-set and hash-set admission samples and emits
`EDITOR23_PAYLOAD_KEY_HASH_MEMBERSHIP_BENCH_V1`. Acceptance requires hash-membership P95 to be at
most 60% of ordered-membership P95. Exact Windows timings remain pending the coordinator run.

## Acceptance

- `optimization_batch_20260826u_editor23_hash_payload_membership_matches_ordered_membership`
  proves the two indexes admit the same number of unique keys.
- `optimization_batch_20260826u_editor23_payload_projection_uses_hash_membership` requires the
  hash admission set while preserving the ordered diagnostics map.
- `optimization_batch_20260826u_editor23_payload_key_hash_membership_performance_evidence` emits
  both P95 values and enforces the 60% threshold.
- Exact-file Rust 1.94.1 formatting, scoped diff checks, and source contracts must pass before
  managed validation submission.

## Remaining Parent-plan Work

Editor23 still needs typed binding/property schemas, async bounded diagnostics and imports,
generation-qualified previews, lossless V2 editing, atomic save/reimport, and large-binding
document qualification.
