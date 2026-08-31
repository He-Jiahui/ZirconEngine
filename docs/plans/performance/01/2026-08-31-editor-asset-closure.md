---
related_code:
  - zircon_editor/src/core/asset
  - zircon_editor/src/core/jobs
  - zircon_editor/src/core/notifications
  - zircon_editor/src/ui/host/asset_editor_sessions
  - zircon_editor/src/ui/host/editor_asset_manager
  - zircon_editor/src/ui/asset_editor/preview
  - zircon_runtime/src/asset
related_plans:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/02-unreal-aligned-engine-system-hard-cutover.md
  - docs/plans/performance/01/2026-08-15-editor-asset-refresh-preview-current-architecture-review.md
  - docs/plans/performance/01/2026-08-16-editor-core-asset-save-registry-current-architecture-review.md
  - docs/plans/performance/01/2026-08-31-editor-jobs-closure.md
  - docs/plans/zircon_editor/editor/09-editor-asset-management.md
  - docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md
  - docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
write_scope: []
status: pending
---

# Editor asset closure

This is a current-source static revalidation of the editor asset dirty/save,
import, index, refactor and type-registry owners. It remains pending: managed
editor Cargo is not green and no current product executable is available for
F1/F4 timing, memory or power evidence. No Rust source was changed.

## Scope and source state

- `zircon_editor/src/core/asset/**`: 53 Rust files, 11,628 physical lines,
  10,419 nonempty lines, 384,429 bytes, 113 test attributes, 12 ignore
  attributes and 16 include sites. Sorted path plus NUL plus raw-content SHA256:
  `396a016e7aee0f75fdadfcf38a27a2dcf39a98a02f35cd42a7a20bc41346cc7e`.
- The scope includes foreign modified and untracked dirty, import-flow,
  index, refactor and type-registry work. Existing changes are preserved; no
  reconciliation edit was attempted.
- Prior 2026-08-15/16 asset reviews remain useful historical evidence, but this
  fingerprint and the current 53-file source are authoritative for this record.
  RenderDoc is not applicable to this CPU/control slice; thumbnail upload and
  Browser presentation remain with their render/UI owners.

## Positive work to preserve

- Dirty state has a bounded generation journal and cursor-based deltas; external
  effect identifiers and document snapshots use ordered, borrowed lookup paths.
  Save preflight validates duplicate documents, toolkit identity, write policy,
  references and estimated bytes before completion application.
- Import flow coalesces the exact `(UUID, URI, source digest)` generation, keeps
  one active UUID transition, bounds flights/result bytes/age and contains
  backend panics. Import and model tickets expose nonblocking `try_take` paths.
- The editor index validates metadata against the runtime registry and tracks
  document membership, dirty/importing UUIDs and pending watch paths. Refactor
  delete/relocation now performs runtime topology preflight and returns typed
  job tickets.
- Type-registry batch materialization validates contributions before publishing,
  groups touched asset types, sorts each collection once and compiles an
  Arc-backed creation-menu generation. Asset IDs, capability fields and
  toolkit routes have construction-time validation and borrowed accessors.
- These are load/save/authoring operations, not default render-frame work. Their
  bounded tickets and generation checks should be retained while ownership is
  consolidated.

## Retained findings

1. **Dirty/save/import/index still form parallel authorities (P0).** A source
   change can enter the runtime watcher, editor index, dirty registry, import
   flight, direct save path and retained catalog independently. Save and direct
   editor operations can refresh/import synchronously after a worker path has
   already prepared the same document. The index and catalog can therefore
   rebuild broad projections from different generations. One
   `RuntimeAssetSourceDelta -> EditorAssetIndexDelta -> AssetImportGeneration ->
   Save/ArtifactGeneration -> Catalog/PreviewDelta` chain must own the accepted
   source/dependency generation.
2. **Dirty snapshots still clone maps under the registry lock (P0/P1).** A
   changed-document delta identifies affected documents, but snapshot and
   `changes_since` materialize each document's full external-effect map while
   holding the mutex. Reset and retry paths can repeat that work. Publish
   immutable per-document effect slices or cursors and apply one generation
   checked save commit; do not make every consumer clone the registry map.
3. **Import single-flight admission can block and retains split results
   (P0/P1).** Flights use Condvar waits for admission/result and serialize UUID
   transitions under a shared state lock. Completed results are cloned for each
   observer, while estimated bytes do not cover every nested allocation. A
   stalled submitter can delay a later watcher/UI request. Admission must return
   a bounded ticket immediately, share one immutable result lease, and reserve
   result/diagnostic/event capacity with the editor-jobs terminal lease.
4. **Editor index and catalog projection remain full-build paths (P1).**
   `rows()` collects and sorts runtime entries; replacement scans metadata and
   dirty/import sets; unknown watch paths remain a separate set until a later
   reconciliation. Catalog/reference/folder generation then rebuilds all rows
   for a small change. This is currently a dormant or selected path, not a
   measured frame hotspot, but activating it as a second Browser authority would
   make the cost structural. Consume stable runtime slots plus affected UUIDs,
   or merge the index into the existing catalog generation.
5. **Production type materialization bypasses the batch core (P1).** The batch
   API already stages contributions and sorts touched collections once, but
   builtin validation/materialization still rebuilds definitions and applies
   contributions through single-entry wrappers on common capability paths. That
   repeats owner/default-document work and can publish a partial valid subset by
   policy. Compile one complete extension/capability generation, preserve
   input-indexed errors, and publish one atomic registry generation.
6. **Save and import payload proposals are not complete (P1).** Save preflight
   tracks estimated bytes, but dependency/artifact serialization, result status,
   notification delivery and filesystem staging are not one admitted envelope.
   Import keys have URI/digest estimates, while decoded artifact/result peaks and
   observer/message retention are separate. Cap+1 source bytes, dependency rows,
   output bytes or deadlines can therefore be discovered after acceptance.
7. **Refactor outputs and metadata remain broad owned vectors (P1).** Delete and
   relocation return complete status lists, and metadata documents can retain
   multiple projected entries. These are valid explicit results, but count,
   byte, age and export budgets are not shared with the jobs/message leases.
   Large operations need paged or Arc-backed terminal receipts.
8. **Identity and replacement generations need one session authority (P1).**
   Import flight, UUID lifecycle, dirty registry, type registry and job IDs each
   maintain separate counters; several use saturating or unchecked increments.
   Reconnect/reload or exhaustion can alias a stale completion. Bind asset,
   import, save, catalog and job generations to the editor session/runtime
   BuildSet and use checked non-repeating identities.
9. **Main-thread and lock boundaries remain implicit (P1).** Public ticket
   waits and UUID/flight handoffs do not structurally reject editor/main/render
   affinity. Commit/hydration paths can hold dependency or session locks across
   parse/compile/artifact work, while save adapters and observer delivery can
   re-enter job/message state. Filesystem, parser, compiler and foreign toolkit
   work must run in Runtime11 lanes; main/UI work only compares and publishes a
   candidate receipt.

## Architecture handoff

1. Compile one `AssetOperationProposal` from source/dependency identity,
   dirty/toolkit generation, affected UUIDs, source/decoded/artifact/result
   bytes, dependency rows, output pages, deadline and owner affinity. Reject
   cap+1 before filesystem read, parse, decode, job/channel or catalog mutation.
2. Seal one immutable `AssetPayloadGeneration`/`AssetCatalogGeneration` and
   share it across dirty, import, save, index, preview and Browser consumers.
   Stable queries borrow ordered slots; changed work owns only affected rows.
3. Map save, import, delete and relocation onto the shared editor/runtime
   TaskGraph generation. Every operation receives one ticket and one terminal
   receipt; replacement, stale source, panic, cancellation, shutdown and
   downstream backpressure are explicit outcomes.
4. Replace blocking flight waits with nonblocking admission and shared result
   leases. Keep UUID serialization as a keyed resource lane, not a caller-held
   Condvar or an editor-private scheduler.
5. Route all enabled type contributions through one staged batch, cache the
   resulting registry by extension/capability generation and publish menus,
   toolkits and commands from that same generation.
6. Move dirty effect-map copying, reference/folder rebuild and preview/catalog
   invalidation behind bounded cursors and affected-key pages. Save All streams
   a bounded window and commits only when the dirty token still matches.
7. Add exact owner/session/runtime generations and aggregate diagnostics for
   source reads, parse/decode, jobs, bytes, lock waits, result retention,
   invalidation, stale completions and terminal outcomes. Disabled diagnostics
   performs no projection or label work.

## Evidence and acceptance gates

Unreal `AssetRegistryState.cpp` updates only changed records and accelerators,
and groups larger removal batches for parallel work; `AssetDataGatherer.cpp`
uses a background gather loop with interruptible end times and staged local
results. `PackageAutoSaver.cpp` maintains dirty package sets from change events,
while `SavePackageUtilities.cpp` exposes phase timing and outstanding async
write counts. These sources support maintained indexes, staged background work
and explicit save receipts, not Zircon's exact thresholds or ABI.

M0 adds RED tests for duplicate source/import/save authority, source/dependency
cap+1, nonblocking submit, full-map delta bytes, batch materialization, result
lease capacity and generation exhaustion. M1-M3 implement the shared proposal,
catalog/payload generations and Runtime11 task facade. M4-M6 add paged UI
projection, diagnostics deltas and managed F1/F4 scale evidence.

Acceptance covers documents/effects and assets at 1/100/10K/100K, duplicate and
distinct UUID imports, watch storms, Save All windows, type contributions and
commands at 0/1/100/10K/cap+1, toolkit/provider failures, stale generations,
panic/cancel/shutdown, result/message backpressure, stable Browser queries and
diagnostics Disabled/Counters/Full. Report proposal/parse/import/save/commit
latency, filesystem calls, lock hold/wait, clones/allocations/bytes, queue and
worker occupancy, generation transitions, stale/drop/backpressure reasons and
terminal outcomes.

Hard gates: current-source Cargo builds; one accepted source generation feeds
dirty/import/save/index/catalog; cap+1 causes zero large reads/decodes/jobs;
main/UI callers never block or parse/compile; all accepted operations terminalize
exactly once; changed work is delta-proportional; type batches publish atomically;
identities never repeat; stable/Disabled queries clone and allocate zero; and
diagnostics match actual bytes, waits, invalidations and drops. No benchmark
artifact or micro-fix is warranted before these ownership corrections.
