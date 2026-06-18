---
related_code:
  - zircon_editor/src/ui/retained_host/app.rs
  - zircon_editor/src/ui/retained_host/app/assets.rs
  - zircon_editor/src/ui/retained_host/app/assets/refresh.rs
  - zircon_editor/src/ui/retained_host/app/backend_refresh.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle.rs
implementation_files:
  - zircon_editor/src/ui/retained_host/app/assets.rs
  - zircon_editor/src/ui/retained_host/app/assets/refresh.rs
  - zircon_editor/src/ui/retained_host/app/backend_refresh.rs
plan_sources:
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - user: 2026-06-18 editor UI architecture implementation, feature first and tests deferred
tests:
  - cargo fmt -p zircon_editor
  - cargo fmt -p zircon_editor --check
  - app assets refresh ownership scan
  - cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never
doc_type: module-detail
---

# Asset Host Actions

`app/assets.rs` owns asset surface bridge creation, model import into the current project, default project material resolution, asset workspace sync entry points, and asset surface control dispatch. It keeps the lifecycle-facing `RetainedEditorHost` methods that connect UI callbacks to the asset surface bridge and project import side effects.

`app/backend_refresh.rs` owns the pure refresh-plan decision model. It consumes asset, editor-asset, and resource change events and returns the catalog/resource/detail/preview/default-scene/invalidation work required for the host.

## Refresh

`app/assets/refresh.rs` owns the runtime-facing asset refresh execution path. It drains asset/resource event receivers, records refresh counters, asks `backend_refresh` for the refresh plan, applies catalog/resource/detail/preview sync, reloads the default scene when required, applies render/presentation invalidation, records paint-only invalidation, and requests shell-frame redraw for asset preview repaint.

It also owns startup event draining, catalog/resource snapshot sync helpers, selected asset detail refresh, visible asset preview refresh, and refresh-plan profiling counters. Methods that are called from app siblings use `pub(in crate::ui::retained_host::app)` so the child module preserves the old app-level method surface without widening it.

## Boundary Rules

- Keep asset surface bridge creation and callback dispatch in `app/assets.rs`.
- Keep model import, animation derivation dispatch, default material resolution, and project-level asset workspace sync entry in `app/assets.rs`.
- Keep refresh event draining, refresh-plan application, catalog/resource snapshot sync helpers, selected detail refresh, visible preview refresh, and refresh profiling counters in `app/assets/refresh.rs`.
- Keep pure refresh planning and event-to-action policy in `app/backend_refresh.rs`; do not fold policy decisions into host side-effect code.

## Validation Notes

The 2026-06-18 refresh split reduced `assets.rs` from 438 lines to 174 lines. `assets/refresh.rs` is 270 lines and owns the refresh execution path, startup event drain, catalog/resource/detail/preview sync helpers, paint-only preview redraw, and refresh plan counters. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app assets refresh ownership scan, scoped `git diff --check`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only. Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.
