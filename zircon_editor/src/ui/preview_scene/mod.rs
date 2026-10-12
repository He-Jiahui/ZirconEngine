//! Shared, editor-side preview-scene lifecycle contract.

#[path = "preview_scene.rs"]
mod preview_scene;
#[path = "preview_subject.rs"]
mod preview_subject;

pub use preview_scene::{
    PreviewPlayback, PreviewScene, PreviewSceneBackend, PreviewSceneError, SharedPreviewScene,
};
pub use preview_subject::PreviewSubject;

#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;
