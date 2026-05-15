---
related_code:
  - Cargo.toml
  - zircon_hub/Cargo.toml
  - zircon_hub/build.rs
  - zircon_hub/src/lib.rs
  - zircon_hub/src/main.rs
  - zircon_hub/src/app/mod.rs
  - zircon_hub/src/app/runtime.rs
  - zircon_hub/src/app/binding.rs
  - zircon_hub/src/app/view_model.rs
  - zircon_hub/src/app/quick_action.rs
  - zircon_hub/src/state/mod.rs
  - zircon_hub/src/state/navigation.rs
  - zircon_hub/src/state/project_view.rs
  - zircon_hub/src/projects/mod.rs
  - zircon_hub/src/engines/mod.rs
  - zircon_hub/src/build/mod.rs
  - zircon_hub/src/settings/mod.rs
  - zircon_hub/src/process/mod.rs
  - zircon_hub/ui/app.slint
  - zircon_hub/ui/shared.slint
  - zircon_hub/ui/projects.slint
  - zircon_hub/ui/editor.slint
  - zircon_hub/ui/builds.slint
  - zircon_hub/ui/placeholder.slint
  - zircon_hub/ui/settings.slint
  - zircon_app/src/entry/entry_runner/editor.rs
  - zircon_editor/src/core/gui_startup_request.rs
  - zircon_editor/src/ui/retained_host/app.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle.rs
  - tools/zircon_build.py
implementation_files:
  - zircon_hub/src/app/runtime.rs
  - zircon_hub/src/app/binding.rs
  - zircon_hub/src/app/view_model.rs
  - zircon_hub/src/app/quick_action.rs
  - zircon_hub/src/state/navigation.rs
  - zircon_hub/src/state/project_view.rs
  - zircon_hub/ui/app.slint
  - zircon_hub/ui/shared.slint
  - zircon_hub/ui/projects.slint
  - zircon_hub/ui/editor.slint
  - zircon_hub/ui/builds.slint
  - zircon_hub/ui/placeholder.slint
  - zircon_hub/ui/settings.slint
  - zircon_hub/src/projects/editor_recent_sync.rs
  - zircon_hub/src/process/editor_launch.rs
  - zircon_hub/src/build/command.rs
  - zircon_hub/src/settings/hub_config.rs
  - zircon_app/src/entry/entry_runner/editor.rs
  - zircon_editor/src/core/gui_startup_request.rs
  - tools/zircon_build.py
plan_sources:
  - .codex/plans/zircon_hub 独立启动器设计.md
tests:
  - cargo fmt -p zircon_hub --check
  - cargo test -p zircon_hub --locked
  - cargo check -p zircon_hub --locked --jobs 1
  - cargo test -p zircon_app --features target-editor-host --no-default-features --locked editor_gui_startup_parser
  - cargo check -p zircon_app --features target-editor-host --no-default-features --locked
  - python tools/zircon_build.py --targets hub,editor,runtime --out <tmp> --mode debug --dry-run
  - python -m py_compile tools/zircon_build.py
doc_type: category-index
---

# Zircon Hub

`zircon_hub` is the standalone desktop launcher for ZirconEngine. It is a top-level workspace package, but it is not an engine runtime module and does not register with `zircon_runtime` lifecycle services.

The Hub owns a real Slint desktop shell with a UnityHub-style layout: a frameless top-level window, a self-drawn title bar, a full navigation rail, a Projects dashboard, an Editor/source-engine page, a Builds page for the source build pipeline, placeholder sections for future Hub areas, and Settings. Slint is intentionally confined to this package; editor UI remains the Rust-owned retained host and does not regain Slint business UI paths.

## Ownership

- `zircon_hub/src/app` initializes Slint, binds callbacks, handles window chrome actions, and projects Rust state into the window.
- `zircon_hub/src/app/view_model.rs` converts runtime snapshots into Slint navigation, card, table, quick-action, and source-engine data models.
- `zircon_hub/src/app/quick_action.rs` owns the stable quick-action identifiers used by the Slint shell and Rust dispatcher.
- `zircon_hub/src/state` stores the selected page, project sort mode, project view mode, search query, and task status snapshot used by the app layer. `HubPage` now covers Projects, Editor, Assets, Builds, Plugins, Cloud, Team, Learn, and Settings.
- `zircon_hub/src/projects` owns recent-project records, project creation requests, project root validation, and Editor recent sync.
- `zircon_hub/src/engines` owns source checkout records, staged output paths, and source-engine validation.
- `zircon_hub/src/build` creates and runs `tools/zircon_build.py --targets editor,runtime` commands for source installs.
- `zircon_hub/src/settings` owns Hub TOML config paths and toolchain/build defaults.
- `zircon_hub/src/process` owns editor launch/open-folder child process commands.

## Shell Layout And Data Projection

`ui/app.slint` is the shell root. It uses `no-frame: true` and `resize-border-width` to remove the native frame while retaining resize behavior. The title bar is drawn in Slint and exposes callbacks for minimize, maximize/restore, close, and drag. Rust handles those callbacks through Slint's window API. Dragging is implemented by storing the physical window origin at pointer-down and applying logical pointer deltas scaled by the current window scale factor.

The left rail is model-driven through `NavItemData`. All navigation entries are switchable. Projects, Editor, Builds, and Settings connect to active Hub behavior; Assets, Plugins, Cloud, Team, and Learn are reserved placeholder pages with the same shell styling so the information architecture can grow without another shell rewrite.

`binding.rs` remains a thin projection surface. It applies a `HubSnapshot`, pushes Slint `ModelRc` values built by `view_model.rs`, and reads editable settings fields back from the UI. Formatting of project cards, recent rows, engine summaries, and quick actions lives outside the binding entry file. Search edits are sent back through a dedicated callback so the Rust snapshot can immediately rebuild the card and table models without waiting for another action.

The page bodies use explicit content dimensions under the Hub title area. Projects, Editor, Builds, and Settings keep their own scroll surfaces where the content can exceed the visible Hub viewport, so lower controls remain reachable instead of being clipped by the bottom status bar.

## Projects Dashboard

The Projects page now uses real recent-project data in two projections:

- Project cards show the first four filtered recent projects with a Slint-drawn placeholder cover, compact engine version label, platform label, path, and relative modified time.
- The Recent Projects table shows up to eight filtered projects with name, engine version placeholder, last-opened label, path, and a row action affordance.

The top Projects toolbar mirrors the UnityHub-style control row: search, All Projects filter, sort selector, grid/list view buttons, Import Project, and New Project. Search, sort, grid/list view mode, import, and new-project are wired to the existing runtime behavior. The current All Projects filter remains a reserved visual affordance until project categories or templates are stored in the Hub model.

Project sorting is part of the Rust snapshot rather than local Slint-only state. The sort selector currently cycles between newest-first and name sorting, and both the card projection and table/list projection are rebuilt from the same filtered recent-project source. Grid mode shows the four-card dashboard; list mode swaps the upper card region for a compact row list while keeping the lower recent-project table available.

Clicking a project card or recent row launches `zircon_editor --project <path>` through the same validation and recent-sync path as manual project opening. Empty states are rendered inside the dashboard instead of falling back to plain text summaries.

Quick Actions are identified by stable ids. `build-project` runs the configured source build path, `open-editor` launches the staged editor without a project, and the device/package actions report an unavailable status until those subsystems exist.

## Builds Page

The Builds page is the Hub-level source build dashboard. It uses the same `SourceEngineData` projection as the Editor page and exposes the active build controls in a build-specific surface: run the configured editor/runtime source build, open the staged output folder, or launch the staged editor. The page also shows a compact build pipeline with source validation, editor compilation, runtime staging, and a disabled package slot reserved for future export work.

## Config And Recent Sync

Hub config is TOML under the user config directory, for example `%LOCALAPPDATA%\ZirconHub\config.toml` on Windows. Editor recent-project sync reads and writes the existing JSON config shape at `editor.startup.session`, preserving unrelated keys in `%LOCALAPPDATA%\ZirconEngine\config.json` or the path selected by `ZIRCON_CONFIG_PATH`.

Recent entries merge by normalized path, keep the entry with the newest `last_opened_unix_ms`, sort newest first, and truncate to eight entries. Hub startup preserves the Editor `last_project_path` while merging recents; Hub only overrides it after an explicit open or create request successfully spawns `zircon_editor` for that project.

The Settings page edits the stored toolchain paths, default project/source/output directories, build profile, build job count, and language preference. The current UI accepts `debug` or `release` for build profile and `English` or `Chinese` for language.

The Editor page replaces the old Installs page. It shows the local source engine record, source checkout path, staged output directory, last build label, build profile, build jobs, and actions for saving source settings, building, opening output, and launching the editor. Hub startup registers a source-engine record from saved source/output settings when those settings exist, so returning users see their source install immediately even before pressing Save again.

## Editor Launch Contract

Hub launches `zircon_editor` as an independent child process. Existing projects use `--project <path>`. New projects use `--create-project --project-name <name> --location <dir> --template renderable-empty`.

Before opening or creating a project, Hub checks whether a preferred editor executable is available. A sibling staged `zircon_editor(.exe)` beside the running Hub takes priority; otherwise Hub uses the configured staged output path. If neither exists, Hub runs the source-install build command first so project launch can use the freshly staged editor/runtime payload.

`zircon_app` parses these GUI startup arguments before the headless operation parser. When one is present, `zircon_editor` receives an `EditorGuiStartupRequest` and opens or creates that project directly through `EditorManager`; it does not call the normal last-project restore path. Empty editor args still use the existing fallback behavior.

## Staged Builds

`tools/zircon_build.py` now accepts the `hub` target. A staged payload can include `zircon_hub.exe`, `zircon_editor.exe`, and the runtime library under one `ZirconEngine` directory. The Hub's own build action still calls the tool with `--targets editor,runtime` because Hub source installs need a staged editor/runtime payload to launch.
