#[path = "collect.rs"]
mod collect;
#[path = "documents.rs"]
mod documents;
#[path = "generation.rs"]
mod generation;
#[path = "parsed_document.rs"]
mod parsed_document;
#[path = "resolution.rs"]
mod resolution;
#[path = "traversal.rs"]
mod traversal;

pub(in crate::ui::host::asset_editor_sessions) use collect::collect_ui_asset_import_document;
pub(super) use documents::UiAssetImportDocuments;
pub(super) use generation::UiAssetImportGeneration;
pub(super) use resolution::UiAssetImportResolution;
pub(super) use traversal::UiAssetImportTraversal;

#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;
