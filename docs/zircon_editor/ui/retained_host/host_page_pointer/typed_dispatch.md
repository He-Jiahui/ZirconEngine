---
related_code:
  - zircon_editor/src/ui/retained_host/host_page_pointer/error.rs
  - zircon_editor/src/ui/retained_host/host_page_pointer/constants.rs
  - zircon_editor/src/ui/retained_host/host_page_pointer/tab_strip_geometry.rs
  - zircon_editor/src/ui/retained_host/host_page_pointer/dispatch_event.rs
  - zircon_editor/src/ui/retained_host/host_page_pointer/handle_click.rs
  - zircon_editor/src/ui/retained_host/host_page_pointer/handle_overflow_click.rs
  - zircon_editor/src/ui/retained_host/host_page_pointer/host_page_pointer_route.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/shared_pointer/host_page.rs
  - zircon_editor/src/ui/workbench/page_tabs/metrics.rs
implementation_files:
  - zircon_editor/src/ui/retained_host/host_page_pointer/error.rs
  - zircon_editor/src/ui/retained_host/host_page_pointer/tab_strip_geometry.rs
  - zircon_editor/src/ui/retained_host/host_page_pointer/dispatch_event.rs
  - zircon_editor/src/ui/retained_host/host_page_pointer/handle_click.rs
  - zircon_editor/src/ui/retained_host/host_page_pointer/handle_overflow_click.rs
plan_sources:
  - docs/plans/engine-code-review-findings-2026-06.md
  - docs/plans/zircon_editor/editor_layout/15a-page-tab-strip-overflow.md
tests:
  - cargo test -p zircon_editor --lib shared_host_page_pointer_bridge_routes_tabs_from_shared_hit_test --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0625 --color never
  - cargo test -p zircon_editor --lib root_host_page_pointer_click_uses_shared_projection_tab_slot --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0625 --color never
  - cargo test -p zircon_editor --lib narrow_tier_caps_visible_tabs_before_overflow --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0625 --message-format short --color never -- --test-threads=1 --nocapture
  - cargo test -p zircon_editor overflow --locked --jobs 1 --message-format short --color never -- --test-threads=1 --nocapture
doc_type: module-detail
status: implemented-focused-passed
---

# Host Page Pointer Typed Dispatch

`host_page_pointer/error.rs` owns the typed error for host page pointer dispatch. The inner pointer layer now returns `HostPagePointerError` instead of a bare string error. The outer callback boundary still converts the error to the existing bridge-facing string form, keeping public behavior stable while removing the local E1 review finding from the touched path.

`handle_click.rs` also uses the actual callback frame for tab hit-testing. Incoming callback points are interpreted as tab-local coordinates inside that frame, then routed through the shared projected slot geometry. This keeps painting and hit-testing in the same coordinate system.

## Overflow Route

The host-page overflow route is now part of the same typed dispatch surface. `HOST_PAGE_OVERFLOW_POINTER_INDEX` is the sentinel used by shared and native pointer dispatch when the overflow button is pressed. `handle_overflow_click.rs` runs the overflow button through the same pointer event flow as visible tabs, while `HostPagePointerRoute::Overflow` carries the hidden page indices into the host contract.

Hidden popup rows do not have visible tab frames. `handle_click.rs` therefore maps a non-visible but valid page index directly to a page route before visible-frame hit testing. This lets the popup list activate hidden pages without duplicating host-page selection logic.

## Responsive Geometry

`tab_strip_geometry.rs` consumes `ui/workbench/page_tabs/metrics.rs` for the project-path reserve and visible-tab cap. This keeps native pointer hit targets aligned with the fallback chrome projection when the Workbench enters the 640 px Narrow tier.

In Narrow tier, the host page pointer layout reserves the right-side project path width, caps visible page tabs to two, and emits an overflow slot when more pages exist. The active page is still kept visible by the same replacement rule used by the chrome projection, so hidden-row popup selection and visible-tab clicking share the same page index model.

## Boundary

This module owns host page pointer routing and typed errors only. Popup geometry, native outside-dismiss behavior, and software painting are documented in `docs/zircon_editor/ui/retained_host/host_contract/host_page_overflow_menu.md`.
