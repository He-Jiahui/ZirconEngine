---
related_code:
  - zircon_editor/src/ui/workbench/startup
  - zircon_editor/src/ui/retained_host/app/startup.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/startup
  - zircon_editor/src/ui/retained_host/app/welcome_session
  - zircon_editor/src/ui/host/startup
  - zircon_editor/src/ui/host/editor_host_startup.rs
  - zircon_editor/src/ui/host/editor_manager_project_session.rs
  - zircon_editor/src/core/hub_link/recent_writeback.rs
  - zircon_editor/src/core/project/authority.rs
  - zircon_runtime/src/scene/world/world.rs
tests:
  - zircon_editor/src/tests/host/manager/bootstrap_and_startup/session_startup.rs
  - zircon_editor/src/tests/host/manager/project_generation_projection.rs
  - zircon_editor/src/ui/retained_host/app/welcome_session/project_probe.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/02-unreal-aligned-engine-system-hard-cutover.md
  - docs/plans/performance/pending.md
  - docs/plans/performance/review.md
owner_plans:
  - docs/plans/optimize/zircon_editor/51-editor-project-startup-open-create-authority-hub-handshake-session-guard-focus-recent-recovery-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/05-scene-ecs-world-lifecycle-review.md
  - docs/plans/optimize/zircon_runtime_interface/06-project-manifest-session-lock-hub-protocol-recent-project-cross-process-contract-product-integration-review.md
  - docs/plans/mvp/02-f1-project-and-assets.md
reference_code:
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/EditorServer.cpp
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/PlayLevel.cpp
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/EditorEngine.cpp
  - dev/UnrealEngine/Engine/Source/Editor/GameProjectGeneration/Private/SProjectBrowser.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/AssetRegistry/Private/AssetRegistry.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/AssetRegistry/Private/AssetDataGatherer.cpp
doc_type: current-architecture-performance-review
status: m1_single_consume_implemented_static_pass_dynamic_pending
source_recheck_required: true
created_at: 2026-08-22
---

# Editor workbench startup single-consume project activation review

## Status

- Result: `M1 single-consume implemented / static pass / dynamic pending`.
- MVP priority: P0 for removing the ordinary startup World deep clone and for removing recent
  history from the project-activation commit gate; P1 for duplicate recent validation, save/play
  snapshot ownership and presentation allocation cleanup.
- Accounting: keep `zircon_editor/src/ui/workbench/startup/**` in `pending.md` as one concise
  `14/14 static reviewed, dynamic pending` entry. Do not move it to `review.md` until M0-M5 pass.
- Code disposition: after this architecture record was written, the narrow M1 cut changed five
  Rust files. It consumes the project payload before branching, moves workspace and World, and adds
  one typed-component preservation test. Recent protocol and generic `World::clone` are unchanged.

## Exact scope

| scope | files | physical lines | raw bytes | sorted path-NUL-content-NUL SHA256 |
|---|---:|---:|---:|---|
| `zircon_editor/src/ui/workbench/startup/**` | 14/14 | 569 | 19,794 | `44273d3e21a4864ef0a67a29e943322a2b95627565d690c1af135a2af4a28079` |

The directory was clean when fingerprinted. Its last owning commit was
`7a20f921bb97ed428ae248cbcaf3c2fac5442ddf`. All 14 Rust files were read in full. The call-chain
review also read startup resolution, project activation, retained-host application, Welcome
project probing, recent-project storage, project authority, `EditorProjectDocument` and the full
`World::clone` implementation.

Focused current tests inspected:

| test surface | lines | tests | current gap |
|---|---:|---:|---|
| manager startup session | 248 | 7 | checks returned documents, not consume-once handoff or clone count |
| project generation projection | 255 | 10 | proves Welcome projection performs no I/O, not startup ownership |
| Welcome project probe | 978 | 18 | covers debounce/admission/cancel/generation well; unrelated to recent restore I/O |
| in-module tests | included above | 4 | path display and relative-time semantics only |

## Current activation sequence

The default valid-recent path currently performs:

1. `resolve_startup_session_with_project_open` loads the recent registry and validates every row.
2. It selects the first valid row and opens the project synchronously.
3. Project activation loads the project document, configures diagnostics/plugins, records the
   recent project under a cross-process writer lease, then publishes document activation.
4. Startup resolution loads and validates the complete recent registry again.
5. `build_startup_state(&session, ...)` clones `session.project`, moves the cloned World into the
   authoring runtime, and returns an `EditorState`.
6. The original `EditorProjectDocument`, including the original World, remains in
   `RetainedEditorHost::startup_session` even though product code has no later consumer for it.

This is not a harmless DTO clone. `Scene` is `World`; `World::clone` snapshots fixed component
families, clones registries/resources/events/messages/observers/caches, collects stable entity IDs,
then rebuilds entity and component-storage projections. Its cost scales with World entities,
registered state and owned payload, and it can omit arbitrary typed plugin components under the
current clone policy.

## Quantified static facts

| fact | current-source count or bound |
|---|---:|
| `session.project.clone()` startup sites | 1 |
| `World::clone`/projection rebuilds implied by a project startup | 1 |
| complete project documents retained after initial host construction | 1 |
| recent snapshot calls in successful automatic restore | 2 |
| recent registry rows | at most 8 |
| validation manifest loads from those two snapshots | at most 16 |
| selected project open manifest load | at least 1 additional load |
| recent registry reads on successful automatic restore | 3: first snapshot, record merge, second snapshot |
| recent registry writes | 1 |
| Windows infinite recent-writer wait sites | 1 |
| Welcome snapshot refresh call sites | 14, event/completion driven |
| explicit menu paths cloning the World through `project_scene()` | 2: Save and Enter Play |

These are source-derived operation counts, not measured latency or allocation results. No dynamic
CPU, RSS, I/O, lock-wait or power value is claimed before the current-source profiler gates pass.

## Findings

### P0-1: Startup clones and retains the complete project World

`build_startup_state` matches on `session.project.clone()`. The cloned World is consumed by
`prepare_authoring_world`, while the source document stays in the host's long-lived startup
session. The later host logic reads recent projects, draft, mode and status from that session; it
does not read the retained project document. Close-project code clones the startup session before
clearing `project`, so the retained source also creates another possible deep clone at close.

Required correction:

- model the loaded project as a one-shot activation payload, not reusable presentation state;
- move the `EditorProjectDocument` and World exactly once into the authoring owner;
- store only lightweight startup/Welcome projection data after commit;
- make accidental payload cloning difficult or impossible at the type boundary;
- require startup World clone count and retained source-World bytes to equal zero.

### P0-2: Best-effort recent history participates in project activation commit

`complete_project_open` calls `record_recent_project` before the document session is published.
Any recent-store failure becomes an activation error and triggers project rollback. On Windows the
writer lease calls `WaitForSingleObject(handle, INFINITE)`, so a stuck Hub/Editor writer can hold
the editor startup or UI open path indefinitely.

This contradicts Optimize51's existing rule that recent history is a derived projection and not a
project-open commit gate. A project that has already passed identity, session admission, manifest,
runtime and plugin activation must not be rolled back because optional history persistence failed.

Required correction:

- commit the project session independently of recent history;
- enqueue a bounded, coalescing, revisioned recent update after commit;
- use deadline/cancel and owner diagnostics for cross-process writer admission;
- publish a typed projection warning on failure; never block or roll back the live project;
- isolate corrupt/oversize recent data and rebuild a bounded projection.

### P1-1: Successful auto-restore repeats registry and manifest I/O

The successful path calls `recent_projects_snapshot` before and after open. Each call validates all
rows by resolving the path, checking existence/canonical structure and loading the manifest. The
record step separately rereads, merges, pretty-serializes and atomically rewrites the registry.
With the current eight-row cap this is bounded in item count, but remote, sleeping or unavailable
volumes make latency depend on serial I/O, not list size.

The activation transaction should reuse one immutable preflight generation. The selected row's
validated identity/manifest summary must flow into open; post-commit history update should return
or publish the resulting registry revision instead of causing another full revalidation.

### P1-2: Save and Enter Play obtain a World by deep clone on the event thread

`EditorState::project_scene()` delegates to `EditorAuthoringWorld::try_snapshot`, which executes
`Clone::clone` under gateway access. Save and Enter Play both call it synchronously from menu event
execution. These are explicit operations rather than per-frame work, but they need a sealed scene
generation and background serialization/fork contract from Runtime05/Editor61. M1 must not hide
this separate structural problem by changing generic `World::clone` semantics.

### P2-1: Presentation helpers allocate bounded replacement snapshots

`welcome_pane_snapshot` clones all labels and collects a new recent-project vector on each user
input or probe completion. `display_project_title` also normalizes and replaces path separators
when host shell data is rebuilt. Current recent rows are bounded to eight and the Welcome probe is
event/completion driven, so this is not the first optimization target. After P0/P1, cache immutable
Welcome/title generations and patch only changed fields.

## Accepted current behavior

- Welcome project path validation is not performed by paint/projection.
- The project probe uses a 50 ms debounce, 250 ms maximum feedback delay, generation rejection,
  cancellation, an admission key, estimated bytes and a background editor job.
- Repeated identical pending/active draft probes are suppressed.
- Recent registry merge is path-deduplicated, deterministically ordered and capped at eight rows.
- Default-node selection is bounded linear startup work, not a stable-frame scan.

These pieces should be preserved while ownership and commit ordering are replaced.

## Unreal source evidence

### Loaded editor World is assigned, not copied into a startup DTO

`UEditorEngine::Map_Load` in `EditorServer.cpp` has explicit CPU/load-time scopes, loads or reuses a
World package, finds its `UWorld`, assigns it directly to the editor `FWorldContext` and `GWorld`,
then initializes that World. Zircon should likewise transfer the prepared authoring World to its
single runtime owner instead of keeping a second source World in startup presentation state.

### Deep World duplication is an explicit Play operation

`PlayLevel.cpp` creates a dedicated PIE package and calls `UWorld::GetDuplicatedWorldForPIE` only
for Play In Editor. It measures and logs both the duplicate-object time and total copied-world
time. This supports a named, measured fork boundary; it does not justify an unobserved clone during
ordinary editor startup.

### Discovery work has explicit refresh and profiler ownership

`SProjectBrowser::FindProjects` has a CPU profiler scope, snapshots recent settings once per
refresh, enumerates known projects, normalizes/deduplicates, then requests a list refresh. The path
is invoked by construction and explicit refresh/F5, not by every presentation projection.

### Heavy asset discovery is asynchronous and time-budgeted

`AssetDataGatherer.cpp` enables asynchronous editor discovery when threading is available, creates
a below-normal-priority discovery thread and uses bounded parallel directory batches.
`AssetRegistry.cpp` processes gathered data behind `TryLock`, supports background interruption and
a maximum tick duration, and defers main-thread events. Zircon's existing Welcome probe follows
this direction; recent validation and project asset discovery must use the same bounded task
principles rather than serial pre-first-present I/O.

### Recent history is bounded derivative state

`UEditorEngine::UpdateRecentlyLoadedProjectFiles` updates a capped ten-entry settings list and
queues a separate project-editor-record update. `SProjectBrowser` consumes that list as browser
presentation input. The transferable principle is that recent history is bounded derivative data;
Zircon must still implement its own cross-process revision, timeout and failure semantics.

## Target architecture

1. Introduce a one-shot `PreparedProjectActivation` that owns canonical identity, the prepared
   `EditorProjectDocument`, admission receipt and activation generation. It is not `Clone`.
2. Split `StartupPresentationState` from that payload. Only mode, Welcome draft, typed status,
   recent projection generation and optional startup-view intent remain in the host.
3. Consume the prepared document exactly once at the authoring gateway commit point. Move World
   and workspace; do not clone either to satisfy borrowing convenience.
4. Publish one `ProjectSessionGeneration` after critical host/runtime/plugin/document effects
   commit. Every later watcher, UI and save/play request keys off that generation.
5. Move recent writeback outside the critical effect set. Coalesce by canonical project identity,
   use revision/CAS plus bounded lease time, and return a typed projection receipt.
6. Resolve recent rows into immutable, paged preflight results on background jobs. Reuse the
   selected row's resolved identity and manifest generation during open.
7. Give Save/Play named sealed-generation capture/fork APIs with progress, cancel, bytes and
   generation mismatch reporting. Ordinary startup must never call that fork API.
8. Cache Welcome/title presentation by generation only after the ownership cut is complete.

## Instrumentation first

M0 must export scenario counters, not rely only on free-form scopes:

| counter | required purpose |
|---|---|
| project activation payload consume count | exactly one for project startup |
| startup World clone count/time/bytes/entities/components | prove ordinary startup reaches zero |
| retained startup project-document bytes | prove the source payload is not kept after commit |
| activation phase CPU/wall/thread | identity, manifest, admission, runtime, plugins, document, UI, first present |
| recent registry read/write/parse/bytes/revision | attribute redundant storage work |
| recent row resolve/exist/manifest-read count | prove selected preflight reuse and bounded background work |
| recent writer wait p50/p95/p99/max/timeout | prove no unbounded UI/startup wait |
| recent update queued/merged/completed/failed | prove best-effort behavior and coalescing |
| project rollback reason | recent projection must never appear after the cut |
| Welcome probe queued/merged/cancelled/stale/completed | preserve existing background behavior |
| save/play scene seal/fork bytes/time/thread | separate explicit snapshots from ordinary startup |
| first native window and first successful present timestamps | define user-visible startup completion |

## Milestones

| milestone | deliverable | dependency |
|---|---|---|
| M0 | Red consume/preservation tests plus clone/I/O/lease/phase counters and current baseline capture. | current source recheck |
| M1 | `build_startup_state` consumes the project document once; stored startup state no longer retains World; typed component preservation test passes. | M0 red test |
| M2 | Project commit and rollback exclude recent history; bounded coalescing writeback receipt. | Editor51 + Interface06 |
| M3 | One immutable recent preflight generation; selected identity/manifest reused; background row validation. | M2 |
| M4 | Save/Play sealed-generation capture/fork replaces generic event-thread World clone. | Runtime05 + Editor61 |
| M5 | Managed Windows functional tests plus WPR/ETW CPU, allocation, file I/O, lock, RSS, latency and package-power matrix. | M0-M4 |

## Acceptance matrix

| gate | matrix | required result |
|---|---|---|
| ownership | Welcome, explicit open, valid recent restore, create, close, reopen | project payload consume `=1`; ordinary startup World clone `=0`; stored source World bytes `=0` |
| data integrity | 1/10k/100k entities, built-in plus plugin typed components/resources | authoring World identity/content preserved without clone-policy loss |
| recent I/O | 0/1/8 rows on local, sleeping, unavailable and corrupt stores | UI reaches first present; bounded background result; no infinite wait; no project rollback |
| reuse | valid recent auto-restore repeated 31 runs | one preflight generation; selected identity/manifest not re-read by presentation/history refresh |
| failure | writer contention, timeout, process death, corrupt/oversize registry | live project remains committed; typed warning and retry receipt; bounded memory/time |
| Welcome input | repeated same draft plus 1k-character burst | debounce/max delay/cancel/stale/admission behavior preserved; no filesystem I/O in projection |
| save/play | 1/10k/100k entities and plugin component matrix | named seal/fork counters only; progress/cancel; no silent component loss |
| product | F4 launch/open/close/reopen/Play, 31 measured runs | phase CPU/wall, alloc/RSS, file I/O, lock wait, first-present p50/p95/p99 and package power reported from identical hardware/config; artifacts only on D/E/F |

RenderDoc is not a primary tool for this module. Use it only if startup ownership changes visible
render output or first-frame GPU composition. WPR/ETW, allocator counters and file/lock traces are
the required tools for the identified CPU, memory and I/O bottlenecks.

## Static gates executed

- Read 14/14 startup Rust files in full and reproduced the fingerprint above.
- Traced both initial host construction paths and Welcome apply/refresh paths.
- Confirmed the sole project-document clone site and the long-lived startup-session owner.
- Read the full `World::clone`; confirmed it rebuilds projections and is not a handle clone.
- Reconstructed the successful recent restore operation counts and the eight-row upper bound.
- Confirmed the infinite Windows writer wait and recent failure-to-activation rollback path.
- Confirmed Welcome validation already uses a bounded background job and projection is I/O-free.
- `rustfmt --edition 2021 --check` passed for all 14 owned startup files.
- Read the cited Unreal map load, PIE duplication, project browser, editor recent update and asset
  registry/gatherer code directly under `dev/UnrealEngine`.
- M1 changes `build_startup_state` and both host construction call paths to use a mutable one-shot
  session payload. Static source checks now report one `session.project.take()` site, zero
  `session.project.clone()` sites and zero `editor_workspace.clone()` sites in startup state build.
- Added `startup_state_consumes_the_project_world_without_clone_policy_loss`: it inserts a custom
  typed component into the prepared World, requires the retained session project to be empty, and
  verifies the component remains present in the authoring World after handoff. This is behavioral
  coverage in addition to the static clone-site check.
- `rustfmt --edition 2021 --check` passes for the five M1 source/test files, and `git diff --check`
  reports no patch whitespace errors.
- Managed Windows validation was requested with `F:\cargo-targets\verify`. The first attempt was
  rejected before Cargo start because compatible pool job `abb01ecff3df4ab288496798b2560a3a`
  was compiling. A later retry submitted coordinator request
  `2a2414d3921442df829708286bc36267`, but `cargo.acquire` had no terminal result after the 15-second
  reconciliation timeout; an unrelated `zircon_runtime` pool lane was still compiling afterward.
  Neither attempt started this module's Cargo command. Executed test count remains zero; M1 is not
  dynamically accepted.
- No fabricated latency, allocation, RSS, I/O, lock-wait or power values are recorded.

## Completion rule

The module remains pending until M0-M5 pass on one current-source fingerprint. Removing one String
clone, caching relative-time labels, lowering the recent row cap, adding a timeout without moving
recent outside the commit gate, or running RenderDoc alone does not complete this module. A
milestone commit and quantified WeCom notification are allowed only after the accepted milestone
has current-product dynamic evidence.
