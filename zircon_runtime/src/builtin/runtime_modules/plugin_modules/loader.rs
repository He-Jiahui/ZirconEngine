use std::sync::Arc;

use crate::engine_module::EngineModule;
#[cfg(feature = "ui")]
use crate::ui;

use super::super::ids::RuntimePluginId;

pub(in crate::builtin::runtime_modules) fn module_for_plugin(
    id: RuntimePluginId,
) -> Option<Arc<dyn EngineModule>> {
    if id != RuntimePluginId::Ui {
        return None;
    }

    #[cfg(feature = "ui")]
    {
        Some(Arc::new(ui::UiModule))
    }
    #[cfg(not(feature = "ui"))]
    {
        None
    }
}
