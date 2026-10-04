---
related_code:
  - zircon_editor/src/ui/retained_host/app.rs
  - zircon_editor/src/ui/retained_host/app/welcome_session.rs
  - zircon_editor/src/ui/retained_host/app/welcome_session/actions.rs
  - zircon_editor/src/ui/retained_host/app/welcome_session/actions/draft.rs
  - zircon_editor/src/ui/retained_host/app/welcome_session/actions/project.rs
  - zircon_editor/src/ui/retained_host/app/welcome_session/actions/recent.rs
  - zircon_editor/src/ui/retained_host/app/welcome_session/actions/startup_views.rs
  - zircon_editor/src/ui/retained_host/app/welcome_session/bridge.rs
  - zircon_editor/src/ui/retained_host/app/welcome_session/session.rs
  - zircon_editor/src/ui/retained_host/app/welcome_session/session/apply.rs
  - zircon_editor/src/ui/retained_host/app/welcome_session/session/present.rs
  - zircon_editor/src/ui/retained_host/app/welcome_session/session/snapshot.rs
  - zircon_editor/src/ui/retained_host/app/welcome_recent_pointer.rs
  - zircon_editor/src/ui/retained_host/app/welcome_recent_pointer/click.rs
  - zircon_editor/src/ui/retained_host/app/welcome_recent_pointer/motion.rs
  - zircon_editor/src/ui/retained_host/app/welcome_recent_pointer/scroll.rs
  - zircon_editor/src/ui/retained_host/app/pointer_layout.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/mod.rs
implementation_files:
  - zircon_editor/src/ui/retained_host/app/welcome_session.rs
  - zircon_editor/src/ui/retained_host/app/welcome_session/actions.rs
  - zircon_editor/src/ui/retained_host/app/welcome_session/actions/draft.rs
  - zircon_editor/src/ui/retained_host/app/welcome_session/actions/project.rs
  - zircon_editor/src/ui/retained_host/app/welcome_session/actions/recent.rs
  - zircon_editor/src/ui/retained_host/app/welcome_session/actions/startup_views.rs
  - zircon_editor/src/ui/retained_host/app/welcome_session/bridge.rs
  - zircon_editor/src/ui/retained_host/app/welcome_session/session.rs
  - zircon_editor/src/ui/retained_host/app/welcome_session/session/apply.rs
  - zircon_editor/src/ui/retained_host/app/welcome_session/session/present.rs
  - zircon_editor/src/ui/retained_host/app/welcome_session/session/snapshot.rs
  - zircon_editor/src/ui/retained_host/app/welcome_recent_pointer.rs
  - zircon_editor/src/ui/retained_host/app/welcome_recent_pointer/click.rs
  - zircon_editor/src/ui/retained_host/app/welcome_recent_pointer/motion.rs
  - zircon_editor/src/ui/retained_host/app/welcome_recent_pointer/scroll.rs
plan_sources:
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - user: 2026-06-18 editor UI architecture implementation, feature first and tests deferred
tests:
  - cargo fmt -p zircon_editor
  - cargo fmt -p zircon_editor --check
  - app welcome-session actions ownership scan
  - app welcome-session action subowner ownership scan
  - app welcome-session bridge/session ownership scan
  - app welcome-session session subowner ownership scan
  - app welcome recent pointer click/motion/scroll subowner ownership scan
  - scoped git diff --check
  - cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never
doc_type: module-detail
---

# Welcome Session Host Actions

`app/welcome_session.rs` is the structural entry for retained-host Welcome surface lifecycle, bridge dispatch, startup-session application, and user action handling.

The module is the boundary between retained host shell state and the Welcome template bridge. It should keep surface presentation and bridge dispatch visible, while user action side effects live in the child module.

## Welcome Bridge And Session

`app/welcome_session/bridge.rs` owns lazy builtin Welcome surface bridge creation and builtin Welcome control dispatch. It maps authored action/control ids to stable bridge control ids, dispatches click/change arguments through `callback_dispatch`, and routes returned `WelcomeHostEvent` values to the action handler.

`app/welcome_session/session.rs` is the structural entry for retained Welcome session state projection. `session/snapshot.rs` owns `WelcomePaneSnapshot` refresh into the runtime, `session/present.rs` owns Welcome page presentation with recent project data, and `session/apply.rs` owns startup session document application, startup builtin view opening, project workspace application, runtime world replacement, and presentation/render invalidation.

## Welcome Actions

`app/welcome_session/actions.rs` is the structural event-dispatch entry for Welcome user action execution. It handles `WelcomeHostEvent` values from both the Welcome surface bridge and recent-project pointer bridge, then delegates side effects to action subowners.

`app/welcome_session/actions/draft.rs` owns project-name and project-location draft mutation plus Welcome snapshot refresh.

`app/welcome_session/actions/project.rs` owns create-project and open-existing-project flows from the Welcome draft.

`app/welcome_session/actions/recent.rs` owns recent-project open/remove flows, including recent list refresh after failures or removals.

`app/welcome_session/actions/startup_views.rs` owns opening the default startup workbench and startup builtin views from the Welcome surface.

The event handler is `pub(in crate::ui::retained_host::app)` because `welcome_recent_pointer/click.rs` dispatches recent-project pointer clicks through the shared Welcome bridge and then applies the returned `WelcomeHostEvent`.

## Welcome Recent Pointer

`app/welcome_recent_pointer.rs` is the structural recent-project pointer entry.

`welcome_recent_pointer/click.rs` owns recent-project click dispatch. It commits the pointer layout, resolves the Welcome callback surface size, syncs the recent-project layout from the current chrome snapshot, ensures the Welcome bridge exists, dispatches the shared click, writes pointer state back, and forwards any returned `WelcomeHostEvent`.

`welcome_recent_pointer/motion.rs` owns hover movement updates. `welcome_recent_pointer/scroll.rs` owns scroll updates. Both reuse the current callback surface size, sync the recent-project bridge size, update pointer state, and write that state back to UI globals.

## Boundary Rules

- Keep `app/welcome_session.rs` as a structural module entry only.
- Keep Welcome surface bridge creation, bridge control-id mapping, and builtin Welcome surface control dispatch in `app/welcome_session/bridge.rs`.
- Keep Welcome session module declarations in `app/welcome_session/session.rs`.
- Keep runtime `WelcomePaneSnapshot` refresh in `app/welcome_session/session/snapshot.rs`.
- Keep Welcome page presentation and recent project data refresh in `app/welcome_session/session/present.rs`.
- Keep startup-session application, startup builtin view opening, project workspace application, runtime world replacement, and presentation/render invalidation in `app/welcome_session/session/apply.rs`.
- Keep `WelcomeHostEvent` dispatch in `app/welcome_session/actions.rs`.
- Keep Welcome draft mutation in `app/welcome_session/actions/draft.rs`.
- Keep create/open-existing project actions in `app/welcome_session/actions/project.rs`.
- Keep recent-project open/remove actions in `app/welcome_session/actions/recent.rs`.
- Keep default workbench and startup builtin view opening in `app/welcome_session/actions/startup_views.rs`.
- Keep `app/welcome_recent_pointer.rs` as a structural recent-project pointer entry.
- Keep recent-project click dispatch and returned `WelcomeHostEvent` forwarding in `app/welcome_recent_pointer/click.rs`.
- Keep recent-project hover movement state updates in `app/welcome_recent_pointer/motion.rs`.
- Keep recent-project scroll state updates in `app/welcome_recent_pointer/scroll.rs`.
- Recent-project pointer handlers may forward events to the Welcome action handler but should not duplicate project-open/remove logic.
- Keep Welcome layout sync in `app/pointer_layout.rs`.

## Validation Notes

The 2026-06-18 Welcome action split reduced `welcome_session.rs` from 288 lines to 143 lines. `welcome_session/actions.rs` is 149 lines and owns Welcome draft/project/recent/startup-view action execution.

Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app welcome-session actions ownership scan, scoped `git diff --check`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only. Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-19 bridge/session split reduced `welcome_session.rs` from 143 lines to 3 lines. `welcome_session/bridge.rs` is 59 lines and owns Welcome bridge creation, control-id mapping, callback dispatch, and event forwarding. `welcome_session/session.rs` is 86 lines and owns Welcome snapshot refresh, page presentation, startup-session application, project workspace application, runtime world replacement, and invalidation. `welcome_session/actions.rs` remains 152 lines and continues to own Welcome user action execution.

Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app welcome-session bridge/session ownership scan, scoped `git diff --check`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only (`zircon_runtime` 142 warnings, `zircon_editor` 63 warnings). Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-19 Welcome session subowner split reduced `welcome_session/session.rs` from 86 lines to a 3-line structural entry. `session/snapshot.rs` is 8 lines and owns runtime Welcome snapshot refresh, `session/present.rs` is 21 lines and owns Welcome page presentation plus recent project data refresh, and `session/apply.rs` is 63 lines and owns startup-session document application, startup builtin view opening, project workspace application, runtime world replacement, and presentation/render invalidation.

Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, and an app welcome-session session subowner ownership scan. A fresh `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-ui-owner-split-0619 --message-format short --color never` remains blocked before editor code by active `zircon_runtime::scene::dynamic_scene::session` owner-split work: `session/io/mod.rs` re-exports private IO helpers, producing E0364/E0603 visibility errors. Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-19 Welcome action subowner split reduced `welcome_session/actions.rs` from 152 lines to a 49-line event dispatcher. `actions/draft.rs` is 11 lines and owns draft project-name/location mutation; `actions/project.rs` is 36 lines and owns create/open-existing project flows; `actions/recent.rs` is 41 lines and owns recent-project open/remove flows; `actions/startup_views.rs` is 34 lines and owns default workbench/startup builtin view opening.

Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app welcome-session action subowner ownership scan, and scoped `git diff --check`, all of which passed except for existing CRLF conversion warnings in the dirty worktree. Focused `cargo check` was not rerun for this slice because independent `zircon_runtime` Cargo test processes were still active; full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-19 Welcome recent pointer click/motion/scroll subowner split reduced `welcome_recent_pointer.rs` from 91 lines to a 3-line structural entry. `welcome_recent_pointer/click.rs` is 42 lines and owns recent-project click dispatch plus returned event forwarding, `welcome_recent_pointer/motion.rs` is 30 lines and owns hover movement updates, and `welcome_recent_pointer/scroll.rs` is 31 lines and owns scroll updates.

Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app welcome recent pointer click/motion/scroll subowner ownership scan, scoped `git diff --check`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-ui-owner-split-0619 --message-format short --color never`, which passed with existing warning noise only (`zircon_runtime` 141 warnings, `zircon_editor` 65 warnings). Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.
