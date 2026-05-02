use super::layout::ParticleGpuLayout;
use super::program::PARTICLE_GPU_WORKGROUP_SIZE;

pub(crate) fn generate_particle_gpu_wgsl(layout: &ParticleGpuLayout) -> String {
    let offset_consts = layout
        .attributes
        .iter()
        .map(|attribute| {
            format!(
                "const OFFSET_{}: u32 = {}u;",
                attribute.name.to_ascii_uppercase(),
                attribute.word_offset
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        r#"{offset_consts}
const PARTICLE_CAPACITY: u32 = {capacity}u;
const EMITTER_COUNT: u32 = {emitter_count}u;
const WORKGROUP_SIZE: u32 = {workgroup_size}u;
const COUNTER_ALIVE_COUNT: u32 = 0u;
const COUNTER_SPAWNED_TOTAL: u32 = 1u;
const COUNTER_DEBUG_FLAGS: u32 = 2u;
const COUNTER_RESERVED: u32 = 3u;
const COUNTER_EMITTER_SPAWN_BASE: u32 = 4u;

struct ParticleWords {{
    words: array<u32>,
}};

struct AliveIndices {{
    indices: array<u32>,
}};

struct Counters {{
    values: array<atomic<u32>>,
}};

struct IndirectDrawArgs {{
    values: array<u32>,
}};

struct GpuEmitterParams {{
    base_slot: u32,
    capacity: u32,
    spawn_count: u32,
    seed: u32,
    coordinate_space: u32,
    shape_kind: u32,
    _pad0: u32,
    _pad1: u32,
    shape_a: vec4<f32>,
    shape_b: vec4<f32>,
    lifetime_size: vec4<f32>,
    initial_velocity_min: vec4<f32>,
    initial_velocity_max: vec4<f32>,
    gravity: vec4<f32>,
    sim: vec4<f32>,
    start_color: vec4<f32>,
    end_color: vec4<f32>,
    size_curve: vec4<f32>,
    transform_row0: vec4<f32>,
    transform_row1: vec4<f32>,
    transform_row2: vec4<f32>,
    transform_row3: vec4<f32>,
}};

struct GpuEmitterParamsBuffer {{
    emitters: array<GpuEmitterParams>,
}};

@group(0) @binding(0) var<storage, read> current_particles: ParticleWords;
@group(0) @binding(1) var<storage, read_write> next_particles: ParticleWords;
@group(0) @binding(2) var<storage, read> emitter_params: GpuEmitterParamsBuffer;
@group(0) @binding(3) var<storage, read_write> counters: Counters;
@group(0) @binding(4) var<storage, read_write> alive_indices: AliveIndices;
@group(0) @binding(5) var<storage, read_write> indirect_args: IndirectDrawArgs;

fn attr_offset(offset: u32, element_words: u32, slot: u32) -> u32 {{
    return offset + slot * element_words;
}}

fn load_current_u32(offset: u32, element_words: u32, slot: u32) -> u32 {{
    return current_particles.words[attr_offset(offset, element_words, slot)];
}}

fn load_current_f32(offset: u32, element_words: u32, slot: u32) -> f32 {{
    return bitcast<f32>(load_current_u32(offset, element_words, slot));
}}

fn load_current_vec3(offset: u32, element_words: u32, slot: u32) -> vec3<f32> {{
    let base = attr_offset(offset, element_words, slot);
    return vec3<f32>(
        bitcast<f32>(current_particles.words[base]),
        bitcast<f32>(current_particles.words[base + 1u]),
        bitcast<f32>(current_particles.words[base + 2u])
    );
}}

fn load_current_vec4(offset: u32, element_words: u32, slot: u32) -> vec4<f32> {{
    let base = attr_offset(offset, element_words, slot);
    return vec4<f32>(
        bitcast<f32>(current_particles.words[base]),
        bitcast<f32>(current_particles.words[base + 1u]),
        bitcast<f32>(current_particles.words[base + 2u]),
        bitcast<f32>(current_particles.words[base + 3u])
    );
}}

fn store_u32(offset: u32, element_words: u32, slot: u32, value: u32) {{
    next_particles.words[attr_offset(offset, element_words, slot)] = value;
}}

fn store_f32(offset: u32, element_words: u32, slot: u32, value: f32) {{
    next_particles.words[attr_offset(offset, element_words, slot)] = bitcast<u32>(value);
}}

fn store_vec3(offset: u32, element_words: u32, slot: u32, value: vec3<f32>) {{
    let base = attr_offset(offset, element_words, slot);
    next_particles.words[base] = bitcast<u32>(value.x);
    next_particles.words[base + 1u] = bitcast<u32>(value.y);
    next_particles.words[base + 2u] = bitcast<u32>(value.z);
}}

fn store_vec4(offset: u32, element_words: u32, slot: u32, value: vec4<f32>) {{
    let base = attr_offset(offset, element_words, slot);
    next_particles.words[base] = bitcast<u32>(value.x);
    next_particles.words[base + 1u] = bitcast<u32>(value.y);
    next_particles.words[base + 2u] = bitcast<u32>(value.z);
    next_particles.words[base + 3u] = bitcast<u32>(value.w);
}}

fn copy_slot(slot: u32) {{
    store_u32(OFFSET_ALIVE, 1u, slot, load_current_u32(OFFSET_ALIVE, 1u, slot));
    store_f32(OFFSET_AGE, 1u, slot, load_current_f32(OFFSET_AGE, 1u, slot));
    store_f32(OFFSET_LIFETIME, 1u, slot, load_current_f32(OFFSET_LIFETIME, 1u, slot));
    store_vec3(OFFSET_POSITION, 3u, slot, load_current_vec3(OFFSET_POSITION, 3u, slot));
    store_vec3(OFFSET_PREVIOUS_POSITION, 3u, slot, load_current_vec3(OFFSET_PREVIOUS_POSITION, 3u, slot));
    store_vec3(OFFSET_VELOCITY, 3u, slot, load_current_vec3(OFFSET_VELOCITY, 3u, slot));
    store_f32(OFFSET_SIZE, 1u, slot, load_current_f32(OFFSET_SIZE, 1u, slot));
    store_f32(OFFSET_INITIAL_SIZE, 1u, slot, load_current_f32(OFFSET_INITIAL_SIZE, 1u, slot));
    store_vec4(OFFSET_COLOR, 4u, slot, load_current_vec4(OFFSET_COLOR, 4u, slot));
    store_vec4(OFFSET_START_COLOR, 4u, slot, load_current_vec4(OFFSET_START_COLOR, 4u, slot));
    store_f32(OFFSET_ROTATION, 1u, slot, load_current_f32(OFFSET_ROTATION, 1u, slot));
    store_f32(OFFSET_ANGULAR_VELOCITY, 1u, slot, load_current_f32(OFFSET_ANGULAR_VELOCITY, 1u, slot));
    store_u32(OFFSET_SEED, 1u, slot, load_current_u32(OFFSET_SEED, 1u, slot));
    store_u32(OFFSET_EMITTER_INDEX, 1u, slot, load_current_u32(OFFSET_EMITTER_INDEX, 1u, slot));
}}

fn find_emitter(slot: u32) -> u32 {{
    var i = 0u;
    loop {{
        if (i >= EMITTER_COUNT) {{
            return 0u;
        }}
        let emitter = emitter_params.emitters[i];
        if (slot >= emitter.base_slot && slot < emitter.base_slot + emitter.capacity) {{
            return i;
        }}
        i = i + 1u;
    }}
}}

fn hash_u32(value: u32) -> u32 {{
    var x = value;
    x = ((x >> 16u) ^ x) * 0x7feb352du;
    x = ((x >> 15u) ^ x) * 0x846ca68bu;
    x = (x >> 16u) ^ x;
    return x;
}}

fn rand01(seed: u32, salt: u32) -> f32 {{
    return f32(hash_u32(seed ^ salt) & 0x00ffffffu) / 16777215.0;
}}

fn rand_signed(seed: u32, salt: u32) -> f32 {{
    return rand01(seed, salt) * 2.0 - 1.0;
}}

fn transform_point(emitter: GpuEmitterParams, position: vec3<f32>) -> vec3<f32> {{
    let p = vec4<f32>(position, 1.0);
    return vec3<f32>(
        dot(emitter.transform_row0, p),
        dot(emitter.transform_row1, p),
        dot(emitter.transform_row2, p)
    );
}}

fn sample_shape(emitter: GpuEmitterParams, seed: u32, slot: u32) -> vec3<f32> {{
    if (emitter.shape_kind == 1u) {{
        let z = 1.0 - 2.0 * rand01(seed, slot + 11u);
        let a = 6.28318530718 * rand01(seed, slot + 17u);
        let radius = emitter.shape_a.x * pow(rand01(seed, slot + 23u), 0.33333334);
        let r = sqrt(max(0.0, 1.0 - z * z)) * radius;
        return vec3<f32>(cos(a) * r, z * radius, sin(a) * r);
    }}
    if (emitter.shape_kind == 2u) {{
        return vec3<f32>(
            rand_signed(seed, slot + 31u) * abs(emitter.shape_a.x),
            rand_signed(seed, slot + 37u) * abs(emitter.shape_a.y),
            rand_signed(seed, slot + 41u) * abs(emitter.shape_a.z)
        );
    }}
    if (emitter.shape_kind == 3u) {{
        let y = rand01(seed, slot + 43u) * max(emitter.shape_a.y, 0.0);
        let taper = 1.0 - clamp(y / max(emitter.shape_a.y, 0.000001), 0.0, 1.0);
        let a = 6.28318530718 * rand01(seed, slot + 47u);
        let r = max(emitter.shape_a.x, 0.0) * taper * sqrt(rand01(seed, slot + 53u));
        return vec3<f32>(cos(a) * r, y, sin(a) * r);
    }}
    return vec3<f32>(0.0, 0.0, 0.0);
}}

fn spawn_particle(slot: u32, emitter_index: u32, emitter: GpuEmitterParams) {{
    let seed = hash_u32(emitter.seed ^ slot ^ bitcast<u32>(emitter.sim.z));
    var position = sample_shape(emitter, seed, slot);
    if (emitter.coordinate_space == 1u) {{
        position = transform_point(emitter, position);
    }}
    let lifetime = mix(emitter.lifetime_size.x, emitter.lifetime_size.y, rand01(seed, slot + 59u));
    let initial_size = max(0.0, mix(emitter.lifetime_size.z, emitter.lifetime_size.w, rand01(seed, slot + 61u)));
    let velocity = vec3<f32>(
        mix(emitter.initial_velocity_min.x, emitter.initial_velocity_max.x, rand01(seed, slot + 67u)),
        mix(emitter.initial_velocity_min.y, emitter.initial_velocity_max.y, rand01(seed, slot + 71u)),
        mix(emitter.initial_velocity_min.z, emitter.initial_velocity_max.z, rand01(seed, slot + 73u))
    );
    store_u32(OFFSET_ALIVE, 1u, slot, 1u);
    store_f32(OFFSET_AGE, 1u, slot, 0.0);
    store_f32(OFFSET_LIFETIME, 1u, slot, max(lifetime, 0.000001));
    store_vec3(OFFSET_POSITION, 3u, slot, position);
    store_vec3(OFFSET_PREVIOUS_POSITION, 3u, slot, position);
    store_vec3(OFFSET_VELOCITY, 3u, slot, velocity);
    store_f32(OFFSET_SIZE, 1u, slot, initial_size * emitter.size_curve.x);
    store_f32(OFFSET_INITIAL_SIZE, 1u, slot, initial_size);
    store_vec4(OFFSET_COLOR, 4u, slot, emitter.start_color);
    store_vec4(OFFSET_START_COLOR, 4u, slot, emitter.start_color);
    store_f32(OFFSET_ROTATION, 1u, slot, 0.0);
    store_f32(OFFSET_ANGULAR_VELOCITY, 1u, slot, 0.0);
    store_u32(OFFSET_SEED, 1u, slot, seed);
    store_u32(OFFSET_EMITTER_INDEX, 1u, slot, emitter_index);
}}

fn update_particle(slot: u32, emitter: GpuEmitterParams) {{
    let dt = emitter.sim.y;
    let previous_position = load_current_vec3(OFFSET_POSITION, 3u, slot);
    var age = load_current_f32(OFFSET_AGE, 1u, slot) + dt;
    let lifetime = load_current_f32(OFFSET_LIFETIME, 1u, slot);
    if (age >= lifetime) {{
        copy_slot(slot);
        store_u32(OFFSET_ALIVE, 1u, slot, 0u);
        return;
    }}
    var velocity = load_current_vec3(OFFSET_VELOCITY, 3u, slot);
    velocity = velocity + emitter.gravity.xyz * dt;
    let drag = clamp(1.0 - max(emitter.sim.x, 0.0) * dt, 0.0, 1.0);
    velocity = velocity * drag;
    let position = previous_position + velocity * dt;
    let normalized_age = clamp(age / max(lifetime, 0.000001), 0.0, 1.0);
    let initial_size = load_current_f32(OFFSET_INITIAL_SIZE, 1u, slot);
    let size_multiplier = mix(emitter.size_curve.x, emitter.size_curve.y, normalized_age);
    let color = mix(emitter.start_color, emitter.end_color, normalized_age);
    copy_slot(slot);
    store_f32(OFFSET_AGE, 1u, slot, age);
    store_vec3(OFFSET_PREVIOUS_POSITION, 3u, slot, previous_position);
    store_vec3(OFFSET_POSITION, 3u, slot, position);
    store_vec3(OFFSET_VELOCITY, 3u, slot, velocity);
    store_f32(OFFSET_SIZE, 1u, slot, max(0.0, initial_size * size_multiplier));
    store_vec4(OFFSET_COLOR, 4u, slot, color);
}}

@compute @workgroup_size(WORKGROUP_SIZE)
fn particle_spawn_update(@builtin(global_invocation_id) id: vec3<u32>) {{
    let slot = id.x;
    if (slot >= PARTICLE_CAPACITY) {{
        return;
    }}
    let emitter_index = find_emitter(slot);
    let emitter = emitter_params.emitters[emitter_index];
    if (slot >= emitter.base_slot + emitter.capacity) {{
        copy_slot(slot);
        return;
    }}
    let alive = load_current_u32(OFFSET_ALIVE, 1u, slot);
    if (alive != 0u) {{
        update_particle(slot, emitter);
        return;
    }}
    let emitted = atomicAdd(&counters.values[COUNTER_EMITTER_SPAWN_BASE + emitter_index], 1u);
    if (emitted < emitter.spawn_count) {{
        atomicAdd(&counters.values[COUNTER_SPAWNED_TOTAL], 1u);
        spawn_particle(slot, emitter_index, emitter);
    }} else {{
        copy_slot(slot);
        store_u32(OFFSET_ALIVE, 1u, slot, 0u);
    }}
}}

@compute @workgroup_size(WORKGROUP_SIZE)
fn particle_compact_alive(@builtin(global_invocation_id) id: vec3<u32>) {{
    let slot = id.x;
    if (slot >= PARTICLE_CAPACITY) {{
        return;
    }}
    let alive = load_current_u32(OFFSET_ALIVE, 1u, slot);
    if (alive != 0u) {{
        let compact_index = atomicAdd(&counters.values[COUNTER_ALIVE_COUNT], 1u);
        alive_indices.indices[compact_index] = slot;
    }}
}}

@compute @workgroup_size(1)
fn particle_build_indirect_args(@builtin(global_invocation_id) id: vec3<u32>) {{
    if (id.x != 0u) {{
        return;
    }}
    indirect_args.values[0] = 6u;
    indirect_args.values[1] = atomicLoad(&counters.values[COUNTER_ALIVE_COUNT]);
    indirect_args.values[2] = 0u;
    indirect_args.values[3] = 0u;
}}
"#,
        capacity = layout.capacity,
        emitter_count = layout.emitter_count.max(1),
        workgroup_size = PARTICLE_GPU_WORKGROUP_SIZE,
    )
}

pub(crate) fn generate_particle_gpu_transparent_wgsl(layout: &ParticleGpuLayout) -> String {
    let position_offset = layout
        .attribute("position")
        .map(|attribute| attribute.word_offset)
        .unwrap_or(0);
    let size_offset = layout
        .attribute("size")
        .map(|attribute| attribute.word_offset)
        .unwrap_or(0);
    let color_offset = layout
        .attribute("color")
        .map(|attribute| attribute.word_offset)
        .unwrap_or(0);

    format!(
        r#"const OFFSET_POSITION: u32 = {position_offset}u;
const OFFSET_SIZE: u32 = {size_offset}u;
const OFFSET_COLOR: u32 = {color_offset}u;

struct SceneUniform {{
    view_proj: mat4x4<f32>,
    light_dir: vec4<f32>,
    light_color: vec4<f32>,
    ambient_color: vec4<f32>,
}};

struct ParticleWords {{
    words: array<u32>,
}};

struct AliveIndices {{
    indices: array<u32>,
}};

struct ParticleTransparentRenderParams {{
    camera_right: vec4<f32>,
    camera_up: vec4<f32>,
}};

struct VertexOutput {{
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
}};

@group(0) @binding(0) var<uniform> scene: SceneUniform;
@group(1) @binding(0) var<storage, read> particles: ParticleWords;
@group(1) @binding(1) var<storage, read> alive_indices: AliveIndices;
@group(1) @binding(2) var<uniform> render_params: ParticleTransparentRenderParams;

fn attr_offset(offset: u32, element_words: u32, slot: u32) -> u32 {{
    return offset + slot * element_words;
}}

fn load_f32(offset: u32, element_words: u32, slot: u32) -> f32 {{
    return bitcast<f32>(particles.words[attr_offset(offset, element_words, slot)]);
}}

fn load_vec3(offset: u32, element_words: u32, slot: u32) -> vec3<f32> {{
    let base = attr_offset(offset, element_words, slot);
    return vec3<f32>(
        bitcast<f32>(particles.words[base]),
        bitcast<f32>(particles.words[base + 1u]),
        bitcast<f32>(particles.words[base + 2u])
    );
}}

fn load_vec4(offset: u32, element_words: u32, slot: u32) -> vec4<f32> {{
    let base = attr_offset(offset, element_words, slot);
    return vec4<f32>(
        bitcast<f32>(particles.words[base]),
        bitcast<f32>(particles.words[base + 1u]),
        bitcast<f32>(particles.words[base + 2u]),
        bitcast<f32>(particles.words[base + 3u])
    );
}}

fn particle_corner(vertex_index: u32) -> vec2<f32> {{
    let corner = vertex_index % 6u;
    if (corner == 0u) {{
        return vec2<f32>(-1.0, 1.0);
    }}
    if (corner == 1u || corner == 4u) {{
        return vec2<f32>(-1.0, -1.0);
    }}
    if (corner == 2u || corner == 3u) {{
        return vec2<f32>(1.0, 1.0);
    }}
    return vec2<f32>(1.0, -1.0);
}}

@vertex
fn particle_gpu_transparent_vs(
    @builtin(vertex_index) vertex_index: u32,
    @builtin(instance_index) instance_index: u32,
) -> VertexOutput {{
    let slot = alive_indices.indices[instance_index];
    let center = load_vec3(OFFSET_POSITION, 3u, slot);
    let size = max(0.0, load_f32(OFFSET_SIZE, 1u, slot));
    let corner = particle_corner(vertex_index) * (size * 0.5);
    let world_position =
        center +
        render_params.camera_right.xyz * corner.x +
        render_params.camera_up.xyz * corner.y;

    var output: VertexOutput;
    output.clip_position = scene.view_proj * vec4<f32>(world_position, 1.0);
    output.color = load_vec4(OFFSET_COLOR, 4u, slot) * vec4<f32>(
        render_params.camera_right.w,
        render_params.camera_right.w,
        render_params.camera_right.w,
        1.0
    );
    return output;
}}

@fragment
fn particle_gpu_transparent_fs(input: VertexOutput) -> @location(0) vec4<f32> {{
    return input.color;
}}
"#,
        position_offset = position_offset,
        size_offset = size_offset,
        color_offset = color_offset,
    )
}
