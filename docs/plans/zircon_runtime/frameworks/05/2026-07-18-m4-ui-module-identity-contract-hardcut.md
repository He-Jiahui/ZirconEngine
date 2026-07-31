---
related_code:
  - zircon_runtime/src/core/framework/ui.rs
  - zircon_runtime/src/core/framework/ui/module_identity.rs
  - zircon_runtime/src/ui/module.rs
  - zircon_runtime/src/ui/mod.rs
  - zircon_runtime/src/ui/prelude.rs
  - zircon_runtime/src/builtin/runtime_modules/tests/registration/behavior.rs
  - zircon_runtime/src/tests/runtime_absorption/builtin_modules/core_spine.rs
implementation_files:
  - zircon_runtime/src/core/framework/ui.rs
  - zircon_runtime/src/core/framework/ui/module_identity.rs
  - zircon_runtime/src/ui/module.rs
  - zircon_runtime/src/ui/mod.rs
  - zircon_runtime/src/ui/prelude.rs
plan_sources:
  - docs/plans/zircon_runtime/frameworks/05-subsystem-decoupling-contracts.md
  - docs/plans/zircon_runtime/frameworks/01-runtime-crate-decomposition.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - tools.tests.test_frameworks_05_layer_direction.Frameworks05LayerDirectionTests.test_ui_module_identity_has_one_neutral_contract_owner
  - tools.tests.test_frameworks_05_layer_direction
  - python tools/runtime_domain_dependency_audit.py
doc_type: milestone-detail
---

# Frameworks05 M4 UI Module Identity Contract Hard Cut

Plan: `docs/plans/zircon_runtime/frameworks/05-subsystem-decoupling-contracts.md`
Milestone: M4 module identity prerequisite
Status: implemented_static_and_review_passed_cargo_blocked
Date: 2026-07-18

## Delivered

| Slice | Status | Evidence |
|---|---|---|
| Neutral identity owner | implemented | `UI_MODULE_NAME` has one declaration in `core/framework/ui/module_identity.rs`. |
| Internal consumers | implemented | UI assembly, builtin registration, and Runtime core-spine checks consume the neutral contract owner. |
| Retired declaration | implemented | `ui/module.rs` no longer declares or exports the identity. |
| Public projection | structural | `zircon_runtime::ui` directly re-exports the neutral owner; App, Editor, and prelude public paths stay stable without a compatibility bridge. |
| TDD guard | passed | The focused UI unique-owner guard was observed RED before the owner existed and then GREEN. |
| Full static suite | passed | The atomic Foundation successor preserves this hard cut: Frameworks05 layer direction is 28/28 GREEN; production audit is 2,401 refs / 74 edges, retired Runtime-internal UI-root identity consumers are 0, and inherited target edges remain 0. |
| Guard structure | passed | The route suite remains 856 lines; reusable Scene/Input/UI/Foundation identity scanning lives in the 594-line focused helper. The four identity checks reuse one ordinal-sorted, process-local read-only Rust source inventory without changing assertions or exemptions. |
| Independent review | passed | Review rounds found 0/1/1 and then 0/1/0 in use-tree coverage. The scanner now handles whitespace, direct/grouped/module/root aliases, and path-aware relative facade access without rejecting neutral framework-relative access. Third-round Critical / Important / Minor is 0 / 0 / 0. |
| Managed Cargo | blocked | Acceptance requires the coordinator-owned immutable full-compile-input validation copy; shared-tree Cargo evidence is not accepted. |

## Architecture Decision

A runtime module identity consumed by assembly and higher packages is contract vocabulary, not UI
implementation detail. Keeping `UI_MODULE_NAME` in `ui/module.rs` made the public facade and internal
registration checks depend on a concrete owner that cannot survive clean `zr_ui` extraction.

The identity therefore moves to `core/framework/ui`. The concrete module consumes the same neutral
owner as internal registration. The established public Runtime facade remains a direct structural
re-export because Frameworks01 preserves public paths while internal owners move; it is not an
old-owner forwarding layer.

## Remaining M4 Work

This slice closes only the UI module-identity owner and atomically preserves the Scene, Platform,
and Input identity hard cuts. Other identities, remaining manager work, full managed Cargo
validation, and Frameworks01 physical crate extraction remain open. It does not promote M4 or the
parent plan to completed.
