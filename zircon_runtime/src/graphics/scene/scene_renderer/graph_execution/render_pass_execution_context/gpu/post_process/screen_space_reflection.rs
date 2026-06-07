use crate::core::framework::render::PostProcessGraphResourceNames;
use crate::render_graph::RenderGraphAttachmentOps;

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
        screen_space_reflection_depth_pyramid_resource_name: &str,
        screen_space_reflection_reflection_pyramid_resource_name: &str,
        screen_space_reflection_depth_pyramid_coarse_resource_name: &str,
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
        let scene_color_view = self
            .resources
            .require_texture_view(scene_color_resource_name)?;
        let scene_depth_view = self
            .resources
            .require_texture_view(scene_depth_resource_name)?;
        let motion_vector_neighbor_max_view = self
            .resources
            .require_texture_view(motion_vector_neighbor_max_resource_name)?;
        let scene_normal_view = self
            .resources
            .require_texture_view(PostProcessGraphResourceNames::GBUFFER_NORMAL)?;
        let scene_material_view = stack
            .material_gbuffer_valid
            .then(|| {
                self.resources
                    .require_texture_view(PostProcessGraphResourceNames::GBUFFER_MATERIAL)
            })
            .transpose()?;
        let ambient_occlusion_view = self
            .resources
            .require_texture_view(PostProcessGraphResourceNames::AMBIENT_OCCLUSION)?;
        let bloom_view = self
            .resources
            .require_texture_view(PostProcessGraphResourceNames::BLOOM)?;
        let depth_of_field_coc_view = self
            .resources
            .require_texture_view(PostProcessGraphResourceNames::DEPTH_OF_FIELD_COC)?;
        let depth_of_field_bokeh_view = self
            .resources
            .require_texture_view(PostProcessGraphResourceNames::DEPTH_OF_FIELD_BOKEH)?;
        let screen_space_reflection_history_view = self
            .resources
            .require_texture_view(screen_space_reflection_history_resource_name)?;
        let screen_space_reflection_depth_pyramid_view = self
            .resources
            .require_texture_view(screen_space_reflection_depth_pyramid_resource_name)?;
        let screen_space_reflection_reflection_pyramid_view = self
            .resources
            .require_texture_view(screen_space_reflection_reflection_pyramid_resource_name)?;
        let screen_space_reflection_depth_pyramid_full_mip_view = self
            .resources
            .owned_texture_full_mip_view(screen_space_reflection_depth_pyramid_resource_name)
            .ok();
        let screen_space_reflection_reflection_pyramid_full_mip_view = self
            .resources
            .owned_texture_full_mip_view(screen_space_reflection_reflection_pyramid_resource_name)
            .ok();
        let screen_space_reflection_depth_pyramid_sampling_view =
            screen_space_reflection_depth_pyramid_full_mip_view
                .as_ref()
                .unwrap_or(screen_space_reflection_depth_pyramid_view);
        let screen_space_reflection_reflection_pyramid_sampling_view =
            screen_space_reflection_reflection_pyramid_full_mip_view
                .as_ref()
                .unwrap_or(screen_space_reflection_reflection_pyramid_view);
        let screen_space_reflection_depth_pyramid_coarse_view = self
            .resources
            .require_texture_view(screen_space_reflection_depth_pyramid_coarse_resource_name)?;
        let screen_space_reflection_reflection_pyramid_coarse_view =
            self.resources.require_texture_view(
                screen_space_reflection_reflection_pyramid_coarse_resource_name,
            )?;
        let screen_space_reflection_specular_occlusion_view = self
            .resources
            .require_texture_view(screen_space_reflection_specular_occlusion_resource_name)?;
        let cluster_buffer = self
            .resources
            .require_buffer(PostProcessGraphResourceNames::LIGHT_LIST)?;
        stack.post_process.execute_screen_space_reflection_resolve(
            self.device,
            self.queue,
            self.encoder,
            target.size,
            target.cluster_dimensions,
            scene_color_view,
            scene_depth_view,
            motion_vector_neighbor_max_view,
            scene_normal_view,
            scene_material_view,
            ambient_occlusion_view,
            history.map(|history| &history.scene_color_view),
            history.map(|history| &history.global_illumination_view),
            history.map(|history| &history.screen_space_reflection_view),
            bloom_view,
            depth_of_field_coc_view,
            depth_of_field_bokeh_view,
            screen_space_reflection_history_view,
            screen_space_reflection_specular_occlusion_view,
            screen_space_reflection_depth_pyramid_sampling_view,
            screen_space_reflection_reflection_pyramid_sampling_view,
            screen_space_reflection_depth_pyramid_coarse_view,
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
        let scene_color_view = self
            .resources
            .require_texture_view(PostProcessGraphResourceNames::SCENE_COLOR)?;
        let scene_depth_view = self
            .resources
            .require_texture_view(PostProcessGraphResourceNames::SCENE_DEPTH)?;
        let motion_vector_neighbor_max_view = self
            .resources
            .require_texture_view(PostProcessGraphResourceNames::MOTION_VECTOR_NEIGHBOR_MAX)?;
        let scene_normal_view = self
            .resources
            .require_texture_view(PostProcessGraphResourceNames::GBUFFER_NORMAL)?;
        let scene_material_view = stack
            .material_gbuffer_valid
            .then(|| {
                self.resources
                    .require_texture_view(PostProcessGraphResourceNames::GBUFFER_MATERIAL)
            })
            .transpose()?;
        let ambient_occlusion_view = self
            .resources
            .require_texture_view(PostProcessGraphResourceNames::AMBIENT_OCCLUSION)?;
        let bloom_view = self
            .resources
            .require_texture_view(PostProcessGraphResourceNames::BLOOM)?;
        let depth_of_field_coc_view = self
            .resources
            .require_texture_view(PostProcessGraphResourceNames::DEPTH_OF_FIELD_COC)?;
        let depth_of_field_bokeh_view = self
            .resources
            .require_texture_view(PostProcessGraphResourceNames::DEPTH_OF_FIELD_BOKEH)?;
        let screen_space_reflection_reflection_pyramid_view = self
            .resources
            .require_texture_view(screen_space_reflection_reflection_pyramid_resource_name)?;
        let screen_space_reflection_reflection_pyramid_coarse_view =
            self.resources.require_texture_view(
                screen_space_reflection_reflection_pyramid_coarse_resource_name,
            )?;
        let cluster_buffer = self
            .resources
            .require_buffer(PostProcessGraphResourceNames::LIGHT_LIST)?;
        let mip_level_count = self
            .resources
            .owned_texture_mip_level_count(screen_space_reflection_reflection_pyramid_resource_name)
            .unwrap_or(1);
        if mip_level_count > 1 {
            for mip_pass in ssr_parent_pyramid_mip_passes(mip_level_count, attachment_ops) {
                let source_view = self.resources.owned_texture_mip_view(
                    screen_space_reflection_reflection_pyramid_resource_name,
                    mip_pass.source_mip_level,
                )?;
                let target_view = self.resources.owned_texture_mip_view(
                    screen_space_reflection_reflection_pyramid_resource_name,
                    mip_pass.target_mip_level,
                )?;
                stack
                    .post_process
                    .execute_screen_space_reflection_reflection_pyramid_coarse(
                        self.device,
                        self.queue,
                        self.encoder,
                        target.size,
                        target.cluster_dimensions,
                        scene_color_view,
                        scene_depth_view,
                        motion_vector_neighbor_max_view,
                        scene_normal_view,
                        scene_material_view,
                        ambient_occlusion_view,
                        history.map(|history| &history.scene_color_view),
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
                target.size,
                target.cluster_dimensions,
                scene_color_view,
                scene_depth_view,
                motion_vector_neighbor_max_view,
                scene_normal_view,
                scene_material_view,
                ambient_occlusion_view,
                history.map(|history| &history.scene_color_view),
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
        let scene_color_view = self
            .resources
            .require_texture_view(scene_color_resource_name)?;
        let scene_depth_view = self
            .resources
            .require_texture_view(PostProcessGraphResourceNames::SCENE_DEPTH)?;
        let motion_vector_neighbor_max_view = self
            .resources
            .require_texture_view(PostProcessGraphResourceNames::MOTION_VECTOR_NEIGHBOR_MAX)?;
        let scene_normal_view = self
            .resources
            .require_texture_view(PostProcessGraphResourceNames::GBUFFER_NORMAL)?;
        let scene_material_view = stack
            .material_gbuffer_valid
            .then(|| {
                self.resources
                    .require_texture_view(PostProcessGraphResourceNames::GBUFFER_MATERIAL)
            })
            .transpose()?;
        let ambient_occlusion_view = self
            .resources
            .require_texture_view(PostProcessGraphResourceNames::AMBIENT_OCCLUSION)?;
        let bloom_view = self
            .resources
            .require_texture_view(PostProcessGraphResourceNames::BLOOM)?;
        let depth_of_field_coc_view = self
            .resources
            .require_texture_view(PostProcessGraphResourceNames::DEPTH_OF_FIELD_COC)?;
        let depth_of_field_bokeh_view = self
            .resources
            .require_texture_view(PostProcessGraphResourceNames::DEPTH_OF_FIELD_BOKEH)?;
        let screen_space_reflection_reflection_pyramid_view = self
            .resources
            .require_texture_view(screen_space_reflection_reflection_pyramid_resource_name)?;
        let cluster_buffer = self
            .resources
            .require_buffer(PostProcessGraphResourceNames::LIGHT_LIST)?;
        stack
            .post_process
            .execute_screen_space_reflection_reflection_pyramid(
                self.device,
                self.queue,
                self.encoder,
                target.size,
                target.cluster_dimensions,
                scene_color_view,
                scene_depth_view,
                motion_vector_neighbor_max_view,
                scene_normal_view,
                scene_material_view,
                ambient_occlusion_view,
                history.map(|history| &history.scene_color_view),
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

    pub(in crate::graphics::scene::scene_renderer) fn record_screen_space_reflection_depth_pyramid_coarse_to_resource(
        &mut self,
        pass_name: &str,
        screen_space_reflection_depth_pyramid_resource_name: &str,
        screen_space_reflection_depth_pyramid_coarse_resource_name: &str,
        attachment_ops: RenderGraphAttachmentOps,
    ) -> Result<(), String> {
        let stack = self.post_process_stack.ok_or_else(|| {
            format!(
                "screen-space reflection depth-pyramid coarse graph executor for pass `{pass_name}` requires post-process stack context"
            )
        })?;
        let target = stack.target;
        let history = stack.history_textures;
        let features = stack.runtime_features;
        let scene_color_view = self
            .resources
            .require_texture_view(PostProcessGraphResourceNames::SCENE_COLOR)?;
        let scene_depth_view = self
            .resources
            .require_texture_view(PostProcessGraphResourceNames::SCENE_DEPTH)?;
        let motion_vector_neighbor_max_view = self
            .resources
            .require_texture_view(PostProcessGraphResourceNames::MOTION_VECTOR_NEIGHBOR_MAX)?;
        let scene_normal_view = self
            .resources
            .require_texture_view(PostProcessGraphResourceNames::GBUFFER_NORMAL)?;
        let scene_material_view = stack
            .material_gbuffer_valid
            .then(|| {
                self.resources
                    .require_texture_view(PostProcessGraphResourceNames::GBUFFER_MATERIAL)
            })
            .transpose()?;
        let ambient_occlusion_view = self
            .resources
            .require_texture_view(PostProcessGraphResourceNames::AMBIENT_OCCLUSION)?;
        let bloom_view = self
            .resources
            .require_texture_view(PostProcessGraphResourceNames::BLOOM)?;
        let depth_of_field_coc_view = self
            .resources
            .require_texture_view(PostProcessGraphResourceNames::DEPTH_OF_FIELD_COC)?;
        let depth_of_field_bokeh_view = self
            .resources
            .require_texture_view(PostProcessGraphResourceNames::DEPTH_OF_FIELD_BOKEH)?;
        let screen_space_reflection_depth_pyramid_view = self
            .resources
            .require_texture_view(screen_space_reflection_depth_pyramid_resource_name)?;
        let screen_space_reflection_depth_pyramid_coarse_view = self
            .resources
            .require_texture_view(screen_space_reflection_depth_pyramid_coarse_resource_name)?;
        let cluster_buffer = self
            .resources
            .require_buffer(PostProcessGraphResourceNames::LIGHT_LIST)?;
        let mip_level_count = self
            .resources
            .owned_texture_mip_level_count(screen_space_reflection_depth_pyramid_resource_name)
            .unwrap_or(1);
        if mip_level_count > 1 {
            for mip_pass in ssr_parent_pyramid_mip_passes(mip_level_count, attachment_ops) {
                let source_view = self.resources.owned_texture_mip_view(
                    screen_space_reflection_depth_pyramid_resource_name,
                    mip_pass.source_mip_level,
                )?;
                let target_view = self.resources.owned_texture_mip_view(
                    screen_space_reflection_depth_pyramid_resource_name,
                    mip_pass.target_mip_level,
                )?;
                stack
                    .post_process
                    .execute_screen_space_reflection_depth_pyramid_coarse(
                        self.device,
                        self.queue,
                        self.encoder,
                        target.size,
                        target.cluster_dimensions,
                        scene_color_view,
                        scene_depth_view,
                        motion_vector_neighbor_max_view,
                        scene_normal_view,
                        scene_material_view,
                        ambient_occlusion_view,
                        history.map(|history| &history.scene_color_view),
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
            .execute_screen_space_reflection_depth_pyramid_coarse(
                self.device,
                self.queue,
                self.encoder,
                target.size,
                target.cluster_dimensions,
                scene_color_view,
                scene_depth_view,
                motion_vector_neighbor_max_view,
                scene_normal_view,
                scene_material_view,
                ambient_occlusion_view,
                history.map(|history| &history.scene_color_view),
                history.map(|history| &history.global_illumination_view),
                history.map(|history| &history.screen_space_reflection_view),
                bloom_view,
                depth_of_field_coc_view,
                depth_of_field_bokeh_view,
                screen_space_reflection_depth_pyramid_view,
                screen_space_reflection_depth_pyramid_coarse_view,
                cluster_buffer,
                self.frame,
                features,
                stack.history_available,
                attachment_ops,
            );
        Ok(())
    }

    pub(in crate::graphics::scene::scene_renderer) fn record_screen_space_reflection_depth_pyramid_to_resource(
        &mut self,
        pass_name: &str,
        scene_depth_resource_name: &str,
        screen_space_reflection_depth_pyramid_resource_name: &str,
        attachment_ops: RenderGraphAttachmentOps,
    ) -> Result<(), String> {
        let stack = self.post_process_stack.ok_or_else(|| {
            format!(
                "screen-space reflection depth-pyramid graph executor for pass `{pass_name}` requires post-process stack context"
            )
        })?;
        let target = stack.target;
        let history = stack.history_textures;
        let features = stack.runtime_features;
        let scene_color_view = self
            .resources
            .require_texture_view(PostProcessGraphResourceNames::SCENE_COLOR)?;
        let scene_depth_view = self
            .resources
            .require_texture_view(scene_depth_resource_name)?;
        let motion_vector_neighbor_max_view = self
            .resources
            .require_texture_view(PostProcessGraphResourceNames::MOTION_VECTOR_NEIGHBOR_MAX)?;
        let scene_normal_view = self
            .resources
            .require_texture_view(PostProcessGraphResourceNames::GBUFFER_NORMAL)?;
        let scene_material_view = stack
            .material_gbuffer_valid
            .then(|| {
                self.resources
                    .require_texture_view(PostProcessGraphResourceNames::GBUFFER_MATERIAL)
            })
            .transpose()?;
        let ambient_occlusion_view = self
            .resources
            .require_texture_view(PostProcessGraphResourceNames::AMBIENT_OCCLUSION)?;
        let bloom_view = self
            .resources
            .require_texture_view(PostProcessGraphResourceNames::BLOOM)?;
        let depth_of_field_coc_view = self
            .resources
            .require_texture_view(PostProcessGraphResourceNames::DEPTH_OF_FIELD_COC)?;
        let depth_of_field_bokeh_view = self
            .resources
            .require_texture_view(PostProcessGraphResourceNames::DEPTH_OF_FIELD_BOKEH)?;
        let screen_space_reflection_depth_pyramid_view = self
            .resources
            .require_texture_view(screen_space_reflection_depth_pyramid_resource_name)?;
        let cluster_buffer = self
            .resources
            .require_buffer(PostProcessGraphResourceNames::LIGHT_LIST)?;
        stack
            .post_process
            .execute_screen_space_reflection_depth_pyramid(
                self.device,
                self.queue,
                self.encoder,
                target.size,
                target.cluster_dimensions,
                scene_color_view,
                scene_depth_view,
                motion_vector_neighbor_max_view,
                scene_normal_view,
                scene_material_view,
                ambient_occlusion_view,
                history.map(|history| &history.scene_color_view),
                history.map(|history| &history.global_illumination_view),
                history.map(|history| &history.screen_space_reflection_view),
                bloom_view,
                depth_of_field_coc_view,
                depth_of_field_bokeh_view,
                screen_space_reflection_depth_pyramid_view,
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
        let scene_color_view = self
            .resources
            .require_texture_view(PostProcessGraphResourceNames::SCENE_COLOR)?;
        let scene_depth_view = self
            .resources
            .require_texture_view(scene_depth_resource_name)?;
        let motion_vector_neighbor_max_view = self
            .resources
            .require_texture_view(PostProcessGraphResourceNames::MOTION_VECTOR_NEIGHBOR_MAX)?;
        let scene_normal_view = self
            .resources
            .require_texture_view(PostProcessGraphResourceNames::GBUFFER_NORMAL)?;
        let scene_material_view = stack
            .material_gbuffer_valid
            .then(|| {
                self.resources
                    .require_texture_view(PostProcessGraphResourceNames::GBUFFER_MATERIAL)
            })
            .transpose()?;
        let ambient_occlusion_view = self
            .resources
            .require_texture_view(PostProcessGraphResourceNames::AMBIENT_OCCLUSION)?;
        let bloom_view = self
            .resources
            .require_texture_view(PostProcessGraphResourceNames::BLOOM)?;
        let depth_of_field_coc_view = self
            .resources
            .require_texture_view(PostProcessGraphResourceNames::DEPTH_OF_FIELD_COC)?;
        let depth_of_field_bokeh_view = self
            .resources
            .require_texture_view(PostProcessGraphResourceNames::DEPTH_OF_FIELD_BOKEH)?;
        let screen_space_reflection_specular_occlusion_view = self
            .resources
            .require_texture_view(screen_space_reflection_specular_occlusion_resource_name)?;
        let cluster_buffer = self
            .resources
            .require_buffer(PostProcessGraphResourceNames::LIGHT_LIST)?;
        stack
            .post_process
            .execute_screen_space_reflection_specular_occlusion(
                self.device,
                self.queue,
                self.encoder,
                target.size,
                target.cluster_dimensions,
                scene_color_view,
                scene_depth_view,
                motion_vector_neighbor_max_view,
                scene_normal_view,
                scene_material_view,
                ambient_occlusion_view,
                history.map(|history| &history.scene_color_view),
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
mod tests {
    use super::ssr_parent_pyramid_mip_passes;
    use crate::render_graph::RenderGraphAttachmentOps;

    #[test]
    fn ssr_parent_pyramid_mip_passes_are_empty_for_single_mip_parent() {
        assert_eq!(
            ssr_parent_pyramid_mip_passes(1, RenderGraphAttachmentOps::load_store())
                .collect::<Vec<_>>(),
            Vec::new()
        );
    }

    #[test]
    fn ssr_parent_pyramid_mip_passes_preserve_graph_alias_ops_for_mip_one() {
        let passes = ssr_parent_pyramid_mip_passes(2, RenderGraphAttachmentOps::load_store())
            .collect::<Vec<_>>();

        assert_eq!(passes.len(), 1);
        assert_eq!(passes[0].source_mip_level, 0);
        assert_eq!(passes[0].target_mip_level, 1);
        assert_eq!(
            passes[0].attachment_ops,
            RenderGraphAttachmentOps::load_store()
        );
    }

    #[test]
    fn ssr_parent_pyramid_mip_passes_clear_later_mips_after_graph_alias_mip() {
        let passes = ssr_parent_pyramid_mip_passes(5, RenderGraphAttachmentOps::load_store())
            .collect::<Vec<_>>();

        assert_eq!(
            passes
                .iter()
                .map(|pass| (pass.source_mip_level, pass.target_mip_level))
                .collect::<Vec<_>>(),
            vec![(0, 1), (1, 2), (2, 3), (3, 4)]
        );
        assert_eq!(
            passes[0].attachment_ops,
            RenderGraphAttachmentOps::load_store()
        );
        assert!(passes[1..]
            .iter()
            .all(|pass| pass.attachment_ops == RenderGraphAttachmentOps::clear_store()));
    }
}
