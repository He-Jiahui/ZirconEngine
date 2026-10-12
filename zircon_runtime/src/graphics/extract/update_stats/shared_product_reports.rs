use crate::graphics::scene::RenderGraphLightGridReport;

#[derive(Clone, Copy, Debug, Default)]
pub(in crate::graphics::runtime::render_framework::submit_frame_extract) struct SharedViewportProductReports
{
    light_grid_report: Option<RenderGraphLightGridReport>,
}

impl SharedViewportProductReports {
    pub(in crate::graphics::runtime::render_framework::submit_frame_extract) const fn new(
        light_grid_report: Option<RenderGraphLightGridReport>,
    ) -> Self {
        Self { light_grid_report }
    }

    pub(super) fn light_grid_report(self) -> Option<RenderGraphLightGridReport> {
        self.light_grid_report
    }
}
