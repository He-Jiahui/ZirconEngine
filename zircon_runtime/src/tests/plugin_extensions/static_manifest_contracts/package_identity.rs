//! 静态清单的包目录、标识符及全局唯一性回归；经共享读取和本领域断言检查当前包声明，不执行插件行为。
mod directories;
mod namespaces;
mod package_id_tokens;
mod uniqueness;
