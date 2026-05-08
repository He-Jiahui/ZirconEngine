mod builder;
mod groups;

pub use builder::{PluginGroup, PluginGroupBuilder, PluginGroupError, ResolvedPluginGroup};
pub use groups::{DefaultPlugins, DevPlugins, HeadlessPlugins, MinimalPlugins};

#[cfg(test)]
mod tests;
