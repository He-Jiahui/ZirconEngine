use crate::core::framework::render::PostProcessGraphResourceNames;
use crate::render_graph::{RenderGraphAttachmentOps, RenderGraphResourceAccessKind};

use super::super::RenderPassGpuExecutionContext;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SsrPyramidMipPass {
    source_mip_level: u32,
    target_mip_level: u32,
    attachment_ops: RenderGraphAttachmentOps,
}

fn ssr_parent_pyramid_mip_passes(
    mip_level_count: u32,
    graph_alias_attachment_ops: RenderGraphAttachmentOps,
) -> impl Iterator<Item = SsrPyramidMipPass> {
    (1..mip_level_count).map(move |target_mip_level| SsrPyramidMipPass {
        source_mip_level: target_mip_level - 1,
        target_mip_level,
        attachment_ops: if target_mip_level == 1 {
            graph_alias_attachment_ops
        } else {
            RenderGraphAttachmentOps::clear_store()
        },
    })
}

impl<'a> RenderPassGpuExecutionContext<'a> {
    pub(in crate::graphics::scene::scene_renderer) fn record_screen_space_reflection_resolve_to_resource(
        &mut self,
        pass_name: &str,
        scene_color_resource_name: &str,
        scene_depth_resource_name: &str,
        motion_vector_neighbor_max_resource_name: &str,
        hzb_furthest_resource_name: &str,
        screen_space_reflection_reflection_pyramid_resource_name: &str,
        screen_space_reflection_reflection_pyramid_coarse_resource_name: &str,
        screen_space_reflection_specular_occlusion_resource_name: &str,
        screen_space_reflection_history_resource_name: &str,
        attachment_ops: RenderGraphAttachmentOps,
    ) -> Result<(), String> {
        let stack = self.post_process_stack.ok_or_else(|| {
            format!(
                "screen-space reflection resolve graph executor for pass `{pass_name}` requires post-process stack context"
            )
        })?;
        let target = stack.target;
        let history = stack.history_textures;
        let features = stack.runtime_features;
        let resources = &*self.resources;
        let resource_resolver = self.resource_resolver;
        let scene_color_view = Self::require_texture_view_by_name(
            resources,
            resource_resolver,
            scene_color_resource_name,
            RenderGraphResourceAccessKind::Read,
        )?;
        let scene_depth_view = Self::require_texture_view_by_name(
            resources,
            resource_resolver,
            scene_depth_resource_name,
            RenderGraphResourceAccessKind::Read,
        )?;
        let motion_vector_neighbor_max_view = Self::require_texture_view_by_name(
            resources,
            resource_resolver,
            motion_vector_neighbor_max_resource_name,
            RenderGraphResourceAccessKind::Read,
        )?;
        let scene_normal_view = Self::require_texture_view_by_name(
            resources,
            resource_resolver,
            PostProcessGraphResourceNames::GBUFFER_NORMAL,
            RenderGraphResourceAccessKind::Read,
        )?;
        let scene_material_view = stack
            .material_gbuffer_valid
            .then(|| {
                Self::require_texture_view_by_name(
                    resources,
                    resource_resolver,
                    PostProcessGraphResourceNames::GBUFFER_MATERIAL,
                    RenderGraphResourceAccessKind::Read,
                )
            })
            .transpose()?;
        let ambient_occlusion_view = Self::optional_texture_view_by_name(
            resources,
            resource_resolver,
            PostProcessGraphResourceNames::AMBIENT_OCCLUSION,
            RenderGraphResourceAccessKind::Read,
        )?
        .unwrap_or_else(|| stack.post_process.white_texture_view());
        let bloom_view = Self::optional_texture_view_by_name(
            resources,
            resource_resolver,
            PostProcessGraphResourceNames::BLOOM,
            RenderGraphResourceAccessKind::Read,
        )?
        .unwrap_or_else(|| stack.post_process.black_texture_view());
        let depth_of_field_coc_view = Self::optional_texture_view_by_name(
            resources,
            resource_resolver,
            PostProcessGraphResourceNames::DEPTH_OF_FIELD_COC,
            RenderGraphResourceAccessKind::Read,
        )?
        .unwrap_or_else(|| stack.post_process.black_texture_view());
        let depth_of_field_bokeh_view = Self::optional_texture_view_by_name(
            resources,
            resource_resolver,
            PostProcessGraphResourceNames::DEPTH_OF_FIELD_BOKEH,
            RenderGraphResourceAccessKind::Read,
        )?
        .unwrap_or_else(|| stack.post_process.black_texture_view());
        let screen_space_reflection_history_view = Self::require_texture_view_by_name(
            resources,
            resource_resolver,
            screen_space_reflection_history_resource_name,
            RenderGraphResourceAccessKind::Write,
        )?;
        let hzb_furthest_view = Self::require_texture_view_by_name(
            resources,
            resource_resolver,
            hzb_furthest_resource_name,
            RenderGraphResourceAccessKind::Read,
        )?;
        let screen_space_reflection_reflection_pyramid_view = Self::require_texture_view_by_name(
            resources,
            resource_resolver,
            screen_space_reflection_reflection_pyramid_resource_name,
            RenderGraphResourceAccessKind::Read,
        )?;
        let hzb_furthest_full_mip_view = Self::optional_owned_texture_full_mip_view_by_name(
            resources,
            resource_resolver,
            hzb_furthest_resource_name,
            RenderGraphResourceAccessKind::Read,
        )?;
        let screen_space_reflection_reflection_pyramid_full_mip_view =
            Self::optional_owned_texture_full_mip_view_by_name(
                resources,
                resource_resolver,
                screen_space_reflection_reflection_pyramid_resource_name,
                RenderGraphResourceAccessKind::Read,
            )?;
        let hzb_furthest_sampling_view = hzb_furthest_full_mip_view
            .as_ref()
            .unwrap_or(hzb_furthest_view);
        let screen_space_reflection_reflection_pyramid_sampling_view =
            screen_space_reflection_reflection_pyramid_full_mip_view
                .as_ref()
                .unwrap_or(screen_space_reflection_reflection_pyramid_view);
        let screen_space_reflection_reflection_pyramid_coarse_view =
            Self::require_texture_view_by_name(
                resources,
                resource_resolver,
                screen_space_reflection_reflection_pyramid_coarse_resource_name,
                RenderGraphResourceAccessKind::Read,
            )?;
        let screen_space_reflection_specular_occlusion_view = Self::require_texture_view_by_name(
            resources,
            resource_resolver,
            screen_space_reflection_specular_occlusion_resource_name,
            RenderGraphResourceAccessKind::Read,
        )?;
        let cluster_buffer = Self::require_buffer_by_name(
            resources,
            resource_resolver,
            PostProcessGraphResourceNames::LIGHT_LIST,
            RenderGraphResourceAccessKind::Read,
        )?;
        stack.post_process.execute_screen_space_reflection_resolve(
            self.device,
            self.queue,
            self.encoder,
            target.cluster_dimensions,
            super::post_process_texture_origin(self.frame, scene_color_resource_name),
            scene_color_view,
            scene_depth_view,
            motion_vector_neighbor_max_view,
            scene_normal_view,
            scene_material_view,
            ambient_occlusion_view,
            None,
            history.map(|history| &history.global_illumination_view),
            history.map(|history| &history.screen_space_reflection_view),
            bloom_view,
            depth_of_field_coc_view,
            depth_of_field_bokeh_view,
            screen_space_reflection_history_view,
            screen_space_reflection_specular_occlusion_view,
            hzb_furthest_sampling_view,
            screen_space_reflection_reflection_pyramid_sampling_view,
            stack.post_process.black_texture_view(),
            screen_space_reflection_reflection_pyramid_coarse_view,
            cluster_buffer,
            self.frame,
            features,
            stack.history_available,
            attachment_ops,
        );
        Ok(())
    }

    pub(in crate::graphics::scene::scene_renderer) fn record_screen_space_reflection_reflection_pyramid_coarse_to_resource(
        &mut self,
        pass_name: &str,
        screen_space_reflection_reflection_pyramid_resource_name: &str,
        screen_space_reflection_reflection_pyramid_coarse_resource_name: &str,
        attachment_ops: RenderGraphAttachmentOps,
    ) -> Result<(), String> {
        let stack = self.post_process_stack.ok_or_else(|| {
            format!(
                "screen-space reflection reflection-pyramid coarse graph executor for pass `{pass_name}` requires post-process stack context"
            )
        })?;
        let target = stack.target;
        let history = stack.history_textures;
        let features = stack.runtime_features;
        let resources = &*self.resources;
        let resource_resolver = self.resource_resolver;
        let scene_color_view = Self::optional_texture_view_by_name(
            resources,
            resource_resolver,
            PostProcessGraphResourceNames::SCENE_COLOR,
            RenderGraphResourceAccessKind::Read,
        )?
        .unwrap_or_else(|| stack.post_process.black_texture_view());
        let scene_depth_view = Self::optional_texture_view_by_name(
            resources,
            resource_resolver,
            PostProcessGraphResourceNames::SCENE_DEPTH,
            RenderGraphResourceAccessKind::Read,
        )?
        .unwrap_or_else(|| stack.post_process.black_texture_view());
        let motion_vector_neighbor_max_view = Self::optional_texture_view_by_name(
            resources,
            resource_resolver,
            PostProcessGraphResourceNames::MOTION_VECTOR_NEIGHBOR_MAX,
            RenderGraphResourceAccessKind::Read,
        )?
        .unwrap_or_else(|| stack.post_process.black_texture_view());
        let scene_normal_view = Self::optional_texture_view_by_name(
            resources,
            resource_resolver,
            PostProcessGraphResourceNames::GBUFFER_NORMAL,
            RenderGraphResourceAccessKind::Read,
        )?
        .unwrap_or_else(|| stack.post_process.black_texture_view());
        let scene_material_view = stack
            .material_gbuffer_valid
            .then(|| {
                Self::optional_texture_view_by_name(
                    resources,
                    resource_resolver,
                    PostProcessGraphResourceNames::GBUFFER_MATERIAL,
                    RenderGraphResourceAccessKind::Read,
                )
                .map(|view| view.unwrap_or_else(|| stack.post_process.black_texture_view()))
            })
            .transpose()?;
        let ambient_occlusion_view = Self::optional_texture_view_by_name(
            resources,
            resource_resolver,
            PostProcessGraphResourceNames::AMBIENT_OCCLUSION,
            RenderGraphResourceAccessKind::Read,
        )?
        .unwrap_or_else(|| stack.post_process.white_texture_view());
        let bloom_view = Self::optional_texture_view_by_name(
            resources,
            resource_resolver,
            PostProcessGraphResourceNames::BLOOM,
            RenderGraphResourceAccessKind::Read,
        )?
        .unwrap_or_else(|| stack.post_process.black_texture_view());
        let depth_of_field_coc_view = Self::optional_texture_view_by_name(
            resources,
            resource_resolver,
            PostProcessGraphResourceNames::DEPTH_OF_FIELD_COC,
            RenderGraphResourceAccessKind::Read,
        )?
        .unwrap_or_else(|| stack.post_process.black_texture_view());
        let depth_of_field_bokeh_view = Self::optional_texture_view_by_name(
            resources,
            resource_resolver,
            PostProcessGraphResourceNames::DEPTH_OF_FIELD_BOKEH,
            RenderGraphResourceAccessKind::Read,
        )?
        .unwrap_or_else(|| stack.post_process.black_texture_view());
        let screen_space_reflection_reflection_pyramid_view = Self::require_texture_view_by_name(
            resources,
            resource_resolver,
            screen_space_reflection_reflection_pyramid_resource_name,
            RenderGraphResourceAccessKind::Read,
        )?;
        let screen_space_reflection_reflection_pyramid_coarse_view =
            Self::require_texture_view_by_name(
                resources,
                resource_resolver,
                screen_space_reflection_reflection_pyramid_coarse_resource_name,
                RenderGraphResourceAccessKind::Write,
            )?;
        let cluster_buffer = Self::require_buffer_by_name(
            resources,
            resource_resolver,
            PostProcessGraphResourceNames::LIGHT_LIST,
            RenderGraphResourceAccessKind::Read,
        )?;
        let mip_level_count = Self::owned_texture_mip_level_count_by_name(
            resources,
            resource_resolver,
            screen_space_reflection_reflection_pyramid_resource_name,
            RenderGraphResourceAccessKind::Read,
        )?;
        if mip_level_count > 1 {
            for mip_pass in ssr_parent_pyramid_mip_passes(mip_level_count, attachment_ops) {
                let source_view = Self::require_owned_texture_mip_view_by_name(
                    resources,
                    resource_resolver,
                    screen_space_reflection_reflection_pyramid_resource_name,
                    screen_space_reflection_reflection_pyramid_resource_name,
                    RenderGraphResourceAccessKind::Read,
                    mip_pass.source_mip_level,
                )?;
                let target_view = Self::require_owned_texture_mip_view_by_name(
                    resources,
                    resource_resolver,
                    screen_space_reflection_reflection_pyramid_coarse_resource_name,
                    screen_space_reflection_reflection_pyramid_resource_name,
                    RenderGraphResourceAccessKind::Write,
                    mip_pass.target_mip_level,
                )?;
                stack
                    .post_process
                    .execute_screen_space_reflection_reflection_pyramid_coarse(
                        self.device,
                        self.queue,
                        self.encoder,
                        target.cluster_dimensions,
                        super::post_process_texture_origin(
                            self.frame,
                            PostProcessGraphResourceNames::SCENE_COLOR,
                        ),
                        scene_color_view,
                        scene_depth_view,
                        motion_vector_neighbor_max_view,
                        scene_normal_view,
                        scene_material_view,
                        ambient_occlusion_view,
                        None,
                        history.map(|history| &history.global_illumination_view),
                        history.map(|history| &history.screen_space_reflection_view),
                        bloom_view,
                        depth_of_field_coc_view,
                        depth_of_field_bokeh_view,
                        &source_view,
                        &target_view,
                        cluster_buffer,
                        self.frame,
                        features,
                        stack.history_available,
                        mip_pass.attachment_ops,
                    );
            }
            return Ok(());
        }
        stack
            .post_process
            .execute_screen_space_reflection_reflection_pyramid_coarse(
                self.device,
                self.queue,
                self.encoder,
                target.cluster_dimensions,
                super::post_process_texture_origin(
                    self.frame,
                    PostProcessGraphResourceNames::SCENE_COLOR,
                ),
                scene_color_view,
                scene_depth_view,
                motion_vector_neighbor_max_view,
                scene_normal_view,
                scene_material_view,
                ambient_occlusion_view,
                None,
                history.map(|history| &history.global_illumination_view),
                history.map(|history| &history.screen_space_reflection_view),
                bloom_view,
                depth_of_field_coc_view,
                depth_of_field_bokeh_view,
                screen_space_reflection_reflection_pyramid_view,
                screen_space_reflection_reflection_pyramid_coarse_view,
                cluster_buffer,
                self.frame,
                features,
                stack.history_available,
                attachment_ops,
            );
        Ok(())
    }

    pub(in crate::graphics::scene::scene_renderer) fn record_screen_space_reflection_reflection_pyramid_to_resource(
        &mut self,
        pass_name: &str,
        scene_color_resource_name: &str,
        screen_space_reflection_reflection_pyramid_resource_name: &str,
        attachment_ops: RenderGraphAttachmentOps,
    ) -> Result<(), String> {
        let stack = self.post_process_stack.ok_or_else(|| {
            format!(
                "screen-space reflection reflection-pyramid graph executor for pass `{pass_name}` requires post-process stack context"
            )
        })?;
        let target = stack.target;
        let history = stack.history_textures;
        let features = stack.runtime_features;
        let resources = &*self.resources;
        let resource_resolver = self.resource_resolver;
        let scene_color_view = Self::require_texture_view_by_name(
            resources,
            resource_resolver,
            scene_color_resource_name,
            RenderGraphResourceAccessKind::Read,
        )?;
        let scene_depth_view = Self::optional_texture_view_by_name(
            resources,
            resource_resolver,
            PostProcessGraphResourceNames::SCENE_DEPTH,
            RenderGraphResourceAccessKind::Read,
        )?
        .unwrap_or_else(|| stack.post_process.black_texture_view());
        let motion_vector_neighbor_max_view = Self::optional_texture_view_by_name(
            resources,
            resource_resolver,
            PostProcessGraphResourceNames::MOTION_VECTOR_NEIGHBOR_MAX,
            RenderGraphResourceAccessKind::Read,
        )?
        .unwrap_or_else(|| stack.post_process.black_texture_view());
        let scene_normal_view = Self::optional_texture_view_by_name(
            resources,
            resource_resolver,
            PostProcessGraphResourceNames::GBUFFER_NORMAL,
            RenderGraphResourceAccessKind::Read,
        )?
        .unwrap_or_else(|| stack.post_process.black_texture_view());
        let scene_material_view = stack
            .material_gbuffer_valid
            .then(|| {
                Self::optional_texture_view_by_name(
                    resources,
                    resource_resolver,
                    PostProcessGraphResourceNames::GBUFFER_MATERIAL,
                    RenderGraphResourceAccessKind::Read,
                )
                .map(|view| view.unwrap_or_else(|| stack.post_process.black_texture_view()))
            })
            .transpose()?;
        let ambient_occlusion_view = Self::optional_texture_view_by_name(
            resources,
            resource_resolver,
            PostProcessGraphResourceNames::AMBIENT_OCCLUSION,
            RenderGraphResourceAccessKind::Read,
        )?
        .unwrap_or_else(|| stack.post_process.white_texture_view());
        let bloom_view = Self::optional_texture_view_by_name(
            resources,
            resource_resolver,
            PostProcessGraphResourceNames::BLOOM,
            RenderGraphResourceAccessKind::Read,
        )?
        .unwrap_or_else(|| stack.post_process.black_texture_view());
        let depth_of_field_coc_view = Self::optional_texture_view_by_name(
            resources,
            resource_resolver,
            PostProcessGraphResourceNames::DEPTH_OF_FIELD_COC,
            RenderGraphResourceAccessKind::Read,
        )?
        .unwrap_or_else(|| stack.post_process.black_texture_view());
        let depth_of_field_bokeh_view = Self::optional_texture_view_by_name(
            resources,
            resource_resolver,
            PostProcessGraphResourceNames::DEPTH_OF_FIELD_BOKEH,
            RenderGraphResourceAccessKind::Read,
        )?
        .unwrap_or_else(|| stack.post_process.black_texture_view());
        let screen_space_reflection_reflection_pyramid_view = Self::require_texture_view_by_name(
            resources,
            resource_resolver,
            screen_space_reflection_reflection_pyramid_resource_name,
            RenderGraphResourceAccessKind::Write,
        )?;
        let cluster_buffer = Self::require_buffer_by_name(
            resources,
            resource_resolver,
            PostProcessGraphResourceNames::LIGHT_LIST,
            RenderGraphResourceAccessKind::Read,
        )?;
        stack
            .post_process
            .execute_screen_space_reflection_reflection_pyramid(
                self.device,
                self.queue,
                self.encoder,
                target.cluster_dimensions,
                super::post_process_texture_origin(self.frame, scene_color_resource_name),
                scene_color_view,
                scene_depth_view,
                motion_vector_neighbor_max_view,
                scene_normal_view,
                scene_material_view,
                ambient_occlusion_view,
                None,
                history.map(|history| &history.global_illumination_view),
                history.map(|history| &history.screen_space_reflection_view),
                bloom_view,
                depth_of_field_coc_view,
                depth_of_field_bokeh_view,
                screen_space_reflection_reflection_pyramid_view,
                cluster_buffer,
                self.frame,
                features,
                stack.history_available,
                attachment_ops,
            );
        Ok(())
    }

    pub(in crate::graphics::scene::scene_renderer) fn record_screen_space_reflection_specular_occlusion_to_resource(
        &mut self,
        pass_name: &str,
        scene_depth_resource_name: &str,
        screen_space_reflection_specular_occlusion_resource_name: &str,
        attachment_ops: RenderGraphAttachmentOps,
    ) -> Result<(), String> {
        let stack = self.post_process_stack.ok_or_else(|| {
            format!(
                "screen-space reflection specular occlusion graph executor for pass `{pass_name}` requires post-process stack context"
            )
        })?;
        let target = stack.target;
        let history = stack.history_textures;
        let features = stack.runtime_features;
        let resources = &*self.resources;
        let resource_resolver = self.resource_resolver;
        let scene_color_view = Self::optional_texture_view_by_name(
            resources,
            resource_resolver,
            PostProcessGraphResourceNames::SCENE_COLOR,
            RenderGraphResourceAccessKind::Read,
        )?
        .unwrap_or_else(|| stack.post_process.black_texture_view());
        let scene_depth_view = Self::require_texture_view_by_name(
            resources,
            resource_resolver,
            scene_depth_resource_name,
            RenderGraphResourceAccessKind::Read,
        )?;
        let motion_vector_neighbor_max_view = Self::optional_texture_view_by_name(
            resources,
            resource_resolver,
            PostProcessGraphResourceNames::MOTION_VECTOR_NEIGHBOR_MAX,
            RenderGraphResourceAccessKind::Read,
        )?
        .unwrap_or_else(|| stack.post_process.black_texture_view());
        let scene_normal_view = Self::optional_texture_view_by_name(
            resources,
            resource_resolver,
            PostProcessGraphResourceNames::GBUFFER_NORMAL,
            RenderGraphResourceAccessKind::Read,
        )?
        .unwrap_or_else(|| stack.post_process.black_texture_view());
        let scene_material_view = stack
            .material_gbuffer_valid
            .then(|| {
                Self::require_texture_view_by_name(
                    resources,
                    resource_resolver,
                    PostProcessGraphResourceNames::GBUFFER_MATERIAL,
                    RenderGraphResourceAccessKind::Read,
                )
            })
            .transpose()?;
        let ambient_occlusion_view = Self::require_texture_view_by_name(
            resources,
            resource_resolver,
            PostProcessGraphResourceNames::AMBIENT_OCCLUSION,
            RenderGraphResourceAccessKind::Read,
        )?;
        let bloom_view = Self::optional_texture_view_by_name(
            resources,
            resource_resolver,
            PostProcessGraphResourceNames::BLOOM,
            RenderGraphResourceAccessKind::Read,
        )?
        .unwrap_or_else(|| stack.post_process.black_texture_view());
        let depth_of_field_coc_view = Self::optional_texture_view_by_name(
            resources,
            resource_resolver,
            PostProcessGraphResourceNames::DEPTH_OF_FIELD_COC,
            RenderGraphResourceAccessKind::Read,
        )?
        .unwrap_or_else(|| stack.post_process.black_texture_view());
        let depth_of_field_bokeh_view = Self::optional_texture_view_by_name(
            resources,
            resource_resolver,
            PostProcessGraphResourceNames::DEPTH_OF_FIELD_BOKEH,
            RenderGraphResourceAccessKind::Read,
        )?
        .unwrap_or_else(|| stack.post_process.black_texture_view());
        let screen_space_reflection_specular_occlusion_view = Self::require_texture_view_by_name(
            resources,
            resource_resolver,
            screen_space_reflection_specular_occlusion_resource_name,
            RenderGraphResourceAccessKind::Write,
        )?;
        let cluster_buffer = Self::require_buffer_by_name(
            resources,
            resource_resolver,
            PostProcessGraphResourceNames::LIGHT_LIST,
            RenderGraphResourceAccessKind::Read,
        )?;
        stack
            .post_process
            .execute_screen_space_reflection_specular_occlusion(
                self.device,
                self.queue,
                self.encoder,
                target.cluster_dimensions,
                super::post_process_texture_origin(
                    self.frame,
                    PostProcessGraphResourceNames::SCENE_COLOR,
                ),
                scene_color_view,
                scene_depth_view,
                motion_vector_neighbor_max_view,
                scene_normal_view,
                scene_material_view,
                ambient_occlusion_view,
                None,
                history.map(|history| &history.global_illumination_view),
                history.map(|history| &history.screen_space_reflection_view),
                bloom_view,
                depth_of_field_coc_view,
                depth_of_field_bokeh_view,
                screen_space_reflection_specular_occlusion_view,
                cluster_buffer,
                self.frame,
                features,
                stack.history_available,
                attachment_ops,
            );
        Ok(())
    }
}

#[cfg(test)]
mod tests;
