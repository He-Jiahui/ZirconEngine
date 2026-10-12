#[path = "build.rs"]
mod build;
#[path = "details.rs"]
mod details;
#[path = "folders.rs"]
mod folders;
#[path = "record.rs"]
mod record;
#[path = "update.rs"]
mod update;

#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;

pub(super) use build::build_catalog_generation;
pub(super) use record::record_to_view;
pub(super) use update::{
    update_asset_in_catalog_generation, update_catalog_record_in_catalog_generation,
    update_catalog_records_in_catalog_generation,
};
