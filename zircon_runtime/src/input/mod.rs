//! Runtime input facade and protocol types.

mod module;
mod runtime;

pub use crate::core::framework::input::{InputButton, InputEvent, InputEventRecord, InputSnapshot};
pub use module::{
    module_descriptor, InputConfig, InputModule, INPUT_DRIVER_NAME, INPUT_MANAGER_NAME,
    INPUT_MODULE_NAME,
};
pub use runtime::{DefaultInputManager, InputDriver};

#[cfg(test)]
mod tests;
