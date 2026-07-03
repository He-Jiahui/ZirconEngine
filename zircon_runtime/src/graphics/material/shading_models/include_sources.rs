use std::fmt::{Display, Formatter};

use crate::asset::ProjectAssetManager;
use crate::core::framework::render::ShadingModelDescriptor;
use crate::core::resource::{ResourceKind, ResourceLocator, ResourceRecord};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ShadingModelIncludeSource {
    pub(crate) token: String,
    pub(crate) source: String,
}

impl ShadingModelIncludeSource {
    fn new(token: impl Into<String>, source: impl Into<String>) -> Self {
        Self {
            token: token.into(),
            source: source.into(),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct ShadingModelIncludeSourceSet {
    forward: Vec<ShadingModelIncludeSource>,
    gbuffer: Vec<ShadingModelIncludeSource>,
    deferred: Vec<ShadingModelIncludeSource>,
}

impl ShadingModelIncludeSourceSet {
    pub(crate) fn from_project_asset_manager(
        asset_manager: &ProjectAssetManager,
        descriptors: &[ShadingModelDescriptor],
    ) -> Result<Self, ShadingModelIncludeSourceError> {
        let shader_records = asset_manager
            .resource_manager()
            .ready_records_for_kind(ResourceKind::Shader);
        let mut set = Self::default();
        for descriptor in descriptors
            .iter()
            .filter(|descriptor| descriptor.id.is_plugin_range())
        {
            set.forward.push(resolve_include_source(
                asset_manager,
                &shader_records,
                &descriptor.forward_include,
            )?);
            set.gbuffer.push(resolve_include_source(
                asset_manager,
                &shader_records,
                &descriptor.gbuffer_encode_include,
            )?);
            set.deferred.push(resolve_include_source(
                asset_manager,
                &shader_records,
                &descriptor.deferred_include,
            )?);
        }
        Ok(set)
    }

    pub(crate) fn forward(&self) -> &[ShadingModelIncludeSource] {
        &self.forward
    }

    pub(crate) fn gbuffer(&self) -> &[ShadingModelIncludeSource] {
        &self.gbuffer
    }

    pub(crate) fn deferred(&self) -> &[ShadingModelIncludeSource] {
        &self.deferred
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ShadingModelIncludeSourceError {
    MissingInclude {
        token: String,
    },
    DuplicateIncludeToken {
        token: String,
        first_locator: String,
        second_locator: String,
    },
    MissingRuntimeSource {
        token: String,
        locator: String,
    },
    LoadShader {
        token: String,
        locator: String,
        message: String,
    },
}

impl Display for ShadingModelIncludeSourceError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingInclude { token } => {
                write!(f, "shading model include `{token}` was not found in ready shader assets")
            }
            Self::DuplicateIncludeToken {
                token,
                first_locator,
                second_locator,
            } => write!(
                f,
                "shading model include `{token}` matched multiple shader assets: {first_locator} and {second_locator}"
            ),
            Self::MissingRuntimeSource { token, locator } => write!(
                f,
                "shading model include `{token}` matched {locator}, but the shader has no runtime WGSL source"
            ),
            Self::LoadShader {
                token,
                locator,
                message,
            } => write!(
                f,
                "failed to load shader asset {locator} for shading model include `{token}`: {message}"
            ),
        }
    }
}

impl std::error::Error for ShadingModelIncludeSourceError {}

fn resolve_include_source(
    asset_manager: &ProjectAssetManager,
    shader_records: &[ResourceRecord],
    token: &str,
) -> Result<ShadingModelIncludeSource, ShadingModelIncludeSourceError> {
    let matches = shader_records
        .iter()
        .filter(|record| record_matches_include_token(record, token))
        .collect::<Vec<_>>();
    let Some(record) = matches.first() else {
        return Err(ShadingModelIncludeSourceError::MissingInclude {
            token: token.to_string(),
        });
    };
    if let Some(second) = matches.get(1) {
        return Err(ShadingModelIncludeSourceError::DuplicateIncludeToken {
            token: token.to_string(),
            first_locator: record.primary_locator.to_string(),
            second_locator: second.primary_locator.to_string(),
        });
    }

    let shader = asset_manager
        .load_shader_asset(record.id)
        .map_err(|error| ShadingModelIncludeSourceError::LoadShader {
            token: token.to_string(),
            locator: record.primary_locator.to_string(),
            message: error.to_string(),
        })?;
    let Some(source) = shader.runtime_wgsl_source() else {
        return Err(ShadingModelIncludeSourceError::MissingRuntimeSource {
            token: token.to_string(),
            locator: record.primary_locator.to_string(),
        });
    };
    Ok(ShadingModelIncludeSource::new(token, source))
}

fn record_matches_include_token(record: &ResourceRecord, token: &str) -> bool {
    locator_matches_include_token(&record.primary_locator, token)
        || record
            .artifact_locator
            .as_ref()
            .is_some_and(|locator| locator_matches_include_token(locator, token))
}

fn locator_matches_include_token(locator: &ResourceLocator, token: &str) -> bool {
    let token = normalize_include_token(token);
    let path = normalize_include_token(locator.path());
    path == token || path.ends_with(&format!("/{token}"))
}

fn normalize_include_token(value: &str) -> String {
    let normalized = value.trim().replace('\\', "/").to_ascii_lowercase();
    normalized
        .strip_suffix(".wgsl")
        .unwrap_or(normalized.as_str())
        .to_string()
}

#[cfg(test)]
mod tests {
    use crate::asset::{ProjectAssetManager, ShaderAsset, ShaderSourceLanguage};
    use crate::core::framework::render::{
        builtin_geometry_source_descriptor, GBufferChannelMask, ShaderAssetKind, ShaderPassType,
        ShadingModelDescriptor, ShadingModelId, GEOMETRY_SOURCE_ID_STATIC_MESH,
        SHADING_MODEL_ID_STANDARD_PBR,
    };
    use crate::core::resource::{ResourceId, ResourceKind, ResourceLocator, ResourceRecord};
    use crate::graphics::shader::{
        assemble_deferred_gbuffer_shader_template, assemble_material_shader_template,
        DeferredGBufferShaderTemplateRequest, MaterialShaderTemplateRequest,
    };

    use super::*;

    #[test]
    fn project_shader_records_export_plugin_shading_model_include_sources() {
        let asset_manager = ProjectAssetManager::default();
        register_shader(
            &asset_manager,
            "package://toon/shaders/zr_shading_toon.wgsl",
            "fn shade_forward() {}\n",
        );
        register_shader(
            &asset_manager,
            "package://toon/shaders/zr_gbuffer_encode_toon.wgsl",
            "fn encode_gbuffer() {}\n",
        );
        register_shader(
            &asset_manager,
            "package://toon/shaders/zr_shade_deferred_toon.wgsl",
            "fn shade_deferred_toon() {}\n",
        );

        let set = ShadingModelIncludeSourceSet::from_project_asset_manager(
            &asset_manager,
            &[toon_shading_model_descriptor()],
        )
        .expect("plugin include sources should resolve from ready shader assets");

        assert_eq!(set.forward()[0].token, "zr_shading_toon.wgsl");
        assert!(set.forward()[0].source.contains("fn shade_forward"));
        assert_eq!(set.gbuffer()[0].token, "zr_gbuffer_encode_toon.wgsl");
        assert!(set.gbuffer()[0].source.contains("fn encode_gbuffer"));
        assert_eq!(set.deferred()[0].token, "zr_shade_deferred_toon.wgsl");
        assert!(set.deferred()[0].source.contains("fn shade_deferred_toon"));
    }

    #[test]
    fn exported_include_source_set_feeds_forward_and_gbuffer_template_requests() {
        let asset_manager = ProjectAssetManager::default();
        register_shader(
            &asset_manager,
            "package://toon/shaders/zr_shading_toon.wgsl",
            "fn shade_forward(surface: ZrSurfaceOutput, ctx: ZrShadingContext) -> vec3<f32> { return surface.base_color.rgb + vec3<f32>(ctx.frag_coord.x * 0.0); }\n",
        );
        register_shader(
            &asset_manager,
            "package://toon/shaders/zr_gbuffer_encode_toon.wgsl",
            "fn encode_gbuffer(surface: ZrSurfaceOutput, ctx: ZrShadingContext) -> ZrDeferredGBufferOutput { return ZrDeferredGBufferOutput(surface.base_color, vec4<f32>(0.0), vec4<f32>(ctx.shadow_params.z)); }\n",
        );
        register_shader(
            &asset_manager,
            "package://toon/shaders/zr_shade_deferred_toon.wgsl",
            "fn shade_deferred_toon() {}\n",
        );
        let descriptor = toon_shading_model_descriptor();
        let source_set = ShadingModelIncludeSourceSet::from_project_asset_manager(
            &asset_manager,
            &[descriptor.clone()],
        )
        .expect("plugin include source set");
        let geometry_source = builtin_geometry_source_descriptor(GEOMETRY_SOURCE_ID_STATIC_MESH)
            .expect("static geometry descriptor");

        let forward = assemble_material_shader_template(
            MaterialShaderTemplateRequest::new(
                geometry_source.clone(),
                ShaderPassType::Forward,
                material_surface_source(),
                "user_surface",
            )
            .with_shading_model_descriptor(descriptor.clone())
            .with_shading_model_forward_include_sources(&source_set),
        )
        .expect("forward template should consume exported include source set");
        let gbuffer = assemble_deferred_gbuffer_shader_template(
            DeferredGBufferShaderTemplateRequest::new(
                geometry_source,
                material_surface_source(),
                "user_surface",
            )
            .with_shading_model_descriptor(descriptor)
            .with_shading_model_gbuffer_include_sources(&source_set),
        )
        .expect("deferred GBuffer template should consume exported include source set");

        assert!(forward
            .wgsl_source
            .contains("// include: zr_shading_toon.wgsl"));
        assert!(gbuffer
            .wgsl_source
            .contains("// include: zr_gbuffer_encode_toon.wgsl"));
    }

    #[test]
    fn project_shader_records_match_include_tokens_without_wgsl_extension() {
        let asset_manager = ProjectAssetManager::default();
        register_shader(
            &asset_manager,
            "res://shaders/zr_shading_toon.wgsl",
            "fn shade_forward() {}\n",
        );
        register_shader(
            &asset_manager,
            "res://shaders/zr_gbuffer_encode_toon.wgsl",
            "fn encode_gbuffer() {}\n",
        );
        register_shader(
            &asset_manager,
            "res://shaders/zr_shade_deferred_toon.wgsl",
            "fn shade_deferred_toon() {}\n",
        );

        let descriptor = ShadingModelDescriptor::new(
            ShadingModelId::new(16),
            "toon",
            "zr_shading_toon",
            "zr_gbuffer_encode_toon",
            "zr_shade_deferred_toon",
            GBufferChannelMask::standard_lit(),
        );

        let set =
            ShadingModelIncludeSourceSet::from_project_asset_manager(&asset_manager, &[descriptor])
                .expect("extension-free include tokens should resolve");

        assert_eq!(set.forward()[0].token, "zr_shading_toon");
        assert_eq!(set.gbuffer()[0].token, "zr_gbuffer_encode_toon");
        assert_eq!(set.deferred()[0].token, "zr_shade_deferred_toon");
    }

    #[test]
    fn project_shader_records_skip_builtin_shading_model_descriptors() {
        let asset_manager = ProjectAssetManager::default();
        let set = ShadingModelIncludeSourceSet::from_project_asset_manager(
            &asset_manager,
            &[ShadingModelDescriptor::new(
                SHADING_MODEL_ID_STANDARD_PBR,
                "pbr",
                "zr_shading_standard_pbr.wgsl",
                "zr_gbuffer_encode_standard_pbr.wgsl",
                "zr_shade_deferred_standard_pbr.wgsl",
                GBufferChannelMask::standard_lit(),
            )],
        )
        .expect("builtin descriptor should not require project WGSL assets");

        assert!(set.forward().is_empty());
        assert!(set.gbuffer().is_empty());
        assert!(set.deferred().is_empty());
    }

    #[test]
    fn project_shader_records_report_missing_plugin_include_source() {
        let asset_manager = ProjectAssetManager::default();

        let error = ShadingModelIncludeSourceSet::from_project_asset_manager(
            &asset_manager,
            &[toon_shading_model_descriptor()],
        )
        .expect_err("missing plugin include should be explicit");

        assert_eq!(
            error,
            ShadingModelIncludeSourceError::MissingInclude {
                token: "zr_shading_toon.wgsl".to_string()
            }
        );
    }

    #[test]
    fn project_shader_records_reject_duplicate_include_token_matches() {
        let asset_manager = ProjectAssetManager::default();
        register_shader(
            &asset_manager,
            "res://toon/a/zr_shading_toon.wgsl",
            "fn shade_forward_a() {}\n",
        );
        register_shader(
            &asset_manager,
            "res://toon/b/zr_shading_toon.wgsl",
            "fn shade_forward_b() {}\n",
        );
        register_shader(
            &asset_manager,
            "res://toon/zr_gbuffer_encode_toon.wgsl",
            "fn encode_gbuffer() {}\n",
        );
        register_shader(
            &asset_manager,
            "res://toon/zr_shade_deferred_toon.wgsl",
            "fn shade_deferred_toon() {}\n",
        );

        let error = ShadingModelIncludeSourceSet::from_project_asset_manager(
            &asset_manager,
            &[toon_shading_model_descriptor()],
        )
        .expect_err("duplicate include token should be rejected");

        assert!(matches!(
            error,
            ShadingModelIncludeSourceError::DuplicateIncludeToken { .. }
        ));
    }

    fn toon_shading_model_descriptor() -> ShadingModelDescriptor {
        ShadingModelDescriptor::new(
            ShadingModelId::new(16),
            "toon",
            "zr_shading_toon.wgsl",
            "zr_gbuffer_encode_toon.wgsl",
            "zr_shade_deferred_toon.wgsl",
            GBufferChannelMask::standard_lit(),
        )
    }

    fn register_shader(asset_manager: &ProjectAssetManager, locator_text: &str, source: &str) {
        let locator = ResourceLocator::parse(locator_text).expect("valid shader locator");
        let id = ResourceId::from_locator(&locator);
        let record = ResourceRecord::new(id, ResourceKind::Shader, locator.clone())
            .with_source_hash(format!("{locator_text}-hash"));
        asset_manager.resource_manager().register_ready(
            record,
            ShaderAsset {
                uri: locator,
                kind: ShaderAssetKind::Include,
                source_language: ShaderSourceLanguage::Wgsl,
                source: source.to_string(),
                wgsl_source: String::new(),
                import_path: None,
                entry_points: Vec::new(),
                dependencies: Vec::new(),
                source_files: Vec::new(),
                imports: Vec::new(),
                shader_defs: Vec::new(),
                property_schema: Vec::new(),
                options: Vec::new(),
                texture_slots: Vec::new(),
                shading_model: None,
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
            },
        );
    }

    fn material_surface_source() -> &'static str {
        "fn user_surface(input: ZrVertexOutput) -> ZrSurfaceOutput { return zr_surface_from_base_color(input.color); }\n"
    }
}
