mod build_particle_vertices;
mod particle_renderer;
mod particle_vertex;

pub(crate) use particle_renderer::ParticleRenderer;

#[cfg(test)]
mod tests {
    #[test]
    fn particle_pipeline_keeps_world_hud_billboards_transparent_and_depth_read_only() {
        let source = include_str!("particle_renderer/new.rs");

        assert!(
            source.contains("depth_write_enabled: Some(false)"),
            "particle/world-HUD billboards must not write scene depth"
        );
        assert!(
            source.contains("depth_compare: Some(wgpu::CompareFunction::LessEqual)"),
            "particle/world-HUD billboards should still be depth-tested against opaque scene geometry"
        );
        assert!(
            source.contains("src_factor: wgpu::BlendFactor::SrcAlpha")
                && source.contains("dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha"),
            "particle/world-HUD billboards should render through the transparent blend path"
        );
    }
}
