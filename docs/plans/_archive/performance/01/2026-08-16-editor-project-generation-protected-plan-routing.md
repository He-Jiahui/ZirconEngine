---
related_code:
  - zircon_editor/src/core/project
  - zircon_editor/src/ui/host/startup
  - zircon_editor/src/ui/retained_host/app/welcome_session
  - zircon_runtime/src/asset/project
  - zircon_runtime/src/asset/pipeline/manager
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/02-unreal-aligned-engine-system-hard-cutover.md
  - docs/plans/performance/pending.md
  - docs/plans/performance/review.md
  - docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md
  - docs/plans/zircon_editor/editor/10-project-and-asset-reference-management.md
  - docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - docs/plans/zircon_runtime/frameworks/01-runtime-crate-decomposition.md
  - docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
doc_type: implementation-evidence
status: routing_blocked_by_protected_plan_owner
created_at: 2026-08-16
---

# Editor project generation protected-plan routing evidence (2026-08-16)

## Coordinator decision

The current-source review is complete in
`2026-08-16-editor-project-generation-current-architecture-review.md`: 21/21 Rust files, 3,485
physical lines, 52 inline/module tests and fingerprint
`118ade0bf8275e9fa8d228f1671c34e3d9d275e214ce1d72c789322a9139b454`.

The main performance plan, `pending.md`, `review.md` and numbered owner plans contain foreign
concurrent changes and are not rewritten by this session. This record requests exact merges. It is
not a terminal blocker: review of non-validation editor modules continues while the managed product
build baseline is repaired.

## Required owner merges

### Performance main plan

Replace the stale `core/project/**` scope paragraph with the 21/21 fingerprint and rebase existing
tasks rather than adding a duplicate umbrella task:

| Existing task | Required rebase |
|---|---|
| `PERF-MVP-075` P0 | Make `ProjectIntent -> ProjectIdentityTicket -> PreparedProjectGeneration -> EditorProjectCommit -> RecentProjectDelta` the canonical chain. Include startup validate/open/validate, accepted Welcome result promotion, project root capability reuse and one prepared Runtime generation. Link PERF-MVP-559/638/640. |
| `PERF-MVP-100` P1 -> P0 | Raw startup session bytes/schema/entries must be capped before legacy migration allocation or per-row filesystem I/O. Stable recent projections consume a bounded last-good generation and do zero snapshot I/O. |
| `PERF-MVP-568` P0 | Template creation must reuse the Frameworks01 durable transaction and Runtime04 generation, carry one typed manifest artifact plus shared unchanged bytes, create unique parents once and delete editor post-write load/save/reopen. |
| `PERF-MVP-559` P0 | Preserve the existing single-flight/debounce/cancellation implementation; make its accepted result the promotable PERF-MVP-075 identity ticket. Do not add a second probe cache or debounce owner. |
| `PERF-MVP-638/637` P0 | Remove `ProjectManager: Clone`, `current_project_snapshot`, candidate aggregate clones and editor full-project captures after consumers use immutable runtime generation handles, exact queries and typed deltas. |
| `PERF-MVP-640/453` P0 | Scene open/create/save consumes the same project generation and root capability, uses the Frameworks01 targeted transaction and Runtime04 exact delta, and never compensates with a project-wide rescan. |

The main-plan acceptance clause must link A1-A7 from the current review. Do not implement this as a
mutex reduction, a larger cache, an editor-private worker or WAL, or a compatibility wrapper around
clone-returning project snapshots.

### Performance pending/review indexes

Update the existing concise `zircon_editor/src/core/**` accounting with one module clause:

- `core/project/**`: current 21/21 static reviewed, 3,485 physical lines, 52 tests, fingerprint
  `118ade0bf8275e9fa8d228f1671c34e3d9d275e214ce1d72c789322a9139b454`; PERF-MVP-075/100/568 own
  project generation, bounded startup ingress and durable template creation; PERF-MVP-559/638/640
  own probe, immutable runtime generation and scene integration; dynamic gates remain pending.

Keep the module out of `review.md` until the exact fingerprint passes current-source Cargo,
functional/fault tests, A1-A7 scale counters, F0/F4 WPR/xperf, RSS/energy and relevant first-frame
RenderDoc correlation. Move the one module entry atomically only with measured before/after evidence;
do not create 21 index rows.

### Plan02 hard-cut milestones

- M1 owns the shared task, durable project generation, immutable runtime generation store and root
  capability contracts. It prohibits editor-private pools/WALs and clone-returning project state.
- M2 makes scene preparation consume a project generation/source ticket and return a move-owned
  authoring generation. It shares PERF-MVP-640's document/scene transaction.
- M4 cuts editor/UI composition to intent, prepare ticket, short generation commit and compact
  retained deltas. Startup, Welcome, asset, document, plugin and watcher consumers share one lineage.

### Frameworks01 durable transaction owner

Extend the existing durable project-generation transaction with typed template/create operations;
do not create a second editor transaction. The result must carry exact created files, staged
manifest/registry effects, commit-point disposition, restart recovery and cleanup receipt. Drop does
no recursive deletion or waiting. R8's `DurableCommitDisposition` remains authoritative.

### Runtime04 project/asset owner

Own `ProjectIdentityTicket`, the project root capability, one typed manifest artifact and one
immutable `RuntimeAssetGenerationStore`. Open/create prepares a candidate once, performs recovery
and exact registry/catalog work once, then publishes by generation. Remove deep `ProjectManager`
clones and full reconciliation from normal one-project/one-scene changes; retain explicit recovery
reconciliation for truth loss.

### Runtime11 task owner and Editor14 admission owner

Run project identity, manifest/template I/O, migration, recovery and registry preparation as shared
keyed jobs with count, source/decoded/output bytes, queue age, priority, deadline, cancellation and
single-flight joins. Main/UI may submit and apply a compact completion but may not wait on or perform
filesystem/import work. Close/shutdown has a bounded terminal receipt.

### Editor10 project identity owner

Cap raw startup input before allocation/I/O; retain at most eight deduplicated last-good recent rows;
promote accepted Welcome/startup tickets after mutation-stamp revalidation; move-commit one prepared
project generation. Delete summary-only probe acceptance, validate-open-validate startup and value
snapshot APIs. Project/scene paths are relative typed identities under the generation root
capability, not newly canonicalized strings per consumer.

### Editor01 and EditorUI08 owners

Editor01 owns a short generation-checked project commit that installs editor state and emits facts
after releasing locks. EditorUI08 submits intents and projects compact recent/workbench deltas.
Neither shell nor project locks may cross canonicalization, manifest/template I/O, durable commit,
registry work, plugin callbacks, watcher setup or waits.

### Editor09, plugin and watcher consumers

Editor09 consumes Runtime04 immutable generations and exact asset deltas. Plugin capabilities and
watchers bind to stable project generation handles and receive bounded invalidation/close events;
the dynamic plugin boundary never receives a cloned Rust project aggregate. Project switching
invalidates one lineage and does not rebuild independent per-consumer project caches.

### Runtime07 profiling owner

Export correlated project intent/ticket/generation/commit IDs and counters for session bytes/rows,
canonical/link/handle operations, manifest reads/parses/encodes, template clone/mkdir/write/fsync,
durable journal disposition, registry preparation, aggregate clone bytes, queue dimensions,
lock/wait time and compact delta bytes. Stable paths must still be observable with near-zero disabled
overhead.

## Dependency and deletion order

1. Freeze raw ingress limits, canonical project/scene identity, root capability, generation and
   durable disposition contracts.
2. Extend the one Frameworks01 transaction and Runtime04 prepared generation; add counters without
   changing publication ownership.
3. Add Runtime11/Editor14 bounded keyed preparation and promotable Welcome/startup tickets.
4. Publish `RuntimeAssetGenerationStore`; migrate runtime/editor/plugin/watcher consumers to handles,
   exact queries and deltas.
5. Cut Editor01/10 project commit and EditorUI08 retained projection to compact receipts.
6. Integrate PERF-MVP-640/453 scene open/create/save with the same root and durable generation.
7. Delete deep project snapshots/candidate clones, repeated probe/validation, editor template
   reload/rewrite/reopen, per-scene project ancestor handle rebuild and ordinary full-scan
   compensation. No alias, fallback, dual write or compatibility shim survives.

## Required verification return

- exact current manifests plus the deleted legacy symbols and paths;
- deterministic A1-A6 functional, scale, complexity, copy, I/O, queue, lock and durable fault/restart
  evidence on D/E/F only;
- current managed Windows Cargo/build/stage/run evidence;
- at least 31 comparable F0/F4 WPR/xperf samples with median/range, CPU/File I/O/waits/locks/CSwitch,
  RSS, power mode and profiler overhead;
- RenderDoc only for relevant first-frame upload/copy/pass correlation after project/scene commit;
- independent review, scoped milestone commit and quantified WeCom message only after all gates pass.

Until these returns exist, the module remains `static_complete / dynamic_blocked`; no throughput,
latency, power, optimality or parity claim and no milestone commit/WeCom completion message is
permitted.
