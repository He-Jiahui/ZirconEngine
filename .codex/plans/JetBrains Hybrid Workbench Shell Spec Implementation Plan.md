# JetBrains Hybrid Workbench Shell Spec + Implementation Plan

## Summary
- Build a config-driven HTML prototype and Slint shell for the editor workbench. Visual direction: JetBrains-like hybrid shell with a larger central viewport, cool gray palette, blue focus accents, icon-driven rails, and no cyan button wall.
- Default preset: left `Project/Assets/Hierarchy`, center `Scene/Game`, right collapsed `Inspector`, bottom `Console`. This is only the builtin preset; all panes remain movable through the full dock-tree layout.
- Empty states are pane-owned and move with the pane: `Project/Assets -> No project open + Open Project`, `Hierarchy -> no scene/nodes`, `Scene -> welcome/open/create`, `Prefab Editor -> only exists when opened`, `Inspector -> Nothing selected`, `Console -> No output yet / last task status`.

## Public Interfaces And Types
- Extend [binding.rs](E:/Git/ZirconEngine/zircon_editor_ui/src/binding.rs) so `EditorUiBindingPayload` adds typed `DockCommand`, `SelectionCommand`, `AssetCommand`, and `ViewportCommand` variants. Keep `MenuAction` and `InspectorFieldBatch` unchanged.
- Define stable control ids for shell chrome: `WorkbenchMenuBar/*`, `ActivityRail/*`, `ToolWindow/*`, `DocumentTabs/*`, `InspectorView/*`, `ViewportToolbar/*`, `StatusBar/*`. HTML and Slint controls must both map to these ids.
- Keep [layout.rs](E:/Git/ZirconEngine/zircon_editor/src/workbench/layout.rs) as the authoritative dock-tree model. `WorkbenchLayout`, `DocumentNode`, drawer slot/mode, floating window, and restore policy remain the persisted schema.
- Extend the workbench view model in [model.rs](E:/Git/ZirconEngine/zircon_editor/src/workbench/model.rs) and [view.rs](E:/Git/ZirconEngine/zircon_editor/src/workbench/view.rs) to emit pane metadata, pane empty states, tool-window tab groups, closeability flags, and chrome actions without assuming fixed left/right/bottom ownership.
- Canonical docs live in [editor-workbench-shell.md](E:/Git/ZirconEngine/docs/editor-and-tooling/editor-workbench-shell.md) and [ui-binding-reflection-architecture.md](E:/Git/ZirconEngine/docs/editor-and-tooling/ui-binding-reflection-architecture.md). Older `.codex/plans` workbench notes are background only.

## Implementation Changes
- Formalize the spec in `docs/editor-and-tooling/editor-workbench-shell.md`: document the Hybrid Shell visual system, the default preset, movable pane behavior, pane-owned empty states, close rules (`Scene/Game` non-closeable, `Prefab Editor` on-demand), and the distinction between builtin preset and user layout.
- Update `docs/editor-and-tooling/ui-binding-reflection-architecture.md` to lock the shell event contract: menu actions stay on `WorkbenchMenuBar`, tool-window visibility/activation and dock operations move to `DockCommand`, selection synchronization to `SelectionCommand`, asset open/reveal to `AssetCommand`, viewport toolbar actions to `ViewportCommand`.
- Add a standalone HTML prototype at `docs/editor-and-tooling/prototypes/editor-workbench-hybrid-shell.html`. Make it single-file HTML/CSS/JS, JetBrains-styled, and configuration-driven: it renders the builtin preset plus at least one alternate preset where `Project` docks to the right, proving pane placement comes from layout JSON rather than DOM hardcoding.
- Feed the prototype from fixture-shaped data compatible with [default-layout.json](E:/Git/ZirconEngine/zircon_editor/fixtures/workbench/default-layout.json) and [view-descriptors.json](E:/Git/ZirconEngine/zircon_editor/fixtures/workbench/view-descriptors.json). The prototype does not need full HTML-side drag/drop; it must faithfully render different dock-tree presets and pane empty states.
- Extend the backend-neutral chrome builder and presenter so tool-window content is resolved from instance descriptors and layout host, not from fixed slots. `Project`, `Assets`, `Hierarchy`, `Inspector`, and `Console` remain panes that can live in any drawer stack. `Scene`, `Game`, and `Prefab Editor` remain document panes or exclusive/floating pages according to descriptor and host.
- Add typed shell commands and dispatchers: menu bindings keep using [event.rs](E:/Git/ZirconEngine/zircon_editor/src/workbench/event.rs); new dispatchers translate dock/selection/asset/viewport payloads into `LayoutCommand`, workbench focus changes, selection intents, asset intents, and viewport intents. Hot-path UI must not depend on string parsing.
- Implement the Slint host shell as a config-driven dock host with `WorkbenchShell`, `WorkbenchTopBar`, `ActivityRail`, `ToolWindowStack`, `DocumentWorkspaceHost`, `WorkbenchStatusBar`, and pane components for `Project`, `Assets`, `Hierarchy`, `Inspector`, `Console`, `Scene`, `Game`, and `Prefab Editor`. Material components provide base controls only; apply a JetBrains-like theme layer over them.
- Preserve existing restore semantics in `LayoutManager`: project layout overrides global layout, which overrides builtin Hybrid preset. The builtin preset stays equivalent to the current fixture shape but may evolve visually; the persisted schema remains `WorkbenchLayout`.
- Ensure pane behavior is layout-agnostic: if a user docks `Project` on the right or `Console` on the left, pane content and empty states stay identical. Only the host slot changes.

## Default Behavior
- Default preset on startup: left tool-window stack with `Project` active and `Assets/Hierarchy` alongside it; center `Scene` active with `Game` present; right tool-window collapsed with `Inspector`; bottom tool-window open on `Console`.
- Pane toggles: clicking a rail icon activates the pane if hidden, switches to it if another pane is visible in the same stack, and collapses the stack if the same active pane is clicked again.
- Document tabs: `Scene` and `Game` are always present and not closeable. `Prefab Editor` creates a tab only when a prefab opens and can then dock, float, or become exclusive according to its descriptor and current host.
- Selection sync: hierarchy click updates scene selection and inspector subject; viewport click updates hierarchy and inspector; inspector batch commit remains the only persisted property-edit path.
- Menus vs buttons: project/save/layout/reset/undo/redo/create/open-view stay menu-driven; viewport tools, rail toggles, inspector apply/revert, and run controls remain visible buttons or icon buttons.
- Empty states:
  - `Project` / `Assets`: `No project open` with primary `Open Project`; recent projects stays in a menu.
  - `Hierarchy`: short empty state when no scene or no nodes.
  - `Scene`: welcome state with quick actions when no project; `Open Scene` / `Create Scene` when a project exists but no active scene.
  - `Prefab Editor`: no placeholder tab before open.
  - `Inspector`: `Nothing selected`, no disabled-form graveyard.
  - `Console`: last task status or `No output yet`.

## Test Plan
- Binding tests:
  - `EditorUiBindingPayload` roundtrip for `MenuAction`, `InspectorFieldBatch`, `DockCommand`, `SelectionCommand`, `AssetCommand`, and `ViewportCommand`.
  - Stable `nativeBinding` strings for representative shell controls such as `WorkbenchMenuBar/OpenProject`, `ActivityRail/ProjectToggle`, `InspectorView/ApplyBatchButton`, and `ViewportToolbar/TranslateTool`.
- Layout tests:
  - Existing serialization/restore tests keep passing for `WorkbenchLayout`.
  - Add cases where `Project` or `Inspector` is moved to a non-default drawer slot and the restored layout still renders the same pane content and empty state.
  - Add coverage for `Scene/Game` non-closeable behavior and `Prefab Editor` on-demand tab creation.
- View-model tests:
  - `WorkbenchViewModel` emits correct pane groups, active tabs, visibility flags, closeability flags, and empty-state models for default and alternate presets.
  - Exclusive page mode still hides or freezes drawer chrome without losing persisted drawer state.
- Prototype tests:
  - The standalone HTML renders the builtin preset and the alternate preset from fixture-shaped config without changing DOM structure.
  - The alternate preset proves a right-docked `Project` pane still shows `No project open`.
- Interaction tests:
  - Menu binding dispatch still returns `WorkbenchHostEvent::Menu`.
  - New dock commands drive drawer expand/collapse, tab activation, and main-tab close attempts correctly.
  - Selection propagation and inspector batch commit remain atomic and undoable.
- Manual acceptance:
  - No cyan button wall; palette, borders, tabs, rails, and status bar read as JetBrains-like.
  - The shell still feels editor-first because the center viewport dominates the default preset.
  - Loading a saved project-specific layout rearranges panes without changing pane semantics or empty states.

## Assumptions
- The HTML prototype is the visual oracle for the Slint shell, not the shipping runtime host.
- Full user-driven docking continues to use the existing `WorkbenchLayout` and `LayoutManager` model rather than introducing a second layout schema.
- The builtin Hybrid preset is source-controlled; global and project layouts remain user data.
- `docs/editor-and-tooling/editor-workbench-shell.md` becomes the formal spec for this work. Implementation should update that document in the same change set as the HTML prototype and shell code.
