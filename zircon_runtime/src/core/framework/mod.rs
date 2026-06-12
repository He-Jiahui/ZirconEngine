//! Shared framework contracts and protocol data shared across runtime modules.

pub mod ai;
pub mod animation;
pub mod asset;
pub mod bridge;
pub mod camera_controller;
pub mod channel;
pub mod error;
pub mod events;
pub mod foundation;
pub mod gizmos;
pub mod input;
pub mod navigation;
pub mod net;
pub mod physics;
pub mod picking;
pub mod render;
pub mod scene;
pub mod script;
pub mod sound;
pub mod state;
pub mod tasks;
pub mod time;
pub mod ui;
pub mod window;

#[cfg(test)]
mod tests;
