# Editor Core Usable Loop Design

## Goal

Converge the minimum `zircon_editor` body, excluding concrete plugin package implementations, into a usable first-stage editor loop. The editor should open, refresh, diagnose, and project its core host panes through the existing workbench shell without adding parallel plugin or export execution paths.

The first milestone is not just a compile fix. It must leave the editor body usable enough that the core panes, Plugin Manager host pane, and Build Export host pane/report entry can be reached from the workbench and rendered from stable presentation data.

## Current Evidence

- The repository architecture fixes the public root shape around `zircon_app`, `zircon_runtime`, and `zircon_editor`.
- `zircon_app` is the process/profile/main-loop host and must not regain editor pane, plugin manager, or export business ownership.
- `zircon_runtime` is the runtime absorption layer and internal runtime spine owner. It remains the authority for runtime world/service state and concrete runtime execution.
- `zircon_editor` owns editor host behavior, authoring state, workbench shell state, builtin editor panes, command/event routing, and Slint projection.
- `zircon_editor/src/ui/host/module.rs` already exposes the `EditorModule` descriptor entry point for editor host identity.
- Existing editor paths already identify the target panes through `ViewContentKind::ModulePlugins`, `ViewContentKind::BuildExport`, `editor.module_plugins`, and `editor.build_export_desktop`.
- Recent workspace validation notes report `zircon_editor` Build Export pane wiring blockers around body document identity, `BuildExportPaneViewData` assembly, `PaneData.build_export`, and `ViewContentKind::BuildExport` name/projection mapping.

## Architecture Decision

Use the current editor host architecture and converge it into one coherent usable loop:

- Workbench model and builtin descriptors own how panes are opened and identified.
- Editor state snapshots own pane-specific authoring and host-status data.
- `ShellPresentation::from_state(...)` owns the combined shell presentation boundary.
- Slint host conversion remains a pure projection layer.
- Runtime-facing data is consumed through stable runtime/plugin/export facades or host status DTOs; editor body code must not execute concrete plugin or export package internals.

This is the recommended middle path between a shallow compile-only patch and a broad hard cutover. It satisfies the chosen `Usable editor loop` milestone while minimizing conflict with active plugin/runtime/export implementation work.

## Alternatives Considered

### Usable Editor Loop Convergence

Keep the existing `zircon_editor` host architecture, tighten builtin pane descriptors and projections, and make `ModulePlugins` plus `BuildExport` host panes open, refresh, diagnose, and render consistently.

This is the selected approach because it is the smallest path that still delivers a usable first-stage editor body.

### Fast Compile-Fix Shell

Only repair the known `BuildExport` pane wiring blockers and leave broader host-pane data flow cleanup for later.

This is faster, but it does not satisfy the chosen milestone because the editor would compile without proving a stable open/refresh/diagnose/project loop.

### Hard Cutover Now

Deeply reorganize editor host, projection conversion, plugin manager, and export host boundaries in one pass.

This may be cleaner long-term, but it carries high conflict risk with active plugin/runtime sessions and is too broad for the first usable-loop milestone.

## Boundary Ownership

- `zircon_app`: process entry, profile selection, runtime/editor host launch, and main-loop ownership only.
- `zircon_runtime`: runtime absorption layer, world/service authority, runtime asset/export/plugin-facing facades, and concrete runtime execution.
- `zircon_editor`: editor host, workbench shell, builtin pane descriptors, authoring state, command/event flow, presentation snapshots, and Slint projection.
- Plugin Manager in this milestone: host pane and diagnostic surface only. It may list, refresh, and report plugin host state but must not implement concrete plugin package behavior.
- Build Export in this milestone: host pane/report entry only. It may open, display target/report summaries, refresh host diagnostics, and expose host actions but must not own export packaging internals.

## Component Design

### Workbench Model

The workbench model remains the source of truth for opened editor surfaces. `ViewContentKind::ModulePlugins` and `ViewContentKind::BuildExport` identify the builtin panes. Menu, tab, and activity-view actions should route through the same workbench document/activity view flow instead of introducing a second host-pane route.

### Builtin View Descriptors

Builtin descriptors provide stable host-visible identity. `editor.module_plugins` and `editor.build_export_desktop` need consistent descriptor metadata: label, icon/name mapping, default placement, and document/activity metadata. Descriptor lookup should be the common path for opening these surfaces.

### Editor State Snapshot

Editor state snapshots own the authoring-side pane inputs. For this milestone that includes plugin host status, module/plugin diagnostic rows, export target summaries, export report summaries, last refresh state, and host-level messages.

### Shell Presentation

`ShellPresentation::from_state(...)` combines workbench layout with pane-specific view data. Its assembly path should provide both `ModulePluginsPaneViewData` and `BuildExportPaneViewData` together so pane availability is not dependent on local ad hoc conversion branches.

### Slint Projection

`apply_presentation` and `pane_data_conversion` translate presentation data into Slint-facing structs. They must not load plugins, execute exports, mutate runtime-owned world state, or repair editor business state. Projection failures should collapse into safe fallback rows, cards, and messages.

## Data Flow

The canonical flow is:

```text
menu/action
  -> workbench view/document model
  -> builtin descriptor lookup
  -> editor state snapshot
  -> ShellPresentation::from_state(...)
  -> pane data conversion
  -> Slint host projection
```

This flow keeps opening, refresh, diagnostics, and projection on one host path. It also keeps concrete runtime/plugin/export execution outside the first editor-body milestone.

## Diagnostics And Failure Handling

- Both Plugin Manager and Build Export panes must open even when view data cannot be fully assembled.
- Empty, loading, stale, and error states must be explicit data states rather than implicit missing UI rows.
- Missing descriptors, unavailable runtime/export/plugin host capabilities, stale refresh results, and projection conversion errors should be visible as host-level diagnostics.
- Runtime capability failures should report as editor presentation data, such as capability unavailable, rather than causing the editor to reach around the runtime boundary.
- Refresh operations must be idempotent. Repeated open/refresh actions must not duplicate tabs, corrupt workbench state, or mutate runtime-owned world state.
- Slint conversion failures remain local to projection. Projection code may emit fallback data but should not repair state or trigger runtime side effects.

## Validation Plan

Validation should run in layers so failures identify the broken boundary:

- Compile gate: `cargo check --workspace --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-core-usable-loop --message-format short --color never`
- Editor lib gate: `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-core-usable-loop --message-format short --color never`
- Workbench gate: tests that prove `ViewContentKind::ModulePlugins` and `ViewContentKind::BuildExport` resolve through menu/view descriptors and open as stable workbench surfaces.
- Presentation gate: tests that prove `ShellPresentation::from_state(...)` receives and exposes both `ModulePluginsPaneViewData` and `BuildExportPaneViewData`.
- Projection gate: tests that prove pane conversion maps both panes to Slint-facing data, including empty and error states.
- Regression gate: existing host pane presentation tests such as `zircon_editor/src/tests/host/pane_presentation.rs` and workbench tests such as `zircon_editor/src/tests/workbench/view_model/document_workspace.rs` should cover the known Build Export wiring blocker.

Do not claim workspace-green status without fresh workspace validation after implementation. If unrelated active-session failures remain, record the exact blocker and the evidence that this milestone did not touch that path.

## Acceptance Criteria

- `Plugin Manager` can be opened from the editor workbench route and renders host status, empty, and diagnostic states from presentation data.
- `Build Export` can be opened from the editor workbench route and renders host target/report, empty, and diagnostic states from presentation data.
- `ViewContentKind::ModulePlugins` and `ViewContentKind::BuildExport` have stable descriptor, display-name, menu/open, presentation, and projection coverage.
- `ShellPresentation::from_state(...)` has one coherent state assembly path for both panes.
- Slint projection consumes view data only; it does not own plugin loading, export execution, or runtime mutation.
- Refresh/open actions are idempotent and do not duplicate tabs or corrupt workbench state.
- Validation covers compile, workbench routing, presentation assembly, Slint projection, and empty/error diagnostics.

## Out Of Scope

- Concrete plugin package implementations.
- Runtime plugin package internals.
- Export packaging internals.
- Shipping/export artifact generation.
- Full plugin execution lifecycle beyond host-visible status and diagnostics.
- Broad editor host hard cutover unrelated to the two first-stage host panes.
- Removing every large editor file in this milestone unless the touched code must be split to keep the pane boundary clear.

## Reference Alignment

- Current `zirconEngine` architecture plans lead: fixed root packages remain `zircon_app`, `zircon_runtime`, and `zircon_editor`.
- Fyrox editor precedent supports editor-owned plugin/export panes with engine/runtime behavior kept behind editor-facing interfaces.
- Godot editor precedent supports project/export and plugin settings as editor host surfaces while concrete export/plugin implementations remain separated.
- Slint remains a toolkit projection detail; Slint-facing data should not become the authoring or runtime authority.

## Risks

- Existing projection conversion is concentrated in large files. Mitigation: touch the minimum necessary path, but split only the pane-specific conversion if adding more responsibility would worsen boundary clarity.
- Active plugin/runtime sessions may touch neighboring plugin contracts. Mitigation: keep this milestone at host-pane status/report projection and avoid concrete plugin package internals.
- A compile-only fix may be tempting because the known blocker is concrete. Mitigation: keep the usable-loop acceptance criteria as the milestone gate.
- Runtime capability surfaces may still be incomplete. Mitigation: represent unavailable capabilities as explicit pane diagnostics rather than editor-owned fallback implementations.
