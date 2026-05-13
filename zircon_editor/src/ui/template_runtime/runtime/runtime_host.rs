use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, OnceLock};

use crate::ui::binding::EditorUiBinding;
use crate::ui::control::EditorUiControlService;
use crate::ui::layouts::windows::workbench_host_window::PaneBodyPresentation;
use crate::ui::template::{
    EditorComponentCatalog, EditorComponentDescriptor, EditorTemplateAdapter, EditorTemplateError,
    EditorTemplateRegistry, EditorTemplateRuntimeService,
};
use thiserror::Error;
use zircon_runtime::ui::surface::UiSurface;
use zircon_runtime::ui::template::UiTemplateBuildError;
use zircon_runtime::ui::v2::{
    UiV2CompiledDocument, UiV2PrototypeStoreFileCache, UiV2SurfaceBuilder,
};
use zircon_runtime_interface::ui::{
    event_ui::UiTreeId,
    template::{UiAssetDocument, UiAssetError},
    v2::{UiV2AssetDocument, UiV2AssetError},
};

use crate::ui::template_runtime::{
    RetainedUiHostAdapter, RetainedUiHostModel, RetainedUiHostProjection, RetainedUiProjection,
    UiComponentShowcaseDemoError, UiComponentShowcaseDemoEventInput, UiComponentShowcaseDemoState,
};
use zircon_runtime_interface::ui::component::UiComponentAdapterResult;

use super::{
    build_session::{
        compile_template_document_file, compile_template_document_with_builtin_imports,
        load_builtin_host_templates,
    },
    pane_payload_projection::project_pane_body,
    projection::{
        build_host_model, build_host_model_with_surface, project_document, project_v2_document,
    },
};

#[derive(Debug, Error, PartialEq)]
pub enum EditorUiHostRuntimeError {
    #[error(transparent)]
    Template(#[from] EditorTemplateError),
    #[error(transparent)]
    UiAsset(#[from] UiAssetError),
    #[error(transparent)]
    UiV2Asset(#[from] UiV2AssetError),
    #[error(transparent)]
    UiTemplateBuild(#[from] UiTemplateBuildError),
    #[error("retained host projection is missing binding {binding_id}")]
    MissingProjectionBinding { binding_id: String },
    #[error("shared surface node {node_path} is missing template metadata")]
    MissingSurfaceMetadata { node_path: String },
}

#[derive(Default)]
pub struct EditorUiHostRuntime {
    pub(super) component_catalog: EditorComponentCatalog,
    pub(super) template_registry: EditorTemplateRegistry,
    pub(super) template_adapter: EditorTemplateAdapter,
    pub(super) template_service: EditorTemplateRuntimeService,
    pub(super) v2_documents: BTreeMap<String, EditorUiHostV2Document>,
    pub(super) builtin_host_templates_loaded: bool,
    showcase_demo_state: UiComponentShowcaseDemoState,
}

#[derive(Clone, Debug)]
pub(super) struct EditorUiHostV2Document {
    pub(super) document: Arc<UiV2AssetDocument>,
    pub(super) compiled: Arc<UiV2CompiledDocument>,
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
        let compiled =
            compile_template_document_with_builtin_imports(&self.template_service, &document)?;
        self.template_service
            .register_compiled_document(&mut self.template_registry, document_id, compiled)
            .map_err(EditorUiHostRuntimeError::from)
    }

    pub fn register_document_source(
        &mut self,
        document_id: impl Into<String>,
        source: &str,
    ) -> Result<(), EditorUiHostRuntimeError> {
        let document_id = document_id.into();
        let document = self.template_service.parse_document_source(source)?;
        self.register_asset_document(document_id, document)
    }

    pub fn register_document_file(
        &mut self,
        document_id: impl Into<String>,
        path: impl AsRef<std::path::Path>,
    ) -> Result<(), EditorUiHostRuntimeError> {
        let document_id = document_id.into();
        if is_v2_document_path(path.as_ref()) {
            return self.register_v2_document_file(document_id, path);
        }
        let compiled = compile_template_document_file(&self.template_service, path.as_ref())?;
        self.template_service
            .register_compiled_document(&mut self.template_registry, document_id, compiled)
            .map_err(EditorUiHostRuntimeError::from)
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

    #[cfg(test)]
    pub(crate) fn showcase_demo_state(&self) -> &UiComponentShowcaseDemoState {
        &self.showcase_demo_state
    }

    pub(crate) fn apply_showcase_demo_binding(
        &mut self,
        binding: &EditorUiBinding,
        input: UiComponentShowcaseDemoEventInput,
    ) -> Result<UiComponentAdapterResult, UiComponentShowcaseDemoError> {
        crate::ui::template_runtime::component_adapter::showcase::apply_showcase_component_binding(
            &mut self.showcase_demo_state,
            binding,
            input,
        )
    }

    pub(crate) fn showcase_demo_value_i64(&self, control_id: &str, property: &str) -> Option<i64> {
        self.showcase_demo_state.value_i64(control_id, property)
    }

    pub fn project_document(
        &self,
        document_id: &str,
    ) -> Result<RetainedUiProjection, EditorUiHostRuntimeError> {
        if let Some(document) = self.v2_documents.get(document_id) {
            return project_v2_document(
                document_id,
                document.compiled.as_ref(),
                &self.template_adapter,
            );
        }
        project_document(
            &self.template_service,
            &self.template_registry,
            &self.template_adapter,
            document_id,
        )
    }

    pub(crate) fn project_pane_body(
        &self,
        body: &PaneBodyPresentation,
    ) -> Result<RetainedUiProjection, EditorUiHostRuntimeError> {
        if let Some(document) = self.v2_documents.get(&body.document_id) {
            let mut projection = project_v2_document(
                &body.document_id,
                document.compiled.as_ref(),
                &self.template_adapter,
            )?;
            super::pane_payload_projection::inject_pane_projection_attributes(
                &mut projection.root,
                body,
            );
            super::pane_payload_projection::append_hybrid_slot_anchor_projection(
                &mut projection.root,
                body,
            );
            return Ok(projection);
        }
        project_pane_body(
            &self.template_service,
            &self.template_registry,
            &self.template_adapter,
            body,
        )
    }

    pub fn register_projection_routes(
        &self,
        service: &mut EditorUiControlService,
        projection: &mut RetainedUiProjection,
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
        projection: &RetainedUiProjection,
    ) -> Result<RetainedUiHostModel, EditorUiHostRuntimeError> {
        let mut host_model = build_host_model(projection)?;
        self.showcase_demo_state
            .apply_to_host_model(&mut host_model);
        Ok(host_model)
    }

    pub fn build_host_model_with_surface(
        &self,
        projection: &RetainedUiProjection,
        surface: &UiSurface,
    ) -> Result<RetainedUiHostModel, EditorUiHostRuntimeError> {
        let mut host_model = build_host_model_with_surface(projection, surface)?;
        self.showcase_demo_state
            .apply_to_host_model(&mut host_model);
        Ok(host_model)
    }

    pub fn build_shared_surface(
        &self,
        document_id: &str,
    ) -> Result<UiSurface, EditorUiHostRuntimeError> {
        if let Some(document) = self.v2_documents.get(document_id) {
            return UiV2SurfaceBuilder::build_surface_from_compiled_document(
                UiTreeId::new(format!("template.v2.{document_id}")),
                document.document.as_ref(),
                document.compiled.as_ref(),
            )
            .map_err(EditorUiHostRuntimeError::from);
        }
        let instance = self
            .template_service
            .instantiate(&self.template_registry, document_id)
            .map_err(EditorUiHostRuntimeError::from)?;
        self.template_service
            .build_surface(UiTreeId::new(format!("template.{document_id}")), &instance)
            .map_err(EditorUiHostRuntimeError::from)
    }

    pub fn build_retained_host_projection(
        &self,
        projection: &RetainedUiProjection,
    ) -> Result<RetainedUiHostProjection, EditorUiHostRuntimeError> {
        let host_model = self.build_host_model(projection)?;
        Ok(RetainedUiHostAdapter::build_projection(&host_model))
    }

    pub fn build_retained_host_projection_with_surface(
        &self,
        projection: &RetainedUiProjection,
        surface: &UiSurface,
    ) -> Result<RetainedUiHostProjection, EditorUiHostRuntimeError> {
        let host_model = self.build_host_model_with_surface(projection, surface)?;
        Ok(RetainedUiHostAdapter::build_projection(&host_model))
    }
}

impl EditorUiHostRuntime {
    fn register_v2_document_file(
        &mut self,
        document_id: impl Into<String>,
        path: impl AsRef<std::path::Path>,
    ) -> Result<(), EditorUiHostRuntimeError> {
        let document_id = document_id.into();
        let outcome = v2_template_file_cache()
            .lock()
            .expect("v2 template file cache mutex should not be poisoned")
            .load_store(std::iter::once(path.as_ref().to_path_buf()))?;
        self.v2_documents.insert(
            document_id,
            EditorUiHostV2Document {
                document: outcome.root_document,
                compiled: outcome.compiled,
            },
        );
        Ok(())
    }
}

fn v2_template_file_cache() -> &'static Mutex<UiV2PrototypeStoreFileCache> {
    static CACHE: OnceLock<Mutex<UiV2PrototypeStoreFileCache>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(UiV2PrototypeStoreFileCache::new()))
}

#[cfg(test)]
pub(super) fn clear_v2_template_file_cache_for_tests() {
    v2_template_file_cache()
        .lock()
        .expect("v2 template file cache mutex should not be poisoned")
        .clear();
}

#[cfg(test)]
pub(super) fn v2_template_file_cache_len_for_tests() -> usize {
    v2_template_file_cache()
        .lock()
        .expect("v2 template file cache mutex should not be poisoned")
        .len()
}

fn is_v2_document_path(path: &std::path::Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.ends_with(".v2.ui.toml"))
}
