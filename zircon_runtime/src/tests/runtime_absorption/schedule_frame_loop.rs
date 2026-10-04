//! 帧调度执行链与行为锚点保持由调度模块和文档共同约束。集中挂载下级测试；所有行为断言留在被挂载模块。
#[path = "schedule_frame_loop/inventory.rs"]
mod inventory;
#[path = "schedule_frame_loop/mirror_docs.rs"]
mod mirror_docs;
#[path = "schedule_frame_loop/runtime_anchors.rs"]
mod runtime_anchors;
#[path = "schedule_frame_loop/split_layout.rs"]
mod split_layout;
