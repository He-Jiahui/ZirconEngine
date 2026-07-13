---
related_code:
  - zircon_runtime/src/core/framework/mod.rs
  - zircon_runtime/src/core/framework/platform/mod.rs
  - zircon_runtime/src/core/framework/platform/runtime_target_mode.rs
  - zircon_runtime/src/platform/config.rs
  - zircon_runtime/src/platform/capability/report.rs
  - zircon_runtime/src/builtin/runtime_modules.rs
  - zircon_runtime/src/plugin/runtime_plugin/descriptor.rs
implementation_files:
  - zircon_runtime/src/core/framework/platform/mod.rs
  - zircon_runtime/src/core/framework/platform/runtime_target_mode.rs
plan_sources:
  - docs/plans/zircon_runtime/frameworks/05-subsystem-decoupling-contracts.md
  - docs/plans/zircon_runtime/frameworks/05/failure-2026-07-13-core-contract-reverse-dependencies.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - tools/tests/test_frameworks_05_layer_direction.py::Frameworks05LayerDirectionTests::test_runtime_target_mode_has_one_neutral_owner
  - python tools/runtime_domain_dependency_audit.py --pretty --output .codex/tmp/frameworks05-runtime-target-mode-neutral-owner.json
  - python -m unittest tools.tests.test_frameworks_03_contract_feature_boundary tools.tests.test_runtime_domain_dependency_audit
  - managed Windows cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked (pending until the repository Cargo testing stage is available)
doc_type: module-detail
---

# Platform Framework Contracts

## Purpose

`zircon_runtime::core::framework::platform` owns platform vocabulary that must be shared by runtime assembly, capability policy, plugin manifests, application hosts, the editor, and external plugins without depending on any of those concrete domains. It currently owns the canonical `RuntimeTargetMode` declaration.

## Ownership

`RuntimeTargetMode` selects the client-runtime, server-runtime, or editor-host family. The enum is a serialized contract, not builtin-module assembly behavior. Its unique declaration therefore lives in `core/framework/platform/runtime_target_mode.rs`; concrete target manifests and module lists remain in `builtin`, capability/config evaluation remains in `platform`, and plugin availability/validation remains in `plugin`.

The former `builtin/runtime_modules/ids/target_mode.rs` owner and `zircon_runtime::builtin::RuntimeTargetMode` re-export were deleted in one hard cut. All Runtime, App, Editor, plugin SDK, and first-party plugin call sites use `zircon_runtime::core::framework::platform::RuntimeTargetMode`. No alias, compatibility module, prelude projection, or duplicate declaration remains.

## Dependency Direction

The neutral owner makes the intended direction explicit: concrete platform, builtin assembly, plugin, editor, and application domains may depend on `core/framework/platform`; the framework layer never depends on those facades. This removes all seven production `platform→builtin` references recorded by the Frameworks05 failure handoff while preserving the enum variants and serde wire names.

## Validation State

The focused owner guard was introduced red, then passed after the hard cut. The production dependency audit moved from 2,146 references / 76 edges with seven `platform→builtin` references to 2,136 / 75 with zero `platform→builtin`; the open handoff total fell from 39 to 32. Managed Windows Cargo validation remains pending until the shared repository testing stage is available.
