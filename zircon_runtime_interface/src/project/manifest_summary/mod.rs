//! Hub、Editor 与 Runtime 共用的轻量清单摘要和版本迁移入口。
//! 摘要用于展示与预检；项目运行仍须通过 Runtime 的完整清单校验。

mod admission;
mod error;
mod limits;
mod migration;
mod parse;
mod summary;

pub use error::ProjectManifestSummaryError;
pub use limits::{
    MAX_PROJECT_ASSET_ROOTS, MAX_PROJECT_MANIFEST_ARRAY_ITEMS, MAX_PROJECT_MANIFEST_BYTES,
    MAX_PROJECT_MANIFEST_NESTING_DEPTH, MAX_PROJECT_MANIFEST_TABLE_ENTRIES,
};
pub use migration::{load_project_manifest_value_from_toml_str, PROJECT_MANIFEST_FORMAT_VERSION};
pub use parse::validate_engine_version_req;
pub use summary::ProjectManifestSummary;
