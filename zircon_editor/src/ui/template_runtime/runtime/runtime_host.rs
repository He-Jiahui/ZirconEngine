use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, Mutex, OnceLock};

use crate::ui::binding::EditorUiBinding;
use crate::ui::control::EditorUiControlService;
use crate::ui::template::{
    EditorComponentCatalog, EditorComponentCatalogManifestError, EditorComponentDescriptor,
    EditorTemplateAdapter, EditorTemplateError, EditorTemplateRegistry,
    EditorTemplateRuntimeService,
};
use thiserror::Error;
use zircon_runtime::ui::surface::UiSurface;
use zircon_runtime::ui::template::{UiTemplateBuildError, UiTemplateInstance};
use zircon_runtime::ui::theme::UiThemeRegistry;
use zircon_runtime::ui::v2::{
    UiV2CompiledDocument, UiV2PrototypeStoreFileCache, UiV2SourceFileReceipt, UiV2SurfaceBuilder,
    UiV2UnresolvedSourceImport,
};
use zircon_runtime_interface::ui::{
    component::UiComponentAdapterResult,
    event_ui::{UiNodeId, UiTreeId},
    template::{UiAssetDocument, UiAssetError},
    tree::UiTreeError,
    v2::{UiV2AssetDocument, UiV2AssetError},
};

use crate::ui::template_runtime::{
    RetainedUiHostAdapter, RetainedUiHostModel, RetainedUiHostNodeModel, RetainedUiHostProjection,
    RetainedUiProjection, UiComponentShowcaseDemoError, UiComponentShowcaseDemoEventInput,
    UiComponentShowcaseDemoState,
};
use crate::ui::v2_design_tokens::prepare_editor_v2_document;

use super::{
    build_session::{
        compile_template_document_file, compile_template_document_with_builtin_imports,
        load_builtin_host_templates, load_builtin_host_templates_for_document_ids,
    },
    plugin_documents::{EditorPluginV2DocumentSourceError, EditorUiHostPluginV2Document},
    projection::{
        build_host_model, build_host_model_with_surface, build_host_nodes_with_surface,
        project_instance, project_v2_document,
    },
    template_action_registry::TemplateActionRegistry,
};

mod dynamic_control_state;

#[derive(Debug, Error, PartialEq)]
pub enum EditorUiHostRuntimeError {
    #[error(transparent)]
    ComponentCatalog(#[from] EditorComponentCatalogManifestError),
    #[error(transparent)]
    Template(#[from] EditorTemplateError),
    #[error(transparent)]
    UiAsset(#[from] UiAssetError),
    #[error(transparent)]
    UiV2Asset(#[from] UiV2AssetError),
    #[error(transparent)]
    UiTemplateBuild(#[from] UiTemplateBuildError),
    #[error(transparent)]
    UiTree(#[from] UiTreeError),
    #[error(transparent)]
    PluginDocumentSource(#[from] EditorPluginV2DocumentSourceError),
    #[error("retained host projection is missing binding {binding_id}")]
    MissingProjectionBinding { binding_id: String },
    #[error("shared surface node {node_path} is missing template metadata")]
    MissingSurfaceMetadata { node_path: String },
    #[error("template document {document_id} has no native control {control_id}")]
    MissingTemplateSurfaceControl {
        document_id: String,
        control_id: String,
    },
    #[error("template document {document_id} has duplicate native control {control_id}")]
    DuplicateTemplateSurfaceControl {
        document_id: String,
        control_id: String,
    },
    #[error("template document {document_id} has duplicate retained control {control_id}")]
    DuplicateRetainedControl {
        document_id: String,
        control_id: String,
    },
    #[error("template document {document_id} has no retained control {control_id}")]
    MissingRetainedControl {
        document_id: String,
        control_id: String,
    },
    #[error(
        "template document {document_id} rejected dynamic property {property} on control {control_id}: {detail}"
    )]
    TemplateControlStateRejected {
        document_id: String,
        control_id: String,
        property: String,
        detail: String,
    },
    #[error("plugin V2 document URI {source_uri} is not owned by {owner_id}")]
    PluginDocumentUri {
        owner_id: String,
        source_uri: String,
    },
    #[error("plugin V2 template URI {source_uri} has an invalid relative path")]
    PluginDocumentTemplatePath { source_uri: String },
    #[error("plugin {owner_id} template {template_id} has no host-resolved package root")]
    PluginDocumentTemplateRoot {
        owner_id: String,
        template_id: String,
    },
    #[error("plugin V2 document id {document_id} is already owned by {owner_id}")]
    PluginDocumentIdConflict {
        document_id: String,
        owner_id: String,
    },
    #[error(
        "plugin V2 document generation {requested_generation} for {owner_id} is not newer than current generation {current_generation}"
    )]
    PluginDocumentGenerationStale {
        owner_id: String,
        requested_generation: u64,
        current_generation: u64,
    },
}

#[derive(Default)]
pub struct EditorUiHostRuntime {
    pub(super) component_catalog: EditorComponentCatalog,
    pub(super) template_registry: EditorTemplateRegistry,
    pub(super) template_adapter: EditorTemplateAdapter,
    pub(super) template_service: EditorTemplateRuntimeService,
    pub(super) v2_documents: BTreeMap<String, EditorUiHostV2Document>,
    v2_source_receipts: BTreeMap<String, Vec<UiV2SourceFileReceipt>>,
    v2_source_unresolved_imports: BTreeMap<String, Vec<UiV2UnresolvedSourceImport>>,
    pub(super) plugin_v2_documents: Mutex<BTreeMap<String, EditorUiHostPluginV2Document>>,
    pub(super) plugin_v2_generations: Mutex<BTreeMap<String, u64>>,
    template_action_registry: Mutex<TemplateActionRegistry>,
    pub(super) active_theme: UiThemeRegistry,
    pub(super) builtin_host_templates_loaded: bool,
    showcase_demo_state: UiComponentShowcaseDemoState,
    projection_cache: Mutex<BTreeMap<String, RetainedUiProjection>>,
    template_instance_cache: Mutex<BTreeMap<String, Arc<UiTemplateInstance>>>,
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
            .map_err(EditorUiHostRuntimeError::from)?;
        self.invalidate_projection_cache();
        Ok(())
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
        let document_id = document_id.into();
        self.ensure_document_id_is_not_plugin_owned(&document_id)?;
        let compiled =
            compile_template_document_with_builtin_imports(&self.template_service, &document)?;
        self.template_service
            .register_compiled_document(&mut self.template_registry, document_id, compiled)
            .map_err(EditorUiHostRuntimeError::from)?;
        self.invalidate_projection_cache();
        Ok(())
    }

    pub fn register_v2_template_document_files<P, I>(
        &mut self,
        document_id: impl Into<String>,
        paths: I,
    ) -> Result<(), EditorUiHostRuntimeError>
    where
        P: AsRef<std::path::Path>,
        I: IntoIterator<Item = P>,
    {
        self.register_v2_document_files(document_id, paths)
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
        self.ensure_document_id_is_not_plugin_owned(&document_id)?;
        if is_v2_backed_document_path(path.as_ref()) {
            return self.register_v2_document_file(document_id, path);
        }
        let compiled = compile_template_document_file(&self.template_service, path.as_ref())?;
        self.template_service
            .register_compiled_document(&mut self.template_registry, document_id, compiled)
            .map_err(EditorUiHostRuntimeError::from)?;
        self.invalidate_projection_cache();
        Ok(())
    }

    pub fn register_binding(
        &mut self,
        binding_id: impl Into<String>,
        binding: EditorUiBinding,
    ) -> Result<(), EditorUiHostRuntimeError> {
        self.template_adapter
            .register_binding(binding_id, binding)
            .map_err(EditorUiHostRuntimeError::from)?;
        self.invalidate_projection_cache();
        Ok(())
    }

    /// Reconcile the built-in binding registry after a review host has been
    /// assembled.  Review projections can be created from a fresh V2 file
    /// store while the host document cache is warm; keeping this operation
    /// idempotent guarantees that authored product events remain resolvable.
    pub(crate) fn ensure_builtin_host_bindings(&mut self) -> Result<(), EditorUiHostRuntimeError> {
        for (binding_id, binding) in
            crate::ui::template_runtime::builtin::builtin_template_bindings()
        {
            if !self.template_adapter.contains_binding(binding_id) {
                self.register_binding(binding_id.as_str(), binding.clone())?;
            }
        }
        Ok(())
    }

    pub fn load_builtin_host_templates(&mut self) -> Result<(), EditorUiHostRuntimeError> {
        load_builtin_host_templates(self)
    }

    pub(crate) fn load_builtin_host_templates_for_document_ids(
        &mut self,
        document_ids: &[&str],
    ) -> Result<(), EditorUiHostRuntimeError> {
        load_builtin_host_templates_for_document_ids(self, document_ids)
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
        self.project_document_cached(document_id)
    }

    pub(crate) fn project_document_cached(
        &self,
        document_id: &str,
    ) -> Result<RetainedUiProjection, EditorUiHostRuntimeError> {
        if let Some(projection) = self
            .projection_cache
            .lock()
            .expect("template projection cache mutex should not be poisoned")
            .get(document_id)
            .cloned()
        {
            zircon_runtime::profile_counter!("editor", "ui.template_projection.cache_hit_count", 1);
            return Ok(projection);
        }

        zircon_runtime::profile_counter!("editor", "ui.template_projection.cache_miss_count", 1);
        let projection = self.project_document_uncached(document_id)?;
        self.projection_cache
            .lock()
            .expect("template projection cache mutex should not be poisoned")
            .insert(document_id.to_string(), projection.clone());
        Ok(projection)
    }

    fn project_document_uncached(
        &self,
        document_id: &str,
    ) -> Result<RetainedUiProjection, EditorUiHostRuntimeError> {
        if let Some(document) = self.v2_document(document_id) {
            return project_v2_document(
                document_id,
                document.compiled.as_ref(),
                &self.template_adapter,
            );
        }
        let instance = self.template_instance_cached(document_id)?;
        project_instance(document_id, instance.as_ref(), &self.template_adapter)
    }

    fn template_instance_cached(
        &self,
        document_id: &str,
    ) -> Result<Arc<UiTemplateInstance>, EditorUiHostRuntimeError> {
        if let Some(instance) = self
            .template_instance_cache
            .lock()
            .expect("template instance cache mutex should not be poisoned")
            .get(document_id)
            .cloned()
        {
            zircon_runtime::profile_counter!(
                "editor",
                "ui.template_projection.instance_cache_hit_count",
                1
            );
            return Ok(instance);
        }

        zircon_runtime::profile_counter!(
            "editor",
            "ui.template_projection.instance_cache_miss_count",
            1
        );
        let instance = Arc::new(
            self.template_service
                .instantiate(&self.template_registry, document_id)
                .map_err(EditorUiHostRuntimeError::from)?,
        );
        self.template_instance_cache
            .lock()
            .expect("template instance cache mutex should not be poisoned")
            .insert(document_id.to_string(), Arc::clone(&instance));
        Ok(instance)
    }

    pub(crate) fn invalidate_projection_cache(&self) {
        self.projection_cache
            .lock()
            .expect("template projection cache mutex should not be poisoned")
            .clear();
        self.template_instance_cache
            .lock()
            .expect("template instance cache mutex should not be poisoned")
            .clear();
    }

    pub fn register_projection_routes(
        &self,
        service: &mut EditorUiControlService,
        projection: &mut RetainedUiProjection,
    ) -> Result<(), EditorUiHostRuntimeError> {
        for binding in &mut projection.bindings {
            let route_id = service
                .route_id_for_binding(&binding.binding.as_ui_binding())
                .unwrap_or_else(|| service.register_binding_route(binding.binding.as_ui_binding()));
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
        if let Some(document) = self.v2_document(document_id) {
            let prepared_document = prepare_editor_v2_document(document.document.as_ref());
            return UiV2SurfaceBuilder::build_surface_from_compiled_document_with_theme(
                UiTreeId::new(format!("template.v2.{document_id}")),
                &prepared_document,
                document.compiled.as_ref(),
                &self.active_theme,
            )
            .map_err(EditorUiHostRuntimeError::from);
        }
        let instance = self.template_instance_cached(document_id)?;
        self.template_service
            .build_surface(
                UiTreeId::new(format!("template.{document_id}")),
                instance.as_ref(),
            )
            .map_err(EditorUiHostRuntimeError::from)
    }

    pub(crate) fn retained_document_identity(&self, document_id: &str) -> Option<usize> {
        self.v2_document(document_id)
            .map(|document| Arc::as_ptr(&document.compiled) as usize)
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
        let mut projection = RetainedUiHostAdapter::build_projection(&host_model);
        let frame = surface.surface_frame();
        for node in &mut projection.nodes {
            node.source_surface_frame = Some(Arc::clone(&frame));
        }
        projection.source_surface_frame = Some(frame);
        Ok(projection)
    }

    pub(crate) fn build_retained_host_nodes_with_surface(
        &self,
        projection: &RetainedUiProjection,
        surface: &UiSurface,
        node_ids: &std::collections::BTreeSet<UiNodeId>,
        metadata_index: &crate::ui::template_runtime::RetainedUiProjectionSurfaceMetadataIndex,
    ) -> Result<Vec<(UiNodeId, RetainedUiHostNodeModel)>, EditorUiHostRuntimeError> {
        let raw_nodes =
            build_host_nodes_with_surface(projection, surface, node_ids, metadata_index)?;
        let mut node_ids_by_path = raw_nodes
            .iter()
            .map(|(node_id, node)| (node.node_id.clone(), *node_id))
            .collect::<BTreeMap<_, _>>();
        let mut host_model = RetainedUiHostModel {
            document_id: projection.document_id.clone(),
            nodes: raw_nodes.into_iter().map(|(_, node)| node).collect(),
        };
        self.showcase_demo_state
            .apply_to_host_model(&mut host_model);
        Ok(host_model
            .nodes
            .iter()
            .filter_map(|node| {
                node_ids_by_path.remove(&node.node_id).map(|node_id| {
                    let mut projected = RetainedUiHostAdapter::build_node(node);
                    projected.source_surface_frame = Some(surface.surface_frame());
                    (node_id, projected)
                })
            })
            .collect())
    }
}

impl EditorUiHostRuntime {
    pub(crate) fn loaded_v2_source_receipts_for_documents(
        &self,
        document_ids: &[&str],
    ) -> Result<Vec<UiV2SourceFileReceipt>, String> {
        let mut receipts_by_path = BTreeMap::<std::path::PathBuf, UiV2SourceFileReceipt>::new();
        for document_id in document_ids {
            if !self.v2_documents.contains_key(*document_id) {
                return Err(format!(
                    "V2 source receipt requested for unloaded document {document_id}"
                ));
            }
            let document_receipts = self
                .v2_source_receipts
                .get(*document_id)
                .ok_or_else(|| format!("V2 source receipts missing for document {document_id}"))?;
            if document_receipts.is_empty() {
                return Err(format!(
                    "V2 source receipt closure is empty for document {document_id}"
                ));
            }
            for receipt in document_receipts {
                match receipts_by_path.get(&receipt.physical_path) {
                    Some(existing) if existing != receipt => {
                        return Err(format!(
                            "conflicting V2 source receipts for {}",
                            receipt.physical_path.display()
                        ));
                    }
                    Some(_) => {}
                    None => {
                        receipts_by_path.insert(receipt.physical_path.clone(), receipt.clone());
                    }
                }
            }
        }
        Ok(receipts_by_path.into_values().collect())
    }

    pub(crate) fn loaded_v2_unresolved_imports_for_documents(
        &self,
        document_ids: &[&str],
    ) -> Result<Vec<UiV2UnresolvedSourceImport>, String> {
        let mut unresolved = BTreeSet::new();
        for document_id in document_ids {
            if !self.v2_documents.contains_key(*document_id) {
                return Err(format!(
                    "unresolved-import audit requested for unloaded document {document_id}"
                ));
            }
            let document_imports = self
                .v2_source_unresolved_imports
                .get(*document_id)
                .ok_or_else(|| {
                    format!("V2 unresolved-import audit missing for document {document_id}")
                })?;
            unresolved.extend(document_imports.iter().cloned());
        }
        Ok(unresolved.into_iter().collect())
    }

    fn register_v2_document_file(
        &mut self,
        document_id: impl Into<String>,
        path: impl AsRef<std::path::Path>,
    ) -> Result<(), EditorUiHostRuntimeError> {
        self.register_v2_document_files(document_id, std::iter::once(path))
    }

    fn register_v2_document_files<P, I>(
        &mut self,
        document_id: impl Into<String>,
        paths: I,
    ) -> Result<(), EditorUiHostRuntimeError>
    where
        P: AsRef<std::path::Path>,
        I: IntoIterator<Item = P>,
    {
        let document_id = document_id.into();
        self.ensure_document_id_is_not_plugin_owned(&document_id)?;
        let outcome = v2_template_file_cache()
            .lock()
            .expect("v2 template file cache mutex should not be poisoned")
            .load_store(paths)?;
        self.v2_source_receipts
            .insert(document_id.clone(), outcome.source_receipts.clone());
        self.v2_source_unresolved_imports
            .insert(document_id.clone(), outcome.unresolved_imports.clone());
        self.v2_documents.insert(
            document_id,
            EditorUiHostV2Document {
                document: outcome.root_document,
                compiled: outcome.compiled,
            },
        );
        self.invalidate_projection_cache();
        Ok(())
    }

    pub(super) fn remove_template_actions_for_documents(&self, document_ids: &[String]) {
        let mut registry = self
            .template_action_registry
            .lock()
            .expect("template action registry mutex should not be poisoned");
        for document_id in document_ids {
            registry.remove_document(document_id);
        }
    }
}

pub(super) fn v2_template_file_cache() -> &'static Mutex<UiV2PrototypeStoreFileCache> {
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

fn is_v2_backed_document_path(path: &std::path::Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.ends_with(".zui"))
}

#[cfg(test)]
#[path = "tests/runtime_host_pane_control_state_tests.rs"]
mod pane_control_state_tests;
