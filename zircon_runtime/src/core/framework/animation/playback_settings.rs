use serde::{Deserialize, Serialize};

/// 动画服务的运行开关集合；管理器从配置读取或持久化它，
/// 调用方应在创建逐帧工作前检查总开关及对应播放类别。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AnimationPlaybackSettings {
    pub enabled: bool,
    pub property_tracks: bool,
    pub skeletal_clips: bool,
    pub graphs: bool,
    pub state_machines: bool,
}

impl Default for AnimationPlaybackSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            property_tracks: true,
            skeletal_clips: true,
            graphs: true,
            state_machines: true,
        }
    }
}
