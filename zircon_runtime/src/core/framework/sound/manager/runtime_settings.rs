use super::super::SoundError;

/// 活动声音服务的全局设置；增益写入后端，空间比例意图是未单独指定比例的声源默认值。
pub trait SoundRuntimeSettingsManager {
    fn global_volume_gain(&self) -> Result<f32, SoundError>;
    fn set_global_volume_gain(&self, gain: f32) -> Result<(), SoundError>;
    fn default_spatial_scale(&self) -> Result<f32, SoundError>;
    // BUG: [CR-FRAMEWORK-SOUND-0003] 当前实现只保存默认空间比例；实时声源路径没有读取配置，调用成功后空间听感不变。
    fn set_default_spatial_scale(&self, scale: f32) -> Result<(), SoundError>;
}
