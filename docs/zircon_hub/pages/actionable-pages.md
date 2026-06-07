---
related_code:
  - zircon_hub/src/tauri_app/runtime_state.rs
  - zircon_hub/src/tauri_app/runtime_state/action_targets.rs
  - zircon_hub/src/tauri_app/runtime_state/action_tasks.rs
  - zircon_hub/src/tauri_app/runtime_state/build_actions.rs
  - zircon_hub/src/tauri_app/runtime_state/editor_launch_actions.rs
  - zircon_hub/src/tauri_app/runtime_state/learn_actions.rs
  - zircon_hub/src/tauri_app/runtime_state/output_actions.rs
  - zircon_hub/src/tauri_app/runtime_state/project_actions.rs
  - zircon_hub/src/tauri_app/runtime_state/project_delivery_actions.rs
  - zircon_hub/src/tauri_app/runtime_state/quick_actions.rs
  - zircon_hub/src/tauri_app/runtime_state/scoped_views.rs
  - zircon_hub/src/tauri_app/runtime_state/settings_actions.rs
  - zircon_hub/src/tauri_app/view_model.rs
  - zircon_hub/src/tauri_app/view_model/action_history.rs
  - zircon_hub/src/tauri_app/view_model/catalog.rs
  - zircon_hub/src/tauri_app/view_model/coming_soon.rs
  - zircon_hub/src/tauri_app/view_model/localized.rs
  - zircon_hub/src/tauri_app/view_model/settings_dto.rs
  - zircon_hub/src/tauri_app/view_model/source_engines.rs
  - zircon_hub/src/tauri_app/view_model/ui_text.rs
  - zircon_hub/src/assets/catalog.rs
  - zircon_hub/src/learn/catalog.rs
  - zircon_hub/src/plugins/catalog.rs
  - zircon_hub/src/projects/package.rs
  - zircon_hub/src/projects/device_install.rs
  - zircon_hub/src/team/local_git.rs
  - zircon_hub/web/src/App.tsx
  - zircon_hub/web/src/tauri/hubApi.ts
  - zircon_hub/web/src/tauri/projectTarget.ts
  - zircon_hub/web/src/pages/BuildsPage.tsx
  - zircon_hub/web/src/pages/CatalogPage.tsx
  - zircon_hub/web/src/pages/CloudPage.tsx
  - zircon_hub/web/src/pages/EditorPage.tsx
  - zircon_hub/web/src/pages/ProjectDetailPage.tsx
  - zircon_hub/web/src/pages/ProjectsDashboard.tsx
  - zircon_hub/web/src/pages/ProjectBrowserPage.tsx
  - zircon_hub/web/src/pages/SettingsPage.tsx
  - zircon_hub/web/src/pages/TeamPage.tsx
  - zircon_hub/web/src/pages/WorkspacePage.tsx
  - zircon_hub/web/src/components/data/HubList.tsx
  - zircon_hub/web/src/components/data/QuickActions.tsx
  - zircon_hub/web/src/components/data/SourceEngineList.tsx
  - zircon_hub/web/src/types/hub.ts
  - zircon_hub/tests/hub_docs_contract.rs
  - zircon_hub/tests/project_page_copy_contract.rs
  - zircon_hub/tests/project_workflow_contract.rs
  - zircon_hub/tests/ui_foundation_contract.rs
  - zircon_hub/tests/ui_selected_project_catalog_contract.rs
  - zircon_hub/tests/ui_selected_project_runtime_contract.rs
implementation_files:
  - zircon_hub/src/tauri_app/runtime_state.rs
  - zircon_hub/src/tauri_app/runtime_state/action_targets.rs
  - zircon_hub/src/tauri_app/runtime_state/action_tasks.rs
  - zircon_hub/src/tauri_app/runtime_state/build_actions.rs
  - zircon_hub/src/tauri_app/runtime_state/editor_launch_actions.rs
  - zircon_hub/src/tauri_app/runtime_state/learn_actions.rs
  - zircon_hub/src/tauri_app/runtime_state/output_actions.rs
  - zircon_hub/src/tauri_app/runtime_state/project_actions.rs
  - zircon_hub/src/tauri_app/runtime_state/project_delivery_actions.rs
  - zircon_hub/src/tauri_app/runtime_state/quick_actions.rs
  - zircon_hub/src/tauri_app/runtime_state/scoped_views.rs
  - zircon_hub/src/tauri_app/runtime_state/settings_actions.rs
  - zircon_hub/src/tauri_app/view_model.rs
  - zircon_hub/src/tauri_app/view_model/action_history.rs
  - zircon_hub/src/tauri_app/view_model/catalog.rs
  - zircon_hub/src/tauri_app/view_model/coming_soon.rs
  - zircon_hub/src/tauri_app/view_model/localized.rs
  - zircon_hub/src/tauri_app/view_model/settings_dto.rs
  - zircon_hub/src/tauri_app/view_model/source_engines.rs
  - zircon_hub/src/tauri_app/view_model/ui_text.rs
  - zircon_hub/web/src/App.tsx
  - zircon_hub/web/src/tauri/hubApi.ts
  - zircon_hub/web/src/tauri/projectTarget.ts
  - zircon_hub/web/src/pages/BuildsPage.tsx
  - zircon_hub/web/src/pages/CatalogPage.tsx
  - zircon_hub/web/src/pages/CloudPage.tsx
  - zircon_hub/web/src/pages/EditorPage.tsx
  - zircon_hub/web/src/pages/ProjectDetailPage.tsx
  - zircon_hub/web/src/pages/ProjectsDashboard.tsx
  - zircon_hub/web/src/pages/ProjectBrowserPage.tsx
  - zircon_hub/web/src/pages/SettingsPage.tsx
  - zircon_hub/web/src/pages/TeamPage.tsx
  - zircon_hub/web/src/pages/WorkspacePage.tsx
plan_sources:
  - .codex/plans/Zircon Hub 本地闭环 v1 功能实现设计.md
  - .codex/plans/Zircon Hub Tauri + ReactMUI 硬切换计划.md
  - .opencode/workflows/20260528_231820_026_优化hub[image Zircon Hub 响应式组件化重构计划.md]/hub-docs-contract-refresh/plan.md
  - .opencode/workflows/20260528_231820_026_优化hub[image Zircon Hub 响应式组件化重构计划.md]/hub-docs-contract-refresh/review-surface.md
tests:
  - cargo test --manifest-path zircon_hub/Cargo.toml --test hub_docs_contract -- --nocapture
  - cargo test --manifest-path zircon_hub/Cargo.toml --test project_workflow_contract -- --nocapture
  - cargo test --manifest-path zircon_hub/Cargo.toml --test ui_selected_project_catalog_contract -- --nocapture
  - cargo test --manifest-path zircon_hub/Cargo.toml --test ui_selected_project_runtime_contract -- --nocapture
  - cargo test --manifest-path zircon_hub/Cargo.toml --test project_page_copy_contract -- --nocapture
  - npm run typecheck
  - npm run build
doc_type: workflow-detail
---

# Hub actionable pages

This document owns the React/MUI actionable page surface for the local Hub v1 implementation. It records how Builds, Editor, Assets, Plugins, Learn, Cloud, Team, Settings, and fallback Workspace pages use the Tauri runtime state, typed action payloads, localized DTOs, and reserved disabled capabilities without reintroducing remote service promises.

## Scope Rules

`HubRuntimeSession` remains the single action router. React pages call the one frontend dispatcher in `web/src/tauri/hubApi.ts`, which sends `{ actionId, targetId?, payload? }` to `hub_action`; Rust parses that request in `action_request.rs` and routes it through `src/tauri_app/runtime_state.rs`.

Selected-project workflows resolve through Rust state rather than page-local fallback rules. Project workflow and management pages send the selected project with `projectTargetPayload(project)` or `quickActionProjectTargetPayload(project)`, so build, package, install, open-editor, pin, unpin, remove, request-delete, cancel-delete, and confirm-delete carry `{ projectId, projectPath }` when a selected project is visible. Rust target resolution prefers `projectPath`, then the stable project id, then legacy `targetId` compatibility.

Dashboard-style quick actions may still run without a payload when no selected project exists; that is the only latest-recent fallback surface. Detail, Editor, Builds, Catalog, Cloud, and Team panels pass the selected project payload when one exists so the backend resolves the same target the user sees.

## Builds And Editor

`web/src/pages/BuildsPage.tsx` and `web/src/pages/EditorPage.tsx` compose the local workflow surfaces from DTOs. They render selected-project state, Source Engine rows, task feedback, history rows, and output-folder actions with shared React components, while Rust owns the actual action preparation and completion.

`src/tauri_app/runtime_state/build_actions.rs` prepares and completes `tools/zircon_build.py --targets editor,runtime` work. `src/tauri_app/runtime_state/editor_launch_actions.rs` prepares and completes selected-project or empty-editor launches. `src/tauri_app/runtime_state/action_tasks.rs` marks build, package, install, and open-editor as background actions so Tauri can return a running view model immediately and later publish `hub-state-changed`.

Builds history renders `HubActionHistoryItem.detailRows` directly. The DTO rows include localized target, finish time, output directory, recovery hint, command line, and log excerpt, so the page does not rebuild business copy or punctuation from raw runtime fields. Source Engine build history follows the same rule through `src/tauri_app/view_model/source_engines.rs`, including the backend-owned `secondaryDetail` line for command/log display.

## Catalog And Learn

`web/src/pages/CatalogPage.tsx` is the shared Assets, Plugins, and Learn route. Runtime catalog refresh is owned by `src/tauri_app/runtime_state/scoped_views.rs` plus the discovery modules under `src/assets`, `src/plugins`, and `src/learn`. Selected-project entries are discovered before Source Engine entries, and React filters only the already-projected DTO rows.

Assets and Plugins remain read-only in v1. Asset import, plugin install, plugin enable/disable, and marketplace download are represented by `comingSoon` DTO entries with `disabled = true`, localized title/detail/status/meta, and stable categories. Catalog, Editor plugin, Cloud, and Team pages render those entries without composing category/status language locally.

Learn is the only catalog page with a real row action. It sends `{ resourceId, path }` through `open-resource`; `src/tauri_app/runtime_state/learn_actions.rs` accepts only files that are present in the current Learn catalog and can fall back from a stale row id to the supplied catalog path without opening arbitrary documents.

## Cloud Local Delivery

The stable `cloud` route is a local delivery center in v1. `web/src/pages/CloudPage.tsx` renders Packages, Installs, and Reserved Services from local settings, action history, and `comingSoon` entries. It does not expose account login, remote sync, remote repositories, hosted build workers, or marketplace services as active capabilities.

Package and install actions route to `src/tauri_app/runtime_state/project_delivery_actions.rs`. Packaging copies the selected project to the configured package output and writes `zircon-package.toml`; installing ensures a package first, then copies it into the configured local device install directory. Package and install history lists read from `HubActionHistoryItem` and show the backend `outputDir` line before opening the recorded output folder.

Reserved services are disabled rows from `comingSoon`. They keep a visible future-capability category and recovery-neutral disabled state while making the local-only contract explicit.

## Team

`web/src/pages/TeamPage.tsx` renders local Git identity, contributors, recent Hub actions, and disabled collaboration rows from `HubViewModel`. `src/team/local_git.rs` discovers repository state locally. Runtime scoped refresh prefers the selected project's Git repository and falls back to the Source Engine repository when no selected project repository is available.

Team invitation, permissions, and remote collaboration are not implemented in v1. They stay visible only as disabled `comingSoon` rows.

## Settings

`web/src/pages/SettingsPage.tsx` renders persisted settings and the editable `settingsDraft` DTO from `src/tauri_app/view_model/settings_dto.rs`. The page edits local tool paths, default project/source/build/device directories, build profile, jobs, and language. It displays localized option labels while submitting stable machine values such as `debug`, `release`, `Chinese`, and `English`.

`browse-settings-folder` sends `{ field, initialDir, settings }` and mutates only `settingsDraft`. Native folder picker titles use the draft language. `save-settings` applies the typed settings payload, persists Hub config, registers or updates the Source Engine from the saved source directory, and refreshes Assets, Plugins, Learn, and Team catalogs.

Configuration health is computed from actual settings rows for Python, Cargo, Rustup, and local directory defaults. Completion percentage, row labels, row details, and tones are DTO-owned; the React page only places the rows.

## Contracts And Handoff

The current contract surface is intentionally split by ownership:

- `project_workflow_contract.rs` locks typed IPC shape, runtime routing, background action owners, settings draft routing, and workflow page dispatch.
- `ui_selected_project_runtime_contract.rs` locks selected-project DTO projection and passive page consumption across Projects, Browser, Detail, Editor, Builds, and Cloud.
- `ui_selected_project_catalog_contract.rs` locks catalog discovery scope, Learn `open-resource`, Team Git repository preference, Cloud selected-project surfaces, and Catalog quick-action target payloads.
- `project_page_copy_contract.rs` locks DTO-owned page copy, localized task/action-history fields, template option labels, Catalog search placeholder punctuation, Builds `detailRows`, and Cloud output-directory history rows.
- `hub_docs_contract.rs` keeps this document aligned with the active React/Tauri ownership model and rejects obsolete page-owner references.

## Docs Refresh Handoff

`hub-docs-contract-refresh` keeps actionable page rules aligned with the current component and runtime contracts. The handoff is documentation-only: it does not add asset import, plugin installation, remote account lifecycle, remote sync, remote repository, or new build/install process paths. Future work that implements those capabilities must add new runtime owners, typed actions, DTOs, tests, and visual evidence instead of changing disabled rows into implied behavior.
