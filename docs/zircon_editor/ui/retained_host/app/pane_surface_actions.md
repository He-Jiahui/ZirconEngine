---
related_code:
  - zircon_editor/src/ui/retained_host/app.rs
  - zircon_editor/src/ui/retained_host/app/pane_surface_actions.rs
  - zircon_editor/src/ui/retained_host/app/pane_surface_actions/click.rs
  - zircon_editor/src/ui/retained_host/app/pane_surface_actions/component_showcase.rs
  - zircon_editor/src/ui/retained_host/app/pane_surface_actions/component_showcase/activation.rs
  - zircon_editor/src/ui/retained_host/app/pane_surface_actions/component_showcase/bindings.rs
  - zircon_editor/src/ui/retained_host/app/pane_surface_actions/component_showcase/context.rs
  - zircon_editor/src/ui/retained_host/app/pane_surface_actions/component_showcase/drag.rs
  - zircon_editor/src/ui/retained_host/app/pane_surface_actions/component_showcase/edit.rs
  - zircon_editor/src/ui/retained_host/app/pane_surface_actions/component_showcase/events.rs
  - zircon_editor/src/ui/retained_host/app/pane_surface_actions/component_showcase/inputs.rs
  - zircon_editor/src/ui/retained_host/app/pane_surface_actions/component_showcase/material_lab.rs
  - zircon_editor/src/ui/retained_host/app/pane_surface_actions/component_showcase/option.rs
  - zircon_editor/src/ui/retained_host/app/pane_surface_actions/edit.rs
  - zircon_editor/src/ui/retained_host/app/pane_surface_actions/routing.rs
  - zircon_editor/src/ui/retained_host/app/pane_surface_actions/ui_asset_detail.rs
  - zircon_editor/src/ui/retained_host/app/pane_surface_actions/workbench_surface.rs
  - zircon_editor/src/ui/retained_host/app/pane_surface_actions/workbench_surface/control.rs
  - zircon_editor/src/ui/retained_host/app/pane_surface_actions/workbench_surface/edit.rs
  - zircon_editor/src/ui/retained_host/app/pane_surface_actions/workbench_surface/option.rs
  - zircon_editor/src/ui/retained_host/app/showcase_event_inputs.rs
  - zircon_editor/src/ui/retained_host/app/callback_wiring.rs
implementation_files:
  - zircon_editor/src/ui/retained_host/app/pane_surface_actions.rs
  - zircon_editor/src/ui/retained_host/app/pane_surface_actions/click.rs
  - zircon_editor/src/ui/retained_host/app/pane_surface_actions/component_showcase.rs
  - zircon_editor/src/ui/retained_host/app/pane_surface_actions/component_showcase/activation.rs
  - zircon_editor/src/ui/retained_host/app/pane_surface_actions/component_showcase/bindings.rs
  - zircon_editor/src/ui/retained_host/app/pane_surface_actions/component_showcase/context.rs
  - zircon_editor/src/ui/retained_host/app/pane_surface_actions/component_showcase/drag.rs
  - zircon_editor/src/ui/retained_host/app/pane_surface_actions/component_showcase/edit.rs
  - zircon_editor/src/ui/retained_host/app/pane_surface_actions/component_showcase/events.rs
  - zircon_editor/src/ui/retained_host/app/pane_surface_actions/component_showcase/inputs.rs
  - zircon_editor/src/ui/retained_host/app/pane_surface_actions/component_showcase/material_lab.rs
  - zircon_editor/src/ui/retained_host/app/pane_surface_actions/component_showcase/option.rs
  - zircon_editor/src/ui/retained_host/app/pane_surface_actions/edit.rs
  - zircon_editor/src/ui/retained_host/app/pane_surface_actions/routing.rs
  - zircon_editor/src/ui/retained_host/app/pane_surface_actions/ui_asset_detail.rs
  - zircon_editor/src/ui/retained_host/app/pane_surface_actions/workbench_surface.rs
  - zircon_editor/src/ui/retained_host/app/pane_surface_actions/workbench_surface/control.rs
  - zircon_editor/src/ui/retained_host/app/pane_surface_actions/workbench_surface/edit.rs
  - zircon_editor/src/ui/retained_host/app/pane_surface_actions/workbench_surface/option.rs
  - zircon_editor/src/ui/retained_host/app/showcase_event_inputs.rs
plan_sources:
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - user: 2026-06-18 editor UI architecture implementation, feature first and tests deferred
tests:
  - cargo fmt -p zircon_editor
  - cargo fmt -p zircon_editor --check
  - app pane-surface showcase ownership scan
  - app pane-surface Workbench ownership scan
  - app pane-surface Workbench subowner ownership scan
  - app pane-surface generic click/edit ownership scan
  - app pane-surface component-showcase subowner ownership scan
  - app pane-surface component-showcase callback subowner ownership scan
  - app retained-host owner visibility compile-boundary scan
  - scoped git diff --check
  - cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never
doc_type: module-detail
---

# Pane Surface Host Actions

`app/pane_surface_actions.rs` is the structural entry for retained-host callback routing for pane surface controls. Its child modules focus the callback source window, route componentized Workbench control clicks/options/edits, forward module-plugin and Build/Export actions to their feature owners, dispatch performance timeline actions, resolve built-in pane surface/template bindings, apply dispatch results, and parse UI Asset detail edit binding ids.

## Generic Click And Edit

`app/pane_surface_actions/click.rs` owns generic pane-surface click dispatch. It focuses the callback source window, asks the Workbench surface bridge first, forwards module-plugin, Build/Export, and profiling actions to their owners, then falls back to built-in pane surface and template bindings before applying host dispatch results. Template table selection must retain the projected `pane_id`: a uniquely matched visible active drawer invalidates through the scoped shell-content transaction, while document/floating panes, missing committed state, and ambiguous matches explicitly fall back to full presentation invalidation.

`app/pane_surface_actions/edit.rs` owns generic pane-surface edit dispatch. It routes Workbench and UI Asset detail edit callbacks, resolves built-in pane bridge binding ids, falls back to template bindings, and formats dispatch errors into the retained status line.

`app/pane_surface_actions/routing.rs` owns surface action routing predicates that are shared by the generic click path, including Build/Export wizard action detection and its regression coverage.

`app/pane_surface_actions/ui_asset_detail.rs` owns parsing for UI Asset detail edit binding ids. The edit dispatcher depends on this parser but does not own the binding-id string grammar.

## Component Showcase

`app/pane_surface_actions/component_showcase.rs` is the structural Component Showcase callback entry. `component_showcase/activation.rs`, `drag.rs`, `edit.rs`, `context.rs`, and `option.rs` own the respective callback entry points. They focus the callback source window, resolve action ids through the showcase binding child, build the appropriate `UiComponentShowcaseDemoEventInput`, and forward to the runtime event child.

`app/pane_surface_actions/component_showcase/bindings.rs` owns Component Showcase action-id to runtime binding-id resolution, including `UiComponentShowcase/*` camel-case normalization and Material Lab passthrough detection.

`app/pane_surface_actions/component_showcase/events.rs` owns showcase runtime event application. It loads the showcase runtime if needed, looks up runtime bindings, applies demo binding payloads, writes status-line feedback, and invalidates presentation data when demo state changes.

`app/pane_surface_actions/component_showcase/inputs.rs` owns host-state-aware showcase input construction. It consumes active reference drag payload drops, advances virtual-list visible ranges, advances paged-list state, and falls back to the static showcase input table.

`app/pane_surface_actions/component_showcase/material_lab.rs` owns Material Lab runtime binding validation and status-line feedback.

`app/showcase_event_inputs.rs` remains the pure input mapping table for showcase controls. The pane-surface showcase child owns host-state-aware choices such as active drag payload drops, virtual list paging, runtime projection lookup, and status-line mutation.

## Workbench Surface

`app/pane_surface_actions/workbench_surface.rs` is the structural componentized Workbench surface forwarding entry for pane callbacks. `workbench_surface/control.rs` owns Workbench control click forwarding, including active document checks, action-id to binding-id resolution, popup cancellation, menu item selection, binding dispatch, and generic control click dispatch. `workbench_surface/option.rs` owns option-selected forwarding. `workbench_surface/edit.rs` owns edit forwarding plus command palette commit dispatch.

The generic click/edit children keep the high-level pane-surface callback entry points and ask this child first when a callback may belong to the componentized Workbench shell.

## Boundary Rules

- Keep `app/pane_surface_actions.rs` as a structural module entry that only declares the pane-surface action family and imports the parent app scope required by legacy child modules.
- Keep generic pane surface control click routing, feature forwarding, built-in pane surface fallback, template binding fallback, and result application in `app/pane_surface_actions/click.rs`.
- Keep template table selection target preservation in `app/pane_surface_actions/click.rs`; do not widen a known active drawer pane to host-wide presentation, and do not claim shell-content coverage for document/floating panes.
- Keep generic pane surface edit routing, Workbench/UI Asset edit forwarding, built-in pane bridge lookup, template binding fallback, and edit error formatting in `app/pane_surface_actions/edit.rs`.
- Keep shared route predicates such as Build/Export action detection in `app/pane_surface_actions/routing.rs`.
- Keep UI Asset detail edit binding parsing in `app/pane_surface_actions/ui_asset_detail.rs`.
- Keep `app/pane_surface_actions/component_showcase.rs` as the structural Component Showcase callback entry.
- Keep Component Showcase activate/drag/edit/context/option callback flow in the matching `component_showcase/{activation,drag,edit,context,option}.rs` files.
- Keep Component Showcase action-id normalization and runtime binding lookup in `app/pane_surface_actions/component_showcase/bindings.rs`.
- Keep showcase runtime event application/status/invalidation in `app/pane_surface_actions/component_showcase/events.rs`.
- Keep host-state-aware showcase input generation in `app/pane_surface_actions/component_showcase/inputs.rs`.
- Keep Material Lab feedback handling in `app/pane_surface_actions/component_showcase/material_lab.rs`.
- Keep `app/pane_surface_actions/workbench_surface.rs` as the structural Workbench surface forwarding entry.
- Keep componentized Workbench control click forwarding in `app/pane_surface_actions/workbench_surface/control.rs`.
- Keep componentized Workbench option-selected forwarding in `app/pane_surface_actions/workbench_surface/option.rs`.
- Keep componentized Workbench edit and command palette commit forwarding in `app/pane_surface_actions/workbench_surface/edit.rs`.
- Keep static demo input mapping helpers in `app/showcase_event_inputs.rs`; do not move host-state mutation or runtime projection lookup into that pure mapping module.
- Keep feature-specific action execution in their owners (`module_plugin_actions`, `build_export_actions`, performance timeline); pane surface routing should only identify and forward those actions.

## Validation Notes

The 2026-06-18 showcase split reduced `pane_surface_actions.rs` from 587 lines to 290 lines. `pane_surface_actions/component_showcase.rs` is 301 lines and owns Component Showcase/Material Lab action id resolution, demo event input selection, runtime binding lookup, virtual-list and paged-list state progression, status-line handling, and presentation invalidation.

Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app pane-surface showcase ownership scan, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only. Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-18 Workbench surface split reduced `pane_surface_actions.rs` from 276 lines to 165 lines. `pane_surface_actions/workbench_surface.rs` is 115 lines and owns componentized Workbench control click/option/edit forwarding, Workbench binding-id resolution, popup cancellation, menu selection, and command palette commit dispatch.

Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app pane-surface Workbench ownership scan, scoped `git diff --check`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only. Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-19 generic click/edit split reduced `pane_surface_actions.rs` from 165 lines to 7 lines. `pane_surface_actions/click.rs` is 53 lines and owns generic click routing, feature forwarding, fallback binding dispatch, and result application. `pane_surface_actions/edit.rs` is 55 lines and owns generic edit routing, Workbench/UI Asset edit forwarding, built-in/template binding dispatch, and edit error formatting. `pane_surface_actions/routing.rs` is 28 lines and owns shared route predicates. `pane_surface_actions/ui_asset_detail.rs` is 33 lines and owns UI Asset detail edit binding parsing.

Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app pane-surface generic click/edit ownership scan, scoped `git diff --check`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only (`zircon_runtime` 142 warnings, `zircon_editor` 63 warnings). Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-19 Component Showcase subowner split reduced `pane_surface_actions/component_showcase.rs` from 283 lines to 97 lines. `component_showcase/bindings.rs` is 65 lines and owns binding-id resolution/normalization. `component_showcase/events.rs` is 52 lines and owns runtime event application/status/invalidation. `component_showcase/inputs.rs` is 66 lines and owns host-state-aware demo input derivation. `component_showcase/material_lab.rs` is 28 lines and owns Material Lab feedback.

Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app pane-surface component-showcase subowner ownership scan, and `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-ui-owner-split-0619 --message-format short --color never`, which passed with existing warning noise only (`zircon_runtime` 142 warnings, `zircon_editor` 65 warnings). Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-19 Workbench surface subowner split reduced `pane_surface_actions/workbench_surface.rs` from 115 lines to a 3-line structural entry. `workbench_surface/control.rs` is 63 lines and owns control click forwarding. `workbench_surface/option.rs` is 25 lines and owns option-selected forwarding. `workbench_surface/edit.rs` is 35 lines and owns edit/command-palette commit forwarding.

Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app pane-surface Workbench subowner ownership scan, and scoped `git diff --check` (only the existing CRLF conversion warning appeared). Focused `cargo check` was not rerun for this slice because independent Cargo/rustc processes were active; full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-19 retained-host owner visibility compile-boundary correction widened the Workbench surface forwarding methods in `workbench_surface/control.rs`, `workbench_surface/option.rs`, and `workbench_surface/edit.rs` to `pub(in crate::ui::retained_host::app)`. The methods remain app-internal, but sibling pane-surface owners can call them from generic click/edit and Component Showcase option routing after the Workbench subowner split. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app retained-host owner visibility compile-boundary scan, scoped `git diff --check`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-ui-owner-split-0619 --message-format short --color never`, which passed with existing warning noise only (`zircon_runtime` 141 warnings, `zircon_editor` 65 warnings). Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-19 Component Showcase callback subowner split reduced `pane_surface_actions/component_showcase.rs` from 97 lines to a 10-line structural entry. `component_showcase/activation.rs` is 15 lines and owns activation input dispatch, `drag.rs` is 21 lines and owns drag delta input selection, `edit.rs` is 17 lines and owns edit input parsing, `context.rs` is 25 lines and owns context popup dispatch, and `option.rs` is 26 lines and owns Workbench option forwarding plus showcase option input dispatch.

Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app pane-surface component-showcase callback subowner ownership scan, scoped `git diff --check`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-ui-owner-split-0619 --message-format short --color never`, which passed with existing warning noise only (`zircon_runtime` 141 warnings, `zircon_editor` 65 warnings). Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.
