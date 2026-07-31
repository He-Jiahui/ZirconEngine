---
related_code:
  - zircon_runtime/src/core/runtime/error.rs
  - zircon_runtime/src/core/runtime/mod.rs
  - zircon_runtime/src/core/mod.rs
  - zircon_runtime/src/core/runtime/lifecycle.rs
  - zircon_runtime/src/core/runtime/descriptors/module_order.rs
  - zircon_runtime/src/core/runtime/tasks/mod.rs
  - zircon_runtime/src/asset/pipeline/worker_pool.rs
  - zircon_runtime/src/prelude.rs
implementation_files:
  - zircon_runtime/src/core/runtime/error.rs
  - zircon_runtime/src/core/runtime/mod.rs
  - zircon_runtime/src/core/mod.rs
plan_sources:
  - docs/plans/zircon_runtime/frameworks/01-runtime-crate-decomposition.md
  - docs/plans/zircon_runtime/frameworks/02-module-kernel-and-lifecycle-unification.md
  - docs/plans/zircon_runtime/runtime/02-core-spine-and-root-surface.md
tests:
  - tools/tests/test_frameworks_01_runtime_error_owner_boundary.py
  - tools/tests/test_frameworks_02_core_error_single_source.py
doc_type: module-detail
---

# Core Runtime Error Contract

`zircon_runtime/src/core/runtime/error.rs` is the single physical owner of `CoreError` and
`CoreResult`. The enum describes kernel registry, module activation, service resolution, runtime
availability, channel/thread, and configuration failures. Several variants carry `ServiceKind`, so
the owner belongs with the runtime kernel rather than the neutral framework contract tree.

The supported public surface remains `zircon_runtime::core::{CoreError, CoreResult}`. That curated
root export is the stable runtime facade, not a migration bridge. The removed
`core::framework::error` module is not re-exported, aliased, or retained as a forwarding module.

## Dependency Boundary

- runtime kernel code imports the sibling error owner directly;
- framework traits may name `crate::core::CoreError` through the curated facade;
- the kernel error owner never imports `core::framework`;
- domain-specific errors remain in their lowest owner and are not added to `CoreError` merely to
  preserve an old dependency direction.

## Validation

The Frameworks01 owner-boundary guard requires the runtime error file, rejects the retired framework
file and imports, and proves there is exactly one `CoreError` definition. The Frameworks02
single-source guard continues to enforce the channel/thread variants, public root facade, and absence
of the retired parallel error enum.
