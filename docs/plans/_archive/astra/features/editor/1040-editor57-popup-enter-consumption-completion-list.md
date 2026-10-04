---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/window/text_input/keyboard.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/text_input/keyboard/popup.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_keyboard.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_keyboard/dispatch.rs
  - zircon_editor/src/tests/host/retained_window/host_page_overflow_keyboard.rs
  - docs/plans/optimize/zircon_editor/57/2026-09-28-editor57-popup-enter-consumption.md
plan_sources:
  - docs/plans/astra/features/editor/1037-editor57-asset-browser-context-menu-open-completion-list.md
status: regression_added_pending_validation
validation_status: scoped_static_checks_passed_managed_validation_pending
---

# Editor1040 / Editor57 popup Enter consumption completion list

| Plan item | Source result | Evidence and remaining gate | Status |
| --- | --- | --- | --- |
| Empty active popup owns Enter | The popup route identifies an active popup Accept command independently from its redraw result, so an empty target does not leak into host/native fallback or force a redraw. | The host regression dispatches Enter through dispatch_keyboard_event and observes callback suppression; paired no-popup coverage preserves normal fallback. Cargo execution is pending. | regression_added_pending_validation |
| Valid popup row activation | Existing selected-row Enter dispatch remains unchanged. | The existing page-overflow and workbench menu keyboard regressions cover selected-row dispatch; grouped managed validation remains pending. | validation_pending |
| ED57-P0-03 completion | Not complete. | Common asset activation intent/receipt, Enter activation of Asset Browser targets, catalog-generation fencing, toolkit presentation, and performance gates remain open. | product_gate_pending |

No Cargo validation or performance measurement was run for this slice. Grouped
coordinator source attribution completed under request
`9006f4b3eef74e2a837772ecf0341d68`.
