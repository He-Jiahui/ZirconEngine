---
related_code:
  - zircon_runtime/src/ui/accessibility/action.rs
  - zircon_runtime/src/ui/accessibility/action/result.rs
  - zircon_runtime/src/ui/accessibility/action/result/binding.rs
  - zircon_runtime/src/ui/accessibility/action/text.rs
  - zircon_runtime/src/ui/accessibility/action/text/replace.rs
  - zircon_runtime/src/ui/accessibility/action/text/selection.rs
implementation_files:
  - zircon_runtime/src/ui/accessibility/action/result.rs
  - zircon_runtime/src/ui/accessibility/action/result/binding.rs
  - zircon_runtime/src/ui/accessibility/action/text.rs
  - zircon_runtime/src/ui/accessibility/action/text/replace.rs
  - zircon_runtime/src/ui/accessibility/action/text/selection.rs
plan_sources:
  - .codex/plans/ZirconEngine Bevy 式 Platform Window Input Gilrs 完成度计划.md
  - .codex/plans/ZirconEngine UITextInputA11y 缺口收束计划.md
tests:
  - .codex/skills/zircon-dev/scripts/validate-matrix.ps1 -TargetDir F:\cargo-targets\zircon-platform-m5-workspace -VerboseOutput
  - .codex/skills/zircon-dev/scripts/validate-matrix.ps1 -SkipBuild -SkipTest -RunExportPlatformContract -ExportContractPlatform headless
  - rustfmt --edition 2021 --check zircon_runtime/src/ui/accessibility/action/result/binding.rs zircon_runtime/src/ui/accessibility/action/text/replace.rs zircon_runtime/src/ui/accessibility/action/text/selection.rs
doc_type: module-detail
---

# Accessibility Text Dispatch Visibility

## Purpose

`zircon_runtime/src/ui/accessibility/action/text.rs` is the structural boundary for TextInput edit actions. It re-exports the concrete `ReplaceSelectedText` and `SetTextSelection` dispatch helpers so the parent `action.rs` dispatcher can route neutral accessibility actions without knowing each text submodule.

The concrete helpers stay internal to `zircon_runtime::ui::accessibility::action`. They are not crate-public API, but their visibility must be wide enough for `action/text.rs` to re-export them to its parent module.

`zircon_runtime/src/ui/accessibility/action/result.rs` follows the same pattern for binding diagnostics. Its child `result/binding.rs` owns the mutation-report note helper, while action handlers in range, value, popup, expanded, and text paths import it through the `result` module. The helper therefore also stays limited to `crate::ui::accessibility::action`, not visible to the rest of the runtime.

## Boundary

`action/text/replace.rs` owns selected-range replacement behavior. `action/text/selection.rs` owns selection-only metadata updates. Both helpers use `pub(in crate::ui::accessibility::action)` so they can be re-exported by `action/text.rs` as `pub(super)` and consumed by `action.rs`.

`action/result/binding.rs` owns binding-report diagnostic note emission. Its helper uses the same `pub(in crate::ui::accessibility::action)` scope because `action/result.rs` re-exports it for sibling action dispatchers.

This keeps the module split narrow: TextInput edit behavior remains hidden from the rest of `zircon_runtime`, while the top-level accessibility action dispatcher can still compile through the normal module path.

## Validation

The real selected headless export-platform validator reached `zircon_runtime` compilation and failed before the export-policy test because the two helpers were only `pub(super)` inside their child modules. That made the `action/text.rs` re-export wider than the original item visibility.

After widening the helpers to `pub(in crate::ui::accessibility::action)`, scoped formatting and text checks passed for the two edited Rust files. The selected headless export-platform validator remains the next Cargo check once unrelated active compiler lanes quiet down.

The broader M5 workspace validator later reached the same module-boundary class for `append_binding_report_diagnostic`: `result.rs` re-exported the helper to sibling action dispatchers while `result/binding.rs` only exposed it to its immediate parent. The binding diagnostic helper now uses the same action-scoped visibility as the text dispatch helpers.
