use zircon_editor_ui::EditorComponentDescriptor;

use super::{
    ASSET_SURFACE_DOCUMENT_ID, INSPECTOR_SURFACE_DOCUMENT_ID, PANE_SURFACE_DOCUMENT_ID,
    SCENE_VIEWPORT_TOOLBAR_DOCUMENT_ID, WELCOME_SURFACE_DOCUMENT_ID, WORKBENCH_SHELL_DOCUMENT_ID,
};

pub(crate) fn builtin_component_descriptors() -> Vec<EditorComponentDescriptor> {
    vec![
        EditorComponentDescriptor::new(
            "WorkbenchShell",
            WORKBENCH_SHELL_DOCUMENT_ID,
            "WorkbenchShell",
        ),
        EditorComponentDescriptor::new("MenuBar", WORKBENCH_SHELL_DOCUMENT_ID, "WorkbenchMenuBar"),
        EditorComponentDescriptor::new("ActivityRail", WORKBENCH_SHELL_DOCUMENT_ID, "ActivityRail"),
        EditorComponentDescriptor::new("DocumentHost", WORKBENCH_SHELL_DOCUMENT_ID, "DocumentHost"),
        EditorComponentDescriptor::new("StatusBar", WORKBENCH_SHELL_DOCUMENT_ID, "StatusBar"),
        EditorComponentDescriptor::new(
            "SceneViewportToolbar",
            SCENE_VIEWPORT_TOOLBAR_DOCUMENT_ID,
            "ViewportToolbar",
        ),
        EditorComponentDescriptor::new(
            "AssetSurfaceControls",
            ASSET_SURFACE_DOCUMENT_ID,
            "AssetSurface",
        ),
        EditorComponentDescriptor::new(
            "WelcomeSurfaceControls",
            WELCOME_SURFACE_DOCUMENT_ID,
            "WelcomeSurface",
        ),
        EditorComponentDescriptor::new(
            "InspectorSurfaceControls",
            INSPECTOR_SURFACE_DOCUMENT_ID,
            "InspectorView",
        ),
        EditorComponentDescriptor::new(
            "PaneSurfaceControls",
            PANE_SURFACE_DOCUMENT_ID,
            "PaneSurface",
        ),
    ]
}
