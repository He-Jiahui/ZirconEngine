//! Input module wired into the core runtime with a stable façade.

mod module;
mod runtime;

pub use module::{
    module_descriptor, InputConfig, INPUT_DRIVER_NAME, INPUT_MANAGER_NAME, INPUT_MODULE_NAME,
};
pub use runtime::{DefaultInputManager, InputDriver};

#[cfg(test)]
mod tests;
