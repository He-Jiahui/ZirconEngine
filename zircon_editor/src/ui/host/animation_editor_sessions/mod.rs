mod editing;
mod lifecycle;
mod sync;

use std::path::PathBuf;

use crate::ui::animation_editor::AnimationEditorSession;

pub(crate) struct AnimationEditorWorkspaceEntry {
    pub(crate) source_path: PathBuf,
    pub(crate) session: AnimationEditorSession,
}
