# Frameworks01 M1 `zr_diagnostics` current-source preflight (2026-08-30)

## Status

- `preflight_complete`
- `physical_hard_cut_not_started`
- `zr_kernel_predecessor_missing`
- `diagnostic_capability_runtime_assembly_boundary_locked`
- `current_source_profile_required_before_algorithm_optimization`
- `milestone_not_accepted`

This is an architecture and measurement-admission record. It does not claim a crate migration,
runtime validation, performance improvement, power reduction, reference-engine parity, or milestone
acceptance. Coordinator plan authorization request `90bfbf48352e4a5ab6a59a0b9f83f672` allowed this
child-plan path; the parent numbered plan remains protected from ordinary business-session writes.

## Current Source Snapshot

The review used shared main HEAD `cc5cadbd597c3707954ebd6109fad0fd5643a152`. The current
`zircon_runtime/src/diagnostic_log` tree contains 32 Rust files, 4,548 lines, and 149,744 bytes.
The path-sorted `relative-path<TAB>lowercase-file-sha256` manifest has SHA-256
`3bc2c668cbdf0e6057250b24a811e4f421b8daf43f9d0cf6dddf8eb2e6e44392`.

The current `zircon_runtime/src/core/runtime/diagnostics` tree is materially larger: 80 Rust files,
13,593 lines, and 436,379 bytes. Of those, `render_stats_store` accounts for 55 files/8,737 lines and
`profiling` for 16 files/3,802 lines. A lexical dependency audit finds no production imports of
`core::manager`, asset, scene, editor UI, render-graph, WGPU, Naga, or Glyphon code. It does find 53
files consuming `core::framework` render-stat contracts and eight files consuming
`zircon_runtime_interface`. Therefore this tree is part of the physical crate boundary review; it
cannot be excluded merely because its current path is under `core/runtime`.

Coordinator ownership matrix request `710aea8f6a90405d9dacccfdbcb38c45` reports every current
diagnostic-log blob as dirty and without a live lease. Existing attributions are either missing or
refer to stale, archived, or cancelled Sessions. This makes the tree eligible for a future explicit
scope rotation, but it is not current ownership attribution and does not authorize this Session to
edit the production tree.

A tracked working-tree lexical inventory finds 52 Rust files that directly name
`crate::diagnostic_log` or `zircon_runtime::diagnostic_log`; 39 are outside the implementation tree.
It separately finds eight `runtime_diagnostics` consumers, with one file in both sets. This is only a
lower-bound review inventory. The physical hard cut must rebuild a structured use-tree plus literal
consumer manifest over tracked and nonignored untracked Rust inputs before any move.

Neither `zircon_runtime/crates/zr_kernel` nor `zircon_runtime/crates/zr_diagnostics` exists. The
parent DAG requires `zr_math/zr_resource -> zr_contracts -> zr_kernel -> zr_diagnostics`, so creating
`zr_diagnostics` first would violate the planned foundation order even though most logger files have
few dependencies.

## Current Architecture Review

The current source contains three different domains:

1. `diagnostic_log::{level,settings,platform,sink,timestamp}` owns process logging policy, compiled
   scope filters, location selection, the bounded producer queue, the worker, batching, console/file
   output, flush/shutdown, and sink metrics. Production dependencies are `std`, `chrono`, `arc-swap`,
   and `crossbeam-channel`.
2. `core::runtime::diagnostics` owns diagnostic DTOs and mutable store state, profiling capture and
   export, render-stat stores, and mostly pure snapshot projectors. Its file export path is a
   diagnostics capability, not Runtime manager resolution. These components are future
   `zr_diagnostics` candidates once the framework contracts they consume have a legal low-layer
   owner.
3. `runtime_diagnostics` resolves render, physics, animation, and other registered services through
   `CoreHandle`. Current `core::runtime::diagnostics::devtools` is also not purely low-level: it locks
   `CoreRuntimeInner.modules`, `services`, and the built-in devtools catalog. Manager/service
   collection and this devtools assembly projector belong to the Runtime facade even when their DTOs
   live in `zr_diagnostics`.

Moving the Runtime assembly domain into `zr_diagnostics` would recreate an upward dependency from a
low-layer crate to the facade. Moving `CoreRuntimeInner` into `zr_kernel` while it directly stores a
`DiagnosticStore` would create the opposite cycle once `zr_diagnostics` depends on kernel contracts.
The facade assembly must therefore remain above and depend on both crates; kernel algorithms must
not own the diagnostic store.

The current hot-path algorithms are:

- `CompiledDiagnosticLogFilter` builds a byte-prefix trie once. Construction is proportional to the
  total rule-prefix bytes; lookup is proportional to the visited scope-prefix bytes and does not
  scan every configured rule.
- An accepted record owns a copied `scope: String` and `message: String`, then enters a bounded
  crossbeam channel. Debug/log records fail fast when the queue is already full; warn/error records
  wait for the configured bounded timeout. The two copies are a measurement target, not yet a proven
  product bottleneck.
- The worker batches by record count, estimated bytes, or deadline and writes one contiguous byte
  buffer to configured outputs. Work is proportional to the records and output bytes in the batch.
- `LogRecord` stores an enqueue `Instant` only for queue-age metrics. The rendered wall-clock
  timestamp is generated once inside `flush_pending`, so every record in a delayed batch receives the
  flush timestamp rather than its event/enqueue timestamp. That is an observable ordering/diagnostic
  semantics defect under backlog, not a formatting micro-optimization.
- Library-unload shutdown waits for active senders and worker closure with `thread::yield_now` loops.
  Its CPU/energy impact under contention is unmeasured and must not be optimized from intuition.

The existing ignored `PERF-MVP-434` matrix covers 54 combinations: three requested rates, two caller
counts, three scoped-rule counts, and three output delays. It records caller P95, RSS, queue depth and
age, drops, batching, output calls, malformed/duplicate records, and shutdown state. It does not
record allocation counts, bytes allocated per accepted/dropped call, event-to-rendered timestamp
error, worker CPU time, disk I/O cost, or power data. Its 50 ms caller-P95 and 128 MiB RSS thresholds
are safety budgets, not reference-engine performance parity targets.

## Reference-Engine Findings

Unreal Engine is the primary reference:

- `Core/Public/Logging` separates category/verbosity and suppression policy from output devices.
- `FOutputDevice` defines serialization, flush, teardown, thread-safety, and panic-thread capability.
- `FOutputDeviceRedirector` owns the multi-producer buffering, primary logging thread, output-device
  fanout, flush fencing, backlog, and panic transition. Its buffered line carries an event time; it
  does not assign one wall-clock value to an entire later flush batch.
- `Logging/LogTrace` projects log category/message metadata into the Trace system as a separate
  channel instead of making the output transport own runtime profiling collectors.

Unreal's module boundary also keeps lifecycle contracts and ordered startup/shutdown in Core while
the module manager owns loading and assembly. That supports separating stable lifecycle/task
contracts from Runtime's registry-resolving collectors rather than treating the current
`core/runtime` directory as one indivisible kernel owner.

Bevy's independent `bevy_log` crate cross-checks the physical owner and curated plugin/configuration
surface: runtime filters and subscriber layers belong to the logging facility, while application
state collection remains outside it. Fyrox's simple global mutex logger is useful as a small-engine
comparison but is not adopted because it serializes producers through one synchronous owner.

These references support a diagnostics capability crate plus higher Runtime adapters. They do not
support moving `CoreHandle`, `CoreRuntimeInner`, render/physics/animation manager resolution, or
module/service assembly into `zr_diagnostics`.

## Locked Hard-Cut Shape

After `zr_kernel` is physically present and accepted, the `zr_diagnostics` batch must:

1. Move level/filter contracts, compiled filtering, settings, platform log-path selection,
   timestamp primitives, sink lifecycle, bounded queue, worker, output durability, and sink metrics
   into `zircon_runtime/crates/zr_diagnostics` as the only implementation owner.
2. Move diagnostic DTOs, diagnostic store state, profiling capture/export, render-stat stores, and
   pure store/snapshot formatting into `zr_diagnostics` after their framework contracts are first
   available from `zr_contracts`/`zr_kernel`. `diagnostic_log::diagnostics` may be absorbed because it
   formats the same crate-owned store; it must not retain an old implementation owner.
3. Keep `runtime_diagnostics` service/manager collectors and the `CoreHandle`-reading devtools
   assembly projector in the Runtime facade. Keep `CoreRuntime`, `CoreHandle`, `CoreWeak`, and
   `CoreRuntimeInner` as facade assembly above `zr_kernel` and `zr_diagnostics`; the new diagnostics
   crate must not import them or manager resolvers.
4. Preserve `zircon_runtime::diagnostic_log` as a curated external product surface for App and Editor
   consumers while deleting the old internal implementation in the same atomic batch. Do not add an
   old-owner module, implementation alias, wildcard projection, compatibility shim, or direct
   App/Editor dependency on `zr_diagnostics`.
5. Capture event time when a record is admitted and format it on the worker. Preserve monotonic
   enqueue time separately for queue-age metrics. A timestamp semantic fix must be tested across a
   delayed multi-record batch before any allocation optimization is attempted.
6. Regenerate the full consumer/move manifest and update manifests, lockfile, Runtime root/facade,
   prelude, tests, guards, and documentation atomically through the coordinator. Root and Runtime
   Cargo manifests are currently owned by another Session and must not be rewritten opportunistically.

## Profiling And Optimization Gate

Algorithm optimization is not admitted until an exact current-source Windows managed build is
GREEN and a baseline is captured outside the C drive. The baseline and post-change run must use the
same source manifest, build profile, host, output fixture, rates, caller counts, filters, queue
capacity, batch limits, and sink delays. Required evidence is:

- caller accepted/drop P50/P95/P99 and achieved rate;
- allocations and allocated bytes per attempted, accepted, and dropped record;
- enqueue-to-dequeue age plus event-time-to-rendered-time error;
- queue peak, drops by level, critical backpressure, batch size, writes, flushes, and syncs;
- worker/process CPU time, peak/after RSS, and file bytes/I/O latency;
- ETW/WPR scheduling and I/O evidence, plus available system power/energy counters when the host
  exposes reliable data.

Only a measured dominant cost may select the implementation direction. Candidate changes such as
scope interning, borrowed/static category IDs, reusable batch buffers, event timestamp encoding, or
event/fence-based shutdown remain hypotheses until that profile. Reference-engine timing or energy
parity and an "optimal scale" claim require comparable workload definitions and measured data; this
record makes neither claim.

## Admission Boundary

No production or test file was changed for this preflight. The next executable Frameworks01 work
remains the final managed GREEN for the already implemented `zr_resource` path-normalization fix.
`zr_diagnostics` implementation stays behind the `zr_kernel` physical predecessor, fresh ownership
rotation, complete current-source consumer manifest, managed baseline, and exact manifest/Cargo
coordination gates.
