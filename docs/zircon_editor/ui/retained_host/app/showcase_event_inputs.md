---
related_code:
  - zircon_editor/src/ui/retained_host/app/showcase_event_inputs.rs
  - zircon_editor/src/ui/retained_host/app/showcase_event_inputs/action.rs
  - zircon_editor/src/ui/retained_host/app/showcase_event_inputs/edit.rs
  - zircon_editor/src/ui/retained_host/app/pane_surface_actions/component_showcase.rs
implementation_files:
  - zircon_editor/src/ui/retained_host/app/showcase_event_inputs.rs
  - zircon_editor/src/ui/retained_host/app/showcase_event_inputs/action.rs
  - zircon_editor/src/ui/retained_host/app/showcase_event_inputs/edit.rs
plan_sources:
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - user: 2026-06-18 editor UI architecture implementation, feature first and tests deferred
tests:
  - cargo fmt -p zircon_editor
  - cargo fmt -p zircon_editor --check
  - app showcase edit-input ownership scan
  - app showcase action-input ownership scan
  - cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never
doc_type: module-detail
---

# Component Showcase Event Inputs

`app/showcase_event_inputs.rs` is the structural entry for pure Component Showcase demo input conversion. It re-exports action-triggered and edit-string conversion, owns shared action-id normalization, shared virtual/paged list defaults, and the common option-selection helper.

## Action Inputs

`app/showcase_event_inputs/action.rs` owns action-to-demo-input mapping. It maps stable showcase action ids to `UiComponentShowcaseDemoEventInput`, including field value demos, toggles, option selection, reference drops, collection edits, list events, virtual-list defaults, paged-list defaults, world-surface changes, asset field binding suffix handling, and action-input regressions.

## Edit Payloads

`app/showcase_event_inputs/edit.rs` owns edit-string payload parsing. It converts committed edit text for context menus, virtual lists, paged lists, array rows, map rows, number fields, range fields, and fallback string values into typed `UiComponentShowcaseDemoEventInput` values.

The root re-exports `demo_input_for_showcase_edit(...)` for the host-side showcase dispatcher. The edit child imports the root action matching helpers and list/page defaults, but keeps the string parsing helpers and edit-focused regressions local.

## Boundary Rules

- Keep action click/activation demo mapping and `select_option(...)` in `app/showcase_event_inputs.rs`.
- Keep action click/activation demo mapping, transient bool events, reference drop inputs, collection action values, list events, world-surface action values, and action-input regressions in `app/showcase_event_inputs/action.rs`.
- Keep shared action-id normalization, shared virtual/paged list defaults, and `select_option(...)` in `app/showcase_event_inputs.rs`.
- Keep edit string parsing, row payload parsing, page/range request parsing, and edit-specific regressions in `app/showcase_event_inputs/edit.rs`.
- Keep host-state-aware dispatch, runtime binding lookup, status-line mutation, and presentation invalidation in `app/pane_surface_actions/component_showcase.rs`.

## Validation Notes

The 2026-06-18 edit-input split reduced `showcase_event_inputs.rs` from 645 lines to 349 lines. `showcase_event_inputs/edit.rs` is 306 lines and owns edit payload parsing plus the moved edit-input regressions.

Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app showcase edit-input ownership scan, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only. Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-18 action-input split reduced `showcase_event_inputs.rs` from 333 lines to 44 lines. `showcase_event_inputs/action.rs` is 296 lines and owns action id to demo input mapping plus the moved action-input regressions; `showcase_event_inputs/edit.rs` is 298 lines after explicit dependency imports.

Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app showcase action-input ownership scan, scoped `git diff --check`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only. Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.
