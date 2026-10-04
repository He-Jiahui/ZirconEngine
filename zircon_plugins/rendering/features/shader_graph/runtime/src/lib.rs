//! 着色器图的运行时公共契约；特性提供者将此处元数据提交到目录与图编译。
use zircon_runtime::graphics::{
    RenderFeatureDescriptor, RenderFeaturePassDescriptor, RenderPassExecutionContext,
    RenderPassExecutorRegistration, RenderPassStage,
};
use zircon_runtime::render_graph::QueueLane;

mod capability;
mod plugin;

pub use capability::{EDITOR_CAPABILITY, RUNTIME_CAPABILITIES, RUNTIME_CAPABILITY};
pub use plugin::{
    feature_manifest, plugin_feature_registration, runtime_plugin_feature,
    RenderingShaderGraphRuntimeFeature,
};

pub const FEATURE_ID: &str = "rendering.shader_graph";
pub const FEATURE_NAME: &str = "shader_graph";
pub const EXECUTOR_ID: &str = "shader-graph.post-process";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// 目标区分材质与后处理入口名称；调用方仍需提供对应的消费管线。
pub enum ShaderGraphTarget {
    Material,
    PostProcess,
}

#[derive(Clone, Debug, PartialEq)]
/// 作者图的中间表示；节点引用和类型尚未由结构本身保证有效。
pub struct ShaderGraphAsset {
    pub name: String,
    pub target: ShaderGraphTarget,
    pub nodes: Vec<ShaderGraphNode>,
}

#[derive(Clone, Debug, PartialEq)]
/// 可输出的图节点族；节点顺序决定生成 WGSL 的声明先后，引用须在使用前可解析。
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
/// WGSL 源码和诊断一起返回；空诊断仅表示已实现的检查通过，不保证引用、类型或源码可编译。
pub struct ShaderGraphCompileReport {
    pub wgsl: String,
    pub diagnostics: Vec<String>,
}

/// 为外部着色器消费链生成函数体；仅靠节点存在性无法保证返回源码可通过 WGSL 编译。
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
    // BUG: [CR-PLUGIN-RENDERING-0004] 无定义的节点引用会生成无效 WGSL 且 diagnostics 为空；例如只有 Add(sum, missing, missing) 与 ColorOutput(sum) 时缺少 missing 声明；证据：匹配分支直接拼接引用，唯一诊断只检查空图和输出节点。
    ShaderGraphCompileReport {
        wgsl: format!("fn {entry}() -> vec4<f32> {{\n{}\n}}\n", body.join("\n")),
        diagnostics,
    }
}

/// 声明可选着色器图后处理槽位；源码生成结果与此图槽位之间没有自动安装关系。
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

/// 为此特性的 executor 标识提供实现句柄；与图描述符一并安装到同一运行期目录。
pub fn render_pass_executor_registration() -> RenderPassExecutorRegistration {
    RenderPassExecutorRegistration::new(EXECUTOR_ID, noop_render_executor)
}

// TODO: [CR-PLUGIN-RENDERING-0005] 确认着色器图编译结果如何绑定后处理通道；当前唯一 executor 只返回成功，源码生成函数无生产调用；下一步追踪图资产加载与管线编译的连接点。
fn noop_render_executor(_context: &mut RenderPassExecutionContext<'_>) -> Result<(), String> {
    Ok(())
}

// 此测试边界覆盖声明与注册约束；GPU 效果证据需由对应产品测试另行提供。
#[cfg(test)]
#[path = "tests/lib.rs"]
mod tests;
