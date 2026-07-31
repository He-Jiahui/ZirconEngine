---
related_code:
  - zircon_runtime/src/core/framework/foundation/mod.rs
  - zircon_runtime/src/core/framework/foundation/config_manager.rs
  - zircon_runtime/src/core/framework/foundation/config_persistence_report.rs
  - zircon_runtime/src/core/framework/foundation/event_manager.rs
  - zircon_runtime/src/core/framework/foundation/module_identity.rs
  - zircon_runtime/src/foundation/module.rs
  - zircon_runtime/src/foundation/mod.rs
  - zircon_runtime/src/asset/module.rs
  - zircon_runtime/src/platform/module.rs
implementation_files:
  - zircon_runtime/src/core/framework/foundation/mod.rs
  - zircon_runtime/src/core/framework/foundation/config_manager.rs
  - zircon_runtime/src/core/framework/foundation/config_persistence_report.rs
  - zircon_runtime/src/core/framework/foundation/event_manager.rs
  - zircon_runtime/src/core/framework/foundation/module_identity.rs
  - zircon_runtime/src/foundation/module.rs
  - zircon_runtime/src/foundation/mod.rs
  - zircon_runtime/src/asset/module.rs
  - zircon_runtime/src/platform/module.rs
plan_sources:
  - docs/plans/zircon_runtime/frameworks/05-subsystem-decoupling-contracts.md
  - docs/plans/zircon_runtime/frameworks/01-runtime-crate-decomposition.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - tools.tests.test_frameworks_05_layer_direction.Frameworks05LayerDirectionTests.test_foundation_module_identity_has_one_neutral_contract_owner
  - tools.tests.test_frameworks_05_layer_direction
  - python -B -m unittest tools.tests.test_frameworks_05_layer_direction
  - rustfmt +1.94.1 --edition 2021 --check <exact Rust manifest>
  - git -c core.safecrlf=false diff --check -- <exact40 manifest>
  - python tools/runtime_domain_dependency_audit.py
  - python .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py --json
doc_type: milestone-detail
---

# Frameworks05 M4 Foundation Contract Owner Hard Cut

Plan: docs/plans/zircon_runtime/frameworks/05-subsystem-decoupling-contracts.md
Milestone: M4
Status: waiting_prerequisite_owner_commits_and_re_review
Files: ["docs/plans/zircon_runtime/frameworks/05-subsystem-decoupling-contracts.md", "docs/plans/zircon_runtime/frameworks/05/2026-07-18-m4-foundation-module-identity-contract-hardcut.md", "docs/plans/zircon_runtime/frameworks/05/2026-07-18-m4-input-module-identity-contract-hardcut.md", "docs/plans/zircon_runtime/frameworks/05/2026-07-18-m4-platform-module-identity-contract-hardcut.md", "docs/plans/zircon_runtime/frameworks/05/2026-07-18-m4-ui-module-identity-contract-hardcut.md", "docs/zircon_runtime/core/framework/foundation.md", "docs/zircon_runtime/core/framework/input.md", "docs/zircon_runtime/core/framework/platform.md", "docs/zircon_runtime/core/framework/ui.md", "tools/tests/frameworks_05_module_identity.py", "tools/tests/test_frameworks_05_layer_direction.py", "zircon_runtime/src/asset/module.rs", "zircon_runtime/src/builtin/runtime_modules/tests/registration/behavior.rs", "zircon_runtime/src/core/framework/foundation.rs", "zircon_runtime/src/core/framework/foundation/config_manager.rs", "zircon_runtime/src/core/framework/foundation/config_persistence_report.rs", "zircon_runtime/src/core/framework/foundation/event_manager.rs", "zircon_runtime/src/core/framework/foundation/mod.rs", "zircon_runtime/src/core/framework/foundation/module_identity.rs", "zircon_runtime/src/core/framework/input/mod.rs", "zircon_runtime/src/core/framework/input/module_identity.rs", "zircon_runtime/src/core/framework/platform/mod.rs", "zircon_runtime/src/core/framework/platform/module_identity.rs", "zircon_runtime/src/core/framework/ui.rs", "zircon_runtime/src/core/framework/ui/module_identity.rs", "zircon_runtime/src/foundation/mod.rs", "zircon_runtime/src/foundation/module.rs", "zircon_runtime/src/graphics/runtime_builtin_graphics/host/module_host/module_registration/module_descriptor.rs", "zircon_runtime/src/input/mod.rs", "zircon_runtime/src/input/module/descriptor.rs", "zircon_runtime/src/input/module/mod.rs", "zircon_runtime/src/input/module/module_type.rs", "zircon_runtime/src/platform/mod.rs", "zircon_runtime/src/platform/module.rs", "zircon_runtime/src/script/vm/reflection/tests.rs", "zircon_runtime/src/tests/plugin_extensions/profile_maturity.rs", "zircon_runtime/src/tests/runtime_absorption/builtin_modules/core_spine.rs", "zircon_runtime/src/ui/mod.rs", "zircon_runtime/src/ui/module.rs", "zircon_runtime/src/ui/prelude.rs"]
Date: 2026-07-18

## Scope Delivered

| Slice | Status | Evidence |
|---|---|---|
| Folder-backed contract | implemented | The retired `core/framework/foundation.rs` owner is deleted; structural `foundation/mod.rs` projects four named contract owners. |
| Neutral identity owner | implemented | `FOUNDATION_MODULE_NAME` has one declaration in `core/framework/foundation/module_identity.rs`. |
| Internal consumers | implemented | Foundation, Asset, Platform, builtin registration, and Runtime core-spine consumers use the neutral contract owner. |
| Public projection | structural | `zircon_runtime::foundation` directly re-exports the neutral owner; no duplicate declaration, alias, shim, or old-owner forwarder remains. |
| TDD guard | passed | The focused guard was observed RED against the old single-file/concrete-owner structure and then GREEN. |
| Full static suite | passed | The post-review-correction current-source run is 28/28 GREEN in 151.241 seconds. Production audit is 2,415 refs / 74 edges, the Foundation identity definition count is 1, and retired production identity consumers are 0. The four identity checks reuse one ordinal-sorted, process-local read-only Rust source inventory; assertions and narrow exemptions are unchanged. |
| Structure audit | scoped pass | Foundation has a real `FoundationModule` owner, runtime stub usage is empty, and non-network server debt is 0. The global module gate remains RED with four foreign migration-debt groups; Runtime is not claimed converged. |
| Remaining seam | open | `asset→foundation=8` remains through the concrete `foundation::persistence::atomic_file` implementation and is outside this identity slice. |
| Independent review | blocked | The current exact40 review is Critical 2 / Important 1 / Minor 0. It found that clean HEAD lacks the Scene neutral identity predecessor and the Runtime02/Runtime07 Foundation persistence, config-manager, and event contracts consumed by this manifest. The earlier 0/0/0 verdict applies only to the shared-tree source view and is not milestone acceptance. |
| Managed Cargo | blocked | Acceptance requires the Scene, Runtime02, and Runtime07 prerequisite owner commits, followed by a fresh coordinator-owned immutable full-compile-input validation copy. Shared-tree Cargo evidence is not accepted. |

## Fresh Testing Evidence

The current shared-tree exact40 source passes `python -B -m unittest tools.tests.test_frameworks_05_layer_direction` with 28/28 tests in 151.241 seconds. Exact Rust `rustfmt +1.94.1 --edition 2021 --check` and exact40 `git -c core.safecrlf=false diff --check` also pass. The suite reports one Foundation identity definition, ordinal-sorted process-local source inventory, and no retired concrete module-identity consumer. These checks include uncommitted prerequisite sources outside exact40, so they are static directional evidence only and are not clean-HEAD acceptance.

## Review

The fresh exact40 clean-HEAD review is Critical 2 / Important 1 / Minor 0. Critical findings are the omitted Scene neutral identity predecessor and omitted Runtime02/Runtime07 Foundation support; the Important finding corrects the current audit count and historical review overclaim. A final independent review must run after those owner commits and managed validation.

## Architecture Decision

Foundation module identity is dependency vocabulary consumed by Asset and Platform, not concrete
Foundation implementation detail. Its canonical owner is therefore the neutral Foundation contract.
The touched contract was also converted from a mixed single file into a folder-backed owner tree so
`mod.rs` stays structural and each public declaration remains independently reviewable under the
engine code-structure convention.

The established Runtime facade is kept only as the public package projection required by
Frameworks01. It reads the neutral owner directly and does not preserve the deleted declaration.

## Remaining M4 Work

This worktree slice implements the Foundation, Platform, Input, and UI identity hard cuts, but exact40
is not independently committable until the Scene identity predecessor and Runtime02/Runtime07
Foundation support land through their owners. Other non-identity seams, managed Cargo, and
Frameworks01 physical crate extraction remain open. It does not promote M4 or the parent plan to
completed.
