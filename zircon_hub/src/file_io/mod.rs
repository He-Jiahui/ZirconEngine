#[path = "anchored_tree.rs"]
mod anchored_tree;
#[path = "regular_file.rs"]
mod regular_file;
#[cfg(windows)]
#[path = "windows.rs"]
pub(crate) mod windows;

pub(crate) use anchored_tree::{
    AnchoredDirectory, AnchoredEntry, TreeRemovalLimits, TreeRemovalUsage,
};
pub(crate) use regular_file::read_bounded_regular;
