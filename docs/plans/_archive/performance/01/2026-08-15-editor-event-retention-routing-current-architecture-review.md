# Editor event retention and routing current-architecture review

## Status

- Result: `static_complete / dynamic_pending`.
- Review date: 2026-08-15.
- MVP priority: P0 for viewport/pointer input latency and retained-host authoring dispatch; P1 for
  journal export, listener polling and plugin observation.
- Owner: Editor02. EditorUI08 consumes the authoring result/effect contract; Editor12 and Plugins01
  consume bounded observation pages; Runtime11 may schedule only work with explicit non-main affinity.
- Accounting: keep this module in `pending.md`. Do not add it to `review.md` until current-source
  managed Cargo, allocation/lock counters and an F4 WPR trace pass the acceptance matrix.
- Code disposition: no Rust source changed. The reviewed source contains pre-existing modified and
  untracked work and was preserved at the recorded hashes.

## Exact scope

| scope | files | physical lines | tests | ordered path-and-raw-content SHA256 |
|---|---:|---:|---:|---|
| `zircon_editor/src/core/editor_event/**` | 36/36 | 2,667 | 8 inline | `56de05ee5ca871b79aaa37370bb0ecbd20b863e855ef91ede324d83848e1e4da` |
| `zircon_editor/src/tests/editor_event/**` | 30/30 | 8,128 | 138 | `c611be77d08414f426cc72863b0863ad445d9e4ab308bacc62b60164e2c27684` |

The fingerprint is SHA256 over normalized sorted path, NUL, raw file bytes, NUL. Every Rust file in
both folders was read in full. The retained-host dispatch chain through
`ui/host/editor_event_dispatch.rs`, viewport callback dispatch and listener control projection was
also traced. Supporting callers are evidence, not additional folder accounting.

## Architecture verdict

The July `PERF-MVP-067` diagnosis is materially stale. Current retention is bounded by independent
durable, frame-local and latest-state count/serialized-byte/age budgets. Latest replacement uses a
key-to-cursor `HashMap` plus ordered `BTreeMap`/`BTreeSet` indexes, so the old `VecDeque` scan and
middle shift are gone. Listener registry mutation publishes an immutable `Arc<[route]>`; event
filtering and per-inbox enqueue occur after the registry mutex is released. Full record enumeration
uses a three-way ordered merge, and listener polling uses a cursor-first count-bounded merge capped
at 256 rows. The old global-lock fanout, linear latest lookup and full-set sort claims must be removed.

The remaining P0 issue is structural: synchronous editor command execution, audit retention and
plugin/listener observation still share one event-record completion path. Every successful event
deep-clones the owned record so the host can both retain and return it
(`ui/host/editor_event_dispatch.rs:125-152`). `EditorEventService::record` then synchronously creates
a shared owner, serializes the complete record into a temporary `Vec<u8>` only to obtain its length,
updates the journal indexes and sequentially locks every matching listener inbox
(`service/editor_event_service.rs:49-59`, `retention.rs:263-275`). Latest-state coalescing bounds the
resulting queue; it does not bound or remove this per-arrival work.

For a record of encoded size `B`, `L` listener routes, filter work `F`, journal size `J` and inbox size
`I`, current arrival cost is approximately `O(B + log J + L * (F + log I))`, plus one deep record
clone on successful host dispatch. This is acceptable only after measurements prove that the audit
and observer portion stays outside the interactive frame budget. No such measurement exists.

## Current-source corrections

1. Journal defaults are 16,384/64 MiB/24 h durable, 512/4 MiB/2 s frame-local and 256/4 MiB/30 min
   latest. Listener defaults are 1,024/16 MiB/10 min, 128/1 MiB/2 s and 128/2 MiB/10 min
   (`retention.rs:125-137`). These are hard encoded-size budgets, not heap/RSS budgets.
2. Latest replacement is near `O(log N)` and retains arrival cursor order. Tests cover out-of-order
   event sequences, 1,000 latest replacements and exact coalesced diagnostics.
3. Route generation is immutable. A short registry lock clones the route slice; filter probes and
   inbox locks happen afterward. Tests cover 1,000 listeners by 1,000 events and 10,000 paused events.
4. `records()` is a three-way merge without sorting (`retention.rs:509-541`). Listener pages apply the
   delivery cursor first and stop at the requested count (`retention.rs:544-590`).
5. DTO projection is outside the registry and inbox locks. The old missing-field test-literal compile
   warning is no longer present in the current 30-file test tree.

## Remaining bottlenecks

### P0: one synchronous path owns command completion, audit and observation

- `serde_json::to_vec(&record)` allocates and fills a complete buffer on every event, then discards it
  after reading `len`. The retained owner keeps the typed record, not those encoded bytes.
- Success dispatch clones all event strings, JSON values, effects and result before retention. This
  applies to viewport pointer-move bindings as well as low-frequency authoring commands.
- Journal insertion and every matching listener enqueue execute inline on the caller. One contended
  inbox can head-of-line block later independent inboxes; no lock wait/hold or per-route wall counter
  can currently prove otherwise.
- Zero listeners do not skip record serialization because every event is still journaled. Existing
  tests prove bounded final state, not bounded main-thread work per input event.

### P1: polling and snapshots remain materialization-heavy

- The listener page is count-bounded only. It has no returned-byte or deadline budget and can clone
  up to 256 wide records into owned DTOs, then clone/project the DTO fields into JSON
  (`listener/types.rs:32-51`, `listener/projection.rs:24-41`).
- `EditorEventJournalStore::snapshot` merges every retained record and deep-clones the complete set.
  This is not proven per-frame, but is a scale amplifier for diagnostics, replay export and tests.
- Encoded JSON length ignores `Arc`, maps, trees, strings' spare capacity and allocator overhead.
  A 64 MiB serialized budget therefore is not a 64 MiB process-memory bound.
- Filters use linear vectors for prefixes, groups and sources. Keep this simple for small filters; add
  an index only if route/filter counters show it is material at 1k/10k listener scale.
- Saturating event and delivery cursors eventually stop advancing at `u64::MAX`. This is a correctness
  boundary to hard-fail or rotate explicitly, not a present-day performance target.

## Reference-engine evidence

- Unreal `FMessageRouter` owns a command-processing loop that sleeps on a work event
  (`dev/UnrealEngine/Engine/Source/Runtime/Messaging/Private/Bus/MessageRouter.cpp:53-63,256-264`). It
  first builds the recipient set, invokes `AnyThread` recipients directly and dispatches receivers
  with named-thread affinity through TaskGraph (`MessageRouter.cpp:118-182`). Adopt the separation of
  routing ownership and declared affinity; do not copy a private router thread into the MVP blindly.
- Unreal Slate owns unique pre/prepass/post invalidation heaps and inserts dirty widgets uniquely
  (`dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/FastUpdate/SlateInvalidationRoot.cpp:231-233,317-329`).
  It measures fast paint and invalidation processing as distinct stages (`SlateInvalidationRoot.cpp:723-726,1281-1289`).
  Zircon likewise needs separate command, journal and observer stages before changing scheduling.
- Bevy `MessageCursor` derives pending and missed counts from monotonic message counts without scanning
  payloads (`dev/bevy/crates/bevy_ecs/src/message/message_cursor.rs:119-145`). Zircon's cursor-first
  page is aligned with this; keep its stronger durable/frame/latest and acknowledgement semantics.
- Godot `CallQueue` uses 4 KiB pages, rejects a message larger than one page, exposes maximum buffer
  usage and has an explicit statistics pass
  (`dev/godot/core/object/message_queue.h:46-50,137-144`; `message_queue.cpp:85-107,347-424,457-458`).
  This supports explicit admission/usage telemetry, not an unbounded deferred callback queue.

## Optimization plan

### Milestone 1: instrument before ownership changes

- Add stage spans/counters for record construction, success clone, encoded-size traversal/allocation,
  journal prune/index/coalesce, route snapshot/filter visits, each inbox lock wait/hold/enqueue and
  listener page projection/JSON bytes. Report caller thread/affinity and event retention class.
- Measure 0/1/1k/10k listeners, 0/50/100% filter match, 64 B/2 MiB/64 MiB records, 1/16 producers,
  current/60 s stalled consumers and pointer move at 125/500/1,000 Hz.
- Capture F4 editor interaction with WPR/xperf once the approved-root build helper is fixed. Attribute
  main-thread wall to command execution versus audit versus observation; collect CPU, context switch,
  allocation/RSS and package-power evidence. RenderDoc is not applicable to this CPU-only boundary.

### Milestone 2: split the ownership contract

- Make command completion produce one shared immutable result/effect owner. The retained host applies
  effects from that owner without deep-cloning a second complete record.
- Keep synchronous authoring mutation and effect projection on the editor owner. Move only audit and
  explicitly non-main observers behind a bounded handoff; preserve source order, failure reporting,
  undo/replay identity and main-affinity callbacks.
- Decide from measurements whether journal retention should own typed data plus calibrated heap
  accounting, one reusable encoded owner, or a compact transient/latest representation. Do not retain
  both full typed and discarded full encoded copies merely to enforce a nominal byte limit.

### Milestone 3: bound fanout and materialization

- Give observation delivery an entry/byte/deadline slice with `remaining`, oldest wall age, dropped
  and coalesced diagnostics. A slow inbox must not serialize independent owners indefinitely.
- Add byte/deadline limits to listener pages. Keep shared rows in-process and create owned JSON once at
  the plugin/ABI boundary. Replace journal full snapshots with cursor/page export for product callers.
- Use Runtime11 only for declared non-main work with bounded admission and generation-safe
  cancellation. Do not introduce a second editor scheduler or fire-and-forget pool.

### Milestone 4: close product-scale evidence

- Repeat the same WPR scenario before and after. Stable no-listener input must do zero listener work;
  latest replacement must not grow retained records; accepted observers must preserve order and
  exactly-once acknowledgement semantics.
- Compare stage shape, scheduling and admission behavior with the Unreal evidence above. Numeric
  acceptance comes from Zircon's same-machine baseline and frame budget, not invented Unreal values.

## Quantified acceptance

| matrix | required measurements | acceptance |
|---|---|---|
| pointer/viewport: 125/500/1,000 Hz; listeners 0/1/1k; payload 64 B/2 MiB | command/audit/observer wall p50/p95/p99, full record clone bytes, serde traversals/allocated bytes, journal operations, CPU/context switches/package power | redundant complete record clone=0; no-listener observer visits/locks=0; size traversal/encoding has one retained owner and no discarded full buffer; order/effects unchanged |
| fanout: listeners 1/1k/10k; match 0/50/100%; producers 1/16; one inbox stalled 60 s | route visits, global/per-inbox lock wait+hold, enqueue wall, backlog entries/bytes/age, dropped/coalesced counts, UI input p95 | registry lock excludes filter/enqueue; stalled inbox cannot create unbounded caller wall/backlog; hard admission and accepted ordering hold |
| polling: cursor 0/1/99%; page 1/64/256; payload 64 B/2 MiB | rows visited/returned, shared/owned/JSON bytes, page wall, `remaining`, oldest age, RSS | cursor applied before materialization; count+byte+deadline bounds hold; no full journal snapshot in polling path; no duplicate/skip across pages |
| product F4 before/after | WPR CPU stacks, ready/running time, context switches, allocations/RSS, package power, input-to-present p50/p95 | audit/observer work is separately attributable and reduced; no regression in command/undo/replay/plugin semantics; power and frame claims reported only from measured same-machine deltas |

## Static gates executed

- Read all 36 production files and all 30 external test files twice at the recorded fingerprints, plus
  the current retained-host callers and the Unreal/Bevy/Godot primary sources above.
- `rustfmt --edition 2021 --check` is green for all 36 production files. External checks are
  formatting-only red through four entry files: `mod.rs`, `runtime/mod.rs`,
  `runtime/animation_assets.rs` and `support.rs`; no foreign source was formatted.
- Managed Cargo did not run because `tools/build-editor.ps1:130` still rejects valid D:/E:/F: roots
  through its single-quoted doubled-separator bug. See
  `failure-2026-08-15-build-editor-approved-root-separator.md`.
- WPR/xperf and RenderDoc are installed, but there is no launchable current-source editor binary.
  RenderDoc does not validate this CPU routing slice and no rendering claim is made.
- No latency, power or algorithmic improvement is claimed. Dynamic evidence remains mandatory.
