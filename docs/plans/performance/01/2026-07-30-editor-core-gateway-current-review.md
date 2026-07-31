# Editor core gateway current-source review

## Status

- Result: `source_review_complete / static_gate_pending / dynamic_pending`.
- Review date: 2026-07-30.
- Owners: Editor01 owns the stable gateway and editor/runtime boundary; Editor02/04 own active plugin-event and Play callers; Runtime10/11 own the serialized session lane and bounded worker execution; Render17/EditorUI08 own the viewport texture/readback path.
- Accounting: keep `zircon_editor/src/core/gateway/**` in `pending.md`. Do not add it to `review.md` until current-source formatting, managed gateway tests, slow-callback/scale counters and F4 product evidence are GREEN.
- Code disposition: no Rust source was changed. The current tracked dirty gateway and external gateway tests were preserved.

## Exact scope

| scope | files | physical lines | inline tests | ordered path-and-raw-content SHA256 |
|---|---:|---:|---:|---|
| `zircon_editor/src/core/gateway/**` | 8/8 | 1,577 | 3 | `3e65d19c702dfa957f3921e87935be485864d85833c4aacec34ac4f9369f5abc` |
| `zircon_editor/src/tests/gateway/**` | 4/4 | 1,459 | 36 | `0c2267ff03420bd3b6817fe1cb7905da5e6cb87bf813da6119fe351f904bde3b` |

Both fingerprints stream each sorted native workspace-relative path, a zero byte, raw file bytes and a zero byte into SHA256. The source and its external contract tests were read in full. Production reachability was traced through runtime-library session construction, edit-authoring world access, Play gateway attachment, retained-host frame tick and runtime-event consumer drain.

## Per-file review

| file | current-source performance result |
|---|---|
| `capabilities.rs` | Capability/plugin rows are sorted and deduplicated once when a gateway generation is built. Stable reads return a shared `Arc`; no per-query String deep clone remains. Construction is O(C log C + P log P), off the frame path. |
| `contract.rs` | Frame demand is a typed OnDemand/SleepUntil/Continuous value. `EditorRuntimeFrame` privately owns either a local Vec or foreign ABI storage and exposes a borrowed slice, so the former mandatory gateway RGBA copy is gone. All runtime methods remain synchronous contracts. |
| `detached.rs` | Constant-time typed fallback errors only. |
| `error.rs` | Typed error payloads only; formatting allocation occurs on failures. |
| `handle.rs` | Current transport/capabilities are one immutable `GatewayGeneration` published through `ArcSwap`; stable calls do an atomic snapshot and retain the old generation for the call, with no shared read lock. Replacement alone uses a mutex and invokes the incoming gateway's `capabilities()` while holding it; this is low-frequency but needs a bounded publication contract. |
| `in_process.rs` | TLS reentry detection is O(1) and prevents same-thread deadlock. The user callback still executes while `LevelSystem` holds its World mutex, so full scene clone/serialization or a slow edit callback extends world-lock hold; route large snapshots through PERF-MVP-550 and keep edit deltas under PERF-MVP-063. |
| `mod.rs` | Module wiring and exports only. |
| `session.rs` | Owned ABI buffers are validated and released exactly once; frame bytes remain zero-copy until explicit release/drop. Active Play calls `tick_frame`, then runtime-event pumping calls `drain_plugin_events` on the retained UI tick. The latter performs synchronous runtime FFI plus bounded-page JSON decode on that caller. Timing is recorded, but there is no single-flight worker lane or deadline preventing a slow provider from stalling the frame. Operation poll/harvest also decode owned JSON per call, but no editor product caller currently uses those methods. |

## External test review

| file | coverage result |
|---|---|
| `tests/gateway/handle.rs` | Six tests cover stable identity, capability Arc reuse, in-flight generation lifetime, writer recovery, no RwLock and detached errors. No high-frequency atomic-load or slow replacement scale gate exists. |
| `tests/gateway/in_process.rs` | Seven tests cover read/write, handle forwarding, reentry, panic recovery and TLS isolation. No long-callback/world-lock contention test exists. |
| `tests/gateway/mod.rs` | Test module wiring only. |
| `tests/gateway/session.rs` | Twenty-three tests cover ABI demand, owned-buffer cleanup, frame shape/lifetime, event/operation identity and limits. No slow FFI, max-page decode allocation/latency or poll-storm counter exists. |

## Corrected and remaining tasks

### PERF-MVP-023: gateway frame copy is fixed; GPU readback is not

`SessionGateway::capture_frame` now retains the runtime-owned buffer and provider owner behind a private pixel object, validates the shape once and releases on explicit release/drop. The former gateway-sized RGBA clone is stale. Normal viewport performance still requires the render/framework path to avoid synchronous GPU-to-CPU readback and CPU re-upload; no editor production caller currently invokes this gateway capture method.

### PERF-MVP-068: stable gateway lock and capability clone are fixed

The current handle uses `ArcSwap<GatewayGeneration>` and shared capability `Arc`s. Stable tick/event/capture/profile/subscription/operation calls no longer take an `RwLock`, and capability queries no longer clone Strings. Remaining acceptance is current-source managed tests plus replacement/tick stress and F4 trace; replacement must not hold its writer mutex across an unbounded capability callback.

### PERF-MVP-424: runtime demand propagation is fixed

`SessionGateway` validates and maps the ABI demand into OnDemand, bounded SleepUntil or Continuous, and the retained host applies it. The former constant-true/dropped-delay statement is stale. Remaining cadence work is unrelated window/device event filtering and proving idle/continuous behavior under product traces.

### PERF-MVP-597: synchronous gateway work still runs on the retained frame caller

The active Play path calls runtime `tick_frame` and then plugin-event drain/decode synchronously from retained-host polling. Page count/encoded bytes and elapsed metrics are bounded/observed, but a slow provider call or max-page JSON decode directly becomes UI frame wall. Editor01/02/04 and Runtime10/11 must introduce one ordered, bounded, generation-owned session ticket lane: at most one tick/drain flight per session, no stale completion apply, explicit cancel/shutdown, and shared decoded page ownership. Do not create a gateway-private pool. Operation/profile/capture use the same lane only when product callers appear.

## Acceptance plan

- Stable handle: 1/1M calls, replacement 0/1/1K and 1/16 threads. Count shared-lock acquisitions, Arc/String clone bytes, atomic snapshots, generation retention and replace wait/hold. Stable shared lock and deep capability clone must remain zero.
- Session lane: provider latency 0/1/16ms/10s, event pages 0/1/64 deliveries and 0/1KiB/max encoded bytes, ticks 30/60/120Hz. Record UI-thread FFI/JSON wall, in-flight/queued entries and bytes, age, decode allocation, stale completions and cancellation. UI-thread foreign/decode wall must be zero and lane capacity hard-bounded.
- World access: callback 0/1/16ms/10s, readers/writers 1/16 and scenes 1/1K/100K entities. Record World mutex wait/hold and cloned bytes; large snapshot/serialization must not execute as an unbudgeted edit callback.
- Run current-source managed `zircon_editor` gateway and app lifecycle tests, then F4 embedded Play start/idle/continuous/event-storm/stop. RenderDoc belongs to PERF-MVP-023's render-owner first-frame/readback gate; this CPU/control review had no product gateway capture caller to capture.

## Reference check

- Unreal `dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/PlayLevel.cpp` stores a Play request and starts it later through `StartQueuedPlaySessionRequest`; Zircon should similarly publish session work before executing foreign work while retaining one lifecycle authority.
- Bevy `dev/bevy/crates/bevy_tasks/src/usages.rs` separates cross-frame compute from I/O-intensive work through shared task-pool owners. Zircon's runtime FFI/JSON lane should reuse Runtime10/11 ownership rather than adding a per-gateway executor.
- Godot `dev/godot/editor/run/editor_run_bar.cpp` centralizes start/stop replacement under one run owner. Zircon must preserve the gateway generation/session authority while moving blocking execution outside the retained UI caller.

## Static gates executed

- Read 8/8 production files and 4/4 external gateway test files in full; traced the active product callers listed above.
- `git diff --check -- zircon_editor/src/core/gateway zircon_editor/src/tests/gateway` passed; Git reported only existing LF-to-CRLF checkout warnings.
- Final `rustfmt --edition 2021 --check` is RED in production `handle.rs` and `session.rs`, plus external `tests/gateway/session.rs` (import ordering and two assertion layouts). These dirty files belong to existing work and were not reformatted here.
- `review.md` remained unchanged. No managed Cargo, allocation/lock/latency scale run, WPR F4 trace, RenderDoc capture or independent dynamic review ran.
