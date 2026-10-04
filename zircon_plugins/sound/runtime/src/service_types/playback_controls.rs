//! 一次性播放控制先轮询完成状态，再更新 Kira 句柄；后端操作成功后才改服务镜像，调用方不可把已结束 ID 当作可复用句柄。
mod gain;
mod mute;
mod pause;
mod seek;
mod speed;

pub(crate) use seek::kira_slice_position_for_absolute_frame;
