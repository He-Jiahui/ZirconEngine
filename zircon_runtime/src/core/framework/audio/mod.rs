//! Backend-neutral audio format data shared by assets and optional sound services.

mod channel_layout;

pub use channel_layout::{AudioChannelLayout, AudioSpeakerChannel};

#[cfg(test)]
mod tests;
