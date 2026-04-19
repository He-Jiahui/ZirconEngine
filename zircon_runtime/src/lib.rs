//! Runtime absorption layer for the built-in high-level engine subsystems.

pub mod asset;
mod builtin;
pub mod extensions;
pub mod foundation;
pub mod graphics;
pub mod input;
pub mod platform;
pub mod scene;
pub mod script;
pub mod ui;

pub use builtin::builtin_runtime_modules;

#[cfg(test)]
mod tests;
