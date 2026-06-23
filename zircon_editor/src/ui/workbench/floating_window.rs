use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FloatingWindowKind {
    CommandPalette,
    Preferences,
    DetachedEditor,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FloatingLayer {
    TopOverlay,
    ModalOverlay,
    NativeDetached,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FloatingWindowPlacement {
    TopCenter,
    WorkbenchCenter,
    NativeDetached,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FloatingWindowContentLayout {
    CommandPalette,
    NavigationContent,
    PageTemplate,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FloatingWindowInteractionMode {
    KeyboardDriven,
    ModalSettings,
    DetachedEditorPage,
}

/// Static design-parity contract used by tests and docs to keep floating
/// window declarations aligned with the editor layout design references.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FloatingWindowDesignContract {
    pub kind: FloatingWindowKind,
    pub modal: bool,
    pub layer: FloatingLayer,
    pub placement: FloatingWindowPlacement,
    pub content_layout: FloatingWindowContentLayout,
    pub interaction_mode: FloatingWindowInteractionMode,
    pub requires_editor_tokens: bool,
}

const COMMAND_PALETTE_DESIGN_CONTRACT: FloatingWindowDesignContract =
    FloatingWindowDesignContract {
        kind: FloatingWindowKind::CommandPalette,
        modal: false,
        layer: FloatingLayer::TopOverlay,
        placement: FloatingWindowPlacement::TopCenter,
        content_layout: FloatingWindowContentLayout::CommandPalette,
        interaction_mode: FloatingWindowInteractionMode::KeyboardDriven,
        requires_editor_tokens: true,
    };

const PREFERENCES_DESIGN_CONTRACT: FloatingWindowDesignContract = FloatingWindowDesignContract {
    kind: FloatingWindowKind::Preferences,
    modal: true,
    layer: FloatingLayer::ModalOverlay,
    placement: FloatingWindowPlacement::WorkbenchCenter,
    content_layout: FloatingWindowContentLayout::NavigationContent,
    interaction_mode: FloatingWindowInteractionMode::ModalSettings,
    requires_editor_tokens: true,
};

const DETACHED_EDITOR_DESIGN_CONTRACT: FloatingWindowDesignContract =
    FloatingWindowDesignContract {
        kind: FloatingWindowKind::DetachedEditor,
        modal: false,
        layer: FloatingLayer::NativeDetached,
        placement: FloatingWindowPlacement::NativeDetached,
        content_layout: FloatingWindowContentLayout::PageTemplate,
        interaction_mode: FloatingWindowInteractionMode::DetachedEditorPage,
        requires_editor_tokens: true,
    };

pub const FLOATING_WINDOW_DESIGN_CONTRACTS: [FloatingWindowDesignContract; 3] = [
    COMMAND_PALETTE_DESIGN_CONTRACT,
    PREFERENCES_DESIGN_CONTRACT,
    DETACHED_EDITOR_DESIGN_CONTRACT,
];

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FloatingWindow {
    pub kind: FloatingWindowKind,
    pub modal: bool,
    pub layer: FloatingLayer,
    pub content_asset: String,
}

impl FloatingWindow {
    pub fn command_palette() -> Self {
        Self {
            kind: FloatingWindowKind::CommandPalette,
            modal: false,
            layer: FloatingLayer::TopOverlay,
            content_asset: "res://ui/editor/components/workbench/floating/command_palette.zui"
                .to_string(),
        }
    }

    pub fn preferences() -> Self {
        Self {
            kind: FloatingWindowKind::Preferences,
            modal: true,
            layer: FloatingLayer::ModalOverlay,
            content_asset: "res://ui/editor/components/workbench/floating/preferences.zui"
                .to_string(),
        }
    }

    pub fn detached_editor(content_asset: impl Into<String>) -> Self {
        Self {
            kind: FloatingWindowKind::DetachedEditor,
            modal: false,
            layer: FloatingLayer::NativeDetached,
            content_asset: content_asset.into(),
        }
    }

    pub fn design_contract(&self) -> &'static FloatingWindowDesignContract {
        self.kind.design_contract()
    }
}

impl FloatingWindowKind {
    pub const fn design_contract(self) -> &'static FloatingWindowDesignContract {
        match self {
            Self::CommandPalette => &COMMAND_PALETTE_DESIGN_CONTRACT,
            Self::Preferences => &PREFERENCES_DESIGN_CONTRACT,
            Self::DetachedEditor => &DETACHED_EDITOR_DESIGN_CONTRACT,
        }
    }
}
