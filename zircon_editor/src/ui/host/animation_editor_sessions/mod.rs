mod editing;
mod history;
mod lifecycle;
mod save;
mod sync;

use crate::core::asset::AssetToolkitOpenRoute;
use crate::core::editor_message::DocumentId;
use crate::ui::animation_editor::AnimationEditorSession;

pub(crate) struct AnimationEditorWorkspaceEntry {
    pub(crate) document: DocumentId,
    pub(crate) route: AssetToolkitOpenRoute,
    pub(crate) disk_source: Vec<u8>,
    pub(crate) session: AnimationEditorSession,
}
