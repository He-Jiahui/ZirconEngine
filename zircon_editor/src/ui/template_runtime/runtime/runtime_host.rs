use std::fs;

use crate::ui::{
    EditorComponentCatalog, EditorComponentDescriptor, EditorTemplateAdapter, EditorTemplateError,
    EditorTemplateRegistry, EditorUiBinding, EditorUiControlService,
};
use thiserror::Error;
use zircon_runtime::ui::template::{
    UiAssetError, UiAssetLoader, UiTemplateBuildError, UiTemplateError, UiTemplateLoader,
    UiTemplateSurfaceBuilder,
};
use zircon_runtime::ui::{event_ui::UiTreeId, surface::UiSurface, template::UiAssetDocument};

use crate::ui::template_runtime::{
    SlintUiHostAdapter, SlintUiHostModel, SlintUiHostProjection, SlintUiProjection,
};

use super::{
    build_session::load_builtin_host_templates,
    projection::{build_host_model, build_host_model_with_surface, project_document},
};

#[derive(Debug, Error, PartialEq)]
pub enum EditorUiHostRuntimeError {
    #[error(transparent)]
    Template(#[from] EditorTemplateError),
    #[error(transparent)]
    UiAsset(#[from] UiAssetError),
    #[error(transparent)]
    UiTemplate(#[from] UiTemplateError),
    #[error(transparent)]
    UiTemplateBuild(#[from] UiTemplateBuildError),
    #[error("slint projection is missing binding {binding_id}")]
    MissingProjectionBinding { binding_id: String },
    #[error("shared surface node {node_path} is missing template metadata")]
    MissingSurfaceMetadata { node_path: String },
}

#[derive(Default)]
pub struct EditorUiHostRuntime {
    pub(super) component_catalog: EditorComponentCatalog,
    pub(super) template_registry: EditorTemplateRegistry,
    pub(super) template_adapter: EditorTemplateAdapter,
    pub(super) builtin_host_templates_loaded: bool,
}

impl EditorUiHostRuntime {
    pub fn register_component(
        &mut self,
        descriptor: EditorComponentDescriptor,
    ) -> Result<(), EditorUiHostRuntimeError> {
        self.component_catalog
            .register(descriptor)
            .map_err(EditorUiHostRuntimeError::from)
    }

    pub fn component_descriptor(&self, component_id: &str) -> Option<&EditorComponentDescriptor> {
        self.component_catalog.descriptor(component_id)
    }

    pub fn register_template_document_file(
        &mut self,
        document_id: impl Into<String>,
        path: impl AsRef<std::path::Path>,
    ) -> Result<(), EditorUiHostRuntimeError> {
        self.register_document_file(document_id, path)
    }

    pub fn register_asset_document(
        &mut self,
        document_id: impl Into<String>,
        document: UiAssetDocument,
    ) -> Result<(), EditorUiHostRuntimeError> {
        self.template_registry
            .register_asset_document(document_id, document)
            .map_err(EditorUiHostRuntimeError::from)
    }

    pub fn register_document_source(
        &mut self,
        document_id: impl Into<String>,
        source: &str,
    ) -> Result<(), EditorUiHostRuntimeError> {
        let document_id = document_id.into();
        if let Ok(document) = UiAssetLoader::load_toml_str(source) {
            return self.register_asset_document(document_id, document);
        }

        let document = UiTemplateLoader::load_toml_str(source)?;
        self.template_registry
            .register_document(document_id, document)
            .map_err(EditorUiHostRuntimeError::from)
    }

    pub fn register_document_file(
        &mut self,
        document_id: impl Into<String>,
        path: impl AsRef<std::path::Path>,
    ) -> Result<(), EditorUiHostRuntimeError> {
        let source =
            fs::read_to_string(path).map_err(|error| UiAssetError::Io(error.to_string()))?;
        self.register_document_source(document_id, &source)
    }

    pub fn register_binding(
        &mut self,
        binding_id: impl Into<String>,
        binding: EditorUiBinding,
    ) -> Result<(), EditorUiHostRuntimeError> {
        self.template_adapter
            .register_binding(binding_id, binding)
            .map_err(EditorUiHostRuntimeError::from)
    }

    pub fn load_builtin_host_templates(&mut self) -> Result<(), EditorUiHostRuntimeError> {
        load_builtin_host_templates(self)
    }

    pub fn load_builtin_workbench_shell(&mut self) -> Result<(), EditorUiHostRuntimeError> {
        self.load_builtin_host_templates()
    }

    pub fn project_document(
        &self,
        document_id: &str,
    ) -> Result<SlintUiProjection, EditorUiHostRuntimeError> {
        project_document(&self.template_registry, &self.template_adapter, document_id)
    }

    pub fn register_projection_routes(
        &self,
        service: &mut EditorUiControlService,
        projection: &mut SlintUiProjection,
    ) -> Result<(), EditorUiHostRuntimeError> {
        for binding in &mut projection.bindings {
            let route_id = service
                .route_id_for_binding(&binding.binding.as_ui_binding())
                .unwrap_or_else(|| service.register_route_stub(binding.binding.as_ui_binding()));
            binding.route_id = Some(route_id);
        }
        Ok(())
    }

    pub fn build_host_model(
        &self,
        projection: &SlintUiProjection,
    ) -> Result<SlintUiHostModel, EditorUiHostRuntimeError> {
        build_host_model(projection)
    }

    pub fn build_host_model_with_surface(
        &self,
        projection: &SlintUiProjection,
        surface: &UiSurface,
    ) -> Result<SlintUiHostModel, EditorUiHostRuntimeError> {
        build_host_model_with_surface(projection, surface)
    }

    pub fn build_shared_surface(
        &self,
        document_id: &str,
    ) -> Result<UiSurface, EditorUiHostRuntimeError> {
        let instance = self
            .template_registry
            .instantiate(document_id)
            .map_err(EditorUiHostRuntimeError::from)?;
        UiTemplateSurfaceBuilder::build_surface(
            UiTreeId::new(format!("template.{document_id}")),
            &instance,
        )
        .map_err(EditorUiHostRuntimeError::from)
    }

    pub fn build_slint_host_projection(
        &self,
        projection: &SlintUiProjection,
    ) -> Result<SlintUiHostProjection, EditorUiHostRuntimeError> {
        let host_model = self.build_host_model(projection)?;
        Ok(SlintUiHostAdapter::build_projection(&host_model))
    }

    pub fn build_slint_host_projection_with_surface(
        &self,
        projection: &SlintUiProjection,
        surface: &UiSurface,
    ) -> Result<SlintUiHostProjection, EditorUiHostRuntimeError> {
        let host_model = self.build_host_model_with_surface(projection, surface)?;
        Ok(SlintUiHostAdapter::build_projection(&host_model))
    }
}
