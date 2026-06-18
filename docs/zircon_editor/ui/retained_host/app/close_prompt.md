---
related_code:
  - zircon_editor/src/ui/retained_host/app.rs
  - zircon_editor/src/ui/retained_host/app/close_prompt.rs
  - zircon_editor/src/ui/retained_host/app/close_prompt/actions.rs
  - zircon_editor/src/ui/retained_host/app/close_prompt/model.rs
  - zircon_editor/src/ui/retained_host/app/close_prompt/presentation.rs
  - zircon_editor/src/ui/retained_host/app/native_window_close.rs
  - zircon_editor/src/ui/retained_host/app/native_window_close/prompt_actions.rs
implementation_files:
  - zircon_editor/src/ui/retained_host/app/close_prompt.rs
  - zircon_editor/src/ui/retained_host/app/close_prompt/actions.rs
  - zircon_editor/src/ui/retained_host/app/close_prompt/model.rs
  - zircon_editor/src/ui/retained_host/app/close_prompt/presentation.rs
plan_sources:
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - user: 2026-06-18 editor UI architecture implementation, feature first and tests deferred
tests:
  - cargo fmt -p zircon_editor
  - cargo fmt -p zircon_editor --check
  - app close-prompt model/presentation ownership scan
  - git diff --check
  - cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never
doc_type: module-detail
---

# Retained Host Close Prompt

## Purpose

The retained-host close prompt boundary owns dirty-document close prompt data, prompt projection into host presentation data, and close prompt action id parsing. It preserves the `close_prompt::...` call surface used by native window close handling while separating model policy from presentation geometry.

This split supports the 08 M3.S2 retained-host cleanup by making `app/close_prompt.rs` a structural re-export point instead of a mixed model/presentation/action helper file.

## Related Files

- `zircon_editor/src/ui/retained_host/app/close_prompt.rs` declares and re-exports the close prompt child modules.
- `zircon_editor/src/ui/retained_host/app/close_prompt/actions.rs` owns stable close prompt action id parsing for save, discard, and cancel.
- `zircon_editor/src/ui/retained_host/app/close_prompt/model.rs` owns `ClosePromptTarget`, `DirtyCloseView`, `PendingClosePrompt`, dirty-view collection, and save eligibility policy.
- `zircon_editor/src/ui/retained_host/app/close_prompt/presentation.rs` owns `HostClosePromptData` construction, dialog/button geometry, title/message/details text, target window id selection, and UI show/clear helpers.
- `zircon_editor/src/ui/retained_host/app/native_window_close/prompt_actions.rs` owns the retained-host prompt action flow that consumes this model and presentation layer.

## Behavior Model

Dirty document collection is model-only. Main-window close requests collect all dirty views; floating-window close requests collect dirty views only for candidate view instances. Each dirty entry stores the view instance id, descriptor id, and title so later prompt actions can save/discard/close the same documents.

Prompt presentation reads the current window size and creates a centered dialog with stable button geometry. It projects the prompt target, title, dirty-count message, dirty view details, and save eligibility into `HostClosePromptData`, then writes that data into either the main UI or native floating-window UI selected by the prompt action owner.

Action parsing accepts only `save`, `discard`, and `cancel`. Unknown action ids are rejected before close prompt state mutates.

## Design and Rationale

Close prompt model policy and presentation geometry change for different reasons. Dirty-view filtering depends on Workbench view state and document type save support. Presentation depends on native host window dimensions and button layout. Action parsing is a small stable id grammar consumed by callback handling. Splitting these concerns keeps native close request handling focused on close flow instead of carrying prompt DTO construction details.

The root module re-exports the same app-visible names, so existing callers keep using `close_prompt::DirtyCloseView`, `close_prompt::show_prompt(...)`, and `close_prompt::close_action_id(...)` without learning the child layout.

## Edge Cases and Constraints

- Save is enabled only when all dirty views have descriptor ids supported by automatic save.
- Prompt details show at most the first three dirty view titles and use an ellipsis when more dirty views exist.
- Floating-window prompt target ids use the `MainPageId` payload; the main window target uses the stable `main` id.
- Dialog width clamps between 280 and 500 pixels with a 48-pixel viewport margin.

## Test Coverage

Implementation-slice validation covers formatting, ownership scans, scoped diff checks, and the current practical Cargo check status. `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, the app close-prompt ownership scan, scoped `git diff --check`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never` pass in the current worktree. Cargo still emits existing warning noise from active runtime/editor work, but no close-prompt diagnostics remain. Full Cargo tests remain deferred to the milestone testing stage per the user's instruction.

## Plan Sources

This module belongs to `docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md`, M3.S2, where retained-host Workbench shell behavior is being converged into runtime UI backed surfaces with narrow app owners.

## Open Issues or Follow-up

- Keep dirty-view model/save policy in `model.rs`, host prompt projection in `presentation.rs`, and action id parsing in `actions.rs`.
