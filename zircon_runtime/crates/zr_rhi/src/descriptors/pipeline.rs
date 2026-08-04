use serde::{Deserialize, Serialize};
use std::ops::{BitOr, BitOrAssign};

use super::{CompareFunction, TextureFormat};
use crate::device::{BindGroupLayoutHandle, PipelineLayoutHandle, ShaderModuleHandle};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShaderStage {
    Vertex,
    Fragment,
    Compute,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShaderModuleDesc {
    pub label: Option<String>,
    pub source: String,
    pub stage: ShaderStage,
    pub entry_point: String,
}

impl ShaderModuleDesc {
    pub fn new(
        label: impl Into<String>,
        stage: ShaderStage,
        entry_point: impl Into<String>,
        source: impl Into<String>,
    ) -> Self {
        Self {
            label: Some(label.into()),
            source: source.into(),
            stage,
            entry_point: entry_point.into(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PipelineKind {
    Raster,
    Compute,
    RayTracing,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PipelineLayoutDesc {
    pub label: Option<String>,
    pub bind_group_layouts: Vec<BindGroupLayoutHandle>,
}

impl PipelineLayoutDesc {
    pub fn new(label: impl Into<String>, bind_group_layouts: Vec<BindGroupLayoutHandle>) -> Self {
        Self {
            label: Some(label.into()),
            bind_group_layouts,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrimitiveTopology {
    PointList,
    LineList,
    LineStrip,
    TriangleList,
    TriangleStrip,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FrontFace {
    Ccw,
    Cw,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CullMode {
    None,
    Front,
    Back,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ColorWriteMask(u32);

impl ColorWriteMask {
    pub const NONE: Self = Self(0);
    pub const RED: Self = Self(1 << 0);
    pub const GREEN: Self = Self(1 << 1);
    pub const BLUE: Self = Self(1 << 2);
    pub const ALPHA: Self = Self(1 << 3);
    pub const COLOR: Self = Self(Self::RED.0 | Self::GREEN.0 | Self::BLUE.0);
    pub const ALL: Self = Self(Self::COLOR.0 | Self::ALPHA.0);

    pub const fn bits(self) -> u32 {
        self.0
    }

    pub const fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    pub const fn has_unknown_bits(self) -> bool {
        (self.0 & !Self::ALL.0) != 0
    }
}

impl Default for ColorWriteMask {
    fn default() -> Self {
        Self::ALL
    }
}

impl BitOr for ColorWriteMask {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for ColorWriteMask {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlendFactor {
    Zero,
    One,
    Src,
    OneMinusSrc,
    SrcAlpha,
    OneMinusSrcAlpha,
    Dst,
    OneMinusDst,
    DstAlpha,
    OneMinusDstAlpha,
    SrcAlphaSaturated,
    Constant,
    OneMinusConstant,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlendOperation {
    Add,
    Subtract,
    ReverseSubtract,
    Min,
    Max,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlendComponentDesc {
    pub src_factor: BlendFactor,
    pub dst_factor: BlendFactor,
    pub operation: BlendOperation,
}

impl BlendComponentDesc {
    pub const fn replace() -> Self {
        Self {
            src_factor: BlendFactor::One,
            dst_factor: BlendFactor::Zero,
            operation: BlendOperation::Add,
        }
    }

    pub const fn alpha_blending() -> Self {
        Self {
            src_factor: BlendFactor::SrcAlpha,
            dst_factor: BlendFactor::OneMinusSrcAlpha,
            operation: BlendOperation::Add,
        }
    }

    pub const fn additive() -> Self {
        Self {
            src_factor: BlendFactor::One,
            dst_factor: BlendFactor::One,
            operation: BlendOperation::Add,
        }
    }
}

impl Default for BlendComponentDesc {
    fn default() -> Self {
        Self::replace()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlendStateDesc {
    pub color: BlendComponentDesc,
    pub alpha: BlendComponentDesc,
}

impl BlendStateDesc {
    pub const fn replace() -> Self {
        Self {
            color: BlendComponentDesc::replace(),
            alpha: BlendComponentDesc::replace(),
        }
    }

    pub const fn alpha_blending() -> Self {
        Self {
            color: BlendComponentDesc::alpha_blending(),
            alpha: BlendComponentDesc {
                src_factor: BlendFactor::One,
                dst_factor: BlendFactor::OneMinusSrcAlpha,
                operation: BlendOperation::Add,
            },
        }
    }

    pub const fn additive() -> Self {
        Self {
            color: BlendComponentDesc::additive(),
            alpha: BlendComponentDesc::additive(),
        }
    }
}

impl Default for BlendStateDesc {
    fn default() -> Self {
        Self::replace()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ColorTargetDesc {
    pub format: TextureFormat,
    #[serde(default)]
    pub blend: Option<BlendStateDesc>,
    pub write_mask: ColorWriteMask,
}

impl ColorTargetDesc {
    pub fn new(format: TextureFormat) -> Self {
        Self {
            format,
            blend: None,
            write_mask: ColorWriteMask::ALL,
        }
    }

    pub const fn with_blend(mut self, blend: BlendStateDesc) -> Self {
        self.blend = Some(blend);
        self
    }

    pub const fn with_write_mask(mut self, write_mask: ColorWriteMask) -> Self {
        self.write_mask = write_mask;
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrimitiveStateDesc {
    pub topology: PrimitiveTopology,
    pub front_face: FrontFace,
    pub cull_mode: CullMode,
}

impl PrimitiveStateDesc {
    pub const fn triangle_list() -> Self {
        Self {
            topology: PrimitiveTopology::TriangleList,
            front_face: FrontFace::Ccw,
            cull_mode: CullMode::None,
        }
    }

    pub const fn with_topology(mut self, topology: PrimitiveTopology) -> Self {
        self.topology = topology;
        self
    }

    pub const fn with_front_face(mut self, front_face: FrontFace) -> Self {
        self.front_face = front_face;
        self
    }

    pub const fn with_cull_mode(mut self, cull_mode: CullMode) -> Self {
        self.cull_mode = cull_mode;
        self
    }
}

impl Default for PrimitiveStateDesc {
    fn default() -> Self {
        Self::triangle_list()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DepthStencilStateDesc {
    pub format: TextureFormat,
    pub depth_write_enabled: bool,
    pub depth_compare: CompareFunction,
    pub stencil_enabled: bool,
}

impl DepthStencilStateDesc {
    pub const fn new(
        format: TextureFormat,
        depth_write_enabled: bool,
        depth_compare: CompareFunction,
    ) -> Self {
        Self {
            format,
            depth_write_enabled,
            depth_compare,
            stencil_enabled: false,
        }
    }

    pub const fn with_stencil_enabled(mut self, stencil_enabled: bool) -> Self {
        self.stencil_enabled = stencil_enabled;
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum VertexFormat {
    Uint8x2,
    Uint8x4,
    Sint8x2,
    Sint8x4,
    Unorm8x2,
    Unorm8x4,
    Snorm8x2,
    Snorm8x4,
    Uint16x2,
    Uint16x4,
    Sint16x2,
    Sint16x4,
    Unorm16x2,
    Unorm16x4,
    Snorm16x2,
    Snorm16x4,
    Float16x2,
    Float16x4,
    Float32,
    Float32x2,
    Float32x3,
    Float32x4,
    Uint32,
    Uint32x2,
    Uint32x3,
    Uint32x4,
    Sint32,
    Sint32x2,
    Sint32x3,
    Sint32x4,
}

impl VertexFormat {
    pub const fn size_bytes(self) -> u64 {
        match self {
            Self::Uint8x2 | Self::Sint8x2 | Self::Unorm8x2 | Self::Snorm8x2 => 2,
            Self::Uint8x4
            | Self::Sint8x4
            | Self::Unorm8x4
            | Self::Snorm8x4
            | Self::Uint16x2
            | Self::Sint16x2
            | Self::Unorm16x2
            | Self::Snorm16x2
            | Self::Float16x2
            | Self::Float32
            | Self::Uint32
            | Self::Sint32 => 4,
            Self::Uint16x4
            | Self::Sint16x4
            | Self::Unorm16x4
            | Self::Snorm16x4
            | Self::Float16x4
            | Self::Float32x2
            | Self::Uint32x2
            | Self::Sint32x2 => 8,
            Self::Float32x3 | Self::Uint32x3 | Self::Sint32x3 => 12,
            Self::Float32x4 | Self::Uint32x4 | Self::Sint32x4 => 16,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum VertexStepMode {
    Vertex,
    Instance,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VertexAttributeDesc {
    pub shader_location: u32,
    pub offset: u64,
    pub format: VertexFormat,
}

impl VertexAttributeDesc {
    pub const fn new(shader_location: u32, offset: u64, format: VertexFormat) -> Self {
        Self {
            shader_location,
            offset,
            format,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VertexBufferLayoutDesc {
    pub array_stride: u64,
    pub step_mode: VertexStepMode,
    pub attributes: Vec<VertexAttributeDesc>,
}

impl VertexBufferLayoutDesc {
    pub fn new(array_stride: u64, attributes: Vec<VertexAttributeDesc>) -> Self {
        Self {
            array_stride,
            step_mode: VertexStepMode::Vertex,
            attributes,
        }
    }

    pub fn with_step_mode(mut self, step_mode: VertexStepMode) -> Self {
        self.step_mode = step_mode;
        self
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VertexInputLayoutDesc {
    pub buffers: Vec<VertexBufferLayoutDesc>,
}

impl VertexInputLayoutDesc {
    pub fn empty() -> Self {
        Self {
            buffers: Vec::new(),
        }
    }

    pub fn new(buffers: Vec<VertexBufferLayoutDesc>) -> Self {
        Self { buffers }
    }
}

impl Default for VertexInputLayoutDesc {
    fn default() -> Self {
        Self::empty()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RasterPipelineStateDesc {
    pub color_targets: Vec<ColorTargetDesc>,
    pub depth_stencil: Option<DepthStencilStateDesc>,
    pub primitive: PrimitiveStateDesc,
    pub sample_count: u32,
    #[serde(default)]
    pub vertex_input: VertexInputLayoutDesc,
}

impl RasterPipelineStateDesc {
    pub fn new(color_targets: Vec<ColorTargetDesc>) -> Self {
        Self {
            color_targets,
            depth_stencil: None,
            primitive: PrimitiveStateDesc::default(),
            sample_count: 1,
            vertex_input: VertexInputLayoutDesc::empty(),
        }
    }

    pub fn single_color(format: TextureFormat) -> Self {
        Self::new(vec![ColorTargetDesc::new(format)])
    }

    pub fn depth_only(depth_stencil: DepthStencilStateDesc) -> Self {
        Self {
            color_targets: Vec::new(),
            depth_stencil: Some(depth_stencil),
            primitive: PrimitiveStateDesc::default(),
            sample_count: 1,
            vertex_input: VertexInputLayoutDesc::empty(),
        }
    }

    pub fn with_depth_stencil(mut self, depth_stencil: DepthStencilStateDesc) -> Self {
        self.depth_stencil = Some(depth_stencil);
        self
    }

    pub const fn with_primitive(mut self, primitive: PrimitiveStateDesc) -> Self {
        self.primitive = primitive;
        self
    }

    pub const fn with_sample_count(mut self, sample_count: u32) -> Self {
        self.sample_count = sample_count;
        self
    }

    pub fn with_vertex_input(mut self, vertex_input: VertexInputLayoutDesc) -> Self {
        self.vertex_input = vertex_input;
        self
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PipelineDesc {
    pub label: Option<String>,
    pub kind: PipelineKind,
    pub layout: Option<PipelineLayoutHandle>,
    pub vertex_shader: Option<ShaderModuleHandle>,
    pub fragment_shader: Option<ShaderModuleHandle>,
    pub compute_shader: Option<ShaderModuleHandle>,
    pub raster_state: Option<RasterPipelineStateDesc>,
}

impl PipelineDesc {
    pub fn new(label: impl Into<String>, kind: PipelineKind) -> Self {
        Self {
            label: Some(label.into()),
            kind,
            layout: None,
            vertex_shader: None,
            fragment_shader: None,
            compute_shader: None,
            raster_state: None,
        }
    }

    pub const fn with_layout(mut self, layout: PipelineLayoutHandle) -> Self {
        self.layout = Some(layout);
        self
    }

    pub const fn with_vertex_shader(mut self, shader: ShaderModuleHandle) -> Self {
        self.vertex_shader = Some(shader);
        self
    }

    pub const fn with_fragment_shader(mut self, shader: ShaderModuleHandle) -> Self {
        self.fragment_shader = Some(shader);
        self
    }

    pub const fn with_compute_shader(mut self, shader: ShaderModuleHandle) -> Self {
        self.compute_shader = Some(shader);
        self
    }

    pub fn with_raster_state(mut self, raster_state: RasterPipelineStateDesc) -> Self {
        self.raster_state = Some(raster_state);
        self
    }
}
