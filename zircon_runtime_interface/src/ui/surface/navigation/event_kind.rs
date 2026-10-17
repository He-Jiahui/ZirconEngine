use serde::{Deserialize, Serialize};

/// 导航路由携带的命令种类；标识方向或动作，不执行导航。
/// 序列化保留变体的原始拼写，本类型未声明重命名或默认变体。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum UiNavigationEventKind {
    Activate,
    Cancel,
    Next,
    Previous,
    Home,
    End,
    Up,
    Down,
    Left,
    Right,
}
