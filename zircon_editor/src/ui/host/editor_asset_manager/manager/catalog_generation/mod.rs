mod build;
mod details;
mod folders;
mod record;
mod update;

#[cfg(test)]
mod tests;

pub(super) use build::build_catalog_generation;
pub(super) use record::record_to_view;
pub(super) use update::{
    update_asset_in_catalog_generation, update_catalog_record_in_catalog_generation,
    update_catalog_records_in_catalog_generation,
};
