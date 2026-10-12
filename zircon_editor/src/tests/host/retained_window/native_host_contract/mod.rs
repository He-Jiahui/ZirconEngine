// 汇集原生宿主指针、拖动、输入和共享夹具测试模块。
#[path = "../../native_host_contract/chrome_pointer.rs"]
mod chrome_pointer;
#[path = "../../native_host_contract/drag_projection.rs"]
mod drag_projection;
#[path = "../../native_host_contract/capture_pointer_owner.rs"]
mod native_capture_pointer_owner;
#[path = "support.rs"]
mod support;
#[path = "input.rs"]
mod template_input;
#[path = "../viewport_hierarchy_menu/mod.rs"]
mod viewport_hierarchy_menu;
