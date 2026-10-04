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
  - docs/plans/optimize/zircon_runtime/04-core-resource-asset-serialization-review.md
  - docs/plans/zircon_editor/editor/10-project-and-asset-reference-management.md
  - docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md
  - docs/plans/zircon_runtime/frameworks/01-runtime-crate-decomposition.md
  - docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Editor/GameProjectGeneration/Private/GameProjectUtils.cpp
  - dev/UnrealEngine/Engine/Source/Editor/GameProjectGeneration/Private/SProjectBrowser.cpp
  - dev/godot/editor/project_manager/project_list.cpp
tests:
  - 21 of 21 current Rust files reconciled and reviewed
  - 3485 physical lines and 52 inline or module tests
  - ordered path and raw-content SHA-256 118ade0bf8275e9fa8d228f1671c34e3d9d275e214ce1d72c789322a9139b454
  - managed current-source Cargo and product WPR/xperf remain blocked by the non-runnable editor baseline
doc_type: implementation-evidence
status: static_complete_dynamic_blocked
created_at: 2026-08-16
---

# Editor project generation current architecture review (2026-08-16)

## Scope freeze and method

This review supersedes the 2026-07-30 18-file project review for current-source accounting. It
freezes `zircon_editor/src/core/project/**` at **21/21 Rust files, 3,485 physical lines and 52 inline
or module tests**. The fingerprint is
`118ade0bf8275e9fa8d228f1671c34e3d9d275e214ce1d72c789322a9139b454`; it streams each sorted
workspace-relative path, a zero byte, the raw file bytes and a zero byte into SHA-256.

Every Rust file was read in full, including support DTOs and all test modules. Production
reachability was followed through startup restoration, recent-project projection, the retained
Welcome probe, project creation/open, Runtime project activation, scene load/create and project
snapshot consumers. Ten files in this root contain foreign uncommitted changes. They were reviewed
as current source and were not rewritten.

The approved D/E/F build-root separator failure at `tools/build-editor.ps1:130` still prevents a
current editor executable. WPR/xperf measurements against a stale executable would not validate
this fingerprint. RenderDoc is not a CPU/filesystem validator for this module and is reserved for
the first rendered frame after a committed project/scene generation. All latency, throughput,
power and before/after fields therefore remain `not_measured`; this module remains in `pending.md`.

## Architecture verdict

The P0 defect is not one slow filesystem call. Project identity, project preparation, runtime
activation, editor publication and recent-project presentation are represented as independently
repeatable operations. Accepted work is discarded between those phases, while the active project is
exposed as a deeply clonable mutable aggregate. The current product can therefore repeat canonical
path walks, manifest reads/parses, registry preparation, complete aggregate clones and recent-list
validation for one user intent.

The required hard-cut chain is:

`ProjectIntent -> ProjectIdentityTicket -> PreparedProjectGeneration -> EditorProjectCommit ->
RecentProjectDelta`

`ProjectIdentityTicket` owns the canonical path identity, descriptor fingerprint, security result
and a project-root capability/lease. `PreparedProjectGeneration` owns the one prepared Runtime04
asset/project generation and the Frameworks01 durable outcome. The editor performs one short
generation-checked move commit and publishes compact deltas. Runtime and editor consumers retain
immutable generation handles; no consumer receives a value-cloned `ProjectManager`. Startup and
Welcome project rows are last-good projections refreshed by bounded jobs, never a reason for UI
snapshot I/O.

## Current-source findings

### P0: startup validates the same project generation repeatedly

`ui/host/startup/resolve_session.rs:23` validates every stored recent project. It then opens the last
row at line 58, persists it again at line 73, and calls `recent_projects_snapshot` at line 79.
`ui/host/startup/recent_projects.rs:11-16` reloads the session and validates every recent project
again. The last manifest can therefore be validated, opened with Runtime recovery/registry work and
validated again during one startup. `ProjectManager::open_resolved` additionally loads the manifest,
ensures project layout, recovers durable work and loads or rebuilds the asset registry. This
rebaselines PERF-MVP-075; it is not evidence for a new parallel cache.

### P0: the accepted Welcome probe cannot be promoted

The background probe calls `ProjectAuthority::probe_draft`, but
`welcome_session/project_probe.rs:346-360` reduces its result to validity and diagnostic projection.
The click path at `welcome_session/actions/project.rs:63-72` calls `probe_draft` synchronously and
then opens by path. Canonical root and manifest summary in `ProjectProbe` are discarded, and the
result lacks a mutation stamp or prepared generation needed for a safe promotion. PERF-MVP-559's
bounded debounce/single-flight behavior should remain; PERF-MVP-075 must supply a promotable ticket
with commit-time identity revalidation.

### P0: template creation has multiple ownership and parse phases

`ProjectAuthority::create_project` validates the draft before rendering, while
`create_rendered_project` resolves and validates the target again. It calls `create_dir_all` and
`fs::write` for each rendered entry (`authority.rs:90-100`), then reloads and saves the manifest
(`authority.rs:107-109`) before publishing and opening the Runtime manager. Runtime open loads the
manifest again and may load/rebuild the registry. Failure cleanup can recursively remove staging on
the caller thread. The safe target is PERF-MVP-568: one typed manifest/template artifact, shared
unchanged entry bytes, unique parent preparation, one Runtime11 write ticket and one Frameworks01
durable transaction. An editor-private journal is forbidden.

### P0: the active project API deep-clones runtime authority

`zircon_runtime/src/asset/project/manager/mod.rs:31-43` derives `Clone` for `ProjectManager`, which
owns paths, manifest, resource and asset indexes, package registry, catalog/import/artifact state,
shader dependencies and task-pool state. `AssetManagerContract::current_project_snapshot`
(`service_contracts/asset_manager_contract.rs:61-63`) clones it. Editor project activation,
scene routing, layout, asset editors and watchers call this snapshot API; `scan_and_import` also
clones the complete manager before preparation (`scan_and_import.rs:88,100`). This is the same root
cause already owned by PERF-MVP-638/637: one immutable `RuntimeAssetGenerationStore` plus compact
mutation tickets, not a faster deep clone.

### P0: hostile startup input is bounded after migration work

`decode_startup_session` migrates raw JSON before enforcing a typed size/entry bound. Legacy
migration reserves `Vec::with_capacity(recent_projects.len())` at `authority.rs:333` and may probe
every row missing a summary. Normal writes truncate to eight entries, but an old, corrupt or hostile
file can allocate and perform per-row canonical/manifest I/O first. PERF-MVP-100 must cap raw bytes,
schema and entry count before migration or filesystem access, then perform stable deduplication.

### P0: scene path protection reconstructs a project-wide handle chain per operation

`protect_scene_path` first calls `reject_linked_components`, which performs metadata checks for
existing ancestors. On Windows it then builds guarded paths from volume root through project root
and the relative scene parents, allocates a vector and opens a no-follow handle for each path
(`filesystem.rs:132-253`). This is a defensible security boundary, but every scene open/create
reconstructs it. The project generation should own one platform root directory capability and
resolve descendant operations relative to it, with an exact relative source ticket. No security
check may be removed to improve a benchmark. This contract extends PERF-MVP-075/640; local reference
engines are not cited as proof for Zircon's platform-security mechanism.

### Existing progress to preserve

- `open_project` resolves once and transfers one prepared manager into the host; the retired
  Authority/Runtime/Editor three-manager reopen chain is not current.
- normal recent-project writes deduplicate and retain eight rows;
- the Welcome probe already has trailing debounce, maximum feedback delay, one pending plus one
  active generation and cancellation points;
- project creation has staging, backup, rollback and fault-injection coverage;
- Frameworks01 owns the single durable project-generation transaction and typed deferred-recovery
  outcomes;
- Runtime04 has targeted import and durable recovery primitives. Full reconciliation remains valid
  only for explicit truth loss or recovery, not ordinary project open/save/create.

## Per-file review

| File | Tests | Current-source performance result |
|---|---:|---|
| `authority.rs` | 2 | Owns startup decode/migration, create/open/probe/recent orchestration. Prepared open is preserved; raw legacy migration, repeated recent validation and synchronous multi-phase template publication remain structural bottlenecks. |
| `created_project.rs` | 0 | Move-transfers the prepared manager, but derives a cloneable wrapper around the deep aggregate and duplicates root/summary presentation data. No independent steady-frame loop. |
| `error.rs` | 1 | Typed source/rollback/scene errors only. It does not own retries or queues and should preserve exact durable outcomes. |
| `filesystem.rs` | 2 | Enforces canonical ownership and link/reparse safety. Per-scene ancestor metadata plus handle-chain reconstruction must become a generation root capability without weakening security. |
| `mod.rs` | 0 | Narrow declarations/re-exports only. No algorithmic work. |
| `new_project_draft.rs` | 3 | Small validation and target construction. Revalidation at commit is required; accepted background validation should carry a ticket instead of repeating all preparation. |
| `new_project_template.rs` | 0 | Enum-to-pack identifier mapping only. No hot-path work. |
| `opened_project.rs` | 0 | Move boundary for a prepared manager. Target is an immutable generation handle plus compact presentation receipt, not a clonable authority. |
| `project_probe.rs` | 0 | Carries canonical root and cloned summary but no physical identity, mutation stamp, root lease or prepared generation, so accepted probe work cannot be safely promoted. |
| `recent_project_entry.rs` | 0 | Compact UI row. Cost is acceptable only after persisted input is bounded and rows are projected from a retained validation generation. |
| `recent_project_validation.rs` | 0 | Small result enum; no local performance issue. |
| `scene_document.rs` | 0 | Owns full `Scene` clones, synchronous path guards and create/import/rollback. `finish` clones the document and failure can run project-wide import; PERF-MVP-640 is canonical. |
| `stored_recent_project_entry.rs` | 0 | Raw DTO has no local bounds. Raw session ingress must reject/cap before converting or probing entries. |
| `stored_startup_session.rs` | 0 | Unconstrained persisted `Vec`; normal eight-row truncation does not protect decode/migration input. |
| `tests/boundary.rs` | 1 | Guards module ownership and retired template paths. No complexity or I/O counter. |
| `tests/directory_transaction.rs` | 7 | Covers staging/backup/rollback failure windows. Preserve semantics while moving file work to the shared durable transaction. |
| `tests/mod.rs` | 1 | Test wiring and E-drive temporary-root helper. No product behavior. |
| `tests/recent_projects.rs` | 9 | Covers ordering, dedup, validation and migration behavior. Missing raw pre-cap byte/entry/I/O scale and retained-generation no-work assertions. |
| `tests/root_resolution.rs` | 6 | Covers canonical root/link/reparse/alias cases. Extend with root-capability reuse and replacement/stale-ticket cases; never relax existing checks. |
| `tests/scene_document.rs` | 5 | Covers scene identity and document behavior. Full clone bytes, root-handle opens, stale generation and targeted rollback remain unmeasured. |
| `tests/template_creation.rs` | 15 | Broad correctness/fault coverage. Missing shared-byte, unique-directory, single parse/encode, durable restart and off-main-thread scale counters. |

## Required hard-cut plan

1. **Bound external state before work.** Editor10 enforces startup-session byte, schema and entry
   limits before JSON migration, allocation or project probing. The retained recent generation is
   always at most eight stable deduplicated rows.
2. **Create one project identity ticket.** Runtime04/Platform resolution produces canonical logical
   identity, physical file identity, descriptor digest/mutation stamp and one root capability.
   Welcome and startup retain the accepted ticket; commit revalidates only its mutation evidence.
3. **Prepare once outside editor locks.** Runtime11 admits keyed open/create work with count, source
   bytes, decoded bytes, age, priority, deadline and cancellation budgets. Equal identities join one
   flight. Preparation returns one move-owned `PreparedProjectGeneration`.
4. **Reuse the single durable owner.** Template creation submits typed manifest plus shared static
   entries to Frameworks01's durable project transaction. Runtime04 stages exact registry/catalog
   effects in that same generation. No editor WAL, duplicate staging owner or post-write reopen
   remains.
5. **Publish an immutable runtime generation.** Runtime04 installs one
   `RuntimeAssetGenerationStore` through a short generation check. Remove `ProjectManager: Clone`,
   `current_project_snapshot`, candidate deep clones and complete-project editor captures after all
   consumers migrate to handles, queries, leases and deltas.
6. **Commit editor state briefly.** Editor01/10 installs the project/document/settings/plugin/watcher
   lineage under a short lock and publishes facts after releasing it. Foreign callbacks, filesystem
   work, registry preparation and waits cannot occur in this commit section.
7. **Project compact deltas.** EditorUI08 updates recent rows, Welcome state, workbench project
   identity and affected asset/document surfaces from a compact typed delta. Stable snapshots do no
   filesystem work and allocate no project aggregate.
8. **Delete legacy paths in the same milestone.** Remove summary-only accepted probe projection,
   validate-open-validate startup, deep project snapshots, per-operation project-root guard rebuild,
   editor template load/save/reopen and full-scan compensation. No alias, fallback, dual write or
   compatibility shim survives.

## Complexity and measurement acceptance

| Gate | Frozen input matrix | Required counters | Acceptance invariant |
|---|---|---|---|
| A1 startup/recent | session `1KiB/1MiB/64MiB`, rows `0/8/1K/1M`, current/legacy/malformed | bytes read before cap, allocations, rows migrated/probed, canonical/stat/read/parse, UI wait, RSS | oversized input stops before per-row I/O; accepted last project has one identity/manifest preparation; stable row projection I/O is zero |
| A2 probe/open | paths `1/100/1K`, manifest `1KiB/64MiB`, unchanged/replaced/link | ticket joins/cancels, canonical/link walks, manifest reads/parses, Runtime opens/recovery/registry work, caller wait | one accepted intent has at most one prepare generation; replaced/stale ticket is rejected; no security downgrade |
| A3 create | entries `1/1K/100K`, bytes `1KiB/1GiB`, shared/deep parents, injected phase faults | cloned bytes, unique parents, mkdir/write/fsync, manifest parse/encode, journal bytes, cleanup time/RSS | unchanged entry clone bytes zero; one typed manifest generation; parent creation near unique parents; one durable owner; Drop I/O zero |
| A4 active project access | registry rows `1/1K/100K`, consumers `1/16/128` | aggregate clone count/bytes, Arc/lease acquisitions, lock wait/hold, query rows | `current_project_snapshot` calls and deep project clone bytes zero; generation acquisition O(1); queries proportional to requested keys |
| A5 root capability | path depth `1/16/128`, scene operations `1/1K` | metadata calls, `CreateFileW`/relative opens, handle count, path/wide allocations | project ancestors above root are acquired once per generation; each scene operation opens only required descendants; link/reparse tests remain GREEN |
| A6 durable faults | prepare/write/replace/commit/sync/restart/cleanup fault points | dispositions, live rollback, restart recovery, orphan cleanup, duplicate WAL count | exactly one Frameworks01 transaction owner; old or new complete generation after restart; editor-private WAL count zero |
| A7 product trace | F0 cold/warm startup/open/create/close and F4 project/scene switch, at least 31 comparable samples after baseline repair | WPR CPU/File I/O/waits/locks/CSwitch/RSS/power plus generation phase IDs | no UI/main filesystem wait in stable projection or background prepare; report median/range and profiler overhead; no comparison claim without measured data |

RenderDoc is used only to correlate the first rendered frame after a committed project or scene with
asset upload/copy/pass effects. It cannot accept startup JSON, canonicalization, file transactions,
locks, deep clones or power behavior.

## Reference evidence and boundary

- Unreal `GameProjectUtils::CreateProject` (`GameProjectUtils.cpp:843-877`) exposes one named slow
  task and explicit create failure cleanup. Template creation builds `FilesToCopy` and
  `FilesThatNeedContentsReplaced` (`:1742-1842`) and tracks exact created files for cleanup
  (`DeleteCreatedFiles`, `:2099-2115`). Zircon adopts explicit named phases, progress and exact
  created-file ownership. Unreal's synchronous recursive copy and full-file replacement passes are
  not adopted as an optimal algorithm.
- Unreal `SProjectBrowser::FindProjects` has a CPU profiler scope (`SProjectBrowser.cpp:809-812`),
  deduplicates candidates with `TSet` (`:821`) and builds retained project items before sorting
  (`:758-904`). It may still re-query project status on open, so it is not evidence that Zircon's
  repeated validation is desirable. It supports named measurement, candidate deduplication and a
  retained presentation generation.
- Godot `ProjectList::_scan_thread`/`_scan_finished` (`project_list.cpp:712-744`) perform discovery in
  a cancellable thread and publish afterward; `load_project_data` (`:793-870`) loads one project
  record, and `scan` starts the thread at `:1050-1081`. This corroborates background discovery and
  retained projection. Zircon keeps stronger canonical/link identity and uses the shared Runtime11
  scheduler rather than a project-private thread.
- `docs/plans/optimize/zircon_runtime/04-core-resource-asset-serialization-review.md` is used as a
  design constraint for exact typed generations, artifact identity and transactions. Its local
  container/string improvements are post-hard-cut measurement work, not substitutes for the owner
  change above.

Reference engines establish behavior and ownership patterns, not Zircon timing targets. Numeric
latency, throughput and power comparison requires identical hardware, fixtures and profiler setup;
no such comparison is claimed by this static review.

## Static validation record

- all 21/21 Rust files were read and reconciled at the frozen fingerprint;
- `rustfmt --edition 2021 --check` passed for all 21/21 Rust files;
- `git diff --check` passed for `zircon_editor/src/core/project/**` and both new evidence records;
- all 32 `related_code`/`plan_sources`/`reference_sources` paths across the two records exist;
- the documentation convention gate found zero violation in these two records. The repository-wide
  gate remains RED with 652 pre-existing path violations across 235 of 2,496 documents; those
  unrelated findings were not rewritten or attributed to this module;
- coordinator plan audit and session heartbeat completed successfully at this fingerprint;
- current-source Cargo, fault/scale counters, F0/F4 WPR/xperf, RSS/energy and relevant first-frame
  RenderDoc correlation remain pending behind the managed editor build failure;
- no Rust source was changed because the required repair is cross-owner and ten scoped files contain
  foreign concurrent modifications.

Until A1-A7 and the current-source functional gates pass, `zircon_editor/src/core/project/**` stays
out of `review.md`; no performance improvement, optimality, engine parity, power reduction or
milestone completion is authorized.
