---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/134-editor-settings-preferences-scope-persistence-locale-i18n-appearance-plugin-extensibility-current-source-review.md
  - docs/plans/optimize/zircon_editor/309/2026-08-30-scene-mode-entry-registration.md
  - docs/plans/optimize/zircon_editor/159/2026-08-26-export-pipeline-stage-capacity.md
related_records:
  - docs/plans/astra/features/editor/664-20260911-editor-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_editor/src/core/settings/tests/mod.rs
  - zircon_editor/src/scene/modes/scene_mode_registry/entry_registration_tests.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/progress.rs
---

# Editor899 Library-Test Import Contract Repairs

## 计划完成列表

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Settings persistence, registry, and value-batch tests | Import the existing numeric-step direction, color channel, and value-source types once in the clean test parent module so its children retain their original assertions. | v27 Editor managed test-profile check reported 12 missing-type errors in these children; local Rustfmt and scoped diff checks pass, without a post-repair Cargo run. | implemented_pending_validation |
| Scene-mode entry-registration tests | Import `SceneModeCtx`, `InputOutcome`, `ViewportOverlayBuilder`, and `ViewportInput` from their present owners instead of relying on parent-module imports. | v27 Editor check reported seven missing-type errors; existing order/duplicate and ignored performance assertions remain unchanged. Local Rustfmt and scoped diff checks pass. | implemented_pending_validation |
| Export-wizard stage-capacity benchmark | Import `std::hint::black_box` inside the existing test module; production wizard progress remains unchanged. | v27 Editor check reported five missing-function errors in the benchmark. Local Rustfmt and scoped diff checks pass; ignored Release evidence has not run. | implemented_pending_validation |

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/core/settings/tests/mod.rs` | `DB9D9B2C4E547E3BD1CB2F02ECE9D0565C3D3984F79094626B5076CA723F7348` |
| `zircon_editor/src/scene/modes/scene_mode_registry/entry_registration_tests.rs` | `5D0DAD3602E3AD052CCA4677003A76867A228A901C830A7C25E277E530172609` |
| `zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/progress.rs` | `00474E3C9C7EDD03E9F2CB2373F313E352F7C4709A7C74344B89204BE34F2C46` |

## Managed gate

The v27 Editor test-profile check reached Rust but failed before test execution
with 220 compiler errors across 110 owner files. These three fixes postdate
that snapshot; they address 24 diagnosable missing imports but have not been
recompiled, nor do they resolve the other shared failures. Do not infer that
Editor tests, ignored Release performance markers, allocation budgets, or
product p50/p95/p99 have passed. Recheck together with Runtime880 after
owner-scoped repairs to the remaining lower contracts.
