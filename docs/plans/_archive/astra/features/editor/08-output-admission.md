---
status: in_progress
plan_sources:
  - docs/plans/optimize/zircon_editor/269-editor-build-export-preset-pipeline-cook-pack-platform-bundle-publishing-resume-determinism-current-working-tree-review.md
---

# Output admission recovery

## Current source and repair

EXPORT-GATE-29 is reproduced by source inspection: the per-stage counter limits
the entire job lifetime to 16 live output events even when the receiver drains
every event immediately. Only pending work should consume queue capacity.

Use the existing crossbeam-channel dependency for its bounded queue occupancy.
The single wizard producer admits output while fewer than 128 events are queued;
the existing 192-slot channel reserves room for Created, Started, two control
events per unique planned stage, and one terminal event (at most 19 controls).
All sends use try_send. Consumers can only reduce occupancy between admission
and send. Output coalescing totals remain monotonic and accompany terminal truth.
This fixes the current in-memory terminal delivery bound of EXPORT-GATE-30;
durable terminal replay remains outside this repair.

The controller receiver and view-model drain contract now use crossbeam-channel.
Event transport stays in a private controller child; process completion channels
retain their existing owners. The PlayOutputPump's release-on-drain behavior is
the local behavioral reference; no new scheduler or queue implementation is added.

## Plan completion list

| Batch | Work | Status | Validation evidence |
|---|---|---|---|
| M22 | Restore live output admission after actual queue consumption; reserve terminal capacity | implemented_pending_validation | Five regressions including the real pipeline with absent/disconnected consumers; combined batch pending |

Replace the obsolete fixed lifetime-counter source checks and synthetic counter
benchmark with regressions for all stages, 0/1/1k/10k source events, partial drain,
slow/absent consumers, disconnected receivers, and ordered terminal delivery.
No elapsed-time improvement is claimed for restoring previously discarded work.
M19/M21 provide separate semantically equivalent release performance comparisons.
