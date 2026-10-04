# Editor core script-build generation architecture revalidation

## Status

- Result: `static_complete / dynamic_pending / product_integration_pending`.
- Review date: 2026-08-16.
- MVP priority: P0 before script edit/build/Play is connected to the basic editor; P1 for fixed-step
  allocation after the generation and execution contracts are correct.
- Owners: Editor13 owns orchestration and source intent; Runtime13 owns compilation/artifacts;
  Editor14/Runtime11 own bounded execution; Editor04 owns Play resume; Editor17 owns diagnostic
  ingress. Render17 owns F4 product measurement.
- Accounting: retain `zircon_editor/src/core/script_build/**` in `pending.md`. Do not add it to
  `review.md` before real VM/job/Play integration, current managed tests and the matrix below pass.
- Code disposition: no Rust source changed. Four tracked files are foreign modified and
  `diagnostics_sink.rs` is foreign untracked current work; their owners were preserved.

## Exact scope

| scope | files | physical lines | tests | ordered path-and-raw-content SHA256 |
|---|---:|---:|---:|---|
| `zircon_editor/src/core/script_build/**` | 5/5 | 1,582 | 26 | `6805fdfc541c8374c0c1790b01c513bb8f9f422315a07ad6014721e9a69790c5` |

The fingerprint is SHA256 over sorted normalized path, NUL, raw file bytes, NUL. Every current Rust
file and all tests were read in full. Exact symbol search outside this module found no production
orchestrator, diagnostics-sink, watch, command or Play caller. The UI only exposes a console source
filter for records already labelled as script-build output. This is therefore a pre-integration
capacity and correctness gate, not a measured current frame hotspot.

This supersedes the 2026-07-30 4-file/912-line/13-test fingerprint `3e664862...` and corrects the
stale main-plan accounting. The current source adds diagnostics projection, first-event latency,
path-byte admission, one coalesced pending request, cancellation and request-derived generation.

## Per-file acceptance record

| file | current-source verdict |
|---|---|
| `diagnostics_sink.rs` | Generation/request/step cursor prevents exact replay and stale projection, but every admitted diagnostic is formatted, cloned and synchronously emitted one-by-one through the canonical log path. Retained log capacity does not bound ingress CPU, I/O or allocations. |
| `mod.rs` | Export/test shell only; no independent runtime work. |
| `orchestrator.rs` | Watch paths are bounded by 20 entries/64 KiB and first-event latency by 1 s; explicit storms coalesce to active plus one pending request. Request id is still used as source generation, and failure/cancel intentionally deletes all queued and debouncing newer changes. |
| `request.rs` | Typed triggers and completion identity are useful. A request still heap-allocates a fixed three-step Vec and dispatch clones its current step/path batch. More importantly, request/source/artifact/binding identities are not distinct. |
| `tests.rs` | Strong pure-state coverage includes 1M explicit triggers and diagnostic replay, but explicitly accepts dropping later source changes on failure/cancel and proves only bounded retained logs, not bounded diagnostic ingress work. No VM/job/artifact/binding/Play product fixture exists. |

## Corrected positive baseline

The current foreign work fixes most admission defects recorded by PERF-MVP-557:

- continuous watch storms cannot slide forever; deadline is capped from the first event;
- incremental identity storage is bounded by both count and path bytes and collapses to full rebuild;
- Command/Play storms occupy one request and promote intent `Watch < Command < Play`;
- active work plus at most one queued generation replaces the unbounded `VecDeque`;
- last outcome is shared by `Arc`, cancellation is explicit, and completion validates request/step;
- diagnostic replay is cursor-based and the retained log store remains bounded.

These improvements should be retained. The remaining issue is not queue container selection.

## Structural bottlenecks

### P0: `ScriptBuildGeneration` is a request alias, not a source or artifact generation

`ScriptBuildRequest::new` constructs generation directly from request id (`request.rs:88-103`). No
accepted watch/source revision exists before request creation; no compiled artifact identity is
sealed after compile; validation and binding refresh carry only the same request-derived value.
Consequently the system cannot prove which source snapshot produced an artifact, whether a binding
refresh applies that artifact, or which generation Play is waiting for.

Create explicit immutable identities and receipts:

1. `ScriptSourceGeneration` advances when a watch/save batch is admitted and owns its normalized
   changed-module set or full-rebuild marker;
2. `ScriptBuildIntent { source, priority, observers, play_waiters }` has one active plus one latest
   pending generation; same-source requests merge observers and priority without creating work;
3. Runtime13 returns `ScriptArtifactGeneration { source, artifact_digest, ledger_digest }` only after
   compilation and artifact durability succeed;
4. ledger validation returns a receipt for that exact artifact; binding publication creates
   `ScriptBindingGeneration { artifact, runtime_session }` at the runtime/editor safe point;
5. Play resumes only when the accepted binding generation satisfies its required source generation.

Request ids remain observer/ticket identities and must not masquerade as content generations.

### P0: failure and cancellation erase newer source changes

On any non-success completion, `complete` removes the active request, takes the queued request and
clears all pending watch state (`orchestrator.rs:270-284`). The test
`failure_drops_queued_and_debouncing_followups` requires this behavior. A compile error in generation
N can therefore delete valid edits from N+1 and leave the editor on stale artifacts indefinitely.

Failure/cancel must terminate only the exact active ticket. Preserve the latest pending source
generation and schedule it according to retry policy; remove only observers whose explicit request
was cancelled. A failed generation may remain visible for diagnostics, but cannot overwrite newer
diagnostics, artifacts, bindings or Play state. Shutdown uses an explicit fence/terminal receipt
rather than interpreting cancellation as permission to erase source facts.

### P0: bounded log retention hides unbounded diagnostic ingress work

`ScriptBuildDiagnosticsSink::project` iterates every diagnostic and calls `EditorLogService::emit`
for each (`diagnostics_sink.rs:85-95`). The 256-row test confirms all 256 emissions even when the log
retains eight. Current logging analysis shows each emit can synchronously traverse sink/file/fanout
work, so a compiler error storm runs on whichever thread projects completion.

Runtime13 must deliver count+byte-bounded diagnostic pages keyed by source/artifact/step. Editor17 and
PERF-MVP-644 provide one batch log ingress with a truncation/continuation receipt, severity counts and
one visible-generation invalidation. Formatting and jump-path ownership happen once per admitted row;
rows evicted by policy are not first pushed through unbounded per-record persistence. The cursor must
advance only through a successfully admitted page.

### P1: fixed three-step allocation is subordinate

Every request owns a heap `Vec<ScriptBuildStep>` and each dispatch clones the selected step, including
at most 20 paths/64 KiB. Replace this with a compact phase enum and one shared immutable source batch
when implementing the generation cut. Do not optimize it first: the module has no product caller and
generation correctness dominates one bounded allocation per accepted build.

## Execution and thread ownership

The module is deliberately a pure state machine today. Product integration must submit compile and
artifact I/O through the shared Editor14/Runtime11 job authority with entry/byte/age/deadline budgets,
cancellation and a `script_artifacts` exclusion group. It may not create a watcher thread, compiler
thread pool or private process-output readers. The UI thread only admits/coalesces facts, consumes
bounded completion pages and publishes the accepted binding generation.

## Unreal source evidence

- `dev/UnrealEngine/Engine/Source/Developer/Windows/LiveCoding/Public/ILiveCodingModule.h` exposes
  distinct `Success`, `NoChanges`, `InProgress`, `CompileStillActive`, `Failure` and `Cancelled`
  results instead of treating every request as one undifferentiated completion.
- `LiveCodingModule.cpp:699-724` rejects overlapping active compiles and returns immediately for
  asynchronous work. `808-924` applies the loaded patch, finalizes reinstancing and broadcasts patch
  completion only after compile state reaches the synchronization boundary.
- `dev/UnrealEngine/Engine/Source/Developer/HotReload/Private/HotReload.cpp:945-1005` refuses a second
  compile, builds a changed-module set, starts asynchronous compilation and passes the successful
  changed-module result into a separate reload phase.

The transferable standard is single-flight compilation plus explicit compile/apply/reload outcomes
and changed-module identity. Zircon should not copy Unreal's global object scans or GC at patch time;
its VM can publish immutable artifact/binding generations and budget safe-point application.

## Acceptance matrix

| gate | matrix | required result |
|---|---|---|
| admission | paths `1/20/21/10k`, bytes `64B/64KiB/1MiB`, watch `1/60/1k Hz`, explicit triggers `1/1k/1M` | resident active+pending intents <=2; path storage bounded; first-event latency bounded; same source compiles <=1 |
| generation | changes during compile/validate/bind, success/failure/cancel/supersede, runtime session replace | newer source fact loss=0; stale artifact/binding/diagnostic/Play apply=0; every receipt carries exact source/artifact/binding identity |
| execution | compile `0/16ms/10s`, artifact `1KiB/1GiB`, worker stall/failure/shutdown | UI compile/process/I/O wall=0; one shared job authority; queue bytes/age/RSS bounded; deterministic cancellation/fence |
| diagnostics | rows `0/1/1k/1M`, row `64B/8KiB`, retained `8/2,048`, consumer stall `0/60s` | count+bytes+deadline pages; batch ingress; bounded RSS/I/O; one truncation receipt; stable visible projection scans/formats 0 |
| product | script save storm, manual build, build-before-Play, failure recovery, cancel, project/runtime replace | managed tests plus 31-run F4 WPR CPU/waits/wakeups/file-I/O/RSS/package-power distributions; behavior changes only after accepted binding generation |

RenderDoc is not applicable because this module has no render path. No product trace or parity claim is
valid before the VM/job/Play integration exists. No artifact was written to C:.

## Static gates executed

- Read 5/5 current Rust files and all 26 tests at the recorded stable fingerprint; exact external
  caller search confirmed product integration remains absent.
- `python -m unittest tools.tests.test_editor13_script_build_orchestrator_contract -v` passed 8/8
  static contract tests.
- Isolated `rustfmt --edition 2024 --check --config skip_children=true` passed
  `diagnostics_sink.rs`, `orchestrator.rs` and `request.rs`; foreign `mod.rs` and `tests.rs` differ.
- Managed Cargo and F4 WPR remain blocked by the recorded editor build approved-root separator defect.
- Protected plans/indexes were not modified. This is not an accepted milestone, so no commit or WeCom
  notification is due.
