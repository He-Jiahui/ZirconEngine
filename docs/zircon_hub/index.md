---
related_code:
  - Cargo.toml
  - zircon_hub/Cargo.toml
  - zircon_hub/build.rs
  - zircon_hub/src/lib.rs
  - zircon_hub/src/main.rs
  - zircon_hub/src/app/mod.rs
  - zircon_hub/src/app/runtime.rs
  - zircon_hub/src/app/runtime/asset_catalog.rs
  - zircon_hub/src/app/runtime/folder_picker.rs
  - zircon_hub/src/app/runtime/learn_catalog.rs
  - zircon_hub/src/app/runtime/window_controls.rs
  - zircon_hub/src/app/runtime/team_overview.rs
  - zircon_hub/src/app/binding.rs
  - zircon_hub/src/app/view_model.rs
  - zircon_hub/src/app/view_model/assets.rs
  - zircon_hub/src/app/view_model/cloud.rs
  - zircon_hub/src/app/view_model/learn.rs
  - zircon_hub/src/app/view_model/media.rs
  - zircon_hub/src/app/view_model/plugins.rs
  - zircon_hub/src/app/view_model/team.rs
  - zircon_hub/src/app/localization.rs
  - zircon_hub/src/app/quick_action.rs
  - zircon_hub/src/state/mod.rs
  - zircon_hub/src/state/navigation.rs
  - zircon_hub/src/state/project_view.rs
  - zircon_hub/src/projects/mod.rs
  - zircon_hub/src/projects/cover.rs
  - zircon_hub/src/projects/package.rs
  - zircon_hub/src/projects/device_install.rs
  - zircon_hub/src/assets/mod.rs
  - zircon_hub/src/assets/catalog.rs
  - zircon_hub/src/learn/mod.rs
  - zircon_hub/src/learn/catalog.rs
  - zircon_hub/src/plugins/mod.rs
  - zircon_hub/src/plugins/catalog.rs
  - zircon_hub/src/team/mod.rs
  - zircon_hub/src/team/local_git.rs
  - zircon_hub/src/engines/mod.rs
  - zircon_hub/src/engines/source_engine_install.rs
  - zircon_hub/src/engines/registry.rs
  - zircon_hub/src/build/mod.rs
  - zircon_hub/src/settings/mod.rs
  - zircon_hub/src/process/mod.rs
  - zircon_hub/src/process/folder_picker.rs
  - zircon_hub/ui/app.slint
  - zircon_hub/ui/shared.slint
  - zircon_hub/ui/components.slint
  - zircon_hub/ui/projects.slint
  - zircon_hub/ui/editor.slint
  - zircon_hub/ui/builds.slint
  - zircon_hub/ui/assets.slint
  - zircon_hub/ui/cloud.slint
  - zircon_hub/ui/learn.slint
  - zircon_hub/ui/plugins.slint
  - zircon_hub/ui/team.slint
  - zircon_hub/ui/placeholder.slint
  - zircon_hub/ui/settings.slint
  - zircon_hub/assets/brand/zircon-mark.svg
  - zircon_hub/assets/icons/nav/projects.svg
  - zircon_hub/assets/icons/nav/editor.svg
  - zircon_hub/assets/icons/nav/assets.svg
  - zircon_hub/assets/icons/nav/builds.svg
  - zircon_hub/assets/icons/nav/plugins.svg
  - zircon_hub/assets/icons/nav/cloud.svg
  - zircon_hub/assets/icons/nav/team.svg
  - zircon_hub/assets/icons/nav/learn.svg
  - zircon_hub/assets/icons/nav/settings.svg
  - zircon_hub/assets/icons/actions/build-project.svg
  - zircon_hub/assets/icons/actions/install-device.svg
  - zircon_hub/assets/icons/actions/package-project.svg
  - zircon_hub/assets/icons/actions/open-editor.svg
  - zircon_hub/assets/icons/status/running.svg
  - zircon_hub/assets/icons/status/success.svg
  - zircon_hub/assets/icons/status/warning.svg
  - zircon_hub/assets/icons/status/error.svg
  - zircon_hub/assets/icons/ui/search.svg
  - zircon_hub/assets/icons/ui/chevron-down.svg
  - zircon_hub/assets/icons/ui/chevron-left.svg
  - zircon_hub/assets/icons/ui/chevron-right.svg
  - zircon_hub/assets/icons/ui/plus.svg
  - zircon_hub/assets/icons/ui/import.svg
  - zircon_hub/assets/icons/ui/grid.svg
  - zircon_hub/assets/icons/ui/list.svg
  - zircon_hub/assets/icons/ui/folder.svg
  - zircon_hub/assets/icons/ui/sort.svg
  - zircon_hub/assets/icons/ui/bell.svg
  - zircon_hub/assets/icons/ui/help.svg
  - zircon_hub/assets/icons/ui/settings.svg
  - zircon_hub/assets/icons/ui/minimize.svg
  - zircon_hub/assets/icons/ui/maximize.svg
  - zircon_hub/assets/icons/ui/close.svg
  - zircon_hub/assets/icons/ui/more-vertical.svg
  - zircon_hub/assets/icons/ui/refresh.svg
  - zircon_hub/assets/icons/ui/collapse.svg
  - zircon_hub/assets/icons/ui/alert.svg
  - zircon_hub/assets/icons/ui/edit.svg
  - zircon_hub/assets/covers/project-elysium.svg
  - zircon_hub/assets/covers/project-stellar-outpost.svg
  - zircon_hub/assets/covers/project-sands-of-time.svg
  - zircon_hub/assets/covers/project-whispering-woods.svg
  - zircon_hub/assets/covers/project-neon-streets.svg
  - zircon_app/src/entry/entry_runner/editor.rs
  - zircon_editor/src/core/gui_startup_request.rs
  - zircon_editor/src/ui/retained_host/app.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle.rs
  - tools/zircon_build.py
implementation_files:
  - zircon_hub/src/app/runtime.rs
  - zircon_hub/src/app/runtime/asset_catalog.rs
  - zircon_hub/src/app/runtime/folder_picker.rs
  - zircon_hub/src/app/runtime/learn_catalog.rs
  - zircon_hub/src/app/runtime/window_controls.rs
  - zircon_hub/src/app/runtime/team_overview.rs
  - zircon_hub/src/app/binding.rs
  - zircon_hub/src/app/view_model.rs
  - zircon_hub/src/app/view_model/assets.rs
  - zircon_hub/src/app/view_model/cloud.rs
  - zircon_hub/src/app/view_model/learn.rs
  - zircon_hub/src/app/view_model/media.rs
  - zircon_hub/src/app/view_model/plugins.rs
  - zircon_hub/src/app/view_model/team.rs
  - zircon_hub/src/app/localization.rs
  - zircon_hub/src/app/quick_action.rs
  - zircon_hub/src/state/navigation.rs
  - zircon_hub/src/state/project_view.rs
  - zircon_hub/ui/app.slint
  - zircon_hub/ui/shared.slint
  - zircon_hub/ui/components.slint
  - zircon_hub/ui/projects.slint
  - zircon_hub/ui/editor.slint
  - zircon_hub/ui/builds.slint
  - zircon_hub/ui/assets.slint
  - zircon_hub/ui/cloud.slint
  - zircon_hub/ui/learn.slint
  - zircon_hub/ui/plugins.slint
  - zircon_hub/ui/team.slint
  - zircon_hub/ui/placeholder.slint
  - zircon_hub/ui/settings.slint
  - zircon_hub/assets/brand/zircon-mark.svg
  - zircon_hub/assets/icons/nav/projects.svg
  - zircon_hub/assets/icons/nav/editor.svg
  - zircon_hub/assets/icons/nav/assets.svg
  - zircon_hub/assets/icons/nav/builds.svg
  - zircon_hub/assets/icons/nav/plugins.svg
  - zircon_hub/assets/icons/nav/cloud.svg
  - zircon_hub/assets/icons/nav/team.svg
  - zircon_hub/assets/icons/nav/learn.svg
  - zircon_hub/assets/icons/nav/settings.svg
  - zircon_hub/assets/icons/actions/build-project.svg
  - zircon_hub/assets/icons/actions/install-device.svg
  - zircon_hub/assets/icons/actions/package-project.svg
  - zircon_hub/assets/icons/actions/open-editor.svg
  - zircon_hub/assets/icons/status/running.svg
  - zircon_hub/assets/icons/status/success.svg
  - zircon_hub/assets/icons/status/warning.svg
  - zircon_hub/assets/icons/status/error.svg
  - zircon_hub/assets/icons/ui/search.svg
  - zircon_hub/assets/icons/ui/chevron-down.svg
  - zircon_hub/assets/icons/ui/chevron-left.svg
  - zircon_hub/assets/icons/ui/chevron-right.svg
  - zircon_hub/assets/icons/ui/plus.svg
  - zircon_hub/assets/icons/ui/import.svg
  - zircon_hub/assets/icons/ui/grid.svg
  - zircon_hub/assets/icons/ui/list.svg
  - zircon_hub/assets/icons/ui/folder.svg
  - zircon_hub/assets/icons/ui/sort.svg
  - zircon_hub/assets/icons/ui/bell.svg
  - zircon_hub/assets/icons/ui/help.svg
  - zircon_hub/assets/icons/ui/settings.svg
  - zircon_hub/assets/icons/ui/minimize.svg
  - zircon_hub/assets/icons/ui/maximize.svg
  - zircon_hub/assets/icons/ui/close.svg
  - zircon_hub/assets/icons/ui/more-vertical.svg
  - zircon_hub/assets/icons/ui/refresh.svg
  - zircon_hub/assets/icons/ui/collapse.svg
  - zircon_hub/assets/icons/ui/alert.svg
  - zircon_hub/assets/icons/ui/edit.svg
  - zircon_hub/assets/covers/project-elysium.svg
  - zircon_hub/assets/covers/project-stellar-outpost.svg
  - zircon_hub/assets/covers/project-sands-of-time.svg
  - zircon_hub/assets/covers/project-whispering-woods.svg
  - zircon_hub/assets/covers/project-neon-streets.svg
  - zircon_hub/src/projects/editor_recent_sync.rs
  - zircon_hub/src/projects/cover.rs
  - zircon_hub/src/projects/package.rs
  - zircon_hub/src/projects/device_install.rs
  - zircon_hub/src/assets/catalog.rs
  - zircon_hub/src/learn/catalog.rs
  - zircon_hub/src/plugins/catalog.rs
  - zircon_hub/src/team/local_git.rs
  - zircon_hub/src/engines/source_engine_install.rs
  - zircon_hub/src/engines/registry.rs
  - zircon_hub/src/process/editor_launch.rs
  - zircon_hub/src/process/folder_picker.rs
  - zircon_hub/src/build/command.rs
  - zircon_hub/src/settings/hub_config.rs
  - zircon_app/src/entry/entry_runner/editor.rs
  - zircon_editor/src/core/gui_startup_request.rs
  - tools/zircon_build.py
plan_sources:
  - .codex/plans/zircon_hub 独立启动器设计.md
  - .codex/plans/Zircon Hub 典型组件样例设计.md
  - .codex/plans/Zircon Hub Flex 组件化重构设计方案.md
  - user: 2026-05-16 generate Hub and Editor SVG resources from reference screenshots
  - user: 2026-05-17 require Slint 1.16.1 Taffy/Flex layout and Material UI component taxonomy
  - user: 2026-05-17 continue Hub componentized design and match docs/ui-and-layout/hub.png
  - user: 2026-05-18 implement approved non-Projects tab component layout plan
  - https://mui.com/material-ui/all-components/
  - docs/superpowers/specs/2026-05-16-hub-editor-svg-resources-design.md
  - docs/superpowers/plans/2026-05-16-hub-editor-svg-resources.md
tests:
  - cargo fmt -p zircon_hub --check
  - cargo check -p zircon_hub --offline --jobs 1
  - cargo test -p zircon_hub --locked
  - cargo check -p zircon_hub --locked --jobs 1
  - cargo check -p zircon_hub --locked --offline --jobs 1
  - cargo test -p zircon_hub --locked --offline --jobs 1
  - cargo build -p zircon_hub --locked --offline --jobs 1
  - Select-String -Path 'zircon_hub\ui\*.slint' -Pattern '^\s*(x|y):'
  - Select-String -Path 'zircon_hub\ui\*.slint' -Pattern 'text:\s*"(\+|>|<|\[\]|::|==|v|!|\?)"'
  - python tools/zircon_build.py --targets editor,runtime --out <tmp> --mode debug --cargo cargo-nextest --dry-run
  - cargo test -p zircon_app --features target-editor-host --no-default-features --locked editor_gui_startup_parser
  - cargo check -p zircon_app --features target-editor-host --no-default-features --locked
  - python tools/zircon_build.py --targets hub,editor,runtime --out <tmp> --mode debug --dry-run
  - python -m py_compile tools/zircon_build.py
doc_type: category-index
---

# Zircon Hub

`zircon_hub` is the standalone desktop launcher for ZirconEngine. It is a top-level workspace package, but it is not an engine runtime module and does not register with `zircon_runtime` lifecycle services.

The Hub owns a real Slint desktop shell with a UnityHub-style layout: a frameless top-level window, a self-drawn title bar, a full navigation rail, a Projects dashboard, an Editor/source-engine page, an Assets catalog page, a Plugins catalog page, a local Cloud readiness page, a local Team page, a Learn documentation page, a Builds page for the source build pipeline, and Settings. Slint is intentionally confined to this package; editor UI remains the Rust-owned retained host and does not regain Slint business UI paths.

## Ownership

- `zircon_hub/src/app` initializes Slint, binds callbacks, and projects Rust state into the window.
- `zircon_hub/src/app/runtime/asset_catalog.rs` refreshes the runtime asset catalog from recent project roots and source-checkout roots whenever projects or source engines change.
- `zircon_hub/src/app/runtime/folder_picker.rs` owns runtime folder-browse dispatch for project roots, project locations, source checkouts, staged output, and local device install targets.
- `zircon_hub/src/app/runtime/learn_catalog.rs` refreshes the local Learn catalog from source-checkout and repository documentation roots and opens selected resource files through the system file opener.
- `zircon_hub/src/app/runtime/team_overview.rs` refreshes the local Team overview from the active source checkout, current working directory, and compile-time workspace root.
- `zircon_hub/src/app/runtime/window_controls.rs` owns frameless-window behavior: startup geometry restore, normal-window geometry capture, minimize/maximize/close callbacks, title-bar dragging, and double-click maximize persistence.
- `zircon_hub/src/app/view_model.rs` converts runtime snapshots into Slint navigation, card, table, quick-action, source-engine, and header/status data models. Its `view_model/assets.rs`, `view_model/cloud.rs`, `view_model/learn.rs`, `view_model/plugins.rs`, and `view_model/team.rs` children own catalog-specific row projection for assets, local cloud readiness, Learn resources, plugins, and local Git team data.
- `zircon_hub/src/app/view_model/media.rs` owns the Hub static SVG image lookup layer. It resolves bundled navigation, quick-action, status, and fallback project-cover assets under `zircon_hub/assets` and preserves the existing real-project cover lookup before falling back to bundled cover art.
- `zircon_hub/src/app/localization.rs` owns the first Hub UI string table for English and Chinese. The view-model layer uses it for navigation, page titles, quick actions, source-engine status, configuration health, header pills, relative time labels, and the Slint `UiTextData` bundle.
- `zircon_hub/src/app/quick_action.rs` owns the stable quick-action identifiers used by the Slint shell and Rust dispatcher.
- `zircon_hub/src/state` stores the selected page, project filter mode, project sort mode, project view mode, search query, and task status snapshot used by the app layer. `HubPage` now covers Projects, Editor, Assets, Builds, Plugins, Cloud, Team, Learn, and Settings.
- `zircon_hub/src/projects` owns recent-project records, project creation requests, project root validation, project cover discovery, local project package staging, local device-folder installation, and Editor recent sync.
- `zircon_hub/src/assets` owns the local asset catalog scanner. It scans recent project `Assets`/`assets` folders plus engine asset roots under the active source checkout, current directory, and compile-time workspace root.
- `zircon_hub/src/learn` owns the local Learn documentation scanner. It scans `docs/**/*.md`, extracts headings and summaries, skips transient directories, and returns capped catalog rows for the Hub page.
- `zircon_hub/src/plugins` owns local plugin catalog discovery from `zircon_plugins/**/plugin.toml` under the active source checkout, with current-directory and compiled-workspace fallbacks for development runs.
- `zircon_hub/src/team` owns local Git team discovery. It reads repository identity and recent contributors from local `git` commands only; it does not connect accounts, cloud services, or repository hosting providers.
- `zircon_hub/src/engines` owns source checkout records, staged output paths, source-engine validation, recent source-build records, and registry helpers for upsert, active selection, and removal. Hub configuration can now store multiple source-engine records plus an active engine id.
- `zircon_hub/src/build` creates and runs `tools/zircon_build.py --targets editor,runtime` commands for source installs.
- `zircon_hub/src/settings` owns Hub TOML config paths and toolchain/build defaults.
- `zircon_hub/src/process` owns editor launch/open-folder child process commands.
- `zircon_hub/src/process/folder_picker.rs` owns Hub folder selection integration. The current implementation uses the Windows system folder browser through PowerShell/WinForms and returns a clear unavailable status on non-Windows platforms until a cross-platform picker is added.
- `zircon_hub/ui/components.slint` owns reusable Hub UI surfaces and rows such as `HubPanel`, `PanelHeader`, `SearchBox`, `ToolbarSelect`, `DropDownButton`, `InfoRow`, `ActionRow`, `DataTable`, `ProjectTableRow`, `Badge`, `StatusBadge`, `PageScrollSurface`, `CatalogListPanel`, `ResponsivePanelFlow`, `ScrollablePanel`, `TwoColumnSurface`, and `HeaderGroup`. Pages are expected to compose these components instead of reimplementing row, badge, table, button, panel, scroll, and responsive-flow alignment locally.

## Shell Layout And Data Projection

`ui/app.slint` is the shell root. It uses `no-frame: true` and `resize-border-width` to remove the native frame while retaining resize behavior. The title bar is drawn in Slint and exposes callbacks for minimize, maximize/restore, close, and drag. Rust handles those callbacks through Slint's window API. Dragging is implemented by storing the physical window origin at pointer-down and applying logical pointer deltas scaled by the current window scale factor.

The default Hub window is 1600x1024 to match the reference Projects composition: four project cards in the first row, Recent Projects and Quick Actions below them, and the Button States validation strip visible at the bottom of the first viewport. Saved user window state can still override that default after the first run.

Hub chrome uses repository-owned SVG assets instead of placeholder text glyphs for the brand mark, navigation rail, quick-action entries, status pills, toolbar controls, dropdown chevrons, window buttons, and bundled project covers. Static chrome such as the brand mark and Hub-owned UI glyphs are loaded directly from Slint with `@image-url`, while state-projected icons are loaded in Rust by `view_model/media.rs` and passed to Slint as `image` fields with text fallbacks retained only for unavailable asset cases.

Shared icon-bearing controls route their glyphs through a fixed-height centered icon slot before text is laid out. `PillButton`, `IconButton`, `StatusPill`, `WindowButton`, `NavButton`, `SearchBox`, `ToolbarSelect`, `ActionRow`, and the Button States showcase therefore keep SVG marks vertically centered without changing the horizontal distribution of search fields, toolbars, or action rows.

The Projects title band and its primary actions use vertical spacer wrappers rather than page coordinates, so the title/subtitle group and Import/New buttons remain vertically centered inside the 96px shell title area. Navigation rail rows, sidebar footer actions, and status-panel buttons use left-aligned icon/text rows to match the reference reading direction. Dropdown-style controls reserve a fixed trailing chevron slot so popup expand arrows stay right-aligned and vertically centered. Dashboard tables reserve enough width for `Engine Version` and `Last Modified`, while Quick Actions keep a dedicated right-side arrow slot so the arrow buttons form a stable trailing column.

Window placement is now part of Hub configuration. Rust restores the previous physical position, size, and maximized state after loading the Hub config. It captures normal-window geometry on drag end, minimize, maximize/restore, and close, with a minimum size guard so stale or tiny saved values cannot hide the Hub surface. A second title-bar press within the double-click window toggles maximize through the same persisted path. Platform edge snapping remains delegated to the operating system; no custom snap zones are implemented in the Hub yet.

The top title bar status pills are data-driven through `HeaderStatusData`. The first pill reflects the current task state (`Running` or `Ready`). The success, warning, and error pills aggregate the same Rust-side configuration health data used by Settings, plus action failures where applicable, so the shell no longer shows static sample status labels.

The title-bar engine capsule displays the active source engine and opens a Slint `PopupWindow` source-engine selector when clicked. The popup is anchored relative to the dropdown button instead of being positioned from the window root. It is backed by the same `SourceEngineRowData` model as the Editor page and dispatches the same Rust active-engine selection callback, so header switching and Editor-page switching share one persisted active-engine path.

The left rail is model-driven through `NavItemData`. All navigation entries are switchable. Projects, Editor, Assets, Plugins, Cloud, Builds, Team, Learn, and Settings connect to active Hub behavior. The rail can collapse to an icon-only mode using local Slint window state; this is treated as a transient layout preference and is not written into the Rust runtime snapshot or persisted Hub configuration.

`binding.rs` remains a thin projection surface. It applies a `HubSnapshot`, pushes Slint `ModelRc` values built by `view_model.rs`, and reads editable settings fields back from the UI. Formatting of project cards, recent rows, engine summaries, and quick actions lives outside the binding entry file. Search edits are sent back through a dedicated callback so the Rust snapshot can immediately rebuild the card and table models without waiting for another action.

The page bodies use explicit content dimensions under the Hub title area. Projects, Editor, Assets, Plugins, Cloud, Team, Learn, Builds, and Settings keep their own scroll surfaces where the content can exceed the visible Hub viewport, so lower controls remain reachable instead of being clipped by the bottom status bar. Non-Settings business pages now prefer `PageScrollSurface` for page-level overflow, `CatalogListPanel` for catalog rows and empty states, and `ResponsivePanelFlow` for multi-panel work areas that must wrap rather than squeeze on narrow windows.

Hub layout is owned by Slint page composition and reusable components, not by a Rust-side coordinate calculator. `zircon_hub` depends on Slint 1.16.1 or newer and the build script compiles the UI through `i-slint-compiler` with experimental layout support enabled so the Projects dashboard can use Slint's native `FlexboxLayout` and `flex-wrap`. Slint owns drawing, input, model repeaters, flex wrapping, and native `ScrollView` clipping; Rust only projects data models and handles business callbacks.

The Hub component system follows a Material UI-style taxonomy for organization, while the visual language remains the custom Zircon Hub reference screenshot rather than Material Design styling. MUI's public component catalog groups primitives into Inputs, Data display, Feedback, Surfaces, Navigation, Layout, and Utils; Hub maps those categories into Slint components so page files compose named primitives instead of rebuilding local layouts:

- Inputs: `PillButton`, `IconButton`, `SearchBox`, `ToolbarSelect`, `DropDownButton`, segmented controls, and future text-field wrappers.
- Data display: `Badge`, `StatusBadge`, `InfoRow`, `ActionRow`, `DataTable`, `ProjectTableRow`, project cards, and typography wrappers.
- Feedback: header status pills, configuration-health rows, disabled-state rows, and future alert/snackbar/dialog surfaces.
- Surfaces: `HubPanel`, `PanelHeader`, `ScrollablePanel`, project cards, Recent Projects, Quick Actions, and settings panels.
- Navigation: `HeaderGroup`, title-bar engine selector, `NavButton`, nav rail collapse control, tabs/view-mode controls, and popup/menu affordances.
- Layout: `PageScrollSurface`, `ResponsivePanelFlow`, `TwoColumnSurface`, page scroll shells, toolbar rows, Flexbox card grids, catalog list shells, and stack-style horizontal/vertical groups.
- Utils: SVG icon resources, popup anchoring, hover/pressed state wrappers, and local fallback behavior for missing image assets.

Slint 1.16.1's Taffy-backed `FlexboxLayout` is the required basis for wrapping dashboard regions such as project cards and lower two-column work surfaces. Business pages should prefer `VerticalLayout`, `HorizontalLayout`, `FlexboxLayout`, and panel-owned `ScrollView` composition. Pixel values are allowed for design tokens such as button height, icon size, spacing, border radius, and card minimum width; they should not be used as page-level coordinate math. `PopupWindow` may still use local `x`/`y` offsets relative to its owning dropdown button because Slint requires an anchor position for popup placement.

## Projects Dashboard

The Projects page now uses real recent-project data in two projections:

- Project cards show the first four filtered recent projects with a real project cover when the project directory contains `.zircon/cover.*`, `.zircon/thumbnail.*`, root-level cover/thumbnail/project images, or matching `Assets`/`assets` images. If no supported image exists or Slint cannot load it, the card falls back to one of the bundled SVG cover thumbnails under `zircon_hub/assets/covers`; the Slint-drawn accent background remains the last-resort visual if a bundled asset cannot be loaded.
- The Recent Projects table shows up to eight filtered projects with cover thumbnail, name, engine version placeholder, last-opened label, path, and a row action affordance.

The Projects shell title area owns the primary Import Project and New Project actions, matching the reference screenshot's title-and-actions composition. The page toolbar below it carries search, project filter, sort selector, and grid/list view buttons on a single aligned row. Search uses the shared `SearchBox` component with a bundled SVG icon and stable focus styling instead of a text placeholder glyph. Search, project filtering, sort, and grid/list view mode update the dashboard immediately. Import Project and New Project open an inline action panel instead of keeping manual entry fields permanently visible. The import panel accepts a project root path and can open the system folder picker before calling the existing open-project flow; the create panel accepts project name and location and can browse for the target parent directory before calling the existing create-project flow.

The Projects dashboard uses Slint `FlexboxLayout` for the upper project-card flow. Each card is a vertical cover/title/path/time/tag surface with a 296px flex basis, a 260px minimum width, and `flex-grow` enabled, so wide windows land on the reference-style four-card row while narrower windows naturally wrap down to fewer columns. The card area starts as one visible row and can expand to additional rows through the Show More control. Lower Recent Projects and Quick Actions panels sit in a wrapping flex surface; the recent list is a real `DataTable` with proportional columns, and Quick Actions use `ActionRow` so the right-side arrow slot remains fixed.

The Recent Projects panel keeps the `View All Projects` action in the header's right action slot for the current Hub layout pass. The action uses the shared panel-header action button with a right-aligned chevron, leaving the table body free to use the remaining vertical space and keeping the bottom status bar independent from table actions. Activating it now resets the Projects dashboard to the all-projects grid, clears search text, closes inline import/create entry panels, and expands the card flow so the action behaves as navigation over the project set instead of opening the import workflow.

The Projects page also carries a compact Button States showcase at the bottom of the scrollable page body. It is a development validation surface for the shared button/icon components, matching the reference screenshot's component-verification band without adding a separate user-facing workflow.

Project filtering and sorting are part of the Rust snapshot rather than local Slint-only state. The filter selector cycles through all projects, existing paths, and missing paths. The sort selector cycles between newest-first and name sorting. The card, compact-list, and table projections are rebuilt from the same filtered recent-project source. Grid mode shows a capped flow of recent-project cards; list mode swaps the upper card region for a compact four-row list while keeping the lower recent-project table available.

Clicking a project card or recent row now selects that project as the dashboard target and projects a selected state back into the card, compact-list row, and Recent Projects table row. Explicit row/card action buttons and the manual open flow still launch `zircon_editor --project <path>` through the same validation and recent-sync path. Empty states are rendered inside the dashboard instead of falling back to plain text summaries.

Quick Actions are identified by stable ids. `build-project` runs the configured source build path. `open-editor` opens the selected recent project when one is selected and launches the staged editor without a project only when no project target is selected. `package-project` and `install-device` use the selected project first, falling back to the newest recent project when the dashboard has no explicit selection, and their status details include the project name plus output path. The package action copies the project into `packages/<project>-<timestamp>/project`, skips heavy transient folders such as `.git` and `target`, and writes `zircon-package.toml` metadata beside the staged project. `install-device` now uses the same package step and copies that package into the configured local device install folder. This is a real local deployment path, but it is still a folder-backed device target rather than a USB, ADB, console SDK, or remote runtime deployer.

## Builds Page

The Builds page is the Hub-level source build dashboard. It uses the same `SourceEngineData` projection as the Editor page and exposes the active build controls in a build-specific surface: run the configured editor/runtime source build, open the staged output folder, or launch the staged editor. The page also shows a compact build pipeline with source validation, editor compilation, runtime staging, and a disabled package slot reserved for future export work.

The Builds page now composes the shared Hub design-system components instead of local card and row implementations. Source/output summaries use `InfoRow`, build controls use `ActionRow`, pipeline status uses right-aligned badges, and all visible action marks come from bundled SVG assets rather than text glyphs. Its overview/control and pipeline/status regions sit inside `ResponsivePanelFlow` groups under a `PageScrollSurface`, so narrow windows stack the panels vertically while preserving the same internal action and row components.

## Assets Page

The Assets page is backed by a lightweight local catalog. On Hub startup, after project open/create, and whenever the active source checkout changes, Rust refreshes `HubSnapshot.assets` from recent project roots and engine roots. Project roots scan `Assets` and `assets`; engine roots scan `zircon_editor/assets` and `zircon_runtime/assets` from the active source checkout, current directory, and compile-time workspace root.

The scanner classifies common extensions into image, model, audio, shader, data, scene, UI, or file kinds, skips transient `.git` and `target` directories, sorts entries by source/kind/name/path, and caps the list to keep the Hub responsive. Slint receives compact `AssetData` rows with name, kind, source, size, path, and accent index. This page is a catalog browser only; import pipelines, thumbnails, dependency graphs, and asset editing remain owned by the Editor/runtime asset systems.

The Assets page now uses `PageScrollSurface` plus `CatalogListPanel` around shared `InfoRow` rows. Asset rows use the bundled Assets SVG as the leading mark and put the asset kind in the standardized right-side badge slot, so the page follows the same Data display and Surfaces component categories as Projects, Team, and Cloud without carrying its own hand-built title, empty-state, or content-height formula.

## Plugins Page

The Plugins page is backed by real local source data. On Hub startup and whenever the active source checkout changes, Rust scans for `zircon_plugins/**/plugin.toml`, parses each manifest, sorts entries by plugin id, and projects display name, category, maturity, default packaging modes, module count, description, and package path into `PluginData` rows. If the configured source checkout is missing or does not contain a plugin tree, discovery falls back to the current working directory and the compile-time workspace root so development runs from this repository still show the local catalog.

This page is a catalog browser, not yet a plugin build or enable/disable manager. Plugin build execution remains in `tools/zircon_build.py --targets plugins`, and runtime load-manifest generation remains owned by that build script.

The Plugins page now uses `PageScrollSurface` plus `CatalogListPanel` instead of a locally hand-built catalog shell. Each plugin row uses the bundled Plugins SVG leading mark, projects category into the shared badge slot, and leaves plugin packaging/module/maturity details in the row meta line.

## Cloud Page

The Cloud page is a local readiness surface, not an account or network integration. Rust projects `CloudSummaryData` from existing Hub settings: account status is always local/offline, build output and device install rows use the configured directories, and package status is derived from `<build-output>/packages` by counting local package directories created by the existing package action.

The service list is intentionally a set of reserved slots. It shows account sync, remote build, and package upload as non-connected local/offline capabilities so the navigation entry has a real operational surface without implying that hosted services, authentication, licenses, uploads, or remote build workers already exist. Future cloud work can replace these rows with real service health once those backends exist.

The Cloud page uses `PageScrollSurface` for page overflow and `ResponsivePanelFlow` for the four readiness metrics. The service list keeps its own panel-owned `ScrollView`, so reserved service rows can grow independently from the page while the metrics wrap into more rows on narrow windows.

## Learn Page

The Learn page is backed by local repository documentation instead of static placeholder cards. On Hub startup and whenever the active source checkout changes, Rust scans `docs/**/*.md` from the configured source checkout, current working directory, and compile-time workspace root. Discovery deduplicates documentation roots, skips transient `.git` and `target` folders, sorts by category/title/path, and caps the result set to keep the Hub responsive.

Each Learn row extracts the first markdown H1 as the title, the first readable paragraph as the summary, the first folder below `docs` as the category, and the source path as the open target. The page is a local documentation browser only; it does not yet manage online tutorials, sample downloads, or remote learning feeds.

The Learn page now uses `PageScrollSurface` plus `CatalogListPanel` as a clickable documentation list. The category is pinned in the badge slot, the right arrow uses the shared SVG action affordance, and clicking the row dispatches the existing local-resource open callback.

## Team Page

The Team page is backed by local Git data instead of an account service. On Hub startup and whenever source-engine settings change, Rust looks for a Git workspace from the active source checkout, current working directory, and compile-time workspace root. The first valid repository root becomes the Team overview source.

Discovery reads `git config --get user.name`, `git config --get user.email`, and recent author pairs from `git log --all --format=%an%x1f%ae -n 200`. Authors are counted by name/email, sorted by commit count and stable identity text, capped for the Hub surface, and projected as `TeamMemberData` rows. Missing Git, missing config values, or a non-repository workspace produce a local empty state rather than failing Hub startup.

This page is intentionally local-only. It does not manage repository permissions, invite members, connect hosting providers, sync cloud teams, or display remote review state. Those behaviors belong to a future collaboration service or Cloud page instead of the current source-local Hub shell.

The Team page uses `PageScrollSurface` and `ResponsivePanelFlow` for its local identity and repository summary cards, then keeps member rows in a panel-owned `ScrollView`. This gives the summary section the same wrapping behavior as Cloud while keeping long contributor lists from pushing the bottom status bar.

## Config And Recent Sync

Hub config is TOML under the user config directory, for example `%LOCALAPPDATA%\ZirconHub\config.toml` on Windows. It stores Hub settings, recent projects, source-engine records, the active source-engine id, and the last known Hub window placement. Editor recent-project sync reads and writes the existing JSON config shape at `editor.startup.session`, preserving unrelated keys in `%LOCALAPPDATA%\ZirconEngine\config.json` or the path selected by `ZIRCON_CONFIG_PATH`.

Recent entries merge by normalized path, keep the entry with the newest `last_opened_unix_ms`, sort newest first, and truncate to eight entries. Hub startup writes the merged recent list back to Hub TOML and the Editor JSON session while preserving the Editor `last_project_path`; Hub only overrides `last_project_path` after an explicit open or create request successfully spawns `zircon_editor` for that project.

The Settings page edits the stored toolchain paths, default project/source/output directories, default local device install directory, build profile, build job count, and language preference. Toolchain and path values remain text fields because they can point at arbitrary local commands or directories. Build profile and language use Hub-styled segmented controls backed by the same stored values: `debug` or `release` for build profile and `English` or `Chinese` for language. Hub uses the configured Python executable as the build-script runner and forwards the configured Cargo executable through `tools/zircon_build.py --cargo <cargo-path>`, so source installs can use a non-default Cargo shim without editing the environment.

Settings also projects a Configuration Health panel from Rust view-model data. Bare command names such as `python`, `cargo`, or `rustup` are marked as environment-resolved commands, path-like executable values are checked for existence, and default project/source/output/device directories are classified as ready or create-on-use paths, with source checkout still reported as missing when the checkout is absent. This is only local configuration readability for now; it does not run toolchain subprocess probes during normal Hub rendering.

Default project, source checkout, staged output, and local device install directories now expose Browse buttons in Settings. The Editor/source-engine page also exposes Browse buttons for source checkout and staged output. Selecting a folder updates the in-memory Hub settings and the visible source-engine projection; settings are still persisted through the normal Save Settings action.

The language setting now drives a real English/Chinese UI text bundle instead of only being stored in config. Rust projects localized navigation labels, page titles/subtitles, project filter/sort labels, quick actions, source-engine status, build-history status, relative time labels, Settings health rows, and header status pills. Slint receives the remaining static surface text through `UiTextData`, which currently covers the shell, local user label, Projects, Editor, Assets, Plugins, Cloud, Team, Learn, Builds, and Settings. This is still a compact two-language string table rather than a full resource-file i18n system; future work can move the same keys into external resource catalogs if the language set grows.

The Editor page replaces the old Installs page. It shows the active source engine record, source checkout path, staged output directory, last build label, build profile, build jobs, and actions for saving source settings, building, opening output, and launching the editor. Hub startup registers a source-engine record from saved source/output settings when those settings exist, so returning users see their source install immediately even before pressing Save again.

The Editor page now follows the same component-first layout contract as Projects and Builds. The active-engine overview uses `HubPanel`, `PanelHeader`, `InfoRow`, and `Badge`; action rows use the shared `ActionRow`; source-engine and build-history lists own their internal `ScrollView` regions so long lists scroll inside their panels instead of pushing the status bar or adjacent form region. Its overview/actions and settings/list regions are grouped with `ResponsivePanelFlow` under `PageScrollSurface`, so narrow windows stack the same panels instead of compressing the source path fields and build-history surfaces.

The Editor page also renders the registered source-engine list from `SourceEngineRowData`. Saving a source checkout now appends or updates a stable source-engine record derived from the source directory instead of replacing the entire engine list. Existing display names and build history are preserved when the same source checkout is saved again, so saving or building does not discard user-managed engine metadata. Selecting an engine marks it active, updates the visible source/output settings, persists the active engine id, and makes subsequent build/launch paths use that engine's staged output.

The active engine can be renamed from the Editor page, and any registered engine can be removed from the list; removing the active engine automatically falls back to the next available engine. Each source engine also stores a compact build history through `SourceBuildRecord`. Successful builds update `last_build_unix_ms`; successful and failed build attempts are both inserted into the per-engine history and truncated to the newest eight records. The Editor page shows the newest active-engine build-history rows with status, profile/jobs, output path, relative finish time, and detail. Version tagging beyond the package version label and expanded build-log inspection remain reserved for later engine-management work.

## Editor Launch Contract

Hub launches `zircon_editor` as an independent child process. Existing projects use `--project <path>`. New projects use `--create-project --project-name <name> --location <dir> --template renderable-empty`.

Before opening or creating a project, Hub checks whether a preferred editor executable is available. A sibling staged `zircon_editor(.exe)` beside the running Hub takes priority; otherwise Hub uses the configured staged output path. If neither exists, Hub runs the source-install build command first so project launch can use the freshly staged editor/runtime payload.

`zircon_app` parses these GUI startup arguments before the headless operation parser. When one is present, `zircon_editor` receives an `EditorGuiStartupRequest` and opens or creates that project directly through `EditorManager`; it does not call the normal last-project restore path. Empty editor args still use the existing fallback behavior.

## Staged Builds

`tools/zircon_build.py` now accepts the `hub` target. A staged payload can include `zircon_hub.exe`, `zircon_editor.exe`, and the runtime library under one `ZirconEngine` directory. The Hub's own build action still calls the tool with `--targets editor,runtime` because Hub source installs need a staged editor/runtime payload to launch.
