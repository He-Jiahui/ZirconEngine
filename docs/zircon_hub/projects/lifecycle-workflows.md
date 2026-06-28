---
related_code:
  - zircon_hub/src/tauri_app/action_request.rs
  - zircon_hub/src/tauri_app/action_id.rs
  - zircon_hub/src/tauri_app/runtime_state.rs
  - zircon_hub/src/tauri_app/runtime_state/action_targets.rs
  - zircon_hub/src/tauri_app/runtime_state/project_actions.rs
  - zircon_hub/src/tauri_app/runtime_state/project_actions/tests.rs
  - zircon_hub/src/tauri_app/runtime_state/project_delivery_actions.rs
  - zircon_hub/src/tauri_app/runtime_state/editor_launch_actions.rs
  - zircon_hub/src/tauri_app/runtime_state/scoped_views.rs
  - zircon_hub/src/tauri_app/view_model.rs
  - zircon_hub/src/tauri_app/view_model/coming_soon.rs
  - zircon_hub/src/tauri_app/view_model/project_templates.rs
  - zircon_hub/src/tauri_app/view_model/localized.rs
  - zircon_hub/src/projects/create_project.rs
  - zircon_hub/src/projects/create_project_request.rs
  - zircon_hub/src/projects/editor_recent_sync.rs
  - zircon_hub/src/projects/metadata.rs
  - zircon_hub/src/projects/recent_project.rs
  - zircon_hub/src/projects/recycle_bin.rs
  - zircon_hub/src/projects/validation.rs
  - zircon_hub/src/process/editor_launch.rs
  - zircon_hub/web/src/tauri/projectTarget.ts
  - zircon_hub/web/src/components/data/ProjectCardRail.tsx
  - zircon_hub/web/src/components/data/ProjectDetailSidebar.tsx
  - zircon_hub/web/src/components/data/ProjectMetricsGrid.tsx
  - zircon_hub/web/src/components/data/ProjectTable.tsx
  - zircon_hub/web/src/components/inputs/ProjectsToolbar.tsx
  - zircon_hub/web/src/components/overlays/CreateProjectDialog.tsx
  - zircon_hub/web/src/pages/ProjectsDashboard.tsx
  - zircon_hub/web/src/pages/ProjectBrowserPage.tsx
  - zircon_hub/web/src/pages/ProjectDetailPage.tsx
  - zircon_hub/web/src/pages/BuildsPage.tsx
  - zircon_hub/web/src/pages/CloudPage.tsx
  - zircon_hub/web/src/pages/EditorPage.tsx
  - zircon_hub/web/src/types/hub.ts
  - zircon_hub/tests/hub_docs_contract.rs
  - zircon_hub/tests/project_management_contract.rs
  - zircon_hub/tests/project_page_copy_contract.rs
  - zircon_hub/tests/project_quick_actions_contract.rs
  - zircon_hub/tests/project_source_engine_contract.rs
  - zircon_hub/tests/project_workflow_contract.rs
  - zircon_hub/tests/ui_project_navigation_contract.rs
  - zircon_hub/tests/ui_selected_project_runtime_contract.rs
implementation_files:
  - zircon_hub/src/tauri_app/action_request.rs
  - zircon_hub/src/tauri_app/action_id.rs
  - zircon_hub/src/tauri_app/runtime_state.rs
  - zircon_hub/src/tauri_app/runtime_state/action_targets.rs
  - zircon_hub/src/tauri_app/runtime_state/project_actions.rs
  - zircon_hub/src/tauri_app/runtime_state/project_actions/tests.rs
  - zircon_hub/src/tauri_app/runtime_state/project_delivery_actions.rs
  - zircon_hub/src/tauri_app/runtime_state/editor_launch_actions.rs
  - zircon_hub/src/tauri_app/runtime_state/scoped_views.rs
  - zircon_hub/src/tauri_app/view_model.rs
  - zircon_hub/src/tauri_app/view_model/coming_soon.rs
  - zircon_hub/src/tauri_app/view_model/project_templates.rs
  - zircon_hub/src/tauri_app/view_model/localized.rs
  - zircon_hub/src/projects/create_project.rs
  - zircon_hub/src/projects/create_project_request.rs
  - zircon_hub/src/projects/editor_recent_sync.rs
  - zircon_hub/src/projects/metadata.rs
  - zircon_hub/src/projects/recent_project.rs
  - zircon_hub/src/projects/recycle_bin.rs
  - zircon_hub/src/projects/validation.rs
  - zircon_hub/src/process/editor_launch.rs
  - zircon_hub/web/src/tauri/projectTarget.ts
  - zircon_hub/web/src/components/data/ProjectCardRail.tsx
  - zircon_hub/web/src/components/data/ProjectDetailSidebar.tsx
  - zircon_hub/web/src/components/data/ProjectMetricsGrid.tsx
  - zircon_hub/web/src/components/data/ProjectTable.tsx
  - zircon_hub/web/src/components/inputs/ProjectsToolbar.tsx
  - zircon_hub/web/src/components/overlays/CreateProjectDialog.tsx
  - zircon_hub/web/src/pages/ProjectsDashboard.tsx
  - zircon_hub/web/src/pages/ProjectBrowserPage.tsx
  - zircon_hub/web/src/pages/ProjectDetailPage.tsx
  - zircon_hub/web/src/pages/BuildsPage.tsx
  - zircon_hub/web/src/pages/CloudPage.tsx
  - zircon_hub/web/src/pages/EditorPage.tsx
plan_sources:
  - .codex/plans/Zircon Hub 本地闭环 v1 功能实现设计.md
  - .codex/plans/Zircon Hub Tauri + ReactMUI 硬切换计划.md
  - .opencode/workflows/20260528_231820_026_优化hub[image Zircon Hub 响应式组件化重构计划.md]/hub-docs-contract-refresh/plan.md
  - .opencode/workflows/20260528_231820_026_优化hub[image Zircon Hub 响应式组件化重构计划.md]/hub-docs-contract-refresh/review-surface.md
tests:
  - zircon_hub/tests/hub_docs_contract.rs
  - cargo test --manifest-path zircon_hub/Cargo.toml --test hub_docs_contract -- --nocapture
  - cargo test --manifest-path zircon_hub/Cargo.toml --test project_management_contract -- --nocapture
  - cargo test --manifest-path zircon_hub/Cargo.toml --test project_workflow_contract -- --nocapture
  - cargo test --manifest-path zircon_hub/Cargo.toml --test project_source_engine_contract -- --nocapture
  - cargo test --manifest-path zircon_hub/Cargo.toml --test project_quick_actions_contract -- --nocapture
  - cargo test --manifest-path zircon_hub/Cargo.toml --test ui_project_layout_contract -- --nocapture
  - cargo test --manifest-path zircon_hub/Cargo.toml --test ui_project_navigation_contract -- --nocapture
  - cargo test --manifest-path zircon_hub/Cargo.toml --test ui_selected_project_runtime_contract -- --nocapture
doc_type: workflow-detail
---

# Hub project lifecycle workflows

This document owns the React/MUI project lifecycle surface for local Hub v1. The current lifecycle starts in the Tauri-backed Projects pages, flows through typed `hub_action` payloads, mutates only `zircon_hub` state and project files, then returns a refreshed `HubViewModel` for React to render.

## Action Model

`HubActionRequest` parses project lifecycle actions at the IPC boundary. `CreateProjectActionPayload` carries `name`, `location`, `template`, and `engineId`; `ImportProjectActionPayload` carries an optional project folder; `ProjectTargetActionPayload` carries `projectId` and `projectPath` for selected-project workflows.

`HubRuntimeSession::apply_action()` is the runtime owner. Project creation routes to `create_project_from_payload`, import routes to `import_project_from_action`, and pin/unpin/remove/delete routes to project lifecycle helpers in `src/tauri_app/runtime_state/project_actions.rs`. Workflow buttons for build, package, install, and open-editor first call the shared target resolver in `action_targets.rs`.

Runtime 15 M3 support Hub project-actions tests child-owner split (`runtime_15_support_hub_project_actions_tests_child_owner_split_static_passed_cargo_deferred`) keeps `zircon_hub/src/tauri_app/runtime_state/project_actions.rs` focused on the project-action runtime owner while `zircon_hub/src/tauri_app/runtime_state/project_actions/tests.rs` owns create/import/delete behavior coverage, `session_with_source`, and temp-dir fixtures. Guard `runtime_15_support_hub_project_actions_tests_are_child_owner` prevents those test fixtures from flowing back into the parent and locks the project-actions tests child-owner split docs/status anchors.

React does not infer filesystem identity. `web/src/tauri/projectTarget.ts` exposes `projectTargetPayload(project)` and `quickActionProjectTargetPayload(project)`. Project Detail, Editor, Builds, Catalog, and Cloud pass `{ projectId, projectPath }` when a selected project is visible, while Rust target resolution prefers `projectPath` before the stable id and keeps `targetId` only as compatibility fallback.

Dashboard-style Open Editor can fall back to the latest recent project only when no selected project exists. Selected-project actions must keep using selected-path matching and must not retarget a stale selection.

`ProjectsDashboard.tsx` owns project route state and composes the reusable project surface through `ProjectsToolbar`, `ProjectCardRail`, `ProjectTable`, and `CreateProjectDialog`. `ProjectDetailPage.tsx` owns the detail route shell and delegates overview metrics to `ProjectMetricsGrid` and workflow/support panels to `ProjectDetailSidebar`. These components receive explicit nullable ids such as `selectedProjectId` and `activeSourceEngineId`; absence is represented by `null`, not by omitted props, so the React boundary matches the `HubViewModel` DTO shape.

## Create And Import

`CreateProjectRequest` owns request-level validation and target-root derivation. `create_project()` currently enables only the `renderable-empty` template. It validates that the target directory is empty or missing, creates the standard local project layout, writes `zircon-project.toml`, writes the default scene and starter assets, then returns a `CreateProjectReport`.

`create_project_from_payload` resolves the Source Engine id, rejects disabled templates as localized coming-soon failures, calls `create_project()`, remembers the new project in Hub recent-project metadata, stores the bound engine/template, selects the new project, refreshes selected-project scoped catalogs, records a create-project action-history row, and persists Hub plus Editor recent state. `CreateProjectDialog` still renders disabled template rows from the DTO so future templates remain visible, but it can submit only enabled templates.

Import either uses the typed payload folder or opens the localized native folder picker. It validates `zircon-project.toml`, writes the project to recent-project metadata, binds the requested or active Source Engine when available, selects the imported project, moves Projects to Detail, refreshes scoped catalogs, records an import action-history row, and persists the selected project for Editor startup.

Import paths are normalized through `src/projects/metadata.rs` before matching metadata or recent-project entries. `src/projects/validation.rs` parses `zircon-project.toml` as TOML and rejects missing or malformed manifests with localized recoverable task feedback; a native folder-picker cancellation returns to the current state without adding history or mutating recents. Successful import de-duplicates by normalized path, so the same project cannot appear twice because of slash, casing, or relative-path drift.

Create-project recovery is intentionally conservative. A non-empty target directory does not overwrite files and reports an import-oriented recovery hint. If Hub fails after creating the project files but before recording recent metadata, the generated folder remains on disk for manual import instead of being deleted as part of error recovery.

## Project Detail Management

`ProjectDetailPage.tsx` renders the selected project DTO passively. It does not prune metadata, normalize paths, or decide delete state. `ProjectMetricsGrid` owns the fixed four-card overview, while `ProjectDetailSidebar` owns quick actions, Source Engine rows, package/install commands, and project-management buttons. Pin, unpin, remove-from-hub, request-delete, cancel-delete, and confirm-delete dispatch typed selected-project payloads and wait for the refreshed view model.

Pin and unpin update only Hub project metadata. Remove from Hub prunes the recent project, selected path, pending delete path, and project metadata while leaving files on disk. Delete is a two-step Recycle Bin flow: `request-delete` sets `pending_delete_project_path`, `cancel-delete` clears it, and `confirm-delete` calls the Windows Recycle Bin command before removing Hub metadata. If the Recycle Bin command fails, Hub preserves the selected project and pending delete state so the user can retry or cancel.

The Recycle Bin owner quotes filesystem paths through the command builder instead of string-concatenating raw paths. Contract tests cover spaces, Unicode, quote characters, and newline rejection, plus non-Windows recovery behavior. Delete therefore either moves the exact selected project path to the platform recycle flow or preserves Hub state for retry; it does not silently remove metadata after a failed file operation.

Project status, template display, missing-path copy, pending-delete state, and action feedback are localized at the Rust DTO boundary through `HubTextBundle`. React renders those fields without reconstructing business copy.

## Workflow Hand Offs

Builds, Cloud, and Editor reuse the selected-project payload. Package and install are owned by `src/tauri_app/runtime_state/project_delivery_actions.rs`; package writes `zircon-package.toml`, and install ensures a package before copying to the configured local device install directory. Editor launch is owned by `src/tauri_app/runtime_state/editor_launch_actions.rs` and can launch the selected project or an empty editor.

Editor launch command construction stays in `src/process/editor_launch.rs`. Open-project requests produce `--project <path>`. Empty editor launch omits a project argument. The Hub no longer creates a project by launching the Editor create mode; project creation is local file generation in `src/projects/create_project.rs`.

Every lifecycle and workflow handoff records action-history data through the Hub runtime state. `HubActionRecord` carries stable kind ids, localized detail/log/recovery fields at projection time, command lines, output directories, and child process ids when available. Builds and Cloud render the DTO `detailRows` or `outputDir` instead of page-local diagnostics.

## Persistence

`HubConfig.runtime` stores selected page, project subpage, search/filter/sort/view mode, selected project path, selected template id, new-project location, and new-project engine id. Hub project metadata stores pin state, Source Engine binding, and last selected template by normalized project path. Editor recent sync remains isolated to the existing Editor JSON startup session.

Recent projects merge by normalized path key, keep newest entries, and preserve Editor state that is outside Hub ownership. Hub writes Editor recent state only through the runtime persistence boundary after project create/import/open/remove/delete state changes that intentionally affect startup context.

## Validation Ownership

The project lifecycle gate is covered by Rust module tests and static React/Tauri contracts. `project_management_contract.rs` covers metadata, registry repair, template, Recycle Bin, and persistence boundaries. `project_workflow_contract.rs` covers typed action payload parsing, runtime routing, create/import/workflow actions, and the single frontend dispatcher. `project_source_engine_contract.rs` covers Source Engine binding/default behavior. `project_quick_actions_contract.rs` covers quick-action target fallback and selected-project payload forwarding. `ui_project_layout_contract.rs` protects dashboard toolbar/card rail/dialog, browser table, detail metrics, and detail sidebar composition. `ui_project_navigation_contract.rs` protects Dashboard/New/Browser/Detail routing. `ui_selected_project_runtime_contract.rs` protects passive selected-project DTO consumption.

## Docs Refresh Handoff

`hub-docs-contract-refresh` keeps the project lifecycle rules visible to future page and acceptance work. Dashboard-style Open Editor can fall back to the latest recent project only when no selected project exists; selected-project actions must keep using selected-path matching and must not retarget a stale selection. New Project form state survives restart through `HubConfig.runtime`, while Settings default project location remains only the empty/error fallback.

The handoff contract is split by responsibility: `project_workflow_contract.rs` protects create/import/open/action ordering, `ui_project_navigation_contract.rs` protects Dashboard/New/Browser/Detail routing plus New Project location state, `project_source_engine_contract.rs` protects Source Engine binding/default behavior, and `hub_docs_contract.rs` keeps these documentation links present for the acceptance milestone.
