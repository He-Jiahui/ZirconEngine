---
related_code:
  - zircon_editor/src/ui/retained_host/app.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor/actions.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor/actions/canvas.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor/actions/mode_preview.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor/actions/mode_preview/designer_tool.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor/actions/mode_preview/editor_mode.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor/actions/mode_preview/locale.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor/actions/mode_preview/preview_preset.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor/actions/palette.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor/actions/theme.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor/actions/theme/pseudo_state.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor/actions/theme/selection.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor/actions/theme/source.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor/actions/theme/style_rule.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor/actions/workspace.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor/actions/workspace/diff.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor/actions/workspace/emergency.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor/actions/workspace/history_reference.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor/actions/workspace/local_copy.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor/actions/workspace/save.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events/binding.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events/binding/entry.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events/binding/entry/fields.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events/binding/entry/lifecycle.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events/binding/payload.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events/binding/suggestions.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events/binding/suggestions/action.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events/binding/suggestions/payload.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events/binding/suggestions/route.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events/collection.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events/collection/binding.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events/collection/editor.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events/component_adapter.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events/detail_dispatch.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events/detail_dispatch/binding.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events/detail_dispatch/preview.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events/detail_dispatch/style.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events/detail_dispatch/surface.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events/palette.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events/preview.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events/preview/nested.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events/preview/suggestions.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events/preview/value.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events/source.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events/style.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events/style/class.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events/style/rules.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events/style/rules/declaration.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events/style/rules/rule.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events/style/theme_source.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events/style/tokens.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events/structure.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events/structure/layout.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events/structure/layout/commit.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events/structure/layout/semantic.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events/structure/semantic_paths.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events/structure/slot.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events/structure/slot/commit.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events/structure/slot/semantic.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events/structure/tests.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events/widget.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events/widget/commit.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events/widget/promote.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_routes.rs
  - zircon_editor/src/ui/retained_host/app/invalidation.rs
implementation_files:
  - zircon_editor/src/ui/retained_host/app.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor/actions.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor/actions/canvas.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor/actions/mode_preview.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor/actions/mode_preview/designer_tool.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor/actions/mode_preview/editor_mode.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor/actions/mode_preview/locale.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor/actions/mode_preview/preview_preset.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor/actions/palette.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor/actions/theme.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor/actions/theme/pseudo_state.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor/actions/theme/selection.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor/actions/theme/source.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor/actions/theme/style_rule.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor/actions/workspace.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor/actions/workspace/diff.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor/actions/workspace/emergency.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor/actions/workspace/history_reference.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor/actions/workspace/local_copy.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor/actions/workspace/save.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events/binding.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events/binding/entry.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events/binding/entry/fields.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events/binding/entry/lifecycle.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events/binding/payload.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events/binding/suggestions.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events/binding/suggestions/action.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events/binding/suggestions/payload.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events/binding/suggestions/route.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events/collection.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events/collection/binding.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events/collection/editor.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events/component_adapter.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events/detail_dispatch.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events/detail_dispatch/binding.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events/detail_dispatch/preview.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events/detail_dispatch/style.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events/detail_dispatch/surface.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events/palette.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events/preview.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events/preview/nested.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events/preview/suggestions.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events/preview/value.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events/source.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events/style.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events/style/class.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events/style/rules.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events/style/rules/declaration.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events/style/rules/rule.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events/style/theme_source.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events/style/tokens.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events/structure.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events/structure/layout.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events/structure/layout/commit.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events/structure/layout/semantic.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events/structure/semantic_paths.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events/structure/slot.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events/structure/slot/commit.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events/structure/slot/semantic.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events/structure/tests.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events/widget.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events/widget/commit.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events/widget/promote.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_routes.rs
plan_sources:
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - user: 2026-06-18 editor UI architecture implementation, feature first and tests deferred
tests:
  - cargo fmt -p zircon_editor
  - cargo fmt -p zircon_editor --check
  - app UI Asset editor detail-event/style ownership scan
  - app UI Asset collection-event ownership scan
  - app UI Asset collection-event editor/binding subowner ownership scan
  - app UI Asset binding-detail ownership scan
  - app UI Asset binding entry lifecycle/field subowner ownership scan
  - app UI Asset preview-detail ownership scan
  - app UI Asset preview-detail value/nested/suggestion ownership scan
  - app UI Asset source/palette-detail ownership scan
  - app UI Asset widget/structure/component-adapter ownership scan
  - app UI Asset structure-detail slot/layout ownership scan
  - app UI Asset structure slot commit/semantic subowner ownership scan
  - app UI Asset structure layout commit/semantic subowner ownership scan
  - app UI Asset binding-detail entry/payload/suggestion ownership scan
  - app UI Asset binding suggestion action/payload/route subowner ownership scan
  - app UI Asset action routing ownership scan
  - app UI Asset action domain ownership scan
  - app UI Asset workspace action subowner ownership scan
  - app UI Asset detail dispatch ownership scan
  - app UI Asset detail dispatch group subowner ownership scan
  - app UI Asset style-detail class/theme/rule/token ownership scan
  - app UI Asset widget commit/promote subowner ownership scan
  - app UI Asset style rule/declaration subowner ownership scan
  - app UI Asset theme action subowner ownership scan
  - app UI Asset mode-preview action subowner ownership scan
  - cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never
doc_type: module-detail
---

# UI Asset Editor Host Actions

`app/ui_asset_editor.rs` owns the top-level UI Asset editor command entry from retained host callbacks. It keeps the app-visible `RetainedEditorHost::dispatch_ui_asset_action(...)` method and delegates stable action-id semantics to its action child.

The file should stay as a narrow entry module. It should not collect detail-field mutations, collection-row events, component-adapter envelopes, semantic action-path mapping, command action-id mapping, or detail-specific validation.

## Action Routing

`app/ui_asset_editor/actions.rs` owns top-level UI Asset action-id dispatch. It focuses the callback source window, runs the domain dispatch chain, applies the shared save/workspace-sync and presentation-dirty post-processing, and reports unknown actions. The child action modules own the concrete stable action ids.

`ui_asset_editor/actions/workspace.rs` is the structural entry for workspace-adjacent actions: `workspace/save.rs` owns save/reload/keep-local actions, `workspace/local_copy.rs` owns local-copy status handling, `workspace/diff.rs` owns conflict diff snapshot status handling, `workspace/emergency.rs` owns emergency recovery and Asset Browser open actions, and `workspace/history_reference.rs` owns undo/redo/reference-open actions. `theme.rs` is the structural entry for theme-adjacent actions: `theme/source.rs` owns theme-source operations, `theme/style_rule.rs` owns style rule creation/extraction, `theme/pseudo_state.rs` owns pseudo-state toggles, and `theme/selection.rs` owns indexed theme source selection. `palette.rs` owns palette insertion/drag/target actions. `canvas.rs` owns node move, reparent, convert, extract, promote, wrap, and unwrap actions. `mode_preview.rs` is the structural entry for view-mode adjacent actions: `mode_preview/preview_preset.rs` owns preview preset action ids, `mode_preview/editor_mode.rs` owns editor mode action ids, `mode_preview/designer_tool.rs` owns designer tool action ids, and `mode_preview/locale.rs` owns locale preview action ids.

Keeping action-id mapping in the child module leaves the root as the retained host callback boundary and prevents the command table from crowding out the detail-event, collection-event, and component-adapter boundaries.

## Detail Events

`app/ui_asset_editor_detail_events.rs` is the structural UI Asset detail-event family entry. `app/ui_asset_editor_detail_events/detail_dispatch.rs` owns detail-id family orchestration and the unknown-detail status fallback. Its child dispatch groups own stable detail-id routing by family: `detail_dispatch/surface.rs` routes widget/promote/slot/layout/palette/source details, `detail_dispatch/style.rs` routes style class/theme/rule/token details, `detail_dispatch/preview.rs` routes preview mock details, and `detail_dispatch/binding.rs` routes binding/payload/suggestion details. Concrete mutations remain in component-adapter commits, widget/promote mutation, slot/layout structure mutation, source/palette/preview/binding/style handlers, and collection row selection/activation child modules.

`app/ui_asset_editor_detail_routes.rs` owns narrow route helpers used by detail event handlers. It currently maps generic widget prop/state action ids to component-adapter target paths.

## Collection Events

`app/ui_asset_editor_detail_events/collection.rs` owns UI Asset collection row selection and activation dispatch orchestration. It focuses the callback source window, normalizes the instance/index inputs, asks the child handlers for a known collection dispatch, marks presentation dirty on success, and reports unknown collection ids.

`app/ui_asset_editor_detail_events/collection/editor.rs` owns editor-facing collection selection and activation calls: matched style rule, palette, palette target candidate, hierarchy, preview, source outline, preview mock subject/property, and preview mock nested entries.

`app/ui_asset_editor_detail_events/collection/binding.rs` owns binding and semantic collection selection calls: binding, binding event, binding action kind, binding payload, slot semantic, and layout semantic rows.

Keeping collection events out of the parent detail dispatcher keeps row-list selection flow separate from text/detail mutation flow and leaves room for further collection-specific behavior without expanding the central detail handler.

## Component Adapter Commits

`app/ui_asset_editor_detail_events/component_adapter.rs` owns UI Asset component-adapter commit envelopes. It focuses the source window, builds `UiComponentEventEnvelope` values for asset-editor binding targets, dispatches them through runtime component adapters, forwards status text, and marks presentation dirty when runtime patches or projection refreshes require it.

Keeping component-adapter commits in a dedicated owner gives binding, widget, slot, and layout detail handlers one stable commit path without making the detail dispatcher a runtime-envelope module.

## Widget Details

`app/ui_asset_editor_detail_events/widget.rs` is the structural entry for widget detail mutation. `widget/commit.rs` owns direct widget component-adapter commits, generic widget prop/state route commits, and root-class policy commits. `widget/promote.rs` owns selected widget promotion metadata edits.

Keeping widget handling out of the parent detail dispatcher separates authored widget identity/text/state mutation from slot, layout, source, palette, preview, binding, and style detail flow.

## Structure Details

`app/ui_asset_editor_detail_events/structure.rs` is the structure detail-event family entry. `structure/slot.rs` is the structural slot detail entry: `structure/slot/commit.rs` owns slot mount/padding/preferred size component-adapter commits, and `structure/slot/semantic.rs` owns slot semantic value/delete/field edits. `structure/layout.rs` is the structural layout detail entry: `structure/layout/commit.rs` owns layout preferred size commits, and `structure/layout/semantic.rs` owns layout semantic value/delete/field edits. `structure/semantic_paths.rs` owns slot/layout semantic action-path mapping used by both handlers. `structure/tests.rs` keeps the focused semantic mapping regressions with the mapping owner.

Keeping structure handling out of the parent detail dispatcher isolates layout/slot semantic mapping from general detail-id routing, while the child split keeps slot mutation, layout mutation, and shared semantic path tables from accumulating in one retained-host file.

## Binding Details

`app/ui_asset_editor_detail_events/binding.rs` is the binding detail-event family entry. `binding/entry.rs` is the structural binding entry detail owner: `binding/entry/lifecycle.rs` owns binding add/delete mutation, and `binding/entry/fields.rs` owns binding id/event/route/target component-adapter commits. `binding/payload.rs` owns binding payload upsert/delete mutation, and `binding/suggestions.rs` is the structural suggestion family entry. `binding/suggestions/payload.rs`, `binding/suggestions/route.rs`, and `binding/suggestions/action.rs` own payload, route, and action suggestion application respectively.

Keeping binding handling out of the parent detail dispatcher separates event-binding authoring from widget, slot/layout, source, palette, and preview mock mutations, while the child split keeps direct field commits, payload-map mutation, and suggestion rows independently owned.

## Preview Details

`app/ui_asset_editor_detail_events/preview.rs` is the structural preview mock detail-event family entry. `preview/value.rs` owns preview mock value set/clear, `preview/nested.rs` owns nested preview mock value/upsert/delete, and `preview/suggestions.rs` owns preview mock suggestion application.

Keeping preview mock handling out of the parent detail dispatcher separates interactive preview data authoring from widget, slot/layout, source, palette, binding, and style mutation flow.

## Source Details

`app/ui_asset_editor_detail_events/source.rs` owns UI Asset source detail mutation. It handles source text replacement and source cursor byte-offset selection.

Keeping source handling out of the parent detail dispatcher separates authored source text state from component-adapter widget, slot, and layout detail commits.

## Palette Details

`app/ui_asset_editor_detail_events/palette.rs` owns UI Asset palette drag detail mutation. It parses palette drag hover coordinates and updates the selected palette drag target through `EditorManager`.

Keeping palette drag handling out of the parent detail dispatcher leaves pointer-like palette interaction state separate from text/detail mutation routing.

## Style Detail Events

`app/ui_asset_editor_detail_events/style.rs` is the style detail-event family entry. `style/class.rs` owns style class add/remove, `style/theme_source.rs` owns theme source promotion/refactor helpers, `style/rules.rs` is the structural stylesheet rule detail entry, and `style/tokens.rs` owns style token select/upsert/delete. `style/rules/rule.rs` owns rule selection/rename/delete/reorder; `style/rules/declaration.rs` owns style declaration select/upsert/delete.

Keeping style/theme detail handling out of the parent detail dispatcher prevents the dispatcher from becoming a second large UI Asset editor owner, while the child split keeps class membership, theme helper application, stylesheet rule declarations, and design tokens independently owned.

## Boundary Rules

- Keep the app-visible retained-host UI Asset action entry in `app/ui_asset_editor.rs`.
- Keep source-window focus, domain-dispatch sequencing, unknown-action reporting, asset-workspace sync after save, and presentation invalidation in `app/ui_asset_editor/actions.rs`.
- Keep workspace action orchestration in `app/ui_asset_editor/actions/workspace.rs`.
- Keep save/reload/keep-local actions in `app/ui_asset_editor/actions/workspace/save.rs`.
- Keep local-copy status handling in `app/ui_asset_editor/actions/workspace/local_copy.rs`.
- Keep conflict diff snapshot status handling in `app/ui_asset_editor/actions/workspace/diff.rs`.
- Keep emergency recovery and Asset Browser open actions in `app/ui_asset_editor/actions/workspace/emergency.rs`.
- Keep undo/redo/reference-open actions in `app/ui_asset_editor/actions/workspace/history_reference.rs`.
- Keep theme action orchestration in `app/ui_asset_editor/actions/theme.rs`.
- Keep theme-source open/promote/detach/clone/prune actions in `app/ui_asset_editor/actions/theme/source.rs`.
- Keep style rule create/extract-inline actions in `app/ui_asset_editor/actions/theme/style_rule.rs`.
- Keep pseudo-state toggle actions in `app/ui_asset_editor/actions/theme/pseudo_state.rs`.
- Keep indexed theme source selection parsing and invalid-selection status text in `app/ui_asset_editor/actions/theme/selection.rs`.
- Keep palette insert/drag/target actions in `app/ui_asset_editor/actions/palette.rs`.
- Keep canvas move/reparent/convert/extract/promote/wrap/unwrap actions in `app/ui_asset_editor/actions/canvas.rs`.
- Keep preview preset, mode, designer tool, and locale preview orchestration in `app/ui_asset_editor/actions/mode_preview.rs`.
- Keep preview preset action ids in `app/ui_asset_editor/actions/mode_preview/preview_preset.rs`.
- Keep editor mode action ids in `app/ui_asset_editor/actions/mode_preview/editor_mode.rs`.
- Keep designer tool action ids in `app/ui_asset_editor/actions/mode_preview/designer_tool.rs`.
- Keep locale preview action ids in `app/ui_asset_editor/actions/mode_preview/locale.rs`.
- Keep `app/ui_asset_editor_detail_events.rs` as the structural detail-event family entry.
- Keep detail-id group orchestration and unknown-detail fallback in `app/ui_asset_editor_detail_events/detail_dispatch.rs`.
- Keep widget/promote/slot/layout/palette/source detail-id routing in `app/ui_asset_editor_detail_events/detail_dispatch/surface.rs`.
- Keep style class/theme/rule/token detail-id routing in `app/ui_asset_editor_detail_events/detail_dispatch/style.rs`.
- Keep preview mock detail-id routing in `app/ui_asset_editor_detail_events/detail_dispatch/preview.rs`.
- Keep binding/payload/suggestion detail-id routing in `app/ui_asset_editor_detail_events/detail_dispatch/binding.rs`.
- Keep component-adapter commit envelope construction and runtime dispatch in `app/ui_asset_editor_detail_events/component_adapter.rs`.
- Keep widget detail module declarations in `app/ui_asset_editor_detail_events/widget.rs`.
- Keep widget prop/state component-adapter commit dispatch in `app/ui_asset_editor_detail_events/widget/commit.rs`.
- Keep promote-widget metadata detail dispatch in `app/ui_asset_editor_detail_events/widget/promote.rs`.
- Keep structure detail module declarations in `app/ui_asset_editor_detail_events/structure.rs`.
- Keep slot detail orchestration in `app/ui_asset_editor_detail_events/structure/slot.rs`.
- Keep slot mount/padding/preferred-size component-adapter commits in `app/ui_asset_editor_detail_events/structure/slot/commit.rs`.
- Keep slot semantic value/delete/field edits in `app/ui_asset_editor_detail_events/structure/slot/semantic.rs`.
- Keep layout detail orchestration in `app/ui_asset_editor_detail_events/structure/layout.rs`.
- Keep layout preferred-size component-adapter commits in `app/ui_asset_editor_detail_events/structure/layout/commit.rs`.
- Keep layout semantic value/delete/field edits in `app/ui_asset_editor_detail_events/structure/layout/semantic.rs`.
- Keep semantic action-path helpers in `app/ui_asset_editor_detail_events/structure/semantic_paths.rs`.
- Keep collection event orchestration, source-window focus, unknown-event reporting, and presentation invalidation in `app/ui_asset_editor_detail_events/collection.rs`.
- Keep editor-facing collection selection/activation calls in `app/ui_asset_editor_detail_events/collection/editor.rs`.
- Keep binding and semantic collection selection calls in `app/ui_asset_editor_detail_events/collection/binding.rs`.
- Keep binding detail, binding payload, and binding suggestion dispatch in `app/ui_asset_editor_detail_events/binding.rs`.
- Keep binding detail orchestration and unknown binding action fallback in `app/ui_asset_editor_detail_events/binding/entry.rs`.
- Keep binding add/delete mutation in `app/ui_asset_editor_detail_events/binding/entry/lifecycle.rs`.
- Keep binding id/event/route/target component-adapter commits in `app/ui_asset_editor_detail_events/binding/entry/fields.rs`.
- Keep binding payload suggestion application in `app/ui_asset_editor_detail_events/binding/suggestions/payload.rs`.
- Keep binding route suggestion application in `app/ui_asset_editor_detail_events/binding/suggestions/route.rs`.
- Keep binding action suggestion application in `app/ui_asset_editor_detail_events/binding/suggestions/action.rs`.
- Keep preview mock detail module declarations in `app/ui_asset_editor_detail_events/preview.rs`.
- Keep preview mock value set/clear dispatch in `app/ui_asset_editor_detail_events/preview/value.rs`.
- Keep nested preview mock value/upsert/delete dispatch in `app/ui_asset_editor_detail_events/preview/nested.rs`.
- Keep preview mock suggestion application in `app/ui_asset_editor_detail_events/preview/suggestions.rs`.
- Keep source text/cursor dispatch in `app/ui_asset_editor_detail_events/source.rs`.
- Keep palette drag hover target dispatch in `app/ui_asset_editor_detail_events/palette.rs`.
- Keep style/theme detail handlers in `app/ui_asset_editor_detail_events/style.rs` and its `style/` children; do not add style class, theme source, style rule, style declaration, or style token mutation logic back to the detail dispatcher.
- Keep stylesheet rule selection/rename/delete/reorder in `app/ui_asset_editor_detail_events/style/rules/rule.rs`.
- Keep stylesheet declaration select/upsert/delete in `app/ui_asset_editor_detail_events/style/rules/declaration.rs`.
- Keep detail route parsing helpers in `app/ui_asset_editor_detail_routes.rs`.
- Future growth in detail handlers should split by domain first, for example binding, preview mock, structure/layout, and collection event modules.

## Validation Notes

The 2026-06-18 detail-event extraction reduced `ui_asset_editor.rs` from 1348 lines to 287 lines. `ui_asset_editor_detail_events.rs` took over retained detail dispatch and then moved style/theme handlers into `ui_asset_editor_detail_events/style.rs`, leaving the parent detail dispatcher at 872 lines and the style owner at 204 lines. The 2026-06-19 style-detail child split later reduced `style.rs` to a 5-line family entry and moved class, theme source, rule/declaration, and token handlers into `style/class.rs`, `style/theme_source.rs`, `style/rules.rs`, and `style/tokens.rs`. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app UI Asset editor detail-event/style ownership scan, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only. Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-19 style rule/declaration subowner split reduced `ui_asset_editor_detail_events/style/rules.rs` from 90 lines to a 2-line structural entry. `style/rules/rule.rs` is 46 lines and owns rule selection/rename/delete/reorder. `style/rules/declaration.rs` is 48 lines and owns declaration select/upsert/delete.

Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app UI Asset style rule/declaration subowner ownership scan, scoped `git diff --check`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-ui-owner-split-0619 --message-format short --color never`, which passed with existing warning noise only (`zircon_runtime` 141 warnings, `zircon_editor` 65 warnings). Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-18 collection-event split reduced `ui_asset_editor_detail_events.rs` from 872 lines to 780 lines. `ui_asset_editor_detail_events/collection.rs` is 97 lines and owns UI Asset collection selection/activation dispatch. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app UI Asset collection-event ownership scan, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only.

The 2026-06-19 collection-event editor/binding subowner split reduced `ui_asset_editor_detail_events/collection.rs` from 97 lines to a 50-line dispatch orchestration owner. `collection/editor.rs` is 68 lines and owns editor-facing collection selection/activation calls. `collection/binding.rs` is 43 lines and owns binding and semantic collection selection calls.

Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app UI Asset collection-event editor/binding subowner ownership scan, scoped `git diff --check`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-ui-owner-split-0619 --message-format short --color never`, which passed with existing warning noise only (`zircon_runtime` 141 warnings, `zircon_editor` 65 warnings). Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-18 binding-detail split reduced `ui_asset_editor_detail_events.rs` from 780 lines to 588 lines. `ui_asset_editor_detail_events/binding.rs` is 202 lines and owns binding detail mutation, binding payload mutation, and binding suggestion application. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app UI Asset binding-detail ownership scan, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only.

The 2026-06-19 binding entry lifecycle/field subowner split reduced `ui_asset_editor_detail_events/binding/entry.rs` from 76 lines to a 24-line structural entry. `binding/entry/lifecycle.rs` is 28 lines and owns binding add/delete mutation. `binding/entry/fields.rs` is 28 lines and owns binding id/event/route/target component-adapter commits.

Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app UI Asset binding entry lifecycle/field subowner ownership scan, scoped `git diff --check`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-ui-owner-split-0619 --message-format short --color never`, which passed with existing warning noise only (`zircon_runtime` 141 warnings, `zircon_editor` 65 warnings). Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-19 binding suggestion action/payload/route subowner split reduced `ui_asset_editor_detail_events/binding/suggestions.rs` from 94 lines to a 3-line structural entry. `binding/suggestions/payload.rs`, `binding/suggestions/route.rs`, and `binding/suggestions/action.rs` are 34 lines each and own their respective suggestion application action id, `EditorManager` call, source-window focus, dirty marking, and unknown-action status text.

Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app UI Asset binding suggestion action/payload/route subowner ownership scan, scoped `git diff --check`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-ui-owner-split-0619 --message-format short --color never`, which passed with existing warning noise only (`zircon_runtime` 141 warnings, `zircon_editor` 65 warnings). Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-18 preview-detail split reduced `ui_asset_editor_detail_events.rs` from 588 lines to 490 lines. `ui_asset_editor_detail_events/preview.rs` is 103 lines and owns preview mock value, nested entry, and suggestion dispatch. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app UI Asset preview-detail ownership scan, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only.

The 2026-06-19 preview-detail value/nested/suggestion subowner split reduced `ui_asset_editor_detail_events/preview.rs` from 103 lines to a 3-line structural entry. `preview/value.rs` is 33 lines and owns preview mock value set/clear, `preview/nested.rs` is 44 lines and owns nested preview mock value/upsert/delete, and `preview/suggestions.rs` is 34 lines and owns preview mock suggestion application.

Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app UI Asset preview-detail value/nested/suggestion ownership scan, scoped `git diff --check`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-ui-owner-split-0619 --message-format short --color never`, which passed with existing warning noise only (`zircon_runtime` 141 warnings, `zircon_editor` 65 warnings). Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-18 source/palette-detail split reduced `ui_asset_editor_detail_events.rs` from 490 lines to 416 lines. `ui_asset_editor_detail_events/source.rs` is 38 lines and owns source text/cursor detail dispatch; `ui_asset_editor_detail_events/palette.rs` is 47 lines and owns palette drag hover dispatch. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app UI Asset source/palette-detail ownership scan, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only.

The 2026-06-18 widget/structure/component-adapter split reduced `ui_asset_editor_detail_events.rs` from 416 lines to 104 lines. `ui_asset_editor_detail_events/component_adapter.rs` is 38 lines and owns component-adapter commit envelopes, `widget.rs` is 84 lines and owns widget/promote-widget detail dispatch, and `structure.rs` is 217 lines and owns slot/layout detail dispatch plus semantic action-path tests. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app UI Asset widget/structure/component-adapter ownership scan, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only.

The 2026-06-19 widget commit/promote subowner split reduced `ui_asset_editor_detail_events/widget.rs` from 81 lines to a 2-line structural entry. `widget/commit.rs` is 49 lines and owns direct widget component-adapter commits, generic widget prop/state route commits, and root-class policy commits. `widget/promote.rs` is 35 lines and owns selected widget promotion metadata edits.

Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, and an app UI Asset widget commit/promote subowner ownership scan. A fresh `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-ui-owner-split-0619 --message-format short --color never` remains blocked before editor code by active `zircon_runtime::scene::dynamic_scene::session` owner-split work: `session/io/mod.rs` re-exports private IO helpers, producing E0364/E0603 visibility errors. Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-19 structure slot commit/semantic subowner split reduced `ui_asset_editor_detail_events/structure/slot.rs` from 83 lines to a 21-line structural entry. `structure/slot/commit.rs` is 25 lines and owns slot mount/padding/preferred-size component-adapter commits. `structure/slot/semantic.rs` is 44 lines and owns slot semantic value/delete/field edits.

Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app UI Asset structure slot commit/semantic subowner ownership scan, and `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-ui-owner-split-0619 --message-format short --color never`, which passed with existing warning noise only (`zircon_runtime` 141 warnings, `zircon_editor` 65 warnings). Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-19 structure layout commit/semantic subowner split reduced `ui_asset_editor_detail_events/structure/layout.rs` from 61 lines to a 21-line structural entry. `structure/layout/commit.rs` is 23 lines and owns layout preferred-size component-adapter commits. `structure/layout/semantic.rs` is 44 lines and owns layout semantic value/delete/field edits.

Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app UI Asset structure layout commit/semantic subowner ownership scan, scoped `git diff --check` with only existing LF/CRLF conversion warnings for docs, and `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-ui-owner-split-0619 --message-format short --color never`, which passed with existing warning noise only (`zircon_runtime` 141 warnings, `zircon_editor` 65 warnings). Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-18 action-routing split reduced `ui_asset_editor.rs` from 287 lines to 9 lines. `ui_asset_editor/actions.rs` is 287 lines and owns top-level UI Asset action-id mapping, source-window focus, status-line messages, post-save asset workspace sync, and presentation invalidation. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app UI Asset action routing ownership scan, scoped `git diff --check`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only. Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-19 action domain split reduced `ui_asset_editor/actions.rs` from 287 lines to 53 lines. The new action-domain child owners are `actions/workspace.rs` at 75 lines, `actions/theme.rs` at 86 lines, `actions/palette.rs` at 45 lines, `actions/canvas.rs` at 53 lines, and `actions/mode_preview.rs` at 89 lines. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app UI Asset action domain ownership scan, and `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-ui-owner-split-0619 --message-format short --color never`, which passed with existing warning noise only (`zircon_runtime` 142 warnings, `zircon_editor` 65 warnings). Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-19 workspace action subowner split reduced `ui_asset_editor/actions/workspace.rs` from 75 lines to a 30-line action-family entry. `workspace/save.rs` is 26 lines and owns save/reload/keep-local actions, `workspace/local_copy.rs` is 23 lines and owns local-copy status handling, `workspace/diff.rs` is 27 lines and owns conflict diff snapshot status handling, `workspace/emergency.rs` is 21 lines and owns emergency recovery plus Asset Browser open actions, and `workspace/history_reference.rs` is 23 lines and owns undo/redo/reference-open actions.

Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, and an app UI Asset workspace action subowner ownership scan. A fresh `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-ui-owner-split-0619 --message-format short --color never` remains blocked before editor code by active `zircon_runtime::scene::dynamic_scene::session` owner-split work: `session/io/mod.rs` re-exports private IO helpers, producing E0364/E0603 visibility errors. Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-19 theme action subowner split reduced `ui_asset_editor/actions/theme.rs` from 86 lines to a 25-line action-family entry. `theme/source.rs` is 33 lines and owns theme source open/promote/detach/clone/prune actions, `theme/style_rule.rs` is 21 lines and owns style rule create/extract-inline actions, `theme/pseudo_state.rs` is 34 lines and owns pseudo-state toggles, and `theme/selection.rs` is 28 lines and owns indexed theme source selection parsing plus invalid-selection status text.

Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, and an app UI Asset theme action subowner ownership scan. A fresh `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-ui-owner-split-0619 --message-format short --color never` is currently blocked before editor code by active `zircon_runtime::scene::dynamic_scene::session` owner-split work: `session/io/mod.rs` re-exports private IO helpers, producing E0364/E0603 visibility errors. Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-19 mode-preview action subowner split reduced `ui_asset_editor/actions/mode_preview.rs` from 89 lines to a 28-line action-family entry. `mode_preview/preview_preset.rs` is 40 lines and owns preview preset action ids, `mode_preview/editor_mode.rs` is 32 lines and owns editor mode action ids, `mode_preview/designer_tool.rs` is 36 lines and owns designer tool action ids, and `mode_preview/locale.rs` is 30 lines and owns locale preview action ids.

Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app UI Asset mode-preview action subowner ownership scan, scoped `git diff --check`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-ui-owner-split-0619 --message-format short --color never`, which passed with existing warning noise only (`zircon_runtime` 141 warnings, `zircon_editor` 65 warnings). Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-19 detail-dispatch split reduced `ui_asset_editor_detail_events.rs` from 104 lines to 12 lines. `ui_asset_editor_detail_events/detail_dispatch.rs` is 94 lines and owns `RetainedEditorHost::dispatch_ui_asset_detail_event(...)`, stable detail-id routing, and the unknown detail status fallback. Existing binding, collection, component-adapter, palette, preview, source, style, structure, and widget children continue to own the detail-specific mutations. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app UI Asset detail dispatch ownership scan, and scoped `git diff --check`; scoped diff check only reported the existing CRLF working-tree conversion warning. A fresh full `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never` was not claimed because concurrent `zircon_runtime` Cargo jobs were active. Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-19 detail dispatch group subowner split reduced `ui_asset_editor_detail_events/detail_dispatch.rs` from 93 lines to a 52-line group orchestration owner. `detail_dispatch/surface.rs` is 29 lines and owns widget/promote/slot/layout/palette/source detail-id routing, `detail_dispatch/style.rs` is 43 lines and owns style class/theme/rule/token routing, `detail_dispatch/preview.rs` is 31 lines and owns preview mock routing, and `detail_dispatch/binding.rs` is 39 lines and owns binding/payload/suggestion routing. The root file keeps only group ordering and unknown-detail status text.

Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, and an app UI Asset detail dispatch group subowner ownership scan. A fresh `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-ui-owner-split-0619 --message-format short --color never` remains blocked before editor code by active `zircon_runtime::scene::dynamic_scene::session` owner-split work: `session/io/mod.rs` re-exports private IO helpers, producing E0364/E0603 visibility errors. Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.
