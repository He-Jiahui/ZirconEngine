//! Optional extension module registration surfaces absorbed into the runtime layer.

pub mod navigation;
pub mod net;
pub mod particles;
mod registration;
pub mod sound;
pub mod texture;

pub(crate) use registration::module_descriptor_with_driver_and_manager;
