use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// 活动窗口的宿主方式声明；原生句柄创建和销毁由窗口宿主管理。
pub enum ActivityWindowHostMode {
    EmbeddedMainFrame,
    NativeWindowHandle,
}
