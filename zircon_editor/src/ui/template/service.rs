use std::collections::BTreeMap;
use std::path::Path;

use zircon_runtime::ui::surface::{extract_ui_render_tree, UiSurface};
use zircon_runtime::ui::template::{
    collect_asset_binding_report, UiAssetLoader, UiCompiledDocument, UiDocumentCompiler,
    UiTemplateBuildError, UiTemplateInstance, UiTemplateSurfaceBuilder,
};
use zircon_runtime_interface::ui::event_ui::UiTreeId;
use zircon_runtime_interface::ui::surface::UiRenderExtract;
use zircon_runtime_interface::ui::template::UiBindingReport;
use zircon_runtime_interface::ui::template::{UiAssetDocument, UiAssetError};

use crate::ui::template::{EditorTemplateError, EditorTemplateRegistry};

#[derive(Clone, Debug, Default)]
pub struct EditorTemplateRuntimeService;

impl EditorTemplateRuntimeService {
    pub fn parse_document_source(&self, source: &str) -> Result<UiAssetDocument, UiAssetError> {
        UiAssetLoader::load_toml_str(source).or_else(|error| {
            #[cfg(test)]
            {
                crate::tests::support::load_test_ui_asset(source).or(Err(error))
            }
            #[cfg(not(test))]
            {
                Err(error)
            }
        })
    }

    pub fn load_document_file(
        &self,
        path: impl AsRef<Path>,
    ) -> Result<UiAssetDocument, UiAssetError> {
        UiAssetLoader::load_toml_file(path)
    }

    pub fn compile_document(
        &self,
        document: &UiAssetDocument,
    ) -> Result<UiCompiledDocument, UiAssetError> {
        UiDocumentCompiler::default().compile(document)
    }

    pub fn collect_binding_report(&self, document: &UiAssetDocument) -> UiBindingReport {
        collect_asset_binding_report(document, UiDocumentCompiler::default().component_registry())
    }

    pub fn compile_document_with_import_maps(
        &self,
        document: &UiAssetDocument,
        widget_imports: &BTreeMap<String, UiAssetDocument>,
        style_imports: &BTreeMap<String, UiAssetDocument>,
    ) -> Result<UiCompiledDocument, UiAssetError> {
        let mut compiler = UiDocumentCompiler::default();
        for (reference, widget) in widget_imports {
            compiler.register_widget_import(reference.clone(), widget.clone())?;
        }
        for (reference, style) in style_imports {
            compiler.register_style_import(reference.clone(), style.clone())?;
        }
        compiler.compile(document)
    }

    pub fn register_asset_document(
        &self,
        registry: &mut EditorTemplateRegistry,
        document_id: impl Into<String>,
        document: UiAssetDocument,
    ) -> Result<(), EditorTemplateError> {
        let compiled = self.compile_document(&document)?;
        registry.register_compiled_document(document_id, compiled)
    }

    pub fn register_compiled_document(
        &self,
        registry: &mut EditorTemplateRegistry,
        document_id: impl Into<String>,
        document: UiCompiledDocument,
    ) -> Result<(), EditorTemplateError> {
        registry.register_compiled_document(document_id, document)
    }

    pub fn instantiate(
        &self,
        registry: &EditorTemplateRegistry,
        document_id: &str,
    ) -> Result<UiTemplateInstance, EditorTemplateError> {
        Ok(registry
            .compiled_document(document_id)?
            .clone()
            .into_template_instance())
    }

    pub fn build_surface(
        &self,
        tree_id: UiTreeId,
        instance: &UiTemplateInstance,
    ) -> Result<UiSurface, UiTemplateBuildError> {
        UiTemplateSurfaceBuilder::build_surface(tree_id, instance)
    }

    pub fn build_surface_from_compiled_document(
        &self,
        tree_id: UiTreeId,
        document: &UiCompiledDocument,
    ) -> Result<UiSurface, UiTemplateBuildError> {
        UiTemplateSurfaceBuilder::build_surface_from_compiled_document(tree_id, document)
    }

    pub fn extract_render(&self, surface: &UiSurface) -> UiRenderExtract {
        extract_ui_render_tree(&surface.tree)
    }
}
