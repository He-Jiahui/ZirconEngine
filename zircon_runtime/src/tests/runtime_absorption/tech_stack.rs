//! 运行时技术栈、清单和结构文档维持共同的基础门禁。集中挂载下级测试；所有行为断言留在被挂载模块。
#[path = "tech_stack/behavior_anchors.rs"]
mod behavior_anchors;
#[path = "tech_stack/guard_anchors.rs"]
mod guard_anchors;
#[path = "tech_stack/manifest_inventory.rs"]
mod manifest_inventory;
#[path = "tech_stack/mirror_docs.rs"]
mod mirror_docs;
#[path = "tech_stack/split_layout.rs"]
mod split_layout;
