---
related_code:
  - zircon_runtime/src/asset/project
  - zircon_runtime/src/asset/registry
  - zircon_runtime/src/asset/pipeline
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/02-unreal-aligned-engine-system-hard-cutover.md
  - docs/plans/performance/pending.md
  - docs/plans/performance/review.md
  - docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
  - docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md
  - docs/plans/zircon_editor/editor/09-editor-asset-management.md
doc_type: implementation-evidence
status: routing_blocked_by_protected_plan_owner
created_at: 2026-08-15
---

# Runtime asset project/registry/pipeline protected-plan routing evidence (2026-08-15)

## Coordinator decision

The current-source review is complete in
`2026-08-15-runtime-asset-project-registry-pipeline-current-architecture-review.md`: 118/118 Rust
files, 16,268 physical lines, 71 inline tests and manifest
`6a4e8be301be542dc596861191ce4287af4527f24f66340f7d10ed6cbd03de75`.

Performance01 is authorized only for `docs/plans/performance/01/**`. The numbered runtime/editor
plans and the foreign-dirty `performance/pending.md` and `performance/review.md` remain protected by
their owners. This record requests precise merges without bypassing those boundaries. It is not a
terminal failure handoff: non-validation architecture review can continue while the product build
baseline is repaired.

## Required owner merges

### Performance main plan

Add `PERF-MVP-638` as P0 after PERF-MVP-637 and link it to the existing Runtime04 catalog/watch/
transaction failures, Runtime11 scheduling/backpressure work, PERF-MVP-626/627/636/637 and Plan02
M1/M4:

| ID | Priority | Current root cause | Required hard cut | Acceptance summary |
|---|---|---|---|---|
| PERF-MVP-638 | P0 | `ProjectManager` is a deep-cloned mutable project/registry/import/artifact authority; watch is incremental only for one non-rename event and globally serializes preparation; one targeted update clones broad registry/index state and pretty-serializes all entries; product `ensure_resident` performs synchronous artifact I/O and typed loads clone payloads while the bounded worker pool is not integrated; ready watcher payloads are republished lazy; change/error streams are unbounded and callbacks/sends occur under subscriber/generation locks | Plan02 M1/M4 plus Runtime04/07/11 hard-cut one `RuntimeAssetGenerationStore`: typed add/modify/remove/rename batch journal, immutable indexed generations, dependency-keyed import/residency DAG, stable payload/decoder identity, ready-payload continuity, lease/handle consumers, bounded outside-lock shared delta delivery and asynchronous checkpoint/compaction. Delete all deep-snapshot, exactly-one incremental, full-JSON-per-target, synchronous clone-load, decoder-trial and unbounded callback authorities in the replacing milestones | 118/118 static manifest retained pending. Ordinary K-change source/index/edge work is proportional to K+affected edges/keys; full scan/full JSON=0; deep aggregate/payload clone=0; frame/UI artifact I/O/decode=0; independent keys prepare concurrently; all queues/working sets bounded; callback-under-lock/fence=0. Current Cargo plus 1/1k/100k scale, F0/F4 WPR/xperf, RSS/energy, relevant RenderDoc correlation and functional durability/hot-reload/cancel gates pass |

Do not merge this as a local parallel loop or a larger fixed shard count. The owner/type/generation
contract must land before concurrency; otherwise parallel workers only clone and serialize the same
wrong authority faster.

### Plan02 M1 and M4

M1 must name `RuntimeAssetGenerationStore` as the only project/catalog/registry publication owner,
define stable source/resource/payload/decoder IDs, immutable generation handles, transaction journal
and checkpoint contracts, and include import/residency in the one shared TaskGraph. The milestone
deletes `ProjectManager` value snapshots and duplicate incremental registry authority.

M4 must hard-cut runtime/editor consumers to generation/delta/lease access after Runtime04/11 land.
It must remove clone-returning typed load APIs from steady-state product paths, direct full-catalog
poll/rebuild calls, error-driven decoder trials and editor runtime-project recapture. No aliases,
dual writes or compatibility shims survive the milestone.

### Performance pending/review indexes

Add three concise pending entries, not 118 per-file rows:

- `zircon_runtime/src/asset/project/**`: 41/41 static reviewed, dynamic pending;
- `zircon_runtime/src/asset/registry/**`: 16/16 static reviewed, dynamic pending;
- `zircon_runtime/src/asset/pipeline/**`: 61/61 static reviewed, dynamic pending.

Link the evidence report, physical-line/test totals and manifest. Keep all three out of `review.md`
until current-source Cargo, the A1-A5 hard cut, functional durability/recovery/hot-reload tests,
scale complexity counters, WPR/xperf, RSS/energy and relevant RenderDoc correlation pass. Then move
the same three module entries atomically from pending to review.

### Runtime04 asset pipeline owner

Converge existing Runtime04 catalog generation, management projection, watch/debounce,
transaction-journal and persistence failures into one dependency-ordered implementation:

1. define immutable source/catalog/registry generations and exact typed delta schema;
2. make rename and multi-source batches first-class; reconciliation only follows explicit truth loss;
3. stage changed records/index keys/edges/artifact manifests in an append transaction;
4. publish through stable slots/structural sharing; checkpoint/compact outside interactive commit;
5. carry prepared ready payloads through watcher and explicit import identically;
6. expose generation/query/delta operations and delete whole-project snapshot/staging APIs;
7. add UUID/locator/type/folder/dependency accelerators only for measured product queries.

Do not create a second Runtime04 plan for these findings. This routing strengthens its existing open
failures and supplies the missing complete-current-source evidence.

### Runtime11 task/job owner

Integrate import and residency into the shared keyed task service rather than leaving
`AssetWorkerPool` as a testable standalone facility. Admission must include count, queued/source/
decoded/resident/completion bytes, age, priority, affinity and deadline. Duplicate keys join one
flight; cancellation and publication are generation-safe.

Replace full completion-registry expiry scans with exact timer-key removal or a measured bounded
timer structure. Frame/UI threads may swap completed slots and run budgeted callbacks, but may not
perform artifact I/O/decode or wait on background work except under an explicit boot/commandlet
policy.

### Runtime07 profiling owner

Export existing project-generation phases plus missing counters for project/registry/index/catalog/
payload clone count/bytes; source/meta/artifact reads; duplicate cache reads; index/edge/diagnostic
visits; serialized/journal/checkpoint bytes; queue/working-set dimensions; single-flight joins;
lock/fence wait/hold; subscriber payload owners, queue age and callback-under-lock. Correlate them by
project/run/generation/source/task ID with WPR/xperf.

### Resource, renderer and editor owners

- Framework/resource owner: one ordered artifact/project/resource transaction and a stable
  `ResourceReadinessGeneration`; watcher and explicit import preserve the same ready payload.
- Renderer/material owner: hard-cut render/streaming callers from synchronous clone-returning loads
  to leases/tickets after the residency service lands; PERF-MVP-636 remains the canonical material/
  shader/artifact identity predecessor.
- Editor09: consume runtime immutable generations and typed deltas as PERF-MVP-637 requires; no
  `ProjectManager` snapshot, full registry capture or source/artifact I/O under editor locks.
- Plugin/runtime boundary owner: plugin importers/loaders submit stable typed tasks and capability
  requests; no Rust project/registry aggregate crosses the dynamic plugin boundary.
- Optimize indexes: local String/sort/vector/shard tweaks remain post-hard-cut measurement work and
  must not be promoted into a permanent parallel architecture.

## Dynamic acceptance sequence

1. repair the approved D/E/F build-root separator defect and produce a current editor/runtime binary;
2. freeze 1/1k/100k deterministic fixtures on E: or D: and archive fixture/source fingerprints;
3. run cold/warm open, one/two/32-event add/modify/remove/rename, 1% dependency change and recovery;
4. collect complexity/copy/I/O/queue/lock/callback/working-set counters plus WPR/xperf at least three
   times per product scenario;
5. use RenderDoc only to correlate resulting texture/mesh/material upload/copy/pass effects on a
   stable F2/F4 frame;
6. report median/range, hardware/power plan, profiler overhead and before/after evidence; do not use
   reference-engine source as a numeric substitute;
7. after all static, functional and dynamic gates pass, update pending/review, commit the accepted
   milestone and send quantified WeCom evidence.

## Completion condition

This routing record can be retired only after protected owners merge the task/index updates, the
current product baseline runs, the hard cut deletes all legacy authorities and the cited functional,
complexity, WPR/xperf, RSS/energy and relevant RenderDoc gates pass. Until then the module is
static-complete/dynamic-blocked and no milestone commit or WeCom completion message is permitted.
