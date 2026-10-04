---
doc_type: completion-list
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/82/2026-09-27-physical-key-edit-command-routing.md
  - docs/plans/optimize/zircon_runtime/82-runtime-text-editing-document-selection-caret-hit-test-ime-composition-clipboard-secure-text-product-integration-current-source-review.md
implementation_files:
  - zircon_runtime/src/ui/surface/input/text_keyboard/edit_actions.rs
tests:
  - zircon_runtime/tests/runtime82_physical_key_edit_commands.rs
---

# Runtime82 physical key command completion list

| Plan slice | Completed implementation | Acceptance boundary | Status |
| --- | --- | --- | --- |
| Route physical-only edit commands without accidental SelectAll | Blank logical keys reach the existing physical fallback; logical a/A and physical 65/97 retain SelectAll. Eight real manager/Surface regressions cover navigation, selection, deletion, Undo receipts, unknown keys, secure behavior, and release. | Managed Runtime integration execution is pending with the next grouped batch. Product latency, memory, and Unreal comparison gates remain open. | implemented_pending_validation |
