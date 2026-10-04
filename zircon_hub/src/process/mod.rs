mod child_supervisor;
mod editor_child_reaper;
pub(crate) mod editor_child_receipt;
pub(crate) mod editor_focus;
pub(crate) mod editor_handshake;
mod editor_launch;
mod folder_picker;
mod open_folder;

pub(crate) use child_supervisor::SupervisedChild;
pub(crate) use editor_child_reaper::EditorChildReaper;
pub(crate) use editor_launch::launch_editor;
pub use editor_launch::{
    staged_editor_executable, staged_editor_executable_exists, EditorLaunchCommand,
    EditorLaunchRequest,
};
pub use folder_picker::{pick_folder, FolderPickerRequest};
pub use open_folder::{open_folder, OpenFolderCommand};
