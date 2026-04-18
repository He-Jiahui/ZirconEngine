use crate::{AssetReference, AssetUri};

pub(super) fn builtin_reference(locator: &str) -> AssetReference {
    AssetReference::from_locator(AssetUri::parse(locator).expect("builtin asset reference"))
}
