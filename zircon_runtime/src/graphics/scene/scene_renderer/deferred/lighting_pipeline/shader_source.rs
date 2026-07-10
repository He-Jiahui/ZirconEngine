use crate::core::framework::render::ShadingModelDescriptor;
use crate::graphics::material::ShadingModelIncludeSourceSet;

const GPU_SCENE_INCLUDE_TOKEN: &str = "zr_gpu_scene.wgsl";
const LIGHT_GRID_INCLUDE_TOKEN: &str = "zr_light_grid.wgsl";
const SHADOW_INCLUDE_TOKEN: &str = "zr_shadow.wgsl";
const ENVIRONMENT_INCLUDE_TOKEN: &str = "zr_environment.wgsl";
const DEFERRED_STANDARD_PBR_INCLUDE_TOKEN: &str = "zr_shade_deferred_standard_pbr.wgsl";
const DEFERRED_BLINN_PHONG_INCLUDE_TOKEN: &str = "zr_shade_deferred_blinn_phong.wgsl";
const DEFERRED_UNLIT_INCLUDE_TOKEN: &str = "zr_shade_deferred_unlit.wgsl";
const CUSTOM_DISPATCH_MARKER: &str = "    // zr-deferred-lighting-custom-shading-model-dispatch";

const GPU_SCENE_INCLUDE: &str = include_str!("../../mesh/shaders/zr_gpu_scene.wgsl");
const LIGHT_GRID_INCLUDE: &str = include_str!("../../lighting/shaders/zr_light_grid.wgsl");
const SHADOW_INCLUDE: &str = include_str!("../../shadow/shaders/zr_shadow.wgsl");
const ENVIRONMENT_INCLUDE: &str = include_str!("../../../../shader/wgsl/zr_environment.wgsl");
const DEFERRED_STANDARD_PBR_INCLUDE: &str =
    include_str!("../../../../shader/wgsl/zr_shade_deferred_standard_pbr.wgsl");
const DEFERRED_BLINN_PHONG_INCLUDE: &str =
    include_str!("../../../../shader/wgsl/zr_shade_deferred_blinn_phong.wgsl");
const DEFERRED_UNLIT_INCLUDE: &str =
    include_str!("../../../../shader/wgsl/zr_shade_deferred_unlit.wgsl");
const DEFERRED_LIGHTING_TEMPLATE: &str = include_str!("../shaders/deferred_lighting.wgsl");

pub(in crate::graphics::scene::scene_renderer::deferred) const DEFERRED_LIGHTING_SHADER: &str = concat!(
    "// include: zr_gpu_scene.wgsl\n",
    include_str!("../../mesh/shaders/zr_gpu_scene.wgsl"),
    "\n// include: zr_light_grid.wgsl\n",
    include_str!("../../lighting/shaders/zr_light_grid.wgsl"),
    "\n// include: zr_shadow.wgsl\n",
    include_str!("../../shadow/shaders/zr_shadow.wgsl"),
    "\n// include: zr_shade_deferred_standard_pbr.wgsl\n",
    include_str!("../../../../shader/wgsl/zr_shade_deferred_standard_pbr.wgsl"),
    "\n// include: zr_shade_deferred_blinn_phong.wgsl\n",
    include_str!("../../../../shader/wgsl/zr_shade_deferred_blinn_phong.wgsl"),
    "\n// include: zr_shade_deferred_unlit.wgsl\n",
    include_str!("../../../../shader/wgsl/zr_shade_deferred_unlit.wgsl"),
    "\n// include: deferred_lighting.wgsl\n",
    include_str!("../shaders/deferred_lighting.wgsl"),
    "\n// include: zr_environment.wgsl\n",
    include_str!("../../../../shader/wgsl/zr_environment.wgsl")
);

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::graphics::scene::scene_renderer::deferred) enum DeferredLightingShaderSourceError {
    UnknownDeferredInclude { token: String },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::graphics::scene::scene_renderer::deferred) struct DeferredLightingShaderIncludeSource
{
    token: String,
    source: String,
}

impl DeferredLightingShaderIncludeSource {
    fn new(token: impl Into<String>, source: impl Into<String>) -> Self {
        Self {
            token: token.into(),
            source: source.into(),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(in crate::graphics::scene::scene_renderer::deferred) struct DeferredLightingShaderSourceRequest
{
    shading_model_descriptors: Vec<ShadingModelDescriptor>,
    shading_model_deferred_include_sources: Vec<DeferredLightingShaderIncludeSource>,
}

impl DeferredLightingShaderSourceRequest {
    pub(in crate::graphics::scene::scene_renderer::deferred) fn new() -> Self {
        Self::default()
    }

    pub(in crate::graphics::scene::scene_renderer::deferred) fn with_shading_model_descriptor(
        mut self,
        descriptor: ShadingModelDescriptor,
    ) -> Self {
        self.shading_model_descriptors.push(descriptor);
        self
    }

    pub(in crate::graphics::scene::scene_renderer::deferred) fn with_shading_model_deferred_include_source(
        mut self,
        token: impl Into<String>,
        source: impl Into<String>,
    ) -> Self {
        self.shading_model_deferred_include_sources
            .push(DeferredLightingShaderIncludeSource::new(token, source));
        self
    }

    pub(in crate::graphics::scene::scene_renderer::deferred) fn with_shading_model_deferred_include_sources(
        mut self,
        sources: &ShadingModelIncludeSourceSet,
    ) -> Self {
        for source in sources.deferred() {
            self.shading_model_deferred_include_sources.push(
                DeferredLightingShaderIncludeSource::new(
                    source.token.clone(),
                    source.source.clone(),
                ),
            );
        }
        self
    }
}

pub(in crate::graphics::scene::scene_renderer::deferred) fn assemble_deferred_lighting_shader_source(
    request: DeferredLightingShaderSourceRequest,
) -> Result<String, DeferredLightingShaderSourceError> {
    let mut source = String::new();
    push_include(&mut source, GPU_SCENE_INCLUDE_TOKEN, GPU_SCENE_INCLUDE);
    push_include(&mut source, LIGHT_GRID_INCLUDE_TOKEN, LIGHT_GRID_INCLUDE);
    push_include(&mut source, SHADOW_INCLUDE_TOKEN, SHADOW_INCLUDE);
    push_include(
        &mut source,
        DEFERRED_STANDARD_PBR_INCLUDE_TOKEN,
        DEFERRED_STANDARD_PBR_INCLUDE,
    );
    push_include(
        &mut source,
        DEFERRED_BLINN_PHONG_INCLUDE_TOKEN,
        DEFERRED_BLINN_PHONG_INCLUDE,
    );
    push_include(
        &mut source,
        DEFERRED_UNLIT_INCLUDE_TOKEN,
        DEFERRED_UNLIT_INCLUDE,
    );

    let custom_dispatch = custom_deferred_dispatch(&request)?;
    for descriptor in request.shading_model_descriptors.iter() {
        if builtin_deferred_include_token(descriptor.deferred_include.as_str()) {
            continue;
        }
        let include = request
            .shading_model_deferred_include_sources
            .iter()
            .find(|include| {
                deferred_include_tokens_match(&include.token, &descriptor.deferred_include)
            })
            .ok_or_else(
                || DeferredLightingShaderSourceError::UnknownDeferredInclude {
                    token: descriptor.deferred_include.clone(),
                },
            )?;
        push_include(&mut source, &include.token, &include.source);
    }

    let template = DEFERRED_LIGHTING_TEMPLATE.replace(
        CUSTOM_DISPATCH_MARKER,
        &format!("{CUSTOM_DISPATCH_MARKER}\n{custom_dispatch}"),
    );
    push_include(&mut source, "deferred_lighting.wgsl", &template);
    push_include(&mut source, ENVIRONMENT_INCLUDE_TOKEN, ENVIRONMENT_INCLUDE);
    Ok(source)
}

fn push_include(source: &mut String, token: &str, include: &str) {
    source.push_str("// include: ");
    source.push_str(token);
    source.push('\n');
    source.push_str(include);
    source.push('\n');
}

fn custom_deferred_dispatch(
    request: &DeferredLightingShaderSourceRequest,
) -> Result<String, DeferredLightingShaderSourceError> {
    let mut dispatch = String::new();
    for descriptor in request.shading_model_descriptors.iter() {
        if builtin_deferred_include_token(descriptor.deferred_include.as_str()) {
            continue;
        }
        if !request
            .shading_model_deferred_include_sources
            .iter()
            .any(|include| {
                deferred_include_tokens_match(&include.token, &descriptor.deferred_include)
            })
        {
            return Err(DeferredLightingShaderSourceError::UnknownDeferredInclude {
                token: descriptor.deferred_include.clone(),
            });
        }
        let function_name = deferred_shading_function_name(&descriptor.deferred_include);
        dispatch.push_str("    if (shading_model_id == ");
        dispatch.push_str(&descriptor.id.value().to_string());
        dispatch.push_str("u) {\n        return add_deferred_emissive(");
        dispatch.push_str(&function_name);
        dispatch.push_str("(position, coord, albedo, material, normal), emissive);\n    }\n");
    }
    Ok(dispatch)
}

fn builtin_deferred_include_token(token: &str) -> bool {
    let token = token.trim_end_matches(".wgsl");
    matches!(
        token,
        "zr_shade_deferred_standard_pbr"
            | "zr_shade_deferred_blinn_phong"
            | "zr_shade_deferred_unlit"
    )
}

fn deferred_include_tokens_match(left: &str, right: &str) -> bool {
    left == right || left.trim_end_matches(".wgsl") == right.trim_end_matches(".wgsl")
}

fn deferred_shading_function_name(token: &str) -> String {
    let token = token
        .trim_end_matches(".wgsl")
        .trim_start_matches("zr_shade_deferred_");
    format!("shade_deferred_{token}")
}

#[cfg(test)]
mod tests {
    use crate::core::framework::render::{
        SHADING_MODEL_ID_BLINN_PHONG, SHADING_MODEL_ID_STANDARD_PBR, SHADING_MODEL_ID_UNLIT,
    };

    use super::*;

    #[test]
    fn builtin_deferred_include_tokens_accept_manifest_and_wgsl_forms() {
        for token in [
            "zr_shade_deferred_unlit",
            "zr_shade_deferred_unlit.wgsl",
            "zr_shade_deferred_blinn_phong",
            "zr_shade_deferred_standard_pbr.wgsl",
        ] {
            assert!(builtin_deferred_include_token(token));
        }
    }

    #[test]
    fn deferred_shading_function_names_strip_include_prefix_and_extension() {
        assert_eq!(
            deferred_shading_function_name("zr_shade_deferred_toon.wgsl"),
            "shade_deferred_toon"
        );
    }

    #[test]
    fn builtin_shading_model_ids_are_reserved_by_static_dispatch() {
        assert_eq!(SHADING_MODEL_ID_UNLIT.value(), 0);
        assert_eq!(SHADING_MODEL_ID_BLINN_PHONG.value(), 1);
        assert_eq!(SHADING_MODEL_ID_STANDARD_PBR.value(), 2);
    }
}
