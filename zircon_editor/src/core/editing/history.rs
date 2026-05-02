//! Undoable command stack for scene edits.

use zircon_runtime::scene::{NodeId, Scene};
use zircon_runtime_interface::math::Transform;

use crate::core::editing::command::{EditorCommand, NodeEditState, UpdateNodeCommand};

const HISTORY_LIMIT: usize = 128;

#[derive(Clone, Copy, Debug)]
struct GizmoDragState {
    node_id: NodeId,
    before: Transform,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct EditorHistory {
    undo_stack: Vec<EditorCommand>,
    redo_stack: Vec<EditorCommand>,
    drag_origin: Option<GizmoDragState>,
}

impl EditorHistory {
    pub(crate) fn push(&mut self, command: EditorCommand) {
        self.undo_stack.push(command);
        if self.undo_stack.len() > HISTORY_LIMIT {
            self.undo_stack.remove(0);
        }
        self.redo_stack.clear();
    }

    pub(crate) fn begin_drag(&mut self, scene: &Scene, selected: Option<NodeId>) {
        if self.drag_origin.is_some() {
            return;
        }

        let Some(node_id) = selected else {
            return;
        };
        let Some(node) = scene.find_node(node_id) else {
            return;
        };

        self.drag_origin = Some(GizmoDragState {
            node_id,
            before: node.transform,
        });
    }

    pub(crate) fn end_drag(
        &mut self,
        scene: &Scene,
        selected: Option<NodeId>,
    ) -> Result<Option<EditorCommand>, String> {
        let Some(origin) = self.drag_origin.take() else {
            return Ok(None);
        };
        let Some(current) = scene.find_node(origin.node_id) else {
            return Err(format!(
                "cannot finish gizmo drag for missing node {}",
                origin.node_id
            ));
        };

        Ok(UpdateNodeCommand::new(
            origin.node_id,
            NodeEditState {
                name: current.name.clone(),
                parent: current.parent,
                transform: origin.before,
            },
            NodeEditState {
                name: current.name.clone(),
                parent: current.parent,
                transform: current.transform,
            },
            selected,
            Some(origin.node_id),
        )
        .map(EditorCommand::UpdateNode))
    }

    pub(crate) fn undo(
        &mut self,
        scene: &mut Scene,
        selected: &mut Option<NodeId>,
    ) -> Result<bool, String> {
        self.drag_origin = None;
        let Some(command) = self.undo_stack.pop() else {
            return Ok(false);
        };
        *selected = command.undo(scene)?;
        self.redo_stack.push(command);
        Ok(true)
    }

    pub(crate) fn redo(
        &mut self,
        scene: &mut Scene,
        selected: &mut Option<NodeId>,
    ) -> Result<bool, String> {
        self.drag_origin = None;
        let Some(command) = self.redo_stack.pop() else {
            return Ok(false);
        };
        *selected = command.apply(scene)?;
        self.undo_stack.push(command);
        Ok(true)
    }

    pub(crate) fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
        self.drag_origin = None;
    }

    pub(crate) fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub(crate) fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }
}
