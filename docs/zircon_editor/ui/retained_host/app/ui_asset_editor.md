---
related_code:
  - zircon_editor/src/ui/retained_host/app.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor/actions.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events/binding.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events/collection.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events/component_adapter.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events/palette.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events/preview.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events/source.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events/style.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events/structure.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events/widget.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_routes.rs
  - zircon_editor/src/ui/retained_host/app/invalidation.rs
implementation_files:
  - zircon_editor/src/ui/retained_host/app.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor/actions.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events/binding.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events/collection.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events/component_adapter.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events/palette.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events/preview.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events/source.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events/style.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events/structure.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events/widget.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_routes.rs
plan_sources:
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - user: 2026-06-18 editor UI architecture implementation, feature first and tests deferred
tests:
  - cargo fmt -p zircon_editor
  - cargo fmt -p zircon_editor --check
  - app UI Asset editor detail-event/style ownership scan
  - app UI Asset collection-event ownership scan
  - app UI Asset binding-detail ownership scan
  - app UI Asset preview-detail ownership scan
  - app UI Asset source/palette-detail ownership scan
  - app UI Asset widget/structure/component-adapter ownership scan
  - app UI Asset action routing ownership scan
  - cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never
doc_type: module-detail
---

# UI Asset Editor Host Actions

`app/ui_asset_editor.rs` owns the top-level UI Asset editor command entry from retained host callbacks. It keeps the app-visible `RetainedEditorHost::dispatch_ui_asset_action(...)` method and delegates stable action-id semantics to its action child.

The file should stay as a narrow entry module. It should not collect detail-field mutations, collection-row events, component-adapter envelopes, semantic action-path mapping, command action-id mapping, or detail-specific validation.

## Action Routing

`app/ui_asset_editor/actions.rs` owns top-level UI Asset action-id dispatch. It focuses the callback source window, maps stable action ids such as save, workspace conflict actions, mode/tool changes, palette insertion, canvas movement, theme source selection, preview preset changes, undo/redo, locale preview, and emergency recovery to `EditorManager`, then records presentation-domain invalidation when a command changes editor state.

Keeping action-id mapping in the child module leaves the root as the retained host callback boundary and prevents the command table from crowding out the detail-event, collection-event, and component-adapter boundaries.

## Detail Events

`app/ui_asset_editor_detail_events.rs` owns UI Asset detail-field dispatch. It routes detail ids to focused handler families and leaves component-adapter commits, widget/promote mutation, slot/layout structure mutation, source/palette/preview/binding/style handlers, and collection row selection/activation to child modules.

`app/ui_asset_editor_detail_routes.rs` owns narrow route helpers used by detail event handlers. It currently maps generic widget prop/state action ids to component-adapter target paths.

## Collection Events

`app/ui_asset_editor_detail_events/collection.rs` owns UI Asset collection row selection and activation dispatch. It maps matched style rule, palette, hierarchy, preview, source outline, preview mock, binding, slot semantic, and layout semantic collection ids into the corresponding `EditorManager` selection or activation calls.

Keeping collection events out of the parent detail dispatcher keeps row-list selection flow separate from text/detail mutation flow and leaves room for further collection-specific behavior without expanding the central detail handler.

## Component Adapter Commits

`app/ui_asset_editor_detail_events/component_adapter.rs` owns UI Asset component-adapter commit envelopes. It focuses the source window, builds `UiComponentEventEnvelope` values for asset-editor binding targets, dispatches them through runtime component adapters, forwards status text, and marks presentation dirty when runtime patches or projection refreshes require it.

Keeping component-adapter commits in a dedicated owner gives binding, widget, slot, and layout detail handlers one stable commit path without making the detail dispatcher a runtime-envelope module.

## Widget Details

`app/ui_asset_editor_detail_events/widget.rs` owns widget and promote-widget detail mutation. It handles direct widget component-adapter commits, generic widget prop/state route commits, root-class policy commits, and selected widget promotion metadata edits.

Keeping widget handling out of the parent detail dispatcher separates authored widget identity/text/state mutation from slot, layout, source, palette, preview, binding, and style detail flow.

## Structure Details

`app/ui_asset_editor_detail_events/structure.rs` owns slot and layout detail mutation. It handles slot mount/padding/preferred size commits, slot semantic edits, layout preferred size commits, layout semantic edits, semantic action-path mapping, and the focused semantic mapping regressions.

Keeping structure handling out of the parent detail dispatcher isolates layout/slot semantic mapping from general detail-id routing.

## Binding Details

`app/ui_asset_editor_detail_events/binding.rs` owns UI Asset binding detail mutation. It handles binding add/delete, binding id/event/route/target component-adapter commits, binding payload upsert/delete, binding payload suggestions, binding route suggestions, and binding action suggestions.

Keeping binding handling out of the parent detail dispatcher separates event-binding authoring from widget, slot/layout, source, palette, and preview mock mutations.

## Preview Details

`app/ui_asset_editor_detail_events/preview.rs` owns UI Asset preview mock mutation. It handles preview mock value set/clear, nested preview mock value/upsert/delete, and preview mock suggestion application.

Keeping preview mock handling out of the parent detail dispatcher separates interactive preview data authoring from widget, slot/layout, source, palette, binding, and style mutation flow.

## Source Details

`app/ui_asset_editor_detail_events/source.rs` owns UI Asset source detail mutation. It handles source text replacement and source cursor byte-offset selection.

Keeping source handling out of the parent detail dispatcher separates authored source text state from component-adapter widget, slot, and layout detail commits.

## Palette Details

`app/ui_asset_editor_detail_events/palette.rs` owns UI Asset palette drag detail mutation. It parses palette drag hover coordinates and updates the selected palette drag target through `EditorManager`.

Keeping palette drag handling out of the parent detail dispatcher leaves pointer-like palette interaction state separate from text/detail mutation routing.

## Style Detail Events

`app/ui_asset_editor_detail_events/style.rs` owns style and theme detail-field handlers. It handles style class add/remove, theme source promotion/refactor helpers, style rule selection/rename/delete/reorder, style declaration upsert/delete, and style token upsert/delete.

Keeping style/theme detail handling out of the parent detail dispatcher prevents the dispatcher from becoming a second large UI Asset editor owner.

## Boundary Rules

- Keep the app-visible retained-host UI Asset action entry in `app/ui_asset_editor.rs`.
- Keep top-level toolbar/menu/canvas/mode/palette action-id mapping, source-window focus, status-line messages, asset-workspace sync after save, and presentation invalidation in `app/ui_asset_editor/actions.rs`.
- Keep detail-id dispatch in `app/ui_asset_editor_detail_events.rs`.
- Keep component-adapter commit envelope construction and runtime dispatch in `app/ui_asset_editor_detail_events/component_adapter.rs`.
- Keep widget and promote-widget detail dispatch in `app/ui_asset_editor_detail_events/widget.rs`.
- Keep slot/layout detail dispatch and semantic action-path helpers in `app/ui_asset_editor_detail_events/structure.rs`.
- Keep collection row selection/activation dispatch in `app/ui_asset_editor_detail_events/collection.rs`.
- Keep binding detail, binding payload, and binding suggestion dispatch in `app/ui_asset_editor_detail_events/binding.rs`.
- Keep preview mock value, nested entry, and suggestion dispatch in `app/ui_asset_editor_detail_events/preview.rs`.
- Keep source text/cursor dispatch in `app/ui_asset_editor_detail_events/source.rs`.
- Keep palette drag hover target dispatch in `app/ui_asset_editor_detail_events/palette.rs`.
- Keep style/theme detail handlers in `app/ui_asset_editor_detail_events/style.rs`; do not add style class, theme source, style rule, style declaration, or style token mutation logic back to the detail dispatcher.
- Keep detail route parsing helpers in `app/ui_asset_editor_detail_routes.rs`.
- Future growth in detail handlers should split by domain first, for example binding, preview mock, structure/layout, and collection event modules.

## Validation Notes

The 2026-06-18 detail-event extraction reduced `ui_asset_editor.rs` from 1348 lines to 287 lines. `ui_asset_editor_detail_events.rs` took over retained detail dispatch and then moved style/theme handlers into `ui_asset_editor_detail_events/style.rs`, leaving the parent detail dispatcher at 872 lines and the style owner at 204 lines. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app UI Asset editor detail-event/style ownership scan, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only. Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-18 collection-event split reduced `ui_asset_editor_detail_events.rs` from 872 lines to 780 lines. `ui_asset_editor_detail_events/collection.rs` is 97 lines and owns UI Asset collection selection/activation dispatch. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app UI Asset collection-event ownership scan, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only.

The 2026-06-18 binding-detail split reduced `ui_asset_editor_detail_events.rs` from 780 lines to 588 lines. `ui_asset_editor_detail_events/binding.rs` is 202 lines and owns binding detail mutation, binding payload mutation, and binding suggestion application. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app UI Asset binding-detail ownership scan, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only.

The 2026-06-18 preview-detail split reduced `ui_asset_editor_detail_events.rs` from 588 lines to 490 lines. `ui_asset_editor_detail_events/preview.rs` is 103 lines and owns preview mock value, nested entry, and suggestion dispatch. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app UI Asset preview-detail ownership scan, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only.

The 2026-06-18 source/palette-detail split reduced `ui_asset_editor_detail_events.rs` from 490 lines to 416 lines. `ui_asset_editor_detail_events/source.rs` is 38 lines and owns source text/cursor detail dispatch; `ui_asset_editor_detail_events/palette.rs` is 47 lines and owns palette drag hover dispatch. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app UI Asset source/palette-detail ownership scan, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only.

The 2026-06-18 widget/structure/component-adapter split reduced `ui_asset_editor_detail_events.rs` from 416 lines to 104 lines. `ui_asset_editor_detail_events/component_adapter.rs` is 38 lines and owns component-adapter commit envelopes, `widget.rs` is 84 lines and owns widget/promote-widget detail dispatch, and `structure.rs` is 217 lines and owns slot/layout detail dispatch plus semantic action-path tests. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app UI Asset widget/structure/component-adapter ownership scan, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only.

The 2026-06-18 action-routing split reduced `ui_asset_editor.rs` from 287 lines to 9 lines. `ui_asset_editor/actions.rs` is 287 lines and owns top-level UI Asset action-id mapping, source-window focus, status-line messages, post-save asset workspace sync, and presentation invalidation. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app UI Asset action routing ownership scan, scoped `git diff --check`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only. Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.
