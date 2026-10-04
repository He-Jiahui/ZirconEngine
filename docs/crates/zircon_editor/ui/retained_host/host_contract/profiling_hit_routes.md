---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/profiling_hit_routes.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_hit_routes/routes.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_hit_routes/chrome.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_hit_routes/geometry.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_hit_routes/tabs.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_hit_routes/tabs/document.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_hit_routes/tabs/drawer.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_hit_routes/tabs/floating.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_hit_routes/tabs/host_page.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_hit_routes/tabs/shared.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_hit_routes/template.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_hit_routes/template/floating.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_hit_routes/template/pane.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_hit_routes/template/route_check.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_hit_routes/viewport_toolbar.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_hit_routes/viewport_toolbar/floating.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_hit_routes/viewport_toolbar/pane.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_hit_routes/viewport_toolbar/route_check.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/geometry/
implementation_files:
  - zircon_editor/src/ui/retained_host/host_contract/profiling_hit_routes.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_hit_routes/routes.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_hit_routes/chrome.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_hit_routes/geometry.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_hit_routes/tabs.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_hit_routes/tabs/document.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_hit_routes/tabs/drawer.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_hit_routes/tabs/floating.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_hit_routes/tabs/host_page.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_hit_routes/tabs/shared.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_hit_routes/template.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_hit_routes/template/floating.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_hit_routes/template/pane.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_hit_routes/template/route_check.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_hit_routes/viewport_toolbar.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_hit_routes/viewport_toolbar/floating.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_hit_routes/viewport_toolbar/pane.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_hit_routes/viewport_toolbar/route_check.rs
plan_sources:
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - user: 2026-06-18 editor UI architecture implementation, feature first and tests deferred
tests:
  - cargo fmt -p zircon_editor --check
  - profiling-hit-routes dispatch ownership scan
  - profiling-hit-routes tab-family ownership scan
  - profiling-hit-routes template pane/floating/route-check ownership scan
  - profiling-hit-routes viewport-toolbar pane/floating/route-check ownership scan
  - scoped trailing whitespace scan
  - scoped git diff --check
doc_type: module-detail
---

# Profiling Hit Routes

`profiling_hit_routes.rs` is the retained-host profiling route hit-test entry used by profiling artifact geometry to verify whether sampled route frames still contain their declared hit point. It is now a structural entry that wires route-specific children and re-exports the dispatch function.

`profiling_hit_routes/routes.rs` owns `route_contains_profile_frame(...)` and the route-kind dispatch table. The specialized children keep concrete checks split by route family: `chrome.rs` owns resize splitter and activity rail routes, `viewport_toolbar.rs` orchestrates document/side/bottom/floating viewport-toolbar routes, `template.rs` orchestrates document/side/bottom/floating template-node routes, and `geometry.rs` owns shared containment/translation helpers. Under template routes, `template/pane.rs` owns pane prefix/content/body-toolbar checks, `template/floating.rs` owns floating-window traversal, and `template/route_check.rs` owns runtime surface-frame top-hit/control-id comparison. Under viewport toolbar routes, `viewport_toolbar/pane.rs` owns scene/game toolbar eligibility, prefix/content checks, and toolbar frame creation; `viewport_toolbar/floating.rs` owns floating-window traversal; `viewport_toolbar/route_check.rs` owns viewport-toolbar surface hit/control-id comparison.

`profiling_hit_routes/tabs.rs` is now a structural tab-route entry. `tabs/document.rs` owns document-dock tab routing, `drawer.rs` owns left/right/bottom drawer tab routing, `floating.rs` owns floating-window tab routing, `host_page.rs` owns host-page tab routing, and `shared.rs` owns common tab row containment against translated frames.

The 2026-06-20 dispatch split reduced `profiling_hit_routes.rs` from 27 lines to a 7-line structural entry. `routes.rs` is 23 lines and owns the route-kind match. Validation used `cargo fmt -p zircon_editor --check`, a root ownership scan confirming presentation import, route-kind match, and route dispatch body no longer live in `profiling_hit_routes.rs`, a scoped trailing-whitespace scan, and scoped `git diff --check`. Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction, and package-level Cargo check is still waiting on unrelated `zircon_runtime` render-history compile errors.

The 2026-06-21 tab-family split reduced `profiling_hit_routes/tabs.rs` from 128 lines to a 9-line structural entry. `tabs/{document,drawer,floating,host_page,shared}.rs` now owns the concrete tab route families while preserving the helper names used by `routes.rs`. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, a profiling hit-route tab-family ownership scan, scoped trailing-whitespace scan, and scoped `git diff --check`; package-level Cargo check and full Cargo tests remain deferred to the milestone testing stage per the user's instruction.

The 2026-06-21 template route pane/floating/route-check split reduced `profiling_hit_routes/template.rs` from 105 lines to a 53-line template route orchestration entry. `template/pane.rs` owns pane prefix/content/body-toolbar checks, `template/floating.rs` owns floating-window iteration, and `template/route_check.rs` owns runtime surface-frame top-hit comparison. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, a profiling-hit-routes template pane/floating/route-check ownership scan, scoped trailing-whitespace scan, and scoped `git diff --check`; package-level Cargo check and full Cargo tests remain deferred per the user's feature-first instruction.

The 2026-06-21 viewport-toolbar route pane/floating/route-check split reduced `profiling_hit_routes/viewport_toolbar.rs` from 98 lines to a 53-line viewport-toolbar route orchestration entry. `viewport_toolbar/pane.rs` owns scene/game toolbar eligibility, prefix/content checks, and toolbar frame creation; `viewport_toolbar/floating.rs` owns floating-window iteration; `viewport_toolbar/route_check.rs` owns runtime viewport-toolbar hit/control-id comparison. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, a profiling-hit-routes viewport-toolbar pane/floating/route-check ownership scan, scoped trailing-whitespace scan, and scoped `git diff --check`; package-level Cargo check and full Cargo tests remain deferred per the user's feature-first instruction.
