//! 摄像机输出目标的两种终端执行路径：直导入或按计划写回。
//! 写回仅消费已批准的目标计划；直导入只报告结果，避免对同一目标重复拷贝。

use crate::core::framework::render::RenderCameraTargetWritebackStatus;
use crate::core::math::UVec2;
use crate::graphics::types::GraphicsError;
use crate::render_graph::RenderGraphResourceAccessKind;

use super::RenderPassGpuExecutionContext;

impl<'a> RenderPassGpuExecutionContext<'a> {
    pub(in crate::graphics::scene::scene_renderer) fn record_output_target_direct_import(
        &mut self,
        source_resource_name: &str,
    ) -> Result<(), String> {
        Self::require_texture_desc_by_name(
            &*self.resources,
            self.resource_resolver,
            source_resource_name,
            RenderGraphResourceAccessKind::Read,
        )?;
        let plan = self.output_target_writeback_plan;
        if matches!(
            plan.status,
            RenderCameraTargetWritebackStatus::ReadyForCopy
                | RenderCameraTargetWritebackStatus::ReadyForConversion
        ) {
            return self.fail_output_target_writeback(GraphicsError::Asset(format!(
                "output target direct-import pass received executable writeback plan {:?}",
                plan.status
            )));
        }
        self.output_target_writeback_report = Some(plan);
        Ok(())
    }

    pub(in crate::graphics::scene::scene_renderer) fn record_output_target_writeback(
        &mut self,
        source_resource_name: &str,
        destination_resource_name: &str,
    ) -> Result<(), String> {
        let plan = self.output_target_writeback_plan;
        if !matches!(
            plan.status,
            RenderCameraTargetWritebackStatus::ReadyForCopy
                | RenderCameraTargetWritebackStatus::ReadyForConversion
        ) {
            self.output_target_writeback_report = Some(plan);
            return Ok(());
        }
        let streamer = self.streamer.ok_or_else(|| {
            "output-target-writeback graph pass requires resource streamer context".to_string()
        })?;

        let resources = &*self.resources;
        let resolver = self.resource_resolver;
        let source_desc = Self::require_texture_desc_by_name(
            resources,
            resolver,
            source_resource_name,
            RenderGraphResourceAccessKind::Read,
        )?;
        let destination_desc = Self::require_texture_desc_by_name(
            resources,
            resolver,
            destination_resource_name,
            RenderGraphResourceAccessKind::Write,
        )?;
        let source_texture = Self::require_physical_texture_by_name(
            resources,
            resolver,
            source_resource_name,
            RenderGraphResourceAccessKind::Read,
        )?;
        let destination_texture = Self::require_physical_texture_by_name(
            resources,
            resolver,
            destination_resource_name,
            RenderGraphResourceAccessKind::Write,
        )?;
        let source_view = Self::require_texture_view_by_name(
            resources,
            resolver,
            source_resource_name,
            RenderGraphResourceAccessKind::Read,
        )?;
        let destination_view = Self::require_texture_view_by_name(
            resources,
            resolver,
            destination_resource_name,
            RenderGraphResourceAccessKind::Write,
        )?;
        let result = streamer.encode_planned_output_target_writeback(
            self.device,
            self.encoder,
            plan,
            source_texture,
            source_view,
            UVec2::new(source_desc.width, source_desc.height),
            destination_texture,
            destination_view,
            UVec2::new(destination_desc.width, destination_desc.height),
        );
        match result {
            Ok(report) => {
                self.output_target_writeback_report = Some(report);
                Ok(())
            }
            Err(error) => self.fail_output_target_writeback(error),
        }
    }

    fn fail_output_target_writeback(&mut self, error: GraphicsError) -> Result<(), String> {
        let reason = error.to_string();
        self.output_target_writeback_error = Some(error);
        Err(reason)
    }
}

#[cfg(test)]
#[path = "tests/output_target.rs"]
mod tests;
