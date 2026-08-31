mod binary;
mod budget;
mod error;
mod format;
mod load;
mod loaded;
mod migration;
mod payload_header;
mod schema_id;
mod text;
mod versioned_schema;
mod write;
mod write_error;

pub use budget::SerializationBudget;
pub use error::LoadError;
pub use format::Format;
pub use load::{load_versioned, load_versioned_envelope, load_versioned_legacy_schema_zero};
pub use loaded::Loaded;
pub use migration::{MigrateError, MigrationChain, MigrationStep};
pub use payload_header::PayloadHeader;
pub use schema_id::{SchemaId, SchemaIdError, MAX_SCHEMA_ID_BYTES};
pub use versioned_schema::VersionedSchema;
pub use write::{
    write_canonical_text_to, write_versioned, write_versioned_text, write_versioned_text_to,
};
pub use write_error::{CanonicalTextWriteError, WriteError};

#[cfg(test)]
mod tests;
