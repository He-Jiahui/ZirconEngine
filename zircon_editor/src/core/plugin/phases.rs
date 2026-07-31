//! Loading-phase vocabulary for editor-plugin lifecycle scheduling.

/// The point in editor startup at which a plugin may materialize its contributions.
#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub enum EditorPluginLoadingPhase {
    PreWorkbench,
    #[default]
    Default,
    PostWorkbench,
}
