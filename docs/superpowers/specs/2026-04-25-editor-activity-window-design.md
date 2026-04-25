# Editor ActivityWindow And ActivityDrawerWindow Design

## Goal

Reorganize the editor window architecture so the main frame only hosts the top task bar and window tab strip. Left, right, and bottom `ActivityDrawer` state moves into concrete `ActivityWindow` instances. Windows that need drawer rails declare that capability by using the reusable `ActivityDrawerWindow` `.ui.toml` template. No new Slint business UI is allowed.

## Core Decision

Use a componentized `ActivityDrawerWindow` template.

- `EditorMainFrame` owns only `TaskBar`, `WindowTabStrip`, and `ActiveWindowHost`.
- `WorkbenchWindow`, `AssetWindow`, and `UILayoutEditorWindow` are `ActivityWindow` instances.
- A window that needs left, right, or bottom activity drawers explicitly references `ActivityDrawerWindow`.
- An `ActivityWindow` can run embedded in the main frame or as an independent native window handle.
- Business structure, layout, and event semantics come from `.ui.toml -> UiSurface -> host projection`.

## Window Hierarchy

```text
EditorMainFrame
  TaskBar
  WindowTabStrip
  ActiveWindowHost
    ActivityWindowHandle
      WorkbenchWindow | AssetWindow | UILayoutEditorWindow | ...
```

`EditorMainFrame` must not contain activity rails, drawer shells, document hosts, or workbench-specific content.

## ActivityDrawerWindow Template

Add `zircon_editor/assets/ui/editor/host/activity_drawer_window.ui.toml` as the reusable window shell.

It exposes these slots:

```text
left_top_activity
left_bottom_activity
right_top_activity
right_bottom_activity
bottom_left_activity
bottom_right_activity
content
```

The template owns generic drawer chrome, rail/header/content areas, and resize/hit-test structure. It does not contain Workbench, Asset, or UI Layout Editor business nodes.

## WorkbenchWindow

Add `zircon_editor/assets/ui/editor/windows/workbench_window.ui.toml`.

`WorkbenchWindow` uses `ActivityDrawerWindow`:

```text
left_top_activity: Hierarchy
left_bottom_activity: Project/Assets
right_top_activity: Inspector
right_bottom_activity: Details/Properties
bottom_left_activity: Console
bottom_right_activity: Output/Diagnostics
content: Scene/Game/Document workspace
```

The current root-level `ActivityDrawer` and `ActivityView` ownership moves into this window.

## AssetWindow

Add `zircon_editor/assets/ui/editor/windows/asset_window.ui.toml`.

```text
left_top_activity: Asset Tree
left_bottom_activity: Favorites/Collections
right_top_activity: Asset Details
right_bottom_activity: Preview/References
bottom_left_activity: Import Log
bottom_right_activity: Dependency/Validation
content: Asset Browser grid/list
```

The existing `editor.asset_browser` body becomes the `content` of this window.

## UILayoutEditorWindow

Add `zircon_editor/assets/ui/editor/windows/ui_layout_editor_window.ui.toml`.

```text
left_top_activity: Palette
left_bottom_activity: Hierarchy
right_top_activity: Inspector
right_bottom_activity: Style/Bindings
bottom_left_activity: Diagnostics
bottom_right_activity: Layout Debugger/Source Outline
content: Designer/Split/Source/Preview workspace
```

The existing `editor.ui_asset_editor` body becomes the content of this window.

## Layout Model

Root drawer ownership moves from `WorkbenchLayout` into window-level layout state.

Target model:

```rust
EditorMainFrameLayout {
    active_window: ActivityWindowId,
    window_tabs: Vec<ActivityWindowId>,
}

ActivityWindowLayout {
    window_id: ActivityWindowId,
    descriptor_id: ViewDescriptorId,
    host_mode: ActivityWindowHostMode,
    activity_drawers: BTreeMap<ActivityDrawerSlot, ActivityDrawerLayout>,
    content_workspace: ActivityWindowContentLayout,
    region_overrides: BTreeMap<ShellRegionId, PaneConstraintOverride>,
    view_overrides: BTreeMap<ViewInstanceId, PaneConstraintOverride>,
}
```

## Host Modes

```rust
ActivityWindowHostMode::EmbeddedMainFrame
ActivityWindowHostMode::NativeWindowHandle
```

The same window layout and event route are used in both modes.

## View Model

`ViewKind::ActivityView` should stop representing a root-level drawer view. The long-term shape is:

```rust
ViewKind::Pane
ViewKind::ActivityWindow
```

Whether a pane can enter a drawer is controlled by placement or dock policy rather than a global root drawer view kind.

## Event Flow

```text
Host input
  -> UiSurface hit-test
  -> UiActionRef
  -> EditorEventEnvelope
  -> EditorEventDispatcher
  -> ActivityWindowLayout mutation
  -> projection rebuild
```

Slint callbacks may only adapt host input. They must not directly mutate drawer or window state.

## Cutover Rule

No new Slint business tree may be introduced. Existing Slint remains only as generic host bootstrap. All new window, drawer, and pane structures must be expressed as `.ui.toml` assets.
