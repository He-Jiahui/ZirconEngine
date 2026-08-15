use crate::core::framework::render::{LightProbeGridData, ShL2Rgb};
use crate::core::math::Vec3;
use crate::graphics::scene::scene_renderer::advanced_lighting::froxel::volumetric_apply_bind_group_layout_entries;

use super::*;

#[test]
fn render_env_lightmap_gpu_layout_uses_reserved_bindings() {
    let entries = lightmap_bind_group_layout_entries();

    assert_eq!(
        entries.map(|entry| entry.binding),
        [
            LIGHT_PROBE_GRID_BINDING,
            LIGHTMAP_ATLAS_BINDING,
            LIGHTMAP_SAMPLER_BINDING,
        ]
    );
    assert!(matches!(
        entries[0].ty,
        wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only: true },
            ..
        }
    ));
    assert!(matches!(
        entries[1].ty,
        wgpu::BindingType::Texture {
            view_dimension: wgpu::TextureViewDimension::D2Array,
            sample_type: wgpu::TextureSampleType::Float { filterable: true },
            ..
        }
    ));
}

#[test]
fn render_env_lightmap_bindings_do_not_overlap_volumetric_apply() {
    let lightmap_bindings = [
        LIGHT_PROBE_GRID_BINDING,
        LIGHTMAP_ATLAS_BINDING,
        LIGHTMAP_SAMPLER_BINDING,
    ];
    let volumetric_bindings =
        volumetric_apply_bind_group_layout_entries(wgpu::ShaderStages::FRAGMENT)
            .map(|entry| entry.binding);

    assert_eq!(LIGHTMAP_SAMPLER_BINDING, 28);
    assert!(lightmap_bindings
        .iter()
        .all(|binding| !volumetric_bindings.contains(binding)));
}

#[test]
fn render_env_probe_grid_gpu_storage_preserves_header_and_sh9_order() {
    let mut first = ShL2Rgb::default();
    first.0[0] = Vec3::new(1.0, 2.0, 3.0);
    let grid = LightProbeGridData {
        light_set_generation: 0x0000_0002_0000_0001,
        bounds_min: Vec3::new(-4.0, -3.0, -2.0),
        cell_size: Vec3::new(1.0, 2.0, 3.0),
        dims: [1, 1, 1],
        sh: vec![first],
    };

    let words = encode_light_probe_grid_storage(&grid).expect("grid should encode");

    assert_eq!(words.len(), LIGHT_PROBE_GRID_HEADER_WORDS + 9);
    assert_eq!(words[0], [-4.0, -3.0, -2.0, 1.0]);
    assert_eq!(words[1][0..2], [2.0, 3.0]);
    assert_eq!(words[1][2].to_bits(), 1);
    assert_eq!(words[1][3].to_bits(), 1);
    assert_eq!(words[2][0].to_bits(), 1);
    assert_eq!(words[2][1].to_bits(), 1);
    assert_eq!(words[2][2].to_bits(), 2);
    assert_eq!(words[2][3].to_bits(), 1);
    assert_eq!(words[3], [1.0, 2.0, 3.0, 0.0]);
}
