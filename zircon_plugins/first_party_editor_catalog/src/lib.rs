//! Linked first-party editor provider catalog.
//!
//! `zircon_app` owns project/profile composition. This crate is the narrow
//! fan-out from selected package ids to editor-side registration providers,
//! keeping `zircon_editor` independent from concrete plugin crates.

mod catalog;

#[cfg(test)]
mod tests;

pub use catalog::{
    first_party_editor_plugin_registrations_for_manifest,
    first_party_registration_for_editor_plugin,
};
