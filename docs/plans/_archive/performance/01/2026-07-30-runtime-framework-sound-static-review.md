# Runtime framework sound current-source static performance review

## Status

- Result: `static_complete / dynamic_pending`.
- Review date: 2026-07-30.
- Accounting: keep in `pending.md`; do not add to `review.md` before current-source managed Cargo and product counters are GREEN.
- Code disposition: no Rust source was changed. Dirty sound plugin feature and manifest files belong to other sessions and were preserved.

## Exact scope

| module folder | files | lines | tests | current-source fingerprint |
|---|---:|---:|---:|---|
| `zircon_runtime/src/core/framework/sound` | 28/28 | 2,114 | 8 | `2b8efa191874ebc29fb19dbfe3d230d18306a0501f4ea6de064b81c9a2482656` |

All 28 Rust files were read. Consumer tracing also followed the complete five-file `zircon_plugins/sound/runtime/src/timeline` and five-file `dynamic_events` directories plus the relevant service, engine-state, Kira-device and package-option paths. This does not claim complete static coverage of the sound plugin crate.

## Confirmed bottlenecks

1. Dynamic event admission stores owned invocations in an unbounded `Vec`. Submission clones the complete catalog while holding the manager mutex, then linearly scans descriptors. Dispatch drains every pending event without a count/time/byte budget, clones the complete handler list, scans and sorts matching handlers per invocation, and clones the invocation including `payload: Vec<u8>` once per handler.
2. Dynamic event execution clones the executor map, then invokes arbitrary executors synchronously on the caller after releasing the manager mutex. Lock release avoids a direct re-entrant deadlock, but a slow handler still blocks the frame/caller and there is no timeout, cancellation, affinity, late-result or observer-stall contract.
3. Timeline advance holds the single sound state mutex across all scheduled sequences, tracks, curve validation/sampling and target application. Every tick `mem::take`s the sequence vector, builds new retained/report vectors, validates every curve again, linearly scans keyframe windows, clones each binding and builds per-track sample/application vectors. `timeline_sequences()` separately deep-clones all complete sequences.
4. `mixer_snapshot()` holds the state mutex while cloning the complete graph and then rebuilding/cloning all source descriptors, automation bindings, dynamic event catalog, meters and ray-tracing status. The planned M5 mixer console can turn this control-plane capture into a high-frequency O(graph + sources + bindings + events + meters) poll.
5. The framework still exposes owned `SoundMixBlock` return contracts. Current Kira production consumers explicitly return `UnsupportedAdvancedFeature` for manual mix rendering and backend callback pulls because Kira owns the callback. Therefore the owned sample block is not reported as a current audio-callback allocation; the obsolete contract must be hard-removed or constrained to an explicit offline/control boundary before another backend can accidentally make it real-time.

## Current-source correctness boundary

- `SoundPluginOptions::default().backend` and the sound package manifest both use `kira-cpal`.
- `zircon_runtime/src/core/framework/sound/tests.rs` still asserts `software-mixer`. This is a statically certain unit-test expectation mismatch, but no Cargo result is claimed because no managed sound gate ran in this review.
- Device enumeration is synchronous through CPAL, but it is currently an explicit control/editor query. No product trace proves frame-frequency use, so it is not promoted to a separate performance task.

## Reference implementation checks

- Godot `dev/godot/servers/audio/audio_server.cpp` allocates mix, temporary and per-bus channel buffers in `init_channels_and_buffers()` and reuses them in `_mix_step()`.
- Fyrox `dev/Fyrox/fyrox-sound/src/engine.rs` renders into the output callback's borrowed slice; its bus ping-pong buffers only resize when capacity is insufficient and otherwise clear/reuse storage.
- Bevy `dev/bevy/crates/bevy_audio/src/audio_source.rs` keeps encoded source bytes in `Arc<[u8]>`, supporting a single immutable payload owner across consumers.
- These are boundary references, not wholesale designs: Zircon keeps Kira as the only production audio backend and must use Kira handle/command semantics for the real-time path.

## Plan links and acceptance

- `PERF-MVP-586` / Plugins02 M5: compile event-id to stable pre-sorted handler/executor slots; bound pending entries, bytes and age; share immutable invocation payload; drain by count/time/bytes; execute through explicit-affinity bounded tickets with timeout/cancel/late-result semantics.
- `PERF-MVP-587` / Plugins02 M5-T1: compile timeline bindings and validated curves at schedule time; keep stable active-sequence storage and curve cursors; reuse scratch; make detailed reports opt-in; keep manager lock independent of sequence/track/keyframe work.
- `PERF-MVP-588` / Plugins02 M5-T2: split immutable graph generation from high-rate meter samples; stable polling is O(1) handle/cursor work, while complete owned capture is explicit and budgeted; retire manual mix/callback contracts from the production service boundary.
- Dynamic matrix: events/handlers/sequences/tracks/keyframes/sources/meters `1/100/10k`, payload `0/1KiB/1MiB`, executor delay `0/1ms/2s`, observer stall `0/60s`, polling `0/30/120Hz`. Record caller blocked time, mutex wait/hold, scans/sorts, queue entries/bytes/age/drop, clone bytes, allocations, report/snapshot builds, RSS, underruns and p95.

## Static gates executed

- Read 28/28 framework Rust files and traced the listed production consumer paths.
- `rustfmt --check --edition 2021` passed for all 28 framework files.
- Scoped `git diff --check` passed.
- No Cargo, WPR/ETW product trace or audio callback counter ran. RenderDoc is not applicable to this non-rendering sound slice. Until the stale default test is corrected and the current-source managed gate plus scale/real-time evidence are GREEN, the module remains pending.
