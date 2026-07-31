# Editor core editor extension current-source review

## Status

- Result: `static_complete / dynamic_pending`.
- Review date: 2026-07-30.
- Owners: Editor12 owns the immutable extension registry, capability membership and plugin pane data-source contract; EditorUI08 owns retained-host visibility/dirtiness demand and recompute budgets; Runtime11 is only an optional bounded executor for sources that explicitly declare non-main affinity.
- Accounting: keep `zircon_editor/src/core/editor_extension.rs` and `zircon_editor/src/core/editor_extension/**` in `pending.md`; do not add them to `review.md` before current-source managed Cargo, registry/source scale counters and F4 retained-host traces are GREEN.
- Code disposition: no Rust source was changed by this review. The two modified and two untracked files already present in the shared worktree were preserved exactly.

## Exact scope

| module | files | physical lines | inline tests | ignored | ordered path-and-raw-content SHA256 |
|---|---:|---:|---:|---:|---|
| `zircon_editor/src/core/editor_extension.rs` + `zircon_editor/src/core/editor_extension/**` | 4/4 | 1,375 | 5 | 0 | `883874c3f51ee23e4f20f571a2071b6b3fc81e5c6d1a68671c6d79ca5b574cf8` |

The four current Rust files were read in full. Production reachability was followed through extension registration, chrome/reflection construction, component/template/importer queries, retained-host template document synchronization and host-lifecycle pane payload recomputation. Those callers are supporting evidence, not newly accepted folder accounting.

## Current-source improvements

1. Contributions are stored in deterministic `BTreeMap` owners and duplicate ids fail before publication. Template descriptor/data-source replacement builds candidates and publishes the complete pair atomically, so a failed reload does not expose a partial template set.
2. Template document descriptors are already synchronized by `editor_template_generation` plus enabled capabilities. Stable retained ticks do not rebuild all plugin template documents; PERF-MVP-595 therefore targets pane data snapshots, not descriptor parsing.
3. Enabled asset types already have a capability-keyed cache. Registration-time validation is expensive, but it is a structural F0/reload path rather than a per-frame path and remains secondary to stable chrome and pane recomputation.

## Current bottlenecks

### PERF-MVP-538: registry lookup and structural rebuild

- `EditorExtensionRegistry` owns roughly fifteen maps and derives a deep `Clone`. `active_extension_registries` clones every active registry for chrome/reflection construction, after which component drawer descriptors are allocated into temporary vectors and cloned again into a new map.
- Thirteen public contribution getters return newly collected `Vec<&T>` values. Stable component drawer, UI template and asset-importer queries clone the enabled capability vector, and `EditorExtensionRegistration::is_enabled_by` then rebuilds a `BTreeSet<String>` for every registration. The caller already has one stable capability snapshot, so this multiplies string ownership and tree work by the registration count.
- `register_editor_extension_owned` retains the shell mutex while validating prior contributions, cloning the command registry, rebuilding operation sets and reconstructing builtin asset types. `validate_asset_type_contributions` reapplies all earlier registrations for every new registration, producing approximately quadratic registration/reload work as the catalog grows.
- `ui_template_pane_data_sources` clones the complete template-id-to-`Arc` map on every call, and `bind_matching_ui_templates_to_views` owns a temporary template-id set even though matching ids can be borrowed during the structural mutation.

### PERF-MVP-595: unconditional synchronous plugin pane snapshots

- Every retained host recompute calls `collect_host_lifecycle_pane_payloads`, which unconditionally invokes `ui_template_pane_data_snapshots`; unlike the built-in UI asset and animation panes, plugin template sources are not filtered by visible pane kind or dirtiness.
- Under the shell mutex, this path clones the enabled capability list, repeatedly rebuilds capability sets through `is_enabled_by`, and materializes a complete template-id-to-source map. It releases the shell lock before foreign code, which is correct, but then synchronously executes every plugin `source.snapshot()` on the retained/UI thread and rebuilds every values/patches map.
- `EditorUiTemplatePaneDataSource::snapshot` has no generation, `NotModified`, affinity, deadline, cancellation, entry/byte estimate or bounded last-good contract. A hidden, unchanged or stalled plugin can therefore allocate on every recompute or block F4 interaction indefinitely. This is distinct from the existing Template V2 document/action correctness failures and from PERF-MVP-594 lifecycle callbacks.

## Optimization plan and acceptance

- PERF-MVP-538: publish one `Arc` extension generation with direct id/type/extension indexes and a shared capability membership view. Stable getters borrow iterators or query the indexes; chrome/reflection consumes shared registry handles and only clones the selected output descriptor. One registration/reload transaction builds command, asset-type and template candidates once and publishes once.
- PERF-MVP-595: register pane data sources with `{template, owner generation, data generation, affinity, estimated bytes}` and maintain a retained-host visible/dirty demand index. A stable or hidden source returns `NotModified` without invoking plugin code; changed sources publish an immutable `Arc` snapshot. Invoke callbacks outside shell/registry locks under count, byte and deadline budgets, preserve a bounded last-good snapshot, and reject stale completion after reload/unload.
- Non-main execution is opt-in by declared affinity and uses Runtime11's bounded single-flight ticket; do not create a private pool or move an undeclared UI-affine source. Deterministic application remains generation checked on the UI owner.
- Matrix: registrations/sources `0/1/100/1K`, contributions per family `0/1/100`, enabled capabilities `0/1/100`, visible sources `0/1/100%`, dirty sources `0/1/100%`, payload `0/64 KiB/2 MiB`, callback `0/1/16 ms/10 s`, recompute `60/120 Hz`, threads `1/16`, reload/unload/stale completion.
- Record registry/index/projection builds, capability String/BTree allocations, descriptor/source/snapshot clone bytes, callbacks per recompute, hidden/stable callback count, callback-in-lock wall, budget queue entries/bytes/oldest age, UI p50/p95 and RSS. Require stable chrome registry clones/builds to be zero, stable/hidden pane callbacks to be zero, callback-in-shell-lock to be zero, per-recompute work and retained bytes to be hard bounded, and unload/reload to leave no stale source or snapshot.

## Cross-engine evidence and intentional divergence

- Bevy `dev/bevy/crates/bevy_ecs/src/query/filter.rs` exposes tick-based `Changed<T>` filtering so consumers visit changed data rather than recomputing every value. Zircon needs an analogous source generation/dirty contract, while retaining plugin ownership and typed pane snapshots.
- Fyrox `dev/Fyrox/editor/src/plugin.rs` documents `on_sync_to_model` as action- or explicit-sync-driven. This supports demand-based pane synchronization instead of unconditional callbacks on every retained recompute. Zircon additionally needs visibility, byte/deadline budgets and stale-generation rejection.
- Unreal's plugin manager separates explicit list refresh/addition from ordinary loading-phase notification. PERF-MVP-538 follows the same structural-versus-event distinction but keeps Zircon's immutable generation and rollback model.

## Static gates executed

- Read all 4 Rust files and the registration/query/reflection/retained-host production chain at current source.
- `rustfmt --check --edition 2024 --config skip_children=true` passed for 3/4 exact files. The root file differs only by the existing import ordering `btree_map::Entry, BTreeMap` versus rustfmt's `BTreeMap, btree_map::Entry`.
- No managed Cargo, 1/100/1K registry/source counter run, callback stall run or F4 retained-host WPR ran. RenderDoc is not applicable to this non-rendering slice. The module remains pending.
