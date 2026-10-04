---
doc_type: completion-list
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/82/2026-09-27-inactive-composition-metadata-encoding.md
  - docs/plans/optimize/zircon_runtime/82-runtime-text-editing-document-selection-caret-hit-test-ime-composition-clipboard-secure-text-product-integration-current-source-review.md
implementation_files:
  - zircon_runtime/src/ui/editable_text_composition.rs
  - zircon_runtime/src/ui/component/catalog/material_foundation/shared.rs
  - zircon_runtime/src/ui/surface/input/editable_text/property_transaction.rs
tests:
  - zircon_runtime/tests/runtime82_inactive_composition_regression.rs
  - zircon_runtime/src/ui/tests/widget_text_input_keyboard/word_shortcuts.rs
  - zircon_runtime/src/ui/tests/widget_text_input_keyboard/text_ime.rs
  - zircon_runtime/src/ui/tests/surface_dirty_domains/render_domains.rs
  - zircon_runtime/src/ui/dispatch/input_manager/bound_text_model_updates/tests.rs
  - zircon_runtime/src/ui/tests/accessibility_text_input_actions.rs
  - zircon_runtime/src/ui/tests/accessibility/value_actions.rs
  - zircon_runtime/src/ui/surface/input/text_state/optimization_tests.rs
---

# Runtime82 inactive composition metadata encoding completion list

| Plan slice | Completed implementation | Acceptance boundary | Status |
| --- | --- | --- | --- |
| Separate inactive composition from legitimate empty active preedit | Shared private negative offset convention in transaction and built-in descriptors; retained nonnegative active interpretation and real empty lifecycle. Added nine real Surface/InputManager regressions and updated inactive keyboard/reset/accessibility expectations while preserving foreign edits. | Managed functional execution pending. Selected-empty Commit additionally requires committed-source/epoch repair in this batch. Full Runtime82 performance gates remain open; no timing or memory claim. | implemented_pending_validation |
