mod default_layout;
mod default_registry;
mod design_stack;
mod functional_window;
mod panel_preset;
mod shell_preset;

pub use design_stack::EditorUiDesignStack;
pub use functional_window::{
    EditorFunctionalWindowKind, EditorFunctionalWindowPreset, EditorWindowDockPolicy,
    UnrealWindowModelPreset,
};
pub use panel_preset::{FyroxPanelComponentRole, FyroxPanelInteraction, FyroxPanelPreset};
pub use shell_preset::{
    JetBrainsDrawerPreset, JetBrainsFloatingWindowBehavior, JetBrainsShellPreset,
    JetBrainsTabBehavior,
};
