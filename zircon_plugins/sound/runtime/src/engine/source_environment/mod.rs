//! 这一链路组合监听器、音量区域、HRTF 与卷积的源级投影；当前仅内部算法引用，需与实际输出后端的接入状态分开理解。
mod apply;
mod constants;
mod convolution;
mod hrtf;
mod listener;
mod spatial;
mod volume;

// TODO: [CR-SOUND-AUDIT-0003] 确认此源级空间/HRTF/区域处理何时接入实时输出；目前仅见内部导出，Kira 声源直接播放静态素材，未见该入口的生产调用。
pub(crate) use apply::apply_source_environment;
pub(crate) use hrtf::hrtf_tail_pending_for_source;
pub(crate) use listener::active_listener_for;
