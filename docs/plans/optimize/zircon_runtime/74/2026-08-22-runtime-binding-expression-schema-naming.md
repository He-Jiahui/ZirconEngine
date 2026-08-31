# Runtime74 Binding Expression Schema Naming

Plan: docs/plans/optimize/zircon_runtime/74-runtime-ui-template-component-binding-expression-model-event-command-hot-reload-product-integration-review.md
Milestone: P2 cleanup
Status: validation_pending
Files: ["docs/plans/optimize/zircon_runtime/74/2026-08-22-runtime-binding-expression-schema-naming.md","docs/ui-and-layout/ui-asset-documents-and-editor-protocol.md","docs/ui-and-layout/ui-asset-foundation-descriptors-contracts-invalidation.md","docs/zircon_runtime_interface/ui/mod.md","zircon_runtime/src/ui/template/asset/binding/validation.rs","zircon_runtime/src/ui/tests/asset_binding.rs","zircon_runtime/src/ui/tests/asset_binding/naming_contract.rs"]

- Date: 2026-08-22
- Owner: `optimize-runtime74-param-ref-compile-r3-bee4c707-20260822`
- Source item: `RTB-P2-001`
- Delivery state: implementation complete; grouped coordinator validation pending

## Scope Delivered

- The live validation predicate is named `is_runtime_binding_expression`; current production Rust
  contains no `M18` milestone identifier.
- Current contract prose names the versioned binding-expression schema/capability. Historical plan
  links, old target directories, and prior acceptance rows retain `M18` because they are evidence,
  not current runtime vocabulary.
- A source-boundary regression locks the predicate name and rejects reintroduction of milestone
  vocabulary in the production validation owner. It reads source text only and does not claim that
  parsing, compilation, or Runtime evaluation executed.

## Validation Contract

The P2 child validator checks the production/doc naming anchors and runs the exact source-boundary
regression `binding_validation_source_boundary_uses_schema_name_not_milestone`.
Its SHA-256 is `1E8F1D955ED3AA4EDB282B1B253631BB91E19CE57AB81687B115330E698E1459`.
It is pinned by the 84-task / 56-Cargo-group / 18-performance-row Runtime74 super-batch with
SHA-256 `92059C051B5E6B4341AB2B93242CE201E3F408A88251F313CDA021D8E846A9CC`.

This naming cleanup has no independent performance claim and emits zero performance rows. It rides
the grouped Runtime74 batch, whose 18 existing performance gates remain pending coordinator
execution. No Cargo or behavior pass is claimed.
