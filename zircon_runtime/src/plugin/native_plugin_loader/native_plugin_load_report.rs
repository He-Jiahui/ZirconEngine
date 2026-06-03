mod diagnostics;
mod manifests;
mod registrations;

#[cfg(test)]
mod tests;

use super::{LoadedNativePlugin, NativePluginCandidate};

#[derive(Debug, Default)]
pub struct NativePluginLoadReport {
    pub discovered: Vec<NativePluginCandidate>,
    pub loaded: Vec<LoadedNativePlugin>,
    pub diagnostics: Vec<String>,
}

impl NativePluginLoadReport {
    pub fn has_failures(&self) -> bool {
        !self.diagnostics.is_empty()
    }
}
