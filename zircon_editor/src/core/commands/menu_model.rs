use crate::core::editor_event::MenuAction;
use crate::core::editor_operation::EditorOperationPath;

#[derive(Clone, Debug, PartialEq)]
pub struct MenuBarModel {
    pub menus: Vec<MenuModel>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MenuModel {
    pub label: String,
    pub items: Vec<MenuItemModel>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MenuItemModel {
    pub label: String,
    pub action: Option<MenuAction>,
    pub operation_path: Option<EditorOperationPath>,
    pub shortcut: Option<String>,
    pub enabled: bool,
    pub children: Vec<MenuItemModel>,
}

impl MenuItemModel {
    pub fn leaf(
        label: impl Into<String>,
        action: Option<MenuAction>,
        operation_path: Option<EditorOperationPath>,
        shortcut: Option<String>,
        enabled: bool,
    ) -> Self {
        Self {
            label: label.into(),
            action,
            operation_path,
            shortcut,
            enabled,
            children: Vec::new(),
        }
    }

    pub fn branch(label: impl Into<String>, children: Vec<MenuItemModel>) -> Self {
        Self {
            label: label.into(),
            action: None,
            operation_path: None,
            shortcut: None,
            enabled: children.iter().any(|child| child.enabled),
            children,
        }
    }

    pub fn has_children(&self) -> bool {
        !self.children.is_empty()
    }
}
