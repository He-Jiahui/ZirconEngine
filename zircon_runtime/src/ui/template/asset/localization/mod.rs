mod collect;
mod resolve;

pub use collect::{collect_document_localization_report, validate_document_localization};
pub use resolve::{
    localization_table_keys_from_toml_str, validate_localization_report_against_catalog,
    UiLocalizationTableCatalog,
};
