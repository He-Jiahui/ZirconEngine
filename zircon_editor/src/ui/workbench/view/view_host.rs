//! 具体实例的当前布局宿主；恢复快照后由布局树重新确定，不能仅凭描述符的默认slot推断现址。
use serde::{Deserialize, Serialize};

use crate::ui::workbench::layout::{ActivityDrawerSlot, MainPageId};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ViewHost {
    Drawer(ActivityDrawerSlot),
    Document(MainPageId, Vec<usize>),
    FloatingWindow(MainPageId, Vec<usize>),
    ExclusivePage(MainPageId),
}
