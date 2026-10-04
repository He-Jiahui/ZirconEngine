---
related_code:
  - zircon_editor/src/core/commands
  - zircon_editor/src/ui/host/editor_event_dispatch.rs
  - zircon_editor/src/ui/host/editor_extension_registration.rs
  - zircon_editor/src/ui/host/editor_event_runtime_reflection.rs
  - zircon_editor/src/ui/retained_host/app/command_palette_actions.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/recompute
  - zircon_editor/src/ui/workbench/model
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/review.md
  - docs/plans/performance/pending.md
owner_plans:
  - docs/plans/zircon_editor/editor/08-tool-orchestration-and-commands.md
  - docs/plans/zircon_editor/editor/12-plugin-management.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
source_evidence:
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Commands/InputBindingManager.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Commands/UICommandList.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Commands/UICommandInfo.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/MultiBox/MultiBoxBuilder.cpp
  - dev/godot/editor/settings/editor_command_palette.cpp
---

# Protected plan routing: editor core commands routing and search

## Reason for routing

The main performance plan, `review.md`, `pending.md` and owner plans are protected/foreign dirty in
this session. This record routes the current-source corrections without overwriting their owners.
Evidence source:
`2026-08-16-editor-core-commands-routing-search-current-architecture-review.md`.

## New Performance01 task

Add `PERF-MVP-645` P0:

- every normalized `EditorEvent` without explicit operation metadata currently locks the command
  registry and linearly compares every registered descriptor event; common direct/transient events
  pay the complete failed scan;
- replace reverse discovery with typed
  `CommandRoute::{Command(id), Operation(id), DirectEvent}` produced during keymap/menu/binding/remote
  normalization;
- build only the finite static compatibility/replay event routes in the compiled command generation;
  dynamic payload events are never hash keys;
- inspector's constant fallback route is precompiled and no longer reparsed per event.

Acceptance: commands 1/100/10k and hover/press/focus/viewport/inspector/replay storms 1/1k/1M;
record route kind, registry visits/event comparisons, path parses, command-lock wait/hold, allocations
and p50/p95. Direct events require zero command-registry work. Command/operation identity, enablement,
journal, replay, undo and diagnostics remain equivalent.

## Existing task corrections

### PERF-MVP-074

Keep the current borrowed normalization and signature/candidate index. No second index is planned.
The source implementation remains dynamic-pending for collision/probe/allocation counters, managed
Cargo and F4 keyboard evidence. Keymap filesystem tests must use an approved D:/E:/F: temp root.

### PERF-MVP-211

Remove three stale findings: palette edits no longer deep-clone `CommandEvalCtx`, queries no longer run
under the registry mutex, and non-empty documents are no longer scanned by separate substring and
subsequence passes. Current source shares catalog/context Arcs, chooses the rarest byte posting, uses
one scorer pass and retains bounded top-K handles.

Keep P0 for common/empty/incremental queries: common-byte postings still approach N, empty query scans
all enablement and performs bounded MRU membership, and exact total count requires visiting every
candidate. Measure before selecting prefix/token/trigram indexing. Preserve exact count, score/MRU
order, deep paging and <=12 visible+overscan rows.

### PERF-MVP-076 and PERF-MVP-099

Retain the current O(7N) menu diagnosis. Route the target through one immutable
`CompiledCommandGeneration` with top-level buckets/shared row metadata and one immutable command-eval
generation with dependency bits. Shell/reflection must release command and shell locks before broad
build/publish; stable generations build/evaluate zero menu rows.

### PERF-MVP-079 and PERF-MVP-538

Register one contribution batch against one base compiled generation. Validate and build command-id,
operation, headless route/name, event route, chord, menu and palette indexes once outside shell/command
locks, then generation-check and atomically publish once. Failure publishes nothing; reload/unload
preserves last-good readers and quiescence. Do not add parallel registries or private worker pools.

## Requested owner-plan updates

### Editor08

Own the sole `CompiledCommandGeneration`, typed `CommandRoute`, reverse compatibility routes, static
menu buckets, palette discovery data and headless indexes. Preserve the current keymap and palette
source improvements.

### Editor12

Own contribution candidate preparation and one atomic command-generation commit. All plugin callbacks
and broad validation run outside shell/command locks with stale-base rejection.

### EditorUI08

Consume immutable command/eval generations. Build/apply menus and palette rows at most once per
changed generation, retain <=12 palette rows and do no stable-frame command projection. Eliminate the
parallel visible id array when the runtime-UI typed-row boundary permits it.

## Requested protected index state

- `pending.md`: replace the stale commands row with one concise row for
  `zircon_editor/src/core/commands/**`, `static_complete / dynamic_pending`, 17/17 files, 4,285 lines,
  30 tests and the current-review link.
- `review.md`: do not add it. Managed Cargo, route/menu/palette/registration counters, F4 WPR,
  same-machine CPU/RSS/package-power and input latency are absent.

## Milestone and notification state

This is a static review/routing record, not an accepted performance milestone. No git commit or WeCom
notification is due. Commit and quantified WeCom notification occur only after dynamic evidence closes
the acceptance matrix and protected indexes are reconciled by their owner.

