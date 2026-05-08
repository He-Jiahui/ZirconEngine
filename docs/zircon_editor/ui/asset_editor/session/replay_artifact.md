---
related_code:
  - zircon_editor/src/ui/asset_editor/session/replay_artifact.rs
  - zircon_editor/src/ui/asset_editor/session/journal.rs
  - zircon_editor/src/ui/asset_editor/session/mod.rs
  - zircon_editor/src/ui/asset_editor/undo_stack.rs
  - zircon_editor/src/ui/asset_editor/mod.rs
  - zircon_editor/src/tests/editing/ui_asset_replay.rs
implementation_files:
  - zircon_editor/src/ui/asset_editor/session/replay_artifact.rs
  - zircon_editor/src/ui/asset_editor/session/journal.rs
  - zircon_editor/src/ui/asset_editor/undo_stack.rs
plan_sources:
  - user: 2026-05-08 continue all UI milestones until complete
  - .codex/plans/Zircon UI 资产化 Widget Editor 与共享 Layout.md
  - .codex/plans/UI Asset Editor 与共享 Layout 未完成内容归档.md
  - .codex/plans/UI 后续产品化与验证归档计划.md
tests:
  - zircon_editor/src/tests/editing/ui_asset_replay.rs
  - rustfmt --edition 2021 --check zircon_editor/src/ui/asset_editor/undo_stack.rs zircon_editor/src/ui/asset_editor/session/mod.rs zircon_editor/src/ui/asset_editor/session/replay_artifact.rs zircon_editor/src/ui/asset_editor/mod.rs zircon_editor/src/tests/editing/ui_asset_replay.rs
  - git diff --check -- zircon_editor/src/ui/asset_editor/undo_stack.rs zircon_editor/src/ui/asset_editor/session/mod.rs zircon_editor/src/ui/asset_editor/session/replay_artifact.rs zircon_editor/src/ui/asset_editor/mod.rs zircon_editor/src/tests/editing/ui_asset_replay.rs
  - cargo test -p zircon_editor --lib ui_asset_editor_session_exports_sanitized_bug_report_replay_artifact --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-m9-replay-artifact --message-format short --color never -- --nocapture --test-threads=1
doc_type: module-detail
---

# UI Asset Editor Replay Artifact

`replay_artifact.rs` exports the bug-report replay summary for `UiAssetEditorSession`. It is a diagnostic artifact, not the final executable journal adapter. The artifact captures route, selection, structured diagnostic count, sanitized textual diagnostics, redacted initial/current source summaries, and applied undo-stack replay records in oldest-to-newest order.

## Data Contract

`UiAssetEditorBugReportReplayArtifact` is versioned by `UI_ASSET_EDITOR_BUG_REPORT_REPLAY_ARTIFACT_SCHEMA_VERSION`. Source bodies are never serialized. `UiAssetEditorReplaySourceSummary` records only `redacted=true`, byte length, line count, and a deterministic FNV-1a 64-bit hash. Cross-file external effects similarly keep the affected `asset_id` and a redacted source summary for upsert/restore effects.

Document commands are summarized by command id, target, and safe structural metadata. Node summaries expose ids, node kind, widget/component references, class names, key names, binding ids, and child ids. Style summaries expose stylesheet ids, selectors, rule counts, and declaration key names, but not raw style values or generated TOML source. This keeps bug reports useful for replay triage without leaking full project source text.

## Undo Stack Source

`UiAssetEditorUndoStack::replay_records()` returns only the currently applied undo entries. It does not mutate undo/redo state and it does not include the redo stack. This matches the artifact's purpose: describe how the current editor state was reached from the initial source.

`export_bug_report_replay_artifact()` reconstructs the initial source by cloning the undo stack and applying undo transitions to a copy of the current source. If reconstruction fails, the artifact still exports the best available redacted source summary and records `initial_source_reconstruction_error`.

## Sanitization Boundary

The artifact sanitizer replaces obvious absolute path tokens in textual diagnostics with `[path]`. This is a best-effort privacy pass for bug-report payloads, not a general secret scanner. Callers that persist or upload the artifact should still treat it as diagnostic data and apply their own project-level policy.

## Remaining M9 Work

This slice closes the bug-report artifact export gap. The follow-up command journal adapter is documented in `command_journal.md`. M9 still needs durable artifact persistence, host event capture into journal entries, and broader validation once active runtime blockers no longer stop editor package compilation.

## Validation

The replay artifact slice passed targeted formatting and whitespace checks on 2026-05-08. The focused Cargo test for `ui_asset_editor_session_exports_sanitized_bug_report_replay_artifact` did not reach editor compilation because the active incremental layout lane currently breaks `zircon_runtime` at `zircon_runtime/src/ui/surface/surface.rs` with an unresolved `compute_incremental_layout_tree` import. That blocker is outside this module and remains owned by the active layout/render session.
