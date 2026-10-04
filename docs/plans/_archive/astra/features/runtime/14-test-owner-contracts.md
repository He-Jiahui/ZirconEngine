---
status: in_progress
plan_sources:
  - docs/plans/optimize/zircon_runtime/58-runtime-plugin-interface-bridge-slot-generation-strong-weak-native-vm-lifecycle-diagnostics-product-integration-review.md
---

# Test owner contracts

## Repairs

Window registry invariant tests now compile inside the implementation owner and
retain access to deliberate corrupt-state fixtures without widening production
fields. Native plugin and UI profiling fixtures use their actual test owners.
Diagnostics benchmarks declare the existing group count explicitly.

Preference tests borrow the underlying PlatformManager while keeping the fixture
worker pool alive. Mobile and browser export assertions use slices for different
fragment counts. Shader source checks reuse the parent module's source reader.

Resource durability and typed-error source checks point to zr_resource. Durability
checks follow the transaction replacement, writable-target flush, and directory
sync sequence. Rename checks cover both offline staging and the online commit
operation after the resource publication split.

## Plan completion list

| Batch | Work | Status | Validation evidence |
|---|---|---|---|
| M17 | Test ownership, platform/export fixtures, resource source contracts | implemented_pending_validation | Batch with M13-M16 and earlier optimization regressions; no standalone compile |

The archived preceding run failed with 1 lib and 102 lib-test diagnostics before
tests ran. These source repairs do not establish test acceptance or performance.
