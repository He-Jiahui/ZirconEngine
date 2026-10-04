---
related_code:
  - zircon_editor/src/ui/workbench/startup
  - zircon_editor/src/ui/retained_host/app/startup.rs
  - zircon_editor/src/ui/host/startup
  - zircon_editor/src/ui/host/editor_manager_project_session.rs
  - zircon_editor/src/core/hub_link/recent_writeback.rs
  - zircon_runtime/src/scene/world/world.rs
plan_sources:
  - docs/plans/performance/01/2026-08-22-editor-ui-workbench-startup-single-consume-project-activation-architecture-review.md
owner_plans:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/02-unreal-aligned-engine-system-hard-cutover.md
  - docs/plans/performance/pending.md
  - docs/plans/performance/review.md
  - docs/plans/optimize/zircon_editor/51-editor-project-startup-open-create-authority-hub-handshake-session-guard-focus-recent-recovery-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/05-scene-ecs-world-lifecycle-review.md
  - docs/plans/optimize/zircon_runtime_interface/06-project-manifest-session-lock-hub-protocol-recent-project-cross-process-contract-product-integration-review.md
  - docs/plans/mvp/02-f1-project-and-assets.md
source_evidence:
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/EditorServer.cpp
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/PlayLevel.cpp
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/EditorEngine.cpp
  - dev/UnrealEngine/Engine/Source/Editor/GameProjectGeneration/Private/SProjectBrowser.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/AssetRegistry/Private/AssetRegistry.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/AssetRegistry/Private/AssetDataGatherer.cpp
doc_type: protected-plan-routing
status: requested
created_at: 2026-08-22
---

# Protected plan routing: Startup single-consume project activation

## Reason for routing

The performance ledgers and numbered project/runtime/interface/MVP plans are shared owner
authorities. This record routes the current 14/14-file evidence without overwriting concurrent
work. The detailed source, Unreal evidence, milestones and acceptance matrix live in the sibling
architecture review.

## Requested Performance01 updates

### Add a P0 startup ownership item

Current `build_startup_state` clones `EditorStartupSessionDocument::project`; because `Scene` is
`World`, this performs a deep World clone and projection rebuild before first present. The source
project document remains in the long-lived retained host without a later product consumer.

Acceptance:

- project activation payload is non-cloneable and consumed exactly once;
- ordinary Welcome/open/create/recent startup World clone count/time/bytes equals zero;
- retained startup source-World bytes equals zero after commit;
- arbitrary plugin typed component/resource content survives activation;
- deep World duplication exists only behind named, measured save/play/fork policies.

### Add a P0 recent-not-in-commit item

Current project activation synchronously records recent history; store failure rolls back project
open and Windows writer admission can wait forever. Acceptance must require recent writeback to be
post-commit, bounded, coalescing and best effort. A recent failure may publish a typed warning and
retry receipt but must never block first present or roll back the live project.

### Add the P1 duplicate-I/O item

A successful automatic restore currently performs two full recent snapshots, up to 16 validation
manifest loads, at least one selected-project manifest load, three recent-registry reads and one
write. Require one immutable background preflight generation and reuse its selected canonical
identity/manifest data through open and post-commit projection update.

Suggested new IDs should be allocated only by the Performance01 owner; do not reuse an ID from
this routing record.

## Requested Performance02 update

Under the World/editor hard-cut milestones, explicitly ban a cloneable full World inside startup,
project history or UI presentation envelopes. Add the transaction shape:

`ProjectLaunchIntent -> PreparedProjectActivation -> one World move -> ProjectSessionGeneration`

Recent/Welcome/history consume receipts after commit. Save/Play use explicit sealed-generation
capture/fork operations and do not reintroduce `World: Clone` as an ownership shortcut.

## Requested Editor51 updates

Editor51 already records that recent is a derived projection, identifies the infinite writer wait
and synchronous recent validation, and requires phase performance data. Add the missing current
source evidence:

- E-PROJ P0: `build_startup_state` deep clones the complete `EditorProjectDocument::world` and the
  retained host keeps the source document;
- the successful recent restore path performs two snapshots around open and one writer merge;
- `record_recent_project` remains inside `complete_project_open`, so its failure still triggers
  project activation rollback despite Editor51's target rule;
- M1 must split one-shot `PreparedProjectActivation` from lightweight startup presentation state;
- acceptance includes plugin typed-component preservation, startup clone bytes `=0`, retained
  source bytes `=0`, and recent rollback reason count `=0`.

The first narrow M1 implementation may consume the existing project document in place, but that is
only an intermediate hard cut. Editor51 remains the owner of the versioned launch intent,
activation operation, receipt, rollback and session-generation architecture.

## Requested Runtime05 update

Route the startup clone as a concrete product caller of Runtime05's existing `World::clone`
findings. The runtime plan should distinguish:

- ordinary editor startup: move/attach the prepared World, clone count `=0`;
- Save: sealed authoring generation plus versioned serialization job;
- Play: explicit fork policy with component/resource transfer declarations;
- checkpoint/recovery: separate snapshot policy;
- render: immutable extract generation, never mutation permission obtained by cloning World.

Do not optimize generic `World::clone` to make the startup copy cheaper. Delete the caller first,
then converge the remaining explicit fork/checkpoint contracts.

## Requested RuntimeInterface06 update

The shared recent protocol needs revisioned, bounded and corruption-tolerant projection semantics:

- monotonic registry revision and compare/merge contract;
- bounded payload bytes and row count;
- writer deadline/cancel plus holder diagnostics;
- typed `queued/merged/completed/failed/timed_out` receipt;
- corrupt-store quarantine/rebuild behavior;
- canonical identity distinct from localized/lossy display text;
- no protocol result may be a project-session commit prerequisite.

The interface plan owns cross-process compatibility. Editor51 owns scheduling and UI projection.

## Requested MVP02 update

Project/assets MVP acceptance must add:

- valid recent restore reaches first present with the prepared World moved once;
- startup session retains no second full project World;
- recent-store contention, corruption or unavailability cannot prevent project open;
- 0/1/8 recent rows on local/unavailable paths have bounded behavior;
- current-source startup/open/create/close/reopen samples report CPU/wall, allocations, RSS, file
  I/O, lock wait and first-present p50/p95/p99;
- project asset discovery remains background/budgeted and publishes immutable generations.

## Requested pending/review accounting

Keep a single concise pending entry:

`zircon_editor/src/ui/workbench/startup/** - 14/14 static reviewed; single-consume project payload,
best-effort recent commit separation, managed Cargo, real product WPR/ETW/RSS/I/O/lock/power pending.`

Do not add 14 file-level rows. Do not move the module to `review.md` until the sibling review's
M0-M5 acceptance matrix passes on one current-source fingerprint.

## Cross-plan completion rule

This routing request is acknowledged only when each owner plan records either the required work or
a concrete equivalent contract with the same zero-clone, no-retained-copy, no-recent-commit-gate
and dynamic evidence requirements. A source-shape assertion, recent mutex timeout alone, one warm
unit test, or RenderDoc capture is insufficient.
