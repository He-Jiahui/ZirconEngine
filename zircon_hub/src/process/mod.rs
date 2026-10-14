#[path = "child_supervisor.rs"]
mod child_supervisor;
#[path = "editor_child_reaper.rs"]
mod editor_child_reaper;
#[path = "editor_child_receipt.rs"]
pub(crate) mod editor_child_receipt;
#[path = "editor_focus/mod.rs"]
pub(crate) mod editor_focus;
#[path = "editor_handshake/mod.rs"]
pub(crate) mod editor_handshake;
#[path = "editor_launch.rs"]
mod editor_launch;
#[path = "folder_picker.rs"]
mod folder_picker;
#[path = "open_folder.rs"]
mod open_folder;

pub(crate) use child_supervisor::SupervisedChild;
pub(crate) use editor_child_reaper::EditorChildReaper;
pub(crate) use editor_launch::launch_editor;
pub use editor_launch::{
    EditorLaunchCommand, EditorLaunchRequest, staged_editor_executable,
    staged_editor_executable_exists,
};
pub use folder_picker::{FolderPickerRequest, pick_folder};
pub use open_folder::{OpenFolderCommand, open_folder};
