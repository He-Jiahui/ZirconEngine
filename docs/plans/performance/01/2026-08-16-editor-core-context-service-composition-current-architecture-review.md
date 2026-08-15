# Editor core context service-composition architecture revalidation

## Status

- Result: `static_complete / source_blocked / dynamic_pending`.
- Review date: 2026-08-16.
- MVP priority: P0. `EditorContextBuilder` is the editor F0 composition root used by the product
  `EditorManager`; its current constructor call is not type-correct.
- Owners: Editor00/01 own the composition boundary; Editor17 owns logging/recovery services;
  Editor14 owns the single job scheduler; Editor02 owns message delivery; Editor08 owns tool
  scheduling. Render17 owns F0/F4 product measurement after the source and launcher blockers clear.
- Accounting: retain `zircon_editor/src/core/context/**` in `pending.md`. Do not add it to
  `review.md` before the constructor failure, current managed tests and the F0 matrix below pass.
- Code disposition: no Rust source changed. Three tracked files are foreign modified and
  `builder/quota_startup_tests.rs` is foreign untracked; this session owns performance documents only.

## Exact scope

| scope | files | physical lines | tests | ordered path-and-raw-content SHA256 |
|---|---:|---:|---:|---|
| `zircon_editor/src/core/context/**` | 5/5 | 1,380 | 14 | `3ff94e5bbefae8f0f88b10f5a80b3e007f128af62d7a804b8193af7967b294c4` |

The fingerprint is SHA256 over sorted normalized path, NUL, raw file bytes, NUL. Every current Rust
file was read in full. The call graph was followed through product `EditorManager` construction,
retained autosave/shutdown consumers, job and notification tests, event sinks, settings startup,
tool publication and the editor message bus. This supersedes the 2026-07-30 context fingerprint
`08532c...`; the tools submodule was not re-audited here because its source remains owned by the
separate 4-file report.

## Per-file acceptance record

| file | current-source verdict |
|---|---|
| `builder.rs` | One explicit composition root is correct, but the current call omits autosave and passes a non-shared log value to the new context signature. F0 settings I/O, quota resolution, service construction and wiring form one unmeasured serial block. Log/i18n sinks allocate JSON and invoke message fanout synchronously. |
| `builder/quota_startup_tests.rs` | Verifies settings-derived job limits across two contexts, but does not compile-prove the new autosave/log composition or assert single scheduler ownership. |
| `editor_context.rs` | A typed aggregate with service-owned synchronization is preferable to a global service locator. Accessors are constant-time and mostly borrowed/shared. Its foreign migration introduced the unmatched log/autosave constructor contract. |
| `mod.rs` | Export shell only; no independent runtime work. |
| `tool_scheduler.rs` | Mutation releases its mutex before bus publication and queue scale is bounded. It still reparses the static tool topic and clones publication data per operation; retain the existing Editor08/PERF-MVP-019 route. |

## Immediate source-integrity blocker

`EditorContext::new` now requires `Arc<EditorLogService>` followed by `EditorAutosaveService`
(`editor_context.rs:38-52`). `EditorContextBuilder::build` creates `EditorLogService::default()` and
passes no autosave argument (`builder.rs:296-332`). Starting at the log parameter, the call therefore
has a type mismatch and all arguments after notifications are shifted by one position.

This is an overlapping migration, not a performance result. The minimum owner repair is:

1. create the log service once behind `Arc` and configure that same instance;
2. construct one `EditorAutosaveService` from a clone of the already-created `EditorJobSystem` handle
   and the Editor17-owned policy, never from a second scheduler or job authority;
3. pass autosave between notifications and transactions and publish the context only after all
   services are constructed and wired;
4. add a focused composition test proving log identity, autosave admission through the context job
   system, bounded progress visibility and deterministic shutdown.

The cross-plan failure record is
`failure-2026-08-16-editor-context-autosave-construction-drift.md`. A blind argument insertion is not
enough: the recovery review still requires project-generation fencing and the current autosave poll
method has no retained-tick caller.

## Structural performance verdict

### P0: F0 is a serial, uninstrumented service transaction

The builder registers settings and job quota definitions, synchronously loads the user layer from a
store/environment, resolves worker limits, creates eleven service owners, configures sinks and locale,
then publishes one context. This is dependency-ordered startup work, not a frame loop, so the answer
is not indiscriminate multithreading. The defect is that no stage receipt attributes configuration
I/O, validation, construction, callback wiring or publication latency and no rollback/shutdown record
identifies the last completed stage.

Define one source-bound `EditorStartupAssembly` contract with these measured stages:

1. `Definition`: parse/validate settings and resolve quotas without publishing mutable services;
2. `CoreOwners`: create the one bus, runtime gateway, scheduler-backed job system and notification
   projection in dependency order;
3. `ServiceOwners`: create logging, i18n, recovery, transaction, command and tool owners exactly once;
4. `Wiring`: attach sinks/subscribers using stable shared handles, with no callback during partial
   publication;
5. `Publish`: atomically expose one complete immutable context and a startup receipt;
6. `Activate`: project/recovery/plugin activation occurs after publication under their own generation
   and cancellation contracts.

Each stage records wall/CPU, allocation bytes, blocking I/O, worker starts, message publications and
failure. Only independent blocking I/O may move to Runtime11, and only with an explicit dependency,
deadline and cancellation rule. Keep `EditorContext` as a typed aggregate; do not replace it with a
runtime service map, reflection registry or editor-private executor.

### P0: composition currently duplicates downstream event cost

Log and locale sinks create owned JSON for each delivered fact (`builder.rs:61-82,134-153`) and call
the shared message bus on the producer path. Dropped delivery verification tests membership of every
dropped subscriber in the delivered slice (`builder.rs:168-176`), which is `O(D*R)` at the exact
backpressure point. These are not context-local queues:

- route message construction/fanout, zero-target fast paths and resync receipts to PERF-MVP-019;
- route log ingress/persistence/visible generations and removal of the unused duplicate JSON
  projection to proposed PERF-MVP-644;
- route transaction event separation to PERF-MVP-646;
- keep locale generation/delta ownership in Editor13 and settings persistence in Editor17.

The assembly contract should count publications by service, but must not introduce another bus.

### P1: bounded tool work should remain simple

The tool service owns fixed resource families and bounded queues, releases the scheduler lock before
publication and currently has no product acquire/release consumer. Cache the built-in topic once per
service only when its owner source is stable. Measure the bounded `release_all` path before replacing
the small queue; this module does not justify a general task graph.

## Unreal source evidence

- `dev/UnrealEngine/Engine/Source/Runtime/Launch/Private/LaunchEngineLoop.cpp:1734-1748` separates
  `PreInitPreStartupScreen`, starts boot profiling and wraps the phase in `SCOPED_BOOT_TIMING` before
  initializing thread-sensitive logging.
- The same source at `4880-4895` gives `FEngineLoop::Init` explicit boot timing, load-time cycle stats,
  memory scope and progress; at `6950-6958` plugin loading is a named, ordered phase after config and
  before dependent subsystems.
- `dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/EditorEngine.cpp:1307-1377` measures editor
  initialization as a load-time scope, makes ordering dependencies explicit, then enters
  `InitEditor`; `965-987` separately establishes editor providers, derived-data workers and base
  engine initialization.

The applicable standard is explicit ordered phases plus attribution, not copying Unreal globals or
starting one thread per service. Zircon's typed context can remain smaller while adopting phase
receipts, one-owner construction and measured activation boundaries.

## Acceptance matrix

| gate | matrix | required result |
|---|---|---|
| composition | default/store settings; valid/missing/invalid user layer; workers `1/4/16`; forced failure at every stage | exactly one owner per service; one job authority; no partial context publication; deterministic rollback and terminal receipt |
| scale | subscribers `0/1/100/10k`; startup messages `0/10/10k`; settings `10/1k/10k`; payload `64B/2MiB` | count/bytes/age bounded; zero-target constructs no delivery; startup stage work scales with admitted definitions/events only |
| F0 product | cold/warm startup, no project and MVP project, 31 measured runs after warmup | stage wall/CPU/allocation/I/O/worker/message counters plus WPR CPU, waits, wakeups, file I/O, RSS and package power; no unattributed gap |
| F4 product | idle/edit/autosave/close | stable context assembly work `0`; no second scheduler/bus/log owner; bounded autosave shutdown and generation fence |
| parity | same machine/configuration against the checked-in Unreal reference workflow | report distributions and configuration; no parity claim without comparable traces and confidence intervals |

RenderDoc is not applicable to this CPU/service-composition slice. WPR/xperf is mandatory after the
managed editor launcher and current source build are green. No trace or build artifact was written to
C:.

## Static gates executed

- Read 5/5 current context Rust files, all 14 tests and the product/test call graph at the recorded
  fingerprint.
- Confirmed the constructor mismatch directly from the current signatures and call; managed Cargo is
  additionally blocked by the recorded approved-root separator failure.
- Isolated `rustfmt --edition 2024 --check --config skip_children=true` passed only `mod.rs`; the four
  foreign current files differ from rustfmt and were not rewritten.
- Verified both Unreal primary-source files and all seven owner/optimize plan paths used by the routing
  record exist.
- Protected plans and indexes were not modified. This is not an accepted milestone, so no commit or
  WeCom notification is due.
