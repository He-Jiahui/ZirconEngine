//! PCM/HRTF处理和控制线程标量投影各有模块；projection把空间增益和声像同步到实际Kira句柄。
#[path = "apply.rs"]
mod apply;
#[path = "constants.rs"]
mod constants;
#[path = "convolution.rs"]
mod convolution;
#[path = "hrtf/mod.rs"]
mod hrtf;
#[path = "listener.rs"]
mod listener;
#[path = "projection.rs"]
mod projection;
#[path = "spatial/mod.rs"]
mod spatial;
#[path = "volume/mod.rs"]
mod volume;

// TODO: [CR-SOUND-AUDIT-0003] PCM级HRTF和区域处理仍需独立接入；projection当前只连接实时空间增益和声像，不构成该完整处理链的验收。
pub(crate) use apply::apply_source_environment;
pub(crate) use hrtf::hrtf_tail_pending_for_source;
pub(crate) use listener::active_listener_for;
