# Editor core commands routing and search current-architecture review

## Status

- Result: `static_complete / dynamic_pending`.
- Review date: 2026-08-16.
- MVP priority: P0 for event-to-command routing, menu projection under global locks and interactive
  palette query latency; P1 for extension registration/finalization and headless indexes.
- Owners: Editor08 owns the compiled command generation and typed invocation route; EditorUI08 owns
  change-proportional menu/palette materialization; Editor12 owns atomic contribution transactions.
- Accounting: keep `zircon_editor/src/core/commands/**` in `pending.md`. Do not add it to `review.md`
  before current-source managed Cargo, scale/allocation/lock counters and F4 product evidence pass.
- Code disposition: no Rust source changed. Seven files contain pre-existing changes; all source bytes
  and owners were preserved. The session write scope is `docs/plans/performance` only.

## Exact scope

| scope | files | physical lines | tests | ordered path-and-raw-content SHA256 |
|---|---:|---:|---:|---|
| `zircon_editor/src/core/commands/**` | 17/17 | 4,285 | 30 | `3807492cc6aa0f5e3d337dfcf0b941ee689b32b9633876dc372224bea6acb06e` |

The fingerprint is SHA256 over normalized sorted path, NUL, raw file bytes, NUL. Every Rust file in
the folder was read in full. Production event dispatch, operation dispatch, command-palette actions,
command-evaluation projection, retained recompute/reflection, workbench menu construction and editor
extension registration were traced. The July snapshot (3,888 lines, 21 tests) is obsolete.

## Per-file acceptance record

| file | lines | verdict |
|---|---:|---|
| `asset_write_target.rs` | 28 | Two construction-time owned argument names; no independent hot loop. |
| `contribution.rs` | 111 | Deterministic pending maps and failure checks. Repeated path clones are registration-transaction cost, not frame work. |
| `defaults.rs` | 583 | Built-in descriptors/chords construct at registry bootstrap. The growing defaults surface strengthens the need for one finalize step; it is not a runtime parsing hotspot. |
| `descriptor.rs` | 377 | Direct `is_enabled` avoids materializing `effective_when`; required capability scans remain proportional to a descriptor's requirements. |
| `document_kind.rs` | 96 | Streaming validation only; no temporary segment collection. |
| `eval_snapshot_handle.rs` | 124 | Current readers can share one `Arc<CommandEvalCtx>` with a generation; the July per-palette-query deep-clone finding is fixed. Legacy `snapshot` APIs still deep-clone capabilities for event/operation callers. |
| `key_chord.rs` | 453 | Keyboard input hashes and compares borrowed normalized key forms with no owned hot-path chord. Construction/display allocations remain outside normal event lookup. |
| `keymap.rs` | 214 | Signature-to-candidate index provides near O(1)+collision lookup. Override rebuild still clones complete base/override maps once per settings generation. |
| `keymap/tests.rs` | 287 | Eight tests cover layers, conflicts, 10K bindings and a 1M-event storm. Two filesystem tests use `std::env::temp_dir()` and therefore are not approved for the user's no-C-artifact requirement. |
| `menu_model.rs` | 57 | DTO ownership is reasonable at publication, but complete labels/ids/shortcuts are rebuilt by `menu.rs`. |
| `menu.rs` | 82 | Seven top-level calls each scan the complete registry, split paths, format shortcuts and own row metadata while production callers hold the registry mutex. |
| `mod.rs` | 38 | Narrow module/export boundary; no work. |
| `palette.rs` | 635 | Current generation owns byte postings, normalized documents, enablement slots, a single-pass matcher and bounded top-K heap. Common-byte and empty queries still visit O(N), MRU membership/rank adds bounded 32N comparisons, and exact total count prevents early termination. |
| `play_mode_predicate.rs` | 21 | Constant-time typed matching. |
| `registry_handle.rs` | 27 | One global mutex serializes structural mutation and broad menu/reflection readers. |
| `registry.rs` | 893 | Palette catalog is generation-shared, but event reverse lookup, headless route/name lookup and uniqueness checks scan descriptors. Sequential batch registration advances/invalidate per row. |
| `when.rs` | 259 | Predicate evaluation is borrowed. Context construction still owns a `BTreeSet<String>` and full equality compares revisions/capabilities during broad shell projection. |

## Corrected current-source baseline

The current implementation has four material improvements which plans must preserve:

1. keyboard dispatch creates no owned normalized chord and probes a collision-safe signature index
   (`key_chord.rs:88-198`; `keymap.rs:57-88`);
2. palette consumers clone a stable catalog `Arc` under the registry mutex, release the mutex, then
   query it. The query no longer executes under the registry lock
   (`ui/retained_host/app/command_palette_actions.rs:22-36,62-76,124-139`);
3. the command evaluation handle exposes a shared immutable context and palette consumers use it, so
   capability strings are not deep-cloned per edit/window request
   (`eval_snapshot_handle.rs:31-47`; `command_palette_actions.rs:22,62,124`);
4. non-empty palette queries select the rarest query-byte posting, run contiguous and subsequence
   scoring in one document pass, and retain only `offset+limit` candidates in a heap
   (`palette.rs:145-226,410-541`). The old full-catalog substring-then-subsequence diagnosis is no
   longer true.

These fixes make a second keymap index, a second command context, or an unbounded palette worker queue
incorrect directions.

## Structural bottlenecks

### P0: arbitrary editor events reverse-scan the command registry

Before executing every normalized event without explicit operation metadata,
`dispatch_normalized_event_with_metadata` locks the command registry and calls
`descriptor_for_event`; that method linearly compares the event against every descriptor
(`ui/host/editor_event_dispatch.rs:52-69`; `registry.rs:184-187`). Most transient, pointer, focus,
viewport and replay events are not command descriptors, so their common result is a complete failed
scan. Inspector fallback then parses a constant operation path before doing a direct lookup
(`editor_event_dispatch.rs:438-448`).

This is an ownership error, not a request for hashing arbitrary `EditorEvent` payloads. A normalized
command invocation already knows its command identity and must carry it through execution. Direct
non-command events must be marked as such and skip registry discovery. Only the finite static event
forms which intentionally map to commands need a generation-built reverse route.

Create `PERF-MVP-645` P0: hard-cut dispatch to a typed `CommandRoute::{Command(id), Operation(id),
DirectEvent}` produced at binding/menu/keymap/remote normalization. Build the small static reverse map
once in the compiled registry for compatibility/replay; dynamic payload events never become map keys.
Measure registry visits and command-lock hold for hover/press/focus/viewport/inspector/replay storms.

### P0: menu construction is O(7N) under global locks

`menu_bar_model` invokes `menu_model` for seven labels; each call scans all descriptors and allocates
each accepted row's label, path, shortcut and id (`menu.rs:8-38,40-81`). Retained shell recompute and
viewport fallback hold the command mutex across complete `WorkbenchViewModel` construction
(`ui/retained_host/app/host_lifecycle/recompute/shell/builder.rs:69-77`;
`recompute_viewport.rs:51-55`). Full reflection holds the shell mutex and command mutex together while
building chrome, capabilities, contribution projection, menus, reflection routes and the final
snapshot (`ui/host/editor_event_runtime_reflection.rs:29-89`).

This remains owned by `PERF-MVP-076`/`PERF-MVP-099`. Compile top-level menu buckets and shared static
row metadata once per command generation. A compact command-evaluation generation patches only the
affected enabled/check/visibility fields. Clone the compiled generation under the command lock and
release it before any shell/view/reflection build or publish.

### P0: palette indexing is improved but not yet change-proportional

The rarest-byte posting is a safe necessary-condition filter, but common ASCII query bytes still
produce a posting containing almost every command. Exact match-count semantics then require scoring
the entire posting. Empty query visits every entry, evaluates enablement and calls bounded MRU
membership for each non-MRU row; non-empty matches perform bounded MRU rank lookup. These are O(N)
and O(32N), albeit with bounded retained rows (`palette.rs:175-226,228-272`).

`WorkbenchCommandPaletteOpenState` also materializes up to 12 row maps, a second id array and a selected
id clone (`command_palette_actions.rs:204-230`). This is bounded and secondary, but the ABI should
eventually carry one typed row window rather than parallel representations.

Correct `PERF-MVP-211`: remove the fixed findings about per-query context clone, registry-lock-held
query and two-pass document scan. Retain the current posting/single-pass/top-K baseline. Measure common
and rare queries, then add an incremental prefix/token/trigram frontier only where the measured
crossover justifies it. Preserve exact total count, deterministic MRU/score/order and deep paging.

### P1: registration is a transaction implemented as repeated mutation

Editor extension registration holds the shell mutex, clones the complete command registry, registers
each command/view sequentially, repeatedly scans existing descriptors for duplicate headless route
and name, attaches targets one by one, rebuilds an available-operation set, then commits the complete
registry under the command mutex (`ui/host/editor_extension_registration.rs:119-310`;
`registry.rs:50-142,258-299`). Generation advances and palette invalidation occur per inserted row even
though only the final candidate is published. The same shell critical section also spans plugin
overlay preparation and broad validation.

`PERF-MVP-079`/`PERF-MVP-538` must use one generation-checked candidate transaction: snapshot shared
base generation, validate/normalize all contributed descriptors outside shell/command locks, build
command-id, operation-factory, headless route/name, static event route, chord, menu and palette indexes
once, then atomically publish if the base generation still matches. Failure publishes nothing and
reload/unload waits for old-reader quiescence.

## Required unified architecture

1. Editor08 owns one immutable `CompiledCommandGeneration`, not separate mutable caches. It contains
   shared descriptors plus command-id, operation-factory, headless route/name, key chord, static event
   route, menu bucket and palette discovery indexes.
2. All invocation surfaces normalize to a typed `CommandRoute` before execution. Execution performs
   direct command/operation lookup; direct events bypass the command registry. Replay records preserve
   route identity when present and explicitly use compatibility mapping only for legacy records.
3. `CommandEvalSnapshotHandle` remains the sole interactive context owner and publishes immutable
   generation handles plus changed dependency bits. Menu and palette enablement slots declare their
   dependency mask so a context change reevaluates affected slots only.
4. Retained shell/reflection clones `(compiled command generation, eval generation)` under short
   locks, releases both locks, and builds/applies at most once per changed domain generation. Stable
   frames build zero menus and evaluate zero predicates.
5. Palette retains current bounded windows and single-pass scorer. Query generations reuse a measured
   incremental candidate frontier for append/backspace and common-query cases; cancellation/stale
   rejection prevents an older query from replacing the visible window. No private unbounded worker.
6. Editor12 stages one contribution batch against one base generation, builds every command index once
   outside foreign locks, and commits once. The old compiled generation remains valid for readers until
   quiescent.

## Reference-engine evidence

- Unreal constructs command metadata once and inserts it into a per-context command map; active
  chords are retained in chord-to-command maps
  (`dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Commands/InputBindingManager.cpp:454-525`).
  Active-chord lookup uses those maps and then direct command-info lookup
  (`InputBindingManager.cpp:681-710`). This supports Zircon's current chord index and one compiled
  generation, not event-time global scans.
- Unreal maps `FUICommandInfo` directly to `FUIAction` and executes/checks by command identity
  (`dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Commands/UICommandList.cpp:54-65,108-145,254-275`).
  It does not recover a command by comparing an arbitrary executed UI event to every registered
  command. `CommandRoute` transfers this identity principle while keeping Zircon's typed events.
- Unreal menu construction receives a declared command and adds one menu block referencing that
  command/list (`dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/MultiBox/MultiBoxBuilder.cpp:228-248`).
  It does not scan the global command registry once per top-level menu. Zircon's generation-built menu
  buckets provide the equivalent ownership without copying Slate widgets.
- Unreal command info retains label, description, icon and default chords before registration
  (`dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Commands/UICommandInfo.cpp:126-142`).
  Zircon should similarly share static row metadata and only patch dynamic enablement.
- Godot's command palette scans every command and creates candidate entries on each text change
  (`dev/godot/editor/settings/editor_command_palette.cpp:70-94`). Zircon's postings, shared catalog and
  bounded row window are already a stronger baseline; Godot is evidence not to regress to full
  candidate materialization, not the target scale algorithm.

## Quantified acceptance

| matrix | required measurements | acceptance |
|---|---|---|
| event routes: hover/press/focus/viewport/inspector/replay; 1/1k/1M events; commands 1/100/10k | route kind, registry visits, event comparisons, path parses, command lock wait/hold, owned ids, p50/p95 | direct event registry visits/comparisons/path parses=0; command/operation lookup near O(logN)/O(1); route identity and journal/replay semantics preserved |
| menu/reflection: commands 1/100/10k; rows 1/100/10k; stable/enablement/structural generations | descriptor scans, menu row/string builds, predicate evaluations, shell+command lock overlap, view/reflection builds | structural bucket build=1/generation; stable work=0; enablement work near affected slots; no broad projection/publish while holding command lock |
| palette: commands 1/100/10k/100k; rare/common/empty queries; 1k append/backspace edits; offsets 0/12/504 | posting sizes, document bytes/comparisons, enablement evaluations, heap handles, MRU comparisons, row/UI clone bytes, p50/p95 | rare query retains current indexed behavior; common/incremental work follows candidate frontier; retained rows <=12; exact count/order/deep paging preserved; stable context/catalog build=0 |
| keymap: bindings 1/100/10k; events 1/1M; collisions 0/1/100%; overrides 1/1k | key/String allocations, signature probes, exact comparisons, rebuild bytes/time | stable event owned-key allocation=0; lookup near O(1)+real collisions; one build per settings generation; alias/dead/unidentified/release parity |
| registration: plugins/commands/views 1/100/10k; duplicate first/middle/last; failures/reload/unload | registry clone bytes, descriptor scans, index builds, generations, lock wait/hold, rollback bytes, stale commits | one candidate/index build and one publish per successful batch; failure publish=0; foreign work under shell/command lock=0; old readers quiesce safely |
| F4 product before/after at 30/60/120 Hz | WPR CPU stacks, contention, allocations/RSS, context switches/package power; input-to-present p50/p95/p99 | command stages separately attributable and reduced; keyboard/menu/palette/dispatch behavior passes on same machine; no numeric Unreal budget is invented |

RenderDoc is not applicable to routing, search, locks or registration. After a current editor launches,
it may confirm menu/palette draw-count and overdraw parity only; WPR/xperf owns CPU, contention,
allocation, latency and package-power evidence.

## Static gates executed

- Read all 17 Rust files (4,285 lines, 30 `#[test]`/`#[cfg(test)]` markers) at source fingerprint
  `3807492cc6aa0f5e3d337dfcf0b941ee689b32b9633876dc372224bea6acb06e` and traced the production
  callers and reference sources above. A post-review recount reproduced the same fingerprint.
- Folder `rustfmt --edition 2021 --check` is red on the foreign dirty import ordering in
  `registry.rs:12-15`. Fifteen per-entry checks are green; the direct `registry.rs` entry and the
  recursive `mod.rs` entry both surface that same underlying diff. No source file was formatted or
  changed by this review.
- `git diff --check` is green; Git only reports existing LF/CRLF conversion warnings.
- Explicit local-path validation resolved all 19 referenced source paths (`19/19`).
- The two owned performance documents have zero documentation-convention violations. The repository
  baseline remains 671 violations across 241 of 2,508 checked documents; this review does not claim
  ownership of that pre-existing global debt.
- `python -m tools.session_coordinator --repo-root . --json plan audit` and the session heartbeat are
  green after both documents were written.
- Managed Cargo cannot run while `tools/build-editor.ps1:130` rejects approved D:/E:/F: target roots
  through its literal separator bug. See
  `failure-2026-08-15-build-editor-approved-root-separator.md`.
- WPR/xperf and RenderDoc product acceptance remain pending because no launchable current-source editor
  binary exists. No latency, power or algorithmic improvement is claimed.
