---
related_code:
  - zircon_editor/src/ui/workbench/page_tabs/mod.rs
  - zircon_editor/src/ui/workbench/page_tabs/metrics.rs
  - zircon_editor/src/ui/workbench/autolayout/layout_tier.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/chrome_template_projection.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/chrome_template_projection/tests.rs
  - zircon_editor/src/ui/retained_host/host_page_pointer/host_page_pointer_item.rs
  - zircon_editor/src/ui/retained_host/host_page_pointer/tab_strip_geometry.rs
  - zircon_editor/src/ui/retained_host/host_page_pointer/build_host_page_pointer_layout.rs
  - zircon_editor/src/ui/retained_host/host_contract/host_page_overflow_menu.rs
implementation_files:
  - zircon_editor/src/ui/workbench/page_tabs/metrics.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/chrome_template_projection.rs
  - zircon_editor/src/ui/retained_host/host_page_pointer/host_page_pointer_item.rs
  - zircon_editor/src/ui/retained_host/host_page_pointer/tab_strip_geometry.rs
plan_sources:
  - docs/plans/zircon_editor/editor_layout/15-component-standardization-from-primitives.md
  - docs/plans/zircon_editor/editor_layout/15a-page-tab-strip-overflow.md
  - docs/plans/zircon_editor/editor_layout/15e-domain-breakpoint-adaptation.md
tests:
  - cargo test -p zircon_editor --lib fallback_page_chrome_keeps_medium_width_tabs_readable_before_overflow --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0625 --color never
  - cargo test -p zircon_editor --lib fallback_page_chrome_narrow_tier_caps_visible_tabs_before_project_path --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0625 --message-format short --color never -- --test-threads=1 --nocapture
  - cargo test -p zircon_editor --lib fallback_page_chrome_wide_tier_does_not_force_overflow_when_tabs_fit --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0625 --message-format short --color never -- --test-threads=1 --nocapture
  - cargo test -p zircon_editor --lib shared_host_page_pointer_bridge_routes_tabs_from_shared_hit_test --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0625 --color never
  - cargo test -p zircon_editor --lib root_host_page_pointer_click_uses_shared_projection_tab_slot --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0625 --color never
  - cargo test -p zircon_editor --lib narrow_tier_caps_visible_tabs_before_overflow --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0625 --message-format short --color never -- --test-threads=1 --nocapture
  - cargo test -p zircon_editor capture_m3_gui_acceptance_visual_artifacts --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0625 --message-format short --color never -- --ignored --test-threads=1 --nocapture
  - cargo build -p zircon_editor --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0625 --message-format short --color never
  - cargo fmt --check --
  - static scan: page-tab touched paths no longer contain `TITLE_WIDTH_PER_CHAR`, `main_page_tab_preferred_width(`, or title character-count width formulas
  - focused Cargo for the 2026-07-04 runtime-measured host page-tab tests remains deferred while unrelated cargo/rustc lanes are active
doc_type: module-detail
status: implemented-rustfmt-static-visual-cargo-deferred
---

# Main Page Tab Metrics

`page_tabs/metrics.rs` is the shared owner for main document tab strip dimensions. It keeps strip offset, min/max tab width, tab height, inter-tab gap, overflow-control width, overflow-popup width, project-path reserve width, and preferred-width calculation in one place so retained host hit-testing and chrome projection do not drift.

The owner is intentionally small. It does not create tab state, menu contents, or docking behavior. Those remain in host page pointer, window chrome projection, and later overflow-popup owners.

## Current Contract

- Visible main page tabs must not shrink below `MAIN_PAGE_TAB_MIN_WIDTH`.
- `MAIN_PAGE_TAB_TITLE_FONT_SIZE` is the shared title-measurement font size for host page tabs.
- `main_page_tab_preferred_width_from_title_width(...)` accepts a retained runtime text measurement and clamps the measured title plus chrome reserve between min and max width.
- `main_page_project_path_width(...)` is the canonical right-side reserve used before tab allocation, keeping the project path from competing with visible page tabs.
- `main_page_tab_visible_cap_for_width(...)` maps the shared Workbench layout tier to tab-strip degradation. Narrow tier caps visible page tabs to two before overflow; Regular and Wide tiers keep all tabs visible when they fit.
- Host page pointer geometry and Workbench chrome projection consume the same constants, tier policy, title font size, and runtime-measured preferred width.
- `MAIN_PAGE_TAB_OVERFLOW_POPUP_WIDTH` is the page-tab owner token consumed by the host-page overflow popup geometry.

## Validation

The focused regression set verifies medium-width chrome keeps readable tabs before overflow, narrow chrome caps visible tabs before the project path reserve, wide chrome does not force overflow when tabs fit, shared pointer hit-testing routes the same tabs that projection draws, and root host clicks use the projected tab slot instead of a local synthetic frame.

The 2026-06-25 screenshot pass refreshed `docs/tests/editor/editor-window-m3-svg-icon-scale-small-640x420.png`, `docs/tests/editor/editor-window-m3-workbench-900x620.png`, and `docs/tests/editor/editor-window-m3-svg-icon-scale-large-1260x780.png`. The 640 capture now shows the page strip using visible tabs plus overflow while the right Inspector content stays folded away; the 1260 capture keeps wide tabs visible when there is room. A final `zircon_editor` build passed from the same external target directory with existing warning noise only.

The 2026-07-04 runtime text follow-up removes character-count width estimation from the host page-tab owner. Fallback page chrome measures `TabData.title` with `measure_runtime_text_width(..., MAIN_PAGE_TAB_TITLE_FONT_SIZE)` before it allocates visible tab frames. `host_page_pointer/tab_strip_geometry.rs` receives page titles through `HostPagePointerItem` and uses the same measurement helper when building shared hit frames, so pointer routing and rendered page tabs stay aligned for file-like labels such as `editor base.zui` and `folder-open-line.svg`. Evidence is recorded in `docs/tests/runtime/text/runtime_text_editor_page_tab_runtime_measure_preview_20260704.png`; focused Cargo is still deferred until unrelated compile lanes are idle.
