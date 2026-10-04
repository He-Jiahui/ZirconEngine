use std::collections::HashSet;

use zircon_runtime::asset::{
    AssetImportContext, AssetImportError, AssetImportOutcome, AssetSchemaMigrator, AssetUri,
    ImportedAsset, StaticAssetSchemaMigrator, UiV2ComponentAsset, UiV2StyleAsset, UiV2ViewAsset,
};
use zircon_runtime::core::resource::ResourceScheme;
use zircon_runtime::ui::v2::UiZuiAssetLoader;
use zircon_runtime_interface::ui::v2::{
    UiV2AssetDocument, UiV2AssetKind, UI_V2_ASSET_SCHEMA_VERSION,
};

mod capability;
mod plugin;

pub use capability::{
    IMPORTER_CAPABILITY, MODULE_NAME, NATIVE_PLUGIN_ID, NATIVE_REQUESTED_CAPABILITIES,
    NATIVE_RUNTIME_ENTRY, NATIVE_RUNTIME_REGISTRATION_MANIFEST, PLUGIN_ID, RUNTIME_CAPABILITY,
    RUNTIME_CRATE_NAME, UI_DOCUMENT_IMPORTER_DECLARATION,
};
pub use plugin::{
    asset_importer_descriptors, dist_module_manifest, module_descriptor, package_manifest,
    plugin_registration, runtime_capabilities, runtime_module_manifest, runtime_plugin,
    runtime_plugin_descriptor, runtime_selection, supported_platforms, supported_targets,
    UiDocumentImporterRuntimePlugin, UI_DOCUMENT_IMPORTER_DIST_CRATE_NAME,
    UI_DOCUMENT_IMPORTER_DIST_RUNTIME_ENTRY,
};

pub fn import_ui_zui_document(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
    let document = context.source_str()?;
    let parsed = UiZuiAssetLoader::load_zui_str(document).map_err(|source| {
        AssetImportError::UiV2Document {
            context: "parse .zui ui asset",
            source: source.into(),
        }
    })?;
    let dependencies = collect_zui_dependencies(&parsed)?;
    let migration_report =
        StaticAssetSchemaMigrator::new(UI_V2_ASSET_SCHEMA_VERSION, UI_V2_ASSET_SCHEMA_VERSION)
            .migrate_source_schema(Some(parsed.asset.version))?;
    let imported = match parsed.asset.kind {
        UiV2AssetKind::View => ImportedAsset::UiV2View(UiV2ViewAsset { document: parsed }),
        UiV2AssetKind::Style | UiV2AssetKind::ThemeTokens => {
            ImportedAsset::UiV2Style(UiV2StyleAsset { document: parsed })
        }
        UiV2AssetKind::Component => {
            ImportedAsset::UiV2Component(UiV2ComponentAsset { document: parsed })
        }
    };
    let mut outcome = AssetImportOutcome::new(context.uri.clone(), imported)
        .with_migration_report(migration_report);
    for dependency in dependencies {
        outcome = outcome.with_dependency(dependency);
    }
    Ok(outcome)
}

/// Collects the canonical asset references declared by a `.zui` document.
///
/// The v2 document keeps imports as authored strings so the UI runtime can also
/// resolve local asset IDs. Import products, however, persist dependencies as
/// `AssetUri`s and the current context has no canonical local-ID resolver. Keep
/// this conversion strict at the importer boundary: canonical URIs are projected
/// into the dependency set, while local/unsupported forms fail closed instead of
/// being silently dropped when the source is indexed.
fn collect_zui_dependencies(
    document: &UiV2AssetDocument,
) -> Result<Vec<AssetUri>, AssetImportError> {
    let capacity = document
        .imports
        .widgets
        .len()
        .saturating_add(document.imports.styles.len())
        .saturating_add(document.imports.resources.len().saturating_mul(2));
    let mut dependencies = Vec::with_capacity(capacity);
    let mut seen = HashSet::with_capacity(capacity);

    let mut push = |kind: &str, reference: &str| -> Result<(), AssetImportError> {
        let reference = reference.trim();
        if reference.is_empty() {
            return Err(AssetImportError::Parse(format!(
                ".zui {kind} dependency cannot be empty"
            )));
        }
        let uri = AssetUri::parse(reference).map_err(|source| {
            AssetImportError::Parse(format!(
                "invalid .zui {kind} dependency `{reference}`: {source}"
            ))
        })?;
        if !uri.matches_display(reference)
            || !matches!(
                uri.scheme(),
                ResourceScheme::Res
                    | ResourceScheme::Library
                    | ResourceScheme::Package
                    | ResourceScheme::Builtin
            )
        {
            return Err(AssetImportError::Parse(format!(
                ".zui {kind} dependency must be a canonical persisted asset URI: `{reference}`"
            )));
        }
        let dependency = match uri.label() {
            Some(_) => {
                AssetUri::new(uri.scheme(), uri.path().to_owned(), None).map_err(|source| {
                    AssetImportError::Parse(format!(
                        "invalid canonical .zui {kind} dependency `{reference}`: {source}"
                    ))
                })?
            }
            None => uri,
        };
        if seen.insert(dependency.clone()) {
            dependencies.push(dependency);
        }
        Ok(())
    };

    for reference in &document.imports.widgets {
        push("widget import", reference)?;
    }
    for reference in &document.imports.styles {
        push("style import", reference)?;
    }
    for reference in &document.imports.resources {
        push("resource import", &reference.uri)?;
        if let Some(fallback) = reference.fallback.uri.as_deref() {
            push("resource fallback", fallback)?;
        }
    }

    Ok(dependencies)
}

#[cfg(test)]
#[path = "tests/lib.rs"]
mod tests;
