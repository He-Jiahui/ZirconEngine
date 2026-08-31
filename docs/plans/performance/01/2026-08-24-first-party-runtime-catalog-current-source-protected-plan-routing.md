---
title: First-Party Runtime Catalog Current Source Protected Plan Routing
date: 2026-08-24
scope:
  - zircon_plugins/first_party_runtime_catalog
status: routing_pending_owner_absorption_source_recheck_required
source_review:
  - docs/plans/performance/01/2026-08-24-first-party-runtime-catalog-current-source-performance-review.md
---

# First-Party Runtime Catalog Current Source Protected Plan Routing

The 5/5 Rust-file static review is complete against a shared modified baseline, but product and dynamic acceptance are not. This note routes findings without editing protected ledgers, shared source changes or independently owned plans.

| Owner plan | Required absorption |
|---|---|
| `docs/plans/optimize/zircon_plugins/06-first-party-plugin-source-editor-runtime-dist-catalog-profile-capability-closure-review.md` | Primary owner: update current coverage to 15/30, generate complete fine-grained provider/BuildSet closure, replace empty-vector drops with per-selection receipts and converge source/generated/native authorities. |
| `docs/plans/optimize/zircon_plugins/01-plugin-sdk-package-catalog-distribution-native-abi-review.md` | Make packaging/provider generation/capability/lifecycle equivalent and reject unsupported native/generated contributions instead of silently omitting them. |
| `docs/plans/optimize/zircon_plugins/07-first-party-asset-importer-source-dependency-subasset-artifact-determinism-sandbox-product-integration-review.md` | Classify and close the currently missing importer Runtime providers without adding parallel importer/catalog authorities. |
| `docs/plans/optimize/zircon_runtime/42-builtin-runtime-module-catalog-profile-target-feature-selection-extension-registration-capability-load-report-product-integration-review.md` | Own one immutable profile/target/dependency activation graph and fail Ready on missing required providers. |
| `docs/plans/optimize/zircon_app/01-product-host-bootstrap-loop-dynamic-runtime-shutdown-review.md` | Close all six App BuildSets against profile intent, invoke resolution once and publish phase/startup/first-frame receipts. |
| `docs/plans/optimize/zircon_tooling/20-cargo-package-workspace-feature-dependency-target-graph-build-receipt-review.md` | Persist the actual resolved Cargo package/feature graph and reject profile/buildset contradictions before build acceptance. |
| `docs/plans/optimize/zircon_tooling/32-hot-path-catalog-algorithmic-complexity-data-movement-batching-cache-locality-performance-governance-review.md` | Add provider-count/lookup/construction/allocation/binary-size gates and prove zero stable-frame catalog work. |

Protected `docs/plans/performance/review.md` and `pending.md` remain unchanged. Promotion requires source recheck after shared edits settle, owner absorption, 30-package classification, profile BuildSet closure, required-selection fail-closed behavior, source/generated/native parity, managed Windows tests and current-source WPR/ETW startup/memory/power evidence.
