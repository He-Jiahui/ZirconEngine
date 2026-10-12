//! 刷新服务的并发准入、资源额度、发布和终态契约测试分组。
//! 独立服务与受控收集替身隔离生产进程权威，真实文件预算测试仍复用生产收集入口。
#[path = "admission.rs"]
mod admission;
#[path = "budget.rs"]
mod budget;
#[path = "publication.rs"]
mod publication;
#[path = "support.rs"]
mod support;
#[path = "terminal.rs"]
mod terminal;
