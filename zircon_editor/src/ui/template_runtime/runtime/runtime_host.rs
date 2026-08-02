use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, OnceLock};

use crate::ui::binding::EditorUiBinding;
use crate::ui::control::EditorUiControlService;
use crate::ui::layouts::windows::workbench_host_window::PaneBodyPresentation;
use crate::ui::template::{
    EditorComponentCatalog, EditorComponentCatalogManifestError, EditorComponentDescriptor,
    EditorTemplateAdapter, EditorTemplateError, EditorTemplateRegistry,
    EditorTemplateRuntimeService,
};
use thiserror::Error;
use zircon_runtime::ui::surface::{UiPropertyMutationRequest, UiSurface};
use zircon_runtime::ui::template::UiTemplateBuildError;
use zircon_runtime::ui::theme::UiThemeRegistry;
use zircon_runtime::ui::v2::{
    UiV2CompiledDocument, UiV2PrototypeStoreFileCache, UiV2SurfaceBuilder,
};
use zircon_runtime_interface::ui::{
    component::{UiComponentAdapterResult, UiValue},
    dispatch::UiTemplateActionInvocation,
    event_ui::UiTreeId,
    template::{UiAssetDocument, UiAssetError},
    tree::UiTreeError,
    v2::{UiV2AssetDocument, UiV2AssetError},
};

use crate::ui::template_runtime::{
    RetainedUiHostAdapter, RetainedUiHostModel, RetainedUiHostProjection, RetainedUiNodeProjection,
    RetainedUiProjection, UiComponentShowcaseDemoError, UiComponentShowcaseDemoEventInput,
    UiComponentShowcaseDemoState,
};
use crate::ui::v2_design_tokens::prepare_editor_v2_document;

use super::{
    build_session::{
        compile_template_document_file, compile_template_document_with_builtin_imports,
        load_builtin_host_templates, load_builtin_host_templates_for_document_ids,
    },
    pane_payload_projection::{project_pane_body, template_v2_component_patch_attributes},
    plugin_documents::{EditorPluginV2DocumentSourceError, EditorUiHostPluginV2Document},
    projection::{
        build_host_model, build_host_model_with_surface, project_document, project_v2_document,
    },
    template_action_registry::TemplateActionRegistry,
};

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
    pub(super) plugin_v2_documents: Mutex<BTreeMap<String, EditorUiHostPluginV2Document>>,
    pub(super) plugin_v2_generations: Mutex<BTreeMap<String, u64>>,
    template_action_registry: Mutex<TemplateActionRegistry>,
    pub(super) active_theme: UiThemeRegistry,
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
        let document_id = document_id.into();
        self.ensure_document_id_is_not_plugin_owned(&document_id)?;
        let compiled =
            compile_template_document_with_builtin_imports(&self.template_service, &document)?;
        self.template_service
            .register_compiled_document(&mut self.template_registry, document_id, compiled)
            .map_err(EditorUiHostRuntimeError::from)
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
        if let Some(document) = self.v2_document(document_id) {
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
        if let Some(document) = self.v2_document(&body.document_id) {
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

    pub(crate) fn apply_pane_component_patches_to_surface(
        &self,
        body: &PaneBodyPresentation,
        surface: &mut UiSurface,
    ) -> Result<(), EditorUiHostRuntimeError> {
        apply_template_control_attributes_to_surface(
            &body.document_id,
            surface,
            &template_v2_component_patch_attributes(body),
        )
    }

    pub(crate) fn bind_template_actions_for_pane(
        &self,
        pane_id: &str,
        surface: &mut UiSurface,
        host_model: &mut RetainedUiHostModel,
    ) -> Result<(), EditorUiHostRuntimeError> {
        let document_id = host_model.document_id.clone();
        let plugin_owner = self.plugin_v2_document_owner(&document_id);
        let control_attributes = template_control_attributes_from_host_model(host_model)?;
        let mut registry = self
            .template_action_registry
            .lock()
            .expect("template action registry mutex should not be poisoned");
        let control_attributes = registry.rebind_pane(
            pane_id,
            &document_id,
            plugin_owner.as_ref(),
            control_attributes,
        );
        apply_template_control_attributes_to_host_model(
            &document_id,
            host_model,
            &control_attributes,
        )?;
        for node in &mut host_model.nodes {
            for binding in &mut node.bindings {
                let Some(action_source) = binding.template_action_source.clone() else {
                    continue;
                };
                binding.action_id = registry.bind_for_control(
                    pane_id,
                    &document_id,
                    &binding.binding_id,
                    plugin_owner.clone(),
                    node.control_id.as_deref(),
                    node.attributes.clone(),
                    action_source,
                    control_attributes.clone(),
                );
            }
        }
        drop(registry);
        apply_template_control_attributes_to_surface(&document_id, surface, &control_attributes)
    }

    pub(crate) fn update_template_action_control_state(
        &self,
        pane_id: &str,
        control_id: &str,
        attributes: &BTreeMap<String, toml::Value>,
    ) -> bool {
        self.template_action_registry
            .lock()
            .expect("template action registry mutex should not be poisoned")
            .update_control_attributes_for_pane(pane_id, control_id, attributes)
    }

    pub(crate) fn select_template_table_row(
        &self,
        pane_id: &str,
        control_id: &str,
        source_index: i32,
        identity_kind: &str,
        identity_text: &str,
    ) -> bool {
        self.template_action_registry
            .lock()
            .expect("template action registry mutex should not be poisoned")
            .select_table_row(
                pane_id,
                control_id,
                source_index,
                identity_kind,
                identity_text,
            )
    }

    pub(crate) fn remove_template_actions_for_pane(&self, pane_id: &str) {
        self.template_action_registry
            .lock()
            .expect("template action registry mutex should not be poisoned")
            .remove_pane(pane_id);
    }

    pub(crate) fn dispatch_template_action_for_token<T>(
        &self,
        token: &str,
        dispatch: impl FnOnce(&UiTemplateActionInvocation) -> T,
    ) -> Option<T> {
        // Keep the active document owner and action slot stable through dispatch.
        let plugin_v2_documents = self
            .plugin_v2_documents
            .lock()
            .expect("plugin V2 document catalog mutex should not be poisoned");
        let registry = self
            .template_action_registry
            .lock()
            .expect("template action registry mutex should not be poisoned");
        registry
            .action_for_token(token, |document_id| {
                plugin_v2_documents
                    .get(document_id)
                    .map(|document| document.owner.clone())
            })
            .map(|action| dispatch(&action))
    }

    pub(crate) fn is_template_action_token(&self, token: &str) -> bool {
        token.starts_with("template-v2/")
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

fn template_control_attributes_from_host_model(
    host_model: &RetainedUiHostModel,
) -> Result<BTreeMap<String, BTreeMap<String, toml::Value>>, EditorUiHostRuntimeError> {
    let mut controls = BTreeMap::new();
    for node in &host_model.nodes {
        let Some(control_id) = node.control_id.as_ref() else {
            continue;
        };
        if controls
            .insert(control_id.clone(), node.attributes.clone())
            .is_some()
        {
            return Err(EditorUiHostRuntimeError::DuplicateRetainedControl {
                document_id: host_model.document_id.clone(),
                control_id: control_id.clone(),
            });
        }
    }
    Ok(controls)
}

fn apply_template_control_attributes_to_host_model(
    document_id: &str,
    host_model: &mut RetainedUiHostModel,
    control_attributes: &BTreeMap<String, BTreeMap<String, toml::Value>>,
) -> Result<(), EditorUiHostRuntimeError> {
    for (control_id, attributes) in control_attributes {
        let mut matching_nodes = host_model
            .nodes
            .iter_mut()
            .filter(|node| node.control_id.as_deref() == Some(control_id.as_str()));
        let Some(node) = matching_nodes.next() else {
            return Err(EditorUiHostRuntimeError::MissingRetainedControl {
                document_id: document_id.to_string(),
                control_id: control_id.clone(),
            });
        };
        if matching_nodes.next().is_some() {
            return Err(EditorUiHostRuntimeError::DuplicateRetainedControl {
                document_id: document_id.to_string(),
                control_id: control_id.clone(),
            });
        }
        node.attributes.extend(attributes.clone());
    }
    Ok(())
}

fn apply_template_control_attributes_to_surface(
    document_id: &str,
    surface: &mut UiSurface,
    control_attributes: &BTreeMap<String, BTreeMap<String, toml::Value>>,
) -> Result<(), EditorUiHostRuntimeError> {
    for (control_id, attributes) in control_attributes {
        let matching_node_ids = surface
            .tree
            .nodes
            .iter()
            .filter_map(|(node_id, node)| {
                (node.template_metadata.as_ref()?.control_id.as_deref()
                    == Some(control_id.as_str()))
                .then_some(*node_id)
            })
            .collect::<Vec<_>>();
        let [node_id] = matching_node_ids.as_slice() else {
            return Err(if matching_node_ids.is_empty() {
                EditorUiHostRuntimeError::MissingTemplateSurfaceControl {
                    document_id: document_id.to_string(),
                    control_id: control_id.clone(),
                }
            } else {
                EditorUiHostRuntimeError::DuplicateTemplateSurfaceControl {
                    document_id: document_id.to_string(),
                    control_id: control_id.clone(),
                }
            });
        };
        let disabled = attributes.get("disabled") == Some(&toml::Value::Boolean(true));
        for (property, value) in attributes {
            // `disabled` is terminal input state and must win over a contradictory `enabled` patch.
            if disabled && property == "enabled" {
                continue;
            }
            apply_template_control_property(
                document_id,
                control_id,
                surface,
                *node_id,
                property,
                UiValue::from_toml(value),
            )?;
        }
    }
    Ok(())
}

fn apply_template_control_property(
    document_id: &str,
    control_id: &str,
    surface: &mut UiSurface,
    node_id: zircon_runtime_interface::ui::event_ui::UiNodeId,
    property: &str,
    value: UiValue,
) -> Result<(), EditorUiHostRuntimeError> {
    let report = surface.mutate_property(UiPropertyMutationRequest::new(
        node_id,
        property,
        value.clone(),
    ))?;
    if let Some(detail) = report.message {
        return Err(EditorUiHostRuntimeError::TemplateControlStateRejected {
            document_id: document_id.to_string(),
            control_id: control_id.to_string(),
            property: property.to_string(),
            detail,
        });
    }
    if property == "disabled" {
        let UiValue::Bool(disabled) = value else {
            return Err(EditorUiHostRuntimeError::TemplateControlStateRejected {
                document_id: document_id.to_string(),
                control_id: control_id.to_string(),
                property: property.to_string(),
                detail: "disabled expects a boolean value".to_string(),
            });
        };
        let report = surface.mutate_property(UiPropertyMutationRequest::new(
            node_id,
            "enabled",
            UiValue::Bool(!disabled),
        ))?;
        if let Some(detail) = report.message {
            return Err(EditorUiHostRuntimeError::TemplateControlStateRejected {
                document_id: document_id.to_string(),
                control_id: control_id.to_string(),
                property: "enabled".to_string(),
                detail,
            });
        }
    }
    Ok(())
}

impl EditorUiHostRuntime {
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
        self.v2_documents.insert(
            document_id,
            EditorUiHostV2Document {
                document: outcome.root_document,
                compiled: outcome.compiled,
            },
        );
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
mod pane_control_state_tests {
    use std::collections::BTreeMap;

    use toml::Value;
    use zircon_runtime::ui::surface::UiSurface;
    use zircon_runtime_interface::ui::{
        component::UiValue,
        event_ui::{UiNodeId, UiTreeId},
        layout::UiFrame,
        template::UiActionRef,
        tree::{UiNodePath, UiTemplateNodeMetadata, UiTreeNode},
    };

    use super::*;

    #[test]
    fn pane_control_state_projects_rows_selection_and_disabled_to_native_and_retained_models() {
        let node_id = UiNodeId::new(1);
        let mut surface = UiSurface::new(UiTreeId::new("editor.template.v2.pane-state"));
        surface.tree.insert_root(
            UiTreeNode::new(node_id, UiNodePath::new("root/RowList")).with_template_metadata(
                UiTemplateNodeMetadata {
                    component: "Table".to_string(),
                    control_id: Some("RowList".to_string()),
                    ..Default::default()
                },
            ),
        );
        let rows = Value::Array(vec![Value::Table(toml::map::Map::from_iter([(
            "surface_entity".to_string(),
            Value::Integer(73),
        )]))]);
        let control_attributes = BTreeMap::from([(
            "RowList".to_string(),
            BTreeMap::from([
                ("rows".to_string(), rows.clone()),
                ("selected_row_identity".to_string(), Value::Integer(73)),
                ("disabled".to_string(), Value::Boolean(true)),
                ("enabled".to_string(), Value::Boolean(true)),
            ]),
        )]);
        let mut host_model = RetainedUiHostModel {
            document_id: "plugin.rows.panel".to_string(),
            nodes: vec![RetainedUiHostNodeProjection {
                node_id: "root/RowList".to_string(),
                parent_id: None,
                component: "Table".to_string(),
                control_id: Some("RowList".to_string()),
                frame: UiFrame::default(),
                clip_frame: None,
                z_index: 0,
                attributes: BTreeMap::new(),
                style_overrides: BTreeMap::new(),
                style_tokens: BTreeMap::new(),
                bindings: Vec::new(),
            }],
        };

        apply_template_control_attributes_to_host_model(
            &host_model.document_id.clone(),
            &mut host_model,
            &control_attributes,
        )
        .expect("retained host should receive the current pane control state");
        apply_template_control_attributes_to_surface(
            &host_model.document_id,
            &mut surface,
            &control_attributes,
        )
        .expect("native surface should receive the current pane control state");

        assert_eq!(
            host_model
                .node_by_control_id("RowList")
                .and_then(|node| node.attributes.get("rows")),
            Some(&rows)
        );
        assert_eq!(
            host_model
                .node_by_control_id("RowList")
                .and_then(|node| node.attributes.get("selected_row_identity")),
            Some(&Value::Integer(73))
        );
        assert_eq!(
            surface
                .component_state(node_id)
                .and_then(|state| state.value("selected_row_identity")),
            Some(&UiValue::Int(73))
        );
        assert_eq!(
            surface
                .tree
                .node(node_id)
                .and_then(|node| node.template_metadata.as_ref())
                .and_then(|metadata| metadata.attributes.get("rows")),
            Some(&rows)
        );
        assert!(
            !surface
                .tree
                .node(node_id)
                .expect("native control should remain in the surface")
                .state_flags
                .enabled
        );
    }

    #[test]
    fn plugin_document_replacement_evicts_same_id_action_slots_before_pane_rebuild() {
        let runtime = EditorUiHostRuntime::default();
        let source_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("assets/ui/host/workbench_shell.zui");
        let source = |document_id| {
            super::super::plugin_documents::EditorPluginV2DocumentSource::new(
                document_id,
                "plugins://fixture.plugin/workbench_shell.zui",
                [source_path.clone()],
            )
            .expect("fixture plugin document source should be valid")
        };
        let first =
            super::super::plugin_documents::EditorPluginV2DocumentOwner::new("fixture.plugin", 1)
                .expect("first plugin generation should be valid");
        let second =
            super::super::plugin_documents::EditorPluginV2DocumentOwner::new("fixture.plugin", 2)
                .expect("replacement plugin generation should be valid");
        runtime
            .replace_plugin_v2_documents(first.clone(), [source("fixture.plugin.panel")])
            .expect("first plugin generation should load");
        let token = runtime
            .template_action_registry
            .lock()
            .expect("template action registry mutex should not be poisoned")
            .bind(
                "fixture.plugin.pane",
                "fixture.plugin.panel",
                "Bake/Click",
                Some(first),
                BTreeMap::new(),
                UiActionRef {
                    route: Some("fixture.operation".to_string()),
                    action: None,
                    payload: BTreeMap::new(),
                },
                BTreeMap::new(),
            );

        let update = runtime
            .replace_plugin_v2_documents(second, [source("fixture.plugin.panel")])
            .expect("same-id replacement should load the next generation");

        assert!(update.retired_document_ids().is_empty());
        assert!(!runtime
            .template_action_registry
            .lock()
            .expect("template action registry mutex should not be poisoned")
            .contains_token(&token));
    }
}
