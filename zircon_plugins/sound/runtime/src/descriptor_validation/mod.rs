//! 描述符校验保护作者数据与状态引用的有效性；后端是否支持这些合法描述仍由实际播放及图编译入口判定。
#[path = "coordinates.rs"]
mod coordinates;
#[path = "external_source.rs"]
pub(crate) mod external_source;
#[path = "hrtf.rs"]
pub(crate) mod hrtf;
#[path = "listener.rs"]
pub(crate) mod listener;
#[path = "source/mod.rs"]
pub(crate) mod source;
#[path = "volume.rs"]
pub(crate) mod volume;
