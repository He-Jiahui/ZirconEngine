//! 代码审查回归护栏核对已迁移接口、错误边界、插件入口及镜像状态。集中挂载下级测试；所有行为断言留在被挂载模块。
#[path = "asset_records/authoring.rs"]
mod authoring;
#[path = "asset_records/font.rs"]
mod font;
#[path = "asset_records/meta.rs"]
mod meta;
#[path = "asset_records/navigation.rs"]
mod navigation;
#[path = "asset_records/sound.rs"]
mod sound;
#[path = "asset_records/zshader.rs"]
mod zshader;
