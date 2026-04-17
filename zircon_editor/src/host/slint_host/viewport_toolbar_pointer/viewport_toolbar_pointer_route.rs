#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ViewportToolbarPointerRoute {
    SetTool {
        surface_key: String,
        tool: String,
    },
    SetTransformSpace {
        surface_key: String,
        space: String,
    },
    SetProjectionMode {
        surface_key: String,
        mode: String,
    },
    AlignView {
        surface_key: String,
        orientation: String,
    },
    CycleDisplayMode {
        surface_key: String,
    },
    CycleGridMode {
        surface_key: String,
    },
    CycleTranslateSnap {
        surface_key: String,
    },
    CycleRotateSnapDegrees {
        surface_key: String,
    },
    CycleScaleSnap {
        surface_key: String,
    },
    TogglePreviewLighting {
        surface_key: String,
    },
    TogglePreviewSkybox {
        surface_key: String,
    },
    ToggleGizmosEnabled {
        surface_key: String,
    },
    FrameSelection {
        surface_key: String,
    },
}
