use zr_rhi::{
    BufferUsage, ColorWriteMask, SamplerBindingType, StorageTextureAccess, TextureCopyAspect,
    TextureDimension, TextureFormat, TextureSampleType, TextureUsage, TextureViewAspect,
    TextureViewDimension, VertexAttributeDesc,
};

pub(super) fn wgpu_buffer_usage(usage: BufferUsage) -> wgpu::BufferUsages {
    let mut native = wgpu::BufferUsages::empty();
    if usage.contains(BufferUsage::VERTEX) {
        native |= wgpu::BufferUsages::VERTEX;
    }
    if usage.contains(BufferUsage::INDEX) {
        native |= wgpu::BufferUsages::INDEX;
    }
    if usage.contains(BufferUsage::UNIFORM) {
        native |= wgpu::BufferUsages::UNIFORM;
    }
    if usage.contains(BufferUsage::STORAGE) {
        native |= wgpu::BufferUsages::STORAGE;
    }
    if usage.contains(BufferUsage::STAGING_READ) {
        native |= wgpu::BufferUsages::MAP_READ;
    }
    if usage.contains(BufferUsage::STAGING_WRITE) {
        native |= wgpu::BufferUsages::MAP_WRITE;
    }
    if usage.contains(BufferUsage::INDIRECT) {
        native |= wgpu::BufferUsages::INDIRECT;
    }
    if usage.contains(BufferUsage::COPY_SRC) {
        native |= wgpu::BufferUsages::COPY_SRC;
    }
    if usage.contains(BufferUsage::COPY_DST) {
        native |= wgpu::BufferUsages::COPY_DST;
    }
    native
}

pub(super) fn wgpu_texture_usage(usage: TextureUsage) -> wgpu::TextureUsages {
    let mut native = wgpu::TextureUsages::empty();
    if usage.contains(TextureUsage::RENDER_ATTACHMENT) {
        native |= wgpu::TextureUsages::RENDER_ATTACHMENT;
    }
    if usage.contains(TextureUsage::SAMPLED) {
        native |= wgpu::TextureUsages::TEXTURE_BINDING;
    }
    if usage.contains(TextureUsage::STORAGE) {
        native |= wgpu::TextureUsages::STORAGE_BINDING;
    }
    if usage.contains(TextureUsage::COPY_SRC) {
        native |= wgpu::TextureUsages::COPY_SRC;
    }
    if usage.contains(TextureUsage::COPY_DST) {
        native |= wgpu::TextureUsages::COPY_DST;
    }
    native
}

pub(super) const fn wgpu_texture_dimension(dimension: TextureDimension) -> wgpu::TextureDimension {
    match dimension {
        TextureDimension::D1 => wgpu::TextureDimension::D1,
        TextureDimension::D2 | TextureDimension::D2Array | TextureDimension::Cube => {
            wgpu::TextureDimension::D2
        }
        TextureDimension::D3 => wgpu::TextureDimension::D3,
    }
}

pub(super) const fn wgpu_texture_view_dimension(
    dimension: TextureViewDimension,
) -> wgpu::TextureViewDimension {
    match dimension {
        TextureViewDimension::D1 => wgpu::TextureViewDimension::D1,
        TextureViewDimension::D2 => wgpu::TextureViewDimension::D2,
        TextureViewDimension::D2Array => wgpu::TextureViewDimension::D2Array,
        TextureViewDimension::D3 => wgpu::TextureViewDimension::D3,
        TextureViewDimension::Cube => wgpu::TextureViewDimension::Cube,
        TextureViewDimension::CubeArray => wgpu::TextureViewDimension::CubeArray,
    }
}

pub(super) const fn wgpu_texture_view_aspect(aspect: TextureViewAspect) -> wgpu::TextureAspect {
    match aspect {
        TextureViewAspect::All => wgpu::TextureAspect::All,
        TextureViewAspect::DepthOnly => wgpu::TextureAspect::DepthOnly,
        TextureViewAspect::StencilOnly => wgpu::TextureAspect::StencilOnly,
    }
}

pub(super) const fn wgpu_texture_copy_aspect(aspect: TextureCopyAspect) -> wgpu::TextureAspect {
    match aspect {
        TextureCopyAspect::All => wgpu::TextureAspect::All,
        TextureCopyAspect::DepthOnly => wgpu::TextureAspect::DepthOnly,
        TextureCopyAspect::StencilOnly => wgpu::TextureAspect::StencilOnly,
    }
}

pub(super) const fn wgpu_texture_sample_type(
    sample_type: TextureSampleType,
) -> wgpu::TextureSampleType {
    match sample_type {
        TextureSampleType::Float { filterable } => wgpu::TextureSampleType::Float { filterable },
        TextureSampleType::Depth => wgpu::TextureSampleType::Depth,
        TextureSampleType::Sint => wgpu::TextureSampleType::Sint,
        TextureSampleType::Uint => wgpu::TextureSampleType::Uint,
    }
}

pub(super) const fn wgpu_sampler_binding_type(
    binding_type: SamplerBindingType,
) -> wgpu::SamplerBindingType {
    match binding_type {
        SamplerBindingType::Filtering => wgpu::SamplerBindingType::Filtering,
        SamplerBindingType::NonFiltering => wgpu::SamplerBindingType::NonFiltering,
        SamplerBindingType::Comparison => wgpu::SamplerBindingType::Comparison,
    }
}

pub(super) const fn wgpu_storage_texture_access(
    access: StorageTextureAccess,
) -> wgpu::StorageTextureAccess {
    match access {
        StorageTextureAccess::WriteOnly => wgpu::StorageTextureAccess::WriteOnly,
    }
}

pub(super) const fn wgpu_texture_format(format: TextureFormat) -> wgpu::TextureFormat {
    match format {
        TextureFormat::R8Unorm => wgpu::TextureFormat::R8Unorm,
        TextureFormat::R16Float => wgpu::TextureFormat::R16Float,
        TextureFormat::R32Float => wgpu::TextureFormat::R32Float,
        TextureFormat::Rg16Float => wgpu::TextureFormat::Rg16Float,
        TextureFormat::Rg11b10Ufloat => wgpu::TextureFormat::Rg11b10Ufloat,
        TextureFormat::Rgba8Unorm => wgpu::TextureFormat::Rgba8Unorm,
        TextureFormat::Rgba8UnormSrgb => wgpu::TextureFormat::Rgba8UnormSrgb,
        TextureFormat::Bgra8Unorm => wgpu::TextureFormat::Bgra8Unorm,
        TextureFormat::Bgra8UnormSrgb => wgpu::TextureFormat::Bgra8UnormSrgb,
        TextureFormat::Rgba16Float => wgpu::TextureFormat::Rgba16Float,
        TextureFormat::Rgba32Float => wgpu::TextureFormat::Rgba32Float,
        TextureFormat::Depth24Plus => wgpu::TextureFormat::Depth24Plus,
        TextureFormat::Depth24PlusStencil8 => wgpu::TextureFormat::Depth24PlusStencil8,
        TextureFormat::Depth32Float => wgpu::TextureFormat::Depth32Float,
    }
}

pub(super) const fn wgpu_address_mode(mode: zr_rhi::AddressMode) -> wgpu::AddressMode {
    match mode {
        zr_rhi::AddressMode::ClampToEdge => wgpu::AddressMode::ClampToEdge,
        zr_rhi::AddressMode::Repeat => wgpu::AddressMode::Repeat,
        zr_rhi::AddressMode::MirrorRepeat => wgpu::AddressMode::MirrorRepeat,
    }
}

pub(super) const fn wgpu_filter_mode(mode: zr_rhi::FilterMode) -> wgpu::FilterMode {
    match mode {
        zr_rhi::FilterMode::Nearest => wgpu::FilterMode::Nearest,
        zr_rhi::FilterMode::Linear => wgpu::FilterMode::Linear,
    }
}

pub(super) const fn wgpu_mipmap_filter_mode(
    mode: zr_rhi::MipmapFilterMode,
) -> wgpu::MipmapFilterMode {
    match mode {
        zr_rhi::MipmapFilterMode::Nearest => wgpu::MipmapFilterMode::Nearest,
        zr_rhi::MipmapFilterMode::Linear => wgpu::MipmapFilterMode::Linear,
    }
}

pub(super) const fn wgpu_compare_function(
    function: zr_rhi::CompareFunction,
) -> wgpu::CompareFunction {
    match function {
        zr_rhi::CompareFunction::Never => wgpu::CompareFunction::Never,
        zr_rhi::CompareFunction::Less => wgpu::CompareFunction::Less,
        zr_rhi::CompareFunction::LessEqual => wgpu::CompareFunction::LessEqual,
        zr_rhi::CompareFunction::Equal => wgpu::CompareFunction::Equal,
        zr_rhi::CompareFunction::GreaterEqual => wgpu::CompareFunction::GreaterEqual,
        zr_rhi::CompareFunction::Greater => wgpu::CompareFunction::Greater,
        zr_rhi::CompareFunction::NotEqual => wgpu::CompareFunction::NotEqual,
        zr_rhi::CompareFunction::Always => wgpu::CompareFunction::Always,
    }
}

pub(super) const fn wgpu_primitive_topology(
    topology: zr_rhi::PrimitiveTopology,
) -> wgpu::PrimitiveTopology {
    match topology {
        zr_rhi::PrimitiveTopology::PointList => wgpu::PrimitiveTopology::PointList,
        zr_rhi::PrimitiveTopology::LineList => wgpu::PrimitiveTopology::LineList,
        zr_rhi::PrimitiveTopology::LineStrip => wgpu::PrimitiveTopology::LineStrip,
        zr_rhi::PrimitiveTopology::TriangleList => wgpu::PrimitiveTopology::TriangleList,
        zr_rhi::PrimitiveTopology::TriangleStrip => wgpu::PrimitiveTopology::TriangleStrip,
    }
}

pub(super) const fn wgpu_front_face(front_face: zr_rhi::FrontFace) -> wgpu::FrontFace {
    match front_face {
        zr_rhi::FrontFace::Ccw => wgpu::FrontFace::Ccw,
        zr_rhi::FrontFace::Cw => wgpu::FrontFace::Cw,
    }
}

pub(super) const fn wgpu_cull_mode(cull_mode: zr_rhi::CullMode) -> Option<wgpu::Face> {
    match cull_mode {
        zr_rhi::CullMode::None => None,
        zr_rhi::CullMode::Front => Some(wgpu::Face::Front),
        zr_rhi::CullMode::Back => Some(wgpu::Face::Back),
    }
}

pub(super) const fn wgpu_blend_state(blend: zr_rhi::BlendStateDesc) -> wgpu::BlendState {
    wgpu::BlendState {
        color: wgpu_blend_component(blend.color),
        alpha: wgpu_blend_component(blend.alpha),
    }
}

const fn wgpu_blend_component(component: zr_rhi::BlendComponentDesc) -> wgpu::BlendComponent {
    wgpu::BlendComponent {
        src_factor: wgpu_blend_factor(component.src_factor),
        dst_factor: wgpu_blend_factor(component.dst_factor),
        operation: wgpu_blend_operation(component.operation),
    }
}

const fn wgpu_blend_factor(factor: zr_rhi::BlendFactor) -> wgpu::BlendFactor {
    match factor {
        zr_rhi::BlendFactor::Zero => wgpu::BlendFactor::Zero,
        zr_rhi::BlendFactor::One => wgpu::BlendFactor::One,
        zr_rhi::BlendFactor::Src => wgpu::BlendFactor::Src,
        zr_rhi::BlendFactor::OneMinusSrc => wgpu::BlendFactor::OneMinusSrc,
        zr_rhi::BlendFactor::SrcAlpha => wgpu::BlendFactor::SrcAlpha,
        zr_rhi::BlendFactor::OneMinusSrcAlpha => wgpu::BlendFactor::OneMinusSrcAlpha,
        zr_rhi::BlendFactor::Dst => wgpu::BlendFactor::Dst,
        zr_rhi::BlendFactor::OneMinusDst => wgpu::BlendFactor::OneMinusDst,
        zr_rhi::BlendFactor::DstAlpha => wgpu::BlendFactor::DstAlpha,
        zr_rhi::BlendFactor::OneMinusDstAlpha => wgpu::BlendFactor::OneMinusDstAlpha,
        zr_rhi::BlendFactor::SrcAlphaSaturated => wgpu::BlendFactor::SrcAlphaSaturated,
        zr_rhi::BlendFactor::Constant => wgpu::BlendFactor::Constant,
        zr_rhi::BlendFactor::OneMinusConstant => wgpu::BlendFactor::OneMinusConstant,
    }
}

const fn wgpu_blend_operation(operation: zr_rhi::BlendOperation) -> wgpu::BlendOperation {
    match operation {
        zr_rhi::BlendOperation::Add => wgpu::BlendOperation::Add,
        zr_rhi::BlendOperation::Subtract => wgpu::BlendOperation::Subtract,
        zr_rhi::BlendOperation::ReverseSubtract => wgpu::BlendOperation::ReverseSubtract,
        zr_rhi::BlendOperation::Min => wgpu::BlendOperation::Min,
        zr_rhi::BlendOperation::Max => wgpu::BlendOperation::Max,
    }
}

pub(super) fn wgpu_color_writes(mask: ColorWriteMask) -> wgpu::ColorWrites {
    let mut writes = wgpu::ColorWrites::empty();
    if mask.contains(ColorWriteMask::RED) {
        writes |= wgpu::ColorWrites::RED;
    }
    if mask.contains(ColorWriteMask::GREEN) {
        writes |= wgpu::ColorWrites::GREEN;
    }
    if mask.contains(ColorWriteMask::BLUE) {
        writes |= wgpu::ColorWrites::BLUE;
    }
    if mask.contains(ColorWriteMask::ALPHA) {
        writes |= wgpu::ColorWrites::ALPHA;
    }
    writes
}

pub(super) fn wgpu_vertex_attribute(attribute: &VertexAttributeDesc) -> wgpu::VertexAttribute {
    wgpu::VertexAttribute {
        format: wgpu_vertex_format(attribute.format),
        offset: attribute.offset,
        shader_location: attribute.shader_location,
    }
}

pub(super) const fn wgpu_vertex_step_mode(
    step_mode: zr_rhi::VertexStepMode,
) -> wgpu::VertexStepMode {
    match step_mode {
        zr_rhi::VertexStepMode::Vertex => wgpu::VertexStepMode::Vertex,
        zr_rhi::VertexStepMode::Instance => wgpu::VertexStepMode::Instance,
    }
}

const fn wgpu_vertex_format(format: zr_rhi::VertexFormat) -> wgpu::VertexFormat {
    match format {
        zr_rhi::VertexFormat::Uint8x2 => wgpu::VertexFormat::Uint8x2,
        zr_rhi::VertexFormat::Uint8x4 => wgpu::VertexFormat::Uint8x4,
        zr_rhi::VertexFormat::Sint8x2 => wgpu::VertexFormat::Sint8x2,
        zr_rhi::VertexFormat::Sint8x4 => wgpu::VertexFormat::Sint8x4,
        zr_rhi::VertexFormat::Unorm8x2 => wgpu::VertexFormat::Unorm8x2,
        zr_rhi::VertexFormat::Unorm8x4 => wgpu::VertexFormat::Unorm8x4,
        zr_rhi::VertexFormat::Snorm8x2 => wgpu::VertexFormat::Snorm8x2,
        zr_rhi::VertexFormat::Snorm8x4 => wgpu::VertexFormat::Snorm8x4,
        zr_rhi::VertexFormat::Uint16x2 => wgpu::VertexFormat::Uint16x2,
        zr_rhi::VertexFormat::Uint16x4 => wgpu::VertexFormat::Uint16x4,
        zr_rhi::VertexFormat::Sint16x2 => wgpu::VertexFormat::Sint16x2,
        zr_rhi::VertexFormat::Sint16x4 => wgpu::VertexFormat::Sint16x4,
        zr_rhi::VertexFormat::Unorm16x2 => wgpu::VertexFormat::Unorm16x2,
        zr_rhi::VertexFormat::Unorm16x4 => wgpu::VertexFormat::Unorm16x4,
        zr_rhi::VertexFormat::Snorm16x2 => wgpu::VertexFormat::Snorm16x2,
        zr_rhi::VertexFormat::Snorm16x4 => wgpu::VertexFormat::Snorm16x4,
        zr_rhi::VertexFormat::Float16x2 => wgpu::VertexFormat::Float16x2,
        zr_rhi::VertexFormat::Float16x4 => wgpu::VertexFormat::Float16x4,
        zr_rhi::VertexFormat::Float32 => wgpu::VertexFormat::Float32,
        zr_rhi::VertexFormat::Float32x2 => wgpu::VertexFormat::Float32x2,
        zr_rhi::VertexFormat::Float32x3 => wgpu::VertexFormat::Float32x3,
        zr_rhi::VertexFormat::Float32x4 => wgpu::VertexFormat::Float32x4,
        zr_rhi::VertexFormat::Uint32 => wgpu::VertexFormat::Uint32,
        zr_rhi::VertexFormat::Uint32x2 => wgpu::VertexFormat::Uint32x2,
        zr_rhi::VertexFormat::Uint32x3 => wgpu::VertexFormat::Uint32x3,
        zr_rhi::VertexFormat::Uint32x4 => wgpu::VertexFormat::Uint32x4,
        zr_rhi::VertexFormat::Sint32 => wgpu::VertexFormat::Sint32,
        zr_rhi::VertexFormat::Sint32x2 => wgpu::VertexFormat::Sint32x2,
        zr_rhi::VertexFormat::Sint32x3 => wgpu::VertexFormat::Sint32x3,
        zr_rhi::VertexFormat::Sint32x4 => wgpu::VertexFormat::Sint32x4,
    }
}
