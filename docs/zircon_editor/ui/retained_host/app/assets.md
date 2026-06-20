---
related_code:
  - zircon_editor/src/ui/retained_host/app.rs
  - zircon_editor/src/ui/retained_host/app/assets.rs
  - zircon_editor/src/ui/retained_host/app/assets/bridge.rs
  - zircon_editor/src/ui/retained_host/app/assets/controls.rs
  - zircon_editor/src/ui/retained_host/app/assets/refresh.rs
  - zircon_editor/src/ui/retained_host/app/assets/refresh/apply.rs
  - zircon_editor/src/ui/retained_host/app/assets/refresh/counters.rs
  - zircon_editor/src/ui/retained_host/app/assets/refresh/events.rs
  - zircon_editor/src/ui/retained_host/app/assets/refresh/events/runtime.rs
  - zircon_editor/src/ui/retained_host/app/assets/refresh/events/startup.rs
  - zircon_editor/src/ui/retained_host/app/assets/refresh/snapshots.rs
  - zircon_editor/src/ui/retained_host/app/assets/workspace.rs
  - zircon_editor/src/ui/retained_host/app/backend_refresh.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle.rs
implementation_files:
  - zircon_editor/src/ui/retained_host/app/assets.rs
  - zircon_editor/src/ui/retained_host/app/assets/bridge.rs
  - zircon_editor/src/ui/retained_host/app/assets/controls.rs
  - zircon_editor/src/ui/retained_host/app/assets/refresh.rs
  - zircon_editor/src/ui/retained_host/app/assets/refresh/apply.rs
  - zircon_editor/src/ui/retained_host/app/assets/refresh/counters.rs
  - zircon_editor/src/ui/retained_host/app/assets/refresh/events.rs
  - zircon_editor/src/ui/retained_host/app/assets/refresh/events/runtime.rs
  - zircon_editor/src/ui/retained_host/app/assets/refresh/events/startup.rs
  - zircon_editor/src/ui/retained_host/app/assets/refresh/snapshots.rs
  - zircon_editor/src/ui/retained_host/app/assets/workspace.rs
  - zircon_editor/src/ui/retained_host/app/backend_refresh.rs
plan_sources:
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - user: 2026-06-18 editor UI architecture implementation, feature first and tests deferred
tests:
  - cargo fmt -p zircon_editor
  - cargo fmt -p zircon_editor --check
  - app assets refresh ownership scan
  - app assets refresh event/snapshot/counter ownership scan
  - app assets refresh apply subowner ownership scan
  - app assets refresh events runtime/startup ownership scan
  - app assets bridge/control/workspace ownership scan
  - scoped git diff --check
  - cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never
doc_type: module-detail
---

# Asset Host Actions

`app/assets.rs` is the structural entry for retained-host asset actions. Its child modules own asset surface bridge creation, asset surface callback dispatch, model import into the current project, default project material resolution, asset workspace sync entry points, and refresh execution.

`app/backend_refresh.rs` owns the pure refresh-plan decision model. It consumes asset, editor-asset, and resource change events and returns the catalog/resource/detail/preview/default-scene/invalidation work required for the host.

## Bridge And Controls

`app/assets/bridge.rs` owns lazy `BuiltinAssetSurfaceTemplateBridge` creation and runtime asset surface control dispatch. It focuses the callback source window, validates bridge availability, forwards click/change arguments to the callback dispatcher, and applies host dispatch results.

`app/assets/controls.rs` owns user-facing asset surface control ids. It maps legacy and Workbench action ids into asset surface binding ids, constructs change arguments for search/filter/view/utility controls, and dispatches click actions such as opening the Asset Browser, locating the selected asset, or importing a model.

## Workspace

`app/assets/workspace.rs` owns project-level asset side effects. It reloads the default scene from the current project, imports staged model assets and derived animation assets, resolves the default project material, imports the mesh into the runtime viewport, and provides the asset workspace sync entry point used by startup and import flows.

## Refresh

`app/assets/refresh.rs` owns the runtime-facing asset refresh execution path. It asks the refresh event child for drained asset/resource events, refreshes the editor asset manager for runtime-project asset changes, asks `backend_refresh` for the refresh plan, records refresh-plan counters, and delegates the resulting side effects to the apply child.

`app/assets/refresh/apply.rs` owns refresh-plan side-effect application: catalog/resource/detail/preview sync, default-scene reload, render/presentation invalidation, paint-only invalidation recording, and asset preview repaint requests.

`app/assets/refresh/events.rs` is the structural entry for refresh event data. It owns the `AssetRefreshEvents` DTO and child module boundary.

`app/assets/refresh/events/runtime.rs` owns runtime refresh event receiver draining and asset/editor/resource event-count profiling counters.

`app/assets/refresh/events/startup.rs` owns startup bootstrap event draining and startup drained counters.

`app/assets/refresh/snapshots.rs` owns catalog/resource snapshot sync, selected asset detail sync, visible asset preview refresh, and shell-frame paint-only redraw requests.

`app/assets/refresh/counters.rs` owns refresh-plan profiling counters.

Methods that are called from app siblings use `pub(in crate::ui::retained_host::app)` so the child module preserves the old app-level method surface without widening it.

## Boundary Rules

- Keep `app/assets.rs` as a structural module entry only.
- Keep asset surface bridge creation and runtime surface callback dispatch in `app/assets/bridge.rs`.
- Keep asset UI action-id mapping and click/change argument construction in `app/assets/controls.rs`.
- Keep model import, animation derivation dispatch, default material resolution, default-scene reload, and project-level asset workspace sync entry in `app/assets/workspace.rs`.
- Keep refresh event collection, runtime-project editor asset refresh, refresh-plan construction, and refresh-plan counter recording in `app/assets/refresh.rs`.
- Keep refresh-plan application, catalog/resource/detail/preview sync dispatch, invalidation, default-scene reload dispatch, and paint-only redraw orchestration in `app/assets/refresh/apply.rs`.
- Keep `app/assets/refresh/events.rs` as the structural refresh event DTO entry only.
- Keep runtime refresh event draining and asset/editor/resource event-count counters in `app/assets/refresh/events/runtime.rs`.
- Keep startup bootstrap event draining and startup drained counters in `app/assets/refresh/events/startup.rs`.
- Keep catalog/resource/detail/preview snapshot sync and paint-only shell-frame redraw requests in `app/assets/refresh/snapshots.rs`.
- Keep refresh-plan profiling counters in `app/assets/refresh/counters.rs`.
- Keep pure refresh planning and event-to-action policy in `app/backend_refresh.rs`; do not fold policy decisions into host side-effect code.

## Validation Notes

The 2026-06-18 refresh split reduced `assets.rs` from 438 lines to 174 lines. `assets/refresh.rs` is 270 lines and owns the refresh execution path, startup event drain, catalog/resource/detail/preview sync helpers, paint-only preview redraw, and refresh plan counters. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app assets refresh ownership scan, scoped `git diff --check`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only. Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-19 bridge/control/workspace split reduced `assets.rs` from 174 lines to 4 lines. `assets/bridge.rs` is 45 lines and owns bridge creation plus asset surface dispatch. `assets/controls.rs` is 60 lines and owns action-id mapping plus click/change argument construction. `assets/workspace.rs` is 78 lines and owns default scene reload, project model import, default material resolution, and workspace sync entry points.

Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app assets bridge/control/workspace ownership scan, scoped `git diff --check`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only (`zircon_runtime` 142 warnings, `zircon_editor` 63 warnings). Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-19 refresh event/snapshot/counter split reduced `assets/refresh.rs` from 270 lines to 97 lines. `assets/refresh/events.rs` is 88 lines and owns refresh/startup event draining plus event counters. `assets/refresh/snapshots.rs` is 71 lines and owns catalog/resource/detail/preview sync plus paint-only redraw. `assets/refresh/counters.rs` is 45 lines and owns refresh-plan counters.

Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app assets refresh event/snapshot/counter ownership scan, and `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-ui-owner-split-0619 --message-format short --color never`, which passed with existing warning noise only (`zircon_runtime` 142 warnings, `zircon_editor` 65 warnings). Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-19 refresh apply subowner split reduced `assets/refresh.rs` from 97 lines to 49 lines. `assets/refresh/apply.rs` is 67 lines and owns refresh-plan side-effect application, including catalog/resource/detail/preview sync, default-scene reload, render/presentation invalidation, paint-only invalidation, and asset preview repaint requests.

Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app assets refresh apply subowner ownership scan, scoped `git diff --check`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-ui-owner-split-0619 --message-format short --color never`, which passed with existing warning noise only (`zircon_runtime` 141 warnings, `zircon_editor` 65 warnings). Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-19 refresh events runtime/startup subowner split reduced `assets/refresh/events.rs` from 95 lines to an 18-line refresh event DTO and structural entry. `assets/refresh/events/runtime.rs` is 43 lines and owns runtime refresh event draining plus asset/editor/resource counters. `assets/refresh/events/startup.rs` is 42 lines and owns startup bootstrap event draining plus startup drained counters.

Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app assets refresh events runtime/startup ownership scan, scoped `git diff --check`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-ui-owner-split-0619 --message-format short --color never`, which passed with existing warning noise only (`zircon_runtime` 141 warnings, `zircon_editor` 65 warnings). Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.
