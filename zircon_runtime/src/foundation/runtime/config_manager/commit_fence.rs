#[path = "commit_fence/fence.rs"]
mod fence;
#[path = "commit_fence/path_key.rs"]
mod path_key;
#[path = "commit_fence/registry.rs"]
mod registry;

pub(in crate::foundation::runtime) use fence::ConfigCommitFence;

#[cfg(test)]
#[path = "commit_fence/tests/cases.rs"]
mod tests;
