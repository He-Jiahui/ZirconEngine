use super::super::{LayoutManager, WorkbenchLayout};

impl LayoutManager {
    /// 选取内建混合布局基线；真实视图实例与资源准备仍由宿主启动链完成。
    pub fn default_layout(&self) -> WorkbenchLayout {
        crate::ui::host::builtin_hybrid_layout()
    }
}
