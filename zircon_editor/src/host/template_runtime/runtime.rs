use std::collections::BTreeMap;

use thiserror::Error;
use zircon_editor_ui::{
    AssetCommand, DockCommand, DraftCommand, EditorComponentCatalog, EditorComponentDescriptor,
    EditorTemplateAdapter, EditorTemplateError, EditorTemplateRegistry, EditorUiBinding,
    EditorUiBindingPayload, EditorUiControlService, EditorUiEventKind, InspectorFieldChange,
    ViewportCommand, WelcomeCommand,
};
use zircon_scene::{
    DisplayMode, GridMode, ProjectionMode, SceneViewportTool, TransformSpace, ViewOrientation,
};
use zircon_ui::{
    UiBindingValue, UiSurface, UiTemplateLoader, UiTemplateNode, UiTemplateSurfaceBuilder, UiTree,
    UiTreeId,
};

use super::{
    SlintUiBindingProjection, SlintUiHostAdapter, SlintUiHostBindingProjection, SlintUiHostModel,
    SlintUiHostNodeProjection, SlintUiHostProjection, SlintUiNodeProjection, SlintUiProjection,
};

const WORKBENCH_SHELL_DOCUMENT_ID: &str = "workbench.shell";
const SCENE_VIEWPORT_TOOLBAR_DOCUMENT_ID: &str = "scene.viewport_toolbar";
const ASSET_SURFACE_DOCUMENT_ID: &str = "asset.surface_controls";
const WELCOME_SURFACE_DOCUMENT_ID: &str = "startup.welcome_controls";
const INSPECTOR_SURFACE_DOCUMENT_ID: &str = "inspector.surface_controls";
const PANE_SURFACE_DOCUMENT_ID: &str = "pane.surface_controls";
const DYNAMIC_DOCUMENT_TAB_INSTANCE_ID: &str = "$document_tab_instance";
const DYNAMIC_MAIN_PAGE_ID: &str = "$main_page_id";
const WORKBENCH_SHELL_TEMPLATE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/ui/templates/workbench_shell.toml"
));
const SCENE_VIEWPORT_TOOLBAR_TEMPLATE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/ui/templates/scene_viewport_toolbar.toml"
));
const ASSET_SURFACE_TEMPLATE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/ui/templates/asset_surface_controls.toml"
));
const WELCOME_SURFACE_TEMPLATE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/ui/templates/startup_welcome_controls.toml"
));
const INSPECTOR_SURFACE_TEMPLATE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/ui/templates/inspector_surface_controls.toml"
));
const PANE_SURFACE_TEMPLATE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/ui/templates/pane_surface_controls.toml"
));

#[derive(Debug, Error, PartialEq, Eq)]
pub enum EditorUiHostRuntimeError {
    #[error(transparent)]
    Template(#[from] EditorTemplateError),
    #[error(transparent)]
    UiTemplate(#[from] zircon_ui::UiTemplateError),
    #[error(transparent)]
    UiTemplateBuild(#[from] zircon_ui::UiTemplateBuildError),
    #[error("slint projection is missing binding {binding_id}")]
    MissingProjectionBinding { binding_id: String },
    #[error("shared surface node {node_path} is missing template metadata")]
    MissingSurfaceMetadata { node_path: String },
}

#[derive(Default)]
pub struct EditorUiHostRuntime {
    component_catalog: EditorComponentCatalog,
    template_registry: EditorTemplateRegistry,
    template_adapter: EditorTemplateAdapter,
    builtin_workbench_loaded: bool,
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
        let document = UiTemplateLoader::load_toml_file(path)?;
        self.template_registry
            .register_document(document_id, document)
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

    pub fn load_builtin_workbench_shell(&mut self) -> Result<(), EditorUiHostRuntimeError> {
        if self.builtin_workbench_loaded {
            return Ok(());
        }

        for descriptor in builtin_component_descriptors() {
            self.register_component(descriptor)?;
        }
        for (document_id, template) in builtin_template_documents() {
            let document = UiTemplateLoader::load_toml_str(template)?;
            self.template_registry
                .register_document(document_id, document)
                .map_err(EditorUiHostRuntimeError::from)?;
        }

        for (binding_id, binding) in builtin_template_bindings() {
            self.register_binding(binding_id, binding)?;
        }

        self.builtin_workbench_loaded = true;
        Ok(())
    }

    pub fn project_document(
        &self,
        document_id: &str,
    ) -> Result<SlintUiProjection, EditorUiHostRuntimeError> {
        let instance = self
            .template_registry
            .instantiate(document_id)
            .map_err(EditorUiHostRuntimeError::from)?;

        let mut bindings = Vec::new();
        let root = project_node(&instance.root, &self.template_adapter, &mut bindings)?;
        Ok(SlintUiProjection {
            document_id: document_id.to_string(),
            root,
            bindings,
        })
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
        let bindings = projection
            .bindings
            .iter()
            .cloned()
            .map(|binding| (binding.binding_id.clone(), binding))
            .collect::<BTreeMap<_, _>>();
        let mut nodes = Vec::new();
        collect_host_nodes(&projection.root, None, "root", &bindings, &mut nodes)?;
        Ok(SlintUiHostModel {
            document_id: projection.document_id.clone(),
            nodes,
        })
    }

    pub fn build_host_model_with_surface(
        &self,
        projection: &SlintUiProjection,
        surface: &UiSurface,
    ) -> Result<SlintUiHostModel, EditorUiHostRuntimeError> {
        let bindings = projection
            .bindings
            .iter()
            .cloned()
            .map(|binding| (binding.binding_id.clone(), binding))
            .collect::<BTreeMap<_, _>>();
        let mut nodes = Vec::new();
        for root_id in &surface.tree.roots {
            collect_surface_host_nodes(&surface.tree, *root_id, &bindings, &mut nodes)?;
        }
        Ok(SlintUiHostModel {
            document_id: projection.document_id.clone(),
            nodes,
        })
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

fn project_node(
    node: &UiTemplateNode,
    adapter: &EditorTemplateAdapter,
    bindings: &mut Vec<SlintUiBindingProjection>,
) -> Result<SlintUiNodeProjection, EditorUiHostRuntimeError> {
    let mut binding_ids = Vec::new();
    for binding_ref in &node.bindings {
        let binding = adapter
            .resolve_binding(binding_ref)
            .map_err(EditorUiHostRuntimeError::from)?;
        binding_ids.push(binding_ref.id.clone());
        bindings.push(SlintUiBindingProjection {
            binding_id: binding_ref.id.clone(),
            binding,
            route_id: None,
        });
    }

    Ok(SlintUiNodeProjection {
        component: node.component.clone().unwrap_or_default(),
        control_id: node.control_id.clone(),
        attributes: node.attributes.clone(),
        style_tokens: node.style_tokens.clone(),
        binding_ids,
        children: node
            .children
            .iter()
            .map(|child| project_node(child, adapter, bindings))
            .collect::<Result<Vec<_>, _>>()?,
    })
}

fn collect_host_nodes(
    node: &SlintUiNodeProjection,
    parent_id: Option<&str>,
    node_id: &str,
    bindings: &BTreeMap<String, SlintUiBindingProjection>,
    host_nodes: &mut Vec<SlintUiHostNodeProjection>,
) -> Result<(), EditorUiHostRuntimeError> {
    let node_bindings = node
        .binding_ids
        .iter()
        .map(|binding_id| {
            bindings
                .get(binding_id)
                .map(|binding| SlintUiHostBindingProjection {
                    binding_id: binding.binding_id.clone(),
                    event_kind: binding.binding.path().event_kind,
                    route_id: binding.route_id,
                })
                .ok_or_else(|| EditorUiHostRuntimeError::MissingProjectionBinding {
                    binding_id: binding_id.clone(),
                })
        })
        .collect::<Result<Vec<_>, _>>()?;

    host_nodes.push(SlintUiHostNodeProjection {
        node_id: node_id.to_string(),
        parent_id: parent_id.map(str::to_string),
        component: node.component.clone(),
        control_id: node.control_id.clone(),
        frame: Default::default(),
        clip_frame: None,
        z_index: 0,
        attributes: node.attributes.clone(),
        style_tokens: node.style_tokens.clone(),
        bindings: node_bindings,
    });

    for (index, child) in node.children.iter().enumerate() {
        let child_id = format!("{node_id}.{index}");
        collect_host_nodes(child, Some(node_id), &child_id, bindings, host_nodes)?;
    }
    Ok(())
}

fn collect_surface_host_nodes(
    tree: &UiTree,
    node_id: zircon_ui::UiNodeId,
    bindings: &BTreeMap<String, SlintUiBindingProjection>,
    host_nodes: &mut Vec<SlintUiHostNodeProjection>,
) -> Result<(), EditorUiHostRuntimeError> {
    let node = tree
        .node(node_id)
        .expect("surface traversal should only visit valid nodes");
    let metadata = node.template_metadata.as_ref().ok_or_else(|| {
        EditorUiHostRuntimeError::MissingSurfaceMetadata {
            node_path: node.node_path.0.clone(),
        }
    })?;
    let node_bindings = metadata
        .bindings
        .iter()
        .map(|binding_ref| {
            bindings
                .get(&binding_ref.id)
                .map(|binding| SlintUiHostBindingProjection {
                    binding_id: binding.binding_id.clone(),
                    event_kind: binding.binding.path().event_kind,
                    route_id: binding.route_id,
                })
                .ok_or_else(|| EditorUiHostRuntimeError::MissingProjectionBinding {
                    binding_id: binding_ref.id.clone(),
                })
        })
        .collect::<Result<Vec<_>, _>>()?;

    host_nodes.push(SlintUiHostNodeProjection {
        node_id: node.node_path.0.clone(),
        parent_id: node
            .parent
            .and_then(|parent_id| tree.node(parent_id))
            .map(|parent| parent.node_path.0.clone()),
        component: metadata.component.clone(),
        control_id: metadata.control_id.clone(),
        frame: node.layout_cache.frame,
        clip_frame: node.layout_cache.clip_frame,
        z_index: node.z_index,
        attributes: metadata.attributes.clone(),
        style_tokens: metadata.style_tokens.clone(),
        bindings: node_bindings,
    });

    for child_id in &node.children {
        collect_surface_host_nodes(tree, *child_id, bindings, host_nodes)?;
    }

    Ok(())
}

fn builtin_template_documents() -> [(&'static str, &'static str); 6] {
    [
        (WORKBENCH_SHELL_DOCUMENT_ID, WORKBENCH_SHELL_TEMPLATE),
        (
            SCENE_VIEWPORT_TOOLBAR_DOCUMENT_ID,
            SCENE_VIEWPORT_TOOLBAR_TEMPLATE,
        ),
        (ASSET_SURFACE_DOCUMENT_ID, ASSET_SURFACE_TEMPLATE),
        (WELCOME_SURFACE_DOCUMENT_ID, WELCOME_SURFACE_TEMPLATE),
        (INSPECTOR_SURFACE_DOCUMENT_ID, INSPECTOR_SURFACE_TEMPLATE),
        (PANE_SURFACE_DOCUMENT_ID, PANE_SURFACE_TEMPLATE),
    ]
}

fn builtin_template_bindings() -> BTreeMap<String, EditorUiBinding> {
    BTreeMap::from([
        (
            "WorkbenchMenuBar/OpenProject".to_string(),
            EditorUiBinding::new(
                "WorkbenchMenuBar",
                "OpenProject",
                EditorUiEventKind::Click,
                EditorUiBindingPayload::menu_action("OpenProject"),
            ),
        ),
        (
            "WorkbenchMenuBar/SaveProject".to_string(),
            EditorUiBinding::new(
                "WorkbenchMenuBar",
                "SaveProject",
                EditorUiEventKind::Click,
                EditorUiBindingPayload::menu_action("SaveProject"),
            ),
        ),
        (
            "WorkbenchMenuBar/ResetLayout".to_string(),
            EditorUiBinding::new(
                "WorkbenchMenuBar",
                "ResetLayout",
                EditorUiEventKind::Click,
                EditorUiBindingPayload::menu_action("ResetLayout"),
            ),
        ),
        (
            "ActivityRail/ProjectToggle".to_string(),
            EditorUiBinding::new(
                "ActivityRail",
                "ProjectToggle",
                EditorUiEventKind::Click,
                EditorUiBindingPayload::dock_command(DockCommand::ActivateDrawerTab {
                    slot: "left_top".to_string(),
                    instance_id: "editor.project#1".to_string(),
                }),
            ),
        ),
        (
            "ActivityRail/HierarchyToggle".to_string(),
            EditorUiBinding::new(
                "ActivityRail",
                "HierarchyToggle",
                EditorUiEventKind::Click,
                EditorUiBindingPayload::dock_command(DockCommand::ActivateDrawerTab {
                    slot: "left_top".to_string(),
                    instance_id: "editor.hierarchy#1".to_string(),
                }),
            ),
        ),
        (
            "ActivityRail/ConsoleToggle".to_string(),
            EditorUiBinding::new(
                "ActivityRail",
                "ConsoleToggle",
                EditorUiEventKind::Click,
                EditorUiBindingPayload::dock_command(DockCommand::ActivateDrawerTab {
                    slot: "bottom_left".to_string(),
                    instance_id: "editor.console#1".to_string(),
                }),
            ),
        ),
        (
            "DocumentTabs/ActivateTab".to_string(),
            EditorUiBinding::new(
                "DocumentTabs",
                "ActivateTab",
                EditorUiEventKind::Change,
                EditorUiBindingPayload::dock_command(DockCommand::FocusView {
                    instance_id: DYNAMIC_DOCUMENT_TAB_INSTANCE_ID.to_string(),
                }),
            ),
        ),
        (
            "DocumentTabs/CloseTab".to_string(),
            EditorUiBinding::new(
                "DocumentTabs",
                "CloseTab",
                EditorUiEventKind::Submit,
                EditorUiBindingPayload::dock_command(DockCommand::CloseView {
                    instance_id: DYNAMIC_DOCUMENT_TAB_INSTANCE_ID.to_string(),
                }),
            ),
        ),
        (
            "WorkbenchShell/ActivateMainPage".to_string(),
            EditorUiBinding::new(
                "WorkbenchShell",
                "ActivateMainPage",
                EditorUiEventKind::Change,
                EditorUiBindingPayload::dock_command(DockCommand::ActivateMainPage {
                    page_id: DYNAMIC_MAIN_PAGE_ID.to_string(),
                }),
            ),
        ),
        (
            "ViewportToolbar/SetTool".to_string(),
            EditorUiBinding::new(
                "ViewportToolbar",
                "SetTool",
                EditorUiEventKind::Change,
                EditorUiBindingPayload::viewport_command(ViewportCommand::SetTool(
                    SceneViewportTool::Drag,
                )),
            ),
        ),
        (
            "ViewportToolbar/SetTransformSpace".to_string(),
            EditorUiBinding::new(
                "ViewportToolbar",
                "SetTransformSpace",
                EditorUiEventKind::Change,
                EditorUiBindingPayload::viewport_command(ViewportCommand::SetTransformSpace(
                    TransformSpace::Local,
                )),
            ),
        ),
        (
            "ViewportToolbar/SetProjectionMode".to_string(),
            EditorUiBinding::new(
                "ViewportToolbar",
                "SetProjectionMode",
                EditorUiEventKind::Change,
                EditorUiBindingPayload::viewport_command(ViewportCommand::SetProjectionMode(
                    ProjectionMode::Perspective,
                )),
            ),
        ),
        (
            "ViewportToolbar/AlignView".to_string(),
            EditorUiBinding::new(
                "ViewportToolbar",
                "AlignView",
                EditorUiEventKind::Change,
                EditorUiBindingPayload::viewport_command(ViewportCommand::AlignView(
                    ViewOrientation::User,
                )),
            ),
        ),
        (
            "ViewportToolbar/SetDisplayMode".to_string(),
            EditorUiBinding::new(
                "ViewportToolbar",
                "SetDisplayMode",
                EditorUiEventKind::Change,
                EditorUiBindingPayload::viewport_command(ViewportCommand::SetDisplayMode(
                    DisplayMode::Shaded,
                )),
            ),
        ),
        (
            "ViewportToolbar/SetGridMode".to_string(),
            EditorUiBinding::new(
                "ViewportToolbar",
                "SetGridMode",
                EditorUiEventKind::Change,
                EditorUiBindingPayload::viewport_command(ViewportCommand::SetGridMode(
                    GridMode::Hidden,
                )),
            ),
        ),
        (
            "ViewportToolbar/SetTranslateSnap".to_string(),
            EditorUiBinding::new(
                "ViewportToolbar",
                "SetTranslateSnap",
                EditorUiEventKind::Change,
                EditorUiBindingPayload::viewport_command(ViewportCommand::SetTranslateSnap(0.1)),
            ),
        ),
        (
            "ViewportToolbar/SetRotateSnapDegrees".to_string(),
            EditorUiBinding::new(
                "ViewportToolbar",
                "SetRotateSnapDegrees",
                EditorUiEventKind::Change,
                EditorUiBindingPayload::viewport_command(ViewportCommand::SetRotateSnapDegrees(
                    5.0,
                )),
            ),
        ),
        (
            "ViewportToolbar/SetScaleSnap".to_string(),
            EditorUiBinding::new(
                "ViewportToolbar",
                "SetScaleSnap",
                EditorUiEventKind::Change,
                EditorUiBindingPayload::viewport_command(ViewportCommand::SetScaleSnap(0.05)),
            ),
        ),
        (
            "ViewportToolbar/SetPreviewLighting".to_string(),
            EditorUiBinding::new(
                "ViewportToolbar",
                "SetPreviewLighting",
                EditorUiEventKind::Change,
                EditorUiBindingPayload::viewport_command(ViewportCommand::SetPreviewLighting(
                    false,
                )),
            ),
        ),
        (
            "ViewportToolbar/SetPreviewSkybox".to_string(),
            EditorUiBinding::new(
                "ViewportToolbar",
                "SetPreviewSkybox",
                EditorUiEventKind::Change,
                EditorUiBindingPayload::viewport_command(ViewportCommand::SetPreviewSkybox(false)),
            ),
        ),
        (
            "ViewportToolbar/SetGizmosEnabled".to_string(),
            EditorUiBinding::new(
                "ViewportToolbar",
                "SetGizmosEnabled",
                EditorUiEventKind::Change,
                EditorUiBindingPayload::viewport_command(ViewportCommand::SetGizmosEnabled(true)),
            ),
        ),
        (
            "ViewportToolbar/FrameSelection".to_string(),
            EditorUiBinding::new(
                "ViewportToolbar",
                "FrameSelection",
                EditorUiEventKind::Click,
                EditorUiBindingPayload::viewport_command(ViewportCommand::FrameSelection),
            ),
        ),
        (
            "AssetSurface/SelectFolder".to_string(),
            EditorUiBinding::new(
                "AssetSurface",
                "SelectFolder",
                EditorUiEventKind::Change,
                EditorUiBindingPayload::asset_command(AssetCommand::SelectFolder {
                    folder_id: "Assets".to_string(),
                }),
            ),
        ),
        (
            "AssetSurface/SelectItem".to_string(),
            EditorUiBinding::new(
                "AssetSurface",
                "SelectItem",
                EditorUiEventKind::Change,
                EditorUiBindingPayload::asset_command(AssetCommand::SelectItem {
                    asset_uuid: "00000000-0000-0000-0000-000000000000".to_string(),
                }),
            ),
        ),
        (
            "AssetSurface/SearchEdited".to_string(),
            EditorUiBinding::new(
                "AssetSurface",
                "SearchEdited",
                EditorUiEventKind::Change,
                EditorUiBindingPayload::asset_command(AssetCommand::SetSearchQuery {
                    query: String::new(),
                }),
            ),
        ),
        (
            "AssetSurface/SetKindFilter".to_string(),
            EditorUiBinding::new(
                "AssetSurface",
                "SetKindFilter",
                EditorUiEventKind::Change,
                EditorUiBindingPayload::asset_command(AssetCommand::SetKindFilter {
                    kind: String::new(),
                }),
            ),
        ),
        (
            "AssetSurface/SetViewMode".to_string(),
            EditorUiBinding::new(
                "AssetSurface",
                "SetViewMode",
                EditorUiEventKind::Change,
                EditorUiBindingPayload::asset_command(AssetCommand::SetViewMode {
                    surface: "activity".to_string(),
                    view_mode: "list".to_string(),
                }),
            ),
        ),
        (
            "AssetSurface/SetUtilityTab".to_string(),
            EditorUiBinding::new(
                "AssetSurface",
                "SetUtilityTab",
                EditorUiEventKind::Change,
                EditorUiBindingPayload::asset_command(AssetCommand::SetUtilityTab {
                    surface: "activity".to_string(),
                    tab: "preview".to_string(),
                }),
            ),
        ),
        (
            "AssetSurface/ActivateReference".to_string(),
            EditorUiBinding::new(
                "AssetSurface",
                "ActivateReference",
                EditorUiEventKind::Click,
                EditorUiBindingPayload::asset_command(AssetCommand::ActivateReference {
                    asset_uuid: "00000000-0000-0000-0000-000000000000".to_string(),
                }),
            ),
        ),
        (
            "AssetSurface/OpenAssetBrowser".to_string(),
            EditorUiBinding::new(
                "AssetSurface",
                "OpenAssetBrowser",
                EditorUiEventKind::Click,
                EditorUiBindingPayload::asset_command(AssetCommand::OpenAssetBrowser),
            ),
        ),
        (
            "AssetSurface/LocateSelectedAsset".to_string(),
            EditorUiBinding::new(
                "AssetSurface",
                "LocateSelectedAsset",
                EditorUiEventKind::Click,
                EditorUiBindingPayload::asset_command(AssetCommand::LocateSelectedAsset),
            ),
        ),
        (
            "AssetSurface/ImportModel".to_string(),
            EditorUiBinding::new(
                "AssetSurface",
                "ImportModel",
                EditorUiEventKind::Click,
                EditorUiBindingPayload::asset_command(AssetCommand::ImportModel),
            ),
        ),
        (
            "WelcomeSurface/ProjectNameEdited".to_string(),
            EditorUiBinding::new(
                "WelcomeSurface",
                "ProjectNameEdited",
                EditorUiEventKind::Change,
                EditorUiBindingPayload::welcome_command(WelcomeCommand::SetProjectName {
                    value: String::new(),
                }),
            ),
        ),
        (
            "WelcomeSurface/LocationEdited".to_string(),
            EditorUiBinding::new(
                "WelcomeSurface",
                "LocationEdited",
                EditorUiEventKind::Change,
                EditorUiBindingPayload::welcome_command(WelcomeCommand::SetLocation {
                    value: String::new(),
                }),
            ),
        ),
        (
            "WelcomeSurface/CreateProject".to_string(),
            EditorUiBinding::new(
                "WelcomeSurface",
                "CreateProject",
                EditorUiEventKind::Click,
                EditorUiBindingPayload::welcome_command(WelcomeCommand::CreateProject),
            ),
        ),
        (
            "WelcomeSurface/OpenExistingProject".to_string(),
            EditorUiBinding::new(
                "WelcomeSurface",
                "OpenExistingProject",
                EditorUiEventKind::Click,
                EditorUiBindingPayload::welcome_command(WelcomeCommand::OpenExistingProject),
            ),
        ),
        (
            "WelcomeSurface/OpenRecentProject".to_string(),
            EditorUiBinding::new(
                "WelcomeSurface",
                "OpenRecentProject",
                EditorUiEventKind::Click,
                EditorUiBindingPayload::welcome_command(WelcomeCommand::OpenRecentProject {
                    path: "E:/Projects/Sandbox".to_string(),
                }),
            ),
        ),
        (
            "WelcomeSurface/RemoveRecentProject".to_string(),
            EditorUiBinding::new(
                "WelcomeSurface",
                "RemoveRecentProject",
                EditorUiEventKind::Click,
                EditorUiBindingPayload::welcome_command(WelcomeCommand::RemoveRecentProject {
                    path: "E:/Projects/Sandbox".to_string(),
                }),
            ),
        ),
        (
            "InspectorView/NameField".to_string(),
            EditorUiBinding::new(
                "InspectorView",
                "NameField",
                EditorUiEventKind::Change,
                EditorUiBindingPayload::draft_command(DraftCommand::SetInspectorField {
                    subject_path: "entity://selected".to_string(),
                    field_id: "name".to_string(),
                    value: UiBindingValue::string(String::new()),
                }),
            ),
        ),
        (
            "InspectorView/ParentField".to_string(),
            EditorUiBinding::new(
                "InspectorView",
                "ParentField",
                EditorUiEventKind::Change,
                EditorUiBindingPayload::draft_command(DraftCommand::SetInspectorField {
                    subject_path: "entity://selected".to_string(),
                    field_id: "parent".to_string(),
                    value: UiBindingValue::string(String::new()),
                }),
            ),
        ),
        (
            "InspectorView/PositionXField".to_string(),
            EditorUiBinding::new(
                "InspectorView",
                "PositionXField",
                EditorUiEventKind::Change,
                EditorUiBindingPayload::draft_command(DraftCommand::SetInspectorField {
                    subject_path: "entity://selected".to_string(),
                    field_id: "transform.translation.x".to_string(),
                    value: UiBindingValue::string(String::new()),
                }),
            ),
        ),
        (
            "InspectorView/PositionYField".to_string(),
            EditorUiBinding::new(
                "InspectorView",
                "PositionYField",
                EditorUiEventKind::Change,
                EditorUiBindingPayload::draft_command(DraftCommand::SetInspectorField {
                    subject_path: "entity://selected".to_string(),
                    field_id: "transform.translation.y".to_string(),
                    value: UiBindingValue::string(String::new()),
                }),
            ),
        ),
        (
            "InspectorView/PositionZField".to_string(),
            EditorUiBinding::new(
                "InspectorView",
                "PositionZField",
                EditorUiEventKind::Change,
                EditorUiBindingPayload::draft_command(DraftCommand::SetInspectorField {
                    subject_path: "entity://selected".to_string(),
                    field_id: "transform.translation.z".to_string(),
                    value: UiBindingValue::string(String::new()),
                }),
            ),
        ),
        (
            "InspectorView/ApplyBatchButton".to_string(),
            EditorUiBinding::new(
                "InspectorView",
                "ApplyBatchButton",
                EditorUiEventKind::Click,
                EditorUiBindingPayload::inspector_field_batch(
                    "entity://selected",
                    Vec::<InspectorFieldChange>::new(),
                ),
            ),
        ),
        (
            "InspectorView/DeleteSelected".to_string(),
            EditorUiBinding::new(
                "InspectorView",
                "DeleteSelected",
                EditorUiEventKind::Click,
                EditorUiBindingPayload::menu_action("DeleteSelected"),
            ),
        ),
        (
            "PaneSurface/TriggerAction".to_string(),
            EditorUiBinding::new(
                "PaneSurface",
                "TriggerAction",
                EditorUiEventKind::Click,
                EditorUiBindingPayload::menu_action("OpenProject"),
            ),
        ),
    ])
}

fn builtin_component_descriptors() -> Vec<EditorComponentDescriptor> {
    vec![
        EditorComponentDescriptor::new(
            "WorkbenchShell",
            WORKBENCH_SHELL_DOCUMENT_ID,
            "WorkbenchShell",
        ),
        EditorComponentDescriptor::new("MenuBar", WORKBENCH_SHELL_DOCUMENT_ID, "WorkbenchMenuBar"),
        EditorComponentDescriptor::new("ActivityRail", WORKBENCH_SHELL_DOCUMENT_ID, "ActivityRail"),
        EditorComponentDescriptor::new("DocumentHost", WORKBENCH_SHELL_DOCUMENT_ID, "DocumentHost"),
        EditorComponentDescriptor::new("StatusBar", WORKBENCH_SHELL_DOCUMENT_ID, "StatusBar"),
        EditorComponentDescriptor::new(
            "SceneViewportToolbar",
            SCENE_VIEWPORT_TOOLBAR_DOCUMENT_ID,
            "ViewportToolbar",
        ),
        EditorComponentDescriptor::new(
            "AssetSurfaceControls",
            ASSET_SURFACE_DOCUMENT_ID,
            "AssetSurface",
        ),
        EditorComponentDescriptor::new(
            "WelcomeSurfaceControls",
            WELCOME_SURFACE_DOCUMENT_ID,
            "WelcomeSurface",
        ),
        EditorComponentDescriptor::new(
            "InspectorSurfaceControls",
            INSPECTOR_SURFACE_DOCUMENT_ID,
            "InspectorView",
        ),
        EditorComponentDescriptor::new(
            "PaneSurfaceControls",
            PANE_SURFACE_DOCUMENT_ID,
            "PaneSurface",
        ),
    ]
}
