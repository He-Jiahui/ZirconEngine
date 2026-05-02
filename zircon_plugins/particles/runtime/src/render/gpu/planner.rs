use zircon_runtime::core::math::{Real, Transform, Vec3, Vec4};

use crate::{
    ParticleCoordinateSpace, ParticleEmitterAsset, ParticleGpuLayout, ParticleScalarKey,
    ParticleShape, ParticleSimulationError, ParticleSystemAsset,
};

use super::compile_particle_gpu_layout;

#[derive(Clone, Debug, PartialEq)]
pub struct ParticleGpuEmitterFrameParams {
    pub emitter_index: u32,
    pub base_slot: u32,
    pub capacity: u32,
    pub spawn_count: u32,
    pub seed: u32,
    pub coordinate_space: ParticleCoordinateSpace,
    pub shape: ParticleShape,
    pub lifetime: [Real; 2],
    pub initial_size: [Real; 2],
    pub initial_velocity_min: Vec3,
    pub initial_velocity_max: Vec3,
    pub gravity: Vec3,
    pub drag: Real,
    pub dt: Real,
    pub age_seconds: Real,
    pub start_color: Vec4,
    pub end_color: Vec4,
    pub size_curve: [Real; 2],
    pub transform_rows: [[Real; 4]; 4],
}

impl ParticleGpuEmitterFrameParams {
    pub const ENCODED_SIZE: usize = 256;

    pub fn encode(&self, output: &mut Vec<u8>) {
        push_u32(output, self.base_slot);
        push_u32(output, self.capacity);
        push_u32(output, self.spawn_count);
        push_u32(output, self.seed);
        push_u32(output, coordinate_space_code(self.coordinate_space));
        push_u32(output, shape_kind(self.shape));
        push_u32(output, 0);
        push_u32(output, 0);
        push_vec4(output, shape_a(self.shape));
        push_vec4(output, shape_b(self.shape));
        push_vec4(
            output,
            Vec4::new(
                self.lifetime[0],
                self.lifetime[1],
                self.initial_size[0],
                self.initial_size[1],
            ),
        );
        push_vec4(output, self.initial_velocity_min.extend(0.0));
        push_vec4(output, self.initial_velocity_max.extend(0.0));
        push_vec4(output, self.gravity.extend(0.0));
        push_vec4(output, Vec4::new(self.drag, self.dt, self.age_seconds, 0.0));
        push_vec4(output, self.start_color);
        push_vec4(output, self.end_color);
        push_vec4(
            output,
            Vec4::new(self.size_curve[0], self.size_curve[1], 0.0, 0.0),
        );
        for row in self.transform_rows {
            push_vec4(output, Vec4::new(row[0], row[1], row[2], row[3]));
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ParticleGpuFrameParams {
    pub dt: Real,
    pub age_seconds: Real,
    pub emitters: Vec<ParticleGpuEmitterFrameParams>,
}

impl ParticleGpuFrameParams {
    pub fn encode_emitters(&self, layout: &ParticleGpuLayout) -> Vec<u8> {
        let mut encoded = Vec::with_capacity(
            layout.emitter_count.max(1) as usize * ParticleGpuEmitterFrameParams::ENCODED_SIZE,
        );
        for emitter_index in 0..layout.emitter_count.max(1) {
            if let Some(params) = self
                .emitters
                .iter()
                .find(|params| params.emitter_index == emitter_index)
            {
                let mut params = params.clone();
                params.dt = self.dt;
                params.age_seconds = self.age_seconds;
                params.encode(&mut encoded);
            } else {
                encoded.resize(
                    encoded.len() + ParticleGpuEmitterFrameParams::ENCODED_SIZE,
                    0,
                );
            }
        }
        encoded
    }

    pub fn total_spawn_count(&self) -> u32 {
        self.emitters
            .iter()
            .map(|emitter| emitter.spawn_count)
            .sum()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ParticleGpuFramePlanner {
    asset: ParticleSystemAsset,
    layout: ParticleGpuLayout,
    age_seconds: Real,
    spawn_accumulators: Vec<Real>,
    next_burst_indices: Vec<usize>,
}

impl ParticleGpuFramePlanner {
    pub fn new(asset: ParticleSystemAsset) -> Self {
        let layout = compile_particle_gpu_layout(&asset);
        let emitter_count = asset.emitters.len();
        Self {
            asset,
            layout,
            age_seconds: 0.0,
            spawn_accumulators: vec![0.0; emitter_count],
            next_burst_indices: vec![0; emitter_count],
        }
    }

    pub fn layout(&self) -> &ParticleGpuLayout {
        &self.layout
    }

    pub fn reset(&mut self) {
        self.age_seconds = 0.0;
        self.spawn_accumulators.fill(0.0);
        self.next_burst_indices.fill(0);
    }

    pub fn build_frame(
        &mut self,
        dt: Real,
        transform: Transform,
    ) -> Result<ParticleGpuFrameParams, ParticleSimulationError> {
        if !dt.is_finite() || dt < 0.0 {
            return Err(ParticleSimulationError::InvalidDeltaTime);
        }
        let previous_age = self.age_seconds;
        self.age_seconds += dt;
        let transform_rows = transform_rows(transform);
        let mut emitters = Vec::with_capacity(self.asset.emitters.len());

        for index in 0..self.asset.emitters.len() {
            let emitter = self.asset.emitters[index].clone();
            let Some((base_slot, capacity)) = self
                .layout
                .emitters
                .get(index)
                .map(|layout| (layout.base_slot, layout.capacity))
            else {
                continue;
            };
            let spawn_count = self.spawn_count_for(index, &emitter, previous_age, self.age_seconds);
            emitters.push(emitter_frame_params(
                index as u32,
                base_slot,
                capacity,
                spawn_count.min(capacity),
                self.asset.seed,
                &emitter,
                transform_rows,
            ));
        }

        Ok(ParticleGpuFrameParams {
            dt,
            age_seconds: self.age_seconds,
            emitters,
        })
    }

    fn spawn_count_for(
        &mut self,
        emitter_index: usize,
        emitter: &ParticleEmitterAsset,
        previous_age: Real,
        current_age: Real,
    ) -> u32 {
        let mut spawn_count = 0u32;
        while let Some(burst) = emitter
            .bursts
            .get(self.next_burst_indices[emitter_index])
            .copied()
        {
            let burst_time = burst.time_seconds.max(0.0);
            if burst_time > current_age {
                break;
            }
            if burst_time >= previous_age {
                spawn_count = spawn_count.saturating_add(burst.count);
            }
            self.next_burst_indices[emitter_index] += 1;
        }

        if emitter.spawn_rate_per_second > 0.0 {
            self.spawn_accumulators[emitter_index] +=
                (current_age - previous_age) * emitter.spawn_rate_per_second;
            let continuous = self.spawn_accumulators[emitter_index].floor() as u32;
            self.spawn_accumulators[emitter_index] -= continuous as Real;
            spawn_count = spawn_count.saturating_add(continuous);
        }
        spawn_count
    }
}

fn emitter_frame_params(
    emitter_index: u32,
    base_slot: u32,
    capacity: u32,
    spawn_count: u32,
    system_seed: u64,
    emitter: &ParticleEmitterAsset,
    transform_rows: [[Real; 4]; 4],
) -> ParticleGpuEmitterFrameParams {
    let lifetime = emitter.lifetime.normalized();
    let initial_size = emitter.initial_size.normalized();
    ParticleGpuEmitterFrameParams {
        emitter_index,
        base_slot,
        capacity,
        spawn_count,
        seed: gpu_seed(system_seed, emitter_index),
        coordinate_space: emitter.coordinate_space,
        shape: emitter.shape,
        lifetime: [lifetime.min, lifetime.max],
        initial_size: [initial_size.min, initial_size.max],
        initial_velocity_min: emitter.initial_velocity.min,
        initial_velocity_max: emitter.initial_velocity.max,
        gravity: emitter.gravity,
        drag: emitter.drag.max(0.0),
        dt: 0.0,
        age_seconds: 0.0,
        start_color: emitter.start_color * color_endpoint(&emitter.color_over_lifetime, true),
        end_color: emitter.start_color * color_endpoint(&emitter.color_over_lifetime, false),
        size_curve: [
            scalar_endpoint(&emitter.size_over_lifetime, true),
            scalar_endpoint(&emitter.size_over_lifetime, false),
        ],
        transform_rows,
    }
}

fn gpu_seed(system_seed: u64, emitter_index: u32) -> u32 {
    (system_seed as u32) ^ ((system_seed >> 32) as u32) ^ emitter_index.wrapping_mul(0x9E37_79B9)
}

fn scalar_endpoint(keys: &[ParticleScalarKey], first: bool) -> Real {
    match (first, keys.first(), keys.last()) {
        (true, Some(key), _) => key.value,
        (false, _, Some(key)) => key.value,
        _ => 1.0,
    }
}

fn color_endpoint(keys: &[crate::ParticleColorKey], first: bool) -> Vec4 {
    match (first, keys.first(), keys.last()) {
        (true, Some(key), _) => key.value,
        (false, _, Some(key)) => key.value,
        _ => Vec4::ONE,
    }
}

fn coordinate_space_code(space: ParticleCoordinateSpace) -> u32 {
    match space {
        ParticleCoordinateSpace::Local => 0,
        ParticleCoordinateSpace::World => 1,
    }
}

fn shape_kind(shape: ParticleShape) -> u32 {
    match shape {
        ParticleShape::Point => 0,
        ParticleShape::Sphere { .. } => 1,
        ParticleShape::Box { .. } => 2,
        ParticleShape::Cone { .. } => 3,
    }
}

fn shape_a(shape: ParticleShape) -> Vec4 {
    match shape {
        ParticleShape::Point => Vec4::ZERO,
        ParticleShape::Sphere { radius } => Vec4::new(radius.max(0.0), 0.0, 0.0, 0.0),
        ParticleShape::Box { half_extents } => half_extents.abs().extend(0.0),
        ParticleShape::Cone { radius, height } => {
            Vec4::new(radius.max(0.0), height.max(0.0), 0.0, 0.0)
        }
    }
}

fn shape_b(_shape: ParticleShape) -> Vec4 {
    Vec4::ZERO
}

fn transform_rows(transform: Transform) -> [[Real; 4]; 4] {
    let values = transform.matrix().to_cols_array();
    [
        [values[0], values[4], values[8], values[12]],
        [values[1], values[5], values[9], values[13]],
        [values[2], values[6], values[10], values[14]],
        [values[3], values[7], values[11], values[15]],
    ]
}

fn push_u32(output: &mut Vec<u8>, value: u32) {
    output.extend_from_slice(&value.to_le_bytes());
}

fn push_f32(output: &mut Vec<u8>, value: Real) {
    output.extend_from_slice(&value.to_le_bytes());
}

fn push_vec4(output: &mut Vec<u8>, value: Vec4) {
    push_f32(output, value.x);
    push_f32(output, value.y);
    push_f32(output, value.z);
    push_f32(output, value.w);
}
