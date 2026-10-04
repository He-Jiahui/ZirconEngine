---
title: Editor60 hierarchy rename failed-commit draft preservation
category: zircon_editor
date: 2026-09-28
implementation_status: implemented_pending_validation
validation_status: scoped_static_checks_passed_managed_validation_pending
performance_status: product_gate_pending
related_code:
  - zircon_editor/src/ui/retained_host/app/hierarchy_rename.rs
  - zircon_editor/src/ui/retained_host/app/tests/hierarchy_rename.rs
  - zircon_editor/src/ui/retained_host/app/tests/mod.rs
plan_sources:
  - docs/plans/optimize/zircon_editor/60-editor-scene-hierarchy-outliner-tree-projection-expansion-selection-rename-reparent-drag-drop-visibility-lock-multi-world-product-integration-current-source-review.md
frozen_predecessor_manifest: .codex/state/session-coordinator/async-validation-batches/2026-09-28-astra-editor-wave-source-manifest.json
frozen_predecessor_manifest_sha256: 226787c74a8dd41af1cdddaa6e2097c9c3e2579c2211f3e3f623937130c1aeda
preimage_sha256:
  zircon_editor/src/ui/retained_host/app/hierarchy_rename.rs: da3940cad575fa278d5662a5b96257081703faf312d752dcc44386c0b00ef134
  zircon_editor/src/ui/retained_host/app/tests/mod.rs: 5f51fcbbe6c9571b82cbb31cc4f699f8e34dff5de0dbf519a791374f45d403ea
---

# Editor60 hierarchy rename failed-commit draft preservation

## Failure and change

The commit handler cleared the inline rename focus before synchronously dispatching RenameNode. If the editor executor rejected the event, the handler showed an error after the text input had already been retired, so the user could not correct and retry the draft.

The handler now dispatches first. It clears the inline rename focus only after the editor executor returns success. On failure it keeps the existing focus data and exact input string, then reports the error through the status line. The existing trim and empty-name behavior is unchanged. Focus cleanup is a direct UiHostContext update; it does not call back into the app while the host RefCell is mutably borrowed.

## Behavioral regression

The test failed_hierarchy_rename_commit_keeps_exact_focus_for_successful_retry drives Enter through the real UiHostWindow focused-text commit path and installed pane surface callback into the synchronous EditorHostEventController.

The first commit runs while the loaded scene is in Play mode, where EditorState::apply_intent rejects scene edits. The test checks the exact focus and draft before commit and after the failure, verifies the executor error is surfaced, confirms no history transaction is added, and allows at most the failed rename event in the journal. It then exits Play mode without changing the focus or text and retries Enter on the same node. The retry must emit exactly one normalized RenameNode, add one history transaction, rename the projected row, and clear text focus.

## Validation and evidence

Rust formatting passed with rustfmt 1.94.1 for all three changed Rust paths. Direct Cargo and managed Rust execution were not run in this slice; the behavioral test remains pending managed validation. No live OS interaction or performance measurement was performed, so Editor product and performance gates remain pending.

The frozen 471-path predecessor manifest remains byte-for-byte unchanged. Its SHA-256 and the two modified Rust preimages are recorded in this file front matter. The coordinator lease claim returned internal_error with correlationId adda778f7e614aad8baeda6b11f70153; no source path had a live lease or active overlapping owner at the read-only check, and this successor proceeded as isolated work.
