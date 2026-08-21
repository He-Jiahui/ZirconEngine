# Runtime75 Shared Component Catalog View

Plan: docs/plans/optimize/zircon_runtime/75-runtime-ui-component-catalog-widget-behavior-state-reducer-interaction-semantics-accessibility-product-integration-review.md
Milestone: M0
Status: validation_pending
Files: ["docs/plans/optimize/zircon_runtime/75/2026-08-22-shared-component-catalog-view.md","zircon_runtime/src/ui/template/asset/compiler/ui_document_compiler.rs","zircon_runtime/src/ui/tests/component_catalog/catalog_inventory.rs","zircon_editor/src/ui/component_registry/registry.rs","zircon_editor/src/ui/component_registry/tests.rs"]

- Date: 2026-08-22
- Integration owner: `optimize-runtime75-catalog-native-slot-batch-m0-r1-01a00797-20260822`
- Former owner: `optimize-runtime75-shared-catalog-view-m0-r1-01a00797-20260822`
  (`cancelled` after grouped transfer fingerprint
  `8dbf28b59546e7fe2f2ef68ab0a3da0952f780f867f80995da4dce5c644d1d61`)
- Source item: `RUW-P1-002` and the registry-clone portion of `RUW-P1-047`
- Inherited evidence: `docs/plans/zircon_editor/editor_ui/06/failure-2026-07-18-runtime-ui-component-catalog-deep-clone.md`
- Delivery state: implementation complete; grouped coordinator validation pending

## Problem

`UiComponentDescriptorRegistry` already exposed process-wide immutable showcase and Material
catalogs, but every `UiDocumentCompiler::default()` still called the owned showcase factory and
deep-cloned all 69 descriptors. The Editor retained registry also cloned both complete source
registries before cloning their descriptors again into its final merged `OnceLock` value.

This preserved mutability for custom compiler registries at the cost of making the default,
read-only path pay the same allocation and clone cost. It also obscured the intended M0 boundary:
shared built-in authority views should be immutable, while owned registries should be explicit
overlay or materialization inputs.

## Scope Delivered

- `UiDocumentCompiler` stores a `Cow<'static, UiComponentDescriptorRegistry>`.
- The default compiler borrows `editor_showcase_shared()` and performs no registry or descriptor
  clone. `with_component_registry(...)` remains an isolated owned override for custom registration.
- `with_shared_component_registry(...)` provides an explicit zero-clone constructor path for later
  immutable authority snapshots.
- The Editor retained registry starts empty, iterates the shared showcase followed by the shared
  Material catalog, and clones each descriptor only into the final process-wide merged value.
  Material still wins duplicate IDs, preserving current behavior.
- Owned catalog factories remain available for tests and explicit mutable copies. This slice does
  not misclassify those intentional builder uses as read-only hot paths.

## Deterministic Performance Gate

The ignored release benchmark constructs 256 compilers per sample. The legacy control clones the
69-descriptor showcase registry for every compiler; the optimized path constructs the default
compiler over the borrowed static view. It warms both paths, then records 21 alternating
legacy/optimized sample pairs with 11 legacy-first and 10 optimized-first pairs.

Each legacy sample performs exactly 17,664 descriptor clones (`69 * 256`); the optimized sample
performs zero descriptor clones. The marker includes both raw unsorted nanosecond series and
nearest-rank P50/P95 values so an external validator can recompute the result. Acceptance requires
`optimized_p95_ns * 4 <= legacy_p95_ns`, or at least 75% lower measured P95. Actual timing values
remain pending; the structural clone count is not reported as measured speedup.

## TDD And Static Evidence

- The pointer-identity regression is deterministically red on the prior implementation because
  `UiDocumentCompiler::default()` stored `editor_showcase_shared().clone()`.
- A custom descriptor test proves the owned override remains isolated from the shared catalog.
- The Editor registry test locks the 258-ID union and Material precedence for duplicate IDs.
- `rustfmt +1.94.1` completed for all four owned Rust files.
- Scoped `git diff --check` completed.
- Focused behavior tests, the ignored release benchmark, external marker validation, and package
  checks are pending a later multi-task coordinator batch. No Cargo or performance pass is claimed.

## Remaining Scope

This is the first M0 ownership slice, not the complete Runtime75 authority milestone. Provider
qualification, duplicate-ID diagnostics, schema hash/generation, v2 capability admission,
capability-filtered snapshots, Editor-local providers, palette revision caching, 10/1k/100k widget
workloads, and removal of downstream string classifiers remain open under Runtime75.
