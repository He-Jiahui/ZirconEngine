use crate::ui::retained_host::primitives::{ModelRc, SharedString};

#[derive(Clone, Default)]
pub(crate) struct UiAssetStringSelectionData {
    pub items: ModelRc<SharedString>,
    pub selected_index: i32,
}

/// Created from one real pane hit, preserving the published shell frame and domain rows.
/// Query edits use the active field allocation; row/page events also require this frame.
#[derive(Clone)]
pub(crate) struct UiAssetPaletteInputTarget(std::sync::Arc<UiAssetPaletteInputTargetData>);

struct UiAssetPaletteInputTargetData {
    instance_id: SharedString,
    snapshot: crate::ui::asset_editor::palette::UiAssetPaletteSearchSnapshot,
    shell_frame: std::sync::Arc<zircon_runtime_interface::ui::surface::UiSurfaceFrame>,
}

impl PartialEq for UiAssetPaletteInputTarget {
    fn eq(&self, other: &Self) -> bool {
        std::sync::Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for UiAssetPaletteInputTarget {}

impl UiAssetPaletteInputTarget {
    pub(crate) fn new(
        instance_id: SharedString,
        snapshot: crate::ui::asset_editor::palette::UiAssetPaletteSearchSnapshot,
        shell_frame: std::sync::Arc<zircon_runtime_interface::ui::surface::UiSurfaceFrame>,
    ) -> Self {
        Self(std::sync::Arc::new(UiAssetPaletteInputTargetData {
            instance_id,
            snapshot,
            shell_frame,
        }))
    }

    pub(crate) fn instance_id(&self) -> &str {
        self.0.instance_id.as_str()
    }

    pub(crate) fn snapshot(
        &self,
    ) -> &crate::ui::asset_editor::palette::UiAssetPaletteSearchSnapshot {
        &self.0.snapshot
    }

    pub(crate) fn shell_frame(
        &self,
    ) -> &std::sync::Arc<zircon_runtime_interface::ui::surface::UiSurfaceFrame> {
        &self.0.shell_frame
    }
}
