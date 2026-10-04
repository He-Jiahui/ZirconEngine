use std::sync::Arc;

use crate::core::framework::render::{
    RenderCameraTargetGraphImportReport, RenderCameraTargetGraphImportStatus,
    RenderCameraTargetWritebackReport,
};
use crate::graphics::scene::resources::{OutputTargetTextureResource, ResourceStreamer};
use crate::graphics::types::{GraphicsError, ViewportRenderFrame};

/// 编译图对相机输出目标的单帧选择；直接导入和转换写回互斥。
pub(super) struct FinalTargetOutputSelection {
    pub(super) direct_import: Option<Arc<OutputTargetTextureResource>>,
    pub(super) writeback: Option<Arc<OutputTargetTextureResource>>,
    pub(super) graph_import_report: Option<RenderCameraTargetGraphImportReport>,
    writeback_plan: RenderCameraTargetWritebackReport,
}

impl FinalTargetOutputSelection {
    pub(super) fn imported_resource(&self) -> Option<&OutputTargetTextureResource> {
        self.direct_import.as_deref()
    }

    pub(super) fn output_target_resource(&self) -> Option<&OutputTargetTextureResource> {
        self.direct_import
            .as_deref()
            .or_else(|| self.writeback.as_deref())
    }

    pub(super) const fn writeback_plan(&self) -> RenderCameraTargetWritebackReport {
        self.writeback_plan
    }
}

/// 消费 ResourceStreamer 预先准备的目标计划；不能在绑定阶段重新决定格式或写回策略。
/// 同一帧的 frame target 与计划不一致时应报错，防止把图输出写到错误的相机纹理。
pub(super) fn select_final_target_output(
    streamer: &ResourceStreamer,
    frame: &ViewportRenderFrame,
) -> Result<FinalTargetOutputSelection, GraphicsError> {
    let frame_plan = streamer.output_target_frame_plan();
    if frame_plan.target() != frame.output_target() {
        return Err(GraphicsError::Asset(format!(
            "prepared output target frame plan {:?} does not match frame target {:?}",
            frame_plan.target(),
            frame.output_target()
        )));
    }
    let graph_import_report = frame_plan.graph_import_report();
    let writeback_plan = frame_plan.compiled_graph_writeback_plan();
    let Some(texture) = frame.output_target().texture_handle() else {
        return Ok(FinalTargetOutputSelection {
            direct_import: None,
            writeback: None,
            graph_import_report: None,
            writeback_plan,
        });
    };
    if matches!(
        graph_import_report.status,
        RenderCameraTargetGraphImportStatus::SuppressedByCameraStack
            | RenderCameraTargetGraphImportStatus::PendingTargetDescriptor
            | RenderCameraTargetGraphImportStatus::BlockedFormatMismatch
    ) {
        return Ok(FinalTargetOutputSelection {
            direct_import: None,
            writeback: None,
            graph_import_report: Some(graph_import_report),
            writeback_plan,
        });
    }
    let prepared = streamer
        .output_target_texture_resource(&texture.id())
        .ok_or_else(|| {
            GraphicsError::Asset(format!(
                "prepared output target frame plan references missing texture {}",
                texture.id()
            ))
        })?;
    if graph_import_report.status == RenderCameraTargetGraphImportStatus::ReadyForDirectImport {
        return Ok(FinalTargetOutputSelection {
            direct_import: Some(prepared.clone()),
            writeback: None,
            graph_import_report: Some(RenderCameraTargetGraphImportReport::direct_imported(
                prepared.size(),
            )),
            writeback_plan,
        });
    }
    if graph_import_report.status
        == RenderCameraTargetGraphImportStatus::RequiresConversionWriteback
    {
        return Ok(FinalTargetOutputSelection {
            direct_import: None,
            writeback: Some(prepared.clone()),
            graph_import_report: Some(graph_import_report),
            writeback_plan,
        });
    }
    Err(GraphicsError::Asset(format!(
        "prepared output target frame plan has unexpected graph import status {:?}",
        graph_import_report.status
    )))
}

#[cfg(test)]
#[path = "tests/final_target_output.rs"]
mod tests;
