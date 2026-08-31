---
title: Editor23 Palette Slot Hash Admission
category: zircon_editor
report_id: Editor23-palette-slot-hash-admission-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor23 Palette Slot Hash Admission

## Scope

This slice removes repeated linear duplicate scans from UI asset palette-drop slot ordering. Slot
semantics still choose the group priority, and the published vector still preserves source order
inside each group and for unclassified slots.

## Change

- Add one capacity-sized borrowed `HashSet<&str>` for slot admission.
- Retain the existing `Vec<String>` as the sole publication-order owner.
- Preallocate the published vector to the available slot count and preserve first-seen duplicate
  behavior.
- Keep the near-1,000-line production owner focused by placing behavior, source-contract, and
  performance tests in a child module.

## Deterministic Performance Evidence

| Representative 2,048 slots / 3 semantic groups | Before | After |
|---|---:|---:|
| Duplicate membership checks | repeated vector scan | hash admission |
| Representative membership probes | 8,390,656 string comparisons | 10,240 average O(1) probes |
| Published slot identities | owned strings | unchanged |
| Published order | semantic group, then source order | unchanged |

The ignored release gate runs 17 alternating samples and emits
`EDITOR23_PALETTE_SLOT_HASH_ADMISSION_BENCH_V1`. Acceptance requires hash admission P95 to be at
most 60% of linear admission P95. Exact Windows timings remain coordinator-owned.

## Acceptance

- Semantic group priority, source order, unclassified fallback, and first-seen duplicate behavior
  remain unchanged.
- A bounded source contract requires hash admission plus vector publication and rejects linear
  `ordered.iter().any` membership.
- The release benchmark compares identical output vectors and enforces the 60% P95 threshold.

## Remaining Parent-plan Work

Editor23 still needs schema-qualified drop receipts, lossless V2 editing, atomic revision-checked
saves, schema-driven authoring, and full 1k/10k/100k qualification. This slice only improves slot
identity admission during palette drop resolution.
