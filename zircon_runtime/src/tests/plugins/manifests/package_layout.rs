//! 静态清单的资源根、平台和默认打包策略回归；经共享读取和本领域断言检查当前包声明，不执行插件行为。
#[path = "package_layout/arrays.rs"]
mod arrays;
#[path = "package_layout/default_packaging.rs"]
mod default_packaging;
#[path = "package_layout/roots.rs"]
mod roots;
#[path = "package_layout/supported_platforms.rs"]
mod supported_platforms;
