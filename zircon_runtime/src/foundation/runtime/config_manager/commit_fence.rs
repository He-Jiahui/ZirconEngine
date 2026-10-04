mod fence;
mod path_key;
mod registry;

pub(in crate::foundation::runtime) use fence::ConfigCommitFence;

#[cfg(test)]
#[path = "commit_fence/tests/cases.rs"]
mod tests;
