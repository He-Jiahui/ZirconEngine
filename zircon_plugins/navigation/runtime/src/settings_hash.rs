use zircon_runtime::asset::NavigationSettingsAsset;
use zircon_runtime::core::framework::navigation::NavMeshSurfaceDescriptor;
use zircon_runtime::core::math::Real;

const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

#[derive(Debug)]
struct StableHasher {
    state: u64,
}

impl StableHasher {
    fn new() -> Self {
        Self {
            state: FNV_OFFSET_BASIS,
        }
    }

    fn finish(self) -> u64 {
        self.state
    }

    fn write_bytes(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.state ^= u64::from(*byte);
            self.state = self.state.wrapping_mul(FNV_PRIME);
        }
    }

    fn write_bool(&mut self, value: bool) {
        self.write_bytes(&[u8::from(value)]);
    }

    fn write_u8(&mut self, value: u8) {
        self.write_bytes(&[value]);
    }

    fn write_u32(&mut self, value: u32) {
        self.write_bytes(&value.to_le_bytes());
    }

    fn write_real(&mut self, value: Real) {
        self.write_u32(value.to_bits());
    }

    fn write_real_array(&mut self, values: &[Real; 3]) {
        for value in values {
            self.write_real(*value);
        }
    }

    fn write_str(&mut self, value: &str) {
        self.write_u32(value.len() as u32);
        self.write_bytes(value.as_bytes());
    }

    fn write_string_slice(&mut self, values: &[String]) {
        self.write_u32(values.len() as u32);
        for value in values {
            self.write_str(value);
        }
    }

    fn write_optional_real(&mut self, value: Option<Real>) {
        self.write_bool(value.is_some());
        if let Some(value) = value {
            self.write_real(value);
        }
    }

    fn write_optional_u32(&mut self, value: Option<u32>) {
        self.write_bool(value.is_some());
        if let Some(value) = value {
            self.write_u32(value);
        }
    }
}

pub(crate) fn navigation_settings_hash(
    surface: &NavMeshSurfaceDescriptor,
    settings: &NavigationSettingsAsset,
) -> u64 {
    let mut hasher = StableHasher::new();
    hasher.write_str(&surface.agent_type);
    hasher.write_u8(surface.collect_mode as u8);
    hasher.write_real_array(&surface.volume_center);
    hasher.write_real_array(&surface.volume_size);
    hasher.write_u8(surface.use_geometry as u8);
    hasher.write_string_slice(&surface.include_layers);
    hasher.write_u8(surface.default_area);
    hasher.write_bool(surface.generate_links);
    hasher.write_optional_real(surface.override_voxel_size);
    hasher.write_optional_u32(surface.override_tile_size);
    hasher.write_real(surface.min_region_area);
    hasher.write_bool(surface.build_height_mesh);
    hasher.write_u32(settings.version);
    hasher.write_u32(settings.agents.len() as u32);
    for agent in &settings.agents {
        hasher.write_str(&agent.id);
        hasher.write_str(&agent.display_name);
        hasher.write_real(agent.radius);
        hasher.write_real(agent.height);
        hasher.write_real(agent.max_climb);
        hasher.write_real(agent.max_slope_degrees);
        hasher.write_real(agent.speed);
        hasher.write_real(agent.acceleration);
        hasher.write_real(agent.angular_speed_degrees);
        hasher.write_real(agent.stopping_distance);
    }
    hasher.write_u32(settings.areas.len() as u32);
    for area in &settings.areas {
        hasher.write_u8(area.id);
        hasher.write_str(&area.name);
        hasher.write_real(area.cost);
        hasher.write_bool(area.walkable);
    }
    hasher.finish()
}
