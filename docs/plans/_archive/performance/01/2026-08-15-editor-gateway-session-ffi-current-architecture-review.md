# Editor gateway session FFI current-architecture review

## Status

- Result: `static_complete / dynamic_pending`.
- Review date: 2026-08-15.
- MVP priority: P0 for active Play frame latency, runtime replacement/stop latency and future
  serialized WorldSync; P1 for interactive profile-control latency; P2 for currently unwired overlay
  canonicalization.
- Owners: Editor01 owns the stable gateway and execution-affinity contract; Editor02 owns WorldSync
  transport and invalidation demand; Editor04 owns Play session lifecycle; Runtime10 owns the dynamic
  ABI/session contract; Runtime11 owns the ordered execution lane; EditorUI08 owns retained-frame
  polling and immutable completion application.
- Accounting: keep both reviewed trees in `pending.md`. Do not add them to `review.md` before current
  managed Cargo, slow-provider/scale counters, F4 WPR and same-machine CPU/RSS/power evidence pass.
- Code disposition: no Rust source changed. Nineteen of the 21 current production files and all 11
  current external test files have pre-existing modifications or are untracked; their bytes and
  ownership were preserved.

## Exact scope

| scope | files | physical lines | tests | ignored | ordered path-and-raw-content SHA256 |
|---|---:|---:|---:|---:|---|
| `zircon_editor/src/core/gateway/**` | 21/21 | 2,711 | 10 | 0 | `5a0618728caa7edaa2683fd82dc072e3b62f57a2615fd33202a28d7a516684f0` |
| `zircon_editor/src/tests/gateway/**` | 11/11 | 2,082 | 51 | 0 | `082630729f7b4f70f320b2420ad7dc4f718a167722c477a34130ef8bc459ad10` |

Each fingerprint streams every normalized sorted workspace-relative path, NUL, raw file bytes and
NUL into SHA256. Every Rust file in both trees was read in full. Product reachability was traced
through the retained host, Play controller, runtime-event consumer, WorldSync pump, profiling action
and current gateway call sites.

The 2026-07-30 report's 8 production files/1,577 lines/3 tests and 4 external files/1,459 lines/36
tests is obsolete. The session transport and its tests have been split into focused modules, viewport
surface forwarding and WorldSync tests have expanded, and all current files now pass rustfmt.

## Architecture verdict

The ownership/lifetime layer is substantially healthier than the old architecture:

- `EditorRuntimeGatewayHandle` publishes immutable `GatewayGeneration` values through `ArcSwap`;
  stable calls take an atomic generation snapshot and do not acquire a shared read lock
  (`handle.rs:29-38,74-80,110-242`).
- capability rows are sorted/deduplicated at generation construction and stable reads share one
  `Arc<RuntimeCapabilities>`; no repeated String deep clone remains;
- runtime-owned frame and JSON buffers are validated and released exactly once; captured frame bytes
  remain runtime-owned until explicit release/drop instead of being copied by the gateway;
- frame demand preserves OnDemand, bounded SleepUntil and Continuous semantics;
- plugin-event pages are bounded by 64 deliveries and 128 KiB before application, WorldSync outputs
  are bounded to 1 MiB, and protocol identities/ABI values fail closed.

Those fixes remove the old shared-lock, capability-clone, frame-copy and discarded-demand findings.
They do not define where synchronous transport work executes. Every `SessionGateway` method is still
a direct call-through contract. The current active Play retained tick performs
`play_gateway.tick_frame()` and then `runtime_event_consumers.pump()` inline
(`ui/host/editor_host_event_controller.rs:227-240`) from the retained host tick
(`ui/retained_host/app/host_lifecycle/tick.rs:8-30`). For each active consumer without an editor
pending page, the pump invokes `drain_plugin_events` before its next budget check
(`core/runtime_event_consumer/host.rs:396-431`). That call performs foreign FFI and JSON decode on
the caller (`gateway/session/plugin_events.rs:70-135`).

The 4 ms/256-event pump budget only checks elapsed time before a drain and between callbacks. It
cannot preempt one 10-second provider call or a maximum-page decode. Consequently the current
architecture is memory-bounded but not frame-time-bounded: one provider can still add its entire
wall time to the editor UI frame.

## Structural bottlenecks

### P0: active Play FFI and decode execute on the retained UI frame

`tick_frame` is a synchronous foreign call (`gateway/session/frame.rs:35-44`). The next operation in
the same retained frame snapshots every active event consumer and may synchronously drain/decode one
page per visited consumer. Runtime and decode elapsed values are observed only after they have
already blocked the caller. Per-callback limits do not constrain provider wall time.

The replacement must be one session-owned ordered lane supplied by Runtime11, not a gateway-private
pool. Tick, plugin drain and lifecycle commands submit generation-tagged tickets. The retained host
polls immutable completions, applies only the current gateway/session generation and never performs
foreign work or JSON decode. The lane must be single-flight per session, hard-bound queued entries,
bytes and age, preserve event order, support stop/unload cancellation, and keep provider-owned output
alive until decode/release completes. Explicit thread-affinity classes are required before moving
render/surface or borrowed in-process World work.

This is the current scope of `PERF-MVP-597`. It is not fixed by the existing 4 ms consumer callback
budget.

### P0: watch allocation and revocation run under the gateway replacement mutex

`with_current_gateway_generation` locks replacement and then executes an arbitrary closure
(`gateway/handle.rs:89-101`). `WorldSyncPump::watch_view_with_gateway_generation` calls the transport's
`watch_world`, may call `unwatch_world`, and mutates the editor watch map inside that closure
(`core/sync/pump.rs:137-166`). `unwatch_view` similarly calls the transport under the lock
(`core/sync/pump.rs:170-189`). A slow serialized provider can therefore block runtime replacement,
Play stop and recovery for its entire wall time.

Preserve token/generation correctness without holding the writer mutex across foreign work:

1. obtain a strong generation lease containing id, gateway and provider owner through ArcSwap;
2. allocate/revoke on the session lane outside the replacement mutex;
3. acquire the mutex only to compare the current generation and commit the editor binding;
4. if stale, compensate through the retained generation lease and discard the result;
5. record replacement wait/hold, stale allocation and cleanup failure explicitly.

The generation lease must keep the old session provider alive; token ownership may not be transferred
to the replacement generation.

### P1: serialized WorldSync drain has a 1 MiB caller-side JSON path

The retained host invokes edit-world invalidation pumping every tick before Play/event pumping
(`ui/retained_host/app/host_lifecycle/tick.rs:20-25`). The current edit-authoring gateway is normally
in-process, so its drain is a direct bounded LevelSystem operation. A SessionGateway/remote Play
attachment, however, executes foreign drain plus up to 1 MiB JSON decode synchronously
(`gateway/session/world_sync.rs:100-125`). `query_world` has the same request encode/response decode
shape (`world_sync.rs:14-44`), although there is no current editor product caller outside tests.

Editor02 must reuse the same session lane and publish an immutable invalidation completion keyed by
gateway/session/world generation. Idle serialized sessions should be wake/demand driven rather than
issuing an empty ABI drain every retained tick. The in-process borrowed World path stays direct and
must retain its reentry guard; large snapshot/serialization work remains owned by `PERF-MVP-550` and
large edit deltas by `PERF-MVP-063`.

### P1: replacement materializes foreign capabilities under its writer mutex

`replace` takes the replacement mutex before `GatewayGeneration::new`, which calls incoming
`gateway.capabilities()` (`gateway/handle.rs:40-47,65-71`). Replacement is low-frequency and stable
reads are unaffected, but a slow or panicking capability provider extends/poisons the lifecycle
critical section. Materialize and validate the incoming immutable capability snapshot before the
publication lock, then lock only to allocate the next id and store the generation. Preserve the
existing panic recovery and old-generation lifetime tests.

### P2: highlight canonicalization is eager and currently unwired

`EditorRuntimeHighlightSet::new` inserts all entity ids into a `BTreeSet` and then collects a Vec.
This gives deterministic sorted/deduplicated output but performs O(H log H) tree work and node
allocation per constructed set. No production caller currently constructs or submits a highlight
set; all current call sites are gateway tests. Editor05 should publish a shared sorted selection
generation when overlay wiring becomes product-reachable. A measured `Vec::sort_unstable + dedup`
may replace the tree for one-shot construction, but changing it now would optimize an unwired path
without a baseline.

## Per-file production review

| file | current-source performance result |
|---|---|
| `capabilities.rs` | Construction sorts/deduplicates core/plugin rows once; stable reads share the generation Arc. No frame-path issue. |
| `contract.rs` | Typed synchronous gateway contract; foreign frame storage is borrowed behind a private owner. Execution affinity/deadline is absent by design. |
| `detached.rs` | Constant-time typed capability failures only. |
| `error.rs` | Error formatting allocates only on failure. |
| `handle.rs` | ArcSwap stable path is fixed. Replacement capability callback and generation-bound watch closure still run under the writer mutex. |
| `highlight_set.rs` | Deterministic BTreeSet canonicalization is eager O(H log H); no production constructor caller. |
| `in_process.rs` | TLS reentry guard is O(1). Borrowed callbacks execute while LevelSystem holds World access, so callback wall remains lock hold. |
| `mod.rs` | Export wiring only. |
| `session/contract.rs` | Trait forwarding only; preserves the synchronous execution model. |
| `session/frame.rs` | Direct synchronous tick/event/capture ABI calls. Frame bytes remain zero-copy and safely owned. |
| `session/gateway.rs` | Validates V6/session identity and retains provider owner; no scheduler or deadline authority. |
| `session/mod.rs` | Module/export wiring only. |
| `session/operations.rs` | Submit/harvest JSON and poll fixed status synchronously. No current product operation caller. |
| `session/output.rs` | Centralized owned-output validation/release is single-owner and fail-closed. No redundant validation remains. |
| `session/overlay.rs` | Direct fixed-shape highlight ABI submission; currently test-only. |
| `session/plugin_events.rs` | Hard page limits and elapsed counters are correct, but FFI and JSON decode occur synchronously before the caller regains control. |
| `session/profile.rs` | Interactive profiling actions synchronously encode/call/decode; route through the session lane when dynamic product profiling is available. |
| `session/protocol.rs` | Constant-time ABI/status/shape validation and 60-second demand clamp; no hot algorithm defect. |
| `session/tests.rs` | Ten inline tests cover surface/highlight forwarding, state, missing entries, one-pass validation and invariant cleanup; no slow-provider test. |
| `session/viewport.rs` | Surface bind/unbind/present are direct ABI calls. Keep explicit render/native thread affinity when product wiring lands. |
| `session/world_sync.rs` | JSON request/response and direct FFI with a 1 MiB response cap; serialized caller wall remains unbounded. |

## Per-file external test review

| file | coverage result |
|---|---|
| `tests/gateway/handle.rs` | Stable identity, Arc reuse, in-flight lifetime, panic recovery and no-RwLock are covered; no slow replacement/watch contention scale gate. |
| `tests/gateway/highlight_set.rs` | Sorted/deduplicated output and production/test separation covered; no allocation/scale gate. |
| `tests/gateway/in_process.rs` | Direct World access, queries, watches, overlay, reentry, panic and TLS isolation covered; no long-callback contention gate. |
| `tests/gateway/mod.rs` | Test wiring only. |
| `tests/gateway/session/construction.rs` | Invalid handle/API, capability materialization and serialized-access rejection covered. |
| `tests/gateway/session/fixture.rs` | Shared fake ABI/output owner fixtures; payload construction is test-only. |
| `tests/gateway/session/frame_demand.rs` | Demand mapping/error/profile absence covered; no slow tick/thread-affinity test. |
| `tests/gateway/session/mod.rs` | Test wiring only. |
| `tests/gateway/session/output_ownership.rs` | Provider/frame lifetime and exactly-once release covered. |
| `tests/gateway/session/plugin_operations.rs` | Page limits, crossed identities, malformed statuses and cleanup covered; no max-page allocation/latency or provider-stall gate. |
| `tests/gateway/session/world_sync.rs` | Query/watch/unwatch/drain transport and invalid token covered; no 1 MiB decode, idle-drain cadence or generation-replacement race gate. |

## Unreal primary-source comparison

- Unreal `PlayLevel.cpp:1116-1165` stores and starts one queued Play request, resets it after every
  attempt and retains original request state for asynchronous processes. Zircon should preserve one
  Play/session lifecycle authority while moving execution off the retained UI caller.
- Unreal `Messaging/Private/Bus/MessageRouter.cpp:167-181` executes directly only for AnyThread
  recipients; otherwise it constructs a `FMessageDispatchTask` for the declared recipient thread.
  `MessageDispatchTask.cpp:26-47` resolves the weak recipient and performs the callback on that task.
  Zircon currently has no equivalent execution-affinity decision at the SessionGateway boundary.
- Unreal `Core/Public/Tasks/Pipe.h:20-27,55-78` defines a per-resource FIFO, non-concurrent task
  chain and explicitly describes it as a lightweight named-thread replacement. The corresponding
  Zircon design is a Runtime11-owned per-session lane, not one new OS thread or pool per gateway.

These references establish queue ownership, ordering and thread-affinity structure. They do not by
themselves prove Zircon latency or power parity; that requires the same-machine measurements below.

## Acceptance and measurement plan

| case | matrix | required counters and result |
|---|---|---|
| stable handle | 1/1M calls; replacement 0/1/1K; 1/16 caller threads | shared-lock acquisitions=0, capability String clone bytes=0, generation retention correct, atomic snapshot p50/p95 recorded |
| session lane | provider 0/1/16ms/10s; 30/60/120Hz; consumers 0/1/64; pages 0/1/64 rows and 0/1KiB/128KiB | UI-thread foreign/decode wall=0, per-session in-flight<=1, entries/bytes/age hard bounded, no accepted loss/dup/reorder |
| watch lifecycle | watch/unwatch 0/1/16ms/10s crossed with 0/1/1K replacements | foreign wall under replacement mutex=0, writer hold bounded to commit, stale commit=0, token leaks=0, cleanup failures observable |
| WorldSync | empty/1/1K batches; 0/1KiB/1MiB; 30/60/120Hz | idle empty drains approach demand changes rather than frame count, UI decode wall=0, generation regression/stale completion rejected |
| shutdown/reload | stop/unload during queued/running tick, drain and watch | bounded cancel/join, no use-after-unload, old provider released once, stale completions applied=0 |
| product | F4 embedded Play start/idle/continuous/event storm/stop and profile snapshot | WPR thread/CPU/wake/lock/queue p50/p95, RSS and package-power exported under workspace; current-source Cargo and lifecycle behavior GREEN |

For engine comparison, run the same minimal scene, resolution, frame cap, foreground/background and
30-second idle/continuous windows on the same machine and power plan. Record process CPU, package
power, wakeups, frame p50/p95 and RSS for Zircon and an available local Unreal editor build. Do not
claim parity from source shape or compare different scenes/configurations. RenderDoc is reserved for
the viewport present/readback owner once a runnable product frame exists; it cannot diagnose this
CPU/session scheduling boundary.

## Static gates executed

- Read 21/21 production and 11/11 external gateway test Rust files in full; fingerprints were
  computed twice and remained identical.
- `rustfmt --edition 2021 --check` passed 21/21 production and 11/11 external files.
- `git diff --check -- zircon_editor/src/core/gateway zircon_editor/src/tests/gateway` passed; Git
  emitted only existing LF-to-CRLF checkout warnings.
- Managed Cargo and product profiling were not retried because
  `failure-2026-08-15-build-editor-approved-root-separator.md` records the current Windows build
  helper rejecting valid non-C target roots. No output artifact was written to C:.
- `review.md` and `pending.md` remained unchanged because they are protected/foreign dirty. This is
  not an accepted dynamic milestone, so no commit or WeCom notification is due.
