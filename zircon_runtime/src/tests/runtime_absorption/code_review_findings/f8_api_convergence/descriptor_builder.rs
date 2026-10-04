//! 代码审查回归护栏核对已迁移接口、错误边界、插件入口及镜像状态。集中挂载下级测试；所有行为断言留在被挂载模块。
#[path = "descriptor_builder/first_party_descriptors.rs"]
mod first_party_descriptors;
#[path = "descriptor_builder/scaffold.rs"]
mod scaffold;
#[path = "descriptor_builder/test_fixtures.rs"]
mod test_fixtures;
