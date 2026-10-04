use serde::{Deserialize, Serialize};

/// 启动配置中的主窗口槽位标识；`Option` 的有无决定 App 是否创建主窗口。
/// 它不是 `WindowId` 或后端原生句柄，不能用于运行期窗口寻址。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PrimaryWindowHandle(u64);

impl PrimaryWindowHandle {
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> u64 {
        self.0
    }
}
