---
title: Runtime Resource Registration Concurrent Factory Source Slice
doc_type: implementation-record
status: source_complete_validation_pending
implementation_status: complete
validation_status: rustfmt_static_guard_passed_managed_cargo_pending
owner: Runtime15
related_code:
  - zircon_runtime/src/plugin/extension_registry/register/resource_registration.rs
  - zircon_runtime/src/plugin/extension_registry/register/resource_registration/poison_recovery_tests.rs
plan_sources:
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
  - docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md
tests:
  - resource_registration_factory_can_retry_after_a_panicking_invocation
  - resource_registration_factory_builds_a_fresh_value_for_each_world
  - resource_registration_factory_allows_concurrent_world_initialization
  - resource_registration_duplicate_registration_returns_typed_error
  - resource_registration_rejects_unknown_owner_before_mutating_registry
  - resource_registration_rejects_non_runtime_owner_before_mutating_registry
  - resource_factory_panic_is_reported_without_partial_world_mutation
---

# Runtime Resource Registration Concurrent Factory Source Slice

## Decision

`ResourceRegistration` is a repeatable, immutable factory shared by every world
created from a finalized extension plan. The production owner no longer wraps
the factory in a mutable initializer mutex; the public bound is `Fn() + Send +
Sync`, so independent worlds may initialize the same registered resource
concurrently. Factory execution is caught before insertion into `World`, so a
plugin panic becomes a keyed world-registration error and a later world may
retry the same immutable factory. Duplicate type registration remains an
explicit typed error from the extension-point owner.

## Completed Items

- Preserved panic retry and fresh-per-world behavior regressions.
- Added a fail-closed factory-panic boundary that preserves the panic payload,
  prevents partial resource insertion, and keeps retry semantics for later
  worlds.
- Added a two-thread factory regression with a bounded overlap observation
  inside the factory, proving both world initializations overlap without
  turning a serialized implementation into a test hang.
- Kept the resource registration owner folder-backed; no compatibility API,
  duplicate factory, or world-level cache was introduced.

## Evidence And Remaining Gate

The source gate confirms the production owner contains no `Mutex`, `FnMut`, or
production `expect`, uses the shared immutable `Arc<dyn Fn(&mut World) ->
Result<_, _> + Send + Sync>` factory, catches only the pre-insertion factory
phase, and maps duplicate insertion to `DuplicatePluginResource`. The focused
test owner contains seven behavior tests,
including deterministic concurrent initialization, unknown-owner,
non-runtime-owner, and keyed panic-recovery cases. Scoped Rust formatting and
`git diff --check` pass.

Current-source fingerprint (2026-08-30): `resource_registration.rs` is 162 lines
with SHA-256 `441003C19AC8BED8595A390C93BCA2079CB87731BDF6AB059134D136B127C016`,
and its folder-backed test owner is 205 lines with SHA-256
`E1CC9CCCD42FA5A689867E8E88C8240489F4B705DC26F584F76415A6EF940C5F`. The
unknown-owner regression verifies that an uninterned `PluginModuleId` is rejected
before the typed resource table is mutated. The non-runtime-owner regression keeps
this family aligned with component registration: an interned editor or other
non-`<plugin>.runtime` module is rejected before the table changes.

Managed Cargo and wider plugin/runtime integration remain pending behind the
active validation lease. The focused `validate-matrix.ps1` admission was
attempted for `zircon_runtime` with the panic regression filter on 2026-08-30,
but the coordinator rejected it before Cargo because pre-existing unmanaged
artifacts were detected on D:/E:/F: validation targets. No artifact was
removed, and no Cargo result is claimed from that attempt. This source slice
makes no CPU, GPU, energy, or power claim and does not close Runtime15 or
trigger a milestone commit.

Status token:
`runtime_15_resource_registration_concurrent_factory_source_complete_static_passed_cargo_pre_admission_blocked`.
