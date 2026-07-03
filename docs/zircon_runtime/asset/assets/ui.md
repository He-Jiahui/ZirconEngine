---
related_code:
  - zircon_runtime/src/asset/assets/font.rs
  - zircon_runtime/src/asset/assets/ui.rs
  - zircon_runtime/src/asset/assets/imported.rs
  - zircon_runtime/src/asset/assets/mod.rs
  - zircon_runtime/src/asset/mod.rs
  - zircon_runtime/src/asset/facade/impls.rs
  - zircon_runtime/src/asset/importer/error.rs
  - zircon_runtime/src/asset/importer/ingest/import_ui_zui_asset.rs
  - zircon_runtime/src/asset/importer/ingest/ui_v2_document_import.rs
  - zircon_runtime/src/asset/importer/ingest/import_ui_theme_asset.rs
  - zircon_runtime/src/asset/importer/ingest/import_ui_icon_asset.rs
  - zircon_runtime/src/asset/importer/ingest/asset_importer.rs
  - zircon_runtime/src/ui/v2/loader.rs
  - zircon_runtime/src/asset/artifact/cache_payload.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/loading/load_asset.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/loading/acquire_asset.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/loading/load_imported_asset.rs
  - zircon_runtime/src/asset/pipeline/manager/resource_sync/register_project_resource.rs
  - zircon_runtime/src/asset/pipeline/manager/resource_sync/store_runtime_payload.rs
  - zircon_runtime/src/asset/pipeline/manager/builtins/resource_manager_with_builtins.rs
  - zircon_runtime/src/asset/tests/assets/ui.rs
  - zircon_runtime/src/asset/tests/assets/ui/references.rs
  - zircon_runtime/src/asset/tests/assets/ui/wrappers.rs
  - zircon_runtime/src/asset/tests/assets/ui/importer.rs
  - zircon_runtime/src/asset/tests/assets/ui/fixture_validation.rs
  - zircon_runtime/src/asset/tests/assets/importer/typed_toml_ui.rs
  - zircon_runtime/src/ui/tests/v2_asset/asset_loading.rs
  - zircon_runtime/src/ui/tests/v2_asset/file_cache.rs
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/typed_error_convergence/ui_asset_documents.rs
  - zircon_runtime/src/tests/runtime_absorption/naming_boundary/runtime_15_m2/asset_schema.rs
implementation_files:
  - zircon_runtime/src/asset/assets/ui.rs
  - zircon_runtime/src/asset/assets/imported.rs
  - zircon_runtime/src/asset/assets/mod.rs
  - zircon_runtime/src/asset/mod.rs
  - zircon_runtime/src/asset/facade/impls.rs
  - zircon_runtime/src/asset/importer/error.rs
  - zircon_runtime/src/asset/importer/ingest/import_ui_zui_asset.rs
  - zircon_runtime/src/asset/importer/ingest/ui_v2_document_import.rs
  - zircon_runtime/src/asset/importer/ingest/import_ui_theme_asset.rs
  - zircon_runtime/src/asset/importer/ingest/import_ui_icon_asset.rs
  - zircon_runtime/src/asset/importer/ingest/asset_importer.rs
  - zircon_runtime/src/ui/v2/loader.rs
  - zircon_runtime/src/asset/artifact/cache_payload.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/loading/load_asset.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/loading/acquire_asset.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/loading/load_imported_asset.rs
  - zircon_runtime/src/asset/pipeline/manager/resource_sync/register_project_resource.rs
  - zircon_runtime/src/asset/pipeline/manager/resource_sync/store_runtime_payload.rs
  - zircon_runtime/src/asset/pipeline/manager/builtins/resource_manager_with_builtins.rs
  - zircon_runtime/src/asset/tests/assets/ui.rs
  - zircon_runtime/src/asset/tests/assets/ui/references.rs
  - zircon_runtime/src/asset/tests/assets/ui/wrappers.rs
  - zircon_runtime/src/asset/tests/assets/ui/importer.rs
  - zircon_runtime/src/asset/tests/assets/ui/fixture_validation.rs
  - zircon_runtime/src/asset/tests/assets/importer/typed_toml_ui.rs
  - zircon_runtime/src/ui/tests/v2_asset/asset_loading.rs
  - zircon_runtime/src/ui/tests/v2_asset/file_cache.rs
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/typed_error_convergence/ui_asset_documents.rs
plan_sources:
  - user: 2026-06-12 implement editor UI architecture from docs/plans/zircon_editor/editor_ui
  - docs/plans/zircon_editor/editor_ui/05-ui-asset-management.md
  - docs/plans/zircon_editor/editor_ui/11-zui-suffix-convergence-and-ui-toml-retirement.md
tests:
  - rustfmt --edition 2021 --check touched UI asset/theme importer files (2026-06-12 UiThemeAsset slice: passed)
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-ui-theme-asset-0612-coremin-check --message-format short --color never (2026-06-12 UiThemeAsset slice: passed with existing warnings)
  - cargo test -p zircon_runtime --lib ui_theme_asset --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-ui-theme-asset-0612-coremin-check --message-format short --color never -- --test-threads=1 --nocapture (2026-06-12 UiThemeAsset slice: timed out after 904s while compiling the runtime lib-test target; no Rust diagnostics returned; matching UI-theme cargo/rustc processes stopped)
  - rustfmt --edition 2021 touched UiIconAsset importer, facade, cache, load, sync, and test files (2026-06-12 UiIconAsset slice: passed)
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-ui-icon-asset-0612-coremin-check --message-format short --color never (2026-06-12 UiIconAsset slice: passed with existing warnings)
  - cargo test -p zircon_runtime --lib ui_icon --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-ui-icon-asset-0612-coremin-check --message-format short --color never -- --test-threads=1 --nocapture (2026-06-12 UiIconAsset slice: passed, 4 passed / 0 failed / 3563 filtered out)
  - cargo test -p zircon_runtime --lib asset::tests::assets::ui::project_manager_scans --no-default-features --features core-min --locked --jobs 1 --target-dir target\codex-runtime11-coremin-tasks-0621 --message-format short --color never -- --test-threads=1 --nocapture (2026-06-21 Runtime 11 UI cache boundary: passed, 4 passed / 0 failed / 4693 filtered out)
  - scoped rustfmt/static scans for Runtime 15 F5 UI asset document typed errors (2026-06-27: passed); Cargo deferred because external cargo/rustc lanes were active
  - rustfmt --edition 2021 --check touched .zui M1 loader/importer/test files (2026-06-28 Editor UI 11 M1: passed)
  - cargo test -p zircon_runtime --lib v2_asset --locked --jobs 1 --target-dir E:\cargo-targets\zircon-zui-m1 --message-format short --color never -- --test-threads=1 --nocapture (2026-06-28 Editor UI 11 M1: blocked by existing runtime graphics/test compile errors before filtered tests ran)
  - cargo test -p zircon_runtime --lib importer_decodes_zui_view_and_style_assets_from_zui --locked --jobs 1 --target-dir E:\cargo-targets\zircon-zui-m1 --message-format short --color never -- --test-threads=1 --nocapture (2026-06-28 Editor UI 11 M1: blocked because external zircon_runtime/Cargo.toml and Cargo.lock ttf-parser state requires lock update)
  - zircon_runtime/src/tests/runtime_absorption/naming_boundary/runtime_15_m2/asset_schema.rs::runtime_15_font_ui_asset_schema_names_use_current_policy_terms (2026-06-29 Runtime 15 M2 font/UI asset schema naming hard cutover: runtime_15_font_ui_asset_schema_naming_hard_cutover_static_passed_cargo_deferred static guard added; Cargo deferred by Runtime 15 implementation-slice cadence)
  - python -m unittest tools.tests.test_zui_static_suffix_convergence.ZuiStaticSuffixConvergenceTests.test_runtime_asset_ui_reference_tests_use_zui_suffix tools.tests.test_zui_docs_suffix_convergence.ZuiDocsSuffixConvergenceTests.test_runtime_asset_ui_reference_zui_guard_status_is_recorded (2026-07-02 Editor UI 11 M5 runtime asset UI reference `.zui` fixture guard: passed 2/2)
  - python -m unittest tools.tests.test_zui_static_suffix_convergence tools.tests.test_zui_docs_suffix_convergence (2026-07-02 Editor UI 11 M5 runtime asset UI reference `.zui` fixture guard: passed 41/41)
  - rustfmt --edition 2021 --check zircon_runtime\src\asset\tests\assets\ui.rs zircon_runtime\src\asset\tests\assets\ui\references.rs (2026-07-02 Editor UI 11 M5 runtime asset UI reference `.zui` fixture guard: passed)
  - cargo test -p zircon_runtime --lib ui_asset_direct_references --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-zui-reference-fixture-0702 --message-format short --color never -- --test-threads=1 --nocapture (2026-07-02 Editor UI 11 M5 runtime asset UI reference `.zui` fixture guard: timed out after 604s with no Rust diagnostics and no target test result; matching target-dir cargo/rustc/link process scan found 0)
doc_type: module-detail
---

# Runtime UI Assets

`zircon_runtime::asset::assets::ui` owns the typed asset wrappers for runtime UI documents. The first wave keeps three existing UI document families:

- `UiLayoutAsset`, `UiWidgetAsset`, and `UiStyleAsset` wrap v1 `UiAssetDocument` payloads and validate `[asset.kind]`.
- `UiV2ViewAsset`, `UiV2ComponentAsset`, and `UiV2StyleAsset` wrap v2 `UiV2AssetDocument` payloads. `.zui` is the converged production UI v2 document suffix for component, view, and style documents.
- `UiThemeAsset` wraps `zircon_runtime_interface::ui::style::UiThemeDocument` directly. It is the first standalone theme asset type for editor UI plan 05; production editor style-token files now use `.zui` root/style documents when they are UI v2 documents.
- `UiIconAsset` describes icon sources for editor/tool UI chrome without rasterizing them yet. It supports inline SVG text, external SVG asset URIs, and bitmap asset URIs through `UiIconSource { kind, text, uri }`.

`UiThemeAsset::from_toml_str(...)` parses sparse theme TOML through the interface theme DTO, so omitted palette, typography, shape, spacing, control-size, and elevation fields inherit the default dark-theme values. `UiThemeAsset::to_toml_string()` serializes the same document shape for asset cache and tooling roundtrips.

`UiIconAsset::from_toml_str(...)` validates a positive finite `default_size`, a non-empty `semantic_id`, and source-specific payload fields. External `svg_asset` and `bitmap` sources must carry a valid `ResourceLocator` URI and expose that URI through `direct_references()`. Inline SVG sources must carry non-empty `text` and do not create asset dependency edges. The source shape intentionally stays as a plain struct plus `UiIconSourceKind` enum, because bincode artifact caching requires every serialized field to stay present in order; authoring-only serde conveniences such as internally tagged enums or skipped `Option` fields break binary restore.

## Facade And Import

`UiThemeAsset` uses the existing `UiStyleMarker` and `AssetKind::UiStyle` rather than adding a new public `ResourceKind`. The typed facade label is `ui_theme`, which lets editor/tool surfaces distinguish the payload while preserving the stable UI style resource family used by handles, registry rows, and readiness state.

The built-in `.zui` importer registers as `zircon.builtin.ui_document.zui`. It parses and validates suffix-authority through `UiZuiAssetLoader`, then `ui_v2_document_import.rs` maps `asset.kind` directly to `ImportedAsset::UiV2Component`, `ImportedAsset::UiV2View`, or `ImportedAsset::UiV2Style`. Runtime 15 M2 font/UI asset schema naming hard cutover removed the unused `UiV2DocumentImportProfile::{LegacyToml,Zui}` branch; component/view/style distinction now comes from the current `.zui` document and wrapper boundaries instead of a legacy TOML profile switch. The old `.ui.toml` and `.v2.ui.toml` source importers remain removed; registering those suffixes still fails at the importer registry boundary.

The built-in importer registers `.theme.toml` as `zircon.builtin.ui_theme.toml` and emits `ImportedAsset::UiTheme`. The generic `.toml` data importer still handles ordinary TOML files, while `.ui.toml` and `.v2.ui.toml` are no longer stageable UI document suffixes after the `.zui` cutover. Project scan writes the imported theme to the same `.zasset` artifact cache path as other imported assets, and `ProjectAssetManager::load_imported_asset(...)` restores UI style-family payloads in this order: v2 style, theme, then v1 style.

`UiIconAsset` uses the existing `TextureMarker` and `AssetKind::Texture` with typed facade label `ui_icon`. This keeps icon authoring assets in the image/texture resource family that later atlas and rasterization work will consume, without adding a public icon resource kind before plan 05 M4 defines the runtime atlas channel. The built-in importer registers `.icon.toml` as `zircon.builtin.ui_icon.toml` and emits `ImportedAsset::UiIcon`. Project scan writes icon artifacts under the texture library directory, stores/restores the `UiIconAsset` payload through the binary artifact cache, and `ProjectAssetManager::load_imported_asset(...)` checks UI icon payloads before ordinary texture payloads for `AssetKind::Texture`.

The artifact cache treats v1 and v2 UI document assets as parser-owned document payloads instead of raw bincode DTOs. `UiLayoutAsset`, `UiWidgetAsset`, `UiStyleAsset`, `UiV2ViewAsset`, `UiV2ComponentAsset`, and `UiV2StyleAsset` are normalized to TOML text in `ArtifactCacheAsset` and restored through their typed TOML parsers, preserving the validated authoring shape while avoiding bincode's `deserialize_any` boundary for dynamic UI document values. `UiThemeAsset` and `UiIconAsset` stay as direct cache variants because their payload structs are explicit and bincode-safe.

## Typed Error Boundary

Runtime 15 F5 UI asset document typed errors (`runtime_15_ui_asset_document_typed_errors_static_passed_cargo_deferred`) tightened the wrapper/importer boundary. V1 wrappers now return `UiAssetDocumentError::Parse(#[from] UiAssetError)`, v2 wrappers return `UiV2AssetDocumentError::Parse(#[from] UiV2AssetError)`, and theme/icon TOML parsing preserves `toml::de::Error` as the source instead of storing formatted strings. `UiIconAssetDocumentError::InvalidSourceUri` carries the original `ResourceLocatorError`, while other icon validation failures are explicit variants.

`AssetImportError::UiV2Document`, `AssetImportError::UiThemeDocument`, and `AssetImportError::UiIconDocument` are the importer-side typed boundaries for current UI document source import. The ZUI/theme/icon importers no longer convert document failures into `AssetImportError::Parse(String)`. Direct `UiV2ComponentAsset::from_zui_str(...)` still rejects non-component `.zui` documents through the typed wrapper boundary.

## Coverage

`zircon_runtime/src/asset/tests/assets/ui.rs` covers sparse theme TOML parsing, default palette inheritance, facade label and marker kind, TOML roundtrip, default importer selection for `.theme.toml`, and project-manager scan/restore of the theme payload as `AssetKind::UiStyle`. The same test module covers `.icon.toml` parsing, external icon dependency extraction, `ui_icon` facade label and `TextureMarker` mapping, default importer selection, and project-manager scan plus artifact restore of icon payloads as `AssetKind::Texture`. Runtime 11 asset triage also keeps the UI cache boundary under `asset::tests::assets::ui::project_manager_scans`, covering layout/widget/style asset kind assignment and `.zui` component/view/style restore from `.zasset`.

Runtime 15 typed-error coverage adds `ui_asset_wrappers_preserve_typed_parse_sources`, `ui_icon_asset_reports_typed_validation_errors`, `importer_preserves_typed_theme_and_icon_document_sources`, and the registry rejection coverage for deprecated UI document suffix importers. Editor UI 11 M1/M5 extends coverage with `.zui` view/style loader acceptance, uppercase `.ZUI` file-cache root compilation, `.zui` view/style importer materialization, wrapper rejection of non-component `.zui` documents requested through `UiV2ComponentAsset::from_zui_str`, and `.ui.toml`/`.v2.ui.toml` staging rejection in `tools/zircon_build.py`. `runtime_15_font_ui_asset_schema_names_use_current_policy_terms` rejects reintroducing `UiV2DocumentImportProfile` / `LegacyToml` into `asset/importer/ingest/ui_v2_document_import.rs`, locks the direct `.zui` caller path through `asset/importer/ingest/import_ui_zui_asset.rs`, and cross-checks `asset/assets/font.rs` keeps the schema-v1 render-mode input named `schema_v1_render_mode`. The static review guard is `review_f5_ui_asset_documents_use_typed_errors_before_import_boundary`.

`editor_ui_11_m5_runtime_asset_ui_reference_fixture_zui_guard_passed` records the active reference-fixture cleanup for `zircon_runtime/src/asset/tests/assets/ui.rs` and `zircon_runtime/src/asset/tests/assets/ui/references.rs`. `tools/tests/test_zui_static_suffix_convergence.py::test_runtime_asset_ui_reference_tests_use_zui_suffix` locks `ui_asset_direct_references_include_collected_resource_dependencies` and `ui_v2_asset_direct_references_include_imports_and_resources` so the `ui_asset_references` / `ui_v2_asset_references` tests use `.zui` widget/style import locators instead of retired `.ui.toml` / `.v2.ui.toml` suffixes. This is fixture-only convergence: explicit deprecated-suffix importer rejection tests remain in their own owners, and production UI document source import stays `.zui` only.
