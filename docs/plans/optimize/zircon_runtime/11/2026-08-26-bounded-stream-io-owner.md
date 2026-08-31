# Runtime11 Bounded Stream I/O Owner Architecture Record

- Date: 2026-08-26
- Owner plan: `docs/plans/zircon_runtime/runtime/11-job-system-task-model.md`
- Failure: `docs/plans/zircon_runtime/runtime/11/failure-2026-08-10-blocking-io-process-output-budget.md`
- Status: Runtime owner source and focused structure audit implemented; managed Cargo, Editor14 migration, and product performance evidence pending

## Scope

This slice establishes the reusable Runtime11 owner for bounded subprocess
stdout/stderr capture. It does not edit the concurrently changing Editor Play
backend. Editor14 remains responsible for terminating the child process before
waiting for blocked pipe readers and for formatting typed Runtime records into
editor diagnostics.

## Current-Source Baseline

`zircon_editor/src/core/play/process_backend/output.rs` already bounds one Play
session with:

- an 8 KiB read chunk;
- a 64 KiB retained line limit;
- 1,024 queued entries and 4 MiB of queued record storage;
- 64 records, 256 KiB, and 2 ms per live drain;
- truncation, drop, and oldest-age diagnostics.

The 2026-08-26 Editor07 streaming-decode optimization removed the temporary
`Vec<DecodedOutputLine>` formerly created for every read chunk. Its deterministic
allocation delta is one temporary line collection per chunk to zero, while the
managed Windows P95 values remain queued.

That local repair does not close Runtime11:

- `spawn_reader` still creates two private OS threads per Play process, so
  reader count is `O(active Play sessions)`;
- those threads are absent from `ExecutionRuntime::worker_inventory`,
  `ExecutionScopeCensus`, cancellation, and shutdown accounting;
- dropping a `JoinHandle` can detach a blocked reader from runtime lifecycle
  truth;
- every future process consumer can copy the queue and budget policy.

## Reference Evidence

### Unreal Engine

`dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Misc/MonitoredProcess.h` and
`Private/Misc/MonitoredProcess.cpp` bind pipe reads, cancellation, final residual
output, process termination, and monitor-thread join to one
`FMonitoredProcess` owner. Cancellation terminates the process first, drains the
final pipe bytes, closes the pipe, and only then publishes the terminal event.

Zircon follows that lifecycle order. It deliberately does not copy Unreal's
unbounded `FString OutputBuffer` or one-thread-per-process scaling.

### Fyrox

`dev/Fyrox/fyrox-build-tools/src/build.rs` starts one private thread for each
stdout/stderr pipe, reads lines through `BufReader`, appends them to a shared
unbounded `String`, and clones the complete log into UI updates. This confirms
the common Rust integration shape but also reproduces the ownership, memory,
and thread-budget failure that Runtime11 must remove.

### Zircon Foundation

`zircon_runtime/src/core/runtime/tasks/execution/` already owns fixed
`Compute`, `AsyncCompute`, and `Io` worker domains, scoped task admission,
cooperative cancellation, task census, and deadline-bounded shutdown. The
missing capability is a streaming decoder/queue contract that consumes that
foundation, not another worker owner.

## Chosen Architecture

1. `BoundedStreamIoLane` creates a dedicated `ExecutionScope` and submits every
   reader as an `Io` task. It never calls `std::thread::spawn`.
2. A lane-wide atomic reader admission cap is the smaller of the configured
   cap and the physical Runtime `Io` pool parallelism. It rejects excess
   readers before task execution. A start gate makes multi-reader session
   admission all-or-abort: no stdout reader consumes bytes while stderr
   admission is still fallible, and accepted pipe readers are never left
   permanently queued behind another reader from the same capture.
3. Each capture owns one `Mutex<VecDeque<Record>>`. Enqueue is `O(1)` and
   guarded by both entry and retained-byte limits.
4. Each worker owns one fixed read buffer and one line decoder. Decode is
   single-pass `O(input bytes)` and emits directly into the queue without a
   per-chunk line vector.
5. Records retain typed stream identity, text, truncation count, lossy UTF-8
   state, retained bytes, and capture time. Formatting remains a consumer
   concern.
6. Drain is FIFO and bounded by record count, rendered/retained bytes, and wall
   time. It reports oldest observed age without rescanning the queue.
7. Cancellation is cooperative between blocking reads. Editor14 must terminate
   the child and close its pipe handles before waiting. If a platform read does
   not unblock, the execution scope remains non-quiescent and runtime shutdown
   returns a timeout instead of claiming cleanup or detaching an untracked
   thread.

## Complexity And Memory Bounds

For `R` admitted readers, line limit `L`, read chunk `C`, queue byte limit `B`,
and queue entry limit `E`:

- decode time: `O(total input bytes)`;
- enqueue: amortized `O(1)` per completed line;
- drain: `O(min(queued entries, count/byte/time budget))`;
- reader working memory: `O(R * (C + L))`;
- retained session memory: `O(min(B, E * (L + record overhead)))`;
- physical reader threads: no new threads; bounded by the Runtime `Io` worker
  domain;
- queued/running reader tasks: bounded by the lane reader limit and execution
  scope capacity.

The runtime owner therefore changes reader-thread growth from
`2 * active Play sessions` to the already configured fixed `Io` worker count.
Exact CPU, P95, RSS, and power comparisons remain a managed product gate; this
record makes no timing or power claim.

## Test Matrix

- line order across chunk boundaries, CRLF normalization, unterminated tail;
- invalid UTF-8 replacement and typed lossy diagnostic;
- 64 B, 1 MiB, and synthetic 1 GiB-equivalent unterminated lines without
  retaining the discarded tail;
- entry and byte admission, exact drop/truncation counters, and byte release;
- count/byte/time drain limits and oldest-age reporting;
- stdout/stderr identity and final residual output;
- interrupted reads retry without losing decoder state or becoming failures;
- all-or-abort multi-reader admission;
- reader admission cap across concurrent captures;
- cancellation after producer close, terminal wait, and residual drain;
- blocked-reader shutdown remains non-quiescent rather than falsely terminal;
- a Runtime with one physical `Io` worker rejects a two-pipe capture before
  either pipe is read;
- source guard rejecting `thread::spawn` in the Runtime stream owner.

## 2026-08-26 Focused Static Evidence

- Scoped Rust formatting and check passed for the eight `bounded_stream_io`
  files and the touched Runtime11 Rust audit owners.
- `test_runtime_job_system_has_no_editor_dependency` and
  `test_runtime_task_diagnostic_structure_is_current` passed 2/2 in 15.965 s.
- `job_system_boundary` reports `expected_module_count = 14`,
  `behavior_test_anchor_count = 58`, all missing module/declaration/public/API/
  behavior lists empty, `oversized_modules = []`, and
  `runtime_editor_dependency_references = []`.
- The aggregate three-test audit remains red only because the pre-existing
  `graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command_list/builder.rs`
  direct-Rayon path is outside the Runtime11 whitelist. It reports two risks;
  this slice does not bless that path or alter its whitelist.
- No Cargo test, product P95/RSS comparison, or power measurement ran in this
  bypass slice. No such result is claimed.

## Remaining Acceptance

- Prove in a source-bound product run that long-lived stream readers do not
  starve unrelated work on the shared `Io` pool. The current lane prevents
  over-admission beyond physical parallelism but does not reserve an `Io`
  worker for unrelated tasks; if contention is material, Runtime11 must add a
  separately inventoried blocking-stream worker domain or a platform
  multiplexing backend before Editor14 cutover.
- Migrate Editor14 Play output to this owner after its current output changes
  are integrated.
- Run managed Runtime focused tests and Editor upward tests.
- Measure 1/1K/1M lines, 64 B/1 MiB/1 GiB unterminated input, 30/120 Hz polls,
  concurrent sessions, stop/drop, P95, RSS, and process power.
- Compare the same source-bound build before and after migration; only then
  claim that the original PERF-MVP-552 bottleneck is removed.
