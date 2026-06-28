//! Shared runtime font database used by text shaping and raster paths.

mod asset_registration;
mod coverage;
mod database;
mod default_families;
#[cfg(test)]
mod test_font_fixtures;

pub(crate) use database::FontDatabase;
#[cfg(test)]
pub(crate) use default_families::default_runtime_font_families;
