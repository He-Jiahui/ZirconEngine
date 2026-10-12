//! 输入动作、手柄、宿主请求与公共契约保持由运行时输入栈拥有。集中挂载下级测试；所有行为断言留在被挂载模块。
#[path = "input_stack/action_mapping.rs"]
mod action_mapping;
#[path = "input_stack/contracts.rs"]
mod contracts;
#[path = "input_stack/gamepad_bridge.rs"]
mod gamepad_bridge;
#[path = "input_stack/inventory.rs"]
mod inventory;
#[path = "input_stack/split_layout.rs"]
mod split_layout;
#[path = "input_stack/support.rs"]
mod support;
