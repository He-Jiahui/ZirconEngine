//! 带版本的文本/二进制载荷、迁移和边界检查的统一接口。
//!
//! 调用方通过 [`VersionedSchema`] 声明持久载荷身份及迁移链，再以 [`load_versioned`]
//! 等入口读取；模块内的格式实现负责验证 envelope、限制输入并将旧版本迁移到当前类型。
//! Runtime、Editor 和接口 crate 自己的持久化文档都经由本模块导出的契约接入。

#[path = "binary/mod.rs"]
mod binary;
#[path = "budget.rs"]
mod budget;
#[path = "error.rs"]
mod error;
#[path = "format.rs"]
mod format;
#[path = "load.rs"]
mod load;
#[path = "loaded.rs"]
mod loaded;
#[path = "migration/mod.rs"]
mod migration;
#[path = "payload_header.rs"]
mod payload_header;
#[path = "positional.rs"]
mod positional;
#[path = "schema_id.rs"]
mod schema_id;
#[path = "text/mod.rs"]
mod text;
#[path = "versioned_schema.rs"]
mod versioned_schema;
#[path = "write.rs"]
mod write;
#[path = "write_error.rs"]
mod write_error;

pub use budget::SerializationBudget;
pub use error::LoadError;
pub use format::Format;
pub use load::{load_versioned, load_versioned_envelope, load_versioned_legacy_schema_zero};
pub use loaded::Loaded;
pub use migration::{MigrateError, MigrationChain, MigrationStep};
pub use payload_header::PayloadHeader;
pub use positional::{PositionalDecodeError, PositionalEncodeError, PositionalEncoding};
pub use schema_id::{SchemaId, SchemaIdError, MAX_SCHEMA_ID_BYTES};
pub use versioned_schema::VersionedSchema;
pub use write::{
    write_canonical_text_to, write_versioned, write_versioned_text, write_versioned_text_to,
};
pub use write_error::{CanonicalTextWriteError, WriteError};

#[cfg(test)]
#[path = "tests/mod.rs"]
mod tests;
