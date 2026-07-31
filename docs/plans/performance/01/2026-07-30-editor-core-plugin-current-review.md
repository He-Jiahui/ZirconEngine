# Editor core plugin current-source review

## Status

- Result: `static_complete / dynamic_pending`.
- Review date: 2026-07-30.
- Owners: Editor12 owns editor catalog generations, lifecycle state and extension materialization; Plugins01 owns native plugin instances and serialized contributions; Editor02 owns the bounded message page; Runtime11 owns an optional bounded affinity ticket only when a plugin contract explicitly permits non-main execution.
- Accounting: keep `zircon_editor/src/core/plugin/**` in `pending.md`; do not add it to `review.md` before current-source managed Cargo, plugin/history scale counters and F0/F4 product traces are GREEN.
- Code disposition: no Rust source was changed. The current 35-file folder is untracked in the shared worktree and was preserved exactly.

## Exact scope

| module | files | physical lines | inline tests | ignored | ordered path-and-raw-content SHA256 |
|---|---:|---:|---:|---:|---|
| `zircon_editor/src/core/plugin` | 35/35 | 6,318 | 51 | 0 | `df4815e4600e626fb23da798a3633759512ac4bb2dd8a09f47af3e2235f730b6` |

All Rust files under the root, `manager/**` and `sdk/**` were read in full. The production chain was followed through `EditorManager` construction/project publication, commandlet/panel snapshot readers, native contribution materialization, retained-host lifecycle tick and editor message transport. Related external tests and UI callers are supporting evidence, not newly accepted folder accounting.

## Current-source improvements

1. Stable reads now clone an `Arc<EditorPluginManagerSnapshot>` or `Arc<EditorPluginCatalogSnapshot>`. Package, registration, capability and panel-row indexes are built with the immutable catalog generation, and panel rows borrow the canonical projection rather than rebuilding it.
2. Lifecycle, enablement, project registration and replacement publication are serialized by one mutation gate, validate a candidate before publishing, preserve old-generation readers and publish one manager snapshot for a successful transaction. Project-native rows are explicitly replaced/cleared and lifecycle callback panics are contained as diagnostics.
3. Loading phases and active extension visibility have a typed state machine. Repeated unchanged loading-phase and project-manifest requests return the existing snapshot, which is the correct no-rebuild fast path.
4. These improvements close the old stable-panel rebuild claim, but they do not make event-time mutation cheap. The expensive work below is reachable from every retained tick that receives a document or play-mode lifecycle message.

## Current bottlenecks

### PERF-MVP-594: lifecycle dispatch and retention

- `EditorPluginLifecycleMessageBridge::pump` performs a whole-inbox drain, extends a second unbounded `VecDeque`, retains its mutex and synchronously invokes every active plugin on the UI tick. It has no entry, byte, deadline or oldest-age bound.
- `EditorPluginManager::dispatch_lifecycle_event_to_active` then holds `lifecycle_mutation` across all foreign Rust plugin callbacks. It clones all manager entries and the complete mutable catalog, clones the event per active plugin and linearly finds each registration. A slow or reentrant callback blocks every enable/disable/reload and causes the bridge to defer later lossless messages.
- Every routine external event is appended to each registration's lifetime `EditorPluginLifecycleReport`. The history has no entry/byte/age retention. The next catalog clone copies the complete history again, so time and memory grow with `plugins x historical events`.
- Even when all callbacks succeed and no plugin state changes, dispatch publishes a new catalog generation and manager generation. This invalidates consumers and triggers all PERF-MVP-538 rebuild work. Existing tests currently require this generation bump, so the performance correction needs an intentional contract hard-cut and new identity tests.

### PERF-MVP-538: structural catalog and extension materialization

- `EditorPluginCatalog::clone_catalog` deeply clones all registrations, including package manifests, extension registries, runtime consumer registries, diagnostics and lifecycle history. `EditorPluginCatalogSnapshot::from_catalog` then clones manifests/capability strings again and rebuilds package/registration indexes, fault sets and the panel projection.
- Every `EditorPluginManagerSnapshot::from_parts` calls `build_active_extensions`, even for an external event that leaves the active set and catalog structure unchanged. The builder clones every active view/drawer/menu/component/template/importer/tool/graph/timeline/command descriptor, reconstructs builtin asset types, reapplies all contributions and sorts diagnostics.
- Initial admission calls owned `package_manifests()` repeatedly across graph validation, discovery indexing and snapshot construction. The dependency graph allocates `String`/`BTreeSet` owners and uses recursive DFS, so a pathological deep manifest chain adds avoidable copies and stack-depth risk at F0/project open.
- Native serialized contribution materialization clones a package registry in `NativeEditorContributionMaterialization::materialize_batch`, then `materialize_serialized_contribution_batch` clones that candidate again. Multiple batches for one package repeatedly copy all prior contributions before the final manager/catalog merge.
- Small successful/failed stage sets use linear `Vec::contains`, but the stage vocabulary is fixed at ten; this is bounded and is not an optimization priority.

## Optimization plan and acceptance

- PERF-MVP-538: split the immutable structural catalog/compiled extension owner from lifecycle runtime state. Descriptor, package, capability, consumer and contribution handles belong to one `Arc` structural generation. Cache active extensions by `{catalog_generation, active_set_generation}` and share them across event/diagnostic generations; a successful non-structural callback must not rebuild catalog indexes, projections, manifests, asset types or extension registries.
- Build project/native mutations in one candidate transaction and publish once. Materialize all accepted batches for a package into one transaction-owned candidate; do not nest registry clones per batch. Admission should borrow manifests and use an iterative indexed graph for deep chains. Preserve last-good publication and old-reader quiescence.
- PERF-MVP-594: snapshot ordered active plugin handles plus generation under a short manager lock, release it, then dispatch outside bridge-pending and manager-mutation locks within the tick count/byte/deadline allowance. Commit failures/cancellation only if the generation still matches; a structural mutation may reject or cancel stale completion with typed diagnostics.
- Keep the current-call report separate from retained state. Store only compact latest-success/failure stage state plus a bounded diagnostic/audit ring with entry, byte and age limits; routine successful `SceneChanged`/play messages must not accumulate in the structural catalog. Preserve lossless source ordering and failure retry semantics.
- Matrix: plugins `0/1/100/1K`, contributions per plugin `0/1/100`, lifecycle history `0/1/1K/1M`, messages `0/1/64/4,096`, callback `0/1/16 ms/10 s`, project/native batches `0/1/100`, dependency depth `1/1K/100K`, threads `1/16`, reload/unload/error/stale completion. Record catalog/manager/extension builds, Arc identities, manifest/descriptor/history clone bytes, callback and mutation lock wait/hold, callback-in-lock wall, queue/audit entries+bytes+oldest age, stack depth, UI p50/p95 and RSS.
- Require unchanged successful external events to perform zero catalog/index/projection/extension builds and retain structural snapshot identity; callback-in-bridge-lock and callback-in-manager-lock must be zero; every tick and audit store must be hard bounded; one accepted structural transaction yields one generation/build; no loss, duplication or reorder; failure-to-Faulted, phase, enable/disable, hot reload, project rollback and old-reader behavior remain equivalent.

## Cross-engine evidence and intentional divergence

- Bevy `dev/bevy/crates/bevy_app/src/plugin.rs` gives plugin setup explicit Adding/Ready/Finished/Cleaned phases; `app.rs` runs finish/cleanup over the existing registry instead of rebuilding registry metadata for each callback. Zircon must additionally support hot reload, so it cannot freeze plugins permanently after startup, but structural generations should still change only for structural mutations.
- Fyrox `dev/Fyrox/editor/src/plugin.rs` temporarily takes one plugin from `EditorPluginsContainer`, invokes it and restores it. This demonstrates callback isolation without cloning the complete plugin catalog. Zircon should not copy Fyrox's unbudgeted serial per-frame callbacks; it needs affinity, deadlines and backlog telemetry.
- Unreal `IPluginManager.h` separates explicit `RefreshPluginsList`, a faster single-plugin `AddToPluginsList` and loading-phase completion events. This supports a change-driven catalog generation and event notification that does not imply full rediscovery/reprojection. Zircon keeps its immutable snapshots and typed rollback rather than adopting Unreal's mutable global registry.

## Static gates executed

- Read all 35 Rust files and the retained-host/project/native production chain at current source.
- `rustfmt --check --edition 2024` passed for all 35 exact files.
- No managed Cargo, 1/100/1K plugin/contribution/history benchmark, callback contention/stall run, F0 startup trace or F4 retained-host WPR ran. RenderDoc is not applicable to this non-rendering slice. The folder remains pending.
