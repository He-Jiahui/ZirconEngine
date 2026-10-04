//! 视图描述符的界面类别；实例实际位于哪个抽屉、文档树或原生窗口由宿主布局决定。
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ViewKind {
    ActivityView,
    ActivityWindow,
}
