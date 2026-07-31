mod diagnostics;
mod manifests;
mod projection;
mod registrations;

pub use projection::NativePluginLoadProjection;

#[cfg(test)]
mod tests;

use std::sync::OnceLock;

use super::{LoadedNativePlugin, NativePluginCandidate};

#[derive(Default)]
pub struct NativePluginLoadReport {
    pub(in crate::plugin::native_plugin_loader) discovered: Vec<NativePluginCandidate>,
    pub(in crate::plugin::native_plugin_loader) loaded: Vec<LoadedNativePlugin>,
    pub(in crate::plugin::native_plugin_loader) diagnostics: Vec<String>,
    pub(super) projection: OnceLock<NativePluginLoadProjection>,
}

impl std::fmt::Debug for NativePluginLoadReport {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("NativePluginLoadReport")
            .field("discovered", &self.discovered)
            .field("loaded", &self.loaded)
            .field("diagnostics", &self.diagnostics)
            .finish_non_exhaustive()
    }
}

impl NativePluginLoadReport {
    pub(in crate::plugin::native_plugin_loader) fn diagnostic_only(
        diagnostic: impl Into<String>,
    ) -> Self {
        Self {
            diagnostics: vec![diagnostic.into()],
            ..Self::default()
        }
    }

    pub(in crate::plugin::native_plugin_loader) fn from_discovered(
        discovered: Vec<NativePluginCandidate>,
    ) -> Self {
        Self {
            discovered,
            ..Self::default()
        }
    }

    pub(in crate::plugin::native_plugin_loader) fn from_discovery(
        discovered: Vec<NativePluginCandidate>,
        diagnostics: Vec<String>,
    ) -> Self {
        Self {
            discovered,
            diagnostics,
            ..Self::default()
        }
    }

    pub(in crate::plugin::native_plugin_loader) fn from_loaded(
        loaded: Vec<LoadedNativePlugin>,
    ) -> Self {
        Self {
            loaded,
            ..Self::default()
        }
    }

    pub(in crate::plugin::native_plugin_loader) fn take_discovered(
        &mut self,
    ) -> Vec<NativePluginCandidate> {
        self.invalidate_projection();
        std::mem::take(&mut self.discovered)
    }

    pub(in crate::plugin::native_plugin_loader) fn try_into_discovered(
        self,
    ) -> Result<Vec<NativePluginCandidate>, Self> {
        if self.loaded.is_empty() {
            Ok(self.discovered)
        } else {
            Err(self)
        }
    }

    pub(in crate::plugin::native_plugin_loader) fn restore_discovered(
        &mut self,
        discovered: Vec<NativePluginCandidate>,
    ) {
        self.invalidate_projection();
        self.discovered = discovered;
    }

    pub(in crate::plugin::native_plugin_loader) fn take_loaded(
        &mut self,
    ) -> Vec<LoadedNativePlugin> {
        self.invalidate_projection();
        std::mem::take(&mut self.loaded)
    }

    pub(in crate::plugin::native_plugin_loader) fn push_loaded(
        &mut self,
        loaded: LoadedNativePlugin,
    ) {
        self.invalidate_projection();
        self.loaded.push(loaded);
    }

    pub(in crate::plugin::native_plugin_loader) fn push_diagnostic(
        &mut self,
        diagnostic: impl Into<String>,
    ) {
        self.invalidate_projection();
        self.diagnostics.push(diagnostic.into());
    }

    pub fn discovered(&self) -> &[NativePluginCandidate] {
        &self.discovered
    }

    pub fn loaded(&self) -> &[LoadedNativePlugin] {
        &self.loaded
    }

    pub fn diagnostics(&self) -> &[String] {
        &self.diagnostics
    }

    pub(crate) fn into_loaded(self) -> Vec<LoadedNativePlugin> {
        self.loaded
    }

    pub fn has_failures(&self) -> bool {
        !self.diagnostics.is_empty()
    }

    /// Freezes the derived manifest and diagnostic indexes on first use. Report mutations must
    /// use this owner's controlled APIs so a later mutation invalidates the frozen generation.
    pub fn projection(&self) -> &NativePluginLoadProjection {
        self.projection
            .get_or_init(|| NativePluginLoadProjection::new(self))
    }

    fn invalidate_projection(&mut self) {
        self.projection.take();
    }
}
