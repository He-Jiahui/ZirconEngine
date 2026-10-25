//! Texture inspector facts over the original asset; source admission and the device stay owned
//! by the existing project/document/preview hosts.

mod source;

pub use source::{TextureSource, TextureSourceAdapter};

#[cfg(test)]
mod tests;
