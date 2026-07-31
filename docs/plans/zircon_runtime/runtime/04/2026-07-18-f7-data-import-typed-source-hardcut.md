---
related_code:
  - zircon_runtime/src/asset/importer/error.rs
  - zircon_runtime/src/asset/importer/contract.rs
  - zircon_runtime/src/asset/importer/ingest/import_data_asset.rs
implementation_files:
  - zircon_runtime/src/asset/importer/error.rs
  - zircon_runtime/src/asset/importer/contract.rs
  - zircon_runtime/src/asset/importer/ingest/import_data_asset.rs
  - tools/tests/test_runtime_asset_pipeline_audit.py
plan_sources:
  - docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
  - docs/plans/engine-code-review-findings-2026-06.md
  - docs/plans/engine-code-structure-convention.md
tests:
  - tools.tests.test_runtime_asset_pipeline_audit.RuntimeAssetPipelineAuditTests.test_data_import_errors_preserve_typed_sources
  - asset::importer::ingest::import_data_asset::tests::source_text_decode_error_retains_utf8_source
  - asset::importer::ingest::import_data_asset::tests::toml_data_parse_error_retains_toml_source
  - asset::importer::ingest::import_data_asset::tests::json_data_parse_error_retains_json_source
doc_type: milestone-detail
---

# Runtime04 F7 Data Import Typed Source Hard Cut

Plan: `docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md`
Review finding: F7
Status: implemented_static_passed_cargo_pending
Date: 2026-07-18

## Delivered

| Slice | Status | Evidence |
|---|---|---|
| UTF-8 source decoding | implemented | `SourceTextDecode` retains source path and `FromUtf8Error`. |
| TOML data parsing | implemented | `import_plain_toml_data` uses contextual `TomlDeserialize`. |
| JSON data parsing | implemented | `import_json_data` uses contextual `JsonDeserialize`. |
| String flattening removal | implemented | the three paths no longer construct `Parse(format!(...))`. |
| Static contract | passed | focused RED before implementation, then Runtime04 audit 2/2 GREEN. |
| Independent review | passed | exact six-path review: P0=0, P1=0, P2=0. |
| Rust behavior | pending | three source-chain tests are implemented; managed Cargo is FIFO pending. |

## Boundary

This slice changes only decoding and parser failures that have a concrete lower-level source.
Semantic rejections remain typed by their existing variants. No `From` compatibility route maps
the new variants back to `Parse(String)`, and no facade or duplicate error owner is introduced.

The Runtime04 parent plan is concurrently leased by the project source-index failure owner, so this
record is child-owned and does not mutate that Session's parent-plan scope. Runtime04 remains
`in_progress`; this record does not close its broader asset/package gates or unrelated open failures.
