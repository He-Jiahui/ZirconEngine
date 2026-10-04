use super::scene_bind_group_layout_entries;

#[test]
fn scene_bind_group_layout_includes_source_specular_and_irradiance_cubemaps() {
    let entries = scene_bind_group_layout_entries();

    assert_eq!(
        entries
            .iter()
            .map(|entry| entry.binding)
            .collect::<Vec<_>>(),
        vec![0, 1, 2, 3, 4, 5, 6]
    );
    assert_cube_texture(&entries[1]);
    assert_cube_texture(&entries[4]);
    assert_cube_texture(&entries[5]);
    assert!(matches!(
        entries[6].ty,
        wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            ..
        }
    ));
}

fn assert_cube_texture(entry: &wgpu::BindGroupLayoutEntry) {
    let wgpu::BindingType::Texture {
        multisampled: false,
        view_dimension: wgpu::TextureViewDimension::Cube,
        sample_type: wgpu::TextureSampleType::Float { filterable: true },
    } = &entry.ty
    else {
        panic!(
            "entry {} should be a filterable cube texture",
            entry.binding
        );
    };
}
