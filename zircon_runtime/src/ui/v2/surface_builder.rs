use zircon_runtime_interface::ui::event_ui::UiTreeId;
use zircon_runtime_interface::ui::v2::{UiV2AssetError, UiV2CompiledDocument};

use crate::ui::surface::UiSurface;
use crate::ui::theme::UiThemeRegistry;

use super::cache::UiV2PrototypeStore;
use super::style::{UiV2RuntimeStyleIndex, UiV2StyleResolver};
use super::surface_tree::build_tree_from_arena;
use super::UiV2DocumentCompiler;

#[derive(Default)]
pub struct UiV2SurfaceBuilder;

impl UiV2SurfaceBuilder {
    pub fn build_surface(
        tree_id: UiTreeId,
        document: &zircon_runtime_interface::ui::v2::UiV2AssetDocument,
    ) -> Result<UiSurface, UiV2AssetError> {
        let compiled = UiV2DocumentCompiler::compile(document)?;
        Self::build_surface_from_compiled_document(tree_id, document, &compiled)
    }

    /// Builds a retained surface after expanding imported project or package components.
    ///
    /// Runtime session creation owns the prototype store and supplies every imported
    /// UI document once. Per-frame UI work consequently only rebuilds dirty surface
    /// state rather than reparsing or resolving component imports.
    pub fn build_surface_with_prototype_store(
        tree_id: UiTreeId,
        document: &zircon_runtime_interface::ui::v2::UiV2AssetDocument,
        store: &UiV2PrototypeStore,
    ) -> Result<UiSurface, UiV2AssetError> {
        let compiled = UiV2DocumentCompiler::compile_with_prototype_store(document, store)?;
        Self::build_surface_from_compiled_document(tree_id, document, &compiled)
    }

    pub fn build_surface_from_compiled_document(
        tree_id: UiTreeId,
        document: &zircon_runtime_interface::ui::v2::UiV2AssetDocument,
        compiled: &UiV2CompiledDocument,
    ) -> Result<UiSurface, UiV2AssetError> {
        Self::build_surface_from_compiled_document_with_optional_theme(
            tree_id, document, compiled, None,
        )
    }

    pub fn build_surface_from_compiled_document_with_theme(
        tree_id: UiTreeId,
        document: &zircon_runtime_interface::ui::v2::UiV2AssetDocument,
        compiled: &UiV2CompiledDocument,
        theme: &UiThemeRegistry,
    ) -> Result<UiSurface, UiV2AssetError> {
        Self::build_surface_from_compiled_document_with_optional_theme(
            tree_id,
            document,
            compiled,
            Some(theme),
        )
    }

    fn build_surface_from_compiled_document_with_optional_theme(
        tree_id: UiTreeId,
        document: &zircon_runtime_interface::ui::v2::UiV2AssetDocument,
        compiled: &UiV2CompiledDocument,
        theme: Option<&UiThemeRegistry>,
    ) -> Result<UiSurface, UiV2AssetError> {
        let resolved_styles = if let Some(theme) = theme {
            UiV2StyleResolver::resolve_static_with_theme(document, &compiled.arena, theme)?
        } else {
            UiV2StyleResolver::resolve_static(document, &compiled.arena)?
        };
        let tree = build_tree_from_arena(
            &compiled.asset_id,
            tree_id.clone(),
            document,
            &compiled.arena,
            &resolved_styles,
            theme,
        )?;
        let mut runtime_style = if let Some(theme) = theme {
            UiV2RuntimeStyleIndex::from_document_with_theme(document, theme)?
        } else {
            UiV2RuntimeStyleIndex::from_document(document)?
        };
        runtime_style.capture_baseline_from_tree(&tree);
        let mut surface = UiSurface::new(tree_id);
        surface.tree = tree;
        surface.set_runtime_style_index(runtime_style);
        surface.seed_component_states_from_tree_metadata();
        surface
            .apply_runtime_state_style_all(false)
            .map_err(|error| UiV2AssetError::InvalidDocument {
                asset_id: compiled.asset_id.clone(),
                detail: error.to_string(),
            })?;
        Ok(surface)
    }
}
