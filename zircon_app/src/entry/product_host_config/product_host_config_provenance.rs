use super::{ProductConfigSource, ProductConfigSourceSet};

/// Per-field authority receipt retained beside a resolved product-host configuration.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProductHostConfigProvenance {
    pub(super) profile: ProductConfigSource,
    pub(super) runtime_profile: ProductConfigSource,
    pub(super) target_mode: ProductConfigSource,
    pub(super) platform_target: ProductConfigSource,
    pub(super) project_plugins: ProductConfigSourceSet,
    pub(super) export_profile: ProductConfigSource,
    pub(super) render_profile: ProductConfigSource,
    pub(super) window_descriptor: ProductConfigSource,
    pub(super) editor_enabled_subsystems: ProductConfigSource,
    pub(super) editor_runtime_sandbox: ProductConfigSource,
}

impl ProductHostConfigProvenance {
    pub const fn profile(&self) -> ProductConfigSource {
        self.profile
    }

    pub const fn runtime_profile(&self) -> ProductConfigSource {
        self.runtime_profile
    }

    pub const fn target_mode(&self) -> ProductConfigSource {
        self.target_mode
    }

    pub const fn platform_target(&self) -> ProductConfigSource {
        self.platform_target
    }

    pub const fn project_plugins(&self) -> ProductConfigSourceSet {
        self.project_plugins
    }

    pub const fn export_profile(&self) -> ProductConfigSource {
        self.export_profile
    }

    pub const fn render_profile(&self) -> ProductConfigSource {
        self.render_profile
    }

    pub const fn window_descriptor(&self) -> ProductConfigSource {
        self.window_descriptor
    }

    pub const fn editor_enabled_subsystems(&self) -> ProductConfigSource {
        self.editor_enabled_subsystems
    }

    pub const fn editor_runtime_sandbox(&self) -> ProductConfigSource {
        self.editor_runtime_sandbox
    }
}
