use crate::ui::asset_editor::UiAssetPreviewPreset;
use zircon_runtime::ui::template::{UiCompiledDocument, UiDocumentCompiler};
use zircon_runtime_interface::ui::{
    layout::UiSize,
    template::{UiAssetDocument, UiAssetKind},
};

use super::super::preview::preview_host::UiAssetPreviewHost;
use super::session_state::UiAssetCompilerImports;
use super::UiAssetEditorSessionError;

pub(super) fn compile_preview(
    document: &UiAssetDocument,
    preview_size: UiSize,
    imports: &UiAssetCompilerImports,
) -> Result<(Option<UiCompiledDocument>, Option<UiAssetPreviewHost>), UiAssetEditorSessionError> {
    if matches!(document.asset.kind, UiAssetKind::Style) {
        return Ok((None, None));
    }

    let mut compiler = UiDocumentCompiler::default();
    for (reference, widget) in &imports.widgets {
        compiler.register_widget_import(reference.clone(), widget.clone())?;
    }
    for (reference, style) in &imports.styles {
        compiler.register_style_import(reference.clone(), style.clone())?;
    }
    let compiled = compiler.compile(document)?;
    let preview_host = UiAssetPreviewHost::new(preview_size, &document.asset.id, &compiled)?;
    Ok((Some(compiled), Some(preview_host)))
}

pub(super) fn preview_size_for_preset(preview_preset: UiAssetPreviewPreset) -> UiSize {
    match preview_preset {
        UiAssetPreviewPreset::EditorDocked => UiSize::new(1280.0, 720.0),
        UiAssetPreviewPreset::EditorFloating => UiSize::new(1100.0, 780.0),
        UiAssetPreviewPreset::GameHud => UiSize::new(1920.0, 1080.0),
        UiAssetPreviewPreset::Dialog => UiSize::new(640.0, 480.0),
    }
}

pub(super) fn current_preview_size(
    current: &Option<UiAssetPreviewHost>,
    preview_preset: UiAssetPreviewPreset,
) -> UiSize {
    current
        .as_ref()
        .map(UiAssetPreviewHost::preview_size)
        .unwrap_or_else(|| preview_size_for_preset(preview_preset))
}
