use crate::ui::EditorComponentDescriptor;

use super::{
    ASSET_SURFACE_DOCUMENT_ID, INSPECTOR_SURFACE_DOCUMENT_ID, PANE_SURFACE_DOCUMENT_ID,
    SCENE_VIEWPORT_TOOLBAR_DOCUMENT_ID, UI_HOST_WINDOW_DOCUMENT_ID, WELCOME_SURFACE_DOCUMENT_ID,
};

pub(crate) fn builtin_component_descriptors() -> Vec<EditorComponentDescriptor> {
    vec![
        EditorComponentDescriptor::new("UiHostWindow", UI_HOST_WINDOW_DOCUMENT_ID, "UiHostWindow"),
        EditorComponentDescriptor::new("MenuBar", UI_HOST_WINDOW_DOCUMENT_ID, "WorkbenchMenuBar"),
        EditorComponentDescriptor::new("ActivityRail", UI_HOST_WINDOW_DOCUMENT_ID, "ActivityRail"),
        EditorComponentDescriptor::new("DocumentHost", UI_HOST_WINDOW_DOCUMENT_ID, "DocumentHost"),
        EditorComponentDescriptor::new("StatusBar", UI_HOST_WINDOW_DOCUMENT_ID, "StatusBar"),
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
