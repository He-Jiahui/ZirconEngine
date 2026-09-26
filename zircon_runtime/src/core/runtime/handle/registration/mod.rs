//! 模块注册先验证描述符并准备服务槽位，再一次性提交模块、服务及身份索引。
//! 首次冻结模块图后拒绝继续注册，激活与卸载共用该快照。

mod commit;
mod descriptor_entries;
mod descriptor_entries_five;
mod descriptor_entries_four;
mod descriptor_entries_three;
mod duplicates;
mod entry;
mod register_module;
mod service_lists;
mod validation;
