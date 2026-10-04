//! 将模板实例投影成 retained 树，再为 surface 安装同一实例的绑定程序。
//! 资产路径在上游展开并解析样式；构树保留节点语义和父子边上的 slot 契约，几何与事件在后续 surface 阶段处理。

mod build_error;
mod child_segment;
mod container_inference;
mod interaction;
mod layout_contract;
mod parsers;
mod slot_contract;
mod surface_builder;
mod tree_builder;

pub use build_error::UiTemplateBuildError;
pub use surface_builder::UiTemplateSurfaceBuilder;
pub use tree_builder::UiTemplateTreeBuilder;
