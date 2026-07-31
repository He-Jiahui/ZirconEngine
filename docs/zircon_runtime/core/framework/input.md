---
related_code:
  - zircon_runtime/src/core/framework/input/mod.rs
  - zircon_runtime/src/core/framework/input/module_identity.rs
  - zircon_runtime/src/input/module/descriptor.rs
  - zircon_runtime/src/input/module/module_type.rs
  - zircon_runtime/src/input/mod.rs
  - zircon_runtime/src/ui/module.rs
implementation_files:
  - zircon_runtime/src/core/framework/input/mod.rs
  - zircon_runtime/src/core/framework/input/module_identity.rs
plan_sources:
  - docs/plans/zircon_runtime/frameworks/05-subsystem-decoupling-contracts.md
  - docs/plans/zircon_runtime/frameworks/01-runtime-crate-decomposition.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - tools/tests/frameworks_05_module_identity.py::Frameworks05ModuleIdentityChecks::test_input_module_identity_has_one_neutral_contract_owner
  - python -m unittest tools.tests.test_frameworks_05_layer_direction
  - python tools/runtime_domain_dependency_audit.py --pretty --output .codex/tmp/frameworks05-input-current.json
doc_type: module-detail
---

# Input Framework Contracts

## Purpose

`zircon_runtime::core::framework::input` owns the input traits, DTOs, event vocabulary, and module
identity that Runtime assembly and higher domains share without depending on the concrete Input
implementation.

## Module Identity Ownership

`INPUT_MODULE_NAME` is dependency vocabulary used by Input assembly, UI, builtin registration, App
profile selection, and the Runtime prelude. Its single declaration lives in
`core/framework/input/module_identity.rs`. The concrete descriptor and `InputModule` type consume the
neutral owner directly; UI no longer imports the Input implementation root to declare its dependency.

The established `zircon_runtime::input::INPUT_MODULE_NAME` path remains a structural facade export.
`input/mod.rs` re-exports the neutral owner directly and does not forward through
`input/module/descriptor.rs`. This preserves the public Runtime facade required by Frameworks01
without keeping a legacy owner, alias, shim, or duplicate constant.

## Dependency Direction

Concrete Input depends on platform contracts and input contracts. UI may depend on the neutral input
contract, but not on the concrete `input` module root. That direction permits future `zr_input` and
`zr_ui` extraction without an implementation-root crate edge.

## Validation State

The focused unique-owner guard was observed RED before the neutral owner existed and GREEN after the
hard cut. It includes concrete nested/alias deny fixtures and neutral/external facade allow fixtures.
The current full Frameworks05 layer suite is 28/28 GREEN; the production dependency audit reports
2,401 references / 74 edges with `ui→input=0`, `graphics→platform=0`, and `input→platform=0` for the
targeted implementation-root identities. Managed Cargo remains blocked until the coordinator can
create an immutable full-compile-input validation copy.
