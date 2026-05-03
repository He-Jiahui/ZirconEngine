use crate::ui::template::EditorComponentDescriptor;

use super::{
    ASSET_SURFACE_DOCUMENT_ID, INSPECTOR_SURFACE_DOCUMENT_ID,
    PANE_ANIMATION_GRAPH_BODY_DOCUMENT_ID, PANE_ANIMATION_SEQUENCE_BODY_DOCUMENT_ID,
    PANE_BUILD_EXPORT_BODY_DOCUMENT_ID, PANE_CONSOLE_BODY_DOCUMENT_ID,
    PANE_HIERARCHY_BODY_DOCUMENT_ID, PANE_INSPECTOR_BODY_DOCUMENT_ID,
    PANE_MODULE_PLUGINS_BODY_DOCUMENT_ID, PANE_RUNTIME_DIAGNOSTICS_BODY_DOCUMENT_ID,
    PANE_SURFACE_DOCUMENT_ID, SCENE_VIEWPORT_TOOLBAR_DOCUMENT_ID, UI_HOST_WINDOW_DOCUMENT_ID,
    WELCOME_SURFACE_DOCUMENT_ID,
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
        EditorComponentDescriptor::new(
            "ConsolePaneBody",
            PANE_CONSOLE_BODY_DOCUMENT_ID,
            "ConsolePaneBody",
        ),
        EditorComponentDescriptor::new(
            "InspectorPaneBody",
            PANE_INSPECTOR_BODY_DOCUMENT_ID,
            "InspectorPaneBody",
        ),
        EditorComponentDescriptor::new(
            "HierarchyPaneBody",
            PANE_HIERARCHY_BODY_DOCUMENT_ID,
            "HierarchyPaneBody",
        ),
        EditorComponentDescriptor::new(
            "AnimationSequencePaneBody",
            PANE_ANIMATION_SEQUENCE_BODY_DOCUMENT_ID,
            "AnimationSequencePaneBody",
        ),
        EditorComponentDescriptor::new(
            "AnimationGraphPaneBody",
            PANE_ANIMATION_GRAPH_BODY_DOCUMENT_ID,
            "AnimationGraphPaneBody",
        ),
        EditorComponentDescriptor::new(
            "RuntimeDiagnosticsPaneBody",
            PANE_RUNTIME_DIAGNOSTICS_BODY_DOCUMENT_ID,
            "RuntimeDiagnosticsPaneBody",
        ),
        EditorComponentDescriptor::new(
            "ModulePluginsPaneBody",
            PANE_MODULE_PLUGINS_BODY_DOCUMENT_ID,
            "ModulePluginsPaneBody",
        ),
        EditorComponentDescriptor::new(
            "BuildExportPaneBody",
            PANE_BUILD_EXPORT_BODY_DOCUMENT_ID,
            "BuildExportPaneBody",
        ),
    ]
}
