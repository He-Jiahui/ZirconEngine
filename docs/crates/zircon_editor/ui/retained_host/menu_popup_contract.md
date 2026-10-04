---
related_code:
  - zircon_editor/src/ui/retained_host/menu_popup_contract.rs
  - zircon_editor/src/ui/retained_host/popup_anchor_metrics.rs
  - zircon_editor/src/ui/retained_host/app/helpers/geometry.rs
  - zircon_editor/src/ui/retained_host/menu_pointer/constants.rs
  - zircon_editor/src/ui/retained_host/menu_pointer/build_host_menu_pointer_layout.rs
  - zircon_editor/src/ui/retained_host/menu_pointer/host_menu_pointer_layout.rs
  - zircon_editor/src/ui/retained_host/menu_pointer/popup_layout.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/scene_projection.rs
  - zircon_editor/src/ui/retained_host/host_contract/menu_popup_metrics.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_primitives/text_markers.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/menus/popup.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/menus/rows.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/menus/rows/text_layout.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/native_panes.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/native_panes/scrollbar.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/native_panes/scrollbar/paint.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/menu_geometry/popup/entry/root/frame.rs
implementation_files:
  - zircon_editor/src/ui/retained_host/menu_popup_contract.rs
  - zircon_editor/src/ui/retained_host/app/helpers/geometry.rs
  - zircon_editor/src/ui/retained_host/menu_pointer/constants.rs
  - zircon_editor/src/ui/retained_host/menu_pointer/build_host_menu_pointer_layout.rs
  - zircon_editor/src/ui/retained_host/menu_pointer/host_menu_pointer_layout.rs
  - zircon_editor/src/ui/retained_host/menu_pointer/popup_layout.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/scene_projection.rs
  - zircon_editor/src/ui/retained_host/host_contract/menu_popup_metrics.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_primitives/text_markers.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/menus/popup.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/menus/rows.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/menus/rows/text_layout.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/native_panes.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/native_panes/scrollbar.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/native_panes/scrollbar/paint.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/menu_geometry/popup/entry/root/frame.rs
plan_sources:
  - docs/plans/zircon_editor/editor_layout/15-component-standardization-from-primitives.md
  - docs/plans/zircon_editor/editor_layout/15d-composite-density-and-alignment.md
  - docs/plans/zircon_runtime/text/index.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - window_menu_viewport_consumes_shared_height_and_clamped_scroll
  - ordinary_menu_ignores_stale_window_scroll_state
  - popup_width_follows_longest_runtime_measured_label_and_shortcut
  - popup_width_clamps_to_available_shell_width
  - popup_content_height_uses_shared_slate_row_density
  - shortcut_column_reserves_non_overlapping_runtime_measured_label_clip
  - scrolled_row_label_clip_stays_inside_popup_viewport
  - shared_menu_pointer_layout_content_measures_root_popup_widths
  - cargo check -p zircon_editor --lib --no-default-features --locked --jobs 1 (2026-07-10 passed)
  - exact production contract standalone Rust test entry (2026-07-10 6/6 passed)
  - focused Cargo responsive-menu groups (2026-07-10 15/15 passed)
  - focused Cargo shared scrollbar group (2026-07-10 7/7 passed, 1 ignored capture)
  - M3 GUI capture route (2026-07-10 1/1 passed)
doc_type: module-detail
status: implemented-focused-and-window-visual-passed
---

# Retained Host Menu Popup Contract

This module is the shared responsive contract for native Workbench root menus. It owns the Slate-density popup padding, 28-pixel row height, row gap, minimum viewport height, Window-menu identity, content-height calculation, content-measured width, and Window-menu viewport/scroll resolution. The existing popup-anchor owner remains authoritative for the 3-pixel anchor gap and 8-pixel shell margin; the menu contract aliases those values instead of creating another local literal table.

`scene_projection.rs` measures visible labels and shortcuts through the retained runtime text interface before it writes `HostMenuChromeMenuData.popup_width_px`. `build_host_menu_pointer_layout.rs` performs the same measurement and stores one width per root menu in `HostMenuPointerLayout.popup_widths`. `popup_layout.rs` consumes those projected widths for shared pointer rows, so painting, hover, click, right-edge clamp, and extension-menu behavior no longer diverge when a localized or extension-provided label is wider than the old fixed table.

Window-menu height and scroll are separate from authored full-content height. `root_menu_popup_viewport(...)` applies `HostMenuStateData.window_menu_popup_height_px` and clamps scroll only for the Window root; other menus explicitly ignore stale Window scroll state. The native painter, native pointer containment, and damage geometry consume the resolved viewport rather than silently repainting the full scene height.

Fallback text markers now add the host `text_clip_guard` to runtime-measured text frames. Native menu rows measure their shortcut column instead of reserving a fixed 34 pixels, right-align the shortcut with the shared inset, and clip the label column before the shared label/shortcut gap. This keeps the final glyph, shortcut text, and scrolled partial rows inside the popup without pixel-positioning the content for a single screenshot size.

The production editor library passed a fresh Cargo check. The initial viewport test failed against the old full-content behavior, then the exact production contract file passed six standalone Rust tests after implementation. After the concurrent test-lock owner restored its `LockResult` surface, the fresh editor test binary passed 15 responsive-menu assertions plus seven shared scrollbar assertions. The M3 capture route passed and refreshed `docs/tests/editor/editor-window-m3-menu-popup-svg-icons-900x620.png` (61,129 bytes, SHA256 `594723845CB53484D280530D9B58BD76BF071D4E57039C34A875688FE5CC12BB`). Manual review confirms the clipped full-height list is replaced by a six-row 192-pixel viewport, `Preset 03` and `active` render completely in separate columns, and the shared 8-pixel/4-pixel-radius scrollbar communicates the scrolled position. The external target known-output scan found no matching PNG.
