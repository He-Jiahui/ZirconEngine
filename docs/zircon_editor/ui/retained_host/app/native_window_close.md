---
related_code:
  - zircon_editor/src/ui/retained_host/app.rs
  - zircon_editor/src/ui/retained_host/app/native_window_close.rs
  - zircon_editor/src/ui/retained_host/app/native_window_close/floating_window.rs
  - zircon_editor/src/ui/retained_host/app/native_window_close/prompt_actions.rs
  - zircon_editor/src/ui/retained_host/app/native_window_close/prompt_actions/actions.rs
  - zircon_editor/src/ui/retained_host/app/native_window_close/prompt_actions/completion.rs
  - zircon_editor/src/ui/retained_host/app/native_window_close/prompt_actions/presentation.rs
  - zircon_editor/src/ui/retained_host/app/close_prompt.rs
  - zircon_editor/src/ui/retained_host/app/callback_wiring.rs
  - zircon_editor/src/ui/retained_host/app/native_windows.rs
implementation_files:
  - zircon_editor/src/ui/retained_host/app/native_window_close.rs
  - zircon_editor/src/ui/retained_host/app/native_window_close/floating_window.rs
  - zircon_editor/src/ui/retained_host/app/native_window_close/prompt_actions.rs
  - zircon_editor/src/ui/retained_host/app/native_window_close/prompt_actions/actions.rs
  - zircon_editor/src/ui/retained_host/app/native_window_close/prompt_actions/completion.rs
  - zircon_editor/src/ui/retained_host/app/native_window_close/prompt_actions/presentation.rs
  - zircon_editor/src/ui/retained_host/app/close_prompt.rs
plan_sources:
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - user: 2026-06-18 editor UI architecture implementation, feature first and tests deferred
tests:
  - cargo fmt -p zircon_editor
  - cargo fmt -p zircon_editor --check
  - app native-window close ownership scan
  - app native-window close prompt-actions subowner ownership scan
  - cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never
doc_type: module-detail
---

# Retained Host Native Window Close Boundary

## Purpose

The native-window close boundary translates platform close requests into Workbench document lifecycle behavior. It covers the main editor window, native floating editor windows, and the retained close prompt used when dirty documents need a save/discard/cancel decision.

The app boundary is now split by responsibility:

- `native_window_close.rs` owns only the public close-request entry points and the dirty-document decision that determines whether a prompt is needed.
- `native_window_close/prompt_actions.rs` declares the close prompt action child modules. `prompt_actions/actions.rs` owns close prompt button action parsing and dispatch, `prompt_actions/presentation.rs` owns prompt begin/show/clear and target UI lookup, and `prompt_actions/completion.rs` owns prompted close completion. The former prompt-local automatic-save owner is retired.
- `native_window_close/floating_window.rs` owns floating-window document collection and the no-prompt close path that dispatches `LayoutCommand::CloseView` for each hosted view.
- `close_prompt.rs` remains the prompt data/projection helper: dirty-view DTOs, host prompt geometry, text, and save capability checks.

## Related Files

- `zircon_editor/src/ui/retained_host/app.rs` wires the main window and floating window close callbacks to the retained host methods.
- `zircon_editor/src/ui/retained_host/app/callback_wiring.rs` wires close prompt button callbacks.
- `zircon_editor/src/ui/retained_host/app/native_windows.rs` stores native floating window presenters and provides the target window for prompt display.
- `zircon_editor/src/ui/retained_host/app/close_prompt.rs` converts close prompt state into `HostClosePromptData`.

## Behavior Model

When the main window receives a close request, the host recomputes any pending presentation state, gathers all dirty view instances from the runtime, and either shows a main-window close prompt or allows the platform window to hide.

When a floating window receives a close request, the host resolves the view instances inside that floating window's `DocumentNode` tree. If any of those views are dirty, the prompt targets the floating window and is rendered through that native window's presenter when available. If none are dirty, the host dispatches a close-view layout command for each view in the floating window and only allows the native window to hide once the runtime layout no longer contains that floating window.

Close prompt actions are stable string ids from `close_prompt::close_action_id(...)`:

- `cancel` clears the prompt and leaves the window visible.
- `discard` clears the prompt and finishes the close without saving.
- `save` currently preserves the prompt and reports that documents cannot be saved; callers must choose Discard or Cancel until document-authority save routing is available.

## Design and Rationale

Close-request handling, prompt-button handling, and floating-window layout mutation are related but change for different reasons. Keeping them in separate files avoids turning native close behavior into an umbrella owner for platform callbacks, prompt rendering, document save policy, and Workbench layout mutation.

The root close module keeps the native close callbacks easy to audit. The prompt child owns action parsing, presentation, and close completion, but does not own document persistence. The floating-window child owns recursive `DocumentNode` traversal because that behavior is specific to deciding which view instances a native floating window contains.

The public callback method `close_prompt_action_clicked(...)` is visible only inside the retained-host app boundary. Helper methods shared by the root request flow and prompt action flow are visible only within `native_window_close`, so close prompt UI targeting, save policy, and close completion do not leak beyond the native close family.

## Control Flow

Main window close:

1. `native_main_window_close_requested(...)` recomputes pending retained-host state.
2. Dirty views are collected from all current view instances.
3. Dirty views produce a `PendingClosePrompt` targeted at the main window.
4. No dirty views return `CloseRequestResponse::HideWindow`.

Floating window close:

1. `native_floating_window_close_requested(...)` recomputes pending state.
2. The floating-window workspace is traversed to collect view instance ids.
3. Dirty view ids produce a floating-window targeted prompt.
4. Clean view ids dispatch `LayoutCommand::CloseView` for each hosted view.
5. The native window hides only after the runtime layout confirms the floating window is gone.

Prompt action:

1. `close_prompt_action_clicked(...)` parses the stable action id.
2. The current pending prompt is cloned so the action can clear or re-show UI safely.
3. Save reports the unsupported operation and keeps the prompt visible.
4. Discard completes the prompt by requesting app exit or closing the floating-window instances.

## Edge Cases and Constraints

- A floating-window close request with no discoverable view instances keeps the window shown rather than hiding a window whose runtime ownership is unclear.
- If a close-view layout command fails for one floating-window view, the status line receives the error and the native window remains visible.
- Dirty documents must be discarded or canceled; native close does not bypass document authority with a prompt-local save implementation.
- Prompt rendering targets the native floating window when that presenter exists; otherwise it falls back to the main UI host window.
- The split does not change close prompt action ids or platform close callback names.

## Test Coverage

Implementation-slice validation currently covers formatting, ownership scanning, scoped diff checking, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`. Existing retained-host close prompt and native-window tests remain the intended regression surface; full Cargo tests are deferred to the milestone testing stage per the user's instruction.

The 2026-06-19 prompt-actions subowner split established separate action, presentation, completion, and automatic-save leaves. The automatic-save leaf has since been deleted; current `actions.rs` reports Save as unsupported, `presentation.rs` owns prompt UI targeting plus show/clear behavior, and `completion.rs` owns final close execution.

Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app native-window close prompt-actions subowner ownership scan, scoped `git diff --check`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-ui-owner-split-0619 --message-format short --color never`, which passed with existing warning noise only (`zircon_runtime` 141 warnings, `zircon_editor` 65 warnings). Full Cargo tests remain deferred to the milestone testing stage per the user's instruction.

## Plan Sources

This module is part of `docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md`, M3.S2. The split supports the plan's requirement that Workbench shell behavior use narrow retained-host owners while native windowing remains editor-host state and does not leak into runtime world semantics.

## Open Issues or Follow-up

- The milestone testing stage must still run the declared `zircon_editor` test commands before the 08 plan can be called complete.
- Document-authority save routing must be implemented before the close prompt can safely complete Save.
