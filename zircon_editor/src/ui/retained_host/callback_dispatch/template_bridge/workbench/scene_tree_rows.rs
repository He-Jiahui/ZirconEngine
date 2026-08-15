use toml::Value;
use zircon_runtime::ui::surface::UiSurface;
use zircon_runtime_interface::ui::{
    binding::UiEventKind, template::UiBindingRef, tree::UiTemplateNodeMetadata,
};

use super::super::virtual_rows::{
    TemplateBridgeVirtualRowContext, TemplateBridgeVirtualRowSequence,
};
use super::{
    componentized_window::BuiltinWorkbenchWindowTemplateSurfaceBridge,
    error::BuiltinHostWindowTemplateBridgeError,
};

const SCENE_TREE_CONTAINER: &str = "WorkbenchSceneTree";
// Dynamic rows reuse an authored scene-row selection binding so the retained host projection can
// resolve the route from the source document while each row keeps its own control id and state.
const VIRTUAL_ROW_BINDING_ID: &str = "Hierarchy/SelectSlot10";
const VIRTUAL_ROW_ROUTE: &str = "Hierarchy.SelectSlot10";

pub(super) const SCENE_TREE_STATIC_CONTROLS: &[&str] = &[
    "WorkbenchSceneRootItem",
    "WorkbenchSceneEnvironmentItem",
    "WorkbenchSceneLevelItem",
    "WorkbenchScenePropsItem",
    "WorkbenchScenePlayerItem",
    "WorkbenchSceneAudioItem",
    "WorkbenchSceneSlot07Item",
    "WorkbenchSceneSlot08Item",
    "WorkbenchSceneSlot09Item",
    "WorkbenchSceneSlot10Item",
];

impl BuiltinWorkbenchWindowTemplateSurfaceBridge {
    pub(super) fn reconcile_scene_tree_row_capacity(
        &mut self,
        entry_count: usize,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        let topology_changed = scene_tree_virtual_rows(&self.template_surface.surface)?.reconcile(
            &mut self.template_surface.surface,
            entry_count,
            virtual_metadata_from_prototype,
        )?;
        if topology_changed {
            self.template_surface.refresh_control_node_index()?;
        }
        Ok(())
    }

    pub(super) fn scene_tree_control_ids(
        &self,
    ) -> Result<Vec<String>, BuiltinHostWindowTemplateBridgeError> {
        let mut controls = SCENE_TREE_STATIC_CONTROLS
            .iter()
            .map(|control_id| (*control_id).to_string())
            .collect::<Vec<_>>();
        controls.extend(
            scene_tree_virtual_rows(&self.template_surface.surface)?
                .virtual_control_ids(&self.template_surface.surface),
        );
        Ok(controls)
    }

    pub(super) fn is_scene_tree_control(
        &self,
        control_id: &str,
    ) -> Result<bool, BuiltinHostWindowTemplateBridgeError> {
        Ok(SCENE_TREE_STATIC_CONTROLS.contains(&control_id)
            || self.scene_hierarchy_projection.contains_control(control_id))
    }
}

fn scene_tree_virtual_rows(
    surface: &UiSurface,
) -> Result<TemplateBridgeVirtualRowSequence, BuiltinHostWindowTemplateBridgeError> {
    TemplateBridgeVirtualRowSequence::from_surface_repeat(surface, SCENE_TREE_CONTAINER)
        .map_err(BuiltinHostWindowTemplateBridgeError::from)
}

fn virtual_metadata_from_prototype(
    mut metadata: UiTemplateNodeMetadata,
    context: &TemplateBridgeVirtualRowContext,
) -> UiTemplateNodeMetadata {
    metadata.attributes.insert(
        "text".to_string(),
        Value::String(format!("Scene Item {:02}", context.row_number)),
    );
    metadata
        .attributes
        .insert("tree_depth".to_string(), Value::Integer(0));
    metadata
        .attributes
        .insert("tree_indent_px".to_string(), Value::Float(0.0));
    metadata
        .attributes
        .insert("expanded".to_string(), Value::Boolean(false));
    metadata
        .attributes
        .insert("selected".to_string(), Value::Boolean(false));
    metadata.attributes.insert(
        "visibility".to_string(),
        Value::String("collapsed".to_string()),
    );
    metadata.bindings = vec![UiBindingRef {
        id: VIRTUAL_ROW_BINDING_ID.to_string(),
        event: UiEventKind::Click,
        route: Some(VIRTUAL_ROW_ROUTE.to_string()),
        action: None,
        targets: Vec::new(),
    }];
    metadata
}
