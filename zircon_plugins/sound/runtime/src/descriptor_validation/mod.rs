//! 描述符校验保护作者数据与状态引用的有效性；后端是否支持这些合法描述仍由实际播放及图编译入口判定。
mod coordinates;
pub(crate) mod external_source;
pub(crate) mod hrtf;
pub(crate) mod listener;
pub(crate) mod source;
pub(crate) mod volume;
