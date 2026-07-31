# Editor core editor message current-source review

## Status

- Result: `static_complete / dynamic_pending`.
- Review date: 2026-07-30.
- Owners: Editor02 owns bus/inbox routing and bounded drain; Editor12 and Plugins01 own the plugin lifecycle bridge and callback policy; Runtime11 owns an explicitly declared bounded worker ticket only when callback affinity permits it.
- Accounting: keep the module in `pending.md`; do not add it to `review.md` before current-source managed Cargo, contention/backpressure counters and an F4 retained-host trace are GREEN.
- Code disposition: no Rust source was changed. Existing modified and untracked source was preserved.

## Exact scope

| module | files | physical lines | inline tests | ordered path-and-raw-content SHA256 |
|---|---:|---:|---:|---|
| `zircon_editor/src/core/editor_message` | 29/29 | 2,135 | 2 | `cdaf58c7abde11ad0d5cd56c60d66b00a0615aecb2eb720fd1d9fd4f6ee78eff` |

All production files under `bus.rs`, `ids/**`, `inbox.rs`, `message/**`, `refresh_report.rs`, `retention.rs`, `shared.rs`, `subscriber.rs`, `topic.rs`, `topics.rs` and `view_dirty.rs` were read in full. The row supersedes the earlier 2,126-line/1-test fingerprint after `topic.rs` gained the canonical document-topic regression; that file and its document publisher were reread incrementally. The related external suite `zircon_editor/src/tests/editor_message/**` contains 25 tests across 10/10 files, including one ignored managed backpressure benchmark; all ten files were also read in full.

The production chain was followed through transaction, job, document, tool and scene-inspection publishers and through `EditorPluginLifecycleMessageBridge`, retained-host tick and active-plugin callback dispatch. Supporting consumers are evidence for this slice, not newly accepted folder accounting.

## Current-source corrections

1. The old mixed-`Vec` latest-key scan is no longer current. Inboxes have independent lossless, bounded and latest lanes, O(1) lane-depth counters, a `latest_by_key` map plus stable order, shared `Arc` payloads, a 2 MiB delivery ceiling and a 16 MiB retained-byte ceiling. Default lane capacities are 4,096 lossless, 256 bounded and 256 latest entries.
2. Lossless fanout preflights all targets before enqueue, so a rejected target cannot leave a partially delivered lossless publish. Requests run their handler outside the bus lock and revalidate the target before committing the response.
3. These changes remove earlier unbounded/default-deep-fanout claims, but one global `Arc<Mutex<EditorMessageBus>>` still owns subscriptions, every inbox, dirty state, target collection, delivery sizing and fanout. A wide publish remains `O(subscribers)` under the common lock and unrelated subscribers contend with a stalled owner.

## Current bottlenecks

- Lossless publish constructs and sizes a temporary delivery during preflight, then constructs and sizes the real delivery again. Request additionally creates `EditorMessageRequest` from `message.clone()`, so a `Custom(Value)` can be deeply cloned before the shared delivery exists. All of this happens under the global bus mutex.
- A zero-target publish still allocates a sequence and delivery, clones its topic and walks payload size. Every publish also materializes per-target delivered/coalesced/dropped/backpressured vectors although most production callers discard the report; only the transaction sink currently inspects it.
- `drain_deliveries` takes the whole ordered inbox and collects a full `Vec` while holding the bus mutex. One call can transfer up to the lossless entry/byte limits with no count, byte or deadline budget. Custom JSON size calculation is stack-safe but traverses the full tree under that same lock.
- Latest byte-pressure eviction is bounded to 256 entries but can combine ordered scans with interior `VecDeque` removal. This is a measurement candidate, not yet proof that a more complex slot arena is beneficial. `age_in_messages` is sequence distance rather than wall-clock queue age.
- Built-in topic construction remains inconsistent. Transaction/document callers now use canonical constructors and avoid validation scans, but each call still owns a fresh topic String; job paths cache, while scene inspection parses per publish and tool operations parse per API call. Document cadence is low, so the new constructor is a current-source improvement rather than a separate hotspot.
- `SceneInspection` uses one latest key for generation deltas. Replacing an unread delta without merge or an explicit gap-to-resync contract can leave the consumer unable to reconstruct the current artifact; the dynamic gate must prove gap detection and full resync semantics.
- The retained-host product subscriber is the highest-priority path. `EditorPluginLifecycleMessageBridge::pump` drains its entire bus inbox, appends it to a second `pending: VecDeque`, holds the pending mutex and runs all active-plugin callbacks without count, byte or time budget on the UI tick. Slow callbacks therefore stall the editor and the duplicated pending owner has no entry/byte/oldest-age telemetry. Callback error retry preserves the front item, but it does not bound the next tick.

## Optimization plan and acceptance

- PERF-MVP-019: publish an immutable subscription generation and give each subscriber its own short-held inbox owner. Resolve targets from the frozen generation, prepare/size one shared delivery once, then enqueue outside the route lock. Add a count+bytes+deadline drain page with `remaining` and oldest wall age; keep lossless ordering and atomic fanout. Offer a summary/count dispatch result for callers that do not request per-target diagnostics, and return before sequence/delivery allocation when no target exists.
- PERF-MVP-594: remove the bridge's full-drain/second-unbounded-owner shape. The bridge must pull at most its tick entry/byte/deadline allowance, dispatch outside the pending mutex, expose bus/bridge backlog bytes and oldest age, and preserve terminal lifecycle ordering with typed backpressure rather than dropping. Slow/faulted plugins need generation-aware cancellation, timeout/slow diagnostics and unload/reload-safe quarantine. Keep deterministic editor-state commit on the editor owner; use a Runtime11 bounded single-flight ticket only for plugins that explicitly declare non-main callback affinity, never a private thread pool.
- Matrix for the bus: subscribers `0/1/100/10K`, publishers `1/16`, payload `64 B/2 MiB`, lane depth `0/1/256/4,096`, stall `0/60 s`. Record route/global/per-inbox lock wait and hold, target visits, prepared deliveries, payload traversals/cloned bytes, report-vector allocation, queue entries+bytes+oldest age, drain page count/bytes and RSS.
- Matrix for the bridge: deliveries `0/1/64/4,096`, active plugins `0/1/100/1K`, callback `0/1/16 ms/10 s`, payload `64 B/2 MiB`, error/reload/unload and `1/16` producer threads. Record duplicated payload owners, callbacks per tick, callback-in-lock wall, UI p50/p95, retry age and memory peak.
- Require zero work for a stable zero-target publish beyond target lookup; one payload-size pass and one payload owner per accepted publish; no callback under bus or bridge pending locks; bounded main-thread work per tick; no accepted-message loss, duplication or reordering; atomic lossless fanout; correct request revalidation; correct latest coalesce/gap resync; and existing dirty, mode, document and plugin lifecycle behavior parity.

## Cross-engine evidence and intentional divergence

- Bevy `dev/bevy/crates/bevy_ecs/src/message/messages.rs`, `message/update.rs` and `message/message_cursor.rs` use explicit message generations, cursor state and missed-message accounting. This supports explicit lifetime/backlog telemetry, but Zircon cannot adopt two-update expiry for lossless plugin lifecycle events.
- Godot `dev/godot/core/object/message_queue.{h,cpp}` bounds storage with fixed pages, reports allocation exhaustion and unlocks around callbacks. Zircon should copy the bounded-storage and lock-out-callback properties, while retaining a per-tick budget rather than Godot's flush-all behavior.
- Unreal `Engine/Source/Runtime/Messaging/Private/Bus/MessageRouter.{h,cpp}` routes immutable contexts through an MPSC command path and honors recipient thread affinity. Zircon should likewise keep routing locks separate from callback wall time, but it should not create a private router thread: local editor ordering remains on the deterministic owner and only explicitly declared heavy affinity may use Runtime11.

## Static gates executed

- Read all 29 production files, all 10 related external test files and the tracked production publisher/subscriber chain at current source; after fingerprint drift, reread the changed `topic.rs` plus its document publisher and reproduced the 29-file fingerprint.
- `rustfmt --check --edition 2024` conforms for 26/29 production files. `bus.rs`, `inbox.rs` and `message/delivery.rs` currently fail only import-order formatting; no source was edited to absorb foreign worktree changes.
- No managed Cargo, ignored 1/100-subscriber backpressure benchmark, multi-thread contention run, 60-second stalled-subscriber RSS run or F4 retained-host WPR trace ran. RenderDoc is not applicable to this non-rendering slice. The module remains pending.
