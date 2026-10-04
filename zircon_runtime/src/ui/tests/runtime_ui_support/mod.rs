//! 仅由 ui 的 cfg(test) 入口挂载：夹具管理器将真实 Surface 输入和布局 API 串联，供测试验证宿主顺序。
mod runtime_ui_fixture;
mod runtime_ui_manager;
mod runtime_ui_manager_error;
mod window_event;

pub(crate) use runtime_ui_fixture::RuntimeUiFixture;
pub(crate) use runtime_ui_manager::RuntimeUiManager;
