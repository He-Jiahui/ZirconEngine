use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VmPluginGarbageCollectionMode {
    Disabled,
    Cooperative,
    #[default]
    BackendManaged,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct VmPluginGarbageCollectionPolicy {
    #[serde(default)]
    pub mode: VmPluginGarbageCollectionMode,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub interval_frames: Option<u64>,
}

impl VmPluginGarbageCollectionPolicy {
    pub fn backend_managed() -> Self {
        Self::default()
    }

    pub fn cooperative(interval_frames: impl Into<Option<u64>>) -> Self {
        Self {
            mode: VmPluginGarbageCollectionMode::Cooperative,
            interval_frames: interval_frames.into(),
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if matches!(self.mode, VmPluginGarbageCollectionMode::Disabled)
            && self.interval_frames.is_some()
        {
            return Err("disabled garbage collection cannot set interval_frames".to_string());
        }
        if self.interval_frames == Some(0) {
            return Err("garbage collection interval_frames must be greater than zero".to_string());
        }
        Ok(())
    }
}
