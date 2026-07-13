mod editing;
mod lifecycle;
mod save;
mod sync;

use crate::core::asset::AssetToolkitOpenRoute;
use crate::ui::animation_editor::AnimationEditorSession;

pub(crate) struct AnimationEditorWorkspaceEntry {
    pub(crate) route: AssetToolkitOpenRoute,
    pub(crate) session: AnimationEditorSession,
}
