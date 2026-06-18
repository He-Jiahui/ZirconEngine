---
related_code:
  - zircon_editor/src/ui/retained_host/app.rs
  - zircon_editor/src/ui/retained_host/app/pane_surface_actions.rs
  - zircon_editor/src/ui/retained_host/app/pane_surface_actions/click.rs
  - zircon_editor/src/ui/retained_host/app/pane_surface_actions/component_showcase.rs
  - zircon_editor/src/ui/retained_host/app/pane_surface_actions/edit.rs
  - zircon_editor/src/ui/retained_host/app/pane_surface_actions/routing.rs
  - zircon_editor/src/ui/retained_host/app/pane_surface_actions/ui_asset_detail.rs
  - zircon_editor/src/ui/retained_host/app/pane_surface_actions/workbench_surface.rs
  - zircon_editor/src/ui/retained_host/app/showcase_event_inputs.rs
  - zircon_editor/src/ui/retained_host/app/callback_wiring.rs
implementation_files:
  - zircon_editor/src/ui/retained_host/app/pane_surface_actions.rs
  - zircon_editor/src/ui/retained_host/app/pane_surface_actions/click.rs
  - zircon_editor/src/ui/retained_host/app/pane_surface_actions/component_showcase.rs
  - zircon_editor/src/ui/retained_host/app/pane_surface_actions/edit.rs
  - zircon_editor/src/ui/retained_host/app/pane_surface_actions/routing.rs
  - zircon_editor/src/ui/retained_host/app/pane_surface_actions/ui_asset_detail.rs
  - zircon_editor/src/ui/retained_host/app/pane_surface_actions/workbench_surface.rs
  - zircon_editor/src/ui/retained_host/app/showcase_event_inputs.rs
plan_sources:
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - user: 2026-06-18 editor UI architecture implementation, feature first and tests deferred
tests:
  - cargo fmt -p zircon_editor
  - cargo fmt -p zircon_editor --check
  - app pane-surface showcase ownership scan
  - app pane-surface Workbench ownership scan
  - app pane-surface generic click/edit ownership scan
  - scoped git diff --check
  - cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never
doc_type: module-detail
---

# Pane Surface Host Actions

`app/pane_surface_actions.rs` is the structural entry for retained-host callback routing for pane surface controls. Its child modules focus the callback source window, route componentized Workbench control clicks/options/edits, forward module-plugin and Build/Export actions to their feature owners, dispatch performance timeline actions, resolve built-in pane surface/template bindings, apply dispatch results, and parse UI Asset detail edit binding ids.

## Generic Click And Edit

`app/pane_surface_actions/click.rs` owns generic pane-surface click dispatch. It focuses the callback source window, asks the Workbench surface bridge first, forwards module-plugin, Build/Export, and profiling actions to their owners, then falls back to built-in pane surface and template bindings before applying host dispatch results.

`app/pane_surface_actions/edit.rs` owns generic pane-surface edit dispatch. It routes Workbench and UI Asset detail edit callbacks, resolves built-in pane bridge binding ids, falls back to template bindings, and formats dispatch errors into the retained status line.

`app/pane_surface_actions/routing.rs` owns surface action routing predicates that are shared by the generic click path, including Build/Export wizard action detection and its regression coverage.

`app/pane_surface_actions/ui_asset_detail.rs` owns parsing for UI Asset detail edit binding ids. The edit dispatcher depends on this parser but does not own the binding-id string grammar.

## Component Showcase

`app/pane_surface_actions/component_showcase.rs` owns Component Showcase and Material Lab demo dispatch. It maps callback action ids to showcase runtime bindings, converts user input into `UiComponentShowcaseDemoEventInput`, handles drag/context/option/edit showcase events, routes Material Lab feedback, advances virtual-list/paged-list demo state, and invalidates presentation data when demo state changes.

`app/showcase_event_inputs.rs` remains the pure input mapping table for showcase controls. The pane-surface showcase child owns host-state-aware choices such as active drag payload drops, virtual list paging, runtime projection lookup, and status-line mutation.

## Workbench Surface

`app/pane_surface_actions/workbench_surface.rs` owns componentized Workbench surface forwarding for pane callbacks. It verifies that the active activity window is the Workbench document, resolves action ids to Workbench binding ids, dispatches popup cancellation, menu item selection, generic Workbench control clicks, option selection, edit events, and command palette commit events against `workbench_window_bridge`.

The generic click/edit children keep the high-level pane-surface callback entry points and ask this child first when a callback may belong to the componentized Workbench shell.

## Boundary Rules

- Keep `app/pane_surface_actions.rs` as a structural module entry that only declares the pane-surface action family and imports the parent app scope required by legacy child modules.
- Keep generic pane surface control click routing, feature forwarding, built-in pane surface fallback, template binding fallback, and result application in `app/pane_surface_actions/click.rs`.
- Keep generic pane surface edit routing, Workbench/UI Asset edit forwarding, built-in pane bridge lookup, template binding fallback, and edit error formatting in `app/pane_surface_actions/edit.rs`.
- Keep shared route predicates such as Build/Export action detection in `app/pane_surface_actions/routing.rs`.
- Keep UI Asset detail edit binding parsing in `app/pane_surface_actions/ui_asset_detail.rs`.
- Keep Component Showcase and Material Lab host-state dispatch in `app/pane_surface_actions/component_showcase.rs`.
- Keep componentized Workbench control click/option/edit forwarding in `app/pane_surface_actions/workbench_surface.rs`.
- Keep static demo input mapping helpers in `app/showcase_event_inputs.rs`; do not move host-state mutation or runtime projection lookup into that pure mapping module.
- Keep feature-specific action execution in their owners (`module_plugin_actions`, `build_export_actions`, performance timeline); pane surface routing should only identify and forward those actions.

## Validation Notes

The 2026-06-18 showcase split reduced `pane_surface_actions.rs` from 587 lines to 290 lines. `pane_surface_actions/component_showcase.rs` is 301 lines and owns Component Showcase/Material Lab action id resolution, demo event input selection, runtime binding lookup, virtual-list and paged-list state progression, status-line handling, and presentation invalidation.

Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app pane-surface showcase ownership scan, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only. Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-18 Workbench surface split reduced `pane_surface_actions.rs` from 276 lines to 165 lines. `pane_surface_actions/workbench_surface.rs` is 115 lines and owns componentized Workbench control click/option/edit forwarding, Workbench binding-id resolution, popup cancellation, menu selection, and command palette commit dispatch.

Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app pane-surface Workbench ownership scan, scoped `git diff --check`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only. Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-19 generic click/edit split reduced `pane_surface_actions.rs` from 165 lines to 7 lines. `pane_surface_actions/click.rs` is 53 lines and owns generic click routing, feature forwarding, fallback binding dispatch, and result application. `pane_surface_actions/edit.rs` is 55 lines and owns generic edit routing, Workbench/UI Asset edit forwarding, built-in/template binding dispatch, and edit error formatting. `pane_surface_actions/routing.rs` is 28 lines and owns shared route predicates. `pane_surface_actions/ui_asset_detail.rs` is 33 lines and owns UI Asset detail edit binding parsing.

Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app pane-surface generic click/edit ownership scan, scoped `git diff --check`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only (`zircon_runtime` 142 warnings, `zircon_editor` 63 warnings). Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.
