# Editor notification generation and projection current-architecture review

## Status

- Result: `static_complete / dynamic_pending`.
- Review date: 2026-08-15.
- MVP priority: P0 for stable retained-host tick cost and Play Apply/Discard latency; P1 for activity
  history encoding, toast expiry and job-progress projection.
- Owner: Editor17 owns the unified notification authority and presentation contract; Editor04 owns
  Play decision indexing; Editor14 owns job-progress generations; EditorUI08 owns demand and retained
  application. Existing `PERF-MVP-269` owns the string-array component codec after publication.
- Accounting: keep this module in `pending.md`. Do not add it to `review.md` before current-source
  managed Cargo, scale/allocation/lock counters and F4 WPR acceptance pass.
- Code disposition: no Rust source changed. Fifteen files in the exact production/test tree have
  pre-existing modifications; all source bytes and owners were preserved.

## Exact scope

| scope | files | physical lines | tests | ignored | ordered path-and-raw-content SHA256 |
|---|---:|---:|---:|---:|---|
| `zircon_editor/src/core/notifications/**` | 25/25 | 3,469 | 38 | 0 | `f1a239d85ae1146c13e05e05055d16aaf72f1bbf83c049710787d25ba8512d13` |

The fingerprint is SHA256 over normalized sorted path, NUL, raw file bytes, NUL. Every Rust file in
the folder was read in full. The context/job observer, Play pending-decision adapter, retained-host
tick, activity localization and workbench notification bridge were traced as supporting callers.

The July report's 9 files/1,562 lines/18 tests is obsolete. The module now includes typed identity,
Decision, Toast, Progress, presentation and context-owned service subdomains. The old claim that an
empty decision option set returns before the bridge is also obsolete: current
`sync_activity_notifications` always supplies the empty set, so stale modal rows can be cleared.

## Architecture verdict

The three core authorities have useful hard bounds and should remain authoritative:

- Decision: 128 pending, 256 receipts, 16 options, 8 numeric message facts; typed center/ticket/cursor
  identity and bounded ids/keys. Notification payloads are shared through `Arc`.
- Toast: 128 live rows by default, explicit expiry and one-hour maximum lifetime.
- Progress: 64 JobId-bound rows, automatic bindings replaceable by a source-specific producer.

Publish/resolve/cancel concurrency is linearized, receipts detect cursor gaps, progress observer
recovery handles lifecycle races, and no foreign callback runs inside the notification center locks.
These are stronger contracts than a generic UI toast queue and must be preserved.

The current P0 problem is downstream polling architecture. On every active main-Workbench retained
tick, `sync_activity_notifications` independently snapshots and projects all three authorities before
asking the bridge whether anything changed
(`ui/retained_host/app/host_lifecycle/tick.rs:8-55`;
`ui/retained_host/app/workbench_notifications.rs:36-76`). The no-change comparison suppresses final
host invalidation, but it occurs after locks, snapshots, localization, owned DTO construction, string
formatting and history parsing. The system therefore has bounded memory but not change-proportional
CPU/allocation cost.

For pending decision count `D <= 128`, retained adapter decisions `A <= 256`, toasts `T <= 128`,
progress rows `P <= 64` and bridge history `H <= 64`, one current sync is approximately
`O(D + A*D + T + P log P + H*W)`, where `W` is encoded row width. Stable active ticks repeat this
even when all generations and UI rows are unchanged.

## Current-source corrections

1. `DecisionNotificationCenter` maintains an O(1) pending count and bounded receipt eviction. It is
   not an unbounded history (`decision/center.rs:65-80,116-150,190-264`).
2. Decision snapshots clone tickets/receipts but share notification bodies; the old report's “deep
   clone every notification body” wording is too broad. The Play adapter still constructs new owned
   UI strings after localization.
3. Empty decision state reaches the bridge and can clear old modal rows. Preserve this behavior when
   adding a generation fast path.
4. Toast and Progress are now production-connected. The retained host snapshots/localizes both every
   active tick (`workbench_notifications.rs:47-54`); they are no longer merely future core contracts.
5. Progress publication is event-driven through `EditorJobProgressObserver`, but UI consumption still
   polls the bound center. This is separate from `PERF-MVP-017`, which owns the one-row status-bar
   `primary_snapshot` path.
6. Toast queue semantic comparison deliberately ignores remaining-duration fields, so time-only
   changes already avoid UI mutation. The engine still scans/clones/localizes/formats the rows to
   discover that fact.

## Remaining bottlenecks

### P0: stable tick rebuilds three owned projections

- `pending_play_decision_options` reconciles the prompt, calls `pending_snapshot`, clones all retained
  adapter decisions, performs nested ticket matching, localizes each match and creates owned title,
  message and selection strings (`ui/host/play_pending_decision/adapter.rs:223-263`).
- Toast snapshot takes its mutex, runs `retain` across the live map and clones every live row. The
  activity projection translates title/message and owns new strings each tick
  (`toast/center.rs:100-107`; `ui/activity/view.rs:69-104`).
- Progress snapshot takes the center mutex, constructs a captured `BTreeMap` and JobId vector, asks
  the job source for snapshots, takes the center mutex again, builds another JobId map, prunes and
  clones projected rows (`progress/center.rs:87-181`). This repeats at most 64 rows every tick even
  when job generations do not change.
- `present_decision` performs sequential full-string replacement for up to eight message arguments;
  every presentation clones identities/options and translates text (`presentation.rs:162-201`). That
  is acceptable at a changed locale/decision generation, not as stable frame work.

### P0: duplicate same-frame synchronization

- Tick always calls `sync_activity_notifications`. `publish_activity_toasts` also synchronizes
  immediately, then `apply_dispatch_side_effects` synchronizes again after processing every effect
  (`workbench_notifications.rs:17-34`; `host_lifecycle/dispatch_effects/side_effects.rs:11-60`). A
  toast-producing dispatch can therefore rebuild the complete projection multiple times before the
  next retained frame.
- The active-document gate prevents work outside the main Workbench template, but it is not a
  generation gate. Existing `PERF-MVP-105` owns its full-chrome lookup cost.

### P1: string-array history is the equality and interaction model

- The bridge formats up to 64 decision/progress/toast rows into pipe-delimited owned strings, parses
  them again for unread/kind/id, clones the current string array, then compares arrays
  (`callback_dispatch/template_bridge/workbench/notifications.rs:230-293,402-517`).
- `pipe_value` performs character mapping, creates a string, collects whitespace slices into a new
  vector and joins them into another string. Bounds cap the final row count, not source text bytes or
  transient allocation.
- Toast queue construction also formats up to 64 strings and reparses old/new rows to ignore volatile
  duration fields before returning no-change (`notifications.rs:97-143,438-479`).
- Keep this codec only at the runtime-UI compatibility boundary. It must not remain the authority or
  the stable-frame change detector; typed generation-owned rows belong upstream.

### P1: expiry and receipt polling need explicit wake contracts

- Toast expiry currently requires `snapshot_at(now)` to scan the complete bounded map. Publish should
  expose the earliest expiry so the host wakes/reprojects only on a notification change or deadline.
- `receipts_since` linearly filters up to 256 receipts and clones the suffix. This is bounded and
  resolution-driven, not the stable-frame root cause. Add cursor-first indexing only if measurements
  show it matters during recovery storms.
- `cancel` can take the decision mutex twice before delegating to `resolve`; preserve correctness and
  only converge the locking if contention counters justify it.

## Reference-engine evidence

- Unreal requires direct notification creation on the game thread and provides a thread-safe pending
  queue for other threads (`dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Notifications/NotificationManager.cpp:244-290`).
  Its manager tick drains pending items and maintains live notification windows
  (`NotificationManager.cpp:342-380`); it does not reconstruct a second encoded history from every
  producer authority to detect equality.
- Unreal exposes direct start/update/cancel progress operations to a progress handler
  (`NotificationManager.cpp:292-324`). Zircon should keep its stronger JobId/generation authority but
  publish deltas from that authority instead of resnapshotting the whole bound set.
- Unreal async task notification updates capture changed text for a one-frame game-thread update, and
  its state tick consumes an optional pending state and mutates the widget only when the state differs
  (`dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Notifications/SlateAsyncTaskNotificationImpl.cpp:262-302,314-365`).
  The transferable design is change-driven UI mutation with explicit thread affinity, not Unreal's
  exact widget/window implementation.

## Optimization plan

### Milestone 1: measure each stage before contract changes

- Add generations and counters for Decision pending, Toast live set/next expiry, Progress binding/job
  state and locale. Measure lock wait/hold, rows visited/cloned/localized, translated/encoded bytes,
  placeholder passes, bridge string builds/parses, sync calls/frame and final invalidations.
- Measure empty, stable and 1% changed sets at Decision 0/1/128, adapter 0/1/256, Toast 0/1/128,
  Progress 0/1/64 and row widths 64 B/2 KiB/256 KiB. Include one dispatch that publishes a toast and
  one Play Apply/Discard transition.
- Capture F4 with WPR/xperf after the approved-root build helper is repaired. Attribute activity sync
  separately from status-bar progress (`PERF-MVP-017`) and runtime-UI row parsing (`PERF-MVP-269`).

### Milestone 2: one unified immutable notification projection generation

- Editor17 publishes one shared `ActivityNotificationProjection` keyed by Decision, Toast, Progress
  and locale generations, plus `next_toast_expiry`. It contains typed shared rows, unread/overflow
  aggregates and the selected decision identity. No second mutable notification authority is added.
- Decision publish/resolve/cancel changes pending generation exactly once. Editor04 replaces nested
  ticket matching with direct ticket/notification/selection indexes and builds its typed rows once per
  changed decision or locale generation.
- Editor14 exposes one Progress generation and shared active rows from the job source. The notification
  binding layer applies its 64-row policy on generation change, not by rebuilding two maps per tick.
- Toast publish/expiry changes generation exactly once. Stable time before `next_toast_expiry` returns
  `NotModified` without locking/scanning all rows; expiry wakes once and publishes the new generation.

### Milestone 3: demand-driven retained application

- EditorUI08 stores the last applied unified generation. Tick reads a compact revision token and
  applies at most once when the notification surface is active and changed. Dispatch side effects
  mark dirty; they do not synchronously rerun the full projection.
- Empty generations are first-class and clear modal/toast/progress rows. Generation-checked application
  rejects stale localized results after locale, resolve, cancel or expiry transitions.
- Publish typed `UiValue` rows once at the runtime-UI boundary. If the current string-array ABI must
  remain temporarily, encode once per changed generation and share the result; do not parse it back
  for unread/id/kind aggregates already known upstream.

### Milestone 4: close product and power evidence

- Repeat identical F4 WPR scenarios before/after at 30/60/120 Hz and with 1/16 producers. Verify
  stable snapshot/localization/format/parse work is zero and same-frame accepted changes project once.
- Use RenderDoc only for downstream notification paint/overdraw and draw/resource parity after a
  current editor launches. CPU, lock, allocation and package-power claims come from ETW/WPR on the
  same machine; no Unreal numeric budget is invented.

## Quantified acceptance

| matrix | required measurements | acceptance |
|---|---|---|
| stable: D/A/T/P = 0 or max; 1/1M ticks at 30/60/120 Hz | authority locks, snapshot builds, rows cloned/localized, translated/encoded bytes, string parses, bridge calls, invalidations, CPU/RSS/power | after initial apply and before next expiry, all listed projection work=0; empty generation clears old UI exactly once |
| change: publish/resolve/cancel/toast expiry/job progress/locale; 1/16 producers | source generations, unified builds/applies, lock p50/p95, stale rejects, input-to-present p50/p95/p99 | each accepted source change increments once; each resulting unified generation builds/applies at most once; same-frame duplicate sync=0; order and typed identities preserved |
| scale: D 1/128, A 1/256, T 1/128, P 1/64; 64 B/2 KiB/256 KiB rows | ticket comparisons/index probes, map builds, clone/localization/format bytes, peak RSS | decision matching near O(D+A), stable O(1); Progress does not build duplicate maps per tick; transient bytes occur only on changed generation and obey a byte budget |
| product F4 before/after | WPR CPU stacks, contention, allocations/RSS, context switches/package power; optional RenderDoc notification draws/overdraw | notification stages are separately attributable and reduced; Play Apply/Discard, cursor gap, toast expiry, progress retire/refill, locale switch and runtime-UI interaction all pass |

## Static gates executed

- Read all 25 files twice at the recorded fingerprint and traced the current production callers and
  Unreal primary sources above.
- `rustfmt --edition 2021 --check` is green for all 25 files. No foreign source was formatted.
- Managed Cargo did not run because `tools/build-editor.ps1:130` still rejects valid D:/E:/F: roots
  through its single-quoted doubled-separator bug. See
  `failure-2026-08-15-build-editor-approved-root-separator.md`.
- WPR/xperf and RenderDoc are installed, but there is no launchable current-source editor binary.
  No latency, power, rendering or algorithmic improvement is claimed; dynamic evidence remains
  mandatory.
