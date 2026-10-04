use crate::platform::PlatformFeatureSelection;

/// 保存一次能力矩阵计算所需的编译期/配置期 feature 选择；它不持有宿主对象。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlatformCapabilityMatrix {
    pub features: PlatformFeatureSelection,
}

impl PlatformCapabilityMatrix {
    /// 用调用方提供的 feature 快照创建可重复计算的矩阵。
    pub const fn new(features: PlatformFeatureSelection) -> Self {
        Self { features }
    }

    /// 捕获当前编译 feature；生成静态目录前仍须显式选择 target 与 target_mode。
    pub fn compiled() -> Self {
        Self::new(PlatformFeatureSelection::from_compiled_features())
    }
}
