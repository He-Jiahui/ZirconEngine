---
related_code:
  - zircon_runtime/src/ui/accessibility/action/value.rs
  - zircon_runtime/src/ui/accessibility/action/value/text.rs
  - zircon_runtime/src/ui/accessibility/action/text_state.rs
implementation_files:
  - zircon_runtime/src/ui/accessibility/action/value.rs
  - zircon_runtime/src/ui/accessibility/action/value/text.rs
plan_sources:
  - .codex/plans/ZirconEngine Bevy 式 Platform Window Input Gilrs 完成度计划.md
  - .codex/plans/ZirconEngine UITextInputA11y 缺口收束计划.md
tests:
  - cargo test -p zircon_runtime --lib accessibility_set_value_rejects_read_only_text_input --locked --target-dir F:\cargo-targets\zircon-platform-m5-runtime-check --message-format short -- --test-threads=1
  - cargo check -p zircon_runtime --locked --target-dir F:\cargo-targets\zircon-platform-m5-runtime-check --message-format short
  - .codex/skills/zircon-dev/scripts/validate-matrix.ps1 -TargetDir F:\cargo-targets\zircon-platform-m5-workspace -VerboseOutput
doc_type: module-detail
---

# Accessibility SetValue Text Preparation

## Purpose

`zircon_runtime/src/ui/accessibility/action/value.rs` owns the public action flow for `UiAccessibilityAction::SetValue`. The TextInput-specific preparation is split into `value/text.rs` so whole-field text edits can keep read-only rejection and retained constraint sanitization out of the main dispatcher.

`prepare_text_input_set_value` returns a prepared value and optional constraint note for the accepted path. Rejected preparation returns a small rejection descriptor instead of taking ownership of `UiInputDispatchResult`. The caller keeps the original dispatch result and finishes it through the shared `finish_unhandled` helper, so accepted SetValue mutation, unchanged mutation, and rejected TextInput preparation all use the same diagnostics path.

## Boundary

The helper is intentionally narrower than the main dispatcher. It does not mutate `UiSurface`, emit component events, attach binding reports, or finish a dispatch result. It only answers whether the TextInput value can proceed and what value should be written after retained TextInput constraints are applied.

The main dispatcher still owns mutation, binding diagnostics, edit metadata synchronization, and component event emission. This keeps TextInput preparation reusable without creating a second action-result owner.

## Validation

The M5 workspace validator first passed `cargo build --workspace --locked --verbose` after the binding diagnostic visibility fix, then failed in the `cargo test --workspace --locked --verbose` compile stage because `prepare_text_input_set_value` moved `UiInputDispatchResult` into the preparation helper and the accepted path later needed the same result.

After the helper was changed to return a rejection descriptor, the focused TextInput read-only SetValue test passed and `cargo check -p zircon_runtime --locked` passed against the dedicated M5 runtime target directory. The next M5 step is to rerun the full workspace test gate after unrelated active compiler lanes finish.
