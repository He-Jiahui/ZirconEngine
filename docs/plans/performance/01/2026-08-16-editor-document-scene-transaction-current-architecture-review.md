---
related_code:
  - zircon_editor/src/core/document
  - zircon_editor/src/core/project/scene_document.rs
  - zircon_editor/src/ui/host/editor_manager_project.rs
  - zircon_editor/src/ui/host/editor_scene_document_submission.rs
reference_sources:
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Public/Subsystems/AssetEditorSubsystem.h
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/Subsystems/AssetEditorSubsystem.cpp
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/FileHelpers.cpp
  - dev/Fyrox/editor/src/lib.rs
  - dev/Fyrox/editor/src/message.rs
  - dev/Fyrox/editor/src/scene/container.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/02-unreal-aligned-engine-system-hard-cutover.md
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/zircon_editor/01-retained-ui-architecture-performance-review.md
  - docs/plans/zircon_editor/editor/10/failure-2026-08-02-scene-open-create-project-authority-route-missing.md
  - docs/plans/zircon_runtime/frameworks/01/2026-08-15-m1-project-generation-durable-transaction-review.md
tests:
  - rustfmt per-file check for zircon_editor/src/core/document
  - document lifecycle identity and retention scale contracts
  - scene route stale-session and failure-compensation matrix
  - F4 scene open/create WPR and xperf product trace
doc_type: implementation-evidence
status: static_complete_dynamic_pending
created_at: 2026-08-16
---

# Editor document and scene transaction current-architecture review (2026-08-16)

## Status and disposition

- Result: `static_complete / structural_hard_cut_required / dynamic_pending`.
- Current module scope: 6/6 Rust files, 1,690 physical lines, 19 inline tests, 0 ignored tests.
- Ordered path-and-raw-content SHA256:
  `92f21be754fc1d14b8880ba4d14e03326f8ae09af3e0ac44a65f2ba1f8c7df40`.
- The production call graph was followed through project source loading/creation, runtime/editor catalog
  synchronization, authoring-world preparation, retained shell submission, document publication and
  rollback. Those supporting files are evidence anchors, not part of the six-file accounting row.
- No Rust source was changed. The six-file module and all three production anchors contain foreign
  modified or untracked work. More importantly, the dominant issue is a C3 owner/lifecycle/threading
  defect; a local map swap or lock narrowing without generation contracts would preserve the wrong
  architecture.
- This report supersedes the 2026-07-30 two-file document fingerprint and extends, rather than
  invalidates, `PERF-MVP-593`. The old task remains the bounded root-retention gate. The scene
  transaction and expanded scene-identity registry require a separate P0 task.
- Managed Cargo, WPR/xperf, allocator/RSS/energy sampling and an F4 product operation did not run.
  The approved-root editor build entry remains blocked by
  `failure-2026-08-15-build-editor-approved-root-separator.md`; no timing or power improvement is
  claimed.

## Exact per-file review

| File | Lines | Tests | Current finding and disposition |
|---|---:|---:|---|
| `zircon_editor/src/core/document/lifecycle.rs` | 543 | 0 | Positive: stable IDs, 1,024-entry caps and publish-after-state-lock behavior. P0/P1: route-wide gate, two identity maps with linear reverse occupancy scans, linear eviction scans, owned scene keys and formatted hash identity. Hard-cut with the scene transaction owner. |
| `zircon_editor/src/core/document/lifecycle/retention_snapshot.rs` | 120 | 0 | Controlled diagnostics only. It walks both maps and path bytes under the state mutex; retain as explicit profile capture, never a frame/tick query. |
| `zircon_editor/src/core/document/lifecycle/tests.rs` | 408 | 14 | Correctness covers 100K churn, stable IDs, collision stepping, stale tickets and retained-owner bounds. It does not bound occupancy/eviction visits, lock time, allocations or scene-key churn. Replace timing-free scale shape with exact complexity counters while keeping the behavior matrix. |
| `zircon_editor/src/core/document/mod.rs` | 14 | 0 | Facade only; no independent hotspot. Keep the final public surface small after hard cut. |
| `zircon_editor/src/core/document/scene_route.rs` | 282 | 0 | P0: filesystem load/save/publish/rollback, catalog import/reconcile and authoring installation all run inside the lifecycle route gate. Result retains a full `ProjectSceneDocument`. Replace with prepare tickets and a short generation-checked commit receipt. |
| `zircon_editor/src/core/document/scene_route_tests.rs` | 323 | 5 | Good stale-session, duplicate-open, conflict and rollback behavior coverage. Missing delayed-I/O/project-switch, concurrency, single-flight, clone-byte, lock-hold, bounded-admission and durable crash/restart gates. |

The fingerprint streams each workspace-relative path, a zero byte, raw file bytes and another zero
byte in ordinal path order into SHA256. Physical lines use current raw file contents.

## Confirmed architecture and performance failures

### P0: slow scene work is serialized under two editor locks

`DocumentLifecycleAuthority::with_scene_route` holds `scene_route_gate` for the complete closure
(`lifecycle.rs:376-379`). `SceneDocumentRoute::open` then performs project path checks, synchronous
scene file load/decode, authoring-world installation and document activation inside that closure
(`scene_route.rs:67-107`). `create` additionally performs staging save, publish, catalog import,
catalog rollback and source cleanup under the same gate (`scene_route.rs:109-173`).

The retained host adds a broader lock: both submission methods hold `self.shell().lock()` while the
manager executes the entire route (`editor_scene_document_submission.rs:33-72`). The installer
builds an authoring seed and replaces editor state before that shell lock is released
(`editor_scene_document_submission.rs:21-28`). Therefore an open/create request can block unrelated
workbench state access on disk latency, parser/decode work, asset import, catalog refresh, world
construction and failure cleanup.

The current gate prevents a picker result from crossing a project switch by making project/session
mutation wait for all slow work. This is correctness by global serialization, not a scalable
transaction. The correct replacement is prepare outside owner locks followed by a short commit that
revalidates the existing project generation.

### P0: create failure can trigger a project-wide catalog rebuild in the lock domain

The production catalog adapter imports the one scene and then runs
`refresh_from_runtime_project` (`editor_manager_project.rs:292-320`). On rollback it calls
`asset_manager.reimport_all`, probes the URI and again refreshes the full editor projection
(`editor_manager_project.rs:323-351`). This failure path is inside both the shell lock and scene
route gate. Its work scales with project asset/catalog size rather than the one failed scene and can
produce severe tail latency exactly when error handling is already active.

The replacement must consume the Runtime asset generation's typed add/remove delta and durable
transaction outcome. It must not create another editor-private scan or rollback authority.

### P1: scene identity insertion is bounded but still linear and allocation-heavy

The lifecycle state stores separate root and scene-key `BTreeMap` values. Every new ID candidate
linearly scans the values of both maps (`lifecycle.rs:487-530`). Each map is capped at 1,024, but an
ordinary miss can still examine up to about 2,048 IDs. At 100K distinct identities the static upper
shape is about 204.8 million value comparisons before collision stepping, while the existing 100K
test only checks correctness and retention.

Eviction finds the first non-active row by scanning from the beginning, clones its owned key, then
removes it (`lifecycle.rs:397-429`). A scene lookup also owns `PathBuf + String`; a miss formats
`"scene:{root}:{uri}"`, hashes that temporary string through `Path`, and then clones the typed key
into the map (`lifecycle.rs:504-519`). These are not individually the largest F4 cost, but they show
that the registry lacks a canonical typed key, direct occupied-ID index and explicit retention order.

### P1: open/create clone and retain complete scene payloads across boundaries

`PreparedSceneCreation::finish` clones its full `ProjectSceneDocument` instead of moving it
(`scene_document.rs:132-135`). The host installer then clones `Scene` again to prepare the authoring
world (`editor_scene_document_submission.rs:21-25`). Finally,
`SceneDocumentRouteResult::Activated` retains the source `ProjectSceneDocument` after installation
(`scene_route.rs:215-225`), and the retained caller receives that result.

The create success path therefore performs at least two full scene clone operations; the open path
performs at least one and retains both the source scene and prepared authoring result during commit.
The final route result needs IDs, generations, URI and messages, not the complete source scene.

### P2: controlled retention observation is acceptable, but must stay off hot paths

`DocumentLifecycleRetentionSnapshot::from_state` walks both maps and sums path/string bytes while the
state mutex is held. No production tick consumer was found. This is appropriate for explicit
diagnostics and acceptance capture; it must remain a controlled probe and must not become a stable
frame, pane or plugin query.

## Existing behavior worth preserving

- Exact already-active scene requests avoid reload and return no new document messages.
- Root and scene identity retention are hard capped and stable IDs are rederived after eviction.
- Document facts are returned only after the state mutex is released, preventing bus re-entry into a
  partially committed lifecycle state.
- Picker tickets reject another project session or another lifecycle authority.
- Scene creation does not overwrite existing sources and has explicit source/catalog/install
  compensation cases. These behavior contracts must survive, but durability must converge on the
  shared Runtime transaction owner.

## Reference-engine decisions

### Unreal Engine primary reference

Unreal's `UAssetEditorSubsystem` defines a hashed `FAssetEntry` and maintains both
`OpenedAssets: asset -> editor` and `OpenedEditors: editor -> asset`
(`AssetEditorSubsystem.h:422-446`). Exact lookup uses `OpenedAssets.MultiFind`
(`AssetEditorSubsystem.cpp:285`), registration writes both maps (`:399-400`), and close removes both
directions (`:425-447`). Zircon should adapt the direct indexed identity/reverse-identity principle;
it should not copy Unreal's object pointer identity or assume every broad query is constant time.

Unreal's map load is also explicitly observable: `LoadMap` wraps the operation in
`UE_SCOPED_ENGINE_ACTIVITY`, broadcasts `OnLoadMapStart` before validation and `OnLoadMapEnd` after
post-load work (`FileHelpers.cpp:3248-3256, 3374-3377`). The source path is synchronous, so it is not
evidence that Unreal makes every map load asynchronous. It is evidence that a long editor
transaction has named phases and start/end telemetry rather than hiding behind an unmeasured mutex.

### Fyrox corroborating reference

Fyrox routes `LoadScene(PathBuf)` as an editor message (`message.rs:61-70`). Its editor checks a
`loading_scenes` set for duplicate admission, releases that lock, and loads/finishes the scene on the
engine task pool (`lib.rs:2046-2103`). Completion sends `AddScene { scene, path }`; the editor message
owner then constructs and adds the `EditorSceneEntry` (`lib.rs:2608-2631`,
`scene/container.rs:91-150`). This supports keyed single-flight, outside-lock preparation and
message-owned installation. Fyrox's excerpt does not prove a project-generation or durable commit
contract, so Zircon must not copy it as the final consistency model.

### Zircon existing owner to reuse

Frameworks01 already defines `PreparedFullProjectGeneration`, Runtime Resource reservation,
durable staging/journal/commit-point outcomes, project install, Resource apply and generation-last
publication. Scene creation must extend or consume that transaction. A second staging/WAL/project
generation in Editor would recreate the same structural defect.

## Required hard-cut architecture

The target chain is:

`ProjectGeneration -> ScenePreparationTicket -> PreparedAuthoringSceneGeneration -> DocumentRegistryCommit -> RetainedSurfaceDelta`

1. The UI captures `{project_generation, scene identity, intent}` under the shell lock, submits a
   bounded typed job and releases the lock. The request path performs no filesystem, parser,
   catalog, plugin callback or world-build work.
2. Runtime11/Editor14 execute path resolution, file read/decode/validation and eligible authoring
   preparation outside editor authority locks. Admission is keyed by project generation plus scene
   identity, single-flight for the same key, cancellable, and bounded by count/source bytes/decoded
   bytes/age/deadline.
3. Create uses Frameworks01's durable project-generation transaction for scene source and registry
   effects. Runtime04 publishes an exact catalog delta; Editor09 applies the affected rows only.
   Rollback never calls `reimport_all`.
4. The editor owner performs one short main-affinity commit: revalidate the same project generation,
   move the prepared authoring world into Editor05 state through Editor03's fallible exclusive
   transaction, update the document registry and return immutable commit facts. External callbacks,
   message fanout and retained invalidation occur after owner locks are released.
5. Stale/cancelled completions drop prepared data and return an explicit receipt; they never install
   into a newer project. Shutdown and project close have deadlines and retain cleanup ownership.
6. Replace the two maps plus value scans with one canonical typed `DocumentKey`, a direct key index,
   direct occupied/reverse ID index and explicit bounded closed-document retention order. Exact
   lookup/insertion is average O(1), eviction does not scan the cap, and key hashing consumes typed
   root/asset identity components without an intermediate formatted string.
7. Replace `SceneDocumentRouteResult::Activated(ProjectSceneDocument)` with a compact receipt such as
   `{document_id, scene_uri/asset_id, project_generation, authoring_generation, messages/outcome}`.
   Prepared scene/world ownership is moved exactly once; no complete Scene clone crosses commit.

Do not implement this as a larger route mutex, a background thread that still waits synchronously
under the shell lock, another editor catalog snapshot, or a compatibility facade around the current
route. The old gate/result/rollback authorities are deleted when their replacement lands.

## Measurement and acceptance gates

### A1 - registry complexity and ownership

Run roots/scenes `1/100/10K/100K`, path bytes `16/4KiB`, operations `1/1M` and threads `1/16`.
Record key hash/allocation bytes, ID probes, eviction visits/clones, lock wait/hold, map nodes and RSS.
Require known lookup allocation 0, candidate occupancy visits O(1), eviction visits O(1), one
canonical key body per retained document and stable ID/order after eviction/collision.

### A2 - request, prepare and commit separation

Inject 250ms/2s read, decode, import and authoring-build delays. Require UI request lock hold and
scene commit lock hold to exclude those delays, unrelated shell operations to remain serviceable,
same-key prepare count 1, bounded queue age and explicit cancellation/deadline observations.

### A3 - generation and fault matrix

Cover project switch/close during every prepare stage; duplicate same-URI requests; read/decode,
durable prepare/commit, catalog delta, authoring commit, message publish and cleanup failures.
Require stale apply 0, failed generation publish 0, old project/document/world consistency, exactly
once document facts, restart recovery and no full project reimport.

### A4 - payload and clone budget

At scenes `1/1K/100K` entities and payload `1/64/512MiB`, record source bytes, decoded/resident bytes,
complete Scene clone count/bytes, prepared owner count, peak RSS and commit receipt bytes. Require
complete Scene clone count/bytes 0 after decode, one move into authoring ownership, compact receipt
size independent of scene size and bounded cancelled/stale retention.

### A5 - F4 product evidence

After the managed Windows editor bundle is current, run at least 31 comparable samples for cold/warm
open, create, repeated already-active open, project switch during load, failure rollback and close.
Capture WPR/xperf CPU sampling, File I/O, disk usage, waits/locks, context switches, thread activity,
working set and power; correlate typed phase/generation counters. Report p50/p95/p99 and confidence,
not a single stopwatch value. Compare the same source fingerprint, build, project and hardware before
and after.

RenderDoc is not the bottleneck tool for document registry, file I/O or editor lock analysis. Use it
only after a successfully committed scene reaches the first rendered frame, to correlate unexpected
GPU resource/pipeline recreation with the same scene generation. Do not use the existing unrelated
volumetric capture as evidence.

### A6 - completion rule

The module remains in `pending.md` until A1-A5, current-source managed Cargo, rustfmt and functional
fault gates pass. Only then move the module atomically to `review.md`, record the accepted fingerprint
and report measured before/after values. Static review or source integration alone is not acceptance.

## Static gates executed

- Read all 6/6 current module Rust files, all 19 inline tests and the three production anchor files
  cited above. Read the exact Unreal and Fyrox reference sections used for decisions.
- Per-file `rustfmt --check --edition 2024 --config skip_children=true`: 3/6 pass
  (`retention_snapshot.rs`, `mod.rs`, `scene_route.rs`). `lifecycle.rs`, `lifecycle/tests.rs` and
  `scene_route_tests.rs` have current formatting drift. No source was rewritten.
- Scoped `git diff --check` for the module and production anchors reported no whitespace error.
- No separate Python source contract exists for this module. Existing Rust tests were read but not
  executed because the managed product/Cargo path is blocked before a current binary is available.
- WPR/xperf and RenderDoc were not started; there is no current product trace or performance number.
