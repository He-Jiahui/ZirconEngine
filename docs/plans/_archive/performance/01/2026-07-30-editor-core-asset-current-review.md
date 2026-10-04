# Editor core asset current-source review

## Status

- Result: `static_complete / dynamic_pending`.
- Review date: 2026-07-30.
- Primary owners: Editor09 for asset registry, index, dirty projection and import admission; Editor03 for transaction dirty generations; Editor14 for non-blocking job admission; Runtime04 for the authoritative asset registry/import backend; Plugins12 for catalog-generation materialization.
- Accounting: keep `zircon_editor/src/core/asset/**` in `pending.md`. Do not add it to `review.md` before current-source managed Cargo, deterministic allocation/lock/queue counters and F1/F4 product evidence are GREEN.
- Code disposition: no Rust source was changed. Existing tracked modifications in `mod.rs` and `type_registry/registry.rs`, plus the current untracked `dirty/**`, `import_flow/**`, `index.rs`, `index/**` and `type_registry/registry/**`, were reviewed and preserved.

## Exact scope

| module | files | physical lines | inline tests | ordered path-and-raw-content SHA256 |
|---|---:|---:|---:|---|
| `zircon_editor/src/core/asset/**` | 31/31 | 5,889 | 43 | `d2e770e810f4efac6f656db674b3d08007c8813628fa9cc5c0b77bae66b885f7` |

The fingerprint streams each native workspace-relative path, a zero byte, the file's raw bytes and a zero byte in sorted path order into SHA256. All 31 files were read in full. Production reachability was followed through host/workbench asset-type materialization and UI projection. Repository-wide caller scans found no production owner yet for `DirtyRegistry`, `EditorAssetIndex` or `EditorAssetImportFlow`; their scale findings are pre-integration gates, not claims about a current UI hot path.

## Per-file review

| file | current-source performance result |
|---|---|
| `dirty/error.rs` | Typed errors only. Snapshot instability is bounded to eight attempts and remains observable rather than silently returning a false-clean result. |
| `dirty/external_effect_id.rs` | Canonical owned identifier validation occurs at construction; ordered comparison supports the registry maps and binary-search snapshot lookup. |
| `dirty/mod.rs` | Module wiring and narrow exports only. |
| `dirty/registry.rs` | The old all-document snapshot path is gone. A 4,096-entry generation journal makes stable external work zero and changed work near delta, but `snapshot` still clones one whole effect map and `changes_since` clones every changed/reset map while holding the registry mutex, then unzips it into two vectors. Reset also clones the document set and changed set; concurrent generations can repeat the work up to eight times. |
| `dirty/tests.rs` | Fourteen tests cover stable/one-change 10K scale, retry races, cursor/reset/removal, saved-top behavior and unique effect ownership. They do not measure allocation bytes, mutex hold time or F4 save/close latency. |
| `import_flow/error.rs` | Typed admission/index/job failures own diagnostic payloads only on rejected paths. |
| `import_flow/flight.rs` | Shared flight and reason set correctly coalesce observers. Admission and result are blocking Condvar waits; `try_result`/`wait` clone the whole result, including an optional status record with owned strings, for every observer. |
| `import_flow/job.rs` | Backend work runs in the Editor job and panic/cancel cleanup is lease-owned. The URI is formatted for both start and completion progress, amplifying the job event sink's message clones. |
| `import_flow/mod.rs` | Defines generation requests, shared tickets and default 4,096-flight/4 MiB/5-minute budgets. Blocking `wait` is public; no production caller exists yet, so worker/tool-only use must be enforced before integration. |
| `import_flow/state.rs` | Exact `(uuid, uri, digest)` single-flight, per-UUID lifecycle serialization and completed-flight eviction are implemented. Remaining issues are synchronous UUID-transition waiting, O(active UUIDs) collision scans after formatting each mutex-group identity, and linear removal inside same-`Instant` `Vec` buckets; a distinct-UUID storm can therefore approach O(N^2). Estimated bytes omit some retained/shared payload shape and need allocator/RSS validation. |
| `import_flow/submit.rs` | Index/state/backend locks no longer overlap. Existing observers nevertheless synchronously wait for the first submitter to publish admission, while UUID Starting/Clearing waits in `reserve`; a stalled owner can block a future UI/watch caller. Each new job also formats a label and clones the request before submission. |
| `import_flow/tests.rs` | Eleven tests cover success/failure/retry, 10K duplicate coalescing, path migration, progress, panic, shutdown, cancel and the three admission limits. They do not prove non-blocking submission or distinct-UUID allocator complexity. |
| `import_flow/tests/concurrency.rs` | Five race tests cover admission failure propagation, generation TOCTOU, UUID lifecycle, hot completed expiry and dynamic result-byte reclamation. They intentionally exercise blocking handoffs but have no bounded submit-latency assertion. |
| `index.rs` | Rows borrow runtime entries and metadata projections. `rows()` still calls runtime `entries()`, allocating and path-sorting all assets per query; registry replacement scans and retains every metadata/document/dirty/import set. Unknown added watch paths accumulate in an unbounded `HashSet` until a registry replacement resolves or removes them. The type has no production caller yet. |
| `index/tests.rs` | Twelve tests cover authority borrowing, watch deltas, pending-path reconciliation without key clones, atomic metadata projection, compound documents and ordering. No stable-query allocation or unknown-path storm budget is measured. |
| `mod.rs` | Public asset contract exports only. |
| `source_authority.rs` | Locator classification is O(path parse); canonical root handling allocates one temporary sentinel string only for explicit root targets. This is not a frame path. |
| `toolkit_route.rs` | Stable locator plus operation route DTO; accessors borrow and add no repeated work. |
| `type_registry/asset_type_id.rs` | Canonical ID validation is linear in input bytes and occurs at construction. Borrowed `str` lookup avoids owned IDs on reads. |
| `type_registry/builtin.rs` | Static lookup uses `OnceLock`, but every fresh `AssetTypeRegistry::with_builtins` rebuilds 26 definitions through 26 single-contribution commits. Host materialization repeats this per uncached capability generation. |
| `type_registry/context_command.rs` | Capability lists sort/dedup once in the builder; accessors borrow. Payload strings contribute to registry clone/move cost but no query-time work occurs here. |
| `type_registry/contribution.rs` | Serializable owned delta DTO. Host materialization currently clones every contribution before applying it, even though the registry batch API accepts ownership. |
| `type_registry/creation_template.rs` | Capability sort/dedup is construction-time. Default documents can be wide and must be included in contribution byte accounting. |
| `type_registry/definition.rs` | Materialized owned definition with borrowed accessors; no per-query rebuild inside this file. |
| `type_registry/error.rs` | Typed diagnostics only; `missing_fields.join` allocates on incomplete-definition errors, outside the success hot path. |
| `type_registry/mod.rs` | Module wiring and exports only. |
| `type_registry/presentation.rs` | Compact owned presentation data with O(1) validation/access. |
| `type_registry/registry.rs` | Borrowed validate-then-commit removes the old clone-on-augment behavior. A crate-visible batch API exists and single contribution delegates to it; production host callers still choose the single-entry API. |
| `type_registry/registry/batch.rs` | Contributions are grouped by asset type, staged and each touched collection is extended/sorted once, with one registry generation publish. Owner strings are copied into pending claims and final owner maps, and valid entries commit despite sibling errors by design. Production materialization has not adopted this path, so its scale benefit is currently test-only. |
| `type_registry/thumbnail_provider.rs` | Descriptor/palette data only; no image decode, upload or render work occurs here. |
| `type_registry/toolkit.rs` | Capability sort/dedup is construction-time and accessors borrow. |

## Corrected and remaining tasks

### PERF-MVP-554: dirty delta exists; changed payload cloning remains

The former `dirty_snapshots` all-document path is stale. `DirtyRegistry::changes_since` now combines a bounded generation journal with Editor03's `HistoryDirtyCursor`; stable external generations visit no journal entries, and one external or transaction change emits only the affected document. The remaining work is payload-width and contention: each changed document's `BTreeMap` is deep-cloned under the registry lock, reset clones every document/map, and concurrent changes can replay the batch eight times. Publish immutable per-document effect pairs or generation-owned slices so the mutex only captures identities/generations; preserve the existing single source of dirty truth and cursor reset semantics.

### PERF-MVP-555: single-flight and budgets exist; submission must become non-blocking

The old duplicate-job/unbounded-admission finding is stale. Current source shares one flight for `(uuid, uri, source_digest)`, merges reasons and observers, serializes generations per UUID, and enforces entry/estimated-byte/oldest-age limits. Before product integration, replace caller-thread Condvar waits with a pending-admission ticket/state machine plus cancellation/deadline. Replace formatted mutex-group allocation and full active-UUID collision scans with a typed monotonic identity, and replace timestamp `Vec` buckets with an ordered identity index. Share completed results rather than deep-cloning status strings per observer; count actual retained bytes.

### PERF-MVP-556: index projection is still full-build but currently dormant

`EditorAssetIndex::rows` inherits a fresh full collect and path sort from Runtime `AssetRegistryIndex::entries`; replacement scans all local maps/sets, and unresolved watch paths are unbounded. No production caller currently uses this editor index, so the risk is an integration gate rather than measured Browser cost. Editor09 must either merge it into the existing generation-owned catalog or consume stable ordered runtime slots plus affected UUIDs and a bounded pending-path journal. Do not introduce a second production asset projection.

### PERF-MVP-562: batch core exists; production materialization still bypasses it

`AssetTypeRegistry::apply_contributions` already groups by asset type, validates/stages, sorts each touched collection once and publishes one generation. However, `validate_asset_type_contributions` and `materialize_enabled_asset_types` rebuild builtins and clone/apply every contribution through the single-entry wrapper. Route a complete plugin/capability generation through the batch API while preserving per-contribution failure order and atomic definition semantics; cache the resulting `Arc<AssetTypeRegistry>` by the existing capability/extension generation.

## Acceptance plan

- Dirty: documents/effects `1/100/10K`, stable/1%/reset and writers `1/16`. Count journal visits, document/map cloned bytes, registry/history lock wait/hold, retries and UI-thread time. Stable must build/clone zero; changed work must track delta; reset must remain bounded/paged.
- Import: duplicate and distinct UUID requests `1/10K/1M`, path `64B/4KiB`, owner stall `0/10s`, observers `1/10K`. Count submitted/merged jobs, caller blocked time, identity comparisons, order-bucket visits, result clone bytes, retained entries/actual bytes/age and RSS. Same generation imports once; submit cannot wait on another caller; every retained dimension is hard-bounded.
- Index: assets/metadata/unresolved paths `1/1K/100K`, stable queries `1/100K`, 1% registry change. Count row allocations, sorts/comparisons, projection validations and pending-path bytes/age. Production stable rows must be generation-borrowed/paged with zero full sort, and unknown watch input must be bounded.
- Type registry: types/plugins/entries `1/100/10K/100K`, reverse/random and mixed valid/invalid batches. Count builtin rebuilds, contribution/owner/default-document clone bytes, collection sorts/entries, generation publishes and cache invalidations. One catalog generation publishes once and sorts each touched collection once with unchanged diagnostics/order.
- Run current-source managed asset/type-registry/import/dirty tests and F1 import/reimport plus F4 Browser/Activity/save/close/plugin-reload traces. RenderDoc is not applicable to this CPU/control slice; preview texture upload and Browser rendering remain under their render-owner plans.

## Reference check

- Godot `dev/godot/editor/file_system/editor_file_system.cpp` runs filesystem scans on a low-priority thread, coalesces a pending change scan and applies targeted `update_files` results. Zircon should use its shared Editor job system and generation tickets rather than a private thread, while preserving the targeted-delta shape.
- Bevy `dev/bevy/crates/bevy_asset/src/server/mod.rs` returns an existing handle instead of spawning duplicate work and explicitly drops `AssetInfos` before work that may block on it. Zircon's current single-flight matches the first property; non-blocking admission and short state ownership remain required.
- Unreal `dev/UnrealEngine/Engine/Source/Runtime/AssetRegistry/Private/AssetRegistryState.cpp` accepts compiled filters and enumerates indexed asset data. Zircon's production Browser should consume the existing generation-owned catalog/indices, not activate a second full-sort `EditorAssetIndex` path.

## Static gates executed

- Read all current 31/31 Rust files and the listed production caller chains.
- `rustfmt --edition 2021 --check` passed for all 31 files.
- `git diff --check -- zircon_editor/src/core/asset` passed; Git only reported the existing LF-to-CRLF checkout warning for two tracked files.
- Source inventory was 31 files, 5,889 physical lines and 43 inline tests at fingerprint `d2e770e810f4efac6f656db674b3d08007c8813628fa9cc5c0b77bae66b885f7`.
- `review.md` remained unchanged. No managed Cargo, allocator/RSS/lock scale run, WPR F1/F4 product trace, RenderDoc capture or independent dynamic review ran.
