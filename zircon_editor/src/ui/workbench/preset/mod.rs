#[path = "default_layout.rs"]
mod default_layout;
#[path = "default_registry.rs"]
mod default_registry;
#[path = "design_stack.rs"]
mod design_stack;
#[path = "functional_window.rs"]
mod functional_window;
#[path = "panel_preset.rs"]
mod panel_preset;
#[path = "shell_preset.rs"]
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
