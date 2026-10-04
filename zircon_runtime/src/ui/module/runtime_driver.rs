use std::sync::atomic::{AtomicBool, Ordering};

use serde::{Deserialize, Serialize};

use crate::core::{CoreError, CoreHandle, CoreResult};

use super::UI_RUNTIME_DRIVER_NAME;

/// Configure before activation; changes take effect on the next activation.
pub const UI_CONFIG_KEY: &str = "ui";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiConfig {
    pub enabled: bool,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self { enabled: true }
    }
}

/// Activation-scoped project admission. Surface ownership remains with its host.
#[derive(Debug)]
pub struct UiRuntimeDriver {
    config: UiConfig,
    admission_open: AtomicBool,
}

impl Default for UiRuntimeDriver {
    fn default() -> Self {
        Self::new(UiConfig::default())
    }
}

impl UiRuntimeDriver {
    fn new(config: UiConfig) -> Self {
        Self {
            config,
            admission_open: AtomicBool::new(true),
        }
    }

    pub(super) fn from_core(core: &CoreHandle) -> CoreResult<Self> {
        let config = match core.load_config_value(UI_CONFIG_KEY) {
            Some(value) => serde_json::from_value(value).map_err(|error| {
                CoreError::ConfigParse(UI_CONFIG_KEY.to_owned(), error.to_string())
            })?,
            None => UiConfig::default(),
        };
        Ok(Self::new(config))
    }

    pub fn config(&self) -> &UiConfig {
        &self.config
    }

    /// Checks admission before asset/font/layout work. Already admitted work is not drained here.
    pub fn admit_project(&self) -> CoreResult<bool> {
        if !self.admission_open.load(Ordering::Acquire) {
            return Err(CoreError::ServiceUnavailable(
                UI_RUNTIME_DRIVER_NAME.to_owned(),
            ));
        }
        Ok(self.config.enabled)
    }

    pub(super) fn close_admission(&self) {
        self.admission_open.store(false, Ordering::Release);
    }
}
