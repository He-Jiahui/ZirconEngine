---
related_code:
  - zircon_editor/src/core/asset/type_registry/registry.rs
  - zircon_editor/src/core/asset/type_registry/registry/batch.rs
  - zircon_editor/src/core/asset/type_registry/contribution.rs
  - zircon_editor/src/core/asset/type_registry/definition.rs
  - zircon_editor/src/core/plugin/extension_materialization.rs
implementation_files:
  - zircon_editor/src/core/asset/type_registry/registry.rs
  - zircon_editor/src/core/asset/type_registry/registry/batch.rs
  - zircon_editor/src/core/plugin/extension_materialization.rs
plan_sources:
  - docs/plans/zircon_editor/editor/09-editor-asset-management.md
  - docs/plans/zircon_editor/editor/09/failure-2026-07-17-asset-type-registry-clone-on-augment.md
tests:
  - zircon_editor/src/tests/editor_asset_type_registry/materialization.rs
  - zircon_editor/src/tests/editor_plugin_sdk.rs
doc_type: module-detail
---

# Asset Type Registry Delta And Generation Contract

`AssetTypeRegistry` is the materialized authority for built-in and plugin-provided editor asset types. Plugin catalog materialization applies one ordered contribution batch per catalog generation. If the batch accepts at least one contribution, the registry generation advances exactly once; an empty or all-rejected batch leaves definitions and generation unchanged.

## Transactional Contribution Batch

The batch path uses a strict validate/stage/finalize sequence:

1. Each contribution borrows the current materialized entry plus the accepted pending claims for that asset type. It validates scalar ownership, field values, collection ownership, and duplicate ids without mutating either authority.
2. A valid contribution records its claims and payload in `PendingEntryDelta`. A rejected contribution records its original input index and leaves no scalar, template, or command claim behind; later contributions continue.
3. Finalization walks touched asset types in stable id order, applies scalar assignments, extends each touched collection once, sorts that collection once, publishes new definitions, and advances the generation once for the whole batch.

The former `existing.clone()` rollback strategy and per-contribution binary insertion path are deleted. Existing entries are never cloned for rollback. New definitions remain private pending entries until complete validation and finalization. `apply_contribution` is a single-element façade over the same batch core, so there is no second compatibility implementation.

`AssetTypeRegistryBatchReport` is crate-internal evidence for accepted/rejected counts, touched asset types, ordered indexed errors, collection sort counts, and the full post-extend collection lengths processed by each sort. These are real execution counters used by the 1/100/10k/100k scale contract; they are not estimates of allocator behavior.

## Plugin Catalog Generation

`EditorPluginCatalog` increments its generation for each registered plugin and invalidates its materialized extension cache. `editor_extensions()` returns an `Arc<EditorExtensionCatalogReport>` from a `OnceLock`, so repeated reads of an unchanged catalog reuse the same extension registry and asset type registry. Registering another plugin clears the cache and publishes a report stamped with the new catalog generation.

During cache construction the catalog assigns a monotonic traversal sequence to every fallible extension operation. Asset type contributions are collected and applied once after traversal; indexed batch errors are mapped back to those sequence numbers and stably merged with view, drawer, tool-mode, graph, timeline, and command diagnostics. Deferred batching therefore does not reorder user-visible diagnostics.

This is a hard API cut: callers borrow the shared report; no owned compatibility clone is retained.

The workbench shell owns the enabled registry cache. Its key is the single extension-registration generation plus the sorted capability snapshot; its value is an `Arc<AssetTypeRegistry>`. Extension registration invalidates the cache. Capability changes miss by value without a parallel capability authority. Definition lookup, creation/context lookup, asset open, and both workbench projections route through `enabled_asset_types_for_shell`; the raw materializer is only the cache-miss builder.

## Validation Status

The tests now cover single-contribution failure atomicity, valid-invalid-valid isolation, empty/all-invalid batches, incomplete-new-type recovery, interleaved asset-type claims, one-pass template/command finalization, 1/100/10k/100k scale counters, stable ordering, catalog diagnostic-order parity, plugin-catalog `Arc` identity, host cache hit/materialization counts, and consumer routing. The dedicated static batch contract is green. Source-bound Cargo, broad validation, independent review, failure return, and managed commit remain pending, so the Editor09 failure is still open.
