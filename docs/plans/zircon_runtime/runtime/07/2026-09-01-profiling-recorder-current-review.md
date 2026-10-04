# Runtime07 active profiling recorder current-source review

Date: 2026-09-01

Status: `architecture_reviewed / capture_epoch_isolation_source_complete / capture_config_hard_limits_source_complete / current_source_release_baseline_queued / production_parallelization_not_started`

## Current architecture and bottleneck

The profiling feature has a correct inactive fast gate and bounded `VecDeque` retention, but every
active producer converges on `GLOBAL_RECORDER: OnceLock<Mutex<ProfileRecorder>>`. One completed
static span takes one global lock at begin and two more at completion (timestamp, then append).
The same path allocates owned name/path/stream/category strings while contending for that lock.
Frames use one begin lock and two completion locks; counters use one lock, with batches only
amortizing counters emitted by one caller.

This is a structural serialization point, not a `VecDeque` or mutex implementation detail. With
`T` producer threads and `E` completed spans, admission remains `Theta(T * E)` operations through
one owner lane. Snapshot/export also deep-clones the retained rings while holding that same lane.

The review also found a correctness defect that must precede parallel recording: scope and frame
tokens do not carry the capture epoch. A token begun before reset/restart can complete into the new
capture, and reset span IDs can make a retired token remove a current stack entry with the same ID.
The focused REDs are
`scope_completion_from_a_retired_capture_epoch_does_not_enter_the_new_capture` and
`frame_context_from_a_retired_capture_epoch_does_not_attach_to_the_new_capture`.

Pre-repair current SHA-256 inputs:

- `profiling/mod.rs`: `8978ddcb3194d07d44d1dbbbe4344f0c6bcd419943fe63738a02833862a8fc12`
- `profiling/recorder.rs`: `eac7abde1ab7b05a709f4bc8ffdd224dc31c7443189fb2a61feb537b61362cf8`
- `profiling/scope.rs`: `54becbe0134a932d7f4d5b63f03cc71a17af0775fc54bfefab25d1ca7712ffbc`

## Capture epoch correctness slice

The source repair is complete before recorder parallelization:

- start/stop/reset update capture state while still owning the recorder control lock, closing the
  old-recorder/new-epoch transition window;
- scope, frame and propagated frame-context tokens carry their capture epoch; stack removal uses
  `(epoch, id)` so reset ID reuse cannot remove a current entry;
- begin/counter admission rechecks the epoch while holding the recorder lock, and retired
  completion detaches its own local stack entry but does not publish into the current capture;
- scope/frame completion now computes timestamp and appends under one recorder lock instead of
  two, without adding payload work to that critical section.

Current post-repair SHA-256 is
`profiling/mod.rs = 92b911ecd2ca4e6e5467d16fd6f64c26c988aaff3206dbe63eaca584b928fe84`
and
`profiling/scope.rs = ea2045188012f17a520ea7e1cef8b2823aee65f8b3784b8a88037bd212ad8f43`.
Rustfmt, scoped diff checks and locked Cargo metadata pass. Managed behavior validation remains
required before this slice is accepted. Request
`runtime07-profile-recorder-epoch-isolation-20260901-r1` produced ticket
`7f1ffd70b39346609d83b535e25a59ab`; its initial state is `queued` and is not polled.

## Capture configuration resource limits

The dynamic-runtime ABI previously normalized zero capacities but accepted `usize::MAX` for each
retained sample class. Because the current recorder owns in-memory rings and snapshot/export
deep-clones them, this was an external configuration path to unbounded entry retention. Invalid
non-finite and non-positive frame budgets were already repaired in current source; the same
contract still accepted arbitrarily large finite values.

The source contract now clamps requested values to explicit effective maxima:

- frames: default 512, hard maximum 4,096;
- spans: default 16,384, hard maximum 131,072;
- counters: default 4,096, hard maximum 32,768;
- frame budget: default 16.67 ms, hard maximum 60,000 ms.

`ProfileRecorder` already stores the normalized config, and its retention snapshot reports those
effective capacities, so no compatibility fallback or second runtime limit exists. Focused tests
cover zero/default recovery, `usize::MAX`, `f64::MAX`, NaN and both infinities. The current
SHA-256 values are
`zircon_runtime_interface/src/profiling.rs = cbe91e1829c75414eb491c4943f9912c6ee8987e957401d8a83ea0da2fbcf52e`
and
`profiling/mod.rs = acb7be28f21ab4fb397bc60fd2fe4868302ebae78127c3d5fc0cbf5a278cb699`,
with the runtime effective-capacity regression in
`profiling/recorder.rs = 266ef5d0245f6a6dc1e5440ee55fcedb34c56a50ebc5cfb6a87e0fa8e652e2b7`.
This closes the ABI entry-count amplification surface only. Per-metadata field length, aggregate
retained-byte accounting, bounded export pages and interned static metadata remain acceptance
requirements for the producer-lane architecture; no claim is made that entry caps alone bound all
capture memory.

Focused managed validation was submitted without polling:

- interface normalization: ticket `6329ad3f7ca2414ca091dd0c64f2408a`, manifest
  `014ef703fb5f1b819d098f0d797c4f19686ea21f1b8a99d6f9b55dbc3ab1618d`;
- runtime effective capacity: ticket `0a4a99145f894cc8b1ff26ab80211df2`, manifest
  `5d30f78f6e05432094aeb0f0568acc312d1f4f2f4d9038f54b565ad2992edd55`.

Both receipts were initially `queued`; neither receipt is treated as terminal validation evidence.

## Reference boundary

Unreal TraceLog is the primary reference. `Trace/TlsBuffer.cpp:38-53` gives each producer a TLS
write buffer and publishes new thread lists atomically; `Writer_DrainBuffers` snapshots the
available per-thread buffer chains before draining them. `Writer.cpp:1054-1167` owns periodic
drain/flush on one trace worker. Per-event producers therefore do not serialize through the writer
or perform output work. Buffer retirement and thread exit are explicit lifecycle operations.

Zircon does not need to clone Unreal's byte protocol, but it needs the same ownership split:
capture control/configuration, producer-local bounded recording, and one generation-qualified
merge/retention authority. A fixed array of per-event mutex shards is not the final design because
it retains lock/format overhead per sample and complicates the global retention cap.

## Baseline and acceptance

`runtime_profiling_recorder_performance.rs` records the current source at 1/8/64 threads and
0/100/10,000 spans per thread. It reports P50/P95 elapsed time, allocation count/bytes and retained
span count under `RUNTIME07_PROFILE_RECORDER_BASELINE_V1`. The zero-event lane isolates worker and
barrier overhead. The ignored release run must be coordinator-managed on an E/F artifact root;
no current-source timing is claimed until that ticket is terminal.

The baseline request is `runtime07-profile-recorder-baseline-release-20260901-r1`, ticket
`aca4f3bf3d9a4097aefe4a38d237788b`, source-manifest SHA-256
`fe6e1c19f4a80bfa26d255e899aabec416fe40b1821547070c1d2c788f4073ed`. Its initial state is
`queued`; it is not polled and is not performance evidence.

The production hard cut begins only after the baseline is available:

1. stamp scope/frame/context records with capture epoch and reject retired completion without
   touching the current capture;
2. replace active per-event global locking with bounded producer-local buffers and a single merge
   authority; preserve global newest-N retention and observable drops/overwrites;
3. keep static labels allocation-free on the producer path, while dynamic labels use an explicit
   bounded payload path;
4. snapshot/export seals a generation, merges outside producer lanes, and never holds a producer
   lock while cloning or serializing the full capture;
5. prove exact epoch isolation, nested parent/frame identity, newest-N ordering, reset/stop/thread
   exit behavior, bounded memory, and diagnostics-disabled zero work.

Acceptance requires current-source before/after 1/8/64-thread data, allocations and retained/drop
counts, plus WPR CPU/lock-wait and process RSS/power evidence. A microbenchmark alone cannot claim
Unreal parity or close the Runtime07 observability Failure.
