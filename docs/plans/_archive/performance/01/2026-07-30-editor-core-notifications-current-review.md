# Editor core notifications current-source performance review

## Status

- Review state: `static_complete / dynamic_pending`.
- Exact production scope: `zircon_editor/src/core/notifications/**`, **9/9** current Rust files, **1,562** lines, **18** tests, **0** ignored.
- Ordered `path + NUL + raw content + NUL` SHA-256: `a0a79cac0bb3727f3a4fbbb43216057639a94eff090824da309b2fe7fee181fc`.
- Formatting: scoped `rustfmt --edition 2021 --check` passed **9/9** files.
- Production path checked: context construction, play pending-decision publish/resolve adapter, retained-host tick projection, and notification-center bridge.
- Supporting tests read: the 3 play pending-decision adapter tests and 2 retained notification bridge tests.
- Code disposition: no Rust source changed. The complete notifications tree was already untracked in the shared worktree and was preserved at the fingerprint above.

This slice stays in `pending.md`. It has no current-source managed Cargo result, retained-tick scale counters, F4 WPR trace, or independent acceptance.

## Current authority verified

- `DecisionNotificationCenter` has hard pending and receipt capacities, defaulting to 128 and 256. Resolved entries are removed as their bounded receipts are evicted, so the center is not an unbounded notification history.
- Notifications and text fields use shared `Arc` owners; options are capped at 16, notification ids at 192 bytes, option ids at 64 bytes, source ids at 128 bytes, and localization keys at 256 bytes.
- Tickets and cursors carry a center-instance id; stale/foreign ticket, cursor gap, idempotent resolution, conflicting resolution, cancellation, capacity, and concurrent linearization are covered.
- The center mutex recovers poison and never invokes foreign callbacks. The play adapter also caps retained selection mappings at 256.
- `receipts_since` clones at most 256 small receipts and currently has no non-test product caller found in this review; it is not the primary F4 bottleneck.

## P0 bottleneck: PERF-MVP-596

The retained host calls `sync_pending_play_decisions` on every active Workbench tick, before dirty recompute:

1. `DecisionNotificationCenter::pending_snapshot` locks the authority, allocates a new vector, and clones every pending snapshot, even when its state generation is unchanged.
2. `PlayPendingEditDecisionAdapter::pending_options` locks a second authority and matches up to 256 retained decisions against up to 128 pending snapshots with nested `any` scans. Even an empty stable center revisits all retained decision rows.
3. Every match deep-clones `PlayPendingDecisionOption` strings (`selection_id`, title, message) into a new vector.
4. The retained bridge then encodes every option into history strings, clones the existing notification array, filters/clones up to 64 old history rows, recomputes unread, and only after that compares the rebuilt result with current state.
5. The no-change result avoids host invalidation, but does not avoid the snapshot, nested matching, option clones, history construction, or lock work that preceded it.

This is distinct from PERF-MVP-269's general NotificationCenter navigation/toast model and PERF-MVP-551's unbounded pending-edit operation queue. PERF-MVP-596 owns the decision authority -> play adapter -> retained-tick projection generation boundary.

## Optimization contract

- Editor04 makes the decision center publish a monotonic pending generation with an immutable shared pending snapshot or bounded delta. Publish/resolve/cancel change it exactly once; stable reads return `NotModified` without cloning rows.
- The play adapter maintains direct ticket/notification and selection indexes. It creates one immutable `PlayPendingDecisionProjection` per changed generation rather than cross-scanning two retained sets or cloning strings per tick.
- EditorUI08 stores the last applied decision generation and only calls the bridge when the Workbench decision surface is visible and that generation changed. Stable active ticks and inactive/hidden views perform zero snapshot/projection/history work.
- The bridge consumes typed shared rows and a generation-owned history/unread aggregate. It does not serialize rows into strings or clone/parse the current notification array merely to discover equality.
- A transition to an empty generation must clear previously projected modal rows; external resolve/cancel cannot leave stale UI. Generation-checked application rejects old results.
- Reuse the existing center/adapter capacities. Do not add an unbounded channel, duplicate notification authority, private worker pool, or reduced global tick rate as a substitute for change-driven projection.

## Dynamic acceptance matrix

| Dimension | Cases | Required evidence |
| --- | --- | --- |
| pending decisions | 0, 1, 128 | pending snapshot builds/cloned rows, center lock wait/hold, generation publishes |
| retained adapter history | 0, 1, 256 | ticket comparisons, nested-scan visits, option String clone bytes, index lookups |
| notification history | 0, 1, 64 | history row encode/parse/clone bytes, unread scans, bridge mutation count |
| cadence | 1 and 1M stable ticks at 30/60/120 Hz | stable snapshot/projection/history builds and allocations all `0`; host invalidation `0` |
| change | publish, same-option resolve, conflicting resolve, cancel, cursor gap, capacity | each accepted change publishes/projects at most once; empty generation clears modal rows; no stale apply |
| contention | 1 and 16 producers/readers | center/adapter lock p50/p95, sequence/order/loss/dup, F4 frame p50/p95/RSS |

Focused managed Cargo must cover the 18 core tests plus play/retained bridge contracts. Product acceptance needs an F4 pending-edit publish/apply/discard trace with stable-frame allocation counters. This is a non-rendering state slice, so RenderDoc is reserved for the downstream notification paint/overdraw gate rather than used here.

## Reference-engine routing

- Unreal Slate accepts notifications by explicit `AddNotification` or a thread-safe pending queue drained on tick. It does not rebuild a separate full decision snapshot and encoded history merely to detect no change. Zircon should keep its stronger typed tickets and hard capacities while adopting event/generation-driven handoff.
- Godot `EditorToaster` updates controls on popup/timeout, caps temporary visible/history rows, and disables internal processing when empty. Zircon should similarly gate work on active state/generation, while preserving modal play-decision receipts rather than copying Godot's toast semantics.
- Neither reference supplies Zircon's stale-ticket, cursor-gap, and idempotent resolve guarantees; those contracts remain mandatory.

## Static gates

- All 9 production files and the relevant product callers/tests were read at current source.
- Existing hard bounds and low-frequency paths were separated from the stable retained-tick hot path.
- Scoped rustfmt, task uniqueness, cross-plan link, source-fingerprint, trailing-whitespace, and `review.md` no-change gates are required for this record.
- No Cargo reservation, commit, RenderDoc capture, or Rust edit belongs to this static-only slice.
