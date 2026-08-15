---
related_code:
  - zircon_editor/src/core/script_build
  - zircon_runtime/src/script/vm
  - zircon_runtime_interface/src
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/review.md
  - docs/plans/performance/pending.md
  - docs/plans/optimize/zircon_editor/01-retained-ui-architecture-performance-review.md
owner_plans:
  - docs/plans/zircon_editor/editor/04-pie-and-simulation.md
  - docs/plans/zircon_editor/editor/13-script-compilation-management.md
  - docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md
  - docs/plans/zircon_editor/editor/17-editor-services-and-recovery.md
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
  - docs/plans/zircon_runtime/runtime/13-script-binding-and-reflection.md
source_evidence:
  - dev/UnrealEngine/Engine/Source/Developer/Windows/LiveCoding/Public/ILiveCodingModule.h
  - dev/UnrealEngine/Engine/Source/Developer/Windows/LiveCoding/Private/LiveCodingModule.cpp
  - dev/UnrealEngine/Engine/Source/Developer/HotReload/Private/HotReload.cpp
---

# Protected plan routing: script source, artifact and binding generations

## Reason for routing

The main performance plan, `review.md`, `pending.md`, optimize plan and numbered owner plans are
protected/foreign dirty in this session. This record routes the exact current-source findings without
overwriting their owners. Evidence source:
`2026-08-16-editor-core-script-build-generation-current-architecture-review.md`.

## PERF-MVP-557 correction

Retain PERF-MVP-557 at P0 but replace its stale current-source diagnosis:

- first-event max latency, 20-path/64-KiB bounds, single pending generation, Command/Play coalescing,
  trigger precedence, explicit cancellation and 1M-request constant admission are now implemented;
- current scope is 5/5 files, 1,582 lines, 26 tests, fingerprint `6805fdfc...`;
- `ScriptBuildGeneration` remains request-id-derived, not source/artifact/binding identity;
- failure/cancel deliberately deletes the queued generation and all later watch changes;
- diagnostic projection emits every row individually through the synchronous canonical log path;
- no production watcher, command, Play, job, VM or diagnostics-sink caller exists.

Required target: `ScriptSourceGeneration` -> `ScriptBuildIntent` -> `ScriptArtifactGeneration` ->
`ScriptBindingGeneration`. One active plus one latest pending source intent is retained. Failure or
cancellation terminates only the active ticket and never erases a newer source generation. Same-source
Command/Play requests merge observers and latest Play intent. Product execution uses the shared job
authority, and Play waits for the binding generation required by its source intent.

Do not create a new task for diagnostic storms. Route count+byte-bounded diagnostic pages, batch log
ingress, truncation receipts and visible-generation invalidation through PERF-MVP-644.

## Requested owner-plan updates

### Editor13

Own source intent and the four-stage receipt chain. Split request identity from content generation;
replace the test contract that drops later facts on failure/cancel; retain the current admission hard
bounds and trigger precedence. Add product adapters only after the generation tests pass.

### Runtime13

Compile one immutable source generation and return a content-addressed artifact plus ledger digest.
Validation and binding refresh must reject stale source/artifact/runtime-session identities. Expose
changed-module facts directly; do not require the editor to rediscover changes by scanning all scripts.

### Editor14 and Runtime11

Execute compile/artifact work through the one shared job/process/I/O authority with entry, byte, age
and deadline admission, cancellation, process-output paging and `script_artifacts` exclusion. No
script-build-private thread pool, watcher thread or output-reader pair is allowed.

### Editor04

Bind each Play waiter to its required source generation. Resume only after the matching artifact is
validated and its binding generation is accepted by the active runtime session. Failure, cancel,
supersede and project/runtime replacement terminate or advance the exact waiter deterministically.

### Editor17 and PERF-MVP-644

Provide one batch diagnostic ingress with count/byte/deadline bounds, severity totals, continuation or
truncation receipts and bounded visible projection. Retained ring capacity alone is not an ingress
budget; a million rejected/evicted diagnostics may not perform a million per-record file/fanout calls.

### Optimize zircon_editor/01

Add F4 script-save/build/Play scenarios only after real product integration. Measure admission,
compile worker/process, artifact I/O, safe-point binding, diagnostic batching and Play resume as
separate stages across 31 runs. Stable editor frames perform zero script-build work.

## Requested protected index state

- `pending.md`: replace stale 4-file accounting with 5/5 files, 1,582 lines, 26 tests, fingerprint,
  `static_complete / dynamic_pending / product_integration_pending`, the current review and corrected
  PERF-MVP-557.
- `review.md`: do not add the module. Require source/artifact/binding generation gates, preserved
  newer facts after failure/cancel, shared-job VM integration, batch diagnostics and F4 WPR evidence.

## Milestone and notification state

This is static architecture evidence and routing, not an accepted milestone. No git commit or WeCom
notification is due. Both become mandatory only after owner implementation, dynamic acceptance and
protected index reconciliation are complete.
