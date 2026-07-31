---
related_code:
  - zircon_runtime/src/core/framework/physics/manager.rs
  - zircon_runtime/src/core/framework/physics/settings_store_error.rs
  - zircon_plugins/physics/runtime/src/manager/settings.rs
  - zircon_plugins/physics/runtime/src/manager/service.rs
implementation_files:
  - zircon_runtime/src/core/framework/physics/manager.rs
  - zircon_runtime/src/core/framework/physics/settings_store_error.rs
  - zircon_runtime/src/core/framework/physics/mod.rs
  - zircon_plugins/physics/runtime/src/manager/settings.rs
  - zircon_plugins/physics/runtime/src/manager/service.rs
plan_sources:
  - docs/plans/zircon_runtime/frameworks/01-runtime-crate-decomposition.md
  - docs/plans/zircon_runtime/frameworks/03-optional-features-and-profile-matrix.md
tests:
  - tools.tests.test_frameworks_01_physics_settings_error_boundary
  - physics_settings_store_errors_are_domain_owned_and_stable
doc_type: milestone-detail
---

# Frameworks01 M1 Physics Settings Error DAG Prerequisite

Plan: `docs/plans/zircon_runtime/frameworks/01-runtime-crate-decomposition.md`
Milestone: M1 owner-DAG prerequisite
Status: implemented_static_and_review_passed_cargo_pending
Date: 2026-07-18

## Delivered

| Slice | Status | Evidence |
|---|---|---|
| Physics error owner | implemented | `PhysicsSettingsStoreError` is uniquely defined beside the physics contract. |
| Trait hard cut | implemented | `PhysicsManager::store_settings` no longer returns kernel `CoreError`. |
| Plugin persistence boundary | implemented | config-store failures are explicitly projected to the typed physics persistence variant. |
| Compatibility removal | implemented | no old return type, `From<CoreError>` shim, duplicate enum, or alias survives. |
| Static guard | passed | focused guard observed RED before migration and is now GREEN. |
| Independent review | passed | final current-source review reports P0/P1/P2 = 0/0/0 after guard and status-record findings were closed. |
| Managed Cargo | pending | Runtime physics contract and plugin package tests remain coordinator FIFO pending. |

## Architecture Decision

The optional physics contract must not depend upward on the runtime kernel merely to report a
read-only backend or a settings-persistence failure. These failures belong to the physics service
surface, so `PhysicsSettingsStoreError` owns them next to `PhysicsManager`.

The concrete plugin remains responsible for persistence. It converts `CoreHandle::store_config`
failure text at that boundary without retaining `CoreError` as a source type, conversion impl, or
compatibility wrapper in the contract crate.

## Remaining M1 DAG Work

This removes only the physics settings error edge. Framework foundation and scene contracts still
name `CoreError`; the scene owner is frozen by a separate accepted static milestone and the
foundation owner has an active Runtime02 persistence handoff. Physical M1 crate extraction and all
managed acceptance gates remain open.
