//! 脚本宿主绑定、游戏流程和门禁文档需对应同一运行时契约。集中挂载下级测试；所有行为断言留在被挂载模块。
#[path = "script_binding/gameplay_host.rs"]
mod gameplay_host;
#[path = "script_binding/inventory.rs"]
mod inventory;
#[path = "script_binding/mirror_docs.rs"]
mod mirror_docs;
#[path = "script_binding/split_layout.rs"]
mod split_layout;
#[path = "script_binding/support.rs"]
mod support;
