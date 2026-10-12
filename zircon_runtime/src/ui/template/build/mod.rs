//! 将模板实例投影成 retained 树，再为 surface 安装同一实例的绑定程序。
//! 资产路径在上游展开并解析样式；构树保留节点语义和父子边上的 slot 契约，几何与事件在后续 surface 阶段处理。

#[path = "build_error.rs"]
mod build_error;
#[path = "child_segment.rs"]
mod child_segment;
#[path = "container_inference.rs"]
mod container_inference;
#[path = "interaction.rs"]
mod interaction;
#[path = "layout_contract.rs"]
mod layout_contract;
#[path = "parsers.rs"]
mod parsers;
#[path = "slot_contract.rs"]
mod slot_contract;
#[path = "surface_builder.rs"]
mod surface_builder;
#[path = "tree_builder.rs"]
mod tree_builder;

pub use build_error::UiTemplateBuildError;
pub use surface_builder::UiTemplateSurfaceBuilder;
pub use tree_builder::UiTemplateTreeBuilder;
