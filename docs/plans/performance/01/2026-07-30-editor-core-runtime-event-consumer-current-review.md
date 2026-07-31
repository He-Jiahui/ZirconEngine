# Editor core runtime event consumer current-source review

## Status

- Result: `static_complete / dynamic_pending`.
- Review date: 2026-07-30.
- Owners: Editor02 owns main-thread pump/fairness; Plugins01 and Runtime10 own plugin-event transport/ABI; Runtime11 owns any bounded decode ticket used after measurement.
- Accounting: keep the module in `pending.md`; do not add it to `review.md` before current-source managed Cargo, slow-consumer/idle ABI counters and an F4 active-play trace are GREEN.
- Code disposition: no Rust source was changed. Existing modified and untracked source was preserved.

## Exact scope

| module | files | physical lines | inline tests | ordered path-and-raw-content SHA256 |
|---|---:|---:|---:|---|
| `zircon_editor/src/core/runtime_event_consumer` | 6/6 | 1,180 | 0 | `cbf07a2e88b3692d50515bd47db3f67276feb22f8649ec5f51308c4ddb6e5bf3` |

The fingerprint streams each sorted workspace-relative path, a zero byte, the file's raw bytes and a zero byte into SHA256. The six files are `error.rs`, `host.rs`, `manifest.rs`, `mod.rs`, `pump.rs` and `registration.rs`; all were read in full. The related external suites contain 16 tests across `tests/runtime_event_consumer.rs` and `tests/runtime_event_consumer_bounded_pump.rs`, including one ignored managed 1K/10K ABI benchmark.

The product chain was followed from retained-host `tick`, through `EditorHostEventController`, `EditorRuntimeEventConsumerHost`, editor gateway/session decode, runtime dynamic-session mirror and the subscription queue. Supporting boundary files are evidence for this slice, not newly accepted folder accounting.

## Current-source corrections

1. The 2026-07-22 statement that production still drains an unlimited delivery `Vec` is no longer current. Runtime mirror pages are now capped at 64 events and 128 KiB payload; the wire page is capped at 256 KiB; each subscription queue is capped at 16K events and 64 MiB with typed overflow/oversized errors. Runtime focused event-mirror evidence exists, but the editor ABI benchmark is still ignored and has no current managed result.
2. Editor callback ownership is materially safer: active consumers are snapshotted, gateway/decode/callback work happens outside the active-map lock, an atomic execution owner excludes lifecycle mutation, sequence commit is generation/subscription checked, and round-robin plus 256-event/64-per-consumer/4-ms apply budgets exist.
3. The fixed producer page does not bound editor retention. Every visited consumer is drained before its local apply loop, and all returned deliveries are appended to an uncapped `pending: VecDeque`. A callback slower than the producer can move one page per tick out of the bounded runtime queue into editor memory indefinitely. The report exposes entry count and sequence span only, not pending bytes or oldest wall age.
4. A stable active-play tick still clones the capability snapshot/enabled list, clones all registrations, rebuilds desired/existing maps and sets, and reconciles every subscription even when neither generation changed. This is PERF-MVP-565. It also snapshots every active consumer by cloning its id and registration owner.
5. Every active consumer is polled on every active-play tick. The current runtime encodes even an empty delivery batch into an owned JSON buffer and the editor decodes/releases it; the host also allocates two duration sample vectors and sorts page samples for a last-tick-only p95 report that no production observer reads. Empty polling and transport conversion remain PERF-MVP-432/PERF-MVP-069 rather than a new task.
6. Non-empty payloads cross repeated representations: producer event to queued JSON bytes, queued bytes back to `Value`, delivery batch back to wire JSON, editor wire JSON back to `Value`, then typed `from_value`. Event id and schema are cloned per delivery despite stable subscription identity. Each applied event also takes the global active map once to pop and again to commit its sequence.

## Optimization plan and acceptance

- PERF-MVP-565: publish immutable capability and registration generations. Reconcile only at session begin or generation change and compute affected subscription deltas; a stable tick must perform zero capability/registration clones, map/set builds or subscribe calls.
- PERF-MVP-069: do not request a new page while that consumer retains a prior page. Give editor pending explicit entry, encoded-byte and oldest-age limits and expose runtime `remaining/has_more`; preserve accepted-event order and make overload a typed diagnostic. Batch pop/commit or move queue state behind a per-consumer owner so an event does not take the global active map twice.
- PERF-MVP-432: return an empty ABI buffer without JSON encode/decode, identify descriptors once per subscription, and carry one owned payload representation across the boundary. If the fixed 256 KiB page cannot meet the 4-ms gate, hard-cut to a request-aware count/byte/deadline page in the next API table rather than pretending callback budget covers transport.
- Keep sequence/lifecycle commit and editor state application on the deterministic editor owner. Only if measured decode p95 breaches budget, send the owned wire page to one Runtime11 bounded, generation-tagged, per-subscription single-flight decode ticket; do not create a private thread pool or allow parallel reorder.
- Matrix: consumers `1/4/64`, producer backlog `0/1/64/1K/10K`, payload `64 B/2 KiB/128 KiB`, callback `0/1/4/16 ms`, stall `0/60 s`, tick `60/120/240 Hz`, capabilities/registrations `1/100/10K`, stable versus 1% generation changes. Record empty ABI calls/encoded bytes, page count/bytes, encode/decode/typed-decode wall, pending/runtime queue entries+bytes+oldest age, active-map locks, descriptor/payload clone bytes, callback/main-thread p50/p95, drops/errors and RSS.
- Require stable empty tick encode/decode/allocation to be zero, editor pending to stay within one configured page or stricter budget per consumer, accepted-event loss/duplication zero, lifecycle lock-held callback zero, stable reconcile work zero and main-thread p95 within Editor02's F4 budget. Session, sequence, schema, rollback, reentrancy, unsubscribe and reload tests must remain equivalent.

## Cross-engine evidence and intentional divergence

- Bevy `dev/bevy/crates/bevy_ecs/src/message/messages.rs` keeps two message generations and swaps/clears the old buffer once per update; `message_cursor.rs` exposes how many messages a reader missed. This proves lifetime and cursor state must be explicit. Zircon intentionally cannot use Bevy's two-frame expiry for lossless plugin operations, so it needs bounded backpressure and typed overflow instead of silent expiry.
- Godot `dev/godot/core/object/message_queue.{h,cpp}` uses fixed 4 KiB pages, a configurable maximum page count and explicit out-of-memory failure. Its flush pre-advances, unlocks around each callback and relocks afterward, matching Zircon's lock-out callback requirement. Zircon intentionally keeps a per-frame count/time budget rather than adopting Godot's flush-all loop.

## Static gates executed

- Read the six production files, both related test files, retained tick/controller, gateway decode, runtime mirror and producer queue at current source.
- `rustfmt --check --edition 2024` passed for all six exact production files.
- No managed Cargo, ignored 1K/10K ABI benchmark, slow-producer 60-second RSS run or WPR F4 product trace ran. RenderDoc is not applicable to this non-rendering slice. The module remains pending.
