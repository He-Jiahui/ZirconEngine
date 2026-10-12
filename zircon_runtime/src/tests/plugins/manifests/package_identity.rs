//! 静态清单的包目录、标识符及全局唯一性回归；经共享读取和本领域断言检查当前包声明，不执行插件行为。
#[path = "package_identity/directories.rs"]
mod directories;
#[path = "package_identity/namespaces.rs"]
mod namespaces;
#[path = "package_identity/package_id_tokens.rs"]
mod package_id_tokens;
#[path = "package_identity/uniqueness.rs"]
mod uniqueness;
