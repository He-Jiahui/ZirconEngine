---
related_code:
  - zircon_runtime/src/core/framework/input/module_identity.rs
  - zircon_runtime/src/core/framework/input/mod.rs
  - zircon_runtime/src/input/module/descriptor.rs
  - zircon_runtime/src/input/module/mod.rs
  - zircon_runtime/src/input/module/module_type.rs
  - zircon_runtime/src/input/mod.rs
  - zircon_runtime/src/ui/module.rs
implementation_files:
  - zircon_runtime/src/core/framework/input/module_identity.rs
  - zircon_runtime/src/core/framework/input/mod.rs
  - zircon_runtime/src/input/module/descriptor.rs
  - zircon_runtime/src/input/module/mod.rs
  - zircon_runtime/src/input/module/module_type.rs
  - zircon_runtime/src/input/mod.rs
  - zircon_runtime/src/ui/module.rs
plan_sources:
  - docs/plans/zircon_runtime/frameworks/05-subsystem-decoupling-contracts.md
  - docs/plans/zircon_runtime/frameworks/01-runtime-crate-decomposition.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - tools.tests.test_frameworks_05_layer_direction.Frameworks05LayerDirectionTests.test_input_module_identity_has_one_neutral_contract_owner
  - tools.tests.test_frameworks_05_layer_direction
  - python tools/runtime_domain_dependency_audit.py
doc_type: milestone-detail
---

# Frameworks05 M4 Input Module Identity Contract Hard Cut

Plan: `docs/plans/zircon_runtime/frameworks/05-subsystem-decoupling-contracts.md`
Milestone: M4 module identity prerequisite
Status: implemented_static_and_review_passed_cargo_blocked
Date: 2026-07-18

## Delivered

| Slice | Status | Evidence |
|---|---|---|
| Neutral identity owner | implemented | `INPUT_MODULE_NAME` has one declaration in `core/framework/input/module_identity.rs`. |
| Internal consumers | implemented | Input descriptor/type, UI, builtin registration, and Runtime absorption use the neutral contract owner. |
| Retired declaration | implemented | `input/module/descriptor.rs` no longer declares or exports the identity. |
| Public projection | structural | `zircon_runtime::input` directly re-exports the neutral owner; App and prelude public paths stay stable without a compatibility bridge. |
| Guard owner split | implemented | Scene/Input/UI/Foundation identity checks live in `tools/tests/frameworks_05_module_identity.py`; the route suite stays at 856 lines and the focused helper is 594 lines. One ordinal-sorted, process-local read-only Rust source inventory is reused by the four identity checks without changing their assertions or exemptions. |
| TDD guard | passed | Focused Input unique-owner guard was observed RED before the owner existed and then GREEN. |
| Full static suite | passed | The atomic Foundation successor preserves this hard cut: Frameworks05 layer direction is 28/28 GREEN; production audit is 2,401 refs / 74 edges with `ui→input=0` and both platform target edges still 0. |
| Formatting | passed | Canonical Rust 1.94.1 rustfmt, Python bytecode compilation, and scoped diff-check are clean. |
| Independent review | passed | First review found 0 Critical / 1 Important / 0 Minor because the guard exempted the whole concrete Input subtree. The exemption is now limited to real facade/test surfaces, all production Input paths run through the generic scanner, descriptor/module-type carry explicit neutral assertions, and the independent recheck is Critical 0 / Important 0 / Minor 0. |
| Managed Cargo | blocked | Acceptance requires the coordinator-owned immutable full-compile-input validation copy; shared-tree Cargo evidence is not accepted. |

## Architecture Decision

UI needs the Input module identity to declare activation order, but it does not need the Input
implementation. Owning the constant in `input/module/descriptor.rs` created a concrete `ui→input`
edge that would survive as an invalid dependency when `zr_input` and `zr_ui` are extracted.

The identity therefore moves to `core/framework/input`, beside the traits and DTOs already shared by
both domains. The concrete descriptor and every internal cross-domain consumer read the neutral
owner. The established public Runtime facade remains a direct structural re-export, not an old-owner
forwarding layer.

## Remaining M4 Work

This slice closes only the Input identity edge and preserves the Platform/Scene identity results in
the atomic successor scope. Other module identities, remaining manager work, whole-workspace Cargo
validation, and Frameworks01 physical crate extraction remain open. It does not promote M4 or the
parent plan to completed.
