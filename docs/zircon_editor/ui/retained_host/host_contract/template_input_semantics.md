---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/template_input_semantics.rs
  - zircon_editor/src/ui/retained_host/host_contract/template_input_semantics/classification.rs
  - zircon_editor/src/ui/retained_host/host_contract/template_input_semantics/target.rs
  - zircon_editor/src/ui/retained_host/host_contract/template_input_semantics_tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/button_dispatch.rs
  - zircon_editor/src/ui/retained_host/host_contract/surface_hit_test.rs
implementation_files:
  - zircon_editor/src/ui/retained_host/host_contract/template_input_semantics.rs
  - zircon_editor/src/ui/retained_host/host_contract/template_input_semantics/classification.rs
  - zircon_editor/src/ui/retained_host/host_contract/template_input_semantics/target.rs
  - zircon_editor/src/ui/retained_host/host_contract/template_input_semantics_tests.rs
plan_sources:
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - user: 2026-06-20 editor UI architecture implementation, feature first and tests deferred
tests:
  - cargo fmt -p zircon_editor --check
  - host_contract template-input-semantics classification/target/test ownership scan
  - scoped trailing whitespace scan
  - scoped git diff --check
  - milestone testing stage: cargo check/test deferred until runtime render-history errors are resolved
doc_type: module-detail
---

# Template Input Semantics

`template_input_semantics.rs` is the retained-host boundary that translates a template pointer hit into text-input focus behavior. It now stays as a structural entry that re-exports classification and edit-target resolution.

`template_input_semantics/classification.rs` owns the text-input predicate. Popup menu and dropdown option rows are explicitly excluded, while Welcome text fields, `TemplateComponentFamily::TextInput`, and input/number component roles are accepted.

`template_input_semantics/target.rs` owns edit-target id resolution. Explicit edit actions win first, Welcome text actions are preserved for legacy startup surfaces, and componentized text inputs fall back to binding ids when present.

`template_input_semantics_tests.rs` owns regressions for binding-backed text fields and popup rows that must not steal text focus.

## Validation Notes

This slice is implementation-first. Formatting, root ownership scans, scoped whitespace scans, and scoped diff checks cover the handoff. Full Cargo check/test validation remains deferred to the milestone testing stage because current package checks fail in unrelated `zircon_runtime` render-history code before editor diagnostics are reached.
