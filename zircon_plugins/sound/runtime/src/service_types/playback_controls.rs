//! 一次性播放控制先轮询完成状态，再更新 Kira 句柄；后端操作成功后才改服务镜像，调用方不可把已结束 ID 当作可复用句柄。
#[path = "playback_controls/gain.rs"]
mod gain;
#[path = "playback_controls/mute.rs"]
mod mute;
#[path = "playback_controls/pause.rs"]
mod pause;
#[path = "playback_controls/seek.rs"]
mod seek;
#[path = "playback_controls/speed.rs"]
mod speed;

pub(crate) use seek::kira_slice_position_for_absolute_frame;
