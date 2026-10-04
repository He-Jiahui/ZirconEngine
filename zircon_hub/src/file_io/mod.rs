mod anchored_tree;
mod regular_file;
#[cfg(windows)]
pub(crate) mod windows;

pub(crate) use anchored_tree::{
    AnchoredDirectory, AnchoredEntry, TreeRemovalLimits, TreeRemovalUsage,
};
pub(crate) use regular_file::read_bounded_regular;
