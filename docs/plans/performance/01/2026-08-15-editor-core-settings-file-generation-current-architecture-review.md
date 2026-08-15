# Editor core settings file-generation current-architecture review

## Status

- Result: `static_complete / dynamic_pending`.
- Review date: 2026-08-15.
- MVP priority: P0 for settings-file write amplification, authority-lock hold time and stable retained
  frame projection; P1 for keymap publication and bulk layer replacement.
- Owners: Editor17 owns the settings authority and persistence contract; Runtime11 owns the bounded
  I/O lane; EditorUI08 owns retained hot projection; Editor05 owns viewport settings receipts;
  Editor11 owns versioned text; Editor12 owns plugin setting definitions; Render17 owns measurement.
- Accounting: keep `zircon_editor/src/core/settings/**` in `pending.md`. Do not add it to `review.md`
  before current managed Cargo, file-generation scale gates, F0/F4 WPR and CPU/RSS/power evidence.
- Code disposition: no Rust source changed. Eleven of the 16 present files are foreign modified or
  untracked current work, and the retired monolithic `tests.rs` deletion was preserved. The P0
  persistence fix changes request, receipt and flush ownership and is not a one-line lane-key edit.

## Exact scope

| scope | files | physical lines | tests | ignored | ordinal path-and-raw-content SHA256 |
|---|---:|---:|---:|---:|---|
| `zircon_editor/src/core/settings/**` | 16/16 | 3,937 | 34 | 0 | `a8a4570756f9a33280550c6d14df9ef50780d38ab6576fbc0682c70987a8c71e` |

The fingerprint streams every ordinal-sorted normalized workspace-relative path, NUL, raw file
bytes and NUL into SHA256. All 16 current Rust files were read in full. The production path was
traced through context startup, project open/close, viewport persistence tickets, retained-host
tick, V2 token projection, keymap dispatch, Runtime11's keyed lane and atomic file output.

The 2026-07-30 report's 9 files/1,625 lines/12 tests is obsolete. Authority, snapshot, startup and
folder-backed test owners now exist, and persistence behavior has materially changed.

## Current positive baseline

- `SettingsAuthority` is the production owner and publishes immutable snapshots through `ArcSwap`.
  Startup loads the User layer once; project loads are cached by physical settings path.
- A no-op set returns without revision, event or snapshot publication. Built-in keys are parsed at
  registration, and unchanged snapshot payloads reuse their existing `Arc` identity.
- Change history is bounded by entries, estimated bytes and cursor age, with explicit snapshot
  resynchronization for lagging readers.
- `SettingsPersistenceService` moves filesystem work off the UI caller into a dedicated bounded
  Runtime11 lane with entry/byte limits, typed tickets, cancellation, retry, fence and shutdown.
- Atomic output writes a same-directory temporary file, flushes it, replaces the target and retains
  the failure state. The viewport uses the shared context authority and persistence service.
- Current tests cover scope/schema/no-op behavior, reentrant notification, startup/project cache,
  atomic replacement, single-key coalescing, retry, fence, shutdown and change-log retention.

These repairs close the old claims of three production registries, unbounded history and synchronous
UI filesystem work. They do not close the file-generation and hot-projection architecture below.

## Architecture verdict

The logical mutation unit is a setting key, but the durable unit is one physical settings file.
Current persistence keys work by `(scope, target path, setting key)` while every admitted worker
serializes and atomically replaces the complete scope file. Runtime11 executes one active entry at a
time in this lane, so there is no concurrent stale-overwrite race; however, distinct keys cannot
coalesce and each queued trigger can repeat the same full encode/write/flush.

The request also carries only key/scope/revision. It does not own the value or an immutable file
generation; the worker reads whichever complete authority layer exists when it runs. A ticket
revision is therefore a scheduling trigger, not the identity of the bytes made durable. Per-key
cancel/retry receipts cannot precisely state which physical file generation reached disk.

The correct unit is a versioned physical settings-file projection: one ordered single-flight owner
per canonical file, one latest dirty generation, and receipts tied to that file generation. Mutation
remains key-addressed in memory; serialization and durability do not.

## Structural bottlenecks

### P0: distinct-key bursts repeat whole-file durable writes

`SettingsPersistenceRequest::lane_key` includes the setting key
(`persistence.rs:62-80`). Each admitted closure nevertheless calls
`save_authority_layer(scope, authority)`, which prepares the complete persistent BTreeMap and invokes
the same atomic writer (`persistence.rs:282-327`; `io.rs:242-275`). The Runtime11 lane has one
`active` entry and one pump (`bounded_keyed_io/lane.rs:58-77,493-565`), while coalescing matches a
key. Thus a burst touching `K` different settings can execute up to `K` complete serialization,
temporary-file, write, `sync_all` and rename sequences even if every worker observes the same latest
authority state.

There is no dirty-file debounce or comparison with the last durable bytes. The lane is bounded, so
this is not unbounded memory growth; it is bounded write amplification, latency and storage/power
cost. Changing only `lane_key` to omit the setting key is unsafe because existing tickets, retry and
viewport tracking still describe one key while coalescing would supersede work whose file generation
also makes other keys durable.

### P0: full encode executes while holding the authority mutex

The worker obtains `SettingsAuthorityState` and calls `prepare_registry_layer` before releasing it
(`authority.rs:214-243`). `write_versioned_text` walks and materializes the complete persistent map
into a `String` under that lock (`io.rs:253-268`). Project writes additionally retain the project
source lock across this work so queued writes cannot cross projects.

Although filesystem calls are off the UI thread, UI `set`, `clear`, MRU mutation, project replace and
snapshot reads that need authority state can wait behind `O(N + encoded bytes)` serialization. The
request does not seal a delta or immutable file projection at mutation time, so the worker cannot
encode outside the lock without either re-reading mutable state or cloning the whole map.

### P0: retained frames poll settings and take a global write lock

Every retained-host tick calls `sync_settings_projections`
(`retained_host/app/host_lifecycle/tick.rs:35`). It loads the authority `ArcSwap` snapshot and calls
`install_editor_v2_design_tokens` (`retained_host/app.rs:523-529`). The latter takes a process-global
`RwLock` write guard and clones the token handle merely to perform `Arc::ptr_eq`
(`ui/v2_design_tokens.rs:45-68`). Stable frames avoid theme rebuild, but still pay the atomic load,
global write-lock acquisition and Arc traffic at frame frequency.

The authority exposes one contextual `SettingsChangeSubscriber`, currently needed by locale hot
apply (`authority.rs:101-110,337-347`). It cannot independently notify design tokens, keymap and
other projections. The result is one subscriber plus polling rather than one generation-owned
dispatcher with an affected-slot mask.

### P1: keymap reads retain a mutex and legacy full-clone path

Every keyboard lookup loads the settings snapshot and locks `EditorKeymapProjection`; a changed
override rebuilds the keymap and a matched action is copied into a new String
(`ui/host/editor_manager.rs:69-90`). The legacy `keymap()` accessor calls `snapshot()`, which clones
the complete `EditorKeymap`, including its binding/index containers (`editor_manager.rs:162-165`).

Publish `Arc<EditorKeymap>` once per override payload identity and resolve through that immutable
owner. Callers that need a snapshot should receive the Arc, not a deep copy.

### P1: bulk layer replacement multiplies maps and transient snapshots

`SettingsRegistry::replace_persistent_layer` clones the complete previous BTreeMap, chains old/new
keys into a cloned `BTreeSet`, then installs the new map (`registry.rs:202-259`). Authority then
constructs one transient `SettingsSnapshot` per changed key before publishing only the last one
(`authority.rs:188-206`). For `N` imported/plugin settings this is at least `O(N log N)` key work,
one full old-layer clone and `N` snapshot/Arc-handle churn.

Use a sorted linear old/new diff that takes ownership of the old map and publish one final snapshot
plus affected built-in mask. Each built-in slot is recalculated at most once per replacement.

### P1: large typed payloads have two owned bodies per changed generation

The registry owns `SettingValue`; when a built-in value changes, snapshot construction resolves it
and creates a new Arc from a cloned design-token/keymap/MRU payload (`snapshot.rs:179-218` and its
typed extractors). Large plugin values are also encoded from the registry again during persistence.
The file-generation design should share immutable typed payload handles between layer, snapshot and
sealed persistence delta instead of deep-copying bodies at each boundary.

### P2: remaining local costs

- `SettingsRegistry::set` clones `SettingSchema`, including enum sets, only to release the definition
  borrow before mutation (`registry.rs:111-141`). Validate through the borrowed definition, retain
  only copy-sized metadata, then mutate.
- `decode_current_document` parses a full `serde_json::Value` to inspect `$zircon`, then sends the
  same bytes through the generic versioned reader (`io.rs:392-413`). This remains PERF-MVP-570 and
  belongs to Editor11; settings must not create a private parser.
- Project settings read/decode is synchronous inside serialized project-open work
  (`authority.rs:284-320`). It is not a stable-frame hotspot, but large-file open time must be
  measured before deciding whether to prepare off-thread and generation-check the commit.

## Required architecture hard cut

1. Editor17 publishes `SettingsFileGeneration` keyed by `(scope, canonical physical path)`. It owns
   monotonic file generation, latest dirty generation, changed-key mask, immutable/shared values and
   durable/failed generation. A setting revision remains diagnostic metadata, not the durability key.
2. Mutation seals a typed delta or updates a persistent immutable file projection under a short
   authority lock. Full text encoding occurs outside that lock. Do not move the cost by cloning the
   complete BTreeMap on every key change.
3. Runtime11 runs at most one settings write per physical file and retains at most one latest pending
   generation for that file. Interactive changes use a measured debounce; explicit Apply/Close,
   project switch and shutdown bypass debounce through a fence.
4. A file-generation receipt covers all changed keys included in its projection. Cancellation,
   retry, failure and shutdown report file generation and target identity. A newer successful file
   generation satisfies older included key changes without reporting false failure.
5. The writer skips unchanged durable bytes, preserves dirty state on failure and keeps current
   atomic replacement/durability behavior. It records encode bytes/time, write bytes/time, flush,
   retry and coalescing counters centrally.
6. Authority publication includes an affected built-in/projection mask. One dispatcher fans out
   immutable current-generation handles after releasing authority locks. Retained UI only commits
   changed projections; stable ticks perform no settings lock or Arc traffic.
7. Keymap, design tokens, MRU and plugin payloads use shared immutable owners. Bulk replacement uses
   a sorted linear diff and publishes one final snapshot/mask.

## Unreal primary-source comparison

- Unreal `ConfigCacheIni.cpp:3220-3235` marks a config file dirty only when a key is new or its saved
  value changes. Zircon's in-memory no-op gate matches this principle.
- `ConfigCacheIni.cpp:2871-2910` owns output at the file level, skips non-dirty writes, compares
  generated text with original contents, clears dirty only on equality/success and retains it on
  failure. Zircon currently atomically writes each admitted trigger without the file dirty/original
  comparison.
- `ConfigCacheIni.cpp:4815-4848` routes flush through the centralized config cache and flushes one
  named file/branch or all files. `FConfigCacheIni::SetString` routes mutations through that same
  file cache (`5260-5269`). This is evidence for file-level ownership, not a claim that Unreal uses
  Zircon's asynchronous lane.
- Unreal `ConsoleManager.cpp:504-507,2129-2141,3577-3580` marks sinks dirty on mutation and invokes
  them only while that flag is set. Zircon's retained UI should likewise be change-driven instead of
  taking a global projection write lock on every frame.
- As secondary local evidence, Godot `editor_settings_dialog.cpp:69-71,199-205,1107-1112` restarts a
  one-shot 1.5-second timer on settings change and saves at timeout or explicit close. Zircon should
  measure its own debounce interval, but the source confirms burst coalescing belongs above physical
  file output.

## Acceptance and measurement plan

| case | matrix | required result |
|---|---|---|
| file burst | files 1/2; distinct keys 1/10/1K; changes 1/60/1K Hz; debounce 0/250/1500ms | writes/encodes/flushes scale with committed file generations, not keys; one running plus at most one latest pending/file; final bytes contain every accepted change |
| authority contention | keys 7/1K/100K; values 0/1KiB/1MiB; encode delay 0/1/16ms | full encode under authority/project lock=0; set/clear/MRU lock hold independent of total file bytes; UI filesystem wall=0 |
| durability | disk delay 0/10ms/2s; fail before write/flush/rename; retry/cancel/shutdown/project switch | receipt names exact file generation; newest accepted generation durable exactly once or explicit failure; stale project apply=0; atomic/crash semantics unchanged |
| stable UI | 60/120/240Hz for 10/300s; unrelated/design/keymap changes 0/1/1K | stable settings snapshot loads, projection locks, Arc clones and theme/keymap rebuilds=0; changed slot commits<=1/generation |
| bulk load | entries 1/1K/100K; changed 0/1/10/100%; payload 16B/1MiB | old-map full clone=0; diff near sorted linear; final snapshot publications=1; each built-in recalculated<=1; memory hard bounded |
| product | F0 startup/project open and F4 idle/theme/keymap/snap burst/close | current Cargo plus WPR CPU/thread/wake/lock/file-I/O p50/p95, allocation/RSS/package power and same-machine Unreal comparison GREEN |

Instrument per-file dirty generation, admitted/coalesced/superseded work, encode count/bytes/wall,
authority and project lock wait/hold, write/flush/rename count/wall, durable generation, projection
dispatch/apply, full clones and retained bytes. Algorithmic acceptance is stable `O(0)`, key mutation
near `O(log N)`, bulk sorted diff `O(N)` and full encode `O(N + bytes)` no more than once per committed
file generation. Source comparison alone does not establish time or power parity.

RenderDoc is not applicable to this CPU/configuration slice. WPR/xperf is the product tool once the
managed editor launcher is available; the existing approved-root separator failure currently blocks
that launch.

## Per-file review

| file | current-source performance result |
|---|---|
| `authority.rs` | Single authority, ArcSwap publication and cached project source are positive. Full encode is state-lock-held; subscriber cardinality is one; bulk replacement publishes through N transient snapshots. |
| `change_log.rs` | Entry/estimated-byte/age bounds and resync semantics are sound. Needs central counters in product traces, not a second log. |
| `defaults.rs` | Built-ins register once; no hot algorithmic issue found. Large default payload ownership should converge with shared immutable values. |
| `definition.rs` | Validation is explicit and bounded by schema data. `set` currently clones schema before validation. |
| `io.rs` | Atomic writer is correct baseline. Complete map encode per trigger, no unchanged-byte skip and double current-text parse remain. |
| `keymap_overrides.rs` | Small typed DTO; no independent hotspot. Its payload should be shared into the published keymap. |
| `mod.rs` | Export-only owner; no independent runtime work. |
| `page.rs` | Contribution descriptor only; plugin-scale definitions must be measured through Editor12's compiled generation. |
| `persistence.rs` | Filesystem is off UI and bounded. Key-addressed coalescing/receipts do not match whole-file output; every distinct key can repeat the file transaction. |
| `registry.rs` | No-op mutation and precedence are sound. Set clones schema; persistent replace clones old layer/changed keys and emits N changes. |
| `scope.rs` | Copy-sized precedence enum; no performance issue found. |
| `snapshot.rs` | Built-in slot caching and unchanged Arc reuse are positive. Changed large payloads are deep-cloned; bulk replace repeats snapshot construction. |
| `startup.rs` | User layer loads once and captures diagnostics; no stable-frame work. Startup bytes/decode passes remain a scale gate. |
| `tests/mod.rs` | Shared fixtures are appropriately centralized; no product cost. |
| `tests/persistence.rs` | Strong single-key, retry/fence/shutdown and atomic coverage. Missing distinct-key same-file burst, exact file-generation receipt, lock-hold and unchanged-byte gates. |
| `tests/registry.rs` | Broad semantics/no-op/reentrancy/cache/retention coverage. Missing 100K bulk linearity, one final snapshot and shared-payload byte counters. |

## Static gates executed

- Read 16/16 current Rust files in full and traced all relevant production consumers, Runtime11 lane
  behavior, Unreal Config/CVar sources and Godot editor-settings debounce.
- `rustfmt --edition 2021 --check` passed all 16 current Rust files.
- Scoped `git diff --check` passed; Git emitted only existing LF-to-CRLF checkout warnings.
- `tools.tests.test_editor17_settings_owner_modules_contract` and
  `tools.tests.test_editor12_settings_page_contribution_contract` passed 6/6 tests.
- Managed Cargo, F0/F4 WPR and product file-I/O capture remain blocked by the recorded build-helper
  approved-root separator defect. No output artifact was written to C:.
- Protected plans/indexes were not modified. This static review is not an accepted milestone, so no
  commit or WeCom notification is due.
