---
related_code:
  - zircon_hub/src/state/scope.rs
  - zircon_hub/src/state/hub_snapshot.rs
  - zircon_hub/src/state/navigation.rs
  - zircon_hub/src/state/project_view.rs
  - zircon_hub/src/state/action_history.rs
  - zircon_hub/src/state/task_status.rs
  - zircon_hub/src/projects/metadata.rs
  - zircon_hub/src/app/runtime.rs
  - zircon_hub/src/app/runtime/source_scoped_views.rs
  - zircon_hub/src/app/runtime/asset_catalog.rs
  - zircon_hub/src/app/runtime/plugin_catalog.rs
  - zircon_hub/src/app/runtime/learn_catalog.rs
  - zircon_hub/src/app/runtime/team_overview.rs
  - zircon_hub/src/app/runtime/project_workspace.rs
  - zircon_hub/src/app/runtime/persistence.rs
  - zircon_hub/src/settings/hub_config.rs
  - zircon_hub/src/settings/mod.rs
  - zircon_hub/src/settings/config_path.rs
  - zircon_hub/src/settings/paths.rs
  - zircon_hub/src/assets/catalog.rs
  - zircon_hub/src/plugins/catalog.rs
  - zircon_hub/src/learn/catalog.rs
  - zircon_hub/src/team/local_git.rs
  - zircon_hub/src/app/binding.rs
  - zircon_hub/src/app/view_model.rs
  - zircon_hub/src/app/view_model/projects.rs
  - zircon_hub/src/app/view_model/quick_actions.rs
  - zircon_hub/src/app/view_model/workspace_actions.rs
  - zircon_hub/src/app/view_model/assets.rs
  - zircon_hub/src/app/view_model/plugins.rs
  - zircon_hub/src/app/view_model/learn.rs
  - zircon_hub/src/app/view_model/team.rs
  - zircon_hub/src/app/view_model/cloud.rs
  - zircon_hub/src/process/editor_launch.rs
  - zircon_hub/src/process/open_folder.rs
  - zircon_hub/ui/shared.slint
  - zircon_hub/ui/app.slint
  - zircon_hub/ui/builds.slint
  - zircon_hub/ui/builds_page_components.slint
  - zircon_hub/ui/editor.slint
implementation_files:
  - zircon_hub/src/state/scope.rs
  - zircon_hub/src/state/hub_snapshot.rs
  - zircon_hub/src/state/navigation.rs
  - zircon_hub/src/state/project_view.rs
  - zircon_hub/src/state/action_history.rs
  - zircon_hub/src/state/task_status.rs
  - zircon_hub/src/app/runtime.rs
  - zircon_hub/src/app/runtime/source_scoped_views.rs
  - zircon_hub/src/app/runtime/asset_catalog.rs
  - zircon_hub/src/app/runtime/plugin_catalog.rs
  - zircon_hub/src/app/runtime/learn_catalog.rs
  - zircon_hub/src/app/runtime/team_overview.rs
  - zircon_hub/src/app/runtime/project_workspace.rs
  - zircon_hub/src/app/runtime/persistence.rs
  - zircon_hub/src/settings/hub_config.rs
  - zircon_hub/src/settings/mod.rs
  - zircon_hub/src/assets/catalog.rs
  - zircon_hub/src/plugins/catalog.rs
  - zircon_hub/src/learn/catalog.rs
  - zircon_hub/src/team/local_git.rs
  - zircon_hub/src/app/binding.rs
  - zircon_hub/src/app/view_model.rs
  - zircon_hub/src/app/view_model/projects.rs
  - zircon_hub/src/app/view_model/quick_actions.rs
  - zircon_hub/src/app/view_model/workspace_actions.rs
  - zircon_hub/src/process/editor_launch.rs
  - zircon_hub/src/process/open_folder.rs
plan_sources:
  - user: 2026-05-28 优化hub[image Zircon Hub 响应式组件化重构计划.md]
  - .opencode/workflows/20260528_231820_026_优化hub[image Zircon Hub 响应式组件化重构计划.md]/workflow.xml
  - .opencode/workflows/20260528_231820_026_优化hub[image Zircon Hub 响应式组件化重构计划.md]/hub-runtime-state-integration/decomposition.md
  - .opencode/workflows/20260528_231820_026_优化hub[image Zircon Hub 响应式组件化重构计划.md]/hub-runtime-state-integration-scope-model/plan.md
  - .opencode/workflows/20260528_231820_026_优化hub[image Zircon Hub 响应式组件化重构计划.md]/hub-runtime-state-integration-scope-model/decomposition.md
  - .opencode/workflows/20260528_231820_026_优化hub[image Zircon Hub 响应式组件化重构计划.md]/hub-runtime-state-integration-scope-model/review-surface.md
  - .opencode/workflows/20260528_231820_026_优化hub[image Zircon Hub 响应式组件化重构计划.md]/hub-runtime-state-integration-command-actions/plan.md
  - .opencode/workflows/20260528_231820_026_优化hub[image Zircon Hub 响应式组件化重构计划.md]/hub-runtime-state-integration-command-actions/review-surface.md
  - .opencode/workflows/20260528_231820_026_优化hub[image Zircon Hub 响应式组件化重构计划.md]/hub-runtime-state-integration-scoped-snapshots/plan.md
  - .opencode/workflows/20260528_231820_026_优化hub[image Zircon Hub 响应式组件化重构计划.md]/hub-runtime-state-integration-scoped-snapshots/review-surface.md
  - .opencode/workflows/20260528_231820_026_优化hub[image Zircon Hub 响应式组件化重构计划.md]/hub-runtime-state-integration-restart-persistence/plan.md
  - .opencode/workflows/20260528_231820_026_优化hub[image Zircon Hub 响应式组件化重构计划.md]/hub-runtime-state-integration-restart-persistence/review-surface.md
  - .opencode/workflows/20260528_231820_026_优化hub[image Zircon Hub 响应式组件化重构计划.md]/hub-runtime-state-integration-contract-docs/plan.md
  - .opencode/workflows/20260528_231820_026_优化hub[image Zircon Hub 响应式组件化重构计划.md]/hub-runtime-state-integration-contract-docs/review-surface.md
  - .opencode/workflows/20260528_231820_026_优化hub[image Zircon Hub 响应式组件化重构计划.md]/hub-docs-contract-refresh/plan.md
  - .opencode/workflows/20260528_231820_026_优化hub[image Zircon Hub 响应式组件化重构计划.md]/hub-docs-contract-refresh/review-surface.md
  - .opencode/workflows/20260528_190023_866_继续完善hub/hub-foundations-contracts-docs/plan.md
  - .opencode/workflows/20260528_190023_866_继续完善hub/hub-foundations-contracts-docs/decomposition.md
tests:
  - zircon_hub/src/state/scope.rs
  - zircon_hub/src/state/hub_snapshot.rs
  - zircon_hub/src/state/navigation.rs
  - zircon_hub/src/state/project_view.rs
  - zircon_hub/src/state/action_history.rs
  - zircon_hub/src/state/task_status.rs
  - zircon_hub/src/app/runtime.rs
  - zircon_hub/src/settings/hub_config.rs
  - zircon_hub/src/app/runtime/source_scoped_views.rs
  - zircon_hub/src/assets/catalog.rs
  - zircon_hub/src/plugins/catalog.rs
  - zircon_hub/src/learn/catalog.rs
  - zircon_hub/src/app/view_model.rs
  - zircon_hub/src/app/view_model/quick_actions.rs
  - zircon_hub/tests/project_management_contract.rs
  - zircon_hub/tests/project_quick_actions_contract.rs
  - zircon_hub/tests/project_source_engine_contract.rs
  - zircon_hub/tests/ui_selected_project_runtime_contract.rs
  - zircon_hub/tests/ui_project_scope_contract.rs
  - zircon_hub/tests/ui_project_navigation_contract.rs
  - zircon_hub/tests/project_workflow_contract.rs
  - zircon_hub/tests/project_path_scope_contract.rs
  - zircon_hub/tests/ui_selected_project_catalog_contract.rs
  - cargo test -p zircon_hub --test project_management_contract --locked -- --nocapture
  - cargo test -p zircon_hub --test project_quick_actions_contract --locked -- --nocapture
  - cargo test -p zircon_hub --test project_source_engine_contract --locked -- --nocapture
  - cargo test -p zircon_hub --test ui_selected_project_runtime_contract --locked -- --nocapture
  - cargo test -p zircon_hub --test ui_project_scope_contract --locked -- --nocapture
  - cargo test -p zircon_hub state::scope --locked -- --nocapture
  - cargo test -p zircon_hub state::hub_snapshot --locked -- --nocapture
  - cargo test -p zircon_hub state::action_history --locked -- --nocapture
  - cargo test -p zircon_hub state::task_status --locked -- --nocapture
  - cargo test -p zircon_hub settings::hub_config --locked -- --nocapture
  - cargo test -p zircon_hub app::runtime --locked -- --nocapture
  - cargo test -p zircon_hub app::runtime::source_scoped_views --locked -- --nocapture
  - cargo test -p zircon_hub assets::catalog --locked -- --nocapture
  - cargo test -p zircon_hub plugins::catalog --locked -- --nocapture
  - cargo test -p zircon_hub learn::catalog --locked -- --nocapture
  - cargo test -p zircon_hub app::view_model --locked -- --nocapture
  - cargo test -p zircon_hub app::view_model::quick_actions --locked -- --nocapture
  - cargo test -p zircon_hub --test project_workflow_contract --locked -- --nocapture
  - cargo test -p zircon_hub --test project_path_scope_contract --locked -- --nocapture
  - cargo test -p zircon_hub --test ui_selected_project_catalog_contract --locked -- --nocapture
  - cargo check -p zircon_hub --locked
  - cargo fmt -p zircon_hub --check
  - cargo check -p zircon_hub --locked --jobs 1
  - cargo test -p zircon_hub --test ui_selected_project_runtime_contract --locked --jobs 1 -- --nocapture
  - cargo test -p zircon_hub --test ui_selected_project_catalog_contract --locked --jobs 1 -- --nocapture
  - cargo test -p zircon_hub --test project_quick_actions_contract --locked --jobs 1 -- --nocapture
  - cargo test -p zircon_hub --test project_workflow_contract --locked --jobs 1 -- --nocapture
  - cargo test -p zircon_hub --test ui_project_navigation_contract --locked --jobs 1 -- --nocapture
  - cargo test -p zircon_hub --test project_source_engine_contract --locked --jobs 1 -- --nocapture
  - cargo test -p zircon_hub --test project_path_scope_contract --locked --jobs 1 -- --nocapture
  - cargo test -p zircon_hub --locked --jobs 1
  - zircon_hub/tests/hub_docs_contract.rs
  - cargo test -p zircon_hub --test hub_docs_contract --locked --jobs 1 -- --nocapture
doc_type: module-detail
---

# Hub foundations state model

Hub foundations are the shared state contracts that keep pages from each inferring their own selected project or Source Engine target. The scope-model rule is: view-model and binding code derive visible Hub project/source-engine copy from one canonical `HubScope`, and Slint pages consume projected DTO fields instead of recomputing fallback rules.

## Canonical scope

`HubSnapshot::scope()` builds a `HubScope` from the selected project path, recent-project registry, project metadata, registered Source Engines, and active Source Engine id. It distinguishes four project cases:

- `Selected`: the selected path still resolves to a recent project.
- `StaleSelection`: the selected path no longer resolves and must not silently fall back to the latest recent project.
- `LatestRecent`: no selection exists, so Dashboard-style quick actions may use the latest recent project.
- `None`: no project exists.

The same scope also derives Source Engine context: selected-project bound engine, selected-project unbound state, selected-project unavailable engine, active engine fallback, or no engine. View-model projections for Source Engine summary, build history, and quick actions use this state instead of repeating metadata lookups.

The scope model locks the important separation between project-bound and Hub-active engines. A selected project with no metadata is `ProjectUnbound`; a selected project whose metadata references a removed engine is `ProjectEngineUnavailable`; neither case is allowed to borrow the active engine fallback. Active or first-engine fallback only applies when there is no selected project context. Project detail, project-row, build history, Source Engine summary, and workspace readiness projections therefore show `Unbound`, `Unavailable`, or explicit disabled reasons for the project itself instead of making the active Source Engine look like the selected project's bound engine.

The responsive Hub runtime-state scope-model milestone extends the same rule into user-facing quick-action copy. `QuickActionData.detail` is derived from `HubSnapshot::scope()` rather than a static action blurb: selected projects name the selected target, no-selection fallback names the latest recent target, stale selections say the selected project is no longer available, unbound projects ask for a Source Engine binding, unavailable bindings say the bound Source Engine is unavailable, and a truly empty Hub asks the user to select a project before project-only actions. This keeps disabled-state reasons aligned with the canonical scope enum and prevents Slint pages from recomputing target state from visible text.

## Passive DTO projection

`app/view_model.rs`, `projects.rs`, `quick_actions.rs`, and `workspace_actions.rs` copy `HubScope` into Slint-facing DTOs. `SourceEngineData` and build-history rows use project-bound engines when the selected project has a valid binding, return explicit missing/unavailable copy for selected-project problems, and only use the active or first registered engine when there is no selected project context. `ProjectDetailData` remains an explicit no-selection state until `selected_project_path` resolves, while Dashboard quick actions are the only UI surface in this milestone allowed to name the latest recent project fallback.

`binding.rs` forwards these DTOs to `HubWindow` in one pass: `project_detail`, `quick_actions`, `source_engine`, and `workspace_action_readiness` all come from the snapshot and not from page-local Slint expressions. `shared.slint`, `app.slint`, `builds.slint`, and `editor.slint` consume the projected fields passively. The scope-model milestone intentionally does not own process launch, persisted action history, restart restore, or broad catalog refresh; the command-action milestone below owns the process/action-history behavior layered behind the same visible scope.

## Command action runtime scope

`runtime.rs` and `runtime/project_workspace.rs` keep Dashboard quick actions separate from selected-project page actions. Dashboard quick actions call fallback-capable helpers such as `selected_or_latest_recent_project_for_action()` so a no-selection Dashboard can target the latest recent project. Builds, Editor, Cloud package/install, and other selected-project controls use `selected_project_for_named_action()` or `selected_project_with_engine_for_named_action()` so stale or missing selections report explicit errors instead of silently promoting another recent project.

Command handlers record recoverable outcomes through `HubActionRecord`. Build/open-output/open-editor/package/install records preserve command lines, process ids, output or install directories, log excerpts, exact failure detail, and recovery guidance when those values are available. `runtime/persistence.rs` owns the `record_action_and_persist()` path, and command-created action records use that helper so the in-memory history mutation and Hub config save stay one operation from the command handler's point of view.

`TaskStatus` carries the same scope into the visible status banner. Each command outcome sets an operation kind and target, then `binding.rs` projects `operation_summary()` and `detail_with_recovery()` into Slint. `binding::apply_snapshot()` also refreshes quick actions, project detail, operation timeline, Source Engine build history, status banner, and active Source Engine data after command results, so command feedback is visible without restarting Hub.

## Scoped catalog snapshots

`runtime/source_scoped_views.rs` converts `HubScope` into scanner roots for
catalog-like pages. `selected_project_catalog_root()` returns a root only when
the current selection resolves to a real recent project, so stale selected paths
do not leak into Assets, Plugins, Learn, Team, Cloud, Builds, or Editor
projections. `source_engine_catalog_roots()` prefers the Source Engine bound to
the selected project, refuses to borrow the active Source Engine for an unbound
or unavailable selected project, and falls back to the active or first Source
Engine only when no selected-project context exists.

The refresh wrappers in `asset_catalog.rs`, `plugin_catalog.rs`,
`learn_catalog.rs`, and `team_overview.rs` now receive those scope-derived roots
instead of reading raw settings directly. Selected-project content is offered to
the scanner before Source Engine or global roots, while the scanner-level
dedupe keeps a repeated selected-project root from producing duplicate cards or
rows. This makes catalog snapshots follow the same target model as the command
and view-model layers: project-specific content wins when the project is valid,
explicit project problems stay visible, and global Source Engine content is not
silently promoted into a broken selected project.

## Restart persistence

`HubConfig` is the persisted owner for Hub runtime state. Its
`HubRuntimeState` group stores the selected navigation page, Projects subpage,
project filter/sort/view mode, search text, selected project path, selected
template, New Project location, and New Project Source Engine selection. The
navigation and Projects view enums serialize with stable kebab-case ids, so the
TOML state can round-trip without depending on Slint labels or editor recent
JSON.

`runtime/persistence.rs` restores Hub runtime state before refreshing snapshots.
A persisted selected project path is canonicalized when it still matches a
recent project, but a missing persisted selected path remains in
`selected_project_path` so the view-model can show a stale selection instead of
promoting the latest recent project. Editor recent sync remains only an
import/export bridge: it supplies a startup fallback when Hub has no persisted
selected project, and it does not own project metadata, Source Engine bindings,
or page/subpage state.

Successful UI callbacks call `persist_hub_config()` after the action completes,
so lightweight navigation changes such as page switches, Projects subpage
changes, browser filter/sort/view choices, project selection, template
selection, and Source Engine selection survive a restart. Command paths that
already save action history keep using `record_action_and_persist()`, while the
callback-level save records the latest visible Hub state without requiring each
page method to duplicate TOML persistence.

## Validation ownership

Unit tests in `scope.rs`, `hub_snapshot.rs`, `view_model.rs`, and `quick_actions.rs` lock the low-level scope behavior. Unit tests in `action_history.rs`, `task_status.rs`, `hub_config.rs`, `runtime.rs`, and `source_scoped_views.rs` lock action-history retention, recoverable status text, runtime-state TOML round trips, target resolution, persisted command outcomes, and scope-derived catalog roots. Static contract tests in `project_quick_actions_contract.rs`, `ui_selected_project_runtime_contract.rs`, `ui_project_scope_contract.rs`, `project_workflow_contract.rs`, `project_path_scope_contract.rs`, and `ui_selected_project_catalog_contract.rs` ensure the view-model, passive UI layers, command handlers, persistence paths, path-scope helpers, and catalog refresh wrappers keep using the shared scope and action-history helpers instead of reintroducing page-local fallback/persistence branches. The cross-child consolidation milestone is the promotion gate that records how those focused checks fit together.

For `hub-runtime-state-integration-scope-model`, acceptance is the focused Hub package set listed in the document header. The scope tests cover selected, stale selected, latest recent, no-project, project-bound, unbound, unavailable, active fallback, first-engine fallback, and no-engine states. View-model tests cover selected-project Source Engine summaries, build-history selection, new-project unavailable-engine copy, project detail build readiness, and quick-action stale/latest/no-selection copy. The static contracts protect `HubSnapshot::scope()`, `binding.rs` DTO forwarding, `WorkspaceActionReadinessData`, and the passive Builds/Editor/UI consumption paths without claiming command actions, persistence, scoped catalogs, or restart restore for this milestone.

## Foundation contract gate

The `hub-foundations-contracts-docs` workflow gate records the first `继续完善hub` foundation acceptance layer. Its contract tests span the repaired registry/scope model, project quick-action copy, Source Engine binding contracts, and selected-project runtime static contracts. The gate exists so higher page milestones can rely on one canonical foundation model instead of duplicating registry repair, Source Engine fallback, and selected-project stale-state rules inside individual page tests.

Foundation-gate validation is intentionally narrower than the later page-family gates: `project_management_contract.rs` protects registry repair and documentation evidence, `project_quick_actions_contract.rs` protects Dashboard quick-action scope copy, `project_source_engine_contract.rs` protects Source Engine binding/metadata behavior, and `ui_selected_project_runtime_contract.rs` protects passive selected-project runtime DTO consumption. The accepted command set is `cargo test -p zircon_hub --test project_management_contract --locked -- --nocapture`, `cargo test -p zircon_hub --test project_quick_actions_contract --locked -- --nocapture`, `cargo test -p zircon_hub --test project_source_engine_contract --locked -- --nocapture`, and `cargo test -p zircon_hub --test ui_selected_project_runtime_contract --locked -- --nocapture`, followed by the milestone's scoped Hub check/build commands.

### 2026-05-30 scope-model evidence

The `hub-runtime-state-integration-scope-model` validation pass kept the milestone limited to canonical scope, scope-derived view-model copy, passive Slint DTO forwarding, focused contracts, and this document. During the cleanup pass, the focused static contracts were narrowed to remove assertions for command actions, persistence, scoped catalogs, restart restore, Cloud package/install status, operation timeline persistence, process launch, and unrelated runtime helpers. Those topics remain assigned to their later child milestones. The following commands passed with `--locked` where applicable:

- `cargo fmt -p zircon_hub --check`.
- `cargo test -p zircon_hub --test project_quick_actions_contract --locked -- --nocapture`.
- `cargo test -p zircon_hub --test ui_selected_project_runtime_contract --locked -- --nocapture`.
- `cargo test -p zircon_hub --test ui_project_scope_contract --locked -- --nocapture`.
- `cargo test -p zircon_hub state::scope --locked -- --nocapture`.
- `cargo test -p zircon_hub state::hub_snapshot --locked -- --nocapture`.
- `cargo test -p zircon_hub app::view_model::quick_actions --locked -- --nocapture`.
- `cargo test -p zircon_hub app::view_model --locked -- --nocapture`.
- `cargo check -p zircon_hub --locked`.

The repository working tree already contained unrelated active Hub, editor, runtime, plugin, asset, sound, and visual-reference changes from other sessions. This evidence is intentionally scoped to the files and tests owned by this milestone; it does not promote those unrelated changes.

The executable approval boundary for this child milestone is recorded in
`hub-runtime-state-integration-scope-model/review-surface.md`. That file lists
the accepted files, explicit exclusions, and focused validation gate so this
scope-model prerequisite can be reviewed without pulling in later command,
persistence, scoped-snapshot, contract-docs, or unrelated non-Hub changes from
the shared working tree.

### 2026-05-30 command-actions evidence

The `hub-runtime-state-integration-command-actions` pass kept command execution
scope separate from the accepted scope-model layer. The code change in this
pass routes build command success and failure records through
`record_action_and_persist()` instead of splitting `record_action()` from manual
config persistence. The `project_workflow_contract.rs` contract now locks that
path, while existing runtime tests cover selected-project-only Builds actions,
Dashboard fallback behavior, Source Engine binding errors, persisted package and
install failures, process id retention, and recoverable task status copy.

The following commands passed with `--locked` where applicable:

- `cargo fmt -p zircon_hub --check`.
- `cargo test -p zircon_hub state::action_history --locked --jobs 1 -- --nocapture`.
- `cargo test -p zircon_hub state::task_status --locked --jobs 1 -- --nocapture`.
- `cargo test -p zircon_hub app::runtime --locked --jobs 1 -- --nocapture`.
- `cargo test -p zircon_hub --test project_quick_actions_contract --locked --jobs 1 -- --nocapture`.
- `cargo test -p zircon_hub --test ui_selected_project_runtime_contract --locked --jobs 1 -- --nocapture`.
- `cargo test -p zircon_hub --test project_workflow_contract --locked --jobs 1 -- --nocapture`.
- `cargo check -p zircon_hub --locked --jobs 1`.

The first `state::action_history` invocation timed out at the command wrapper
while Cargo was still compiling under concurrent repository validation load. The
same process finished, and the rerun above captured the passing result. The
review boundary for this child milestone is recorded in
`hub-runtime-state-integration-command-actions/review-surface.md`; it should be
used instead of broad working-tree status when reviewing this command-action
slice.

### 2026-05-30 scoped-snapshots evidence

The `hub-runtime-state-integration-scoped-snapshots` pass connected Assets,
Plugins, Learn, and Team runtime refreshes to `HubScope` through
`source_scoped_views.rs`. The focused tests cover stale selected projects being
ignored for selected-project scanner roots, valid project-bound Source Engines
being preferred over the active engine, unbound selected projects refusing
active-engine fallback, and no-selection Hub state using the active Source
Engine. The static catalog contract locks the same helper usage in the runtime
refresh wrappers and prevents those pages from rebuilding raw project or Source
Engine fallback logic.

The following commands passed with `--locked` where applicable:

- `cargo fmt -p zircon_hub --check`.
- `cargo test -p zircon_hub app::runtime::source_scoped_views --locked --jobs 1 -- --nocapture`.
- `cargo test -p zircon_hub assets::catalog --locked --jobs 1 -- --nocapture`.
- `cargo test -p zircon_hub plugins::catalog --locked --jobs 1 -- --nocapture`.
- `cargo test -p zircon_hub learn::catalog --locked --jobs 1 -- --nocapture`.
- `cargo test -p zircon_hub --test ui_selected_project_catalog_contract --locked --jobs 1 -- --nocapture`.
- `cargo check -p zircon_hub --locked --jobs 1`.

The first `app::runtime::source_scoped_views` invocation timed out at the
command wrapper while Cargo was still compiling. The build process completed,
and the rerun above captured the passing result. The review boundary for this
child milestone is recorded in
`hub-runtime-state-integration-scoped-snapshots/review-surface.md`; it scopes
review to source-derived catalog refresh behavior and excludes restart restore,
command persistence, and unrelated workspace changes.

### 2026-05-30 restart-persistence evidence

The `hub-runtime-state-integration-restart-persistence` pass moved Hub-visible
runtime state into `HubConfig.runtime` and made successful UI callbacks save
that state through `persist_hub_config()`. Runtime load now prefers the
Hub-owned selected project path over editor recent fallback, preserves a missing
persisted path as stale, canonicalizes a persisted path when it still matches a
recent project, and uses editor recent only when Hub has no persisted selected
project. Background build reload now trusts the persisted selected page instead
of passing a stale pre-build page value across the worker boundary.

The following commands passed with `--locked` where applicable:

- `cargo fmt -p zircon_hub`.
- `cargo check -p zircon_hub --locked --jobs 1`.
- `cargo test -p zircon_hub settings::hub_config --locked --jobs 1 -- --nocapture`.
- `cargo test -p zircon_hub app::runtime --locked --jobs 1 -- --nocapture`.
- `cargo test -p zircon_hub --test project_workflow_contract --locked --jobs 1 -- --nocapture`.
- `cargo test -p zircon_hub --test project_source_engine_contract --locked --jobs 1 -- --nocapture`.
- `cargo test -p zircon_hub --test project_path_scope_contract --locked --jobs 1 -- --nocapture`.

The first `settings::hub_config` invocation timed out at the command wrapper
while Cargo was still compiling. No Cargo or Rust compiler process remained
after the interrupted wait, and the rerun above captured the passing result.
`project_path_scope_contract.rs` initially failed because its static assertion
still expected Assets/Learn/Plugins refresh modules to call `root_paths`
directly. The root-path ownership had already moved to
`source_scoped_views.rs`, so the contract was updated to assert the current
single-entry helper instead.

### 2026-05-30 contract-docs evidence

The `hub-runtime-state-integration-contract-docs` pass promotes the four
runtime-state child milestones as one reviewable contract set. It found no
missing child acceptance area beyond documentation consolidation: scope-model
owns `HubSnapshot::scope()` and passive DTO projection, command-actions owns
selected-project command routing plus persisted action history, scoped-snapshots
owns selected-project-before-Source-Engine catalog refresh, and
restart-persistence owns `HubConfig.runtime` plus stale selected-path restore.

This promotion gate deliberately changes documentation and static contract
coverage rather than runtime behavior. It ties the child review surfaces to
`docs/zircon_hub/state/foundations.md`,
`docs/zircon_hub/ui/responsive-component-system.md`, and
`docs/zircon_hub/index.md` so follow-on `hub-docs-contract-refresh` and
`hub-acceptance-validation` work can validate runtime-state ownership without
guessing from the shared dirty working tree.

The following validation gate passed with `--locked` where applicable:

- `cargo fmt -p zircon_hub --check`.
- `cargo check -p zircon_hub --locked --jobs 1`.
- `cargo test -p zircon_hub --test ui_selected_project_runtime_contract --locked --jobs 1 -- --nocapture`.
- `cargo test -p zircon_hub --test ui_selected_project_catalog_contract --locked --jobs 1 -- --nocapture`.
- `cargo test -p zircon_hub --test project_quick_actions_contract --locked --jobs 1 -- --nocapture`.
- `cargo test -p zircon_hub --test project_workflow_contract --locked --jobs 1 -- --nocapture`.
- `cargo test -p zircon_hub --test ui_project_navigation_contract --locked --jobs 1 -- --nocapture`.
- `cargo test -p zircon_hub --test project_source_engine_contract --locked --jobs 1 -- --nocapture`.
- `cargo test -p zircon_hub --test project_path_scope_contract --locked --jobs 1 -- --nocapture`.
- `cargo test -p zircon_hub --locked --jobs 1`.

The first full-package run exposed an outdated
`ui_project_navigation_contract.rs` assertion that still expected New Project
location initialization to be owned directly by Settings defaults. The runtime
had already moved editable New Project form state into `HubConfig.runtime`, so
the contract was updated to protect that persisted runtime owner plus the
Settings default-project-directory fallback used for empty/error defaults.

The review boundary for this consolidation pass is recorded in
`hub-runtime-state-integration-contract-docs/review-surface.md`; it is the
handoff surface for broader Hub docs and acceptance validation.

## Runtime-State Docs Refresh Handoff

`hub-docs-contract-refresh` consumes this document as the runtime-state
ownership map. It should cite
`hub-runtime-state-integration-contract-docs/review-surface.md` instead of
reconstructing the split from raw diffs: `HubSnapshot::scope()` remains the
scope resolver, `HubConfig.runtime` remains the restart-visible UI state owner,
`record_action_and_persist()` remains the command action-history persistence
path, and `source_scoped_views.rs` remains the single catalog-root owner.

`hub_docs_contract.rs` now scans this handoff so the broader docs refresh and
the following acceptance validation cannot drop the runtime-state split, the
child review surfaces, or the focused validation commands.
