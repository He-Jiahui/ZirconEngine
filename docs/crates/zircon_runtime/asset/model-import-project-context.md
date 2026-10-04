---
related_code:
  - zircon_runtime/src/asset/importer/contract.rs
  - zircon_runtime/src/asset/importer/ingest/import_from_source.rs
  - zircon_runtime/src/asset/importer/ingest/import_model.rs
  - zircon_runtime/src/asset/project/manager/scan_and_import.rs
  - zircon_runtime/src/asset/tests/assets/importer/shader_model.rs
implementation_files:
  - zircon_runtime/src/asset/importer/contract.rs
  - zircon_runtime/src/asset/importer/ingest/import_from_source.rs
  - zircon_runtime/src/asset/importer/ingest/import_model.rs
  - zircon_runtime/src/asset/project/manager/scan_and_import.rs
plan_sources:
  - docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
  - docs/plans/zircon_runtime/shader/01-shader-asset-kinds-and-zshader-v2.md
  - user: 2026-07-12 continue shader broad-gate convergence
tests:
  - zircon_runtime/src/asset/tests/assets/importer/shader_model.rs::importer_backfills_virtual_geometry_for_model_toml_without_dropping_base_mesh
  - zircon_runtime lib-test binary filter: shader
doc_type: module-detail
---

# Model Import Project Context

Persisted `.model.toml` documents are project assets. Their mesh fields use persisted project references, so importing them requires the same registry and project-root resolver used by `ProjectAssetManager::scan_and_import`.

`AssetImporter::import_from_source` remains appropriate for source formats that do not carry project references, such as WGSL, OBJ, and glTF. For `.model.toml`, `.scene.toml`, and `.zmaterial`, `import_context` rejects a context without a project resolver by returning `AssetImportError::ProjectContextRequired`. This is an intentional hard boundary: an importer must not reinterpret path hints or fabricate runtime locators without the project registry.

Tests that exercise a self-contained `.model.toml` still provide an explicit resolver. An empty `AssetRegistryIndex` and empty root list are sufficient when the fixture has no persisted mesh reference; their presence proves the test is using the project-aware path, while unresolved references continue to fail through the normal resolver contract. Assertions borrow the imported root entry so validation does not require cloning the complete model payload.

On 2026-07-13, the virtual-geometry backfill fixture was moved from `import_from_source` to an explicit `AssetImportContext` with a project resolver. The exact current-source test passed 1/1. The broader `shader` filter then reported 384 passed, 14 failed, and 4 ignored; the model-import project-context failure no longer appears.
