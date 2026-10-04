---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/frame_geometry.rs
  - zircon_editor/src/ui/retained_host/host_contract/frame_geometry/hit.rs
  - zircon_editor/src/ui/retained_host/host_contract/frame_geometry/union.rs
  - zircon_editor/src/ui/retained_host/host_contract/frame_geometry/visibility.rs
  - zircon_editor/src/ui/retained_host/host_contract/frame_geometry/tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/
  - zircon_editor/src/ui/retained_host/host_contract/redraw/
  - zircon_editor/src/ui/retained_host/host_contract/template_geometry/
implementation_files:
  - zircon_editor/src/ui/retained_host/host_contract/frame_geometry.rs
  - zircon_editor/src/ui/retained_host/host_contract/frame_geometry/hit.rs
  - zircon_editor/src/ui/retained_host/host_contract/frame_geometry/union.rs
  - zircon_editor/src/ui/retained_host/host_contract/frame_geometry/visibility.rs
  - zircon_editor/src/ui/retained_host/host_contract/frame_geometry/tests.rs
plan_sources:
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - user: 2026-06-18 editor UI architecture implementation, feature first and tests deferred
tests:
  - cargo fmt -p zircon_editor --check
  - frame-geometry hit/union/visibility/test ownership scan
  - scoped trailing whitespace scan
  - scoped git diff --check
doc_type: module-detail
---

# Frame Geometry

`frame_geometry.rs` is the retained-host shared frame-math entry used by native pointer damage, keyboard popup damage, redraw coalescing, popup dismiss targets, and template popup bounds. It is now a structural module entry that only wires child owners and re-exports the shared helpers.

`frame_geometry/visibility.rs` owns finite positive-size frame validation. `frame_geometry/hit.rs` owns point containment. `frame_geometry/union.rs` owns frame union and optional-frame union. `frame_geometry/tests.rs` owns the local visibility and union regressions that previously lived inline in the root file.

The 2026-06-20 hit/union/visibility split reduced `frame_geometry.rs` from 63 lines to an 8-line structural entry. The new child files are `hit.rs` at 4 lines, `union.rs` at 23 lines, `visibility.rs` at 9 lines, and `tests.rs` at 27 lines. The 2026-06-20 owner visibility sweep kept the hit/union/visibility helpers exported only within the `host_contract` owner boundary while preserving native pointer, popup dismiss, redraw, and template geometry callers. Validation used `cargo fmt -p zircon_editor --check`, a root ownership scan confirming `FrameRect` imports, helper bodies, and inline tests no longer live in `frame_geometry.rs`, scoped `git diff --check`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passes with existing warning noise only. Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.
