use serde::{Deserialize, Serialize};

use crate::asset::ASSET_MODULE_NAME;
use crate::core::framework::foundation::FOUNDATION_MODULE_NAME;
use crate::core::framework::input::INPUT_MODULE_NAME;
use crate::core::framework::platform::PLATFORM_MODULE_NAME;
use crate::core::framework::render::GRAPHICS_MODULE_NAME;
use crate::core::framework::scene::SCENE_MODULE_NAME;
use crate::core::runtime::modules::{
    DIAGNOSTICS_CORE_MODULE_NAME, FRAME_COUNT_MODULE_NAME, LOG_MODULE_NAME, TASKS_MODULE_NAME,
    TIME_MODULE_NAME,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BuiltinRuntimeModuleId {
    Foundation,
    Log,
    Tasks,
    Time,
    FrameCount,
    DiagnosticsCore,
    Platform,
    Input,
    Asset,
    Scene,
    #[cfg(feature = "graphics")]
    Graphics,
    #[cfg(feature = "script")]
    Script,
}

impl BuiltinRuntimeModuleId {
    pub fn module_name(self) -> &'static str {
        match self {
            Self::Foundation => FOUNDATION_MODULE_NAME,
            Self::Log => LOG_MODULE_NAME,
            Self::Tasks => TASKS_MODULE_NAME,
            Self::Time => TIME_MODULE_NAME,
            Self::FrameCount => FRAME_COUNT_MODULE_NAME,
            Self::DiagnosticsCore => DIAGNOSTICS_CORE_MODULE_NAME,
            Self::Platform => PLATFORM_MODULE_NAME,
            Self::Input => INPUT_MODULE_NAME,
            Self::Asset => ASSET_MODULE_NAME,
            Self::Scene => SCENE_MODULE_NAME,
            #[cfg(feature = "graphics")]
            Self::Graphics => GRAPHICS_MODULE_NAME,
            #[cfg(feature = "script")]
            Self::Script => crate::script::SCRIPT_MODULE_NAME,
        }
    }

    pub fn for_module_name(module_name: &str) -> Option<Self> {
        match module_name {
            FOUNDATION_MODULE_NAME => Some(Self::Foundation),
            LOG_MODULE_NAME => Some(Self::Log),
            TASKS_MODULE_NAME => Some(Self::Tasks),
            TIME_MODULE_NAME => Some(Self::Time),
            FRAME_COUNT_MODULE_NAME => Some(Self::FrameCount),
            DIAGNOSTICS_CORE_MODULE_NAME => Some(Self::DiagnosticsCore),
            PLATFORM_MODULE_NAME => Some(Self::Platform),
            INPUT_MODULE_NAME => Some(Self::Input),
            ASSET_MODULE_NAME => Some(Self::Asset),
            SCENE_MODULE_NAME => Some(Self::Scene),
            #[cfg(feature = "graphics")]
            GRAPHICS_MODULE_NAME => Some(Self::Graphics),
            #[cfg(feature = "script")]
            crate::script::SCRIPT_MODULE_NAME => Some(Self::Script),
            _ => None,
        }
    }
}
