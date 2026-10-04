//! 实体组件存储与标识能力吸收后，核心拥有者和文档锚点需一致。集中挂载下级测试；所有行为断言留在被挂载模块。
#[path = "ecs_kernel_data/component_storage.rs"]
mod component_storage;
#[path = "ecs_kernel_data/docs.rs"]
mod docs;
#[path = "ecs_kernel_data/guard_coverage.rs"]
mod guard_coverage;
#[path = "ecs_kernel_data/identity_storage.rs"]
mod identity_storage;
#[path = "ecs_kernel_data/inventory.rs"]
mod inventory;
#[path = "ecs_kernel_data/runtime_flow.rs"]
mod runtime_flow;
#[path = "ecs_kernel_data/split_layout.rs"]
mod split_layout;
#[path = "ecs_kernel_data/support.rs"]
mod support;
