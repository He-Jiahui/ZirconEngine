//! Shared framework contracts and protocol data shared across runtime modules.

#[cfg(feature = "ai-contracts")]
pub mod ai;
pub mod animation;
pub mod asset;
pub mod audio;
pub mod bridge;
pub mod camera_controller;
pub mod channel;
pub mod events;
pub mod foundation;
pub mod gizmos;
pub mod input;
pub mod navigation;
#[cfg(feature = "net-contracts")]
pub mod net;
#[cfg(feature = "physics-contracts")]
pub mod physics;
pub mod picking;
pub mod platform;
pub mod project;
pub mod render;
pub mod scene;
pub mod script;
#[cfg(feature = "sound-contracts")]
pub mod sound;
pub mod state;
pub mod tasks;
pub mod text;
pub mod time;
pub mod ui;
pub mod window;

#[cfg(test)]
mod tests;
