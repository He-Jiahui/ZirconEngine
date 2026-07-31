---
related_code:
  - zircon_runtime/src/core/framework/ui.rs
  - zircon_runtime/src/core/framework/ui/module_identity.rs
  - zircon_runtime/src/ui/module.rs
  - zircon_runtime/src/ui/mod.rs
  - zircon_runtime/src/ui/prelude.rs
implementation_files:
  - zircon_runtime/src/core/framework/ui.rs
  - zircon_runtime/src/core/framework/ui/module_identity.rs
plan_sources:
  - docs/plans/zircon_runtime/frameworks/05-subsystem-decoupling-contracts.md
  - docs/plans/zircon_runtime/frameworks/01-runtime-crate-decomposition.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - tools/tests/frameworks_05_module_identity.py::Frameworks05ModuleIdentityChecks::test_ui_module_identity_has_one_neutral_contract_owner
  - python -m unittest tools.tests.test_frameworks_05_layer_direction
  - python tools/runtime_domain_dependency_audit.py --pretty --output .codex/tmp/frameworks05-ui-current.json
doc_type: module-detail
---

# UI Framework Contracts

## Purpose

`zircon_runtime::core::framework::ui` owns UI vocabulary shared by Runtime assembly and higher
domains without requiring them to depend on the concrete UI implementation.

## Module Identity Ownership

`UI_MODULE_NAME` is activation and registration vocabulary, so its single declaration lives in
`core/framework/ui/module_identity.rs`. The concrete `UiModule`, builtin registration checks, and
Runtime core-spine checks consume that neutral owner directly.

The established `zircon_runtime::ui::UI_MODULE_NAME` path remains a structural public projection.
`ui/mod.rs` re-exports the neutral owner directly; it does not forward the identity through
`ui/module.rs`. The UI prelude projects the same facade item. No compatibility owner, alias, shim,
fallback, or duplicate constant remains.

## Dependency Direction

Concrete UI assembly may depend on UI, Input, Scene, and Render framework contracts. Runtime
assembly and sibling domains may consume the neutral UI identity, but they must not reach through
the concrete `ui` root for internal dependency vocabulary. External App and Editor callers remain
behind the public Runtime facade required by Frameworks01.

## Validation State

The focused unique-owner guard was observed RED before the neutral owner existed and GREEN after the
hard cut. The atomic Foundation successor preserves the complete Frameworks05 layer suite at 28/28
GREEN. The current production-only audit is 2,401 references / 74 edges; retired internal UI-root identity consumers are 0, and the inherited
`ui→input`, `graphics→platform`, and `input→platform` target edges remain 0. Independent review is
Critical / Important / Minor 0 / 0 / 0 after closing whitespace, alias, and relative-path scanner
findings. Managed Cargo remains blocked until the coordinator creates an immutable full-compile-input
validation copy.
