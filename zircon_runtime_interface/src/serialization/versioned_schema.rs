use super::{MigrationChain, SchemaId};

/// Declares the current version and complete forward migration chain for a payload.
/// 迁移链须覆盖从 0 到当前版本的每一步；读取当前版本也会校验整条链。
pub trait VersionedSchema: Sized {
    const SCHEMA: SchemaId;
    const VERSION: u32;

    fn migrations() -> &'static MigrationChain<Self>;
}
