use zircon_runtime::graphics::{
    RenderFeatureDescriptor, RenderFeaturePassDescriptor, RenderPassExecutionContext,
    RenderPassExecutorRegistration, RenderPassStage,
};
use zircon_runtime::render_graph::QueueLane;

pub const FEATURE_ID: &str = "rendering.shader_graph";
pub const FEATURE_NAME: &str = "shader_graph";
pub const EXECUTOR_ID: &str = "shader-graph.post-process";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShaderGraphTarget {
    Material,
    PostProcess,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ShaderGraphAsset {
    pub name: String,
    pub target: ShaderGraphTarget,
    pub nodes: Vec<ShaderGraphNode>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ShaderGraphNode {
    ConstantFloat {
        id: String,
        value: f32,
    },
    ConstantColor {
        id: String,
        value: [f32; 4],
    },
    TextureSample {
        id: String,
        binding: u32,
    },
    Add {
        id: String,
        left: String,
        right: String,
    },
    Multiply {
        id: String,
        left: String,
        right: String,
    },
    ColorOutput {
        input: String,
    },
    MaterialOutput {
        base_color: String,
        roughness: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShaderGraphCompileReport {
    pub wgsl: String,
    pub diagnostics: Vec<String>,
}

pub fn compile_shader_graph_to_wgsl(asset: &ShaderGraphAsset) -> ShaderGraphCompileReport {
    let mut diagnostics = Vec::new();
    if asset.nodes.is_empty() {
        diagnostics.push(format!("shader graph `{}` has no nodes", asset.name));
    }
    let mut body = Vec::new();
    for node in &asset.nodes {
        match node {
            ShaderGraphNode::ConstantFloat { id, value } => {
                body.push(format!("    let {id}: f32 = {value};"));
            }
            ShaderGraphNode::ConstantColor { id, value } => {
                body.push(format!(
                    "    let {id}: vec4<f32> = vec4<f32>({:.6}, {:.6}, {:.6}, {:.6});",
                    value[0], value[1], value[2], value[3]
                ));
            }
            ShaderGraphNode::TextureSample { id, binding } => {
                body.push(format!(
                    "    let {id}: vec4<f32> = zircon_sample_texture_{binding}();"
                ));
            }
            ShaderGraphNode::Add { id, left, right } => {
                body.push(format!("    let {id} = {left} + {right};"));
            }
            ShaderGraphNode::Multiply { id, left, right } => {
                body.push(format!("    let {id} = {left} * {right};"));
            }
            ShaderGraphNode::ColorOutput { input } => {
                body.push(format!("    return {input};"));
            }
            ShaderGraphNode::MaterialOutput {
                base_color,
                roughness,
            } => {
                body.push(format!(
                    "    return vec4<f32>({base_color}.rgb, clamp({roughness}, 0.0, 1.0));"
                ));
            }
        }
    }
    if !body
        .iter()
        .any(|line| line.trim_start().starts_with("return "))
    {
        diagnostics.push(format!("shader graph `{}` has no output node", asset.name));
        body.push("    return vec4<f32>(1.0, 0.0, 1.0, 1.0);".to_string());
    }

    let entry = match asset.target {
        ShaderGraphTarget::Material => "zircon_material_graph",
        ShaderGraphTarget::PostProcess => "zircon_post_process_graph",
    };
    ShaderGraphCompileReport {
        wgsl: format!("fn {entry}() -> vec4<f32> {{\n{}\n}}\n", body.join("\n")),
        diagnostics,
    }
}

#[derive(Clone, Debug)]
pub struct RenderingShaderGraphRuntimeFeature;

impl zircon_runtime::plugin::RuntimePluginFeature for RenderingShaderGraphRuntimeFeature {
    fn manifest(&self) -> zircon_runtime::plugin::PluginFeatureBundleManifest {
        feature_manifest()
    }

    fn register_runtime_extensions(
        &self,
        registry: &mut zircon_runtime::plugin::RuntimeExtensionRegistry,
    ) -> Result<(), zircon_runtime::plugin::RuntimeExtensionRegistryError> {
        registry.register_render_feature(render_feature_descriptor())?;
        registry.register_render_pass_executor(render_pass_executor_registration())
    }
}

pub fn runtime_plugin_feature() -> RenderingShaderGraphRuntimeFeature {
    RenderingShaderGraphRuntimeFeature
}

pub fn plugin_feature_registration(
) -> zircon_runtime::plugin::RuntimePluginFeatureRegistrationReport {
    zircon_runtime::plugin::RuntimePluginFeatureRegistrationReport::from_feature(
        &runtime_plugin_feature(),
    )
}

pub fn feature_manifest() -> zircon_runtime::plugin::PluginFeatureBundleManifest {
    zircon_plugin_rendering_runtime::feature_manifest(
        zircon_plugin_rendering_runtime::RenderingFeatureKind::ShaderGraph,
    )
}

pub fn render_feature_descriptor() -> RenderFeatureDescriptor {
    RenderFeatureDescriptor::new(
        FEATURE_NAME,
        vec!["materials".to_string(), "post_process".to_string()],
        Vec::new(),
        vec![RenderFeaturePassDescriptor::new(
            RenderPassStage::PostProcess,
            "shader-graph-post-process",
            QueueLane::Graphics,
        )
        .with_executor_id(EXECUTOR_ID)
        .read_texture("scene-color")
        .write_texture("scene-color")],
    )
}

pub fn render_pass_executor_registration() -> RenderPassExecutorRegistration {
    RenderPassExecutorRegistration::new(EXECUTOR_ID, noop_render_executor)
}

fn noop_render_executor(_context: &mut RenderPassExecutionContext<'_>) -> Result<(), String> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shader_graph_compiles_minimal_color_output_to_wgsl() {
        let asset = ShaderGraphAsset {
            name: "flat".to_string(),
            target: ShaderGraphTarget::PostProcess,
            nodes: vec![
                ShaderGraphNode::ConstantColor {
                    id: "color".to_string(),
                    value: [0.2, 0.3, 0.4, 1.0],
                },
                ShaderGraphNode::ColorOutput {
                    input: "color".to_string(),
                },
            ],
        };

        let report = compile_shader_graph_to_wgsl(&asset);

        assert!(report.diagnostics.is_empty(), "{:?}", report.diagnostics);
        assert!(report.wgsl.contains("fn zircon_post_process_graph"));
        assert!(report.wgsl.contains("return color;"));
    }

    #[test]
    fn shader_graph_feature_is_opt_in() {
        let report = plugin_feature_registration();

        assert!(report.is_success(), "{:?}", report.diagnostics);
        assert!(!report.manifest.enabled_by_default);
        assert_eq!(report.extensions.render_features()[0].name, FEATURE_NAME);
    }
}
