use std::sync::Arc;

use zircon_runtime::builtin::runtime_modules_for_runtime_profile_manifest_with_plugin_registration_reports;
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
    let composition =
        runtime_modules_for_runtime_profile_manifest_with_plugin_registration_reports(
            profile_id,
            &manifest,
            std::iter::empty::<&RuntimePluginRegistrationReport>(),
        )
        .map_err(|rejection| PluginGroupError::ModuleOrder {
            group: group_name.to_owned(),
            reason: rejection.to_string(),
        })?;

    let mut builder =
        PluginGroupBuilder::from_modules(group_name, composition.modules().iter().cloned())?;
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
