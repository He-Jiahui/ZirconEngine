use crate::core::editor_event::MenuAction;
use crate::core::editor_operation::EditorOperationPath;

use super::{EditorCommandEnablement, EditorKeyChord};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditorCommandDescriptor {
    id: String,
    label: String,
    description: String,
    category: EditorCommandCategory,
    menu_path: Option<String>,
    action: EditorCommandAction,
    default_chord: Option<EditorKeyChord>,
    enablement: EditorCommandEnablement,
    keywords: Vec<String>,
}

impl EditorCommandDescriptor {
    pub fn new(
        id: impl Into<String>,
        label: impl Into<String>,
        category: EditorCommandCategory,
        action: EditorCommandAction,
    ) -> Self {
        let label = label.into();
        Self {
            id: id.into(),
            description: label.clone(),
            label,
            category,
            menu_path: None,
            action,
            default_chord: None,
            enablement: EditorCommandEnablement::Always,
            keywords: Vec::new(),
        }
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    pub fn with_menu_path(mut self, menu_path: impl Into<String>) -> Self {
        self.menu_path = Some(menu_path.into());
        self
    }

    pub fn with_default_chord(mut self, chord: EditorKeyChord) -> Self {
        self.default_chord = Some(chord);
        self
    }

    pub fn with_enablement(mut self, enablement: EditorCommandEnablement) -> Self {
        self.enablement = enablement;
        self
    }

    pub fn with_keywords<I, S>(mut self, keywords: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.keywords = keywords.into_iter().map(Into::into).collect();
        self.keywords.sort();
        self.keywords.dedup();
        self
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn label(&self) -> &str {
        &self.label
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn category(&self) -> EditorCommandCategory {
        self.category
    }

    pub fn menu_path(&self) -> Option<&str> {
        self.menu_path.as_deref()
    }

    pub fn action(&self) -> &EditorCommandAction {
        &self.action
    }

    pub fn default_chord(&self) -> Option<&EditorKeyChord> {
        self.default_chord.as_ref()
    }

    pub fn enablement(&self) -> EditorCommandEnablement {
        self.enablement
    }

    pub fn enabled_route(&self) -> Option<&'static str> {
        self.enablement.route()
    }

    pub fn keywords(&self) -> &[String] {
        &self.keywords
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum EditorCommandCategory {
    File,
    Edit,
    Selection,
    Runtime,
    View,
    Window,
    Help,
    Command,
}

impl EditorCommandCategory {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::File => "File",
            Self::Edit => "Edit",
            Self::Selection => "Selection",
            Self::Runtime => "Play",
            Self::View => "View",
            Self::Window => "Window",
            Self::Help => "Help",
            Self::Command => "Command",
        }
    }

    pub fn source_tag(self) -> &'static str {
        match self {
            Self::File => "file",
            Self::Edit => "edit",
            Self::Selection => "selection",
            Self::Runtime => "runtime",
            Self::View => "view",
            Self::Window => "window",
            Self::Help => "help",
            Self::Command => "command",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EditorCommandAction {
    Menu(MenuAction),
    Operation(EditorOperationPath),
    OpenCommandPalette,
}
