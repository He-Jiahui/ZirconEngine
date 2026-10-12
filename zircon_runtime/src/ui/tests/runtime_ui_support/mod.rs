//! 仅由 ui 的 cfg(test) 入口挂载：夹具管理器将真实 Surface 输入和布局 API 串联，供测试验证宿主顺序。
#[path = "runtime_ui_fixture.rs"]
mod runtime_ui_fixture;
#[path = "runtime_ui_manager.rs"]
mod runtime_ui_manager;
#[path = "runtime_ui_manager_error.rs"]
mod runtime_ui_manager_error;
#[path = "window_event.rs"]
mod window_event;

pub(crate) use runtime_ui_fixture::RuntimeUiFixture;
pub(crate) use runtime_ui_manager::RuntimeUiManager;
