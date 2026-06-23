use super::*;

pub(super) fn assert_ready_record(manager: &ProjectManager, uri: &str, kind: AssetKind) {
    let record = resource_record(manager, uri);
    assert_eq!(record.kind, kind);
    assert_eq!(record.state, ResourceState::Ready);
    assert!(
        record.diagnostics.is_empty(),
        "{uri} should not emit diagnostics: {:?}",
        record.diagnostics
    );
}

pub(super) fn assert_dependencies(manager: &ProjectManager, from: &str, expected: &[&str]) {
    let record = resource_record(manager, from);
    let mut actual = record.dependency_ids.clone();
    actual.sort();
    let mut expected_ids = expected
        .iter()
        .map(|dependency| resource_record(manager, dependency).id())
        .collect::<Vec<_>>();
    expected_ids.sort();
    assert_eq!(
        actual, expected_ids,
        "{from} dependencies should match the project sample graph"
    );
}

pub(super) fn assert_loaded_with_dependencies<TAsset: Asset>(
    manager: &ProjectAssetManager,
    uri_text: &str,
) {
    let handle = manager.load::<TAsset>(&uri(uri_text)).unwrap();
    assert_eq!(manager.load_state(handle), AssetLoadState::Loaded);
    assert_eq!(
        manager.dependency_load_state(handle),
        DependencyLoadState::Loaded
    );
    assert_eq!(
        manager.recursive_dependency_load_state(handle),
        RecursiveDependencyLoadState::Loaded
    );
    assert_eq!(
        manager.load_states(handle),
        AssetLoadStates {
            load_state: AssetLoadState::Loaded,
            dependency_load_state: DependencyLoadState::Loaded,
            recursive_dependency_load_state: RecursiveDependencyLoadState::Loaded,
        }
    );
    assert!(manager.is_loaded_with_dependencies(handle));
}

pub(super) fn resource_record<'a>(
    manager: &'a ProjectManager,
    uri_text: &str,
) -> &'a crate::core::resource::ResourceRecord {
    manager
        .registry()
        .get_by_locator(&uri(uri_text))
        .unwrap_or_else(|| panic!("missing resource record for {uri_text}"))
}

pub(super) fn load_model(manager: &ProjectManager, uri_text: &str) -> crate::asset::ModelAsset {
    match manager.load_artifact(&uri(uri_text)).unwrap() {
        ImportedAsset::Model(model) => model,
        other => panic!("unexpected model artifact for {uri_text}: {other:?}"),
    }
}

pub(super) fn load_mesh(manager: &ProjectManager, uri_text: &str) -> crate::asset::MeshAsset {
    match manager.load_artifact(&uri(uri_text)).unwrap() {
        ImportedAsset::Mesh(mesh) => mesh,
        other => panic!("unexpected mesh artifact for {uri_text}: {other:?}"),
    }
}

pub(super) fn load_scene(manager: &ProjectManager, uri_text: &str) -> crate::asset::SceneAsset {
    match manager.load_artifact(&uri(uri_text)).unwrap() {
        ImportedAsset::Scene(scene) => scene,
        other => panic!("unexpected scene artifact for {uri_text}: {other:?}"),
    }
}

pub(super) fn load_material(manager: &ProjectManager, uri_text: &str) -> MaterialAsset {
    match manager.load_artifact(&uri(uri_text)).unwrap() {
        ImportedAsset::Material(material) => material,
        other => panic!("unexpected material artifact for {uri_text}: {other:?}"),
    }
}

pub(super) fn load_shader(manager: &ProjectManager, uri_text: &str) -> crate::asset::ShaderAsset {
    match manager.load_artifact(&uri(uri_text)).unwrap() {
        ImportedAsset::Shader(shader) => shader,
        other => panic!("unexpected shader artifact for {uri_text}: {other:?}"),
    }
}

pub(super) fn load_texture(manager: &ProjectManager, uri_text: &str) -> TextureAsset {
    match manager.load_artifact(&uri(uri_text)).unwrap() {
        ImportedAsset::Texture(texture) => texture,
        other => panic!("unexpected texture artifact for {uri_text}: {other:?}"),
    }
}

pub(super) fn texture_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("zircon-project-asset-flow-texture-layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    multisampled: false,
                    view_dimension: wgpu::TextureViewDimension::D2,
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ],
    })
}

pub(super) fn uri(value: &str) -> AssetUri {
    AssetUri::parse(value).unwrap()
}
