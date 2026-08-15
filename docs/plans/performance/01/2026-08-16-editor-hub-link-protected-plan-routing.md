---
related_code:
  - zircon_editor/src/core/hub_link
  - zircon_editor/src/core/project/authority.rs
  - zircon_editor/src/ui/host/startup
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/pending.md
  - docs/plans/performance/review.md
owner_plans:
  - docs/plans/zircon_editor/editor/10-project-and-asset-reference-management.md
  - docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md
  - docs/plans/zircon_editor/editor/16-cli-args-and-hub-integration.md
source_evidence:
  - dev/UnrealEngine/Engine/Source/Runtime/Projects/Public/ProjectEditorRecords.h
  - dev/UnrealEngine/Engine/Source/Runtime/Projects/Private/ProjectEditorRecords.cpp
  - dev/UnrealEngine/Engine/Source/Editor/GameProjectGeneration/Private/SProjectBrowser.cpp
---

# Protected plan routing: editor Hub link

## Reason for routing

Performance01, `pending.md`, `review.md` and Editor16 are protected/foreign dirty in this session.
The complete `core/hub_link` module is also foreign untracked work. This record requests
current-source plan corrections without overwriting any owner. Canonical evidence is
`2026-08-16-editor-hub-link-current-architecture-review.md`.

## Requested Performance01 correction

Record `zircon_editor/src/core/hub_link/**` as **6/6 Rust files, 721 physical lines, 3 tests** with
ordered path-and-raw-content fingerprint
`e1eccf96c9abb19758af48d17bd7644079ae743ea90dd5f22cc7c16ec66671a0`.

Retain these current facts:

- handshake is a one-shot atomic mailbox and focus uses one non-recursive OS watcher, not UI-frame
  polling;
- project liveness remains exclusively in Editor17 `SessionGuard`;
- recent-project mutation performs system-wide lock, read/decode/validate, mutation, revalidation,
  pretty encoding and atomic replacement synchronously;
- Windows uses an infinite lock wait, and the mutation is inside project activation before document
  session/guard commit, so optional history contention or failure can reject a usable project;
- startup restore reads/probes every recent row, opens and writes the chosen project, then reads and
  probes the list again; Welcome presentation/removal can repeat the same synchronous path;
- the list limit is eight, so collection sorting is not the root bottleneck. Storage/lock placement,
  repeated filesystem validation and authority coupling are.

## Proposed PERF-MVP-643

| id | priority | current diagnosis | required cutover | acceptance |
|---|---|---|---|---|
| PERF-MVP-643 | P0 | A recoverable recent-history projection blocks project activation on an infinite cross-process wait and full read-modify-write. Startup/Welcome repeatedly reload and re-probe the bounded list. | Editor10/14/16 publish one `HubRecentProjectsGeneration` through a bounded ordered projection lane. Project/session commit is independent; record/remove intents coalesce by canonical key; finite-lease read-merge-write owns retry/terminal receipts; startup reads once, project open reuses authoritative validation and remaining health probes publish affected-row deltas. Keep existing event-driven focus/handshake and delete no `SessionGuard` authority. | rows `0/1/8`, writers `1/2/16`, delay `0/10/100/1000ms`, bad/abandoned inputs: project main-thread lock/I/O wait 0; read/decode <=1/file generation; chosen manifest validation duplicate 0; queues/bytes/age bounded; finite lock deadline; deterministic order/retry; focus frame polls 0; current managed tests and F0/F1 WPR CPU/wait/file-I/O/RSS/power pass |

## Requested owner-plan updates

### Editor10

Make `ProjectAuthority`/`SessionGuard` commit the only project availability authority. Recent history
is a projection and cannot roll back an opened project. Reuse the authoritative open/probe result for
the selected restore candidate. Publish health changes only for affected rows rather than loading
every manifest on every UI snapshot.

### Editor14

Provide the single ordered projection lane using the existing job system. Bound pending/running/
result entries, bytes and age; coalesce by canonical project key; expose finite deadline, cancellation,
retry, shutdown flush and phase counters. Do not add a Hub-specific worker pool or Condvar wait.

### Editor16

Preserve the v1 atomic handshake, event-driven focus watcher and Editor17 liveness boundary. Replace
direct recent-project writeback with the projection service, and make Hub consume the same typed
generation/file contract. Add deterministic fake-filesystem delay and real two-process lease tests.

### Render17

Add F0/F1 marks for registry load/decode, row probes, intent admission, lock wait, merge/encode/write,
retry and UI generation application. RenderDoc is not an acceptance tool for this CPU/process slice.

## Requested protected index state

- `pending.md`: add the frozen module counts/fingerprint, `static_complete / dynamic_blocked`, the
  canonical review and PERF-MVP-643.
- `review.md`: do not add the module until PERF-MVP-643, current managed Cargo, two-process
  contention and 31-run F0/F1 WPR CPU/RSS/file-I/O/power gates are green.

## Milestone and notification state

This is static architecture evidence only. The product build blocker prevents dynamic acceptance,
so no performance milestone commit or WeCom notification is due. Commit and quantified WeCom report
become mandatory after owner-plan acceptance and the dynamic matrix passes.
