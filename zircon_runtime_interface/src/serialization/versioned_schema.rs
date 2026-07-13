use super::{MigrationChain, SchemaId};

/// Declares the current version and complete forward migration chain for a payload.
pub trait VersionedSchema: Sized {
    const SCHEMA: SchemaId;
    const VERSION: u32;

    fn migrations() -> &'static MigrationChain<Self>;
}
