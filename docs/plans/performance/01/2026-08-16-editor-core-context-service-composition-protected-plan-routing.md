---
related_code:
  - zircon_editor/src/core/context
  - zircon_editor/src/core/recovery/autosave_service.rs
  - zircon_editor/src/ui/host/editor_manager.rs
  - zircon_editor/src/ui/retained_host/app/autosave.rs
  - zircon_editor/src/ui/retained_host/app.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/review.md
  - docs/plans/performance/pending.md
  - docs/plans/optimize/zircon_editor/01-retained-ui-architecture-performance-review.md
owner_plans:
  - docs/plans/zircon_editor/editor/00-editor-architecture-overview.md
  - docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md
  - docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md
  - docs/plans/zircon_editor/editor/08-tool-orchestration-and-commands.md
  - docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md
  - docs/plans/zircon_editor/editor/17-editor-services-and-recovery.md
source_evidence:
  - dev/UnrealEngine/Engine/Source/Runtime/Launch/Private/LaunchEngineLoop.cpp
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/EditorEngine.cpp
---

# Protected plan routing: editor context composition and F0 attribution

## Reason for routing

The main performance plan, `review.md`, `pending.md`, optimize plan and numbered owner plans are
protected/foreign dirty in this session. This record routes exact current-source findings without
overwriting their owners. Evidence source:
`2026-08-16-editor-core-context-service-composition-current-architecture-review.md`.

## Performance01 corrections

Replace the stale 2026-07-30 context accounting with 5/5 Rust files, 1,380 physical lines, 14 tests
and fingerprint `3ff94e5b...`. The tools accounting remains separate. The current context source is
not dynamically acceptable: its builder does not match the migrated log/autosave signature.

Do not create another performance task ID for the following downstream work:

- builder log/i18n JSON allocation, fanout and `O(D*R)` resync membership strengthen
  PERF-MVP-019;
- log ingress and duplicate bus projection strengthen proposed PERF-MVP-644;
- transaction sink semantics strengthen PERF-MVP-646;
- autosave activation/generation/admission remains in the current Editor17/Editor14 recovery task;
- tool topic caching and bounded `release_all` remain in the existing Editor08/context-tools record.

Add the F0 assembly matrix to the existing product startup measurement gate: settings definition/load,
quota resolution, owner construction, sink wiring, context publication and activation must have
separate wall/CPU/allocation/I/O/worker/message counters. The current evidence does not justify a
new executor or speculative parallel startup.

## Requested owner-plan updates

### Editor00 and Editor01

Own a typed, dependency-ordered `EditorStartupAssembly` contract. Keep one `EditorContext` aggregate,
publish it only after all owners and sinks are valid, and expose a terminal stage receipt. Specify
rollback and shutdown order. Do not introduce a service locator, duplicate runtime gateway or
editor-private scheduler.

### Editor17

Resolve `failure-2026-08-16-editor-context-autosave-construction-drift.md`: share one log service,
construct autosave from the existing job-system handle and pass both through the current context
signature. Preserve the recovery generation/fence plan; connecting the retained poll is a separate
activation step and must not precede its P0 scale corrections.

### Editor14

Assert that context, autosave and all editor adapters share one scheduler-backed `EditorJobSystem`
authority and quota/progress state. Startup may create zero private pools. Shutdown fences recovery
work before the shared job authority terminates.

### Editor02

Absorb the builder's log/i18n delivery into PERF-MVP-019 counters and bounded receipt design. Replace
the dropped-versus-delivered nested membership check with a receipt representation whose validation
does not become quadratic during backpressure. Do not add a context-local queue.

### Editor08

Retain the fixed-small bounded tool scheduler. Cache or type the built-in tool topic once per service
only with existing ordering tests; replace `release_all` only if the configured-cap measurements fail.

### Optimize zircon_editor/01

Add an F0 composition band before retained UI measurement: cold/warm settings load, service assembly,
wiring, publication and project/recovery activation. F4 stable traces must show context assembly work
zero. Record 31-run distributions for wall/CPU/allocation/file-I/O/waits/wakeups/RSS/package power and
the same-machine Unreal workflow before making efficiency or power-parity claims.

## Requested protected index state

- `pending.md`: retain one concise `zircon_editor/src/core/context/**` row with
  `static_complete / source_blocked / dynamic_pending`, 5/5 files, 1,380 lines, 14 tests, fingerprint,
  the current review and the open constructor failure.
- `review.md`: do not add the module. Require the owner repair, current managed tests, assembly/scale
  counters and F0/F4 WPR evidence.

## Milestone and notification state

This is static architecture evidence and routing, not an accepted milestone. No git commit or WeCom
notification is due. Both become mandatory only after the source failure, dynamic matrix and protected
index reconciliation are complete.
