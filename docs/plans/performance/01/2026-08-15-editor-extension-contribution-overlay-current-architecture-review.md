# Editor extension contribution and overlay current-architecture review

## Status

- Result: `static_complete / dynamic_pending`.
- Review date: 2026-08-15.
- MVP priority: P0 for editor/plugin registration and first/cache-miss viewport interaction; P1 for
  full plugin-pane recompute and compatibility query allocation.
- Owner: Editor12 owns the plugin registration transaction; Editor05 owns viewport overlay demand;
  EditorUI08 owns pane demand; Plugins01 owns contribution generation and reload/unload identity;
  Runtime11 is used only after a callback declares non-main affinity and bounded immutable input.
- Accounting: keep this module in `pending.md`. Do not add it to `review.md` before the current-source
  managed Cargo gate, scale counters and F0/F4 WPR acceptance matrix pass.
- Code disposition: no Rust source changed. Several reviewed production/caller files contain
  pre-existing modified or untracked work; those owners and bytes were preserved.

## Exact scope

| scope | files | physical lines | tests | ordered path-and-raw-content SHA256 |
|---|---:|---:|---:|---|
| `zircon_editor/src/core/editor_extension.rs` plus `core/editor_extension/**` | 5/5 | 1,540 | 5 inline | `3a23c87b5234807b1768eb04425b7f3138a30741e75b965f584ceb31a3e81777` |

The fingerprint is SHA256 over normalized sorted path, NUL, raw file bytes, NUL. Every Rust file in
the exact scope was read in full. Registration, immutable contribution storage, host accessors,
retained pane collection, viewport provider installation/extraction and interaction-extract caching
were traced as supporting callers. Existing registration/validation, plugin SDK, store publication,
pane-gating and interaction-cache tests were inspected for behavioral coverage; they do not add to
the five-file accounting above.

## Architecture verdict

The July `PERF-MVP-538` diagnosis is partly stale. Current stable contribution readers no longer
clone the complete active registry. `ContributionSnapshot` owns one `Arc<BTreeMap<...>>` per typed
family, shares required capability slices through `Arc<[String]>`, and exposes borrowed filtered
iterators (`core/extension/store/model/snapshot.rs:23-90`). Targeted shell-content recompute asks for
only the active template source, and `ViewportInteractionExtractCache` reuses one shared extract on
key hits. The old report must not describe all pane sources or overlay providers as unconditional
per-frame work.

The current P0 problem is instead the registration transaction. `register_editor_extension_owned`
holds the workbench shell mutex from line 125 through line 310. Under that lock it validates and
materializes the candidate, deep-clones the full `ContributionBatch`, clones the complete command
registry, installs views/modes/providers and finally publishes the contribution
(`ui/host/editor_extension_registration.rs:115-310`). Most critically, provider preparation invokes
the foreign plugin factory `registration.create()` while the shell mutex is held
(`scene/viewport/controller/scene_viewport_controller_overlay_providers.rs:66-86`). A slow, blocked
or reentrant factory can therefore stall unrelated editor shell work or deadlock on shell-dependent
plugin code.

Registration also rebuilds an `AssetTypeRegistry` from builtins and clones/reapplies every existing
asset-type contribution for every new extension (`editor_extension_registration.rs:475-487`). With
`P` plugins each contributing one asset family, sequential admission can approach `O(P^2)` replay.
The candidate installs manager views and viewport registries before contribution-store publication;
failure atomicity is already owned by
`docs/plans/zircon_editor/editor/02/failure-2026-08-01-plugin-registration-runtime-consumer-atomicity.md`.
Performance work must converge with that transaction fix instead of introducing a second cache or
parallel registration authority.

## Current-source corrections

1. `ContributionSnapshot` is a shallow generation snapshot. Stable readers borrow typed rows and
   filter against shared capability requirements; the legacy allocating getters on
   `EditorExtensionRegistry` remain builder/compatibility overhead, not the primary stable path.
2. `collect_shell_content_pane_payloads` gates by visible content kind and resolves one active
   template id (`ui/retained_host/app/host_lifecycle/pane_payloads.rs:28-108`). The broad host
   lifecycle path still calls every enabled source only when a full host recompute is requested
   (`pane_payloads.rs:180-183`). Existing `PERF-MVP-595` correctly owns that case.
3. Pane source callbacks run after the shell mutex is released. The full accessor does clone all
   enabled ids and source `Arc`s into a `BTreeMap`; the targeted accessor performs an indexed lookup
   and invokes one source outside the lock
   (`ui/host/editor_event_runtime_access/extension_access.rs:111-155`).
4. Overlay callbacks are not unconditional per frame. They execute when the interaction extract is
   rebuilt after a key miss or explicit invalidation; stable key hits clone only the cached `Arc`
   (`scene/viewport/interaction_extract/cache.rs:24-103`). Cache hit/miss and rebuild counters already
   exist.
5. Provider callback panic/failure crosses a plugin boundary, records the last failure and
   quarantines that provider (`scene_viewport_controller_overlay_providers.rs:140-170`). This bounds
   repeated failure, but not a slow successful callback.
6. UI template replacement prepares candidate templates/sources and publishes them together
   (`core/editor_extension/template_contributions.rs:54-100`). Preserve this single-candidate model.

## Remaining bottlenecks

### P0: foreign factory execution and wide candidate work under the shell mutex

- `ContributionBatch::clone` duplicates all owned descriptors, ids, strings and maps so one copy can
  be destructively consumed and the other published. `EditorCommandRegistry::clone` copies the full
  prior command registry before adding the candidate (`editor_extension_registration.rs:137,159`).
- View validation, asset replay, capability maps/sets, operation binding validation, provider factory
  calls and manager/provider/mode install all extend one shell-lock critical section. No counter
  records shell wait/hold time or the individual registration stages.
- Calling `registration.create()` in this critical section admits arbitrary foreign code under a
  central editor mutex. Panic quarantine does not bound a callback that returns slowly or waits for a
  shell service.
- Existing functional tests prove validation and publication behavior, but no test asserts that all
  foreign callbacks occur outside shell/command locks or measures work across 1/100/10k plugins.

### P0: cache-miss overlay callback amplification

- On a miss, extraction iterates every enabled, non-faulted provider and synchronously calls
  `provider.extract(&context)`, then concatenates every returned vector
  (`scene_viewport_controller_overlay_providers.rs:140-170`). One slow provider delays first render,
  invalidation rebuild and pointer fallback.
- The provider receives the whole borrowed `&Scene`. There is no declared data generation, dirty
  predicate, callback affinity, per-provider span/wall counter, deadline, last-good result or output
  entry/byte cap. Moving this callback to a worker without an immutable scene contract would violate
  World/editor authority and is not an acceptable shortcut.
- The cache already removes steady-key repetition. Optimize the invalidation/rebuild contract and
  provider demand, not a falsely asserted per-frame loop.

### P1: broad pane snapshots and query projection

- `EditorUiTemplatePaneDataSource::snapshot` returns a fully owned snapshot and exposes no generation,
  `NotModified`, dirtiness, deadline, cancellation, encoded-byte or affinity contract
  (`core/editor_extension/view_descriptor.rs:101-103`). Full host recompute synchronously calls every
  enabled source and materializes all results.
- `ui_template_descriptor` scans all enabled templates before cloning the hit; importer lookup scans
  all importers; several host accessors rebuild an owned `CapabilitySet` from enabled strings per
  call (`extension_access.rs:22-57,158-183`). These are scale amplifiers, but measurement must establish
  whether they matter after registration and broad pane work are separated.
- `enabled_plugin_template_descriptors` clones and partitions every plugin template, but its consumer
  is generation/capability cached. Treat this as generation-rebuild cost, not stable frame work.

## Reference-engine evidence

- Unreal refreshes plugin discovery through the explicit `FPluginManager::RefreshPluginsList`
  operation, rebuilding discovery and indexes only for that lifecycle action
  (`dev/UnrealEngine/Engine/Source/Runtime/Projects/Private/PluginManager.cpp:555-594`). It does not
  model plugin catalog discovery as a viewport-frame task.
- Unreal wraps enabled-plugin phase loading in a CPU trace scope, reports progress over the plugin
  count and enforces monotonic loading phases (`PluginManager.cpp:2884-2903,2951-2977`). Zircon should
  likewise measure validation, factory creation and publication as distinct lifecycle stages before
  changing scheduling.
- Newly created Unreal plugins are mounted explicitly and then broadcast as one lifecycle event
  (`PluginManager.cpp:3336-3348`). This supports a prepared-candidate then publish boundary; it does
  not justify running plugin code under Zircon's shell mutex.
- Unreal editor component visualizers iterate the visualizers selected for the current selection,
  check an enabled predicate, then draw/HUD them
  (`dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/UnrealEdEngine.cpp:2111-2131`). Level editor
  viewports call that path only in the relevant editor/non-game views
  (`dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/LevelEditorViewport.cpp:5089-5092,5373-5381`).
  The transferable principle is demand/selection-scoped visualization. Unreal is not evidence that
  a borrowed live scene callback is safe to run asynchronously.

## Optimization plan

### Milestone 1: instrument the actual structural boundaries

- Add spans/counters for shell and command lock wait/hold; validation, asset replay, batch/registry
  clone bytes, provider factory, candidate compile, install and publish wall; candidate rows visited;
  generation/publish count and rollback stage.
- Add per-provider cache-miss callback wall, affinity, output entries/estimated bytes and failure;
  retain the existing cache hit/miss/rebuild counters. Add full/targeted pane source counts, callback
  wall, returned values/patches/estimated bytes and generation reason.
- Measure cold start, enable, disable, reload and failed registration at 1/100/1k/10k plugins with
  0/1/100 contributions per family; run F0 startup and F4 edit/viewport scenarios through WPR/xperf
  when the approved-root build helper is repaired. RenderDoc is relevant only after a current editor
  can render a scene; it cannot prove this CPU lock/callback boundary by itself.

### Milestone 2: one prepared candidate, foreign work outside locks, one commit

- Snapshot the immutable contribution, command and capability generations under short locks. Validate
  pure descriptors and build a single owned candidate outside the shell lock. Invoke plugin factories
  outside shell/command locks through the existing plugin boundary.
- Reacquire the shell once, verify the captured generations, and atomically publish manager views,
  command registry, contribution snapshot, scene modes and providers. On generation conflict retry the
  pure preparation once or return a typed conflict; on failure retain the complete last-good state.
- Eliminate the full `ContributionBatch` clone by splitting declarations from prepared executable
  objects or by moving one batch into a candidate whose shared frozen generation is the published
  product. Fold asset-type validation into that same candidate instead of replaying all prior rows per
  plugin.

### Milestone 3: converge queries and pane demand on the generation

- Hard-cut stable consumers from allocating compatibility getters to direct indexed snapshot queries.
  Reuse one shared enabled-capability generation rather than rebuilding `BTreeSet<String>` per host
  accessor; add id/extension indexes only in the single contribution generation.
- Extend pane sources with source generation/dirty state and `NotModified`; query only visible/dirty
  sources during normal recompute. Give explicit broad diagnostic/export collection an entry, byte
  and deadline slice. Preserve one authoritative source and one retained-host publication.

### Milestone 4: bound cache-miss overlay work

- Give each provider a declaration of selection applicability, input generation and dirty reason.
  Reuse last-good immutable output when the provider generation is unchanged. Enforce output
  entry/byte budgets and emit typed slow/over-budget/quarantine diagnostics.
- Keep main-affinity providers on the editor owner under a measured per-rebuild slice. Only providers
  that declare non-main affinity and accept a sealed immutable scene extract may use Runtime11 with
  generation cancellation. Do not pass live `&Scene` to an unbounded worker callback.
- Repeat F4 WPR and a real-scene RenderDoc capture. RenderDoc acceptance is draw/resource parity and
  absence of extra overlay submissions; CPU/power acceptance remains WPR/ETW on the same machine.

## Quantified acceptance

| matrix | required measurements | acceptance |
|---|---|---|
| registration: plugins 1/100/1k/10k; contributions/family 0/1/100; success/fail/reload | shell/command wait+hold, factory-under-lock count, batch/registry clone bytes, asset rows replayed, candidate builds/publishes, wall/RSS | foreign callbacks under shell/command locks=0; full prior-registry clone=0; work near changed rows plus one candidate; successful transaction publishes one generation and failure publishes zero |
| pane: sources 0/1/100/1k; stable/1% dirty; visible one/full export | callbacks, source/value/patch visits, owned/encoded bytes, main-thread wall, generation/NotModified | stable callbacks/materialization=0; normal visible recompute invokes at most the targeted dirty source; broad path obeys entry+byte+deadline budget without stale publication |
| overlay: providers 0/1/16/100; selected none/one; stable/miss/invalidate; output 0/1k/100k | cache hit/miss, per-provider callback wall/affinity, output entries/bytes, rebuild/input-to-present p50/p95/p99, CPU/context switches/package power | stable key callback=0; unchanged provider generation reuses last-good output; one slow provider is attributed and cannot make work/backlog unbounded; selection and render parity hold |
| product F0/F4 before/after | WPR CPU stacks, lock/contention, ready/running time, allocations/RSS, package power; RenderDoc draw/resource/event counts | registration and cache-miss stages are separately attributable and reduced; startup, reload, pane, selection, overlay, undo/replay and plugin quarantine semantics pass; numeric claims use same-machine deltas only |

## Static gates executed

- Read all 5 production files twice at the recorded fingerprint and traced the current registration,
  contribution snapshot, host pane/query and viewport extraction callers plus the Unreal sources.
- `rustfmt --edition 2021 --check` is green for all 5 exact-scope files. No foreign source was
  formatted or rewritten.
- Managed Cargo did not run because `tools/build-editor.ps1:130` still rejects valid D:/E:/F: roots
  through its single-quoted doubled-separator bug. See
  `failure-2026-08-15-build-editor-approved-root-separator.md`.
- WPR/xperf and RenderDoc are installed, but there is no launchable current-source editor binary.
  No latency, power, rendering or algorithmic improvement is claimed; dynamic evidence remains
  mandatory.
