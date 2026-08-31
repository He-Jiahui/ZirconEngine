#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum ShaderBindingStage {
    Vertex,
    Task,
    Mesh,
    Fragment,
    Compute,
    RayGeneration,
    Miss,
    AnyHit,
    ClosestHit,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct ShaderBindingVisibility(u16);

impl ShaderBindingVisibility {
    pub(crate) fn from_stages(stages: impl IntoIterator<Item = ShaderBindingStage>) -> Self {
        let mut visibility = Self::default();
        for stage in stages {
            visibility.insert(stage);
        }
        visibility
    }

    pub(crate) fn contains(self, stage: ShaderBindingStage) -> bool {
        self.0 & shader_binding_stage_bit(stage) != 0
    }

    pub(crate) fn insert(&mut self, stage: ShaderBindingStage) {
        self.0 |= shader_binding_stage_bit(stage);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum ShaderTextureViewDimension {
    D1,
    D2,
    D2Array,
    Cube,
    CubeArray,
    D3,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum ShaderTextureSampleType {
    Float,
    Depth,
    Sint,
    Uint,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum ShaderBindingResourceType {
    UniformBuffer,
    StorageBuffer {
        read_only: bool,
    },
    SampledTexture {
        view_dimension: ShaderTextureViewDimension,
        sample_type: ShaderTextureSampleType,
        multisampled: bool,
    },
    Sampler {
        comparison: bool,
    },
    Unsupported,
}

const fn shader_binding_stage_bit(stage: ShaderBindingStage) -> u16 {
    match stage {
        ShaderBindingStage::Vertex => 1 << 0,
        ShaderBindingStage::Task => 1 << 1,
        ShaderBindingStage::Mesh => 1 << 2,
        ShaderBindingStage::Fragment => 1 << 3,
        ShaderBindingStage::Compute => 1 << 4,
        ShaderBindingStage::RayGeneration => 1 << 5,
        ShaderBindingStage::Miss => 1 << 6,
        ShaderBindingStage::AnyHit => 1 << 7,
        ShaderBindingStage::ClosestHit => 1 << 8,
    }
}
