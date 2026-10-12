#[path = "definitions/conflict.rs"]
mod conflict;
#[path = "definitions/merge.rs"]
mod merge;
#[path = "definitions/registration.rs"]
mod registration;
#[path = "definitions/registration_match.rs"]
mod registration_match;

pub(super) use merge::{
    merge_runtime_feature_definitions, product_catalog_feature_registration_count,
};
