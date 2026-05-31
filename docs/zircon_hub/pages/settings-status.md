---
related_code:
  - zircon_hub/src/app/view_model.rs
  - zircon_hub/src/app/binding.rs
  - zircon_hub/src/app/runtime.rs
  - zircon_hub/ui/app.slint
  - zircon_hub/ui/shared.slint
  - zircon_hub/ui/settings.slint
  - zircon_hub/ui/settings_page_components.slint
implementation_files:
  - zircon_hub/src/app/view_model.rs
  - zircon_hub/src/app/binding.rs
  - zircon_hub/ui/shared.slint
  - zircon_hub/ui/settings.slint
  - zircon_hub/ui/settings_page_components.slint
plan_sources:
  - user: 2026-05-28 继续完善hub / hub-pages-settings-status milestone
  - .opencode/workflows/20260528_190023_866_继续完善hub/workflow.xml
  - .opencode/workflows/20260528_190023_866_继续完善hub/hub-pages-settings-status/plan.md
  - .opencode/workflows/20260528_190023_866_继续完善hub/hub-pages-settings-status/decomposition.md
  - .opencode/workflows/20260528_231820_026_优化hub[image Zircon Hub 响应式组件化重构计划.md]/hub-docs-contract-refresh/plan.md
  - .opencode/workflows/20260528_231820_026_优化hub[image Zircon Hub 响应式组件化重构计划.md]/hub-docs-contract-refresh/review-surface.md
tests:
  - zircon_hub/src/app/view_model.rs
  - zircon_hub/tests/ui_workspace_layout_contract.rs
  - zircon_hub/tests/ui_panel_slot_contract.rs
  - zircon_hub/tests/ui_inputs_contract.rs
  - cargo fmt -p zircon_hub --check
  - cargo test -p zircon_hub app::view_model --locked -- --nocapture
  - cargo test -p zircon_hub --test ui_workspace_layout_contract --locked -- --nocapture
  - cargo test -p zircon_hub --test ui_panel_slot_contract --locked -- --nocapture
  - cargo test -p zircon_hub --test ui_inputs_contract --locked -- --nocapture
  - cargo check -p zircon_hub --locked
  - zircon_hub/tests/hub_docs_contract.rs
  - cargo test -p zircon_hub --test hub_docs_contract --locked --jobs 1 -- --nocapture
doc_type: module-detail
---

# Hub Settings Status Page

The Settings page is the repair surface for Hub configuration. It projects a status-driven view from the same `HubSnapshot` used by the rest of Hub, so rows can explain the current selected project, Source Engine, toolchain, project root, Source checkout, staged output, and device-install root without guessing page-local context.

## Projection Contract

`view_model::settings_statuses` derives rows from `HubSnapshot` rather than raw `HubSettings`. This lets the Settings page distinguish a selected project from latest-recent fallback, stale selections, project-bound Source Engines, active Source Engines, unbound projects, unavailable engine bindings, and a completely missing Source Engine registry.

Each `SettingStatusData` row carries:

- `label`, `detail`, and `scope` for visible status copy.
- `state` as `ok`, `warn`, `error`, or `info` for the status icon and badge tone.
- `disabled-reason` when the row needs recovery copy instead of silently hiding a missing path or binding.
- `action-id`, `action-label`, and `actionable` for row-level repair affordances.

Toolchain rows still validate path-looking executable values with filesystem existence and treat bare commands as PATH-resolved. Directory rows classify empty paths as errors, missing but creatable paths as warnings, and existing directories as ready. Source checkout rows use `save-settings` as the repair action because saving registers the checkout as a Source Engine through `HubRuntime::save_settings`.

## Slint Ownership

`settings.slint` remains a page composition file. It forwards the status model to `SettingsConfigurationHealthPanel`, owns only responsive section sizing, and routes row actions to existing callbacks:

- `save-settings` calls the existing save/register runtime path.
- `browse-project-location`, `browse-output`, and `browse-device-install` call the existing folder picker targets.
- The source checkout row currently uses `save-settings`, keeping registration centralized instead of inventing a second runtime path.

`settings_page_components.slint` owns `SettingStatusRow`. The row renders `detail`, combines `scope` and `disabled-reason` into supporting text, uses the state-specific status icon, and shows an arrow only when the row is actionable.

## Runtime Boundary

Settings does not register with `zircon_runtime` and does not move behavior into `zircon_editor`. `HubRuntime::save_settings` remains the owner for syncing UI values, registering or updating the Source Engine from settings, refreshing source-scoped views, persisting Hub config, and setting user-visible task status.

## Validation Evidence

This milestone's acceptance commands are the scoped Hub formatting, view-model, static UI contract, and package check commands listed in the document header. Record the final command results in the workflow executor summary when this milestone closes.

## Docs Refresh Handoff

`hub-docs-contract-refresh` treats Settings status as a snapshot-derived page, not a static settings form. `HubSnapshot` is the source for `SettingStatusData`; row action ids such as `save-settings` remain repair affordances that route through existing runtime callbacks. `hub_docs_contract.rs` keeps those ownership statements linked from the broader Hub docs so acceptance validation can inspect Settings without inventing a second status model.
