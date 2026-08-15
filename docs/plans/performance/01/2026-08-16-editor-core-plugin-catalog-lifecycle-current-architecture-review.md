# Editor core plugin catalog and lifecycle architecture revalidation

## Status

- Result: `static_complete / dynamic_pending`.
- Review date: 2026-08-16.
- MVP priority: P0. Builtin plugin startup, project open/close, basic editor extension visibility,
  document lifecycle and play-mode transitions are editor MVP paths.
- Owners: Editor12 owns editor plugin definition/runtime-state separation; Editor06 owns compiled
  extension generations; Editor02 owns bounded lifecycle delivery; Plugins01 owns native discovery,
  load and serialized contribution input; Runtime11 owns only explicitly non-main callback work.
- Accounting: retain `zircon_editor/src/core/plugin/**` in `pending.md`. Do not add it to `review.md`
  before managed tests, scale counters, F0/F4 WPR evidence and the acceptance matrix below pass.
- Code disposition: no Rust source changed. Nine production files and two internal test files in the
  exact scope are foreign modified; related host/project/tick callers are also foreign modified and
  were preserved.

## Exact scope

| scope | files | physical lines | tests | ordered path-and-raw-content SHA256 |
|---|---:|---:|---:|---|
| `zircon_editor/src/core/plugin/**` | 35/35 | 6,318 | 51 | `635e8144b0f6988458026dfbffae0a6ee8a725fb33c9aa7fcf8fc37dada9c44e` |

The fingerprint is SHA256 over sorted normalized path, NUL, raw file bytes, NUL. Every Rust file was
read in full. Production calls were followed through `EditorManager` construction, retained-host
tick, project open/close, native contribution materialization, panel/export readers and the editor
message bus. This supersedes the 2026-07-30 source fingerprint `df4815...`; its architectural
diagnosis remains present in current source.

## Per-file acceptance record

| file | current-source verdict |
|---|---|
| `admission.rs` | Owned manifest clones, string-keyed trees and recursive DFS remain on startup/project publication. |
| `capability_report.rs` | Small result wrapper; no local hot loop. |
| `catalog.rs` | Runtime manifest indexing improved, but lifecycle registration lookup is still linear and catalog cloning is deep. |
| `catalog_gen.rs` | Build-generated builtin descriptor construction is startup-only; it allocates each owned descriptor once. |
| `catalog_snapshot.rs` | Stable reads are shared, but publication duplicates manifests, capabilities, indexes and panel projection beside the full catalog. |
| `catalog_store.rs` | `Arc` snapshot publication and last-good ownership are appropriate. |
| `descriptor.rs` | Clear SDK contract; package attachment copies capabilities and event-consumer manifests at construction time. |
| `extension_catalog_report.rs` | Small generation envelope; its owned registries are rebuilt too often by the manager. |
| `extension_materialization.rs` | Re-clones every active contribution family and rebuilds builtin asset types for each manager snapshot. |
| `isolation.rs` | Panic containment is correct; it does not provide scheduling, timeout or callback quiescence. |
| `lifecycle_message_bridge.rs` | Whole-inbox drain, second unbounded queue and callback execution while holding the pending mutex remain. |
| `manager.rs` | Structural definition, runtime state, callback history and extension publication remain one mutation/copy boundary. |
| `manager/discovery.rs` | Typed source/phase metadata is appropriate; error surface is not a hot loop. |
| `manager/lifecycle_replacement.rs` | Correct cleanup ordering, but serial callbacks and string sets run inside the global mutation transaction. |
| `manager/project_registration.rs` | Project replacement deep-clones the complete current catalog before validation/publication. |
| `manager/project_selection.rs` | Unchanged manifests reuse the snapshot; changed selection scans all entries and may clone the full catalog. |
| `manager/publication.rs` | Atomic last-good publication is correct; replacement clones both previous and candidate state and runs callbacks inside the gate. |
| `manager/snapshot.rs` | Every new manager generation unconditionally rebuilds all active extensions, even when structure and active set are unchanged. |
| `manager/state.rs` | Small typed state machine; no independent performance fault. |
| `manager/tests.rs` | Covers state/phase correctness but has no clone, build, lock, queue or callback-budget assertions. |
| `manager/tests/lifecycle_broadcast.rs` | Explicitly requires routine external events to increment both manager and catalog generations; this contract must hard-cut. |
| `manager/tests/lifecycle_replacement.rs` | Strong cleanup/rollback coverage; no latency, quiescence or scale boundary. |
| `manager/tests/lifecycle_state.rs` | Small transition contract only. |
| `manager/tests/project_registration.rs` | Covers host-owned lifecycle, but repeated identical project reports are allowed to republish/reinitialize. |
| `manager/tests/project_selection.rs` | Validates atomic manifest selection and stable exact repeat; no large-catalog work counters. |
| `manager/tests/snapshot_publication.rs` | Proves stable read identity, but normal state generations still rebuild active registries. |
| `materializer.rs` | Atomicity is implemented by cloning the entire destination registry per batch. |
| `mod.rs` | Export shell only; it currently exposes structural snapshots and lifecycle reports from one broad subsystem. |
| `panel_source.rs` | Correctly borrows one immutable generation and indexed projection; retain this read behavior. |
| `phases.rs` | Small loading-phase vocabulary; no local hot loop. |
| `projection.rs` | Rebuild clones package/display/category/crate/capability strings for every structural snapshot. |
| `registration.rs` | Lifetime lifecycle report is unbounded and cloned on every append; fixed-stage success/failure checks are bounded and low priority. |
| `sdk/examples.rs` | Example-only registration work; no frame path. |
| `sdk/lifecycle.rs` | Current-call and retained lifetime history use the same unbounded owned report type. |
| `sdk/mod.rs` | Re-export shell only. |

## Structural verdict

The current subsystem has a valid immutable-reader mechanism but the wrong publication unit. It
stores four kinds of state in one deep-cloned catalog generation:

1. immutable plugin definitions, manifests, capabilities and contribution descriptors;
2. active/disabled/faulted runtime lifecycle state;
3. transient document/play/asset/UI facts delivered to callbacks;
4. lifetime callback records and diagnostics.

Consequently a transient fact is treated as a structural catalog mutation. This is the dominant
design fault; replacing individual `Vec` or `BTreeMap` operations cannot close it.

### P0: routine lifecycle facts rebuild the complete plugin product

The retained host calls the bridge every tick (`ui/retained_host/app/host_lifecycle/tick.rs:16`). A
non-empty pump drains the subscriber completely, extends a second `VecDeque`, and holds its mutex
while dispatching every callback (`lifecycle_message_bridge.rs:41-55`). There is no entry, byte,
deadline, callback-time or oldest-age allowance.

`dispatch_lifecycle_event_to_active` then holds `lifecycle_mutation` across all foreign callbacks,
clones every entry and the complete catalog, clones the event per active plugin, performs a linear
registration lookup, records the event and publishes (`manager.rs:376-414`; `catalog.rs:132-149`).
Publication constructs a new manager snapshot, and `from_parts` unconditionally invokes
`build_active_extensions` (`manager.rs:423-446`; `manager/snapshot.rs:99-108`). The builder clones
every contribution family and rebuilds builtin asset types (`extension_materialization.rs:9-138`).

Even a successful callback with no state or contribution change therefore performs work proportional
to packages, retained history and active contributions. The current regression deliberately requires
both generations to advance (`manager/tests/lifecycle_broadcast.rs:95-96`), proving this is a contract
problem rather than an accidental missed fast path.

### P0: callback latency and history growth are placed on the editor owner

The bridge pending mutex and manager mutation gate both span callback execution. One slow or reentrant
plugin stalls later lifecycle messages and every enable, disable, reload and project mutation. On a
manager mutation conflict the lossless delivery is pushed back into the same unbounded queue, so
backpressure is neither bounded nor attributed.

Every callback result is appended to the registration's lifetime report
(`registration.rs:103-120`). The report has no count, byte or age retention. The next deep catalog
clone copies that complete history, so repeated facts create increasing allocation/copy cost even
when plugin behavior is stable.

### P0: project open performs nested registry transactions and two publications

Project open synchronously discovers/loads native editor plugins, publishes project registration
reports, then applies the project manifest (`ui/host/editor_manager_project.rs:192-219`). Native
materialization clones the package registry outside the core materializer, while the core
materializer clones it again (`native_contribution.rs:66-75`; `materializer.rs:71-79`). For multiple
batches of one package, all prior contributions are repeatedly copied, producing quadratic
accumulation in batch count/contribution count before the manager performs another complete merge.

The registration publish and manifest apply are separate manager transactions. A successful project
open can therefore build and publish intermediate state twice instead of compiling one validated
project plugin plan and publishing once.

### P1: discovery/admission and snapshot structure duplicate cold-path work

Admission first obtains owned package manifests, creates string-owned dependency maps/sets, then uses
recursive DFS (`admission.rs:47-117`). Deep dependency chains retain avoidable clone/tree cost and
stack-depth risk. Snapshot construction stores the full catalog plus cloned manifests, capabilities,
indexes and a cloned panel projection (`catalog_snapshot.rs:28-78`). These costs are not per-frame,
but they directly affect F0 and project-open MVP latency and amplify every erroneous event-time
catalog publication.

## Required unified architecture

1. `EditorPluginDefinitionGeneration` owns immutable package descriptors, dependency graph, plugin
   handles and contribution handles. It changes only on discovery, project package change or reload.
   Stable readers retain one `Arc` generation and indexed ids.
2. `EditorPluginRuntimeGeneration` owns compact active/disabled/faulted state and an active-set
   generation. Successful transient callbacks do not replace the definition generation. Fault
   commits are generation-checked after callbacks finish.
3. `CompiledEditorExtensionGeneration` is keyed by `{definition_generation, active_set_generation}`.
   It builds each contribution/asset/command index once and is shared across unrelated lifecycle,
   audit and diagnostic changes.
4. The bridge requests one count+byte+deadline-bounded page from the existing message authority.
   Under a short manager lock it snapshots ordered active handles, generation and declared affinity;
   it then releases bridge and manager locks before invoking callbacks.
5. Main-affinity callbacks run on the editor owner within the remaining budget. Only callbacks whose
   contract explicitly permits non-main execution use Runtime11's bounded scheduler, with
   cancellation, single-flight admission and stale-generation rejection. No private plugin pool or
   unbounded handoff is added.
6. Current-call results are ephemeral. Retained state stores compact latest stage success/failure and
   one bounded diagnostic/audit ring with count, owned-byte and age limits. Routine successful scene,
   play and UI facts are never appended to the structural catalog.
7. Project/native discovery produces one `CompiledProjectPluginPlan`: borrow/index manifests, use an
   iterative dense dependency graph, materialize all accepted batches into one transaction-owned
   candidate, validate once, activate once and publish once. Preserve last-good readers and explicit
   rollback/quiescence.

## Unreal primary-source evidence

- `IPluginManager` explicitly separates full `RefreshPluginsList` from a faster single-plugin
  `AddToPluginsList`, and loads only modules for one declared loading phase
  (`IPluginManager.h:289-323`). This supports change-driven structural publication rather than
  republishing definitions for ordinary editor facts.
- `FPluginManager::ConfigureEnabledPlugins` performs configuration only while
  `PluginsToConfigure` is non-empty and clears that set after processing
  (`PluginManager.cpp:2034-2085`). Loading phases are explicit and separately profiled
  (`PluginManager.cpp:2884-2988`).
- Unreal discovery prunes traversal below a directory once its plugin descriptor is found and uses
  `ParallelForWithTaskContext` for directory batches (`PluginManager.cpp:1120-1216`). Zircon should
  adopt the indexed/iterative/parallelizable work shape, not Unreal's raw pointer ownership.
- `FModuleManager::LoadModuleWithFailureReason` first checks the existing module, retains a module
  path cache and profiles load/startup as an explicit lifecycle operation
  (`ModuleManager.cpp:980-1039,1819-1912`). Module-change events fire after successful startup or
  shutdown (`ModuleManager.cpp:1096-1112,1316-1399`); they are not a reason to clone all plugin
  metadata for every document message.
- Unreal's message router has its own wait-driven loop, resolves recipients first, invokes
  `AnyThread` receivers directly and schedules named-thread receivers through TaskGraph
  (`MessageRouter.cpp:53-65,118-182`). This supports declared affinity and separating routing from
  UI ownership. Its unbounded command drain is not a Zircon budget precedent; Zircon still needs
  explicit count+byte+deadline admission.

The reference establishes ownership and algorithm shape. It does not supply portable latency, power
or plugin-count budgets, so acceptance remains source-bound and same-machine.

## Dependency-ordered implementation plan

### M0: instrument the current publication unit

- Count bus drain, bridge pending entries/bytes/oldest age, callbacks attempted/completed/deferred,
  callback wall, bridge/manager lock wait+hold and callback-under-lock count.
- Count definition/catalog/manager/projection/extension/asset builds and clone bytes by manifests,
  contributions and lifecycle history.
- Record native discovery/stat/read/load, batches, candidate registry clones, contributions copied,
  admission nodes/edges/stack depth and project publications.

### M1: hard-cut transient lifecycle state from structural definitions

- Introduce the immutable definition generation and compact runtime generation.
- Snapshot active handles under a short lock, execute callbacks outside both locks, then commit only
  fault/cancel changes against the observed generation.
- Replace lifetime successful-event history with bounded diagnostic/audit retention.

### M2: compile extension and project plans once

- Key the extension generation only by definition and active-set identity; unrelated lifecycle events
  must preserve `Arc` identity and perform zero registry/asset/projection builds.
- Materialize all native batches per package in one candidate and publish registration+manifest
  selection as one validated project transaction.
- Replace recursive string-owned dependency traversal with an iterative indexed graph.

### M3: recovery, rollback and product acceptance

- Preserve phase ordering, panic isolation, failure-to-Faulted behavior, disable/unload retries,
  hot-reload quiescence, last-good publication and old-reader lifetime.
- Run managed focused/upward tests, release/profiling scale workloads, F0/F4 WPR/xperf and same-source
  pre/post measurements. RenderDoc is not applicable to this CPU/plugin control-plane slice.

## Quantified acceptance

| matrix | required measurements | acceptance |
|---|---|---|
| plugins `0/1/100/1k`; contributions/plugin `0/1/100/10k` | definition/manager/projection/extension/asset builds, rows/edges, clone bytes, RSS, p50/p95 | one accepted structural transaction builds/publishes once; stable read and successful transient event perform zero structural/registry builds |
| messages `0/1/64/4,096`; callbacks `0/1/16 ms/10 s`; payload `64 B/2 MiB` | drained/deferred, callbacks/tick, queue entries/bytes/age, deadline, lock wait/hold, editor tick p50/p95 | count+byte+deadline bounds always hold; callback-under-bridge-lock=0; callback-under-manager-lock=0; one stalled plugin cannot block unrelated mutation indefinitely |
| history `0/1k/1M`; success/failure/retry | retained entries/bytes/age, history clone bytes, structural identity | successful routine facts retain no structural history and clone zero history bytes; diagnostic retention remains within all configured limits |
| native batches `1/100`; dependency depth `1/1k/100k`; manifests `1/1k/10k` | discovery/read/load, candidate clones, copied contributions, graph visits/stack, publications | one candidate owner per package, one project publication, iterative graph with `O(V+E)` visits and no recursion overflow |
| toggle/reload/close/error/stale completion; threads `1/16` | callback order, quiescence, generation commits/rejections, rollback hashes | no loss/duplication/reorder; stale completions cannot fault a replacement; last-good and old readers remain valid |
| F0/F4 current/candidate | WPR CPU stacks, ready/running time, contention, allocations/RSS, package power, project-open and tick p50/p95/p99 | same-machine deltas reported with identical source-bound windows; MVP project/plugin/document/play behavior passes; no invented Unreal target |

## Static gates and blockers

- Exact recount reproduced 35/35 files, 6,318 lines, 51 tests and the fingerprint above after the
  read. Eleven scoped files are foreign modified; related host/project/tick callers are also foreign
  modified.
- `git diff --check` is green for the reviewed source/caller scope, apart from informational CRLF
  conversion warnings.
- `rustfmt --edition 2024 --check` is red only on current foreign formatting (import ordering and
  assertion wrapping in `admission.rs`, `isolation.rs`, `lifecycle_message_bridge.rs`, manager
  modules/tests and `mod.rs`). No formatting write was performed.
- Managed Cargo remains blocked by the approved-drive separator defect in
  `tools/build-editor.ps1:130`; see
  `failure-2026-08-15-build-editor-approved-root-separator.md`. No current-source plugin scale run,
  WPR/xperf trace, latency/power result or performance-improvement claim exists.
- RenderDoc cannot validate plugin discovery, callbacks, locks, cloning or CPU publication and was
  not run for this slice. It remains reserved for later rendering acceptance where GPU work changes.
- No simple source edit was applied. A local queue cap or one removed clone would preserve the
  invalid structural publication contract, and the relevant implementation is foreign dirty.

