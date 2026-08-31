//! Runtime-owned asset mutation contracts.
//!
//! Editor and headless callers consume immutable preflight views from this module. Actual source
//! mutation will be committed by the project-asset generation owner in a later transaction slice.

mod delete_preflight;
mod relocation_preflight;

pub use delete_preflight::{
    AssetMutationAsset, AssetMutationDeleteDisposition, AssetMutationDeletePreflight,
};
pub use relocation_preflight::{
    AssetMutationRelocationDisposition, AssetMutationRelocationPreflight,
};

#[cfg(test)]
mod tests;
