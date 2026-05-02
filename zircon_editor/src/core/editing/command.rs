//! Undoable editor commands that mutate the ECS world.

use zircon_runtime::scene::components::{NodeKind, NodeRecord};
use zircon_runtime::scene::{NodeId, Scene};
use zircon_runtime_interface::math::Transform;
use zircon_runtime_interface::resource::{MaterialMarker, ModelMarker, ResourceHandle};

#[derive(Clone, Debug)]
pub(crate) enum EditorCommand {
    CreateNode(CreateNodeCommand),
    DeleteNode(DeleteNodeCommand),
    UpdateNode(UpdateNodeCommand),
}

impl EditorCommand {
    pub(crate) fn create_node(
        scene: &mut Scene,
        selected: Option<NodeId>,
        kind: NodeKind,
    ) -> Result<Self, String> {
        Ok(Self::CreateNode(CreateNodeCommand::spawn_node(
            scene, selected, kind,
        )?))
    }

    pub(crate) fn import_mesh(
        scene: &mut Scene,
        selected: Option<NodeId>,
        model: ResourceHandle<ModelMarker>,
        material: ResourceHandle<MaterialMarker>,
    ) -> Result<Self, String> {
        Ok(Self::CreateNode(CreateNodeCommand::import_mesh(
            scene, selected, model, material,
        )?))
    }

    pub(crate) fn delete_node(
        scene: &mut Scene,
        selected: Option<NodeId>,
        node_id: NodeId,
    ) -> Result<Self, String> {
        Ok(Self::DeleteNode(DeleteNodeCommand::capture(
            scene, selected, node_id,
        )?))
    }

    pub(crate) fn rename_node(
        scene: &mut Scene,
        selected: Option<NodeId>,
        node_id: NodeId,
        name: String,
    ) -> Result<Option<Self>, String> {
        Ok(UpdateNodeCommand::capture_name(scene, selected, node_id, name)?.map(Self::UpdateNode))
    }

    pub(crate) fn set_parent(
        scene: &mut Scene,
        selected: Option<NodeId>,
        node_id: NodeId,
        parent: Option<NodeId>,
    ) -> Result<Option<Self>, String> {
        Ok(
            UpdateNodeCommand::capture_parent(scene, selected, node_id, parent)?
                .map(Self::UpdateNode),
        )
    }

    pub(crate) fn set_transform(
        scene: &mut Scene,
        selected: Option<NodeId>,
        node_id: NodeId,
        after: Transform,
    ) -> Result<Option<Self>, String> {
        Ok(
            UpdateNodeCommand::capture_transform(scene, selected, node_id, after)?
                .map(Self::UpdateNode),
        )
    }

    pub(crate) fn update_node(
        scene: &mut Scene,
        selected: Option<NodeId>,
        node_id: NodeId,
        name: String,
        parent: Option<NodeId>,
        transform: Transform,
    ) -> Result<Option<Self>, String> {
        Ok(UpdateNodeCommand::capture(
            scene,
            selected,
            node_id,
            NodeEditState {
                name,
                parent,
                transform,
            },
        )?
        .map(Self::UpdateNode))
    }

    pub(crate) fn apply(&self, scene: &mut Scene) -> Result<Option<NodeId>, String> {
        match self {
            Self::CreateNode(command) => command.apply(scene),
            Self::DeleteNode(command) => command.apply(scene),
            Self::UpdateNode(command) => command.apply(scene),
        }
    }

    pub(crate) fn undo(&self, scene: &mut Scene) -> Result<Option<NodeId>, String> {
        match self {
            Self::CreateNode(command) => command.undo(scene),
            Self::DeleteNode(command) => command.undo(scene),
            Self::UpdateNode(command) => command.undo(scene),
        }
    }

    pub(crate) fn target_node(&self) -> NodeId {
        match self {
            Self::CreateNode(command) => command.node_id(),
            Self::DeleteNode(command) => command.node_id(),
            Self::UpdateNode(command) => command.node_id(),
        }
    }

    pub(crate) fn selection_after(&self) -> Option<NodeId> {
        match self {
            Self::CreateNode(command) => Some(command.node_id()),
            Self::DeleteNode(command) => command.selection_after,
            Self::UpdateNode(command) => command.selection_after,
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct CreateNodeCommand {
    record: NodeRecord,
    previous_selected: Option<NodeId>,
}

impl CreateNodeCommand {
    fn spawn_node(
        scene: &mut Scene,
        previous_selected: Option<NodeId>,
        kind: NodeKind,
    ) -> Result<Self, String> {
        let node_id = scene.spawn_node(kind);
        let record = scene
            .node_record(node_id)
            .ok_or_else(|| format!("created node {node_id} is missing from world"))?;
        Ok(Self {
            record,
            previous_selected,
        })
    }

    fn import_mesh(
        scene: &mut Scene,
        previous_selected: Option<NodeId>,
        model: ResourceHandle<ModelMarker>,
        material: ResourceHandle<MaterialMarker>,
    ) -> Result<Self, String> {
        let node_id = scene.spawn_mesh_node(model, material);
        let record = scene
            .node_record(node_id)
            .ok_or_else(|| format!("imported node {node_id} is missing from world"))?;
        Ok(Self {
            record,
            previous_selected,
        })
    }

    fn apply(&self, scene: &mut Scene) -> Result<Option<NodeId>, String> {
        scene.insert_node_record(self.record.clone())?;
        Ok(Some(self.record.id))
    }

    fn undo(&self, scene: &mut Scene) -> Result<Option<NodeId>, String> {
        if !scene.remove_entity(self.record.id) {
            return Err(format!("cannot remove missing node {}", self.record.id));
        }
        Ok(self.previous_selected)
    }

    fn node_id(&self) -> NodeId {
        self.record.id
    }
}

#[derive(Clone, Debug)]
pub(crate) struct DeleteNodeCommand {
    root_id: NodeId,
    records: Vec<NodeRecord>,
    previous_selected: Option<NodeId>,
    previous_active_camera: NodeId,
    pub(crate) selection_after: Option<NodeId>,
    active_camera_after: NodeId,
}

impl DeleteNodeCommand {
    fn capture(
        scene: &mut Scene,
        previous_selected: Option<NodeId>,
        node_id: NodeId,
    ) -> Result<Self, String> {
        let records = scene.subtree_records(node_id);
        if records.is_empty() {
            return Err(format!("cannot delete missing node {node_id}"));
        }
        let removed_camera_count = records
            .iter()
            .filter(|record| record.camera.is_some())
            .count();
        if removed_camera_count >= scene.camera_count() {
            return Err("cannot delete the last remaining camera".to_string());
        }

        let previous_active_camera = scene.active_camera();
        let fallback_selection = scene
            .parent_of(node_id)
            .filter(|parent| scene.contains_entity(*parent));

        let _removed = scene.remove_entity_recursive(node_id);
        let active_camera_after = scene.active_camera();
        let selection_after = fallback_selection.or(Some(active_camera_after));

        Ok(Self {
            root_id: node_id,
            records,
            previous_selected,
            previous_active_camera,
            selection_after,
            active_camera_after,
        })
    }

    fn apply(&self, scene: &mut Scene) -> Result<Option<NodeId>, String> {
        let removed = scene.remove_entity_recursive(self.root_id);
        if removed.is_empty() {
            return Err(format!("cannot delete missing node {}", self.root_id));
        }
        if scene.contains_entity(self.active_camera_after) {
            scene.set_active_camera(self.active_camera_after);
        }
        Ok(self.selection_after)
    }

    fn undo(&self, scene: &mut Scene) -> Result<Option<NodeId>, String> {
        scene.insert_node_records(&self.records)?;
        scene.set_active_camera(self.previous_active_camera);
        Ok(self.previous_selected)
    }

    fn node_id(&self) -> NodeId {
        self.root_id
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct NodeEditState {
    pub(crate) name: String,
    pub(crate) parent: Option<NodeId>,
    pub(crate) transform: Transform,
}

impl NodeEditState {
    fn capture(scene: &Scene, node_id: NodeId) -> Result<Self, String> {
        let node = scene
            .find_node(node_id)
            .ok_or_else(|| format!("missing node {node_id}"))?;
        Ok(Self {
            name: node.name.clone(),
            parent: node.parent,
            transform: node.transform,
        })
    }
}

#[derive(Clone, Debug)]
pub(crate) struct UpdateNodeCommand {
    node_id: NodeId,
    before: NodeEditState,
    after: NodeEditState,
    selection_before: Option<NodeId>,
    pub(crate) selection_after: Option<NodeId>,
}

impl UpdateNodeCommand {
    pub(crate) fn new(
        node_id: NodeId,
        before: NodeEditState,
        after: NodeEditState,
        selection_before: Option<NodeId>,
        selection_after: Option<NodeId>,
    ) -> Option<Self> {
        (before != after).then_some(Self {
            node_id,
            before,
            after,
            selection_before,
            selection_after,
        })
    }

    fn capture(
        scene: &mut Scene,
        selected: Option<NodeId>,
        node_id: NodeId,
        after: NodeEditState,
    ) -> Result<Option<Self>, String> {
        let after = normalize_edit_state(after)?;
        let before = NodeEditState::capture(scene, node_id)?;
        if before == after {
            return Ok(None);
        }
        Self::apply_state(scene, node_id, &after)?;
        Ok(Some(Self {
            node_id,
            before,
            after,
            selection_before: selected,
            selection_after: Some(node_id),
        }))
    }

    fn capture_name(
        scene: &mut Scene,
        selected: Option<NodeId>,
        node_id: NodeId,
        name: String,
    ) -> Result<Option<Self>, String> {
        let mut after = NodeEditState::capture(scene, node_id)?;
        after.name = name;
        Self::capture(scene, selected, node_id, after)
    }

    fn capture_parent(
        scene: &mut Scene,
        selected: Option<NodeId>,
        node_id: NodeId,
        parent: Option<NodeId>,
    ) -> Result<Option<Self>, String> {
        let mut after = NodeEditState::capture(scene, node_id)?;
        after.parent = parent;
        Self::capture(scene, selected, node_id, after)
    }

    fn capture_transform(
        scene: &mut Scene,
        selected: Option<NodeId>,
        node_id: NodeId,
        transform: Transform,
    ) -> Result<Option<Self>, String> {
        let mut after = NodeEditState::capture(scene, node_id)?;
        after.transform = transform;
        Self::capture(scene, selected, node_id, after)
    }

    fn apply(&self, scene: &mut Scene) -> Result<Option<NodeId>, String> {
        Self::apply_state(scene, self.node_id, &self.after)?;
        Ok(self.selection_after)
    }

    fn undo(&self, scene: &mut Scene) -> Result<Option<NodeId>, String> {
        Self::apply_state(scene, self.node_id, &self.before)?;
        Ok(self.selection_before)
    }

    fn apply_state(
        scene: &mut Scene,
        node_id: NodeId,
        state: &NodeEditState,
    ) -> Result<(), String> {
        if scene.find_node(node_id).is_none() {
            return Err(format!("missing node {node_id}"));
        }
        let _ = scene.set_parent_checked(node_id, state.parent)?;
        scene.rename_node(node_id, state.name.clone())?;
        let _ = scene.update_transform(node_id, state.transform)?;
        Ok(())
    }

    fn node_id(&self) -> NodeId {
        self.node_id
    }
}

fn normalize_edit_state(mut state: NodeEditState) -> Result<NodeEditState, String> {
    state.name = state.name.trim().to_string();
    if state.name.is_empty() {
        return Err("node name cannot be empty".to_string());
    }
    Ok(state)
}
