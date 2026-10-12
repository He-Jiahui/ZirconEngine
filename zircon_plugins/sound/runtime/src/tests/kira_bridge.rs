//! 本组测试挂接相同运行时边界的夹具和行为用例；具体限制由子模块的入口断言界定。
#[path = "kira_bridge/catalog.rs"]
mod catalog;
#[path = "kira_bridge/graph.rs"]
mod graph;
#[path = "kira_bridge/lifecycle.rs"]
mod lifecycle;
#[path = "kira_bridge/playback_data.rs"]
mod playback_data;
#[path = "kira_bridge/source.rs"]
mod source;
