---
related_code:
  - zircon_editor/src/ui/workbench/page_tabs/metrics.rs
  - zircon_editor/src/ui/workbench/page_tabs/mod.rs
  - zircon_editor/src/ui/retained_host/host_contract/host_page_overflow_menu.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/host_interaction/page_overflow.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/host_components/menus.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/button_dispatch/page_overflow_menu.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/routing/chrome/tabs/host_page.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/routing/chrome_route.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_keyboard/target/page_overflow.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_keyboard/target/model.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_keyboard/dispatch/actions.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/scene_layers/overlay/page_overflow.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/scene_layers/overlay.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/scene_layers/overlay/componentized.rs
  - zircon_editor/src/ui/retained_host/app/workbench_pointer/shell_chrome/host_page.rs
  - zircon_editor/src/ui/retained_host/ui/apply_presentation/scene_conversion.rs
implementation_files:
  - zircon_editor/src/ui/workbench/page_tabs/metrics.rs
  - zircon_editor/src/ui/retained_host/host_contract/host_page_overflow_menu.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/host_interaction/page_overflow.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/button_dispatch/page_overflow_menu.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_keyboard/target/page_overflow.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_keyboard/dispatch/actions.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/scene_layers/overlay/page_overflow.rs
  - zircon_editor/src/ui/retained_host/app/workbench_pointer/shell_chrome/host_page.rs
plan_sources:
  - user: 2026-06-25 Optimize Zircon editor UI from primitive components upward before composing drawers and windows
  - docs/plans/zircon_editor/editor_layout/15-component-standardization-from-primitives.md
  - docs/plans/zircon_editor/editor_layout/15a-page-tab-strip-overflow.md
  - docs/plans/zircon_editor/editor_layout/15d-composite-density-and-alignment.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - cargo build -p zircon_editor --locked --jobs 1 --message-format short --color never
  - cargo fmt -p zircon_editor --check
  - cargo test -p zircon_editor overflow --locked --jobs 1 --message-format short --color never -- --test-threads=1 --nocapture
  - cargo test -p zircon_editor --lib fallback_page_chrome_narrow_tier_caps_visible_tabs_before_project_path --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0625 --message-format short --color never -- --test-threads=1 --nocapture
  - cargo test -p zircon_editor --lib narrow_tier_caps_visible_tabs_before_overflow --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0625 --message-format short --color never -- --test-threads=1 --nocapture
  - cargo test -p zircon_editor capture_host_page_overflow_menu_visual_artifact --locked --jobs 1 --message-format short --color never -- --ignored --test-threads=1 --nocapture
  - cargo test -p zircon_editor capture_m3_gui_acceptance_visual_artifacts --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0625 --message-format short --color never -- --ignored --test-threads=1 --nocapture
  - cargo test -p zircon_editor --lib host_page_overflow_keyboard --locked --jobs 1 -- --test-threads=1 --nocapture
  - cargo test -p zircon_editor --lib capture_host_page_overflow_keyboard_visual_artifact --locked --jobs 1 -- --ignored --test-threads=1 --nocapture
  - docs/tests/editor/editor-window-m3-host-page-overflow-420x260.png
  - docs/tests/editor/editor-window-m3-host-page-overflow-keyboard-640x420.png
doc_type: module-detail
status: implemented-focused-passed
---

# Host Page Overflow Menu

## Purpose

The host page overflow menu is the retained-host popup for main-page tabs that cannot stay visible in the page tab strip. It closes the `15a` popup portion by making the overflow button interactive, rendering a compact selectable list, and routing hidden-page row selection back through the existing host-page tab activation path.

This is a component-first slice: the visible behavior is a small popup/select-list built from shared menu density before it is reused by wider drawer and window breakpoint work.

## Related Files

`host_page_overflow_menu.rs` owns popup geometry. It right-aligns the popup under the projected overflow button, calculates row frames from shared menu popup metrics, and exposes hit helpers used by native pointer dispatch.

`data/host_interaction/page_overflow.rs` owns the open/hover state stored in the host contract. The state is copied into `HostWindowPresentationData` during presentation snapshotting and preserved through retained-host presentation apply.

`paint_workbench_renderer/scene_layers/overlay/page_overflow.rs` owns the software-rendered popup surface. Both the regular host scene overlay and the componentized overlay call it, so the visual artifact sees the same layer ordering as the running retained host.

`native_pointer/button_dispatch/page_overflow_menu.rs` owns native pointer behavior for popup rows, padding, outside dismissal, and row selection. It delegates selected hidden pages back to `host_page_pointer_clicked(...)` instead of creating a second page-selection model.

`native_keyboard/target/page_overflow.rs` projects the procedural popup into the same `PopupKeyboardTarget` contract used by authored option and menu popups. Hidden page indices remain the activation identity, while labels feed the shared case-insensitive prefix search. `native_keyboard/dispatch/actions.rs` applies hover, accept, and cancel directly through `UiHostContext`, preserving the existing host-page callback as the only activation path.

## Behavior Model

Clicking the host page overflow button toggles `HostPageOverflowMenuStateData.open`. Opening is paint-only at the bridge boundary; the actual hidden page list comes from `HostPageChromeData.overflow_hidden_tab_indices`, which is projected from the same page chrome data that provides the visible tab and overflow-button frames.

When the popup is open, primary pointer dispatch checks it before regular menu, body, and chrome dispatch. A row hit closes the popup and invokes the real host-page click callback for the row's page index. A click inside popup padding is consumed. A click outside the popup closes it. Clicking the overflow button itself is left to the chrome route so the same control can close or reopen the menu.

Keyboard discovery gives the procedural host-page popup priority while it is open. Arrow Down/Up wrap through hidden rows, Home/End jump to the boundary, typed text selects the next matching page label, Enter activates through `host_page_pointer_clicked`, and Escape dismisses without activation. When no row is highlighted, the first Down selects the first row and the first Up selects the last row instead of skipping an item.

## Design And Rationale

The popup reuses the menu popup metrics introduced for S15.4 for row height, padding, border, shell margin, and anchor gap instead of adding a host-page-specific density table. Its width is now owned by `ui/workbench/page_tabs/metrics.rs` as `MAIN_PAGE_TAB_OVERFLOW_POPUP_WIDTH`, so the page-tab strip and its overflow popup share one component-level width token. The result is a selectable list with the same 1 px neutral border, compact rows, muted hover/selected surfaces, and no decorative shadow.

The route is intentionally split:

- `host_page_pointer` owns shared hit routes and the overflow sentinel.
- The host contract owns presentation state, popup geometry, native pointer ordering, and paint.
- `shell_chrome/host_page.rs` owns the final application state transition from overflow trigger or hidden page row to active page.

This keeps root modules as wiring only and avoids a parallel "overflow selected page" state model.

## Edge Cases And Constraints

- Hidden popup rows can select pages that are not present in `layout.tabs`; `handle_click.rs` maps non-visible indices directly to a page route before visible-tab hit testing.
- The active hidden page row is drawn as selected, and hover state can be represented independently for later pointer-move work.
- The overflow route is not draggable. Drag payload resolution explicitly ignores `HostPageOverflow`.
- The visual screenshot test was added to the existing retained-host screenshot harness because that harness already owns PNG encoding and artifact placement. The production implementation is split into small owners; if another host-page popup visual case is added, the screenshot helper should be extracted from the oversized `visual_screenshot.rs` test file.

## Test Coverage

`shared_host_page_pointer_bridge_routes_overflow_button_from_shared_hit_test` covers shared pointer routing from the overflow button.

`shared_host_page_overflow_click_opens_popup_and_hidden_page_selection_activates_page` covers opening the popup and selecting a hidden page through shared dispatch.

`native_host_pointer_click_routes_host_page_overflow_button_and_popup_rows` covers native host routing, including the overflow chrome route and popup row selection.

`capture_host_page_overflow_menu_visual_artifact` writes `docs/tests/editor/editor-window-m3-host-page-overflow-420x260.png` and compares closed/open pixels inside the popup frame.

`host_page_overflow_keyboard_navigates_searches_and_accepts_hidden_pages` and `host_page_overflow_keyboard_wraps_backward_and_escape_closes_without_activation` cover the procedural popup's shared keyboard contract. `capture_host_page_overflow_keyboard_visual_artifact` writes the 640×420 keyboard-highlighted visual evidence under `docs/tests/editor`.

`fallback_page_chrome_narrow_tier_caps_visible_tabs_before_project_path` and `narrow_tier_caps_visible_tabs_before_overflow` cover the tier-forced overflow policy now consumed by both fallback chrome projection and retained host pointer geometry.

## Open Issues Or Follow-up

The popup/select-list path now includes pointer and keyboard activation. Remaining `15a/15e` work is broader breakpoint polish and long-list viewport scrolling; keyboard navigation must continue to reuse this target rather than introduce a page-tab-specific key handler.
