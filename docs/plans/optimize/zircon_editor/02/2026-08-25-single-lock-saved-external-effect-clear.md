---
title: Editor02 Single-lock Saved External-effect Clear
category: zircon_editor
report_id: Editor02-single-lock-saved-external-effect-clear-2026-08-25
date: 2026-08-25
session_id: root-editor57-name-line-split-20260825
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor02 Single-lock Saved External-effect Clear

## Scope

This slice removes repeated registry locking and snapshot revision searches when a completed save
clears the external dirty effects captured by one `DirtyDocumentSnapshot`. It preserves document and
revision validation, partial-clear reporting, per-effect generation advancement, journal entries,
residual-effect detection, and public save semantics.

## Implementation

`DirtyRegistry::clear_saved_external_effects` previously checked the document generation under one
lock, then called `clear_external_effect` once per captured effect, and finally acquired another lock
to check residual effects. Every loop iteration also binary-searched the snapshot's effect vector to
recover the revision already stored at the same logical position.

The optimized path holds the registry state once for the complete compare-and-clear transaction. It
streams the parallel effect and revision vectors with `zip`, retains the exact per-effect revision
guard, and advances the document/registry generation once for every effect actually removed. A stale
document generation still rejects the complete clear before any mutation.

## Performance Contract

| Evidence for 1,024 effects | Retired path | Optimized gate |
| --- | ---: | ---: |
| Registry lock acquisitions | 1,026 | 1 |
| Snapshot revision binary searches | 1,024 | 0 |
| Generation and journal updates | 1,024 | 1,024 |
| Alternating release benchmark | 11 samples x 64 clears | optimized P95 <= 75% of retired P95 |

The benchmark emits `EDITOR02_SINGLE_LOCK_SAVED_EFFECT_CLEAR_BENCH_V1` with both P95 timings,
reduction basis points, sample/iteration/effect counts, lock acquisitions, and snapshot revision
search counts.

## Validation

Rust 1.94.1 `rustfmt --check`, scoped diff checks, source-structure gates, and the focused behavior
regression are required before submission. One managed Editor02 Cargo invocation filtered by
`editor02_dirty_registry_` covers this regression and ignored release benchmark together with the
delta-partition optimization. Dynamic P95 evidence, integration SHA, and automatic WeCom
performance delivery remain coordinator-owned and pending.

## Remaining Parent-plan Work

Editor02 still owns the product Save All and close prompt flow, write-time conflict protection,
autosave/recovery lifecycle, document-scoped history cleanup, and large-project qualification. This
micro-optimization does not claim those milestones complete.
