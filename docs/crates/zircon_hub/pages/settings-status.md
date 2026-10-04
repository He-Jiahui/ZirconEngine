---
related_code:
  - zircon_hub/src/tauri_app/action_request.rs
  - zircon_hub/src/tauri_app/action_id.rs
  - zircon_hub/src/tauri_app/runtime_state.rs
  - zircon_hub/src/tauri_app/runtime_state/scoped_views.rs
  - zircon_hub/src/tauri_app/runtime_state/settings_actions.rs
  - zircon_hub/src/tauri_app/view_model.rs
  - zircon_hub/src/tauri_app/view_model/settings_dto.rs
  - zircon_hub/src/settings/hub_config.rs
  - zircon_hub/src/engines/validation.rs
  - zircon_hub/src/assets/catalog.rs
  - zircon_hub/src/learn/catalog.rs
  - zircon_hub/src/plugins/catalog.rs
  - zircon_hub/web/src/components/data/SettingsSection.tsx
  - zircon_hub/web/src/pages/SettingsPage.tsx
  - zircon_hub/web/src/settings/options.ts
  - zircon_hub/web/src/tauri/hubApi.ts
  - zircon_hub/web/src/types/hub.ts
implementation_files:
  - zircon_hub/src/tauri_app/runtime_state.rs
  - zircon_hub/src/tauri_app/runtime_state/scoped_views.rs
  - zircon_hub/src/tauri_app/runtime_state/settings_actions.rs
  - zircon_hub/src/tauri_app/view_model/settings_dto.rs
  - zircon_hub/src/assets/catalog.rs
  - zircon_hub/src/learn/catalog.rs
  - zircon_hub/src/plugins/catalog.rs
  - zircon_hub/web/src/components/data/SettingsSection.tsx
  - zircon_hub/web/src/pages/SettingsPage.tsx
  - zircon_hub/web/src/settings/options.ts
plan_sources:
  - user: 2026-06-07 Zircon Hub 本地闭环 v1 功能实现设计
  - .codex/plans/Zircon Hub 本地闭环 v1 功能实现设计.md
  - .opencode/workflows/20260528_231820_026_优化hub[image Zircon Hub 响应式组件化重构计划.md]/hub-docs-contract-refresh/plan.md
  - .opencode/workflows/20260528_231820_026_优化hub[image Zircon Hub 响应式组件化重构计划.md]/hub-docs-contract-refresh/review-surface.md
tests:
  - zircon_hub/src/tauri_app/runtime_state/settings_actions.rs
  - zircon_hub/src/tauri_app/view_model/settings_dto.rs
  - zircon_hub/src/assets/catalog.rs
  - zircon_hub/src/learn/catalog.rs
  - zircon_hub/tests/project_workflow_contract.rs
  - zircon_hub/tests/project_page_copy_contract.rs
  - zircon_hub/tests/ui_workspace_layout_contract.rs
  - zircon_hub/tests/ui_workspace_split_contract.rs
  - zircon_hub/tests/hub_docs_contract.rs
  - cargo test --manifest-path zircon_hub/Cargo.toml settings_draft_folder_changes_wait_for_save_settings -- --nocapture
  - cargo test --manifest-path zircon_hub/Cargo.toml save_settings_refreshes_source_scoped_catalogs_in_returned_view_model -- --nocapture
  - cargo test --manifest-path zircon_hub/Cargo.toml keeps_first_source_engine_root_before_fallback_limit -- --nocapture
  - cargo test --manifest-path zircon_hub/Cargo.toml settings_health_includes_rustup_path_status -- --nocapture
  - cargo test --manifest-path zircon_hub/Cargo.toml settings_health_checks_path_command_availability -- --nocapture
  - cargo test --manifest-path zircon_hub/Cargo.toml --test project_workflow_contract -- --nocapture
  - cargo test --manifest-path zircon_hub/Cargo.toml --test hub_docs_contract -- --nocapture
doc_type: module-detail
---

# Hub Settings Status Page

The Settings page is the local repair and configuration surface for Hub v1. It is now a React/MUI page backed by Tauri state: Rust projects persisted settings and the editable draft as `HubSettingsSummary`, React edits the draft through `update-settings-draft`, and only `save-settings` commits that draft to `hub.toml` and refreshes Source Engine catalogs.

## Projection Contract

`HubViewModel` exposes two settings DTOs:

- `settings` is the persisted configuration from `HubConfig.settings`.
- `settingsDraft` is the editable draft from `HubRuntimeSession.settings_draft`.

Both DTOs use `HubSettingsSummary` from `src/tauri_app/view_model/settings_dto.rs`. The summary carries tool paths, default local directories, build profile, jobs, language, localized field labels, localized option labels, and `HubSettingsHealthSummary`. `SettingsPage.tsx` reads `settingsDraft ?? settings`, so a folder-picker result or typed field edit can be visible before the user saves. Typed edits send `update-settings-draft` with `{ settings: draft }`, letting Rust recompute `settingsDraft.health` from the edited toolchain/path values without persisting `hub.toml`. This keeps the page local and recoverable: a failed folder pick or invalid setting reports task feedback without corrupting the persisted Hub configuration.

Configuration health is derived from actual settings values, not a fixed percentage. Toolchain rows cover Python, Cargo, and Rustup. Path-looking executable values must exist, bare commands are resolved against the current `PATH` and, on Windows, `PATHEXT`; missing commands are errors. Empty executable values are errors, existing directories are ready, and missing but creatable local directories are warnings. The completion percentage is computed from those rows, and the visible label/detail/tone are localized by the Rust DTO boundary.

## Action Ownership

`update-settings-draft` accepts the same typed `{ settings }` payload through `HubSettingsPayload`, applies it only to `HubRuntimeSession.settings_draft`, and returns a refreshed `HubViewModel` so Configuration Health follows the current editable draft before save. It does not persist `hub.toml`, register Source Engines, or refresh source-scoped catalogs.

`save-settings` accepts a typed `{ settings }` payload through `HubSettingsPayload`. If a payload is present, Rust applies it to the draft first; if no payload is present, it saves the existing draft. Saving then registers or updates the Source Engine from the configured source directory, refreshes source-scoped Assets, Plugins, Learn, and Team views, persists Hub config, resets `settingsDraft` to the saved settings, and emits localized task feedback. The returned `HubViewModel` already contains the refreshed `assets`, `plugins`, and `learnResources` rows from the newly configured Source root. Assets and Learn preserve the configured source directory ahead of fallback current-directory or compiled-repository roots before catalog limits truncate discovery results, so a dense fallback tree cannot hide the saved Source Engine checkout.

`discard-settings-draft` and `restore-default-settings` are draft-only actions. Discard copies the persisted settings back into the draft and recomputes health without writing `hub.toml`. Restore builds the default settings draft, recomputes health, and still waits for `save-settings` before persistence. These two actions give React explicit buttons for recovery without making page-local assumptions about default paths or language labels.

`save-settings` validates the configured Source Engine root before registration. `src/engines/validation.rs` requires a real source directory, a workspace `Cargo.toml` containing `zircon_runtime`, and `tools/zircon_build.py`; invalid roots return localized task feedback and leave the previous active Source Engine/catalog state intact. This keeps Settings as the only persistence gate and prevents a bad draft path from becoming an active engine by partial save.

`browse-settings-folder` accepts `{ field, initialDir, settings }`. The optional `settings` object lets the current React draft reach Rust before opening the native folder picker. Picker titles use the draft language. A selected folder updates only `settingsDraft`; cancellation and errors leave persisted settings untouched and return localized recovery guidance. This preserves the v1 rule that browse buttons are real local folder pickers while save remains the only persistence action.

## React Page Boundary

`SettingsPage.tsx` owns the route header, action buttons, status banner, summary metrics, tabs, and responsive main/sidebar split. `SettingsSection.tsx` owns the nested build-defaults, configuration-paths, path-defaults, advanced-configuration, configuration-health, and active-Source-Engine panels, so repeated form/list/tree/sidebar chrome stays in the data component layer instead of being rebuilt inside the page shell. Both files render localized titles, tabs, labels, options, health rows, and save/browse button text from the DTOs. Stable payload values remain machine values such as `debug`, `release`, `Chinese`, and `English`; visible labels come from `settings.text` and `web/src/settings/options.ts`.

The page sends all backend work through the shared action dispatcher:

- `update-settings-draft` with `{ settings: draft }` after typed field changes, so Rust recomputes draft health without saving.
- `save-settings` with `{ settings: draft }`.
- `browse-settings-folder` with the field id, initial directory, and current draft.
- `discard-settings-draft` and `restore-default-settings` for draft recovery.
- `select-engine` for Source Engine list selection.
- `show-page` for navigation back to Projects.

No Settings behavior is registered with `zircon_runtime`, and the page does not introduce a remote service or account model. Settings remains part of `zircon_hub` and configures only local paths, local toolchain values, build defaults, jobs, language, Source Engine registration, and health feedback.

## Validation Evidence

Focused Settings coverage lives in Rust unit tests and static Hub contracts. `settings_draft_folder_changes_wait_for_save_settings` verifies draft folder changes are visible without persistence until `save-settings`. `update_settings_draft_recomputes_health_without_persisting` verifies typed field edits update `settingsDraft.health` while persisted settings remain unchanged. `save_settings_refreshes_source_scoped_catalogs_in_returned_view_model` verifies the returned model includes Source-root Assets, Plugins, and Learn rows immediately after saving. `discover_asset_catalog_keeps_first_source_engine_root_before_fallback_limit` and `discover_learn_catalog_keeps_first_source_engine_root_before_fallback_limit` keep configured Source roots ahead of fallback roots under catalog limits. `settings_folder_picker_title_uses_current_language` and `save_settings_validation_errors_return_localized_view_model` cover localized picker and error feedback. `settings_health_includes_rustup_path_status` keeps Rustup in health computation, while `settings_health_checks_path_command_availability` proves a missing bare PATH command is reported as missing instead of being treated as ready. `project_workflow_contract.rs`, `project_page_copy_contract.rs`, `ui_workspace_layout_contract.rs`, and `ui_workspace_split_contract.rs` lock the typed payload path, DTO-owned copy, page shell, and `SettingsSection` panel ownership.

## Docs Refresh Handoff

`hub-docs-contract-refresh` now treats Settings as a React/Tauri settings-draft page. `HubSettingsSummary`, `settingsDraft`, `update-settings-draft`, `browse-settings-folder`, and `save-settings` are the acceptance surface for the current Hub v1 implementation. `hub_docs_contract.rs` keeps this document tied to the active Tauri/React ownership model and rejects the removed native UI status-row ownership model.
