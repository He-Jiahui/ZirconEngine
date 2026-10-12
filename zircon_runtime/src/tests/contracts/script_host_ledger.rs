//! 脚本宿主函数的能力、注册表和文档账本需相互对应。集中挂载下级测试；所有行为断言留在被挂载模块。
#[path = "script_host_ledger/capability.rs"]
mod capability;
#[path = "script_host_ledger/capability_fixture.rs"]
mod capability_fixture;
#[path = "script_host_ledger/catalog.rs"]
mod catalog;
#[path = "script_host_ledger/ecs_facade.rs"]
mod ecs_facade;
#[path = "script_host_ledger/ledger.rs"]
mod ledger;
#[path = "script_host_ledger/split_layout.rs"]
mod split_layout;
