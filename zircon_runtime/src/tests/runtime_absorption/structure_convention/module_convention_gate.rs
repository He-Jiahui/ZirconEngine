//! 模块约定门禁的结构测试入口；子模块对审计脚本输出、文档元数据和债务分类做静态校验。
#[path = "module_convention_gate/debt_boundary.rs"]
mod debt_boundary;
#[path = "module_convention_gate/helpers.rs"]
mod helpers;
#[path = "module_convention_gate/module_doc_frontmatter.rs"]
mod module_doc_frontmatter;
#[path = "module_convention_gate/output_contract.rs"]
mod output_contract;
#[path = "module_convention_gate/split_layout.rs"]
mod split_layout;
