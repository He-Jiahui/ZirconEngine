---
related_code:
  - zircon_editor/src/ui/workbench/layout_preset.rs
  - zircon_editor/src/ui/workbench/page_layout_template.rs
  - zircon_editor/src/ui/workbench/preset
  - zircon_editor/src/ui/host/layout_persistence.rs
  - zircon_editor/src/ui/host/layout_commands.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/shell_presentation.rs
tests:
  - zircon_editor/src/tests/workbench/layout/layout_preset_persistence.rs
  - zircon_editor/src/tests/workbench/layout/page_layout_templates.rs
  - zircon_editor/src/tests/workbench/layout/editor_layout_contracts/layout_commands.rs
  - zircon_editor/src/tests/host/builtin_window_descriptors.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/pending.md
  - docs/plans/performance/review.md
  - docs/plans/optimize/zircon_editor/13-layout-profile-workspace-state-docking-tab-window-restore-migration-review.md
owner_plans:
  - docs/plans/zircon_editor/editor_layout/04-layout-presets-and-persistence.md
  - docs/plans/zircon_editor/editor_layout/05-page-layout-templates.md
  - docs/plans/zircon_editor/editor_layout/07-windowing-chrome-tabs-and-dockable-drawers.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
reference_code:
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Docking/LayoutService.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Docking/TabManager.cpp
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/WorkflowOrientedApp/WorkflowTabManager.cpp
doc_type: current-architecture-performance-review
status: static_complete_exact_topology_definition_generation_hard_cut_required_dynamic_pending
source_recheck_required: true
created_at: 2026-08-19
---

# Editor layout preset exact topology and compiled-definition review

## Status

- Result: `static_complete / exact_topology_definition_generation_hard_cut_required /
  dynamic_pending`.
- MVP priority: P0 for page switching, topology preservation and live-state atomicity; P1 for the
  presentation-time design-stack construction after it is measured separately.
- Accounting: keep the nine Rust files below in `pending.md`. The existing focused tests do not
  prove product template use, exact roundtrip, atomic switching, scale or steady-state allocation.
- Code disposition: no Rust source changed. Two production files and three focused tests have
  foreign formatting changes. The obvious four-ID presentation cleanup is deliberately not applied
  before allocator/profile counters establish a current-source baseline, as required by this audit.

## Exact scope

| scope | files | physical lines | tests | raw bytes | sorted path-LF-content SHA256 |
|---|---:|---:|---:|---:|---|
| `layout_preset.rs`, `page_layout_template.rs`, `preset/**` | 9/9 | 2,390 | 14 in-module | 81,234 | `93eb2fbaa7957a2c3218a479d927d81c23ae82349a0de381a122d5546a477f81` |
| focused preset/template/descriptor tests | 4/4 | 930 | 21 | 33,155 | `c9f93efff18c09d03e135060214c0a6e932db07dbe52a41175e137e517c08946` |

The fingerprint is SHA256 over each sorted normalized path, LF, then raw file bytes. All nine
production files and all four focused test files were read in full. Product tracing includes page
save/restore, page activation, default hybrid-layout construction, window-registry construction and
both main/native retained shell presentation callers.

## Current acceptance record

| area | current-source verdict |
|---|---|
| preset schema | A preset stores drawer modes, three integer size overrides and only `SingleDocument` or `{axis, leaf_count}`. It does not store split ratios, exact tree, tab-to-leaf allocation or per-leaf active tab. |
| capture | Capturing clones the selected activity-window drawer map, scans the document tree for leaf count and writes another owned preset into a whole-store vector. |
| apply | Applying a preset mutates canonical drawers and the legacy mirror, flattens all document tabs, then creates a new right-deep split tree with empty leaves. It has no changed result or transaction boundary. |
| fallback | Missing and version-mismatched entries produce Authoring fallback, and `restore_into_layout` immediately applies that fallback to the live layout rather than returning a no-op/recovery choice. |
| page switch | The host clones the full layout, deserializes the entire page store, captures and serializes the entire store, changes page and recomputes metadata, then deserializes again, applies preset/fallback and performs another full metadata recompute. |
| store | Entries use a sorted Vec with linear scope lookup/update and full-vector sort on insert. This is secondary to whole-value config parse/serialize and destructive topology semantics. |
| built-in definitions | The Material/Fyrox/JetBrains/Unreal design stack, four presets and 13 page templates are hardcoded Rust constructors that allocate owned strings/vectors on every construction. |
| product wiring | `PageLayoutTemplate` is referenced only by tests and re-export; `page_templates.toml` is only `include_str!`-read by a test. `presets.toml` has no product or test consumer. These assets are duplicate declarations, not compiled runtime authorities. |
| shell hot path | Every `build_host_window_shell_data` constructs the complete design stack only to move four fixed preset ID strings into `HostWindowShellData`. The call is inside the profiled retained presentation path and runs for main and native shell projections. |
| tests | Tests prove small shape, selected field values and source/asset textual agreement. They do not prove exact A-B-A topology, live product template instantiation, allocation counts, no-op behavior, failure atomicity, schema budgets or page-switch latency. |

## Structural bottlenecks

### P0: page layout persistence is a destructive semantic transform

`capture_from_layout()` reduces an arbitrary recursive document workspace to one axis and a u8 leaf
count. `apply_center_split()` then calls `collapse_document_tabs()`, which clones every unique tab
into one stack, and builds a right-deep tree whose other leaves are empty. Ratios, mixed axes, leaf
membership and active tabs are irrecoverable. A page A -> B -> A switch is therefore allowed to
change the user's layout even when no user edit occurred.

The implementation is internally consistent with the old EditorLayout04 specification, but that
specification is the defect: a semantic Authoring/Review/Focus/Debug preference was named and tested
as a persisted layout snapshot. The hard cut must separate three types:

- `LayoutProfile`: exact validated topology, placements, ratios, active identities and window state;
- `LayoutTemplate`: immutable default topology used only when no valid profile exists;
- `LayoutModePatch`: bounded drawer visibility/extent or focus-mode changes that do not rewrite
  document topology unless the user explicitly requests a topology-changing template.

Missing/version-mismatched profile restore returns a staged recovery decision and diagnostic. It
must not silently apply Authoring to the current live session.

### P0: a page switch repeats whole-store and whole-session work on the UI thread

The active path is:

```text
clone full WorkbenchLayout
  -> read/deserialize all page-user presets
  -> clone drawers + scan document tree
  -> linear scope update / possible whole-entry sort
  -> serialize/write all presets
  -> mutate active page + full metadata/window/native sync
  -> read/deserialize all presets again
  -> destructive preset/fallback apply
  -> full metadata/window/native sync again
```

This compounds PERF-MVP-077. A committed page-switch transaction must load the target profile from
an immutable profile generation before live mutation, validate the target page and dirty-document
decision, prepare one layout delta, then publish once. Persistence receives the committed generation
as a debounced/background durability request; no UI event synchronously reparses or serializes the
whole profile store.

### P0: declarations are duplicated but no compiled definition owns them

The actual default layout comes from `EditorUiDesignStack` Rust constructors. Separate Rust
`PageLayoutTemplate` constructors describe 13 pages but are never consumed by product code. A TOML
page-template asset repeats those pages only for a string-matching test, while `presets.toml` is not
consumed at all. The result is the cost and maintenance surface of data-driven definitions without
a loader, validator, compiler, owner generation or runtime lookup.

Choose one authored source and compile it into one immutable `EditorLayoutDefinitionGeneration`
containing typed preset IDs, page/window descriptors, panel roles, default topology and lookup
indexes. Product startup, registry creation, shell projection, tests and serialization schema all
consume that artifact. Remove the parallel Rust/TOML declarations during the same hard cut; do not
keep compatibility readers or tests that only prove two dead sources contain similar strings.

### P1: shell projection reconstructs a full design catalog for four constants

`build_host_window_shell_data()` calls `material_fyrox_jetbrains_unreal()` on every shell
presentation. That constructor creates the shell drawer catalog, 20 panel presets, eight functional
window presets and their owned IDs/titles/view lists, but the caller reads only four top-level ID
strings. The surrounding `apply_shell_presentation_from_state` profile scope confirms this work is
inside retained presentation, not startup-only definition assembly.

M0 must add `design_stack_builds`, allocated objects/bytes and scope timing. After the baseline, the
simple stopgap is to use the existing static preset ID constants or a borrowed compiled definition
handle so steady presentation has `design_stack_builds=0`. The final solution remains the compiled
definition generation above; a process-global lazy clone of the current owned graph is not a second
long-term authority.

### P1: flattening uses quadratic duplicate detection, but that is not the primary fix

`collect_document_tabs()` uses `tabs.contains()` for every encountered tab, so flattening T unique
tabs is worst-case O(T^2) comparisons plus T ID clones. Replacing the Vec check with a set would
improve the wrong operation while preserving data loss. Exact topology profiles should not flatten
at all. A bounded validator may use an indexed identity set once per restore generation to reject or
repair duplicate placements deterministically.

### P1: version and size bounds are too weak for persisted input

The store checks only one format integer. Entry/file/string counts, recursive topology, IDs and
payload bytes have no admission budgets. `CenterSplitLayout` can request 255 right-deep panes, and
size overrides have only a minimum clamp. Optimize13 owns the full validator/migration/LKG boundary;
this module supplies exact topology, profile-scope and performance acceptance requirements.

## Reference-engine evidence

- Unreal `TabManager.cpp:1101-1153` gathers persistent layout from each live docking area and also
  retains collapsed and invalid areas. It preserves the recursive layout representation rather than
  reducing it to axis plus leaf count.
- Unreal `TabManager.cpp:1220-1288` restores at an explicit profiled boundary and records areas that
  cannot be restored. `2668-2703` can keep an unknown tab in saved layout instead of deleting its
  placement when a spawner is unavailable.
- Unreal `LayoutService.cpp:244-350` saves a named layout, loads the exact user layout, validates the
  primary area and falls back to the supplied default; it also identifies/removes older versioned
  keys. This supports explicit version/default handling, not mutation of an already live layout by a
  lossy semantic preset.
- Unreal `TabManager.cpp:1164-1185` coalesces persistent saves on a deferred ticker to avoid resize
  hitches. `WorkflowTabManager.cpp:865-887` saves document payload state through document factories,
  keeping document state ownership separate from dock topology.

Unreal's persistence is not a complete crash-atomic or malicious-input reference. Optimize13 adds
staging, budgets, LKG/quarantine and atomic publication beyond it. The cited source establishes the
required exact-topology, unknown-tab, version and deferred-save direction.

## Required hard cut

1. Split exact `LayoutProfile`, immutable `LayoutTemplate` and bounded `LayoutModePatch` contracts.
2. Capture and restore exact topology with stable IDs, ratios, leaf allocations and active state;
   never flatten document tabs during ordinary page switching.
3. Compile one authored definition source into one immutable typed generation shared by startup,
   view/window registry, page/preset lookup, shell projection and tests. Delete dead duplicate assets
   and constructors in the replacing milestone.
4. Make page switch one generation-checked transaction with target validation, dirty decision,
   prepare/commit/abort and one typed layout delta. Missing/stale profile is a staged fallback choice,
   not immediate mutation.
5. Store profiles behind an immutable indexed generation and incremental dirty records. Serialize
   or flush outside the UI transaction, coalesce requests and return a durability receipt.
6. Eliminate steady presentation design-stack construction; shell reads borrowed/static IDs or the
   committed definition generation.
7. Add bounded parse/schema/migration/identity/topology/numeric admission before any live apply.
8. Replace source-text agreement tests with product instantiation, exact roundtrip, failure/no-op,
   generation and scale tests.

## Milestones

| milestone | deliverable | dependency |
|---|---|---|
| M0 | Counters for page-store read/parse/serialize/write, layout/drawer/tab clone bytes, flatten probes, metadata/native sync and presentation design-stack allocations. | current source recheck |
| M1 | Single compiled `EditorLayoutDefinitionGeneration`; product consumes it and dead Rust/TOML authorities are removed. | EditorLayout05 + EditorUI08 |
| M2 | Exact schema-v2 `LayoutProfile`, explicit template/mode patch and bounded migration/validation. | Optimize13 M2 |
| M3 | Atomic A-B-A page switching and generation-indexed, coalesced durable profile store. | Optimize13 M1/M3/M4 + EditorLayout04 |
| M4 | Shell uses borrowed definition IDs; stable presentation design-stack build/allocation is zero. | M0/M1 + EditorUI08 |
| M5 | Current-source Cargo/F4 plus WPR/ETW allocation, lock, latency, RSS and package-power matrix. | M0-M4 |

## Acceptance matrix

| gate | matrix | required result |
|---|---|---|
| exact roundtrip | mixed-axis trees, ratios, tabs/leaf `1/100/1k/10k`, active tabs, drawers/windows | A -> B -> A is byte-equivalent canonical topology and identities when no user edit occurs; flatten calls/bytes `=0` |
| page switch | pages/profiles `1/13/100/1k/10k`, stored/missing/stale/invalid, repeated switch `1/1k` | one transaction/delta/metadata/native publication; UI-thread whole-store parse/serialize/write `=0`; no-op switch work `=0`; failure/cancel live mutation `=0` |
| definitions | 13 pages, eight windows, 20 panels, plugin add/remove/reload generations | one compiled owner; duplicate Rust/TOML authorities `=0`; stable generation compile/build/alloc `=0`; product lookup uses typed indexes |
| presentation | main/native windows `1/16/1k`, stable/full/scoped recompute `1/1k/1M` | design-stack constructor calls and owned definition allocation bytes `=0`; changed definition compiles once and is shared across presenters |
| schema | legacy/current/future, unknown plugin, duplicate placement, depth/count/string/file over budget | bounded deterministic migration/validation; exact placeholder placement; stale/future input cannot overwrite LKG; second normalize has zero diff |
| product | F4 page/preset switch, dock/split/float, restart, plugin missing/reload; 31 runs | WPR/ETW CPU, allocation, lock wait/hold, switch-to-pixel p50/p95/p99, RSS and package power reported on identical hardware/config; artifacts only on D/E/F |

RenderDoc is conditional on visual changes to page/shell layout and only establishes pixel/draw
parity. It cannot prove exact persistence, allocation removal, UI-thread I/O, atomicity or power by
itself.

## Static gates executed

- Read 9/9 production files and 4/4 focused test files; reproduced the counts and both current-tree
  fingerprints above.
- Traced page save/restore and activation through ConfigManager, two metadata recomputes and native
  synchronization; traced default hybrid layout and design-stack registry construction.
- Confirmed both shell presentation callers rebuild the complete design stack for four IDs inside
  the retained presentation path.
- Confirmed `PageLayoutTemplate` and `page_templates.toml` have test-only references and
  `presets.toml` has no consumer in `zircon_editor`.
- Reproduced lossy capture/apply, O(T^2) flattening, right-deep empty split construction, whole-store
  parse/serialize and fallback mutation from current source.
- Read the cited Unreal exact layout, invalid/unknown tab, versioned load, deferred save and document
  state implementations.
- Two production files and three focused tests are foreign dirty formatting changes and were not
  edited, reverted, formatted, staged or committed.
- No managed Cargo, F4, allocator capture, WPR/ETW, package-power or RenderDoc run was started; active
  shared Cargo/rustc lanes prevent a noncompeting current-product baseline.

## Completion rule

This module remains pending until M0-M5 pass against one current-source fingerprint. A HashSet in
the flatten loop, a cached owned design stack, source-text equality or the historical 2/2 and 4/4
focused tests do not qualify. No milestone commit or WeCom completion message is permitted before
quantified current-product evidence exists.
