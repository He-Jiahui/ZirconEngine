use crate::asset::{AssetEvent, SceneAsset};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// 事件尚未进入准备任务时的拒收原因；后续编译或世界应用错误由失败报告承载。
pub enum DynamicSceneAssetReloadSkipReason {
    Removed,
    ReloadFailed,
    MissingLocator,
    StaleRevision,
    CapacityExceeded,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DynamicSceneAssetReloadSkippedEvent {
    event: AssetEvent<SceneAsset>,
    reason: DynamicSceneAssetReloadSkipReason,
}

impl DynamicSceneAssetReloadSkippedEvent {
    pub fn new(event: AssetEvent<SceneAsset>, reason: DynamicSceneAssetReloadSkipReason) -> Self {
        Self { event, reason }
    }

    pub fn event(&self) -> &AssetEvent<SceneAsset> {
        &self.event
    }

    pub fn reason(&self) -> DynamicSceneAssetReloadSkipReason {
        self.reason
    }
}
