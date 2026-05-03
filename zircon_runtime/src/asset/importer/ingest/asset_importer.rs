use super::{
    import_animation_asset, import_authoring_asset, import_data_asset, import_font_asset,
    import_material, import_model, import_physics_material, import_scene, import_shader,
};
#[cfg(test)]
use super::{import_gltf, import_obj, import_sound, import_texture, import_ui_asset};
use crate::asset::{
    AssetImportError, AssetImporterDescriptor, AssetImporterRegistry, AssetKind,
    DiagnosticOnlyAssetImporter, FunctionAssetImporter,
};

const BUILTIN_IMPORTER_PLUGIN_ID: &str = "zircon.builtin.asset_importers";
const PLUGIN_REQUIRED_IMPORTER_PLUGIN_ID: &str = "zircon.runtime.plugin_required_importers";

#[derive(Clone, Debug)]
pub struct AssetImporter {
    registry: AssetImporterRegistry,
}

impl Default for AssetImporter {
    fn default() -> Self {
        let mut importer = Self {
            registry: AssetImporterRegistry::default(),
        };
        importer
            .register_builtin_importers()
            .expect("builtin asset importers must not conflict");
        importer
    }
}

impl AssetImporter {
    pub fn with_registry(registry: AssetImporterRegistry) -> Self {
        Self { registry }
    }

    pub fn registry(&self) -> &AssetImporterRegistry {
        &self.registry
    }

    pub fn registry_mut(&mut self) -> &mut AssetImporterRegistry {
        &mut self.registry
    }

    fn register_builtin_importers(&mut self) -> Result<(), AssetImportError> {
        self.register_function(
            descriptor("zircon.builtin.data.toml", AssetKind::Data, 1)
                .with_source_extensions(["toml"]),
            import_data_asset::import_plain_toml_data,
        )?;
        self.register_function(
            descriptor("zircon.builtin.data.json", AssetKind::Data, 1)
                .with_source_extensions(["json"]),
            import_data_asset::import_json_data,
        )?;
        self.register_optional(
            descriptor("zircon.optional.data.yaml", AssetKind::Data, 1)
                .with_source_extensions(["yaml", "yml"])
                .with_required_capabilities(["runtime.asset.importer.data.yaml"]),
            "yaml data importer backend is not installed",
        )?;
        self.register_optional(
            descriptor("zircon.optional.data.xml", AssetKind::Data, 1)
                .with_source_extensions(["xml"])
                .with_required_capabilities(["runtime.asset.importer.data.xml"]),
            "xml data importer backend is not installed",
        )?;

        self.register_function(
            descriptor("zircon.builtin.toml.material", AssetKind::Material, 1)
                .with_full_suffixes([".material.toml"]),
            import_material::import_material,
        )?;
        self.register_function(
            descriptor("zircon.builtin.toml.font", AssetKind::Font, 1)
                .with_full_suffixes([".font.toml"]),
            import_font_asset::import_font_asset,
        )?;
        self.register_function(
            descriptor("zircon.builtin.toml.model", AssetKind::Model, 1)
                .with_full_suffixes([".model.toml"]),
            import_model::import_model,
        )?;
        self.register_function(
            descriptor(
                "zircon.builtin.toml.physics_material",
                AssetKind::PhysicsMaterial,
                1,
            )
            .with_full_suffixes([".physics_material.toml"]),
            import_physics_material::import_physics_material,
        )?;
        self.register_function(
            descriptor("zircon.builtin.toml.scene", AssetKind::Scene, 1)
                .with_full_suffixes([".scene.toml"]),
            import_scene::import_scene,
        )?;
        self.register_function(
            descriptor("zircon.builtin.toml.prefab", AssetKind::Prefab, 1)
                .with_full_suffixes([".prefab.toml"]),
            import_authoring_asset::import_prefab,
        )?;
        self.register_function(
            descriptor(
                "zircon.builtin.toml.material_graph",
                AssetKind::MaterialGraph,
                1,
            )
            .with_full_suffixes([".material_graph.toml"]),
            import_authoring_asset::import_material_graph,
        )?;
        self.register_function(
            descriptor("zircon.builtin.toml.terrain", AssetKind::Terrain, 1)
                .with_full_suffixes([".terrain.toml"]),
            import_authoring_asset::import_terrain,
        )?;
        self.register_function(
            descriptor(
                "zircon.builtin.toml.terrain_layer_stack",
                AssetKind::TerrainLayerStack,
                1,
            )
            .with_full_suffixes([".terrain_layers.toml"]),
            import_authoring_asset::import_terrain_layer_stack,
        )?;
        self.register_function(
            descriptor("zircon.builtin.toml.tileset", AssetKind::TileSet, 1)
                .with_full_suffixes([".tileset.toml"]),
            import_authoring_asset::import_tileset,
        )?;
        self.register_function(
            descriptor("zircon.builtin.toml.tilemap", AssetKind::TileMap, 1)
                .with_full_suffixes([".tilemap.toml"]),
            import_authoring_asset::import_tilemap,
        )?;
        self.register_function(
            descriptor("zircon.builtin.toml.navmesh", AssetKind::NavMesh, 1)
                .with_full_suffixes([".navmesh.toml"]),
            import_authoring_asset::import_navmesh,
        )?;
        self.register_function(
            descriptor(
                "zircon.builtin.toml.navigation_settings",
                AssetKind::NavigationSettings,
                1,
            )
            .with_full_suffixes([".navigation.toml"]),
            import_authoring_asset::import_navigation_settings,
        )?;
        self.register_optional(
            plugin_required_descriptor(
                "zircon.plugin_required.ui_document.ui_toml",
                AssetKind::UiLayout,
                1,
            )
            .with_full_suffixes([".ui.toml"])
            .with_additional_output_kinds([AssetKind::UiWidget, AssetKind::UiStyle]),
            "ui document importer plugin is not installed",
        )?;

        self.register_function(
            descriptor(
                "zircon.builtin.animation.zranim.skeleton",
                AssetKind::AnimationSkeleton,
                1,
            )
            .with_full_suffixes([".skeleton.zranim"]),
            import_animation_asset::import_animation_asset,
        )?;
        self.register_function(
            descriptor(
                "zircon.builtin.animation.zranim.clip",
                AssetKind::AnimationClip,
                1,
            )
            .with_full_suffixes([".clip.zranim"]),
            import_animation_asset::import_animation_asset,
        )?;
        self.register_function(
            descriptor(
                "zircon.builtin.animation.zranim.sequence",
                AssetKind::AnimationSequence,
                1,
            )
            .with_full_suffixes([".sequence.zranim"]),
            import_animation_asset::import_animation_asset,
        )?;
        self.register_function(
            descriptor(
                "zircon.builtin.animation.zranim.graph",
                AssetKind::AnimationGraph,
                1,
            )
            .with_full_suffixes([".graph.zranim"]),
            import_animation_asset::import_animation_asset,
        )?;
        self.register_function(
            descriptor(
                "zircon.builtin.animation.zranim.state_machine",
                AssetKind::AnimationStateMachine,
                1,
            )
            .with_full_suffixes([".state_machine.zranim"]),
            import_animation_asset::import_animation_asset,
        )?;

        self.register_optional(
            plugin_required_descriptor(
                "zircon.plugin_required.texture.image",
                AssetKind::Texture,
                1,
            )
            .with_source_extensions([
                "png", "jpg", "jpeg", "bmp", "tga", "tiff", "tif", "gif", "webp", "hdr", "exr",
                "qoi", "pnm", "pbm", "pgm", "ppm",
            ])
            .with_required_capabilities(["runtime.asset.importer.texture.image"]),
            "texture image importer plugin is not installed",
        )?;
        for (id, extension) in [
            ("zircon.optional.texture.psd", "psd"),
            ("zircon.optional.texture.dds", "dds"),
            ("zircon.optional.texture.ktx", "ktx"),
            ("zircon.optional.texture.ktx2", "ktx2"),
            ("zircon.optional.texture.astc", "astc"),
            ("zircon.optional.texture.cubemap", "cubemap"),
            ("zircon.optional.texture.dxgi", "dxgi"),
        ] {
            self.register_optional(
                descriptor(id, AssetKind::Texture, 1)
                    .with_source_extensions([extension])
                    .with_required_capabilities([format!("runtime.asset.importer.{extension}")]),
                format!("{extension} texture importer backend is not installed"),
            )?;
        }

        self.register_optional(
            plugin_required_descriptor("zircon.plugin_required.shader.wgsl", AssetKind::Shader, 1)
                .with_source_extensions(["wgsl"])
                .with_required_capabilities(["runtime.asset.importer.shader.wgsl"]),
            "wgsl shader importer plugin is not installed",
        )?;
        self.register_function(
            descriptor("zircon.builtin.shader.glsl", AssetKind::Shader, 1)
                .with_source_extensions(["glsl", "vert", "frag", "comp", "vs", "fs", "cs"])
                .with_required_capabilities(["runtime.asset.importer.shader.glsl"]),
            import_shader::import_shader,
        )?;
        self.register_function(
            descriptor("zircon.builtin.shader.spirv", AssetKind::Shader, 1)
                .with_source_extensions(["spv"])
                .with_required_capabilities(["runtime.asset.importer.shader.spirv"]),
            import_shader::import_shader,
        )?;
        self.register_optional(
            descriptor("zircon.optional.shader.hlsl_cg", AssetKind::Shader, 1)
                .with_source_extensions(["hlsl", "cg", "fx"])
                .with_required_capabilities(["runtime.asset.importer.native"]),
            "hlsl/cg shader importer requires a NativeDynamic shader toolchain backend",
        )?;

        self.register_optional(
            plugin_required_descriptor("zircon.plugin_required.model.obj", AssetKind::Model, 1)
                .with_source_extensions(["obj"])
                .with_required_capabilities(["runtime.asset.importer.model.obj"]),
            "obj model importer plugin is not installed",
        )?;
        self.register_optional(
            plugin_required_descriptor("zircon.plugin_required.model.gltf", AssetKind::Model, 1)
                .with_source_extensions(["gltf", "glb"])
                .with_required_capabilities(["runtime.asset.importer.model.gltf"]),
            "glTF model importer plugin is not installed",
        )?;
        for extension in [
            "fbx", "dae", "3ds", "dxf", "ply", "stl", "usd", "usda", "usdc", "usdz",
        ] {
            self.register_optional(
                descriptor(
                    format!("zircon.optional.model.{extension}"),
                    AssetKind::Model,
                    1,
                )
                .with_source_extensions([extension])
                .with_required_capabilities([format!("runtime.asset.importer.model.{extension}")]),
                format!("{extension} model importer backend is not installed"),
            )?;
        }

        self.register_optional(
            plugin_required_descriptor("zircon.plugin_required.audio.wav", AssetKind::Sound, 1)
                .with_source_extensions(["wav"])
                .with_required_capabilities(["runtime.asset.importer.audio.wav"]),
            "wav audio importer plugin is not installed",
        )?;
        for extension in ["mp3", "ogg", "flac", "aif", "aiff", "opus"] {
            self.register_optional(
                descriptor(
                    format!("zircon.optional.audio.{extension}"),
                    AssetKind::Sound,
                    1,
                )
                .with_source_extensions([extension])
                .with_required_capabilities([format!("runtime.asset.importer.audio.{extension}")]),
                format!("{extension} audio decoder backend is not installed"),
            )?;
        }

        Ok(())
    }

    fn register_function(
        &mut self,
        descriptor: AssetImporterDescriptor,
        import_fn: fn(
            &crate::asset::AssetImportContext,
        ) -> Result<crate::asset::AssetImportOutcome, AssetImportError>,
    ) -> Result<(), AssetImportError> {
        self.registry
            .register(FunctionAssetImporter::new(descriptor, import_fn))
            .map_err(AssetImportError::from)
    }

    fn register_optional(
        &mut self,
        descriptor: AssetImporterDescriptor,
        message: impl Into<String>,
    ) -> Result<(), AssetImportError> {
        self.registry
            .register(DiagnosticOnlyAssetImporter::new(descriptor, message))
            .map_err(AssetImportError::from)
    }

    #[cfg(test)]
    pub(crate) fn first_wave_plugin_fixture_importers_for_test() -> Vec<FunctionAssetImporter> {
        vec![
            FunctionAssetImporter::new(
                plugin_fixture_descriptor(
                    "ui_document_importer.typed_toml",
                    "ui_document_importer",
                    AssetKind::UiLayout,
                )
                .with_full_suffixes([".ui.toml"])
                .with_additional_output_kinds([AssetKind::UiWidget, AssetKind::UiStyle])
                .with_required_capabilities(["runtime.asset.importer.ui_document"]),
                import_ui_asset::import_ui_asset,
            ),
            FunctionAssetImporter::new(
                plugin_fixture_descriptor(
                    "texture_importer.image",
                    "texture_importer",
                    AssetKind::Texture,
                )
                .with_source_extensions([
                    "png", "jpg", "jpeg", "bmp", "tga", "tiff", "tif", "gif", "webp", "hdr", "exr",
                    "qoi", "pnm", "pbm", "pgm", "ppm",
                ])
                .with_required_capabilities(["runtime.asset.importer.texture.image"]),
                import_texture::import_texture,
            ),
            FunctionAssetImporter::new(
                plugin_fixture_descriptor(
                    "shader_wgsl_importer.wgsl",
                    "shader_wgsl_importer",
                    AssetKind::Shader,
                )
                .with_source_extensions(["wgsl"])
                .with_required_capabilities(["runtime.asset.importer.shader.wgsl"]),
                import_shader::import_shader,
            ),
            FunctionAssetImporter::new(
                plugin_fixture_descriptor("obj_importer.obj", "obj_importer", AssetKind::Model)
                    .with_source_extensions(["obj"])
                    .with_required_capabilities(["runtime.asset.importer.model.obj"]),
                import_obj::import_obj,
            ),
            FunctionAssetImporter::new(
                plugin_fixture_descriptor("gltf_importer.gltf", "gltf_importer", AssetKind::Model)
                    .with_source_extensions(["gltf", "glb"])
                    .with_required_capabilities(["runtime.asset.importer.model.gltf"]),
                import_gltf::import_gltf,
            ),
            FunctionAssetImporter::new(
                plugin_fixture_descriptor("audio_importer.wav", "audio_importer", AssetKind::Sound)
                    .with_source_extensions(["wav"])
                    .with_required_capabilities(["runtime.asset.importer.audio.wav"]),
                import_sound::import_sound,
            ),
        ]
    }

    #[cfg(test)]
    pub(crate) fn register_first_wave_plugin_fixture_importers_for_test(
        &mut self,
    ) -> Result<(), AssetImportError> {
        for importer in Self::first_wave_plugin_fixture_importers_for_test() {
            self.registry
                .register(importer)
                .map_err(AssetImportError::from)?;
        }
        Ok(())
    }
}

fn descriptor(
    id: impl Into<String>,
    output_kind: AssetKind,
    importer_version: u32,
) -> AssetImporterDescriptor {
    AssetImporterDescriptor::new(
        id,
        BUILTIN_IMPORTER_PLUGIN_ID,
        output_kind,
        importer_version,
    )
}

fn plugin_required_descriptor(
    id: impl Into<String>,
    output_kind: AssetKind,
    importer_version: u32,
) -> AssetImporterDescriptor {
    AssetImporterDescriptor::new(
        id,
        PLUGIN_REQUIRED_IMPORTER_PLUGIN_ID,
        output_kind,
        importer_version,
    )
}

#[cfg(test)]
fn plugin_fixture_descriptor(
    id: impl Into<String>,
    plugin_id: impl Into<String>,
    output_kind: AssetKind,
) -> AssetImporterDescriptor {
    AssetImporterDescriptor::new(id, plugin_id, output_kind, 1).with_priority(120)
}
