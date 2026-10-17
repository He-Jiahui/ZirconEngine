//! Runtime 导航回调的注册与派发入口，向外导出 [`UiNavigationDispatcher`]。
//! 路由由 Surface 准备，处理器决议由此层收集，焦点提交仍由 Surface 完成。

#[path = "dispatcher.rs"]
mod dispatcher;

pub use dispatcher::UiNavigationDispatcher;
