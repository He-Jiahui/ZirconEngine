---
related_code:
  - zircon_runtime/src/core/framework/scene/module_identity.rs
  - zircon_runtime/src/core/framework/scene/mod.rs
  - zircon_runtime/src/scene/module/mod.rs
  - zircon_runtime/src/scene/mod.rs
  - zircon_runtime/src/graphics/runtime_builtin_graphics/host/module_host/module_registration/module_descriptor.rs
  - zircon_editor/src/ui/host/module.rs
  - zircon_plugins/plugin_sdk/src/test.rs
  - zircon_plugins/zr_vm_language/runtime/src/tests/registration.rs
  - zircon_runtime/tests/runtime_plugin_world_extensions_contract.rs
implementation_files:
  - zircon_runtime/src/core/framework/scene/module_identity.rs
  - zircon_runtime/src/core/framework/scene/mod.rs
  - zircon_runtime/src/scene/module/mod.rs
  - zircon_runtime/src/scene/mod.rs
  - zircon_runtime/src/graphics/runtime_builtin_graphics/host/module_host/module_registration/module_descriptor.rs
  - zircon_editor/src/ui/host/module.rs
  - zircon_plugins/plugin_sdk/src/test.rs
  - zircon_plugins/zr_vm_language/runtime/src/tests/registration.rs
  - zircon_runtime/tests/runtime_plugin_world_extensions_contract.rs
plan_sources:
  - docs/plans/zircon_runtime/frameworks/05-subsystem-decoupling-contracts.md
  - docs/plans/zircon_runtime/frameworks/01-runtime-crate-decomposition.md
  - docs/plans/engine-code-structure-convention.md
tests:
  - tools.tests.test_frameworks_05_layer_direction.Frameworks05LayerDirectionTests.test_scene_module_identity_has_one_neutral_contract_owner
  - python -m unittest tools.tests.test_frameworks_05_layer_direction -v
  - fresh production-only runtime domain dependency audit
doc_type: milestone-detail
---

# Frameworks05 M4 Scene Module Identity Contract Hard Cut

Plan: `docs/plans/zircon_runtime/frameworks/05-subsystem-decoupling-contracts.md`
Milestone: M4 S3 graphics-scene boundary
Status: implemented_static_passed_cargo_pending
Date: 2026-07-18

## Delivered

| Slice | Status | Evidence |
|---|---|---|
| Neutral identity owner | implemented | `core::framework::scene::module_identity` uniquely defines `SCENE_MODULE_NAME`. |
| Concrete owner removal | implemented | `scene/module/mod.rs` no longer defines it and `scene/mod.rs` no longer exports it. |
| Consumer hard cut | implemented | Runtime src/integration tests, Editor and plugin SDK/runtime tests import the neutral contract. |
| Compatibility removal | implemented | concrete scene-root and scene-module-alias accesses are zero; no alias or forwarding export remains. |
| Dependency edge | passed | fresh production-only scan: 2,380 refs / 78 edges, graphics→scene=0. |
| Static suite | passed | focused guard RED before implementation, then GREEN; full Frameworks05 layer suite 24/24. |
| Independent review | passed | final expanded 27-path review: P0=0, P1=0, P2=0. |
| Managed Cargo | pending | Text01 has closed `sys-locale` into both canonical lockfiles; current-source reservation is not yet accepted. |

## Architecture Decision

A module dependency name is protocol data, not concrete Scene implementation behavior. Keeping it at
`scene` forced graphics, script and UI descriptors to import a neighboring implementation domain
solely to name a dependency. The canonical owner is therefore the neutral scene contract that will
move with `zr_contracts`.

The cut is direct. The concrete scene root does not re-export the constant, and no compatibility
module or duplicate definition remains. Callers now identify the same runtime module through the
contract path, so runtime behavior is unchanged while the physical crate dependency edge is gone.

## Remaining Scope

This closes the S3 module-name edge only. It does not mark Frameworks05 M4/M5 or Frameworks01 Phase
3 complete. Other module identities and remaining ui→graphics/workspace gates retain their own
owners and acceptance work.
