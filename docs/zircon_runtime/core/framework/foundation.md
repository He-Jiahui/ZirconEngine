---
related_code:
  - zircon_runtime/src/core/framework/foundation/mod.rs
  - zircon_runtime/src/core/framework/foundation/config_manager.rs
  - zircon_runtime/src/core/framework/foundation/config_persistence_report.rs
  - zircon_runtime/src/core/framework/foundation/event_manager.rs
  - zircon_runtime/src/core/framework/foundation/module_identity.rs
  - zircon_runtime/src/foundation/module.rs
  - zircon_runtime/src/foundation/mod.rs
implementation_files:
  - zircon_runtime/src/core/framework/foundation/mod.rs
  - zircon_runtime/src/core/framework/foundation/config_manager.rs
  - zircon_runtime/src/core/framework/foundation/config_persistence_report.rs
  - zircon_runtime/src/core/framework/foundation/event_manager.rs
  - zircon_runtime/src/core/framework/foundation/module_identity.rs
plan_sources:
  - docs/plans/zircon_runtime/frameworks/05-subsystem-decoupling-contracts.md
  - docs/plans/zircon_runtime/frameworks/01-runtime-crate-decomposition.md
  - docs/plans/engine-code-structure-convention.md
tests:
  - tools/tests/frameworks_05_module_identity.py::Frameworks05ModuleIdentityChecks::test_foundation_module_identity_has_one_neutral_contract_owner
  - python -m unittest tools.tests.test_frameworks_05_layer_direction
doc_type: module-detail
---

# Foundation Framework Contracts

## Contract

`zircon_runtime::core::framework::foundation` owns the neutral configuration, event, persistence
report, and Foundation module-identity vocabulary shared by runtime assembly and sibling domains.
The contract tree contains declarations only; concrete persistence and service behavior remains in
`zircon_runtime::foundation`.

## Ownership and Constraints

The contract is folder-backed: `mod.rs` is structural, while `ConfigManager`,
`ConfigPersistenceReport`, `EventManager`, and `FOUNDATION_MODULE_NAME` each have a named owner.
The retired `core/framework/foundation.rs` file and the concrete declaration in
`foundation/module.rs` do not survive.

Asset and Platform module descriptors consume the neutral identity directly. The established
`zircon_runtime::foundation::FOUNDATION_MODULE_NAME` path remains a structural public projection
from the neutral owner, not a compatibility owner or forwarding declaration.

Eight existing Asset references still consume the concrete
`foundation::persistence::atomic_file` implementation. They are a separate open decoupling seam and
must not be confused with the module-identity cut.

## Relevant Validation

The focused guard was observed RED on the old single-file/concrete-owner structure and GREEN after
the hard cut. The complete Frameworks05 layer suite is 28/28 GREEN. The production-only dependency
audit is 2,401 references / 74 edges; Foundation identity definitions are 1 and retired production
identity consumers are 0. Managed Cargo remains blocked on the coordinator-owned immutable
full-compile-input validation copy.
