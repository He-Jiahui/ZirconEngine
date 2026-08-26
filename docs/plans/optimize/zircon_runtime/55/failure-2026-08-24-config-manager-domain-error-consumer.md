---
handoff_kind: failure
status: open
created_at: 2026-08-24
summary_slug: config-manager-domain-error-consumer
origin_plan: docs/plans/zircon_runtime/frameworks/01-runtime-crate-decomposition.md
fixing_plan: docs/plans/optimize/zircon_runtime/55-runtime-foundation-module-config-event-service-driver-manager-persistence-lifecycle-product-integration-review.md
origin_child_dir: docs/plans/zircon_runtime/frameworks/01
fixing_child_dir: docs/plans/optimize/zircon_runtime/55
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/core/framework/foundation/config_manager_error.rs
  - zircon_runtime/src/core/framework/foundation/config_manager.rs
  - zircon_runtime/src/foundation/tests.rs
tests:
  - python -B -m unittest tools.tests.test_frameworks_01_contracts_kernel_test_boundary -v
  - .codex/skills/zircon-dev/scripts/validate-matrix.ps1 -Package zircon_runtime -LibTests -TestFilter foundation_registry_services_do_not_retain_the_runtime_root -VerboseOutput
---

# Runtime55: migrate the ConfigManager domain-error consumer

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/frameworks/01-runtime-crate-decomposition.md`
- 来源执行切片：Frameworks01 contracts/kernel boundary guard convergence
- 修复责任计划：`docs/plans/optimize/zircon_runtime/55-runtime-foundation-module-config-event-service-driver-manager-persistence-lifecycle-product-integration-review.md`
- 交接原因：Runtime55 owns the Foundation integration-test consumer while Frameworks01 owns the contract DAG correction.

## 失败现象与复现证据

Frameworks01 strengthened the contracts/kernel boundary guard and obtained a focused TDD RED with
exactly two violations: `foundation/config_manager.rs` and `scene/mod.rs` imported
`crate::core::CoreError`. The production contract now owns `ConfigManagerError`; the static guard is
GREEN `3/3`, and a current-source call-site audit found one stale typed assertion outside the
Frameworks01 ownership boundary.

`zircon_runtime/src/foundation/tests.rs` is a dirty blob owned by active Runtime55 session
`optimize-runtime55-foundation-empty-driver-hard-cut-r1-20260823`. Its
`foundation_registry_services_do_not_retain_the_runtime_root` test still compares
`ConfigManager::set_value` with `CoreError::RuntimeUnavailable`. After the hard cut, the left side is
`Result<(), ConfigManagerError>`, so the assertion must use the contract-owned error.

## 最低共享层根因

Runtime55 owns the Foundation module integration test and its current dirty blob. Frameworks01 owns
the contract DAG correction but must not overwrite Runtime55's active empty-driver hard-cut changes.
The consumer migration is therefore routed to Runtime55 as an exact one-assertion update.

## 架构修复验收

- Change the assertion to `ConfigManagerError::RuntimeUnavailable`, imported from
  `crate::core::framework::foundation` or referenced through that canonical path.
- Preserve Runtime55's existing Foundation descriptor assertions and current blob content.
- Do not add `From<CoreError>`, cross-type `PartialEq`, aliases, re-exports, or any compatibility
  bridge. The contract must remain independent of the runtime kernel.
- Run the focused Foundation test through the managed Windows validator once the existing foreign
  `zr_rhi_wgpu` current-source compiler blockers have converged.

## 禁止临时方案

- Do not add `From<CoreError>`, cross-type `PartialEq`, aliases, re-exports, or any compatibility bridge.
- Do not overwrite unrelated Runtime55 changes in the shared dirty Foundation test blob.
- Do not claim the managed Foundation gate passed from the static boundary guard alone.

## 修复结果与回传

Return this handoff as fixed when the current blob uses `ConfigManagerError`, the Frameworks01
boundary guard remains GREEN, and the focused managed Foundation test executes successfully. Until
then Frameworks01 records source implementation complete but does not claim the product Rust gate.

## Current state

Open: `consumer_migration_required / managed_validation_blocked_by_foreign_rhi`.
