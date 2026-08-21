---
related_code:
  - zircon_editor/src/ui/workbench/event
  - zircon_editor/src/core/editor_event/workbench
  - zircon_editor/src/ui/workbench/layout
  - zircon_editor/src/ui/host/editor_event_execution/layout_command.rs
  - zircon_editor/src/ui/workbench/reflection/model_build.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/pending.md
  - docs/plans/performance/review.md
owner_plans:
  - docs/plans/optimize/zircon_editor/08-command-registry-keymap-menu-palette-context-routing-remote-automation-review.md
  - docs/plans/optimize/zircon_editor/13-layout-profile-workspace-state-docking-tab-window-restore-migration-review.md
  - docs/plans/optimize/zircon_editor/49-editor-event-runtime-envelope-listener-registry-journal-replay-snapshot-dirty-lifecycle-product-integration-review.md
  - docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md
  - docs/plans/zircon_editor/editor/08-tool-orchestration-and-commands.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
source_evidence:
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Framework/Docking/TabManager.h
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Docking/TabManager.cpp
  - dev/UnrealEngine/Engine/Source/Developer/ToolMenus/Public/ToolMenuEntry.h
  - dev/UnrealEngine/Engine/Source/Developer/ToolMenus/Private/ToolMenuEntry.cpp
---

# Protected plan routing: Workbench event canonical Layout command generation

## Reason for routing

The main performance plan, `pending.md`, `review.md`, Optimize08, Optimize13, Optimize49 and numbered
owner plans are protected or foreign dirty. `zircon_editor/src` is held by the active MVP00 session.
This record routes the current 13/13-file evidence without editing those authorities. Evidence source:
`2026-08-19-editor-ui-workbench-event-canonical-layout-command-generation-architecture-review.md`.

## Requested Performance01 updates

### PERF-MVP-077

Add the current in-crate command roundtrip. Retained callbacks convert the 15-variant UI
`LayoutCommand` into an identical Core `LayoutCommand`; shell-locked execution then clones all owned
IDs, names and path vectors back into the UI copy before `LayoutManager::apply`. Replace both with the
single canonical command and ID schema owned by Optimize13's `LayoutAuthority`.

Required counters: conversion passes, ID/String/path copied bytes, shell-lock wait/hold, affected-node
visits and commit publications for every command at `1/1k/10k` tabs. Conversion and duplicate serde
schemas must be zero after hard cutover.

### PERF-MVP-097

Correct the no-op conclusion. `LayoutManager` can return `changed=false`, but
`execute_layout_command()` still publishes `LayoutChanged`, `PresentationChanged` and
`ReflectionChanged`. Downstream retained and reflection code therefore schedules full work.

Required target: typed `LayoutCommitReceipt` with exact diff/effects. Repeated focus/mode/extent/page
at `1/1k/1M` must report no layout/presentation/reflection effects, no chrome/menu/reflection build and
no persistence write. Preset persistence uses a separate explicit receipt domain rather than relying
on topology `changed`.

### PERF-MVP-076 and PERF-MVP-099

Add menu binding amplification to full Workbench reflection. The live pointer projection builds one
owned binding per leaf, while reflection builds two for the same leaf. A no-op Layout command can
trigger the full reflection path because of unconditional effects. Editor08 publishes immutable menu
rows by command definition generation; EditorUI08 shares that generation across pointer, presentation
and reflection and patches only affected dynamic state outside command/shell locks.

Required matrix: menu entries `1/100/10k/100k`, stable/context/reload/no-op layout. Stable static graph,
binding, route, label/path/shortcut String allocation and full registry scan all zero; definition
generation builds once; context changes visit affected entries.

### PERF-MVP-572

Extend compiled binding identity to Workbench menus. A typed `EditorOperationPath` is currently copied
into control and payload Strings; retained menu fallback copies action text into a binding, decodes it
to `MenuAction`, wraps/unwraps `EditorHostEvent`, maps it to another operation string and parses a new
owned path. Local invocation must carry a compiled command handle and perform zero native codec,
binding construction or operation parse.

### PERF-MVP-062 and PERF-MVP-175

PERF-MVP-062's streaming validator does not make per-click reparsing free; parsing must move to command
catalog admission/generation. PERF-MVP-175's stable pointer layout must consume the same immutable
menu graph and compiled action handles; it cannot solve geometry reuse while rebuilding action
bindings for every row.

## Requested Optimize and owner updates

### Optimize13 and EditorUI08

Own one canonical `LayoutAuthority`, command/ID schema, staged transaction and typed commit receipt.
Delete Core/UI duplicate enums and `core_layout_command_from_ui`/`ui_layout_command_from_core` in the
same hard cut. No-op topology emits no UI invalidation; persistence and diagnostics are explicit
receipt domains.

### Optimize08 and Editor08

Attach this evidence to E-CMD-P1-33..36 and M4. One versioned command/menu catalog generates identity,
external codec, binding/route handles and explicit menu entry kinds. Local UI carries handles to the
InvocationGateway. Remove parallel action/control/operation maps, empty bindings and legacy aliases
after metered migration.

### Optimize49 and Editor01

Only changed semantic layout commits become commit receipts. No-op requests may be sampled for audit
but do not advance document revision or trigger journal/listener/UI work. The one-variant
`EditorHostEvent` is not a durable event authority and should be removed.

## Requested protected index state

- `pending.md`: add or retain one concise row for `zircon_editor/src/ui/workbench/event/**` with 13/13
  files, 768 lines, fingerprint `e48bdd7e...`, and
  `static_complete / structural_cutover_required / dynamic_pending`.
- `review.md`: do not add the module. Require canonical Layout hard cutover, no-op effect suppression,
  shared menu generation, legacy codec retirement, scale counters, current-source Cargo, F4 product
  trace, WPR/ETW and package-power evidence.
- Keep protected indexes module-level and concise; detailed evidence stays in the companion review.

## Acceptance handoff

| owner | required proof |
|---|---|
| Optimize13 + EditorUI08 | one Layout command/ID schema; conversion copied bytes `=0`; failure atomicity; no-op effects/rebuild/write `=0`; changed command one generation/diff |
| Optimize08 + Editor08 | one command/menu catalog; stable menu binding/String build `=0`; explicit non-action kinds; owner revoke and stale handle parity |
| Optimize49 + Editor01 | no-op document revision/UI audit work `=0`; changed semantic request has one causal receipt; no host-wrapper execution authority |
| Performance01 | 31-run WPR/ETW, allocation, lock, invalidation, latency, RSS and package-power matrix on identical hardware/assets/settings; D/E/F artifacts |

RenderDoc is conditional on changed menu/layout geometry, clipping, resources or visible output. CPU
command/invalidation ownership is accepted with WPR/ETW, allocation and domain counters; visible
changes additionally require RenderDoc and pixel parity.
