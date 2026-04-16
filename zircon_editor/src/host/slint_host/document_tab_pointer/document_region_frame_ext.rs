use zircon_ui::UiFrame;

use crate::WorkbenchShellGeometry;

pub(in crate::host::slint_host::document_tab_pointer) trait DocumentRegionFrameExt {
    fn document_region_frame(&self) -> UiFrame;
}

impl DocumentRegionFrameExt for WorkbenchShellGeometry {
    fn document_region_frame(&self) -> UiFrame {
        self.region_frame(crate::ShellRegionId::Document)
    }
}
