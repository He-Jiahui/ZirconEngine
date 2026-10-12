//! 模块注册先验证描述符并准备服务槽位，再一次性提交模块、服务及身份索引。
//! 首次冻结模块图后拒绝继续注册，激活与卸载共用该快照。

#[path = "commit.rs"]
mod commit;
#[path = "descriptor_entries.rs"]
mod descriptor_entries;
#[path = "descriptor_entries_five.rs"]
mod descriptor_entries_five;
#[path = "descriptor_entries_four.rs"]
mod descriptor_entries_four;
#[path = "descriptor_entries_three.rs"]
mod descriptor_entries_three;
#[path = "duplicates.rs"]
mod duplicates;
#[path = "entry.rs"]
mod entry;
#[path = "register_module.rs"]
mod register_module;
#[path = "service_lists/mod.rs"]
mod service_lists;
#[path = "validation.rs"]
mod validation;
