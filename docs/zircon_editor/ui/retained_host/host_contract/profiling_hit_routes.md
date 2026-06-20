---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/profiling_hit_routes.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_hit_routes/routes.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_hit_routes/chrome.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_hit_routes/geometry.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_hit_routes/tabs.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_hit_routes/template.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_hit_routes/viewport_toolbar.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/geometry/
implementation_files:
  - zircon_editor/src/ui/retained_host/host_contract/profiling_hit_routes.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_hit_routes/routes.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_hit_routes/chrome.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_hit_routes/geometry.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_hit_routes/tabs.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_hit_routes/template.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_hit_routes/viewport_toolbar.rs
plan_sources:
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - user: 2026-06-18 editor UI architecture implementation, feature first and tests deferred
tests:
  - cargo fmt -p zircon_editor --check
  - profiling-hit-routes dispatch ownership scan
  - scoped trailing whitespace scan
  - scoped git diff --check
doc_type: module-detail
---

# Profiling Hit Routes

`profiling_hit_routes.rs` is the retained-host profiling route hit-test entry used by profiling artifact geometry to verify whether sampled route frames still contain their declared hit point. It is now a structural entry that wires route-specific children and re-exports the dispatch function.

`profiling_hit_routes/routes.rs` owns `route_contains_profile_frame(...)` and the route-kind dispatch table. The specialized children keep concrete checks split by route family: `chrome.rs` owns resize splitter and activity rail routes, `tabs.rs` owns document/drawer/floating/host-page tab routes, `viewport_toolbar.rs` owns toolbar controls, `template.rs` owns template-node routes, and `geometry.rs` owns shared containment/translation helpers.

The 2026-06-20 dispatch split reduced `profiling_hit_routes.rs` from 27 lines to a 7-line structural entry. `routes.rs` is 23 lines and owns the route-kind match. Validation used `cargo fmt -p zircon_editor --check`, a root ownership scan confirming presentation import, route-kind match, and route dispatch body no longer live in `profiling_hit_routes.rs`, a scoped trailing-whitespace scan, and scoped `git diff --check`. Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction, and package-level Cargo check is still waiting on unrelated `zircon_runtime` render-history compile errors.
