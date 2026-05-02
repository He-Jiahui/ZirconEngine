use crate::ParticleSystemAsset;

pub const PARTICLE_GPU_MAX_PARTICLES: u32 = 1_048_576;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParticleGpuValueType {
    Float,
    Vec3,
    Vec4,
    Uint,
}

impl ParticleGpuValueType {
    pub const fn word_count(self) -> u32 {
        match self {
            Self::Float | Self::Uint => 1,
            Self::Vec3 => 3,
            Self::Vec4 => 4,
        }
    }

    pub const fn wgsl_scalar(self) -> &'static str {
        match self {
            Self::Float => "f32",
            Self::Vec3 => "vec3<f32>",
            Self::Vec4 => "vec4<f32>",
            Self::Uint => "u32",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParticleGpuAttribute {
    pub name: &'static str,
    pub value_type: ParticleGpuValueType,
    pub word_offset: u32,
    pub element_words: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParticleGpuEmitterLayout {
    pub emitter_index: u32,
    pub emitter_id: String,
    pub base_slot: u32,
    pub capacity: u32,
    pub requested_capacity: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParticleGpuLayout {
    pub capacity: u32,
    pub requested_capacity: u64,
    pub stride_words: u32,
    pub total_words: u64,
    pub attributes: Vec<ParticleGpuAttribute>,
    pub emitters: Vec<ParticleGpuEmitterLayout>,
    pub emitter_count: u32,
    pub clamped: bool,
}

impl ParticleGpuLayout {
    pub fn attribute(&self, name: &str) -> Option<&ParticleGpuAttribute> {
        self.attributes
            .iter()
            .find(|attribute| attribute.name == name)
    }

    pub fn storage_bytes(&self) -> u64 {
        self.total_words
            .saturating_mul(std::mem::size_of::<u32>() as u64)
    }

    pub fn dispatch_workgroups(&self, workgroup_size: u32) -> u32 {
        if self.capacity == 0 {
            0
        } else {
            self.capacity.div_ceil(workgroup_size.max(1))
        }
    }
}

pub fn compile_particle_gpu_layout(asset: &ParticleSystemAsset) -> ParticleGpuLayout {
    let requested_capacity = asset
        .emitters
        .iter()
        .map(|emitter| emitter.max_particles as u64)
        .sum::<u64>();
    let capacity = requested_capacity.min(PARTICLE_GPU_MAX_PARTICLES as u64) as u32;

    let mut remaining_capacity = capacity;
    let mut base_slot = 0u32;
    let mut emitters = Vec::with_capacity(asset.emitters.len());
    for (index, emitter) in asset.emitters.iter().enumerate() {
        let emitter_capacity = emitter.max_particles.min(remaining_capacity);
        emitters.push(ParticleGpuEmitterLayout {
            emitter_index: index as u32,
            emitter_id: emitter.id.clone(),
            base_slot,
            capacity: emitter_capacity,
            requested_capacity: emitter.max_particles,
        });
        base_slot = base_slot.saturating_add(emitter_capacity);
        remaining_capacity = remaining_capacity.saturating_sub(emitter_capacity);
    }

    let attribute_defs = [
        ("alive", ParticleGpuValueType::Uint),
        ("age", ParticleGpuValueType::Float),
        ("lifetime", ParticleGpuValueType::Float),
        ("position", ParticleGpuValueType::Vec3),
        ("previous_position", ParticleGpuValueType::Vec3),
        ("velocity", ParticleGpuValueType::Vec3),
        ("size", ParticleGpuValueType::Float),
        ("initial_size", ParticleGpuValueType::Float),
        ("color", ParticleGpuValueType::Vec4),
        ("start_color", ParticleGpuValueType::Vec4),
        ("rotation", ParticleGpuValueType::Float),
        ("angular_velocity", ParticleGpuValueType::Float),
        ("seed", ParticleGpuValueType::Uint),
        ("emitter_index", ParticleGpuValueType::Uint),
    ];

    let stride_words = attribute_defs
        .iter()
        .map(|(_, value_type)| value_type.word_count())
        .sum::<u32>();
    let mut word_offset = 0u64;
    let mut attributes = Vec::with_capacity(attribute_defs.len());
    for (name, value_type) in attribute_defs {
        attributes.push(ParticleGpuAttribute {
            name,
            value_type,
            word_offset: word_offset.min(u32::MAX as u64) as u32,
            element_words: value_type.word_count(),
        });
        word_offset = word_offset.saturating_add(capacity as u64 * value_type.word_count() as u64);
    }

    ParticleGpuLayout {
        capacity,
        requested_capacity,
        stride_words,
        total_words: word_offset,
        attributes,
        emitters,
        emitter_count: asset.emitters.len() as u32,
        clamped: requested_capacity > PARTICLE_GPU_MAX_PARTICLES as u64,
    }
}
