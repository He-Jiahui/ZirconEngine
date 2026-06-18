---
related_code:
  - zircon_editor/src/ui/retained_host/app.rs
  - zircon_editor/src/ui/retained_host/app/welcome_session.rs
  - zircon_editor/src/ui/retained_host/app/welcome_session/actions.rs
  - zircon_editor/src/ui/retained_host/app/welcome_recent_pointer.rs
  - zircon_editor/src/ui/retained_host/app/pointer_layout.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/mod.rs
implementation_files:
  - zircon_editor/src/ui/retained_host/app/welcome_session.rs
  - zircon_editor/src/ui/retained_host/app/welcome_session/actions.rs
  - zircon_editor/src/ui/retained_host/app/welcome_recent_pointer.rs
plan_sources:
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - user: 2026-06-18 editor UI architecture implementation, feature first and tests deferred
tests:
  - cargo fmt -p zircon_editor
  - cargo fmt -p zircon_editor --check
  - app welcome-session actions ownership scan
  - scoped git diff --check
  - cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never
doc_type: module-detail
---

# Welcome Session Host Actions

`app/welcome_session.rs` owns retained-host Welcome surface lifecycle and bridge dispatch. It creates the builtin Welcome surface bridge lazily, refreshes `WelcomePaneSnapshot` into the runtime, presents the Welcome page, applies startup session documents, and dispatches builtin Welcome controls into `WelcomeHostEvent`.

The module is the boundary between retained host shell state and the Welcome template bridge. It should keep surface presentation and bridge dispatch visible, while user action side effects live in the child module.

## Welcome Actions

`app/welcome_session/actions.rs` owns Welcome user action execution. It updates draft project fields, creates projects from the Welcome draft, opens existing projects, opens/removes recent projects, opens startup views, and handles `WelcomeHostEvent` values from both the Welcome surface bridge and recent-project pointer bridge.

The event handler is `pub(in crate::ui::retained_host::app)` because `welcome_recent_pointer.rs` dispatches recent-project pointer clicks through the shared Welcome bridge and then applies the returned `WelcomeHostEvent`.

## Boundary Rules

- Keep Welcome surface bridge creation, runtime snapshot refresh, page presentation, startup-session application, and builtin Welcome surface control dispatch in `app/welcome_session.rs`.
- Keep Welcome draft mutation, project create/open/remove actions, startup view opening, and `WelcomeHostEvent` handling in `app/welcome_session/actions.rs`.
- Keep recent-project pointer hit testing and pointer state writeback in `app/welcome_recent_pointer.rs`; it may forward events to the Welcome action handler but should not duplicate project-open/remove logic.
- Keep Welcome layout sync in `app/pointer_layout.rs`.

## Validation Notes

The 2026-06-18 Welcome action split reduced `welcome_session.rs` from 288 lines to 143 lines. `welcome_session/actions.rs` is 149 lines and owns Welcome draft/project/recent/startup-view action execution.

Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app welcome-session actions ownership scan, scoped `git diff --check`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only. Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.
