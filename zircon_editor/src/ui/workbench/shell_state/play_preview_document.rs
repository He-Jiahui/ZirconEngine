use crate::ui::host::EditorManager;
use crate::ui::workbench::layout::DocumentNode;
use crate::ui::workbench::view::{ViewDescriptorId, ViewInstanceId};

const SCENE_VIEW_DESCRIPTOR_ID: &str = "editor.scene";

pub(super) fn capture(
    manager: &EditorManager,
    game: Option<&ViewInstanceId>,
) -> Option<ViewInstanceId> {
    let layout = manager.current_layout();
    let scenes = manager
        .current_view_instances()
        .into_iter()
        .filter(|view| view.descriptor_id == ViewDescriptorId::new(SCENE_VIEW_DESCRIPTOR_ID))
        .map(|view| view.instance_id)
        .collect::<Vec<_>>();
    if let Some(game) = game {
        layout
            .activity_windows
            .values()
            .find_map(|window| previous_in_game_leaf(&window.content_workspace, game, &scenes))
    } else {
        // A newly opened Game view uses the existing document root. Do not guess a
        // target leaf when a split workspace has no existing Game instance.
        match &layout.active_activity_window()?.content_workspace {
            DocumentNode::Tabs(leaf) => leaf.active_tab.clone(),
            DocumentNode::SplitNode { .. } => None,
        }
    }
}

fn previous_in_game_leaf(
    node: &DocumentNode,
    game: &ViewInstanceId,
    scenes: &[ViewInstanceId],
) -> Option<ViewInstanceId> {
    match node {
        DocumentNode::Tabs(leaf) if leaf.tabs.contains(game) => leaf
            .active_tab
            .as_ref()
            .filter(|active| *active != game)
            .cloned()
            .or_else(|| {
                scenes
                    .iter()
                    .find(|scene| leaf.tabs.contains(scene))
                    .cloned()
            }),
        DocumentNode::Tabs(_) => None,
        DocumentNode::SplitNode { first, second, .. } => previous_in_game_leaf(first, game, scenes)
            .or_else(|| previous_in_game_leaf(second, game, scenes)),
    }
}

#[cfg(test)]
#[path = "tests/play_preview_document.rs"]
mod tests;
