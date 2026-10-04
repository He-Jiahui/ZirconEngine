//! 对现有磁盘存档中的槽位执行元数据、删除、重命名和时间更新。
//! 调用方可先预览目标或冲突；提交入口重新载入存档，因此预览不是提交许可。

mod metadata;
mod remove;
mod rename;
mod touch;
