//! 路径入口每次重新加载并校验完整归档，再执行内存入口；读取失败时不修改目标。
//! apply 合入现有世界，restore 构造新世界或替换关卡，diff 只比较世界快照；均不写回归档。
mod apply;
mod diff;
mod restore;
