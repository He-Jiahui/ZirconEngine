---
related_code:
  - zircon_hub/src/app/runtime/project_workspace.rs
  - zircon_hub/src/app/runtime/persistence.rs
  - zircon_hub/src/app/runtime/folder_picker.rs
  - zircon_hub/src/app/runtime/source_scoped_views.rs
  - zircon_hub/src/app/binding.rs
  - zircon_hub/src/app/quick_action.rs
  - zircon_hub/src/app/view_model/projects.rs
  - zircon_hub/src/app/view_model/quick_actions.rs
  - zircon_hub/src/projects/create_project_request.rs
  - zircon_hub/src/projects/metadata.rs
  - zircon_hub/src/projects/recent_project.rs
  - zircon_hub/src/projects/recycle_bin.rs
  - zircon_hub/src/projects/validation.rs
  - zircon_hub/src/process/editor_launch.rs
  - zircon_hub/ui/projects.slint
  - zircon_hub/ui/project_dashboard.slint
  - zircon_hub/ui/project_dashboard_components.slint
  - zircon_hub/ui/project_new_page.slint
  - zircon_hub/ui/project_browser_page.slint
  - zircon_hub/ui/project_browser_components.slint
  - zircon_hub/ui/project_detail_page.slint
  - zircon_hub/ui/project_detail_components.slint
  - zircon_hub/ui/project_page_components.slint
  - zircon_hub/ui/project_pages.slint
  - zircon_hub/ui/shared.slint
implementation_files:
  - zircon_hub/src/app/runtime/project_workspace.rs
  - zircon_hub/src/app/runtime/persistence.rs
  - zircon_hub/src/app/runtime/folder_picker.rs
  - zircon_hub/src/app/runtime/source_scoped_views.rs
  - zircon_hub/src/app/binding.rs
  - zircon_hub/src/app/quick_action.rs
  - zircon_hub/src/app/view_model/projects.rs
  - zircon_hub/src/app/view_model/quick_actions.rs
  - zircon_hub/src/projects/create_project_request.rs
  - zircon_hub/src/projects/metadata.rs
  - zircon_hub/src/projects/recent_project.rs
  - zircon_hub/src/projects/recycle_bin.rs
  - zircon_hub/src/projects/validation.rs
  - zircon_hub/src/process/editor_launch.rs
  - zircon_hub/ui/projects.slint
  - zircon_hub/ui/project_dashboard.slint
  - zircon_hub/ui/project_dashboard_components.slint
  - zircon_hub/ui/project_new_page.slint
  - zircon_hub/ui/project_browser_page.slint
  - zircon_hub/ui/project_browser_components.slint
  - zircon_hub/ui/project_detail_page.slint
  - zircon_hub/ui/project_detail_components.slint
  - zircon_hub/ui/project_page_components.slint
  - zircon_hub/ui/project_pages.slint
  - zircon_hub/ui/shared.slint
plan_sources:
  - user: 2026-05-28 继续完善hub / hub-projects milestone
  - .opencode/workflows/20260528_190023_866_继续完善hub/workflow.xml
  - .opencode/workflows/20260528_190023_866_继续完善hub/hub-projects/decomposition.md
  - .opencode/workflows/20260528_190023_866_继续完善hub/hub-projects-actions-model/plan.md
  - .opencode/workflows/20260528_190023_866_继续完善hub/hub-projects-create-import-templates/plan.md
  - .opencode/workflows/20260528_190023_866_继续完善hub/hub-projects-dashboard-detail-ui/plan.md
  - .opencode/workflows/20260528_190023_866_继续完善hub/hub-projects-editor-launch-contracts/plan.md
  - .opencode/workflows/20260528_231820_026_优化hub[image Zircon Hub 响应式组件化重构计划.md]/hub-docs-contract-refresh/plan.md
  - .opencode/workflows/20260528_231820_026_优化hub[image Zircon Hub 响应式组件化重构计划.md]/hub-docs-contract-refresh/review-surface.md
tests:
  - zircon_hub/src/projects/create_project_request.rs
  - zircon_hub/tests/project_management_contract.rs
  - zircon_hub/tests/project_workflow_contract.rs
  - zircon_hub/tests/project_source_engine_contract.rs
  - zircon_hub/tests/project_quick_actions_contract.rs
  - zircon_hub/tests/ui_project_layout_contract.rs
  - zircon_hub/tests/ui_project_scope_contract.rs
  - zircon_hub/tests/ui_project_navigation_contract.rs
  - zircon_hub/tests/ui_selected_project_runtime_contract.rs
  - cargo test -p zircon_hub projects::create_project_request --locked -- --nocapture
  - cargo test -p zircon_hub app::runtime::project_workspace --locked -- --nocapture
  - cargo test -p zircon_hub app::view_model --locked -- --nocapture
  - cargo test -p zircon_hub --test project_management_contract --locked -- --nocapture
  - cargo test -p zircon_hub --test project_workflow_contract --locked -- --nocapture
  - cargo test -p zircon_hub --test project_source_engine_contract --locked -- --nocapture
  - cargo test -p zircon_hub --test project_quick_actions_contract --locked -- --nocapture
  - cargo test -p zircon_hub --test ui_project_layout_contract --locked -- --nocapture
  - cargo test -p zircon_hub --test ui_project_scope_contract --locked -- --nocapture
  - cargo test -p zircon_hub --test ui_project_navigation_contract --locked -- --nocapture
  - cargo test -p zircon_hub --test ui_selected_project_runtime_contract --locked -- --nocapture
  - cargo test -p zircon_hub --locked
  - cargo build -p zircon_hub --bin zircon_hub --locked
  - zircon_hub/tests/hub_docs_contract.rs
  - cargo test -p zircon_hub --test hub_docs_contract --locked --jobs 1 -- --nocapture
doc_type: workflow-detail
---

# Hub project lifecycle workflows

This document owns the Hub project lifecycle slice that starts at the Projects page and ends at deterministic Editor launch, package, install, remove, or delete behavior. The lifecycle is deliberately split between runtime mutation, project metadata helpers, view-model projection, Slint page composition, and static contract tests so the Hub can keep acting as a standalone launcher without moving project state into the Editor or runtime crates.

## Canonical project action model

`HubRuntime` keeps the selected project as an optional path and resolves actions through narrow helpers in `project_workspace.rs`. Selected-project actions such as Build, Package, Install, Project Detail Open, Remove from Hub, and Delete require the selected path to still match a recent project through `project_paths_match`. Dashboard-style Open Editor can fall back to the latest recent project only when no selected project exists; a stale selected path is treated as an actionable error instead of silently retargeting another project.

The recent-project registry remains ordered by normalized path de-duplication and last-opened timestamps. `remember_project()` selects the remembered project, activates its bound Source Engine when metadata exists, refreshes selected-project scoped Assets, Plugins, Learn, and Team data, and persists the Editor startup pointer through `persist_with_last_project()`. Pin state lives only in Hub project metadata and changes presentation ordering or badges; it does not become the Editor recent-project ordering source.

Remove and delete share the same normalized path cleanup helper but have different side effects. Remove from Hub prunes the recent row, project metadata, selected path, and pending-delete state, then persists Hub and Editor recent state without touching project files. Delete is a two-step Windows-only flow: request sets `pending_delete_project_path`, cancel clears it, and confirm calls the Recycle Bin command before removing Hub metadata. If the Recycle Bin command fails, Hub state remains intact.

## Create, import, and template flow

`CreateProjectRequest` owns request-level create validation and Editor argument mapping. It trims project names at construction time, rejects empty names or locations through `validate_launch_fields()`, computes the target root through `target_root()`, and maps enabled templates to the Editor `--template` argument. The catalog keeps disabled template rows visible so the UI can explain future workflow slots, while `ProjectTemplate::from_enabled_id()` accepts only templates that can currently launch through Editor create mode.

`create_project()` builds the request from Slint form state and `new_project_engine_id`, validates the request before launch, requires the selected Source Engine to exist, activates that engine, and launches `EditorLaunchRequest::CreateProject`. Only after the child process starts does the runtime write project metadata with the requested engine/template, remember the target root, select it, refresh scoped views, and record a successful Open Editor action. This order prevents action history from claiming a successful create when project metadata or Editor recent persistence failed.

Import and open flows validate project roots with `validate_project_root()`, remember valid imports as recents, select them, clear pending-delete state, route to Project Detail, refresh selected-project scoped views, and update Editor recents only through the runtime persistence boundary. Folder-picker callbacks keep import project roots, new-project locations, source checkout paths, staged outputs, and local device install paths separate so one browse action cannot overwrite another field.

## Slint projections and page behavior

`view_model::projects` projects the canonical runtime state into three surfaces: Dashboard cards, Project Browser rows, and Project Detail data. Rows expose selected, pinned, missing, can-open, can-build, can-delete, engine id, engine label, and unavailable/unbound Source Engine state from the same snapshot. Missing or invalid projects cannot open/build/delete; projects without an available bound Source Engine cannot build selected-engine actions even if a different active Source Engine exists.

Projects page composition is split by page responsibility. `project_dashboard.slint` owns the Dashboard shell and delegates cards, recent rows, and Quick Actions to `project_dashboard_components.slint`. `project_new_page.slint`, `project_browser_page.slint`, and `project_detail_page.slint` own the secondary page shells, while `project_page_components.slint`, `project_browser_components.slint`, and `project_detail_components.slint` own reusable controls such as template rails, browser rows, source-engine choices, detail status strips, detail info rows, action buttons, and delete-confirmation sections.

Project Detail renders an explicit no-selection state until `selected_project_path` resolves to a recent project. When a project is selected, it renders path, version, Source Engine binding, status, pin state, pending-delete state, and action buttons. The delete confirmation cluster appears only when the selected pending-delete path matches the selected project, with cancel and confirm callbacks kept separate from Remove from Hub.

## Editor launch contracts

Editor launch command construction stays in `process/editor_launch.rs`. Open-project requests produce `--project <path>`, create-project requests produce `--create-project --project-name <name> --location <dir> --template <template>`, and `EditorLaunchCommand::from_preferred_engine()` prefers a sibling `zircon_editor` beside the Hub executable before falling back to the configured staged engine directory.

Runtime launch callbacks decide the target before constructing commands. Selected-project launch never uses latest-recent fallback when a selected path is stale; general Open Editor uses selected project, latest recent, or no-project launch in that order; create mode uses the selected New Project Source Engine and enabled template. Each launch success or failure writes a scoped `HubActionRecord` with target, command line, recovery text, output directory, and child process id when available so the operation timeline survives restart.

## Validation ownership

The project lifecycle gate is covered by a mix of Rust module tests and static contracts. Module tests cover request/template helpers, project metadata normalization, project view-model projection, runtime selected/latest helpers, and editor command arguments. Contract tests lock the cross-file behavior: `project_management_contract.rs` covers metadata, registry repair, template and Recycle Bin contracts; `project_workflow_contract.rs` covers create/open/selected-project action routing; `project_source_engine_contract.rs` covers new-project engine defaults and shared path keys; `project_quick_actions_contract.rs` covers fallback and stale-selection quick actions; and the UI contract tests cover Dashboard, Browser, Detail, selected-project runtime wiring, and Slint callback surfaces.

The final acceptance gate for this milestone is Hub-scoped rather than whole-workspace because the changes are contained to `zircon_hub`. The required evidence is the focused project lifecycle command set plus `cargo test -p zircon_hub --locked` and `cargo build -p zircon_hub --bin zircon_hub --locked`. If local resource contention prevents a command from completing, record the timeout as risk instead of weakening the lifecycle contract.

## Docs Refresh Handoff

`hub-docs-contract-refresh` keeps the project lifecycle rules visible to future page and acceptance work. Dashboard-style Open Editor can fall back to the latest recent project only when no selected project exists; selected-project actions must keep using selected-path matching and must not retarget a stale selection. New Project form state now survives restart through `HubConfig.runtime`, while Settings default project location remains only the empty/error fallback.

The handoff contract is split by responsibility: `project_workflow_contract.rs` protects create/open/action ordering, `ui_project_navigation_contract.rs` protects Dashboard/New/Browser/Detail routing plus New Project location state, `project_source_engine_contract.rs` protects Source Engine binding/default behavior, and `hub_docs_contract.rs` keeps these documentation links present for the acceptance milestone.
