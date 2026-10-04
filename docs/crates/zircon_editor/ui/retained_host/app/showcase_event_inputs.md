---
related_code:
  - zircon_editor/src/ui/retained_host/app/showcase_event_inputs.rs
  - zircon_editor/src/ui/retained_host/app/showcase_event_inputs/action.rs
  - zircon_editor/src/ui/retained_host/app/showcase_event_inputs/action/collections.rs
  - zircon_editor/src/ui/retained_host/app/showcase_event_inputs/action/fields.rs
  - zircon_editor/src/ui/retained_host/app/showcase_event_inputs/action/lists.rs
  - zircon_editor/src/ui/retained_host/app/showcase_event_inputs/action/references.rs
  - zircon_editor/src/ui/retained_host/app/showcase_event_inputs/action/selection.rs
  - zircon_editor/src/ui/retained_host/app/showcase_event_inputs/action/world_surface.rs
  - zircon_editor/src/ui/retained_host/app/showcase_event_inputs/action_tests.rs
  - zircon_editor/src/ui/retained_host/app/showcase_event_inputs/edit.rs
  - zircon_editor/src/ui/retained_host/app/showcase_event_inputs/edit/collections.rs
  - zircon_editor/src/ui/retained_host/app/showcase_event_inputs/edit/lists.rs
  - zircon_editor/src/ui/retained_host/app/showcase_event_inputs/edit/menu.rs
  - zircon_editor/src/ui/retained_host/app/showcase_event_inputs/edit/values.rs
  - zircon_editor/src/ui/retained_host/app/showcase_event_inputs/edit_tests.rs
  - zircon_editor/src/ui/retained_host/app/pane_surface_actions/component_showcase.rs
implementation_files:
  - zircon_editor/src/ui/retained_host/app/showcase_event_inputs.rs
  - zircon_editor/src/ui/retained_host/app/showcase_event_inputs/action.rs
  - zircon_editor/src/ui/retained_host/app/showcase_event_inputs/action/collections.rs
  - zircon_editor/src/ui/retained_host/app/showcase_event_inputs/action/fields.rs
  - zircon_editor/src/ui/retained_host/app/showcase_event_inputs/action/lists.rs
  - zircon_editor/src/ui/retained_host/app/showcase_event_inputs/action/references.rs
  - zircon_editor/src/ui/retained_host/app/showcase_event_inputs/action/selection.rs
  - zircon_editor/src/ui/retained_host/app/showcase_event_inputs/action/world_surface.rs
  - zircon_editor/src/ui/retained_host/app/showcase_event_inputs/action_tests.rs
  - zircon_editor/src/ui/retained_host/app/showcase_event_inputs/edit.rs
  - zircon_editor/src/ui/retained_host/app/showcase_event_inputs/edit/collections.rs
  - zircon_editor/src/ui/retained_host/app/showcase_event_inputs/edit/lists.rs
  - zircon_editor/src/ui/retained_host/app/showcase_event_inputs/edit/menu.rs
  - zircon_editor/src/ui/retained_host/app/showcase_event_inputs/edit/values.rs
  - zircon_editor/src/ui/retained_host/app/showcase_event_inputs/edit_tests.rs
plan_sources:
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - user: 2026-06-18 editor UI architecture implementation, feature first and tests deferred
tests:
  - cargo fmt -p zircon_editor
  - cargo fmt -p zircon_editor --check
  - app showcase edit-input ownership scan
  - app showcase action-input ownership scan
  - app showcase event-input test ownership scan
  - app showcase edit-input subowner ownership scan
  - cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never
doc_type: module-detail
---

# Component Showcase Event Inputs

`app/showcase_event_inputs.rs` is the structural entry for pure Component Showcase demo input conversion. It re-exports action-triggered and edit-string conversion, owns shared action-id normalization, shared virtual/paged list defaults, and the common option-selection helper.

## Action Inputs

`app/showcase_event_inputs/action.rs` is the action-input structural dispatcher. It preserves the stable `demo_input_for_showcase_action(...)` entry and delegates field values, selections, reference drops, collection edits, list/page actions, and world-surface updates to child owners under `app/showcase_event_inputs/action/`.

`action/fields.rs` owns scalar field, vector field, text field, toggle/radio, slider, and range-slider demo payloads. `action/selection.rs` owns segmented/tab/dropdown/combo/enum/flags/search-select and context action menu payloads. `action/references.rs` owns asset/instance/object reference drop payloads, active drag/drop-hover events, and asset-field binding suffix no-op handling. `action/collections.rs` owns container toggles plus array/map edit payloads. `action/lists.rs` owns row hover/press/click, virtual-list range, and paged-list request payloads. `action/world_surface.rs` owns world-space surface transform/configuration payloads. `action_tests.rs` owns the action-input regressions.

## Edit Payloads

`app/showcase_event_inputs/edit.rs` is the edit-input structural dispatcher. It preserves the stable `demo_input_for_showcase_edit(...)` entry and delegates context-menu anchors, virtual/paged list requests, collection row edits, and fallback value commits to child owners under `app/showcase_event_inputs/edit/`.

`edit/menu.rs` owns context action menu popup-anchor parsing. `edit/lists.rs` owns virtual-list visible-range and paged-list request parsing. `edit/collections.rs` owns array/map row edit payload parsing. `edit/values.rs` owns typed collection value parsing plus number/range/fallback committed values. `edit_tests.rs` owns the edit-focused regressions.

## Boundary Rules

- Keep shared action-id normalization, shared virtual/paged list defaults, and `select_option(...)` in `app/showcase_event_inputs.rs`.
- Keep action-input dispatch and the `ComponentShowcase*` show fallback in `app/showcase_event_inputs/action.rs`.
- Keep scalar/vector/text/toggle/slider action payloads in `app/showcase_event_inputs/action/fields.rs`.
- Keep option selection, search-select, and context action menu payloads in `app/showcase_event_inputs/action/selection.rs`.
- Keep asset/instance/object reference payloads and asset-field binding suffix handling in `app/showcase_event_inputs/action/references.rs`.
- Keep container toggles and array/map action payloads in `app/showcase_event_inputs/action/collections.rs`.
- Keep list row, virtual-list, and paged-list action payloads in `app/showcase_event_inputs/action/lists.rs`.
- Keep world-space surface action payloads in `app/showcase_event_inputs/action/world_surface.rs`.
- Keep action-input regressions in `app/showcase_event_inputs/action_tests.rs`.
- Keep edit-input dispatch in `app/showcase_event_inputs/edit.rs`.
- Keep context menu popup-anchor parsing in `app/showcase_event_inputs/edit/menu.rs`.
- Keep virtual-list and paged-list edit request parsing in `app/showcase_event_inputs/edit/lists.rs`.
- Keep array/map row edit payload parsing in `app/showcase_event_inputs/edit/collections.rs`.
- Keep typed collection values plus number/range/fallback committed values in `app/showcase_event_inputs/edit/values.rs`.
- Keep edit-input regressions in `app/showcase_event_inputs/edit_tests.rs`.
- Keep host-state-aware dispatch, runtime binding lookup, status-line mutation, and presentation invalidation in `app/pane_surface_actions/component_showcase.rs`.

## Validation Notes

The 2026-06-18 edit-input split reduced `showcase_event_inputs.rs` from 645 lines to 349 lines. `showcase_event_inputs/edit.rs` is 306 lines and owns edit payload parsing plus the moved edit-input regressions.

Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app showcase edit-input ownership scan, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only. Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-18 action-input split reduced `showcase_event_inputs.rs` from 333 lines to 44 lines. `showcase_event_inputs/action.rs` is 296 lines and owns action id to demo input mapping plus the moved action-input regressions; `showcase_event_inputs/edit.rs` is 298 lines after explicit dependency imports.

Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app showcase action-input ownership scan, scoped `git diff --check`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only. Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-19 event-input test split moved the remaining inline action/edit regressions into `showcase_event_inputs/action_tests.rs` and `showcase_event_inputs/edit_tests.rs`. `action.rs` dropped from 296 lines to 243 lines, `edit.rs` dropped from 298 lines to 159 lines, `action_tests.rs` is 50 lines, and `edit_tests.rs` is 133 lines. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app showcase event-input test ownership scan, and `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-ui-owner-split-0619 --message-format short --color never`, which passed with existing warning noise only (`zircon_runtime` 142 warnings, `zircon_editor` 65 warnings). Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-19 action subowner split reduced `showcase_event_inputs/action.rs` from 243 lines to 47 lines. New child files are `action/fields.rs` (72 lines), `action/selection.rs` (37 lines), `action/references.rs` (48 lines), `action/collections.rs` (74 lines), `action/lists.rs` (30 lines), and `action/world_surface.rs` (26 lines). The root no longer owns drag payload construction, collection map construction, list defaults, or the large action match table.

Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app showcase action subowner ownership scan, and scoped `git diff --check` (only the existing CRLF conversion warning appeared). `cargo check` was not rerun for this slice because the current workspace remains blocked before `zircon_editor` by the unrelated `zircon_runtime/src/dynamic_api/session/project.rs` AssetManager/ProjectAssetManager interface mismatch recorded in the 08 plan. Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-19 edit subowner split reduced `showcase_event_inputs/edit.rs` from 159 lines to 23 lines. New child files are `edit/menu.rs` (17 lines), `edit/lists.rs` (70 lines), `edit/collections.rs` (64 lines), and `edit/values.rs` (27 lines). The root no longer owns collection row parsing, list/page parsing, popup-anchor parsing, or typed committed-value parsing.

Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app showcase edit-input subowner ownership scan, and scoped `git diff --check` (only the existing CRLF conversion warning appeared). A fresh `cargo check` was deferred for this slice because separate runtime Cargo checks are currently active in this workspace; full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.
