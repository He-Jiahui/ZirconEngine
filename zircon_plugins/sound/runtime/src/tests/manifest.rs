//! 将静态插件清单与运行时声明、内建目录和注册贡献作对照；每组测试只比较其所属契约。
#[path = "manifest/contributions.rs"]
mod contributions;
#[path = "manifest/metadata.rs"]
mod metadata;
#[path = "manifest/options.rs"]
mod options;
#[path = "manifest/support.rs"]
mod support;
