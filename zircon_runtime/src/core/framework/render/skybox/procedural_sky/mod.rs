#[path = "bake_key.rs"]
mod bake_key;
#[path = "constants.rs"]
mod constants;
#[path = "params.rs"]
mod params;
#[path = "resolved_sun.rs"]
mod resolved_sun;
#[path = "sun_resolution.rs"]
mod sun_resolution;

pub use params::{ProceduralSkyParams, PROCEDURAL_SKY_DEFAULT_SOURCE_REVISION};

use constants::{
    PROCEDURAL_SKY_DEFAULT_SUN_ANGULAR_RADIUS_RADIANS,
    PROCEDURAL_SKY_MAX_SUN_ANGULAR_RADIUS_RADIANS, PROCEDURAL_SKY_MIN_SUN_ANGULAR_RADIUS_RADIANS,
    PROCEDURAL_SKY_MIN_SUN_DIRECTION_LENGTH_SQUARED, PROCEDURAL_SKY_SUN_INNER_RADIUS_SCALE,
};
pub(crate) use resolved_sun::ResolvedProceduralSun;
