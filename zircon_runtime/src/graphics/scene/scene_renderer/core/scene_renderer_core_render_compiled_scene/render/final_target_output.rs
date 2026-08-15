use std::sync::Arc;

use crate::core::framework::render::RenderCameraTargetGraphImportReport;
use crate::graphics::scene::resources::{OutputTargetTextureResource, ResourceStreamer};
use crate::graphics::types::{
    ViewportRenderFrame, ViewportRenderOutputTarget, ViewportTextureGraphImportStatus,
};

pub(super) struct FinalTargetOutputSelection {
    pub(super) direct_import: Option<Arc<OutputTargetTextureResource>>,
    pub(super) graph_import_report: Option<RenderCameraTargetGraphImportReport>,
}

impl FinalTargetOutputSelection {
    pub(super) fn imported_resource(&self) -> Option<&OutputTargetTextureResource> {
        self.direct_import.as_deref()
    }
}

pub(super) fn select_final_target_output(
    streamer: &ResourceStreamer,
    frame: &ViewportRenderFrame,
) -> FinalTargetOutputSelection {
    if !frame
        .camera_stack_output_policy()
        .owns_final_target_output()
    {
        return FinalTargetOutputSelection {
            direct_import: None,
            graph_import_report: suppressed_graph_import_report(frame.output_target()),
        };
    }

    let Some(texture) = frame.output_target().texture_handle() else {
        return FinalTargetOutputSelection {
            direct_import: None,
            graph_import_report: None,
        };
    };
    let Some(prepared) = streamer.output_target_texture_resource(&texture.id()) else {
        return FinalTargetOutputSelection {
            direct_import: None,
            graph_import_report: None,
        };
    };

    let plan = frame
        .output_target()
        .graph_import_plan(Some(prepared.descriptor().format.as_str()));
    if plan.status() == ViewportTextureGraphImportStatus::ReadyForDirectImport {
        return FinalTargetOutputSelection {
            direct_import: Some(prepared.clone()),
            graph_import_report: Some(RenderCameraTargetGraphImportReport::direct_imported(
                prepared.size(),
            )),
        };
    }

    FinalTargetOutputSelection {
        direct_import: None,
        graph_import_report: None,
    }
}

fn suppressed_graph_import_report(
    target: ViewportRenderOutputTarget,
) -> Option<RenderCameraTargetGraphImportReport> {
    match target.size() {
        Some(size) if target.texture_handle().is_some() => {
            Some(RenderCameraTargetGraphImportReport::suppressed_by_camera_stack(size))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use crate::core::framework::render::RenderCameraTargetGraphImportStatus;
    use crate::core::math::UVec2;
    use crate::core::resource::{ResourceHandle, ResourceId, TextureMarker};
    use crate::graphics::types::{ViewportRenderOutputTarget, FRAMEWORK_OUTPUT_FORMAT_LABEL};

    use super::suppressed_graph_import_report;

    #[test]
    fn final_target_output_reports_suppressed_texture_children_only() {
        let texture = ResourceHandle::<TextureMarker>::new(ResourceId::from_stable_label(
            "tests/final-target-output/suppressed-texture",
        ));
        let texture_target = ViewportRenderOutputTarget::Texture {
            handle: texture,
            size: UVec2::new(96, 54),
            format: FRAMEWORK_OUTPUT_FORMAT_LABEL,
        };

        let texture_report =
            suppressed_graph_import_report(texture_target).expect("texture suppression report");
        let primary_report =
            suppressed_graph_import_report(ViewportRenderOutputTarget::PrimarySurface);
        let headless_report =
            suppressed_graph_import_report(ViewportRenderOutputTarget::Headless {
                size: UVec2::new(96, 54),
            });

        assert_eq!(
            texture_report.status,
            RenderCameraTargetGraphImportStatus::SuppressedByCameraStack
        );
        assert_eq!(texture_report.target_size, UVec2::new(96, 54));
        assert_eq!(texture_report.direct_import_count, 0);
        assert_eq!(texture_report.conversion_writeback_count, 0);
        assert_eq!(primary_report, None);
        assert_eq!(headless_report, None);
    }
}
