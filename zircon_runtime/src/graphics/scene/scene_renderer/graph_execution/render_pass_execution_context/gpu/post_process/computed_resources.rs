use crate::core::framework::render::RenderExposureMode;
use crate::graphics::scene::scene_renderer::cluster_dimensions_for_size;
use crate::graphics::scene::scene_renderer::graph_execution::{
    RenderGraphComputeDispatchRecord, RenderGraphLightGridReport,
};
use crate::graphics::scene::scene_renderer::history::SceneHistoryDomain;
use crate::graphics::scene::scene_renderer::lighting::light_grid_pass::{
    build_light_grid_for_frame, prepare_light_grid_buffer_uploads,
};
use crate::graphics::scene::scene_renderer::post_process::{
    clustered_lighting_dispatch_groups, clustered_lighting_workgroup_size,
    exposure_histogram_dispatch_groups, exposure_histogram_workgroup_size,
    exposure_resolve_dispatch_groups, exposure_resolve_workgroup_size, hzb_build_dispatch_groups,
    hzb_build_workgroup_size,
};
use crate::graphics::visibility::HzbBuilder;
use crate::render_graph::RenderGraphResourceAccessKind;

use super::super::RenderPassGpuExecutionContext;
use super::{
    EXPOSURE_HISTOGRAM_PIPELINE_LABEL, EXPOSURE_RESOLVE_PIPELINE_LABEL, HZB_BUILD_PIPELINE_LABEL,
};

impl<'a> RenderPassGpuExecutionContext<'a> {
    pub(in crate::graphics::scene::scene_renderer) fn record_exposure_histogram_to_resource(
        &mut self,
        pass_name: &str,
        executor_id: &str,
        scene_color_resource_name: &str,
        histogram_resource_name: &str,
    ) -> Result<(), String> {
        let settings = self.frame.post_process().exposure;
        if settings.mode != RenderExposureMode::Histogram {
            return Ok(());
        }
        let stack = self.post_process_stack.ok_or_else(|| {
            format!(
                "exposure histogram graph executor for pass `{pass_name}` requires post-process stack context"
            )
        })?;
        let resources = &*self.resources;
        let resource_resolver = self.resource_resolver;
        let scene_color_view = Self::require_texture_view_by_name(
            resources,
            resource_resolver,
            scene_color_resource_name,
            RenderGraphResourceAccessKind::Read,
        )?;
        let histogram_buffer = Self::require_buffer_binding_by_name(
            resources,
            resource_resolver,
            histogram_resource_name,
            RenderGraphResourceAccessKind::Write,
        )?;
        let scene_linear_size = stack.scene_linear_size(self.frame, pass_name)?;
        stack.post_process.execute_exposure_histogram(
            self.device,
            self.encoder,
            scene_linear_size,
            scene_color_view,
            histogram_buffer,
        );
        self.compute_dispatches
            .push(RenderGraphComputeDispatchRecord::new(
                pass_name,
                executor_id,
                EXPOSURE_HISTOGRAM_PIPELINE_LABEL,
                exposure_histogram_workgroup_size(),
                exposure_histogram_dispatch_groups(scene_linear_size),
                vec![histogram_resource_name.to_string()],
            ));
        Ok(())
    }

    pub(in crate::graphics::scene::scene_renderer) fn record_exposure_resolve_to_resource(
        &mut self,
        pass_name: &str,
        executor_id: &str,
        histogram_resource_name: &str,
        previous_resource_name: &str,
        current_resource_name: &str,
    ) -> Result<(), String> {
        let settings = self.frame.post_process().exposure;
        let stack = self.post_process_stack.ok_or_else(|| {
            format!(
                "exposure resolve graph executor for pass `{pass_name}` requires post-process stack context"
            )
        })?;
        let resources = &*self.resources;
        let resource_resolver = self.resource_resolver;
        let histogram_buffer = if settings.mode == RenderExposureMode::Histogram {
            Self::require_buffer_binding_by_name(
                resources,
                resource_resolver,
                histogram_resource_name,
                RenderGraphResourceAccessKind::Read,
            )?
        } else {
            stack
                .post_process
                .default_exposure_histogram_buffer_binding()
        };
        let previous_exposure_buffer = if let Some(previous_exposure_buffer) =
            Self::optional_buffer_binding_by_name(
                resources,
                resource_resolver,
                previous_resource_name,
                RenderGraphResourceAccessKind::Read,
            )? {
            previous_exposure_buffer
        } else {
            stack.post_process.default_exposure_buffer_binding()
        };
        let current_exposure_buffer = Self::require_buffer_binding_by_name(
            resources,
            resource_resolver,
            current_resource_name,
            RenderGraphResourceAccessKind::Write,
        )?;
        stack.post_process.execute_exposure_resolve(
            self.device,
            self.encoder,
            histogram_buffer,
            previous_exposure_buffer,
            current_exposure_buffer,
        );
        self.compute_dispatches
            .push(RenderGraphComputeDispatchRecord::new(
                pass_name,
                executor_id,
                EXPOSURE_RESOLVE_PIPELINE_LABEL,
                exposure_resolve_workgroup_size(),
                exposure_resolve_dispatch_groups(),
                vec![current_resource_name.to_string()],
            ));
        self.record_history_write(SceneHistoryDomain::Exposure);
        Ok(())
    }

    pub(in crate::graphics::scene::scene_renderer) fn record_clustered_lighting_to_resources(
        &mut self,
        pass_name: &str,
        executor_id: &str,
        light_grid_params_resource_name: &str,
        light_zbins_resource_name: &str,
        light_tile_masks_resource_name: &str,
        light_list_resource_name: &str,
    ) -> Result<(), String> {
        let stack = self.post_process_stack.ok_or_else(|| {
            format!(
                "light grid graph executor for pass `{pass_name}` requires post-process stack context"
            )
        })?;
        let resources = &*self.resources;
        let resource_resolver = self.resource_resolver;
        let light_grid_params_buffer = Self::require_buffer_binding_by_name(
            resources,
            resource_resolver,
            light_grid_params_resource_name,
            RenderGraphResourceAccessKind::Write,
        )?;
        let light_zbins_buffer = Self::require_buffer_binding_by_name(
            resources,
            resource_resolver,
            light_zbins_resource_name,
            RenderGraphResourceAccessKind::Write,
        )?;
        let light_tile_masks_buffer = Self::require_buffer_binding_by_name(
            resources,
            resource_resolver,
            light_tile_masks_resource_name,
            RenderGraphResourceAccessKind::Write,
        )?;
        let light_list_buffer = Self::require_buffer_binding_by_name(
            resources,
            resource_resolver,
            light_list_resource_name,
            RenderGraphResourceAccessKind::Write,
        )?;
        let enabled = stack.runtime_features.clustered_lighting_enabled;
        let scene_linear_size = stack.scene_linear_size(self.frame, pass_name)?;
        let scene_linear_cluster_dimensions = cluster_dimensions_for_size(scene_linear_size);
        let light_grid =
            build_light_grid_for_frame(&self.frame.extract, scene_linear_size, enabled);
        self.light_grid_report = Some(RenderGraphLightGridReport::from_stats(&light_grid.stats));
        let mut light_grid_uploads = prepare_light_grid_buffer_uploads(
            light_grid_params_buffer,
            light_zbins_buffer,
            light_tile_masks_buffer,
            &light_grid,
        );
        self.append_pre_submit_buffer_uploads(&mut light_grid_uploads);
        let dispatch_groups = clustered_lighting_dispatch_groups(scene_linear_cluster_dimensions);
        let workgroup_size = clustered_lighting_workgroup_size();
        let mut clustered_lighting_uploads = stack.post_process.execute_clustered_lighting(
            self.device,
            self.encoder,
            scene_linear_size,
            scene_linear_cluster_dimensions,
            light_list_buffer,
            &self.frame.extract.lighting.directional_lights,
            enabled,
        );
        self.append_pre_submit_buffer_uploads(&mut clustered_lighting_uploads);
        if enabled {
            self.compute_dispatches
                .push(RenderGraphComputeDispatchRecord::new(
                    pass_name,
                    executor_id,
                    "zircon-cluster-pipeline",
                    workgroup_size,
                    dispatch_groups,
                    vec![
                        light_grid_params_resource_name.to_string(),
                        light_zbins_resource_name.to_string(),
                        light_tile_masks_resource_name.to_string(),
                        light_list_resource_name.to_string(),
                    ],
                ));
        }
        Ok(())
    }

    pub(in crate::graphics::scene::scene_renderer) fn record_hzb_build_to_resource(
        &mut self,
        pass_name: &str,
        executor_id: &str,
        depth_resource_name: &str,
        hzb_resource_name: &str,
    ) -> Result<(), String> {
        let stack = self.post_process_stack.ok_or_else(|| {
            format!("HZB graph executor for pass `{pass_name}` requires post-process stack context")
        })?;
        let resources = &*self.resources;
        let resource_resolver = self.resource_resolver;
        let depth_view = Self::require_texture_view_by_name(
            resources,
            resource_resolver,
            depth_resource_name,
            RenderGraphResourceAccessKind::Read,
        )?;
        let _hzb_view = Self::require_texture_view_by_name(
            resources,
            resource_resolver,
            hzb_resource_name,
            RenderGraphResourceAccessKind::Write,
        )?;
        let scene_depth_sample_count = Self::require_texture_desc_by_name(
            resources,
            resource_resolver,
            depth_resource_name,
            RenderGraphResourceAccessKind::Read,
        )?
        .sample_count;
        let scene_linear_allocation_size =
            stack.scene_linear_allocation_size(self.frame, pass_name)?;
        let plan = HzbBuilder::new(scene_linear_allocation_size).build_plan();
        let params_upload_buffer = stack
            .post_process
            .create_hzb_params_upload_buffer(self.device, plan);
        for mip_level in 0..plan.mip_count {
            let source_view = if mip_level == 0 {
                None
            } else {
                Some(Self::require_owned_texture_mip_view_by_name(
                    resources,
                    resource_resolver,
                    hzb_resource_name,
                    hzb_resource_name,
                    RenderGraphResourceAccessKind::Write,
                    mip_level - 1,
                )?)
            };
            let target_view = Self::require_owned_texture_mip_view_by_name(
                resources,
                resource_resolver,
                hzb_resource_name,
                hzb_resource_name,
                RenderGraphResourceAccessKind::Write,
                mip_level,
            )?;
            let pipeline_statistics_scope = self.reserve_pipeline_statistics_scope(pass_name);
            stack.post_process.execute_hzb_build_mip(
                self.device,
                self.encoder,
                depth_view,
                source_view.as_ref(),
                &target_view,
                plan.mip_size(mip_level),
                mip_level,
                scene_depth_sample_count,
                &params_upload_buffer,
                pipeline_statistics_scope.as_ref(),
            );
        }
        let dispatch_groups = hzb_build_dispatch_groups(plan.hzb_size);
        let workgroup_size = hzb_build_workgroup_size();
        self.compute_dispatches
            .push(RenderGraphComputeDispatchRecord::new(
                pass_name,
                executor_id,
                HZB_BUILD_PIPELINE_LABEL,
                workgroup_size,
                dispatch_groups,
                vec![hzb_resource_name.to_string()],
            ));
        self.record_history_write(SceneHistoryDomain::HzbFurthest);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn exposure_graph_passes_consume_frame_prepared_params_without_queue_access() {
        let source = include_str!("computed_resources.rs");
        let exposure = source
            .split("pub(in crate::graphics::scene::scene_renderer) fn record_exposure_histogram_to_resource")
            .nth(1)
            .and_then(|source| source.split("pub(in crate::graphics::scene::scene_renderer) fn record_clustered_lighting_to_resources").next())
            .expect("exposure graph-pass section");

        assert!(!exposure.contains("self.queue"));
        assert!(!exposure.contains("append_pre_submit_buffer_uploads"));
        assert!(exposure.contains("execute_exposure_histogram("));
        assert!(exposure.contains("execute_exposure_resolve("));
    }

    #[test]
    fn clustered_lighting_appends_feature_uploads_to_the_light_grid_pass() {
        let source = include_str!("computed_resources.rs");
        let clustered = source
            .split("fn record_clustered_lighting_to_resources")
            .nth(1)
            .expect("clustered-lighting graph-pass section");
        let execute = clustered
            .find("execute_clustered_lighting(")
            .expect("clustered-lighting execution");
        let append = clustered[execute..]
            .find("self.append_pre_submit_buffer_uploads(")
            .map(|offset| execute + offset)
            .expect("clustered-lighting upload append");

        assert!(execute < append);
    }
}
