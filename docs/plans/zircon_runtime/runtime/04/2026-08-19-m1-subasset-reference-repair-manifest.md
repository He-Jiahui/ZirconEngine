Plan: docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
Milestone: M1
Status: review_forward_repaired_managed_validation_pending
Files: ["zircon_runtime/src/asset/reference_resolution_error.rs", "zircon_runtime/src/asset/reference_resolver.rs", "zircon_runtime/src/asset/migration/resolver.rs", "zircon_runtime/src/asset/importer/ingest/import_model.rs", "docs/plans/zircon_runtime/runtime/04/2026-08-19-m1-subasset-reference-repair-manifest.md"]

# Runtime 04 M1 subasset reference repair manifest

## Scope

This manifest closes the lower-layer identity failure recorded in
[`failure-2026-08-18-missing-subasset-parent-fallback.md`](failure-2026-08-18-missing-subasset-parent-fallback.md).
It preserves a persisted subasset label as part of the resource identity across the shared
resolver, legacy migration resolver, and importer repair reporting.

## Required managed gates

- The focused shared-resolver regression must prove stale GUID repair preserves an exact existing
  label and a missing label produces a stable typed dangling diagnostic without parent fallback.
- The focused migration regressions must prove a matching GUID repairs a moved source, a parent
  GUID repairs to an exact labeled subasset, and an absent subasset label is classified as
  `DanglingReference`.
- The focused importer regression must prove a repaired subasset reference preserves `sub`.
- The coordinator validation manifest must compile and run only from an approved non-C target
  root, then bind its exact source hashes to this file list.

## Exclusions

- No caller, importer, or registry alias may strip a missing label or retry the parent locator.
- No generated artifacts, compatibility layer, or new crate are part of this repair.
