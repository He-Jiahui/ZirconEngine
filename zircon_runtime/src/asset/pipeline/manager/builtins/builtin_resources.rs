use crate::asset::load::mesh::generate_cube_mesh;
use crate::asset::{
    asset_kind_for_imported_asset, AlphaMode, AssetKind, AssetUri, ImportedAsset, MaterialAsset,
    ModelAsset, ModelPrimitiveAsset, ShaderAsset, ShaderSourceLanguage,
};
use crate::core::framework::render::ShaderAssetKind;
use crate::core::resource::ResourceScheme;

use super::{builtin_pbr_wgsl, builtin_reference};

type BuiltinFactory = fn() -> ImportedAsset;

#[derive(Clone, Copy)]
struct BuiltinResourceDescriptor {
    locator: &'static str,
    kind: AssetKind,
    factory: BuiltinFactory,
}

const BUILTIN_RESOURCE_DESCRIPTORS: [BuiltinResourceDescriptor; 5] = [
    BuiltinResourceDescriptor {
        locator: "builtin://cube",
        kind: AssetKind::Model,
        factory: build_cube_model,
    },
    BuiltinResourceDescriptor {
        locator: "builtin://missing-model",
        kind: AssetKind::Model,
        factory: build_missing_model,
    },
    BuiltinResourceDescriptor {
        locator: "builtin://material/default",
        kind: AssetKind::Material,
        factory: build_default_material,
    },
    BuiltinResourceDescriptor {
        locator: "builtin://missing-material",
        kind: AssetKind::Material,
        factory: build_missing_material,
    },
    BuiltinResourceDescriptor {
        locator: "builtin://shader/pbr.wgsl",
        kind: AssetKind::Shader,
        factory: build_pbr_shader,
    },
];

/// Materializes every builtin for bootstrap registration.
pub(in crate::asset::pipeline::manager) fn builtin_resources() -> Vec<(&'static str, ImportedAsset)>
{
    BUILTIN_RESOURCE_DESCRIPTORS
        .iter()
        .enumerate()
        .map(|(index, descriptor)| (descriptor.locator, construct_builtin(index, descriptor)))
        .collect()
}

/// Looks up one builtin by its canonical locator and materializes only that entry.
pub(in crate::asset::pipeline::manager) fn builtin_resource(
    locator: &AssetUri,
    matches: impl Fn(&AssetUri, &str) -> bool,
) -> Option<ImportedAsset> {
    if locator.scheme() != ResourceScheme::Builtin {
        return None;
    }
    BUILTIN_RESOURCE_DESCRIPTORS
        .iter()
        .enumerate()
        .find_map(|(index, descriptor)| {
            matches(locator, descriptor.locator).then(|| construct_builtin(index, descriptor))
        })
}

fn construct_builtin(index: usize, descriptor: &BuiltinResourceDescriptor) -> ImportedAsset {
    let asset = (descriptor.factory)();
    debug_assert_eq!(asset_kind_for_imported_asset(&asset), descriptor.kind);
    #[cfg(test)]
    record_builtin_construction(index, &asset);
    asset
}

fn build_cube_model() -> ImportedAsset {
    build_model("builtin://cube")
}

fn build_missing_model() -> ImportedAsset {
    build_model("builtin://missing-model")
}

fn build_model(locator: &'static str) -> ImportedAsset {
    let mesh = generate_cube_mesh();
    ImportedAsset::Model(ModelAsset {
        uri: AssetUri::parse(locator).expect("builtin model uri"),
        primitives: vec![ModelPrimitiveAsset {
            vertices: mesh.vertices,
            indices: mesh.indices,
            mesh: None,
            mesh_sdf: None,
            virtual_geometry: None,
        }],
    })
}

fn build_default_material() -> ImportedAsset {
    build_material("Builtin Default", [1.0, 1.0, 1.0, 1.0])
}

fn build_missing_material() -> ImportedAsset {
    build_material("Builtin Missing", [1.0, 0.0, 1.0, 1.0])
}

fn build_material(name: &'static str, base_color: [f32; 4]) -> ImportedAsset {
    ImportedAsset::Material(MaterialAsset {
        name: Some(name.to_string()),
        shader: builtin_reference("builtin://shader/pbr.wgsl"),
        parent: None,
        options: Default::default(),
        queue: None,
        base_color,
        base_color_texture: None,
        normal_texture: None,
        metallic: 0.0,
        roughness: 1.0,
        metallic_roughness_texture: None,
        occlusion_texture: None,
        emissive: [0.0, 0.0, 0.0],
        emissive_texture: None,
        alpha_mode: AlphaMode::Opaque,
        double_sided: false,
        property_values: Default::default(),
        texture_slots: Default::default(),
        validation_diagnostics: Vec::new(),
    })
}

fn build_pbr_shader() -> ImportedAsset {
    ImportedAsset::Shader(ShaderAsset {
        uri: AssetUri::parse("builtin://shader/pbr.wgsl").expect("builtin shader uri"),
        kind: ShaderAssetKind::Surface,
        source_language: ShaderSourceLanguage::Wgsl,
        source: builtin_pbr_wgsl().to_string(),
        wgsl_source: builtin_pbr_wgsl().to_string(),
        import_path: None,
        entry_points: Vec::new(),
        dependencies: Vec::new(),
        source_files: Vec::new(),
        imports: Vec::new(),
        shader_defs: Vec::new(),
        property_schema: Vec::new(),
        options: Vec::new(),
        texture_slots: Vec::new(),
        shading_model: Some("standard_pbr".to_string()),
        render_state: Default::default(),
        queue: None,
        disabled_passes: Vec::new(),
        resources: Vec::new(),
        material_property_layout: Default::default(),
        material_option_table: Default::default(),
        generated_material_wgsl: String::new(),
        editor: Default::default(),
        pipeline_layout: Default::default(),
        validation_diagnostics: Vec::new(),
    })
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct BuiltinConstructionStats {
    pub(super) payload_constructions: usize,
    pub(super) by_descriptor: [usize; 5],
    pub(super) owned_heap_bytes_proxy: usize,
}

#[cfg(test)]
std::thread_local! {
    static BUILTIN_CONSTRUCTION_STATS: std::cell::Cell<BuiltinConstructionStats> =
        const { std::cell::Cell::new(BuiltinConstructionStats {
            payload_constructions: 0,
            by_descriptor: [0; 5],
            owned_heap_bytes_proxy: 0,
        }) };
}

#[cfg(test)]
pub(super) fn reset_builtin_construction_stats() {
    BUILTIN_CONSTRUCTION_STATS.with(|stats| {
        stats.set(BuiltinConstructionStats::default());
    });
}

#[cfg(test)]
pub(super) fn take_builtin_construction_stats() -> BuiltinConstructionStats {
    BUILTIN_CONSTRUCTION_STATS.with(|stats| {
        let current = stats.get();
        stats.set(BuiltinConstructionStats::default());
        current
    })
}

#[cfg(test)]
fn record_builtin_construction(index: usize, asset: &ImportedAsset) {
    BUILTIN_CONSTRUCTION_STATS.with(|stats| {
        let mut current = stats.get();
        current.payload_constructions += 1;
        current.by_descriptor[index] += 1;
        current.owned_heap_bytes_proxy += owned_heap_bytes_proxy(asset);
        stats.set(current);
    });
}

#[cfg(test)]
fn owned_heap_bytes_proxy(asset: &ImportedAsset) -> usize {
    match asset {
        ImportedAsset::Model(model) => model
            .primitives
            .iter()
            .map(|primitive| {
                primitive
                    .vertices
                    .capacity()
                    .saturating_mul(std::mem::size_of::<crate::asset::MeshVertex>())
                    .saturating_add(
                        primitive
                            .indices
                            .capacity()
                            .saturating_mul(std::mem::size_of::<u32>()),
                    )
            })
            .sum(),
        ImportedAsset::Material(material) => material.name.as_ref().map_or(0, String::capacity),
        ImportedAsset::Shader(shader) => shader
            .source
            .capacity()
            .saturating_add(shader.wgsl_source.capacity())
            .saturating_add(shader.shading_model.as_ref().map_or(0, String::capacity)),
        _ => 0,
    }
}

#[cfg(test)]
#[path = "tests/builtin_resources.rs"]
mod tests;

#[cfg(test)]
#[path = "builtin_resources/tests/lazy_lookup_tests.rs"]
mod lazy_lookup_tests;
