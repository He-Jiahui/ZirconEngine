use crate::core::editor_event::MenuAction;

#[derive(Clone, Debug, PartialEq, Eq)]
/// 界面绑定解码后的工作台事件，供不同宿主构造统一核心事件；执行仍由宿主控制器负责。
pub enum EditorHostEvent {
    Menu(MenuAction),
}
