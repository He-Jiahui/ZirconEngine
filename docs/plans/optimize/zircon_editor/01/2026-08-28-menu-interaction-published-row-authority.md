# Menu interaction published-row authority review

Date: 2026-08-28

Status: current-source review and deterministic pressure model complete; production
cutover and product timing remain pending.

## Outcome

The old review finding about `menu_search_options` is no longer current-source
evidence. That symbol and its event-time recursive `MenuSearchOption` projection
have been removed by another owner. The replacement pointer bridge is materially
better: it retains `popup_items`, builds `popup_route_indices` only when popup
content changes, and projects a row from geometry plus a stable route index.

The interaction system is still not one retained authority. Two product paths now
re-materialize a whole visible row set during interaction:

1. `apply_menu_row_hover` iterates and clones every `structured_menu_items` row,
   changes visual flags on one row, and publishes a new `VecModel`.
2. `menu_popup_keyboard_target` constructs a new `Vec<PopupKeyboardRow>` on each
   target resolution, including repeated `SharedString` clones and a second scan
   to find the active row.

Submenu-path changes also call `HostMenuPointerBridge::rebuild_surface`, replacing
the complete menu pointer `UiSurface`, dispatcher, and route-intent map. The rebuilt
surface is small because rows are projected arithmetically, but it is still the
wrong ownership model for a local popup-stack change.

## Current-source evidence

- `host_contract/paint_template_nodes/template_node_pipeline/hover.rs:60-82`
  stores hover/focus/pressed in row payload and clones the whole model to change it.
- `host_contract/native_keyboard/target/menu.rs:13-37` reads every row and creates
  an owned keyboard vector for one key operation.
- `host_contract/native_keyboard/target/selection.rs:18-20` scans that freshly
  created vector and clones the selected row again.
- `menu_pointer/host_menu_pointer_bridge_popup_items.rs:21-30` correctly skips
  unchanged popup content and publishes a route index only on content change.
- `menu_pointer/host_menu_pointer_bridge_project_route.rs:102-166` performs a
  depth-bounded popup projection and a route-index lookup. This is the retained
  behavior to preserve.
- `menu_pointer/host_menu_pointer_bridge_rebuild_surface.rs:22-134` constructs a
  new `UiSurface`, dispatcher, and route map for open/close/submenu stack changes.
- `host_contract/paint_workbench_renderer/menus/popup.rs:95` clones a `ModelRc`.
  This is a reference-counted handle clone, not evidence of deep row copying.

All listed Zircon owner files are shared dirty paths. This review deliberately did
not edit them.

## Root cause

Static menu content and transient interaction state have the same ownership:
`TemplatePaneMenuItemData` combines four content strings and semantic flags with
`focused`, `hovered`, and `pressed`. A one-row visual change therefore needs a new
row value and, because the public model is a `VecModel`, current code rebuilds the
entire visible model.

Keyboard navigation has a separate owned schema (`PopupKeyboardRow`) instead of a
borrowed/indexed view over the same published rows. Pointer routing, paint, hover,
keyboard navigation, and submenu geometry consequently agree by repeated
projection rather than by sharing an immutable generation.

## Unreal reference

`dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/MultiBox/
SMenuEntryBlock.cpp` provides the relevant shape:

- lines 747-756 keep hover on the row widget and forward enter/leave locally;
- lines 1128-1136 schedule submenu opening with an active timer;
- lines 1371-1442 change the active menu anchor rather than rebuilding menu rows;
- lines 1454-1466 navigate the existing widget sequence with
  `SMultiBoxWidget::FocusNextWidget`.

Zircon does not need to copy Slate's widget classes, but it should retain the same
authority boundaries: published entries, row-local interaction state, and a popup
stack that changes independently from entry content.

## Target architecture

Publish one immutable structure per menu-content generation:

```text
PublishedMenuInteractionIndex
  generation
  rows: Arc<[PublishedMenuRow]>
  row_key -> row index
  eligible_prev / eligible_next
  parent / first_child / child_count
  normalized typeahead keys

MenuInteractionState
  hovered_row_key
  focused_row_key
  pressed_row_key
  open_submenu_row_keys
```

`PublishedMenuRow` owns content only: identity, action, label, shortcut, enabled,
separator, checked, parent, and child range. Hover/focus/pressed must not mutate or
republish this array.

Pointer, keyboard, paint, and submenu code consume the same generation and stable
row key:

- pointer motion computes a row key and emits at most two dirty row keys (old/new);
- paint derives visual flags from `MenuInteractionState` without cloning content;
- next/previous keyboard movement follows published eligible adjacency in `O(1)`;
- typeahead scans normalized published keys without allocating row objects;
- submenu changes patch the active layer stack and its hit/damage records only;
- content/eligibility/schema changes rebuild or patch the published index before
  frame publication, never in the input callback.

The existing `popup_route_indices` is a useful interim index, but its
`HashMap<Vec<usize>, usize>` key still hashes and clones path vectors. The hard cut
should use a compact stable row key and retain parent/child adjacency directly.

## Deterministic pressure model

`tools/editor_menu_interaction_projection_pressure.py` counts worst-case row
accesses/materializations. It is not CPU or latency timing. The suite uses 1,000
hover transitions, 1,000 keyboard events, 100 submenu transitions, depth four,
and seven menu buttons.

| Published rows | Current operation units | Target operation units | Reduction |
| ---: | ---: | ---: | ---: |
| 20 | 101,300 | 2,440 | 41.52x |
| 200 | 1,001,300 | 2,800 | 357.61x |
| 10,000 | 50,001,300 | 22,400 | 2,232.20x |

For 10,000 rows, current event-owned projection performs 10,000,000 hover row
materializations plus 10,000,000 keyboard row materializations. The target performs
10,000 publication materializations and zero event-time row materializations.
The model intentionally excludes string length, allocator, paint, GPU, and native
event-loop cost. Typeahead character comparisons remain `O(V)` unless a prefix
index is justified by product profiles; they still must not allocate rows.

Artifact:
`E:\zircon-profiles\editor-menu-interaction-projection-pressure-20260828.json`

## Implementation order

1. Add counters before behavior changes: published-row rebuild/patch count,
   event-time row materialization count, hover dirty-row count, keyboard target
   build count, popup surface full-rebuild/patch count, and generation mismatch.
2. Introduce `PublishedMenuInteractionIndex` at the content publication boundary.
   Prove pointer, keyboard, paint, and submenu paths consume the same generation.
3. Move hover/focus/pressed out of `TemplatePaneMenuItemData` publication. Patch
   old/new row damage and visual state only.
4. Change keyboard targets to borrow an `Arc` plus row indices. Publish eligible
   adjacency; keep typeahead allocation-free.
5. Replace submenu `UiSurface` replacement with popup-layer patching. Full rebuild
   is allowed only for content generation or base geometry invalidation.
6. Remove the old event-owned vector paths in the same cut. Do not leave a fallback
   that silently re-materializes rows on the input thread.

## Acceptance

- 10,000 nested rows and 1,000 pointer transitions create zero event-time row
  vectors, `String`s, or `SharedString` content clones.
- 1,000 next/previous keyboard events create zero row vectors and visit one
  adjacency edge per accepted move.
- Hover changes dirty no more than the previous and next row, plus popup chrome if
  required.
- Opening a submenu does not replace the base menu pointer surface, dispatcher, or
  content index.
- Pointer, keyboard, paint, and action dispatch report identical generation and row
  key; stale generations are rejected before dispatch.
- Product profiling records p50/p95/p99 input-to-hover-paint and input-to-submenu-
  visible latency, allocator deltas, CPU, RSS, row materialization counts, and
  damage rectangles under 20, 200, and 10,000-row fixtures.
- The current menu behavior remains equivalent for disabled/separator rows,
  checked state, nested submenus, scrolling, dismissal, typeahead wrap, and action
  identity.

## Validation status

Static source guards and the deterministic Python model can be run without Cargo.
Managed Rust and Editor product-path validation remain pending lane authorization
and current-source closure. No product timing claim is made by this report.
