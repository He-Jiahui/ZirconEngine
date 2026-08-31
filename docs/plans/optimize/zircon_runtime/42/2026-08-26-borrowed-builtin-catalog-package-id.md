---
title: Runtime42 Borrowed Builtin Catalog Package Id
category: zircon_runtime
report_id: Runtime42-borrowed-builtin-catalog-package-id-2026-08-26
date: 2026-08-26
session_id: root-runtime42-borrowed-builtin-catalog-package-id-20260826
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime42 Borrowed Builtin Catalog Package Id

## Scope

This slice optimizes package-id dispatch while building the runtime plugin builtin catalog. It
does not change catalog rows, descriptor identity, category, capability, optional-feature,
classification, maturity, status, target, output order, builder ownership, or public plugin APIs.

## Change

- The builtin row's existing `&'static str` package id now travels with its descriptor builder as
  an identified stack tuple through augmentation, optional-feature, and classification stages.
- Category and extra-capability augmentation receive that borrowed id directly. Optional-feature
  and classification dispatch consume and return the same identified builder.
- Four `package_id().to_string()` allocations per catalog descriptor are removed. The current 23
  catalog rows therefore avoid 92 temporary string allocations per catalog rebuild.
- The pipeline still runs category/capability augmentation before optional features and
  classification, then unwraps the tuple and uses the repository-blessed
  `.map(RuntimePluginDescriptorBuilder::build)` storage path.
- Existing Rust catalog regressions remain the behavior oracle for category, extra capabilities,
  optional features, maturity, and capability status. A Python source contract prevents owned
  dispatch ids or stage-order drift from returning.

## Deterministic Performance Evidence

The independent release model uses all 23 current package ids across 4,096 catalog rebuilds and
four identical dispatch stages. Each run contains 21 alternating owned-id/borrowed-id sample
pairs, and both variants must produce the same checksum.

| Evidence | Owned dispatch ids | Borrowed row id | Result |
|---|---:|---:|---:|
| Measured allocations | 376,832 | 0 | 100% fewer |
| Run 1 P50 | 40.132 ms | 2.160 ms | 94.617% faster |
| Run 1 P95 | 61.988 ms | 4.162 ms | 93.286% faster |
| Run 2 P50 | 37.375 ms | 2.197 ms | 94.122% faster |
| Run 2 P95 | 58.935 ms | 3.625 ms | 93.849% faster |
| Run 3 P50 | 34.405 ms | 1.964 ms | 94.293% faster |
| Run 3 P95 | 55.283 ms | 3.385 ms | 93.877% faster |

The managed gate requires the exact 376,832-to-0 allocation counts, identical dispatch checksum,
at least 50% P50 improvement, and at least 25% P95 improvement.

## Acceptance

- TDD RED observed two missing borrowed-id pipeline failures while all four existing Rust-oracle
  names were present.
- `tools.tests.test_runtime42_borrowed_builtin_catalog_package_id_performance_contract` passes
  3/3 locally.
- Six exact production files pass `rustfmt --check`; model compilation, three independent model
  runs, and scoped `git diff --check` pass locally.
- The builtin catalog Rust regression batch, source contracts, formatting, performance model, and
  scoped diff checks are submitted together in one coordinator validation ticket.
- Commit integration and automatic WeCom performance notification remain gated on managed
  validation and the repository's independent-review policy.

## Remaining Parent-plan Work

Runtime42 still needs a single immutable composition plan, effective-manifest registration
filtering, unique feature-provider authority, required capability admission, build-set-stable
schema, final App/Core graph equivalence, lifecycle receipts, and product-scale qualification.
