---
record_kind: milestone
status: partially_implemented
created_at: 2026-09-18
plan: docs/plans/astra/optimize/01-review-and-repair.md
milestone: IMP-P1-061/062 UI document importer dependency and schema receipt
session: astra-ui-document-importer-schema-receipt-20260918
---

# UI document importer dependency and schema receipt

This bounded slice records the current `.zui` importer contract without
claiming full migration or product acceptance. The importer consumes the
already-admitted `UiZuiAssetLoader` document, projects only canonical persisted
asset references into the root dependency set, strips component labels to the
file-level locator, deduplicates references, and rejects normalized or transient
schemes instead of silently dropping them.

The importer also writes the source and target schema versions into the shared
`AssetSchemaMigrationReport`. The current `UiZuiAssetLoader` intentionally
accepts only `UI_V2_ASSET_SCHEMA_VERSION`; older and future versions therefore
fail closed before a receipt is published. This is a traceable current-schema
receipt, not an executable historical migration chain.

## Evidence

- Focused importer, capability, owner-boundary, and Frameworks05 asset tests:
  `python -m unittest tools.tests.test_plugin_structure_audit_manifest_schema_asset_importers tools.tests.test_plugin_structure_audit_manifest_schema_asset_importer_capability_gates tools.tests.test_plugin_structure_audit_manifest_schema_asset_importer_semantics tools.tests.test_plugin_validate_asset_importer_owner_boundaries tools.tests.test_frameworks_05_asset_ui_boundary -q`
  — **22/22 passed**.
- Runtime hot-path static suite: `python -m unittest discover -s tools/tests
  -p 'test_runtime*.py' -q` — **2002/2002 passed**.
- `python tools/audits/audit_plugin_structure.py --json` reports zero manifest-schema,
  importer-registration, capability, skeleton, and distribution-boundary
  violations, with a **41/41** standalone distribution matrix.
- Editor, plugin, and Hub static suites remained green in the same source
  snapshot: **1841/1841**, **463/463**, and **42/42** respectively.
- `rustfmt +1.94.1 --edition 2021 --config skip_children=true --check
  zircon_plugins/ui_document_importer/runtime/src/lib.rs` completed successfully;
  scoped `git diff --check` completed with only the repository's existing
  line-ending notice.

Post-edit SHA-256 fingerprints:

```text
zircon_plugins/ui_document_importer/runtime/src/lib.rs 1CA795BE8A86952A8B63D8761D9F2F484BF88BD4AD8C7184317C96152136D1E4
```

## Boundary

No Cargo, native DLL, GPU, or product command ran. The managed validation lane
is still blocked by the external dirty `E:\Git\zr_vm` dependency and the
coordinator's stale unmanaged fixture (`D:\ZirconBuilds\mvp-test-fixtures-28916`);
an official cleanup plan also encountered a coordinator SQLite lock and was
stopped in read-only mode. The remaining work for `IMP-P1-062` is an explicit
source-schema transformation chain, source-span diagnostics, and save/reopen
roundtrip, followed by managed Windows validation.

No commit was created.
