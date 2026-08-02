use std::sync::Arc;

use zircon_runtime::builtin::{
    runtime_modules_for_runtime_profile_manifest_with_plugin_registration_reports,
    RuntimeModuleLoadDiagnostic,
};
use zircon_runtime::core::framework::project::{ProjectPluginManifest, RuntimeProfileId};
use zircon_runtime::engine_module::EngineModule;
use zircon_runtime::plugin::RuntimePluginRegistrationReport;

use super::super::{PluginGroupBuilder, PluginGroupError};

#[derive(Clone, Copy, Debug)]
pub(super) enum BuiltinPluginGroupFeature {
    Ui,
    LogDiagnostics,
}

pub(super) fn resolve_builtin_plugin_group(
    group_name: &'static str,
    profile_id: RuntimeProfileId,
    features: impl IntoIterator<Item = BuiltinPluginGroupFeature>,
) -> Result<PluginGroupBuilder, PluginGroupError> {
    let manifest = ProjectPluginManifest::default();
    let report = runtime_modules_for_runtime_profile_manifest_with_plugin_registration_reports(
        profile_id,
        &manifest,
        std::iter::empty::<&RuntimePluginRegistrationReport>(),
    );
    if let Some(error) = report.diagnostics().iter().find_map(|diagnostic| {
        if let RuntimeModuleLoadDiagnostic::Core(error) = diagnostic {
            Some(error)
        } else {
            None
        }
    }) {
        return Err(PluginGroupError::ModuleOrder {
            group: group_name.to_owned(),
            reason: error.to_string(),
        });
    }

    let mut builder = PluginGroupBuilder::from_modules(group_name, report.modules)?;
    for feature in features {
        if let Some(module) = feature.resolve_module() {
            builder = builder.add_module(module)?;
        }
    }
    Ok(builder)
}

impl BuiltinPluginGroupFeature {
    fn resolve_module(self) -> Option<Arc<dyn EngineModule>> {
        match self {
            Self::Ui => resolve_ui_module(),
            Self::LogDiagnostics => Some(Arc::new(
                zircon_runtime::core::runtime::modules::LogDiagnosticsModule,
            )),
        }
    }
}

fn resolve_ui_module() -> Option<Arc<dyn EngineModule>> {
    #[cfg(feature = "ui")]
    {
        Some(Arc::new(zircon_runtime::ui::UiModule))
    }
    #[cfg(not(feature = "ui"))]
    {
        None
    }
}
