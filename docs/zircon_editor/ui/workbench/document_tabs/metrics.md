---
related_code:
  - zircon_editor/src/ui/workbench/document_tabs/mod.rs
  - zircon_editor/src/ui/workbench/document_tabs/metrics.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/chrome_template_projection/dock_header.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/chrome_template_projection/tests.rs
  - zircon_editor/src/ui/retained_host/tab_drag/tab_width.rs
  - zircon_editor/src/ui/retained_host/document_tab_pointer/constants.rs
  - zircon_editor/src/ui/retained_host/document_tab_pointer/host_document_tab_pointer_bridge.rs
  - zircon_editor/src/ui/retained_host/document_tab_pointer/host_document_tab_pointer_bridge_sync.rs
  - zircon_editor/assets/ui/editor/workbench_dock_header.zui
implementation_files:
  - zircon_editor/src/ui/workbench/document_tabs/metrics.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/chrome_template_projection/dock_header.rs
  - zircon_editor/src/ui/retained_host/tab_drag/tab_width.rs
  - zircon_editor/src/ui/retained_host/document_tab_pointer/constants.rs
  - zircon_editor/assets/ui/editor/workbench_dock_header.zui
plan_sources:
  - docs/plans/zircon_editor/editor_layout/15-component-standardization-from-primitives.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - cargo test -p zircon_editor --lib document_tab --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0626 --message-format short --color never -- --test-threads=1 --nocapture
  - cargo test -p zircon_editor --lib template_icon_buttons --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0626 --message-format short --color never -- --test-threads=1 --nocapture
  - cargo test -p zircon_editor --lib close_outline_maps_to_close_glyph --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0626 --message-format short --color never -- --test-threads=1 --nocapture
  - cargo test -p zircon_editor --lib template_buttons --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0626 --message-format short --color never -- --test-threads=1 --nocapture
  - cargo build -p zircon_editor --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0626 --message-format short --color never
  - cargo test -p zircon_editor capture_m3_gui_acceptance_visual_artifacts --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0626 --message-format short --color never -- --ignored --test-threads=1 --nocapture
  - cargo fmt --check --
  - static scan: document-tab production width paths no longer contain `TITLE_WIDTH_PER_CHAR`, `document_tab_preferred_width(`, `estimate_text_width`, or `ascii_char_width`
  - focused Cargo for the 2026-07-04 runtime-measured document-tab tests remains deferred while unrelated cargo/rustc lanes are active
doc_type: module-detail
status: implemented-rustfmt-static-visual-cargo-deferred
---

# Document Tab Metrics

`document_tabs/metrics.rs` is the shared owner for Workbench document-tab strip geometry. It keeps the tab strip origin, readable tab widths, tab height, inter-tab gap, close-button size, and close-button right inset in one place so the dock-header painter and retained document-tab pointer bridge do not drift.

The module is intentionally narrow. It does not own document state, tab activation, drag behavior, or close dispatch. It only answers the geometry questions that both paint projection and hit testing need to agree on.

## Current Contract

- Closeable document tabs use `DOCUMENT_CLOSEABLE_TAB_MIN_WIDTH`, which keeps labels such as `Asset Browser` readable before any later overflow work.
- Plain document tabs use `DOCUMENT_TAB_MIN_WIDTH`, and all preferred widths are clamped by `DOCUMENT_TAB_MAX_WIDTH`.
- `DOCUMENT_TAB_TITLE_FONT_SIZE` is the shared font-size contract for document-tab title measurement.
- `document_tab_preferred_width_from_title_width(title_width, closeable)` accepts a runtime-measured title width and adds the appropriate close-control reserve instead of estimating from character count.
- `document_tab_close_x(tab_x, tab_width)` is the canonical close-button x coordinate. The dock-header fallback projection and retained pointer tests consume this same helper.
- The close button uses a 20 px icon-button frame with toolbar context styling, so its normal state stays transparent and only the close glyph remains visible.

## Behavior Model

`chrome_template_projection/dock_header.rs` builds fallback document tabs from current tab data. It starts at the shared strip x coordinate, measures each title with `measure_runtime_text_width(..., DOCUMENT_TAB_TITLE_FONT_SIZE)`, calls `document_tab_preferred_width_from_title_width(...)`, places the close button with `document_tab_close_x(...)`, and advances by `DOCUMENT_TAB_GAP`.

`retained_host/tab_drag/tab_width.rs` uses the same runtime text measurement path and preferred-width helper for document-tab drag hitboxes. This keeps visible tab frames, close-button placement, and drag midpoint estimation aligned for file names such as `editor base.zui` and `folder-open-line.svg`.

`document_tab_pointer/constants.rs` aliases the same shared values for the retained pointer bridge. This keeps close hit frames aligned with the visible close button and avoids the earlier split where paint used one tab width while pointer routing used another.

`workbench_dock_header.zui` remains the authored template seed for the static dock-header asset. Its document-tab widths now match the shared metric defaults: closeable tabs are wide enough for `Asset Browser`, and the close controls use a clean icon-only frame rather than a permanent inset button surface.

## Edge Cases and Constraints

- This owner is for dock/document tabs, not the top host page tabs. Top page-strip overflow and tier behavior stay in `ui/workbench/page_tabs/metrics.rs`.
- The width helper accepts text width that has already been measured by the retained-host runtime text path. It is a geometry guard, not a text shaping replacement. Button label rendering still measures the actual node font size before drawing.
- The template asset may declare sample frames, but production fallback projection and pointer routing must use the shared Rust owner when live tab data is projected.

## Test Coverage

The focused document-tab tests verify readable closeable tab width, measured-width clamp behavior, and close-button inset math. Existing retained document-tab pointer tests were updated to click the close center computed from shared metrics. Chrome projection tests assert the projected dock-header nodes keep readable document tabs, measure file-name labels through runtime text width, and keep the transparent close control inside the measured tab frame. Tab-drag tests assert document-tab drag width equals the same measured preferred width and that dock-tab drag width tracks wide versus narrow runtime glyphs.

The button and icon-button regressions cover the supporting visual behavior: close icon names map to the `Close` glyph instead of the more-menu glyph, dock-tab close buttons use toolbar context without a persistent panel surface, and button label measurement uses the declared node font size so `Asset Browser` does not get under-measured at 12 px. The M3 screenshot harness refreshed `docs/tests/editor/editor-window-m3-workbench-900x620.png` after those changes, with build outputs kept in `D:\cargo-targets\zircon-editor-components-0626` rather than repo `target`.

For the 2026-07-04 runtime text follow-up, `docs/tests/runtime/text/runtime_text_editor_document_tab_runtime_measure_preview_20260704.png` records the document-tab width proof for the latest editor crop. Focused Cargo remains deferred until the unrelated compile lanes are idle, so this slice currently relies on rustfmt, static scans, module tests added in source, and the retained visual artifact.
